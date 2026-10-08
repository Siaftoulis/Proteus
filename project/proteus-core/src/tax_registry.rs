//! Official Tax Registry & VIES AFM Validation Engine for Proteus BOS.
//! Implements 100% original, native tax registry verification:
//! - Mathematical Modulo-11 checksum validation for Greek 9-digit AFMs.
//! - Direct lookup via official European Commission VIES REST API (ec.europa.eu).
//! - Local persistent SQLite cache (`tax_registry_cache`) for offline resilience.
//! - Strict Rule 1 (100% Original Codebase) and Rule 5 (Zero Mock Data).

use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

/// Detailed business registration profile retrieved from official tax registries.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaxRegistryRecord {
    pub afm: String,
    pub country_code: String,
    pub legal_name: String,
    pub commercial_title: Option<String>,
    pub address: String,
    pub postal_code: String,
    pub city: String,
    pub doy: Option<String>,
    pub is_active: bool,
    pub is_normal_vat: bool,
    pub cached_at: i64,
}

/// Computes whether a Greek AFM string satisfies the statutory Modulo-11 checksum algorithm.
pub fn validate_greek_afm_checksum(afm: &str) -> bool {
    let clean = afm.trim();
    if clean.len() != 9 || !clean.chars().all(|c| c.is_ascii_digit()) || clean == "000000000" {
        return false;
    }

    let digits: Vec<u32> = clean.chars().map(|c| c.to_digit(10).unwrap()).collect();

    // The first 8 digits are weighted by 2^(8-i)
    let mut sum: u32 = 0;
    for (i, &digit) in digits.iter().take(8).enumerate() {
        let weight = 1 << (8 - i);
        sum += digit * weight;
    }

    let remainder = sum % 11;
    let expected_check = remainder % 10;

    digits[8] == expected_check
}

/// Response payload schema from the European Commission VIES REST API.
#[derive(Debug, Deserialize)]
struct ViesRestResponse {
    #[serde(rename = "isValid")]
    is_valid: bool,
    name: Option<String>,
    address: Option<String>,
}

/// Initializes the local tax registry cache table in SQLite.
pub fn init_tax_registry_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS tax_registry_cache (
            afm TEXT PRIMARY KEY,
            country_code TEXT NOT NULL,
            legal_name TEXT NOT NULL,
            commercial_title TEXT,
            address TEXT NOT NULL,
            postal_code TEXT NOT NULL,
            city TEXT NOT NULL,
            doy TEXT,
            is_active INTEGER NOT NULL DEFAULT 1,
            is_normal_vat INTEGER NOT NULL DEFAULT 1,
            cached_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_tax_registry_name ON tax_registry_cache(legal_name);
        "#,
    )?;
    Ok(())
}

/// Retrieves a cached tax record from SQLite if available.
pub fn get_cached_tax_record(conn: &Connection, afm: &str) -> Result<Option<TaxRegistryRecord>> {
    let clean = afm.trim();
    let mut stmt = conn.prepare(
        r#"
        SELECT afm, country_code, legal_name, commercial_title, address,
               postal_code, city, doy, is_active, is_normal_vat, cached_at
        FROM tax_registry_cache
        WHERE afm = ?1
        "#,
    )?;

    let mut rows = stmt.query(params![clean])?;
    if let Some(row) = rows.next()? {
        let is_act: i64 = row.get(8)?;
        let is_vat: i64 = row.get(9)?;
        Ok(Some(TaxRegistryRecord {
            afm: row.get(0)?,
            country_code: row.get(1)?,
            legal_name: row.get(2)?,
            commercial_title: row.get(3)?,
            address: row.get(4)?,
            postal_code: row.get(5)?,
            city: row.get(6)?,
            doy: row.get(7)?,
            is_active: is_act != 0,
            is_normal_vat: is_vat != 0,
            cached_at: row.get(10)?,
        }))
    } else {
        Ok(None)
    }
}

/// Persists or updates a tax registry record in the local cache.
pub fn save_tax_record(conn: &Connection, record: &TaxRegistryRecord) -> Result<()> {
    conn.execute(
        r#"
        INSERT INTO tax_registry_cache (
            afm, country_code, legal_name, commercial_title, address,
            postal_code, city, doy, is_active, is_normal_vat, cached_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
        ON CONFLICT(afm) DO UPDATE SET
            legal_name = excluded.legal_name,
            commercial_title = excluded.commercial_title,
            address = excluded.address,
            postal_code = excluded.postal_code,
            city = excluded.city,
            doy = excluded.doy,
            is_active = excluded.is_active,
            is_normal_vat = excluded.is_normal_vat,
            cached_at = excluded.cached_at
        "#,
        params![
            record.afm,
            record.country_code,
            record.legal_name,
            record.commercial_title,
            record.address,
            record.postal_code,
            record.city,
            record.doy,
            if record.is_active { 1 } else { 0 },
            if record.is_normal_vat { 1 } else { 0 },
            record.cached_at,
        ],
    )?;
    Ok(())
}

/// Performs a live online lookup via the European Commission VIES REST API.
pub fn lookup_vies_online(country_code: &str, vat_number: &str) -> std::result::Result<Option<TaxRegistryRecord>, String> {
    let clean_country = country_code.trim().to_uppercase();
    let clean_vat = vat_number.trim();

    // European Commission VIES member state code for Greece in VAT numbers is "EL"
    let endpoint_country = if clean_country == "GR" { "EL" } else { &clean_country };

    let url = format!(
        "https://ec.europa.eu/taxation_customs/vies/rest-api/ms/{}/vat/{}",
        endpoint_country, clean_vat
    );

    let resp = ureq::get(&url)
        .timeout(std::time::Duration::from_secs(5))
        .call()
        .map_err(|e| format!("VIES network error: {}", e))?;

    if resp.status() == 200 {
        let parsed: ViesRestResponse = resp.into_json().map_err(|e| format!("VIES JSON parse error: {}", e))?;
        if parsed.is_valid {
            let full_address = parsed.address.unwrap_or_default();
            let now = chrono::Utc::now().timestamp_millis();
            return Ok(Some(TaxRegistryRecord {
                afm: clean_vat.to_string(),
                country_code: clean_country,
                legal_name: parsed.name.unwrap_or_else(|| "Άγνωστη Επωνυμία".to_string()).trim().to_string(),
                commercial_title: None,
                address: full_address.trim().to_string(),
                postal_code: String::new(),
                city: String::new(),
                doy: None,
                is_active: true,
                is_normal_vat: true,
                cached_at: now,
            }));
        }
    }

    Ok(None)
}

/// Unified resolver: Checks local SQLite cache first; if absent, attempts online VIES lookup.
pub fn resolve_afm(
    conn: &Connection,
    country_code: &str,
    afm: &str,
) -> std::result::Result<Option<TaxRegistryRecord>, String> {
    let clean_afm = afm.trim();
    if country_code.eq_ignore_ascii_case("GR") && !validate_greek_afm_checksum(clean_afm) {
        return Err(format!("Μη έγκυρο ΑΦΜ: Αποτυχία αλγορίθμου ελέγχου Modulo-11 ({})", clean_afm));
    }

    if let Ok(Some(cached)) = get_cached_tax_record(conn, clean_afm) {
        return Ok(Some(cached));
    }

    // Attempt online lookup and cache result if successful
    if let Ok(Some(live_record)) = lookup_vies_online(country_code, clean_afm) {
        let _ = save_tax_record(conn, &live_record);
        return Ok(Some(live_record));
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_greek_afm_checksum() {
        // Valid Greek AFMs with correct modulo-11 checksums
        assert!(validate_greek_afm_checksum("094014201")); // Athens Chamber of Commerce (EVEA)
        assert!(validate_greek_afm_checksum("090000045")); // Ministry of Finance
        assert!(validate_greek_afm_checksum("802194518")); // Proteus valid AFM
    }

    #[test]
    fn test_invalid_greek_afm_checksum() {
        // Corrupted 9th digit
        assert!(!validate_greek_afm_checksum("094014202"));
        assert!(!validate_greek_afm_checksum("123456789"));
        assert!(!validate_greek_afm_checksum("000000000"));
        // Wrong length
        assert!(!validate_greek_afm_checksum("80219451"));
        assert!(!validate_greek_afm_checksum("8021945180"));
        // Non-digits
        assert!(!validate_greek_afm_checksum("80219451A"));
    }

    #[test]
    fn test_tax_registry_cache_lifecycle() {
        let conn = Connection::open_in_memory().unwrap();
        init_tax_registry_schema(&conn).unwrap();

        let record = TaxRegistryRecord {
            afm: "094014201".to_string(),
            country_code: "GR".to_string(),
            legal_name: "Athens Chamber of Commerce & Industry".to_string(),
            commercial_title: Some("EVEA".to_string()),
            address: "Akadimias 7".to_string(),
            postal_code: "10671".to_string(),
            city: "Athens".to_string(),
            doy: Some("D' Athinon".to_string()),
            is_active: true,
            is_normal_vat: true,
            cached_at: 1710000000,
        };

        save_tax_record(&conn, &record).unwrap();

        let retrieved = get_cached_tax_record(&conn, "094014201").unwrap().unwrap();
        assert_eq!(retrieved.legal_name, "Athens Chamber of Commerce & Industry");
        assert_eq!(retrieved.doy, Some("D' Athinon".to_string()));
        assert!(retrieved.is_active);

        // Resolve checks cache first
        let resolved = resolve_afm(&conn, "GR", "094014201").unwrap().unwrap();
        assert_eq!(resolved.afm, "094014201");
        assert_eq!(resolved.city, "Athens");
    }

    #[test]
    fn test_resolve_invalid_afm_returns_err() {
        let conn = Connection::open_in_memory().unwrap();
        init_tax_registry_schema(&conn).unwrap();

        let err = resolve_afm(&conn, "GR", "123456789").unwrap_err();
        assert!(err.contains("Μη έγκυρο ΑΦΜ"));
    }
}
