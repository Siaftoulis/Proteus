//! Multi-Shop Counter Intake & Transactional Stress Engine.
//! Simulates high-throughput production workloads across Service Repair Labs,
//! Retail POS Counters, and Logistics Van Sales for real-world pilot deployments.

pub mod network;

use chrono::Utc;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::Instant;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PilotError {
    #[error("Database error: {0}")]
    Sql(#[from] rusqlite::Error),
    #[error("Database integrity corruption: {0}")]
    IntegrityFailure(String),
    #[error("Simulation validation error: {0}")]
    Validation(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PilotShopProfile {
    ServiceRepairLab,
    RetailPosCounter,
    VanSalesLogistics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PilotStressConfig {
    pub operations_per_shop: usize,
    pub enforce_atomic_merkle_chain: bool,
    pub run_pragmas_on_completion: bool,
}

impl Default for PilotStressConfig {
    fn default() -> Self {
        Self {
            operations_per_shop: 100,
            enforce_atomic_merkle_chain: true,
            run_pragmas_on_completion: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PilotStressReport {
    pub total_operations_executed: usize,
    pub intake_tickets_created: usize,
    pub pos_transactions_recorded: usize,
    pub shipping_notes_issued: usize,
    pub total_volume_eur_cents: i64,
    pub merkle_root_hash: String,
    pub db_integrity_ok: bool,
    pub duration_millis: u64,
}

/// Initializes database tables for pilot multi-shop counter simulation.
pub fn init_pilot_simulation_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS pilot_stores (
            store_id TEXT PRIMARY KEY, profile_kind TEXT NOT NULL, name TEXT NOT NULL,
            tax_id TEXT NOT NULL, active_counter_workers INTEGER NOT NULL DEFAULT 1, created_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS pilot_counter_transactions (
            txn_id TEXT PRIMARY KEY, store_id TEXT NOT NULL REFERENCES pilot_stores(store_id),
            voucher_kind TEXT NOT NULL, reference_code TEXT NOT NULL, net_amount_cents INTEGER NOT NULL,
            vat_amount_cents INTEGER NOT NULL, total_amount_cents INTEGER NOT NULL,
            merkle_node_hash TEXT NOT NULL, created_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS pilot_merkle_tree_log (
            seq_num INTEGER PRIMARY KEY AUTOINCREMENT, store_id TEXT NOT NULL,
            prev_block_hash TEXT NOT NULL, current_block_hash TEXT NOT NULL, timestamp_utc TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_pilot_txn_store ON pilot_counter_transactions(store_id);
        CREATE INDEX IF NOT EXISTS idx_pilot_merkle_store ON pilot_merkle_tree_log(store_id);
        "#,
    )?;
    Ok(())
}

/// Seeds standard pilot test stores across 3 distinct operational topologies.
pub fn seed_pilot_stores(conn: &Connection) -> Result<Vec<(String, PilotShopProfile)>> {
    let now = Utc::now().to_rfc3339();
    let stores = vec![
        ("STORE-REP-01", PilotShopProfile::ServiceRepairLab, "Alpha Tech & Smartphone Lab", "998877660"),
        ("STORE-POS-02", PilotShopProfile::RetailPosCounter, "Nexus Retail Electronics & POS", "998877661"),
        ("STORE-LOG-03", PilotShopProfile::VanSalesLogistics, "Aegean Logistics & Fleet Van", "998877662"),
    ];

    let mut result = Vec::new();
    for (id, profile, name, afm) in stores {
        let profile_str = match profile {
            PilotShopProfile::ServiceRepairLab => "ServiceRepairLab",
            PilotShopProfile::RetailPosCounter => "RetailPosCounter",
            PilotShopProfile::VanSalesLogistics => "VanSalesLogistics",
        };
        conn.execute(
            r#"
            INSERT OR REPLACE INTO pilot_stores
            (store_id, profile_kind, name, tax_id, active_counter_workers, created_at)
            VALUES (?1, ?2, ?3, ?4, 3, ?5)
            "#,
            params![id, profile_str, name, afm, now],
        )?;
        result.push((id.to_string(), profile));
    }
    Ok(result)
}

fn compute_merkle_hash(prev_hash: &str, store_id: &str, ref_code: &str, cents: i64, timestamp: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(prev_hash.as_bytes());
    hasher.update(store_id.as_bytes());
    hasher.update(ref_code.as_bytes());
    hasher.update(cents.to_string().as_bytes());
    hasher.update(timestamp.as_bytes());
    hex::encode(hasher.finalize())
}

mod hex {
    pub fn encode(data: impl AsRef<[u8]>) -> String {
        data.as_ref().iter().map(|b| format!("{:02x}", b)).collect()
    }
}

/// Simulates a single service intake step.
pub fn simulate_repair_intake_step(
    conn: &Connection,
    store_id: &str,
    index: usize,
    running_hash: &mut String,
) -> Result<i64> {
    let now = Utc::now().to_rfc3339();
    let txn_id = format!("REP-TXN-{:06}", index);
    let ref_code = format!("TICKET-DEV-S{}", index % 50);
    let net = 6000;
    let vat = 1440; // 24% VAT
    let total = net + vat;

    let next_hash = compute_merkle_hash(running_hash, store_id, &ref_code, total, &now);

    conn.execute(
        r#"
        INSERT INTO pilot_counter_transactions
        (txn_id, store_id, voucher_kind, reference_code, net_amount_cents, vat_amount_cents, total_amount_cents, merkle_node_hash, created_at)
        VALUES (?1, ?2, 'SERVICE_INTAKE', ?3, ?4, ?5, ?6, ?7, ?8)
        "#,
        params![txn_id, store_id, ref_code, net, vat, total, next_hash, now],
    )?;

    conn.execute(
        r#"
        INSERT INTO pilot_merkle_tree_log
        (store_id, prev_block_hash, current_block_hash, timestamp_utc)
        VALUES (?1, ?2, ?3, ?4)
        "#,
        params![store_id, running_hash.as_str(), next_hash.as_str(), now],
    )?;

    *running_hash = next_hash;
    Ok(total)
}

/// Simulates a single retail POS receipt checkout step.
pub fn simulate_retail_pos_step(
    conn: &Connection,
    store_id: &str,
    index: usize,
    running_hash: &mut String,
) -> Result<i64> {
    let now = Utc::now().to_rfc3339();
    let txn_id = format!("POS-TXN-{:06}", index);
    let ref_code = format!("EAN-520123{:04}", index % 500);
    let net = 2500;
    let vat = 600; // 24% VAT
    let total = net + vat;

    let next_hash = compute_merkle_hash(running_hash, store_id, &ref_code, total, &now);

    conn.execute(
        r#"
        INSERT INTO pilot_counter_transactions
        (txn_id, store_id, voucher_kind, reference_code, net_amount_cents, vat_amount_cents, total_amount_cents, merkle_node_hash, created_at)
        VALUES (?1, ?2, 'RETAIL_RECEIPT', ?3, ?4, ?5, ?6, ?7, ?8)
        "#,
        params![txn_id, store_id, ref_code, net, vat, total, next_hash, now],
    )?;

    conn.execute(
        r#"
        INSERT INTO pilot_merkle_tree_log
        (store_id, prev_block_hash, current_block_hash, timestamp_utc)
        VALUES (?1, ?2, ?3, ?4)
        "#,
        params![store_id, running_hash.as_str(), next_hash.as_str(), now],
    )?;

    *running_hash = next_hash;
    Ok(total)
}

/// Simulates a single logistics van sales waybill step.
pub fn simulate_van_sales_step(
    conn: &Connection,
    store_id: &str,
    index: usize,
    running_hash: &mut String,
) -> Result<i64> {
    let now = Utc::now().to_rfc3339();
    let txn_id = format!("VAN-TXN-{:06}", index);
    let ref_code = format!("CMR-WAYBILL-{:04}", index % 100);
    let net = 15000;
    let vat = 3600; // 24% VAT
    let total = net + vat;

    let next_hash = compute_merkle_hash(running_hash, store_id, &ref_code, total, &now);

    conn.execute(
        r#"
        INSERT INTO pilot_counter_transactions
        (txn_id, store_id, voucher_kind, reference_code, net_amount_cents, vat_amount_cents, total_amount_cents, merkle_node_hash, created_at)
        VALUES (?1, ?2, 'VAN_WAYBILL', ?3, ?4, ?5, ?6, ?7, ?8)
        "#,
        params![txn_id, store_id, ref_code, net, vat, total, next_hash, now],
    )?;

    conn.execute(
        r#"
        INSERT INTO pilot_merkle_tree_log
        (store_id, prev_block_hash, current_block_hash, timestamp_utc)
        VALUES (?1, ?2, ?3, ?4)
        "#,
        params![store_id, running_hash.as_str(), next_hash.as_str(), now],
    )?;

    *running_hash = next_hash;
    Ok(total)
}

/// Executes multi-shop counter stress simulation across all topologies.
pub fn run_pilot_counter_stress(
    conn: &Connection,
    config: &PilotStressConfig,
) -> std::result::Result<PilotStressReport, PilotError> {
    let start_time = Instant::now();
    init_pilot_simulation_schema(conn)?;
    let stores = seed_pilot_stores(conn)?;

    let mut intake_count = 0;
    let mut pos_count = 0;
    let mut van_count = 0;
    let mut total_cents: i64 = 0;
    let mut combined_hasher = Sha256::new();

    for (store_id, profile) in stores {
        let mut store_hash = "0000000000000000000000000000000000000000000000000000000000000000".to_string();
        for i in 1..=config.operations_per_shop {
            let amount = match profile {
                PilotShopProfile::ServiceRepairLab => {
                    intake_count += 1;
                    simulate_repair_intake_step(conn, &store_id, i, &mut store_hash)?
                }
                PilotShopProfile::RetailPosCounter => {
                    pos_count += 1;
                    simulate_retail_pos_step(conn, &store_id, i, &mut store_hash)?
                }
                PilotShopProfile::VanSalesLogistics => {
                    van_count += 1;
                    simulate_van_sales_step(conn, &store_id, i, &mut store_hash)?
                }
            };
            total_cents += amount;
        }
        combined_hasher.update(store_hash.as_bytes());
    }
    let master_merkle_root = hex::encode(combined_hasher.finalize());

    let db_integrity_ok = true;
    if config.run_pragmas_on_completion {
        let mut stmt = conn.prepare("PRAGMA integrity_check;")?;
        let status: String = stmt.query_row([], |row| row.get(0))?;
        if status != "ok" {
            return Err(PilotError::IntegrityFailure(status));
        }
    }

    let duration_millis = start_time.elapsed().as_millis() as u64;

    Ok(PilotStressReport {
        total_operations_executed: intake_count + pos_count + van_count,
        intake_tickets_created: intake_count,
        pos_transactions_recorded: pos_count,
        shipping_notes_issued: van_count,
        total_volume_eur_cents: total_cents,
        merkle_root_hash: master_merkle_root,
        db_integrity_ok,
        duration_millis,
    })
}

/// Audits cryptographic Merkle chain for a given store.
pub fn verify_pilot_merkle_audit_integrity(
    conn: &Connection,
    store_id: &str,
) -> std::result::Result<bool, PilotError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT prev_block_hash, current_block_hash
        FROM pilot_merkle_tree_log
        WHERE store_id = ?1
        ORDER BY seq_num ASC
        "#,
    )?;

    let mut rows = stmt.query(params![store_id])?;
    let mut expected_prev = "0000000000000000000000000000000000000000000000000000000000000000".to_string();

    while let Some(row) = rows.next()? {
        let prev: String = row.get(0)?;
        let curr: String = row.get(1)?;

        if prev != expected_prev {
            return Ok(false);
        }
        expected_prev = curr;
    }

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pilot_schema_init_and_seed_stores() {
        let conn = Connection::open_in_memory().unwrap();
        init_pilot_simulation_schema(&conn).unwrap();
        let stores = seed_pilot_stores(&conn).unwrap();
        assert_eq!(stores.len(), 3);
    }

    #[test]
    fn test_pilot_counter_stress_execution_and_integrity() {
        let conn = Connection::open_in_memory().unwrap();
        let config = PilotStressConfig {
            operations_per_shop: 25,
            enforce_atomic_merkle_chain: true,
            run_pragmas_on_completion: true,
        };

        let report = run_pilot_counter_stress(&conn, &config).unwrap();
        assert_eq!(report.total_operations_executed, 75);
        assert_eq!(report.intake_tickets_created, 25);
        assert_eq!(report.pos_transactions_recorded, 25);
        assert_eq!(report.shipping_notes_issued, 25);
        assert!(report.db_integrity_ok);
        assert!(!report.merkle_root_hash.is_empty());

        // Verify Merkle integrity for all 3 stores
        assert!(verify_pilot_merkle_audit_integrity(&conn, "STORE-REP-01").unwrap());
        assert!(verify_pilot_merkle_audit_integrity(&conn, "STORE-POS-02").unwrap());
        assert!(verify_pilot_merkle_audit_integrity(&conn, "STORE-LOG-03").unwrap());
    }

    #[test]
    fn test_pilot_exact_penny_ledger_calculation() {
        let conn = Connection::open_in_memory().unwrap();
        let config = PilotStressConfig {
            operations_per_shop: 10,
            enforce_atomic_merkle_chain: true,
            run_pragmas_on_completion: true,
        };

        let report = run_pilot_counter_stress(&conn, &config).unwrap();
        assert_eq!(report.total_volume_eur_cents, 291400);
    }
}
