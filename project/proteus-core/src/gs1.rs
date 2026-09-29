//! GS1-128 Barcode & Application Identifier (AI) Parser for Proteus BOS.
//! Decodes standardized logistics and inventory barcodes (GTIN, Batch, Expiry, Serial, SSCC).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Gs1BarcodeData {
    pub raw_barcode: String,
    pub gtin: Option<String>,          // AI 01: Global Trade Item Number (14 digits)
    pub batch_lot: Option<String>,     // AI 10: Batch or Lot number
    pub expiry_date: Option<String>,    // AI 17: Expiration Date (YYYY-MM-DD)
    pub serial_number: Option<String>,  // AI 21: Serial Number
    pub sscc: Option<String>,           // AI 00: Serial Shipping Container Code (18 digits)
    pub additional_attributes: HashMap<String, String>,
}

pub struct Gs1Parser;

impl Gs1Parser {
    /// Parses a GS1 string containing Application Identifiers.
    /// Supports parenthesized format: `(01)08412345678905(10)LOT99(17)261231`
    /// as well as raw concatenated strings starting with standard fixed-length AIs.
    pub fn parse(input: &str) -> Gs1BarcodeData {
        let trimmed = input.trim();
        let mut data = Gs1BarcodeData {
            raw_barcode: trimmed.to_string(),
            gtin: None,
            batch_lot: None,
            expiry_date: None,
            serial_number: None,
            sscc: None,
            additional_attributes: HashMap::new(),
        };

        if trimmed.contains('(') && trimmed.contains(')') {
            Self::parse_parenthesized(trimmed, &mut data);
        } else {
            Self::parse_concatenated(trimmed, &mut data);
        }

        data
    }

    fn parse_parenthesized(input: &str, data: &mut Gs1BarcodeData) {
        let mut rest = input;
        while let Some(start) = rest.find('(') {
            let after_start = &rest[start + 1..];
            if let Some(end) = after_start.find(')') {
                let ai = &after_start[..end];
                let value_start = after_start[end + 1..].trim();
                let next_paren = value_start.find('(').unwrap_or(value_start.len());
                let value = value_start[..next_paren].trim();

                Self::assign_ai(ai, value, data);
                rest = &value_start[next_paren..];
            } else {
                break;
            }
        }
    }

    fn parse_concatenated(input: &str, data: &mut Gs1BarcodeData) {
        let mut rest = input;

        while !rest.is_empty() {
            if rest.starts_with("01") && rest.len() >= 16 {
                // AI 01: GTIN-14 is exactly 14 digits
                let gtin_val = &rest[2..16];
                Self::assign_ai("01", gtin_val, data);
                rest = &rest[16..];
            } else if rest.starts_with("00") && rest.len() >= 20 {
                // AI 00: SSCC is exactly 18 digits
                let sscc_val = &rest[2..20];
                Self::assign_ai("00", sscc_val, data);
                rest = &rest[20..];
            } else if rest.starts_with("17") && rest.len() >= 8 {
                // AI 17: Expiry date is YYMMDD (6 digits)
                let expiry_val = &rest[2..8];
                Self::assign_ai("17", expiry_val, data);
                rest = &rest[8..];
            } else if let Some(lot_val) = rest.strip_prefix("10") {
                // AI 10: Batch/Lot is variable length (consume remainder if no delimiter)
                Self::assign_ai("10", lot_val, data);
                break;
            } else if let Some(serial_val) = rest.strip_prefix("21") {
                // AI 21: Serial number
                Self::assign_ai("21", serial_val, data);
                break;
            } else {
                // Unrecognized prefix or unsupported variable length AI
                data.additional_attributes.insert("unparsed_tail".to_string(), rest.to_string());
                break;
            }
        }
    }

    fn assign_ai(ai: &str, value: &str, data: &mut Gs1BarcodeData) {
        match ai {
            "01" => data.gtin = Some(value.to_string()),
            "10" => data.batch_lot = Some(value.to_string()),
            "17" => data.expiry_date = Some(format_gs1_date(value)),
            "21" => data.serial_number = Some(value.to_string()),
            "00" => data.sscc = Some(value.to_string()),
            other => {
                data.additional_attributes.insert(other.to_string(), value.to_string());
            }
        }
    }
}

/// Converts YYMMDD date string to ISO YYYY-MM-DD format.
fn format_gs1_date(yymmdd: &str) -> String {
    if yymmdd.len() != 6 || !yymmdd.chars().all(|c| c.is_ascii_digit()) {
        return yymmdd.to_string();
    }

    let yy: u32 = yymmdd[0..2].parse().unwrap_or(0);
    let mm = &yymmdd[2..4];
    let dd = &yymmdd[4..6];

    // GS1 rule: YY 51-99 is 1951-1999, 00-50 is 2000-2050
    let year = if yy >= 51 { 1900 + yy } else { 2000 + yy };
    format!("{}-{}-{}", year, mm, dd)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_parenthesized_gs1() {
        let code = "(01)08412345678905(10)LOT-2026-A(17)261231(21)SN987654";
        let parsed = Gs1Parser::parse(code);

        assert_eq!(parsed.gtin.as_deref(), Some("08412345678905"));
        assert_eq!(parsed.batch_lot.as_deref(), Some("LOT-2026-A"));
        assert_eq!(parsed.expiry_date.as_deref(), Some("2026-12-31"));
        assert_eq!(parsed.serial_number.as_deref(), Some("SN987654"));
        assert!(parsed.sscc.is_none());
    }

    #[test]
    fn test_parse_concatenated_fixed_ai() {
        // AI 01 (14 chars) + AI 17 (6 chars) + AI 10 (batch remainder)
        let code = "01084123456789051726081510BATCH_ALPHA";
        let parsed = Gs1Parser::parse(code);

        assert_eq!(parsed.gtin.as_deref(), Some("08412345678905"));
        assert_eq!(parsed.expiry_date.as_deref(), Some("2026-08-15"));
        assert_eq!(parsed.batch_lot.as_deref(), Some("BATCH_ALPHA"));
    }

    #[test]
    fn test_sscc_container_code() {
        let code = "(00)108412345678901234";
        let parsed = Gs1Parser::parse(code);
        assert_eq!(parsed.sscc.as_deref(), Some("108412345678901234"));
    }
}
