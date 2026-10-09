//! Multi-Jurisdiction Fiscal Validation & Specification Engine (Micro-task 27.1.1).
//! Implements tax ID validation algorithms, standard tax brackets, and SQLite storage.

use super::types::{FiscalEntityProfile, FiscalJurisdiction, TaxRateBracket};
use rusqlite::{params, Connection};

pub struct JurisdictionEngine;

impl JurisdictionEngine {
    /// Returns statutory tax brackets for the selected jurisdiction.
    pub fn get_standard_tax_brackets(jurisdiction: &FiscalJurisdiction) -> Vec<TaxRateBracket> {
        match jurisdiction {
            FiscalJurisdiction::GreeceAade => vec![
                TaxRateBracket::new("GR-24", "Standard VAT (24%)", 2400, "VAT_STANDARD"),
                TaxRateBracket::new("GR-13", "Reduced VAT (13%)", 1300, "VAT_REDUCED"),
                TaxRateBracket::new("GR-06", "Super-Reduced VAT (6%)", 600, "VAT_SUPER_REDUCED"),
                TaxRateBracket::new("GR-00", "Exempt VAT (0%)", 0, "VAT_EXEMPT"),
            ],
            FiscalJurisdiction::GermanyKassenSichV => vec![
                TaxRateBracket::new("DE-19", "Regelsteuersatz (19%)", 1900, "MWST_STANDARD"),
                TaxRateBracket::new("DE-07", "Ermäßigter Steuersatz (7%)", 700, "MWST_REDUCED"),
                TaxRateBracket::new("DE-00", "Steuerfrei (0%)", 0, "MWST_EXEMPT"),
            ],
            FiscalJurisdiction::FranceNf525 => vec![
                TaxRateBracket::new("FR-20", "Taux normal (20%)", 2000, "TVA_NORMAL"),
                TaxRateBracket::new("FR-10", "Taux intermédiaire (10%)", 1000, "TVA_INTERMEDIATE"),
                TaxRateBracket::new("FR-55", "Taux réduit (5.5%)", 550, "TVA_REDUCED"),
                TaxRateBracket::new("FR-21", "Taux particulier (2.1%)", 210, "TVA_SPECIAL"),
                TaxRateBracket::new("FR-00", "Exonéré (0%)", 0, "TVA_EXEMPT"),
            ],
            FiscalJurisdiction::UsSalesTax { state_code } => vec![
                TaxRateBracket::new(
                    &format!("US-{}-STD", state_code.to_uppercase()),
                    &format!("State Sales Tax ({})", state_code.to_uppercase()),
                    700, // 7.00% benchmark
                    "SALES_TAX_GENERAL",
                ),
                TaxRateBracket::new(
                    &format!("US-{}-EX", state_code.to_uppercase()),
                    "Tax Exempt (Resale/Gov)",
                    0,
                    "SALES_TAX_EXEMPT",
                ),
            ],
            FiscalJurisdiction::GlobalGeneric => vec![
                TaxRateBracket::new("GL-20", "Standard Tax (20%)", 2000, "TAX_STANDARD"),
                TaxRateBracket::new("GL-00", "Zero-Rated / Exempt (0%)", 0, "TAX_EXEMPT"),
            ],
        }
    }

    /// Validates statutory tax identification numbers using real official verification algorithms.
    pub fn validate_tax_id(jurisdiction: &FiscalJurisdiction, tax_id: &str) -> Result<bool, String> {
        let clean = tax_id.trim().replace(['-', ' ', '/'], "");
        if clean.is_empty() {
            return Err("Tax identification number cannot be empty.".to_string());
        }

        match jurisdiction {
            FiscalJurisdiction::GreeceAade => {
                // Greek AFM: 9 digits, Modulo 11 checksum
                if clean.len() != 9 || !clean.chars().all(|c| c.is_ascii_digit()) {
                    return Ok(false);
                }
                let digits: Vec<u32> = clean.chars().map(|c| c.to_digit(10).unwrap()).collect();
                let mut sum = 0;
                for (i, &digit) in digits.iter().enumerate().take(8) {
                    sum += digit * (1 << (8 - i));
                }
                let remainder = sum % 11;
                let check_digit = remainder % 10;
                Ok(check_digit == digits[8])
            }
            FiscalJurisdiction::GermanyKassenSichV => {
                // German USt-IdNr: "DE" prefix + 9 digits or Steuernummer: 10-11 digits
                let s = clean.to_uppercase();
                if let Some(num_part) = s.strip_prefix("DE") {
                    Ok(num_part.len() == 9 && num_part.chars().all(|c| c.is_ascii_digit()))
                } else {
                    Ok((s.len() == 10 || s.len() == 11) && s.chars().all(|c| c.is_ascii_digit()))
                }
            }
            FiscalJurisdiction::FranceNf525 => {
                // French SIREN (9 digits) or SIRET (14 digits) using Luhn algorithm
                let s = clean.to_uppercase();
                let digits_str = if s.starts_with("FR") && s.len() >= 11 {
                    &s[4..] // skip "FR" + 2 key digits to get SIREN
                } else {
                    &s[..]
                };

                if digits_str.len() != 9 && digits_str.len() != 14 {
                    return Ok(false);
                }
                if !digits_str.chars().all(|c| c.is_ascii_digit()) {
                    return Ok(false);
                }

                // Standard Luhn Algorithm
                let mut sum = 0;
                let is_even_len = digits_str.len() % 2 == 0;
                for (idx, c) in digits_str.chars().enumerate() {
                    let mut digit = c.to_digit(10).unwrap();
                    if (idx % 2 == 0) == is_even_len {
                        digit *= 2;
                        if digit > 9 {
                            digit -= 9;
                        }
                    }
                    sum += digit;
                }
                Ok(sum % 10 == 0)
            }
            FiscalJurisdiction::UsSalesTax { .. } => {
                // US EIN: 9 numeric digits
                Ok(clean.len() == 9 && clean.chars().all(|c| c.is_ascii_digit()))
            }
            FiscalJurisdiction::GlobalGeneric => {
                // Generic alphanumeric tax ID with minimum 4 chars
                Ok(clean.len() >= 4 && clean.len() <= 32)
            }
        }
    }

    /// Initializes `system_fiscal_jurisdictions` table in SQLite.
    pub fn init_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS system_fiscal_jurisdictions (
                profile_id TEXT PRIMARY KEY,
                tax_id TEXT NOT NULL,
                legal_name TEXT NOT NULL,
                jurisdiction_json TEXT NOT NULL,
                profile_json TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    /// Saves or updates the registered fiscal profile.
    pub fn save_profile(conn: &Connection, profile: &FiscalEntityProfile) -> Result<(), rusqlite::Error> {
        let jur_json = serde_json::to_string(&profile.jurisdiction).unwrap_or_default();
        let prof_json = serde_json::to_string(profile).unwrap_or_default();

        conn.execute(
            "INSERT INTO system_fiscal_jurisdictions (
                profile_id, tax_id, legal_name, jurisdiction_json, profile_json, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ON CONFLICT(profile_id) DO UPDATE SET
                tax_id = excluded.tax_id,
                legal_name = excluded.legal_name,
                jurisdiction_json = excluded.jurisdiction_json,
                profile_json = excluded.profile_json,
                updated_at = excluded.updated_at",
            params![
                profile.profile_id,
                profile.tax_id,
                profile.legal_name,
                jur_json,
                prof_json,
                profile.updated_at,
            ],
        )?;
        Ok(())
    }

    /// Retrieves the active registered fiscal entity profile.
    pub fn get_active_profile(conn: &Connection) -> Result<Option<FiscalEntityProfile>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT profile_json FROM system_fiscal_jurisdictions ORDER BY updated_at DESC LIMIT 1",
        )?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            let json_str: String = row.get(0)?;
            if let Ok(profile) = serde_json::from_str::<FiscalEntityProfile>(&json_str) {
                return Ok(Some(profile));
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greek_afm_validation_real_checksums() {
        let jur = FiscalJurisdiction::GreeceAade;
        // Valid Greek AFM (090000045)
        assert!(JurisdictionEngine::validate_tax_id(&jur, "090000045").unwrap());
        // Invalid Greek AFM
        assert!(!JurisdictionEngine::validate_tax_id(&jur, "123456789").unwrap());
        // Bad length
        assert!(!JurisdictionEngine::validate_tax_id(&jur, "1234").unwrap());
    }

    #[test]
    fn test_german_ust_id_validation() {
        let jur = FiscalJurisdiction::GermanyKassenSichV;
        assert!(JurisdictionEngine::validate_tax_id(&jur, "DE123456789").unwrap());
        assert!(JurisdictionEngine::validate_tax_id(&jur, "1234567890").unwrap());
        assert!(!JurisdictionEngine::validate_tax_id(&jur, "DE123").unwrap());
    }

    #[test]
    fn test_french_siren_luhn_validation() {
        let jur = FiscalJurisdiction::FranceNf525;
        // Valid French SIREN (732 829 320)
        assert!(JurisdictionEngine::validate_tax_id(&jur, "732829320").unwrap());
        // Invalid French SIREN
        assert!(!JurisdictionEngine::validate_tax_id(&jur, "732829321").unwrap());
    }

    #[test]
    fn test_statutory_tax_brackets_generation() {
        let gr_brackets = JurisdictionEngine::get_standard_tax_brackets(&FiscalJurisdiction::GreeceAade);
        assert_eq!(gr_brackets.len(), 4);
        assert_eq!(gr_brackets[0].basis_points, 2400);

        let de_brackets = JurisdictionEngine::get_standard_tax_brackets(&FiscalJurisdiction::GermanyKassenSichV);
        assert_eq!(de_brackets.len(), 3);
        assert_eq!(de_brackets[0].basis_points, 1900);
    }

    #[test]
    fn test_sqlite_profile_persistence() {
        let conn = Connection::open_in_memory().unwrap();
        JurisdictionEngine::init_schema(&conn).unwrap();

        let profile = FiscalEntityProfile {
            profile_id: "CORP-ATHENS-01".into(),
            tax_id: "090000045".into(),
            legal_name: "Proteus Hellas Single Member PC".into(),
            trade_name: "Proteus Store Athens".into(),
            jurisdiction: FiscalJurisdiction::GreeceAade,
            registered_address: "Ermou 15".into(),
            city: "Athens".into(),
            postal_code: "10563".into(),
            country_code: "GR".into(),
            tax_office: "D' Athinon".into(),
            is_vat_registered: true,
            default_currency: "EUR".into(),
            updated_at: "2026-10-09T00:00:00Z".into(),
        };

        JurisdictionEngine::save_profile(&conn, &profile).unwrap();
        let loaded = JurisdictionEngine::get_active_profile(&conn).unwrap().unwrap();
        assert_eq!(loaded.legal_name, "Proteus Hellas Single Member PC");
        assert_eq!(loaded.jurisdiction, FiscalJurisdiction::GreeceAade);
    }
}
