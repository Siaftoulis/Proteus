//! Metrological Certification Logging & A.1155 Legal Interlock Engine.
//! Records cryptographic audit trails of commercial weighed goods in SQLite,
//! verifying metrological stability (OIML R76 / A.1155) and authentic tare deduction
//! before permitting fiscal invoice and retail receipt finalization.
//! Strict Rule 1 (100% Original Codebase), Rule 5 (Zero Mock Data), Rule 6 (Compliance & Security).

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;

use crate::hardware::scales::bridge::WeighedCartItem;

/// Errors arising from metrological certification and legal fiscal interlocks.
#[derive(Debug, Error, PartialEq)]
pub enum ScaleMetrologyError {
    #[error("Ακυρώθηκε η έκδοση (Α.1155): Ζυγιζόμενο προϊόν '{0}' χωρίς πιστοποιημένη σταθερότητα ζυγαριάς.")]
    UncertifiedScaleStability(String),
    #[error("Μη έγκυρο απόβαρο (Tare: {0} g, Μικτό: {1} g). Το απόβαρο δεν μπορεί να υπερβαίνει το μικτό φορτίο.")]
    InvalidTareWeight(i64, i64),
    #[error("Μηδενικό καθαρό βάρος μετά την αφαίρεση απόβαρου (Net: {0} g).")]
    ZeroNetWeight(i64),
    #[error("Σφάλμα βάσης SQLite μετρολογίας: {0}")]
    DatabaseError(String),
}

/// Tamper-evident metrology record stored in SQLite for tax and weights & measures audits.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScaleMetrologyRecord {
    pub log_id: String,
    pub receipt_number: Option<i64>,
    pub product_name: String,
    pub gross_weight_grams: i64,
    pub tare_weight_grams: i64,
    pub net_weight_grams: i64,
    pub unit_price_per_kg: f64,
    pub total_eur: f64,
    pub protocol_vendor: String,
    pub stability_certified: bool,
    pub tamper_hash: String,
    pub created_at: i64,
}

/// Initializes the statutory metrological audit trail schema in SQLite.
pub fn init_scale_metrology_schema(conn: &Connection) -> Result<(), ScaleMetrologyError> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS scale_metrology_logs (
            log_id TEXT PRIMARY KEY,
            receipt_number INTEGER,
            product_name TEXT NOT NULL,
            gross_weight_grams INTEGER NOT NULL,
            tare_weight_grams INTEGER NOT NULL,
            net_weight_grams INTEGER NOT NULL,
            unit_price_per_kg REAL NOT NULL,
            total_eur REAL NOT NULL,
            protocol_vendor TEXT NOT NULL,
            stability_certified INTEGER NOT NULL,
            tamper_hash TEXT NOT NULL,
            created_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_scale_metrology_receipt ON scale_metrology_logs(receipt_number);
        CREATE INDEX IF NOT EXISTS idx_scale_metrology_created ON scale_metrology_logs(created_at);
        "#,
    )
    .map_err(|e| ScaleMetrologyError::DatabaseError(e.to_string()))?;

    Ok(())
}

/// Computes authentic net weight by deducting verified tare from gross weight.
pub fn compute_tare_deduction(gross_grams: i64, tare_grams: i64) -> Result<i64, ScaleMetrologyError> {
    if tare_grams < 0 || tare_grams >= gross_grams {
        return Err(ScaleMetrologyError::InvalidTareWeight(tare_grams, gross_grams));
    }

    let net_grams = gross_grams - tare_grams;
    if net_grams <= 0 {
        return Err(ScaleMetrologyError::ZeroNetWeight(net_grams));
    }

    Ok(net_grams)
}

/// Generates a SHA-256 tamper-evident digital seal for a metrology measurement.
pub fn compute_metrology_tamper_hash(
    log_id: &str,
    product_name: &str,
    gross_grams: i64,
    tare_grams: i64,
    net_grams: i64,
    unit_price_per_kg: f64,
    created_at: i64,
) -> String {
    let mut hasher = Sha256::new();
    let data = format!(
        "{}:{}:{}:{}:{}:{:.4}:{}",
        log_id, product_name, gross_grams, tare_grams, net_grams, unit_price_per_kg, created_at
    );
    hasher.update(data.as_bytes());
    let result = hasher.finalize();
    let mut hex = String::with_capacity(64);
    for byte in result {
        use std::fmt::Write;
        let _ = write!(&mut hex, "{:02x}", byte);
    }
    hex
}

/// Verifies whether a weighed item conforms to legal A.1155 stability and tare interlocks.
pub fn verify_weighed_item_interlock(item: &WeighedCartItem) -> Result<(), ScaleMetrologyError> {
    if !item.is_certified_stable {
        return Err(ScaleMetrologyError::UncertifiedScaleStability(
            item.description.clone(),
        ));
    }

    if item.tare_weight_grams < 0 || item.net_weight_grams <= 0 {
        return Err(ScaleMetrologyError::InvalidTareWeight(
            item.tare_weight_grams,
            item.net_weight_grams + item.tare_weight_grams,
        ));
    }

    Ok(())
}

/// Logs a certified weighed transaction into the SQLite metrological journal.
pub fn log_scale_metrology_record(
    conn: &Connection,
    record: &ScaleMetrologyRecord,
) -> Result<(), ScaleMetrologyError> {
    conn.execute(
        r#"
        INSERT INTO scale_metrology_logs (
            log_id, receipt_number, product_name, gross_weight_grams,
            tare_weight_grams, net_weight_grams, unit_price_per_kg,
            total_eur, protocol_vendor, stability_certified, tamper_hash, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
        "#,
        params![
            record.log_id,
            record.receipt_number,
            record.product_name,
            record.gross_weight_grams,
            record.tare_weight_grams,
            record.net_weight_grams,
            record.unit_price_per_kg,
            record.total_eur,
            record.protocol_vendor,
            if record.stability_certified { 1 } else { 0 },
            record.tamper_hash,
            record.created_at,
        ],
    )
    .map_err(|e| ScaleMetrologyError::DatabaseError(e.to_string()))?;

    Ok(())
}

/// Creates and records a metrological log entry directly from a WeighedCartItem.
pub fn record_weighed_cart_item(
    conn: &Connection,
    item: &WeighedCartItem,
    receipt_number: Option<i64>,
    protocol_vendor: &str,
) -> Result<ScaleMetrologyRecord, ScaleMetrologyError> {
    verify_weighed_item_interlock(item)?;

    let log_id = Uuid::now_v7().to_string();
    let gross_grams = item.net_weight_grams + item.tare_weight_grams;
    let created_at = chrono::Utc::now().timestamp_millis();

    let tamper_hash = compute_metrology_tamper_hash(
        &log_id,
        &item.description,
        gross_grams,
        item.tare_weight_grams,
        item.net_weight_grams,
        item.unit_price_per_kg,
        created_at,
    );

    let record = ScaleMetrologyRecord {
        log_id,
        receipt_number,
        product_name: item.description.clone(),
        gross_weight_grams: gross_grams,
        tare_weight_grams: item.tare_weight_grams,
        net_weight_grams: item.net_weight_grams,
        unit_price_per_kg: item.unit_price_per_kg,
        total_eur: item.line_total_eur,
        protocol_vendor: protocol_vendor.to_string(),
        stability_certified: item.is_certified_stable,
        tamper_hash,
        created_at,
    };

    log_scale_metrology_record(conn, &record)?;
    Ok(record)
}

/// Retrieves the most recent metrological audit logs from SQLite.
pub fn list_scale_metrology_logs(
    conn: &Connection,
    limit: usize,
) -> Result<Vec<ScaleMetrologyRecord>, ScaleMetrologyError> {
    let mut stmt = conn
        .prepare(
            r#"
            SELECT log_id, receipt_number, product_name, gross_weight_grams,
                   tare_weight_grams, net_weight_grams, unit_price_per_kg,
                   total_eur, protocol_vendor, stability_certified, tamper_hash, created_at
            FROM scale_metrology_logs
            ORDER BY created_at DESC
            LIMIT ?1
            "#,
        )
        .map_err(|e| ScaleMetrologyError::DatabaseError(e.to_string()))?;

    let rows = stmt
        .query_map(params![limit as i64], |row| {
            let cert_int: i64 = row.get(9)?;
            Ok(ScaleMetrologyRecord {
                log_id: row.get(0)?,
                receipt_number: row.get(1)?,
                product_name: row.get(2)?,
                gross_weight_grams: row.get(3)?,
                tare_weight_grams: row.get(4)?,
                net_weight_grams: row.get(5)?,
                unit_price_per_kg: row.get(6)?,
                total_eur: row.get(7)?,
                protocol_vendor: row.get(8)?,
                stability_certified: cert_int == 1,
                tamper_hash: row.get(10)?,
                created_at: row.get(11)?,
            })
        })
        .map_err(|e| ScaleMetrologyError::DatabaseError(e.to_string()))?;

    let mut logs = Vec::new();
    for row in rows {
        logs.push(row.map_err(|e| ScaleMetrologyError::DatabaseError(e.to_string()))?);
    }
    Ok(logs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_tare_deduction_cases() {
        // Valid deduction
        assert_eq!(compute_tare_deduction(1500, 100).unwrap(), 1400);

        // Invalid: tare >= gross
        assert_eq!(
            compute_tare_deduction(500, 500),
            Err(ScaleMetrologyError::InvalidTareWeight(500, 500))
        );
        assert_eq!(
            compute_tare_deduction(500, 600),
            Err(ScaleMetrologyError::InvalidTareWeight(600, 500))
        );

        // Invalid: negative tare
        assert_eq!(
            compute_tare_deduction(1000, -10),
            Err(ScaleMetrologyError::InvalidTareWeight(-10, 1000))
        );
    }

    #[test]
    fn test_verify_weighed_item_interlock_rejections() {
        let stable_item = WeighedCartItem {
            description: "Ελληνικά Μήλα".to_string(),
            quantity_kg: 1.5,
            unit_price_per_kg: 2.0,
            line_total_eur: 3.0,
            net_weight_grams: 1500,
            tare_weight_grams: 50,
            is_certified_stable: true,
        };
        assert!(verify_weighed_item_interlock(&stable_item).is_ok());

        let mut unstable_item = stable_item.clone();
        unstable_item.is_certified_stable = false;
        assert_eq!(
            verify_weighed_item_interlock(&unstable_item),
            Err(ScaleMetrologyError::UncertifiedScaleStability(
                "Ελληνικά Μήλα".to_string()
            ))
        );

        let mut zero_weight_item = stable_item.clone();
        zero_weight_item.net_weight_grams = 0;
        assert_eq!(
            verify_weighed_item_interlock(&zero_weight_item),
            Err(ScaleMetrologyError::InvalidTareWeight(50, 50))
        );
    }

    #[test]
    fn test_scale_metrology_sqlite_journal_and_tamper_hash() {
        let conn = Connection::open_in_memory().unwrap();
        init_scale_metrology_schema(&conn).unwrap();

        let item = WeighedCartItem {
            description: "Φέτα ΠΟΠ".to_string(),
            quantity_kg: 0.850,
            unit_price_per_kg: 12.0,
            line_total_eur: 10.20,
            net_weight_grams: 850,
            tare_weight_grams: 30,
            is_certified_stable: true,
        };

        let record = record_weighed_cart_item(&conn, &item, Some(1042), "Mettler-Toledo MT-SICS").unwrap();
        assert_eq!(record.gross_weight_grams, 880);
        assert_eq!(record.net_weight_grams, 850);
        assert_eq!(record.tare_weight_grams, 30);
        assert_eq!(record.receipt_number, Some(1042));
        assert_eq!(record.tamper_hash.len(), 64);

        let logs = list_scale_metrology_logs(&conn, 10).unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].log_id, record.log_id);
        assert_eq!(logs[0].product_name, "Φέτα ΠΟΠ");
        assert_eq!(logs[0].tamper_hash, record.tamper_hash);
        assert!(logs[0].stability_certified);
    }
}
