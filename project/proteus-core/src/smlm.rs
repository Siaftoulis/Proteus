//! Proteus SMLM (Small Language Context Model) & Semantic Data Profiler.
//! Fast, offline-first semantic ontology and content-driven field inference engine.
//! Designed from first principles for zero-rewrite database federation.

use serde::{Deserialize, Serialize};

/// High-level business and operational semantic intent for a database column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SemanticIntent {
    RecordId,
    CustomerName,
    TaxId,
    PhoneNumber,
    EmailAddress,
    PostalAddress,
    FinancialAmount,
    BarcodeOrSku,
    MaritimeIdentity,
    AviationIdentity,
    VehicleIdentity,
    InventoryQuantity,
    UnitOfMeasure,
    TimestampOrDate,
    JobWorksite,
    GenericText,
    GenericNumeric,
}

/// Comprehensive semantic profile for an analyzed database column.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColumnProfile {
    pub column_name: String,
    pub inferred_intent: SemanticIntent,
    pub confidence: f64,
    pub sample_matches: usize,
    pub total_samples: usize,
    pub content_detected: bool,
}

/// Content-driven data pattern recognizers running locally without external ML models.
pub struct ContentProfiler;

impl ContentProfiler {
    /// Validates Greek AFM (Tax Identification Number) using official Modulo 11 check.
    pub fn is_greek_afm(value: &str) -> bool {
        let cleaned: String = value.chars().filter(|c| c.is_ascii_digit()).collect();
        if cleaned.len() != 9 {
            return false;
        }
        let digits: Vec<u32> = cleaned.chars().map(|c| c.to_digit(10).unwrap()).collect();
        let mut sum = 0;
        for i in 0..8 {
            sum += digits[i] * (1 << (8 - i));
        }
        let remainder = (sum % 11) % 10;
        remainder == digits[8]
    }

    /// Detects phone numbers (Greek mobile/landline or international E.164).
    pub fn is_phone_number(value: &str) -> bool {
        let trimmed = value.trim();
        let digits: String = trimmed.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() == 10 {
            digits.starts_with("69") || digits.starts_with('2')
        } else if digits.len() == 12 && digits.starts_with("30") {
            digits[2..].starts_with("69") || digits[2..].starts_with('2')
        } else {
            trimmed.starts_with('+') && digits.len() >= 8 && digits.len() <= 15
        }
    }

    /// Checks if a string conforms to basic email syntax.
    pub fn is_email_address(value: &str) -> bool {
        let trimmed = value.trim();
        if let Some(at_idx) = trimmed.find('@') {
            if at_idx > 0 && at_idx < trimmed.len() - 1 {
                let domain = &trimmed[at_idx + 1..];
                return domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.');
            }
        }
        false
    }

    /// Checks if a string is an IBAN (starts with 2 letters, followed by 2 check digits).
    pub fn is_iban(value: &str) -> bool {
        let cleaned: String = value.chars().filter(|c| !c.is_whitespace()).collect();
        if cleaned.len() < 15 || cleaned.len() > 34 {
            return false;
        }
        let bytes = cleaned.as_bytes();
        bytes[0].is_ascii_alphabetic() && bytes[1].is_ascii_alphabetic()
            && bytes[2].is_ascii_digit() && bytes[3].is_ascii_digit()
    }

    /// Detects EAN-13, EAN-8 or GS1 Barcodes with Modulo 10 checksum.
    pub fn is_ean_barcode(value: &str) -> bool {
        let digits: String = value.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() != 8 && digits.len() != 12 && digits.len() != 13 && digits.len() != 14 {
            return false;
        }
        let nums: Vec<u32> = digits.chars().map(|c| c.to_digit(10).unwrap()).collect();
        let len = nums.len();
        let check_digit = nums[len - 1];
        let mut sum = 0;
        for i in 0..(len - 1) {
            let weight = if (len - 1 - i) % 2 == 1 { 3 } else { 1 };
            sum += nums[i] * weight;
        }
        let computed = (10 - (sum % 10)) % 10;
        computed == check_digit
    }

    /// Detects Maritime IMO number (7 digits with weighted sum modulo 10).
    pub fn is_imo_number(value: &str) -> bool {
        let cleaned = value.trim().to_uppercase();
        let digits: String = if cleaned.starts_with("IMO") {
            cleaned[3..].chars().filter(|c| c.is_ascii_digit()).collect()
        } else {
            cleaned.chars().filter(|c| c.is_ascii_digit()).collect()
        };
        if digits.len() != 7 {
            return false;
        }
        let nums: Vec<u32> = digits.chars().map(|c| c.to_digit(10).unwrap()).collect();
        let mut sum = 0;
        for i in 0..6 {
            sum += nums[i] * (7 - i as u32);
        }
        (sum % 10) == nums[6]
    }

    /// Detects monetary / financial amounts (e.g. "120.50", "45,00 €", "1250.00").
    pub fn is_financial_amount(value: &str) -> bool {
        let trimmed = value.trim().trim_end_matches(['€', '$', '£', ' ']);
        let normalized = trimmed.replace(',', ".");
        if let Ok(num) = normalized.parse::<f64>() {
            return num.is_finite() && (normalized.contains('.') || trimmed.contains('€') || trimmed.contains('$'));
        }
        false
    }

    /// Detects ISO/European dates or timestamps.
    pub fn is_timestamp_or_date(value: &str) -> bool {
        let s = value.trim();
        if s.len() >= 10 {
            let b = s.as_bytes();
            // YYYY-MM-DD
            if b[4] == b'-' && b[7] == b'-' {
                return true;
            }
            // DD/MM/YYYY
            if b[2] == b'/' && b[5] == b'/' {
                return true;
            }
        }
        false
    }

    /// Detects units of measure (Greek and English).
    pub fn is_unit_of_measure(value: &str) -> bool {
        let s = value.trim().to_lowercase();
        matches!(
            s.as_str(),
            "τεμ" | "τεμ." | "τεμάχιο" | "τεμαχια" | "pcs" | "pc" | "piece" | "item"
                | "kg" | "κιλά" | "κιλο" | "gr" | "γραμμάρια"
                | "m" | "μέτρα" | "μετρο" | "m2" | "τ.μ." | "m3"
                | "l" | "lt" | "λίτρα" | "lit" | "liters"
                | "κιβ" | "κιβώτιο" | "box" | "pack" | "πακέτο"
        )
    }
}

/// Multi-lingual Semantic Ontology dictionary for normalizing and classifying column headers.
pub struct SemanticDictionary;

impl SemanticDictionary {
    /// Normalizes header strings (lowercase, alphanumeric, stripping accents and prefixes).
    pub fn normalize(header: &str) -> String {
        let lower = header.trim().to_lowercase();
        let cleaned = lower
            .replace(['_', '-', '.', ' ', '/', '\\'], "")
            .replace(['ά', 'α'], "a")
            .replace(['έ', 'ε'], "e")
            .replace(['ή', 'η', 'ί', 'ι', 'ϊ', 'ΐ'], "i")
            .replace(['ό', 'ο', 'ώ', 'ω'], "o")
            .replace(['ύ', 'υ', 'ϋ', 'ΰ'], "y")
            .replace('ς', "s");
        cleaned
    }

    /// Resolves semantic intent from column header name.
    pub fn classify_header(header: &str) -> Option<SemanticIntent> {
        let norm = Self::normalize(header);

        if norm == "id" || norm == "recid" || norm == "recordid" || norm == "rowid" {
            return Some(SemanticIntent::RecordId);
        }
        if norm.contains("afm") || norm.contains("vat") || norm.contains("taxid") || norm.contains("tin") {
            return Some(SemanticIntent::TaxId);
        }
        if norm.contains("onoma") || norm.contains("name") || norm.contains("eponimia") || norm.contains("client") || norm.contains("customer") || norm.contains("company") || norm.contains("corp") || norm.contains("etairia") {
            return Some(SemanticIntent::CustomerName);
        }
        if norm.contains("tilefon") || norm.contains("phone") || norm.contains("tel") || norm.contains("kinito") || norm.contains("mobile") {
            return Some(SemanticIntent::PhoneNumber);
        }
        if norm.contains("email") || norm.contains("mail") {
            return Some(SemanticIntent::EmailAddress);
        }
        if norm.contains("dieythyns") || norm.contains("address") || norm.contains("odos") || norm.contains("street") || norm.contains("poli") || norm.contains("city") || norm.contains("tk") || norm.contains("zip") {
            return Some(SemanticIntent::PostalAddress);
        }
        if norm.contains("timi") || norm.contains("price") || norm.contains("poso") || norm.contains("amount") || norm.contains("total") || norm.contains("ypoloip") || norm.contains("balance") || norm.contains("cost") {
            return Some(SemanticIntent::FinancialAmount);
        }
        if norm.contains("barcode") || norm.contains("ean") || norm.contains("sku") || norm.contains("kwdik") || norm.contains("partno") {
            return Some(SemanticIntent::BarcodeOrSku);
        }
        if norm.contains("imo") || norm.contains("mmsi") || norm.contains("vessel") || norm.contains("ploio") {
            return Some(SemanticIntent::MaritimeIdentity);
        }
        if norm.contains("tailno") || norm.contains("flight") || norm.contains("icao") || norm.contains("aeropl") {
            return Some(SemanticIntent::AviationIdentity);
        }
        if norm.contains("vin") || norm.contains("pinakida") || norm.contains("plate") {
            return Some(SemanticIntent::VehicleIdentity);
        }
        if norm.contains("posot") || norm.contains("qty") || norm.contains("stock") || norm.contains("diathesim") {
            return Some(SemanticIntent::InventoryQuantity);
        }
        if norm.contains("monada") || norm.contains("unit") || norm.contains("uom") {
            return Some(SemanticIntent::UnitOfMeasure);
        }
        if norm.contains("imeromin") || norm.contains("date") || norm.contains("created") || norm.contains("timestamp") {
            return Some(SemanticIntent::TimestampOrDate);
        }
        if norm.contains("ergo") || norm.contains("oikodom") || norm.contains("site") || norm.contains("project") || norm.contains("job") {
            return Some(SemanticIntent::JobWorksite);
        }

        None
    }
}

/// Primary SMLM Analysis Engine combining lexical header ontology with content profiling.
pub struct SmlmEngine;

impl SmlmEngine {
    /// Analyzes a single column based on its name and sample rows.
    pub fn analyze_column(column_name: &str, samples: &[String]) -> ColumnProfile {
        let total_samples = samples.len();
        let header_intent = SemanticDictionary::classify_header(column_name);

        // Content profiling across non-empty samples
        let non_empty: Vec<&str> = samples.iter().map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
        let mut afm_matches = 0;
        let mut phone_matches = 0;
        let mut email_matches = 0;
        let mut barcode_matches = 0;
        let mut imo_matches = 0;
        let mut amount_matches = 0;
        let mut date_matches = 0;
        let mut uom_matches = 0;

        for val in &non_empty {
            if ContentProfiler::is_greek_afm(val) { afm_matches += 1; }
            if ContentProfiler::is_phone_number(val) { phone_matches += 1; }
            if ContentProfiler::is_email_address(val) { email_matches += 1; }
            if ContentProfiler::is_ean_barcode(val) { barcode_matches += 1; }
            if ContentProfiler::is_imo_number(val) { imo_matches += 1; }
            if ContentProfiler::is_financial_amount(val) { amount_matches += 1; }
            if ContentProfiler::is_timestamp_or_date(val) { date_matches += 1; }
            if ContentProfiler::is_unit_of_measure(val) { uom_matches += 1; }
        }

        let non_empty_count = non_empty.len().max(1);

        // Evaluate predominant content match
        let (content_intent, content_hits) = [
            (SemanticIntent::TaxId, afm_matches),
            (SemanticIntent::PhoneNumber, phone_matches),
            (SemanticIntent::EmailAddress, email_matches),
            (SemanticIntent::BarcodeOrSku, barcode_matches),
            (SemanticIntent::MaritimeIdentity, imo_matches),
            (SemanticIntent::FinancialAmount, amount_matches),
            (SemanticIntent::TimestampOrDate, date_matches),
            (SemanticIntent::UnitOfMeasure, uom_matches),
        ]
        .into_iter()
        .max_by_key(|&(_, hits)| hits)
        .unwrap();

        let content_ratio = content_hits as f64 / non_empty_count as f64;

        if content_hits > 0 && content_ratio >= 0.4 {
            let confidence = if let Some(h) = header_intent {
                if h == content_intent { 0.98 } else { 0.85 }
            } else {
                0.80 + (content_ratio * 0.15)
            };

            ColumnProfile {
                column_name: column_name.to_string(),
                inferred_intent: content_intent,
                confidence: confidence.min(1.0),
                sample_matches: content_hits,
                total_samples,
                content_detected: true,
            }
        } else if let Some(h) = header_intent {
            ColumnProfile {
                column_name: column_name.to_string(),
                inferred_intent: h,
                confidence: 0.70,
                sample_matches: 0,
                total_samples,
                content_detected: false,
            }
        } else {
            // Fallback generic classification
            let numeric_count = non_empty.iter().filter(|s| s.parse::<f64>().is_ok()).count();
            let is_predominantly_numeric = numeric_count as f64 / non_empty_count as f64 > 0.7;

            ColumnProfile {
                column_name: column_name.to_string(),
                inferred_intent: if is_predominantly_numeric { SemanticIntent::GenericNumeric } else { SemanticIntent::GenericText },
                confidence: 0.50,
                sample_matches: 0,
                total_samples,
                content_detected: false,
            }
        }
    }

    /// Profiles an entire table schema from its column headers and sample data matrix.
    pub fn profile_table(headers: &[String], rows: &[Vec<String>]) -> Vec<ColumnProfile> {
        let mut profiles = Vec::with_capacity(headers.len());
        for (col_idx, header) in headers.iter().enumerate() {
            let col_samples: Vec<String> = rows
                .iter()
                .filter_map(|row| row.get(col_idx).cloned())
                .collect();
            profiles.push(Self::analyze_column(header, &col_samples));
        }
        profiles
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greek_afm_validation() {
        // Valid Greek AFMs (using known modulo 11 test vectors)
        assert!(ContentProfiler::is_greek_afm("094014201"));
        assert!(ContentProfiler::is_greek_afm("090000045"));
        // Invalid length or checksum
        assert!(!ContentProfiler::is_greek_afm("123456789"));
        assert!(!ContentProfiler::is_greek_afm("12345"));
    }

    #[test]
    fn test_phone_number_detection() {
        assert!(ContentProfiler::is_phone_number("6981234567"));
        assert!(ContentProfiler::is_phone_number("2101234567"));
        assert!(ContentProfiler::is_phone_number("+306981234567"));
        assert!(ContentProfiler::is_phone_number("302101234567"));
        assert!(!ContentProfiler::is_phone_number("12345"));
    }

    #[test]
    fn test_email_and_barcode_detection() {
        assert!(ContentProfiler::is_email_address("dimitris@proteus.gr"));
        assert!(!ContentProfiler::is_email_address("invalid-email"));

        // EAN-13 check digit test: 5901234123457
        assert!(ContentProfiler::is_ean_barcode("5901234123457"));
        assert!(!ContentProfiler::is_ean_barcode("5901234123450"));
    }

    #[test]
    fn test_smlm_content_driven_inference() {
        // Header is anonymous "COL_04", but content contains valid Greek AFMs
        let samples = vec![
            "094014201".to_string(),
            "090000045".to_string(),
            "".to_string(),
        ];
        let profile = SmlmEngine::analyze_column("COL_04", &samples);
        assert_eq!(profile.inferred_intent, SemanticIntent::TaxId);
        assert!(profile.content_detected);
        assert!(profile.confidence >= 0.85);
    }

    #[test]
    fn test_smlm_header_ontology_inference() {
        let profile = SmlmEngine::analyze_column("onoma_pelati", &[]);
        assert_eq!(profile.inferred_intent, SemanticIntent::CustomerName);
        assert_eq!(profile.confidence, 0.70);
    }

    #[test]
    fn test_table_profiling() {
        let headers = vec!["custom_field_1".to_string(), "kinito_tilefono".to_string()];
        let rows = vec![
            vec!["094014201".to_string(), "6981234567".to_string()],
            vec!["090000045".to_string(), "6989999999".to_string()],
        ];
        let profiles = SmlmEngine::profile_table(&headers, &rows);
        assert_eq!(profiles.len(), 2);
        assert_eq!(profiles[0].inferred_intent, SemanticIntent::TaxId);
        assert_eq!(profiles[1].inferred_intent, SemanticIntent::PhoneNumber);
    }
}
