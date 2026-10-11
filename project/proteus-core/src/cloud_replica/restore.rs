//! Automated Disaster Recovery Restore & Integrity Verification Engine (Phase 14).
//! Performs pre-flight dry-run cryptographic validation, byte-level SQLite header inspection,
//! atomic file materialization, and PRAGMA integrity verification for multi-store disaster recovery.
//! Strict Rule 1 (100% Original Codebase), Rule 5 (Zero Mock Data), Rule 6 (Compliance & Security).

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;
use uuid::Uuid;

use crate::cloud_replica::manifest::{
    compute_bundle_payload_hash, compute_manifest_seal, ReplicaManifestHeader,
};
use crate::cloud_replica::snapshot::{
    decrypt_snapshot_bundle, ChunkedSnapshotBundle, ReplicaError,
};

/// Canonical 16-byte magic header for authentic SQLite database files.
pub const SQLITE_MAGIC_HEADER: &[u8; 16] = b"SQLite format 3\0";

/// Result of an automated pre-flight disaster recovery dry-run check.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DryRunReport {
    pub is_valid: bool,
    pub store_id: String,
    pub sequence_number: u64,
    pub total_chunks: u32,
    pub uncompressed_bytes: u64,
    pub tamper_check_passed: bool,
    pub auth_tag_verified: bool,
    pub sqlite_header_verified: bool,
    pub duration_ms: u64,
    pub errors: Vec<String>,
}

/// Result of a finalized live disaster recovery restoration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisasterRecoveryResult {
    pub restored_file_path: String,
    pub table_count: usize,
    pub total_bytes: u64,
    pub integrity_check_status: String,
}

/// Audit log record documenting disaster recovery dry-runs and restores.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DrAuditRecord {
    pub audit_id: String,
    pub store_id: String,
    pub bundle_id: String,
    pub sequence_number: u64,
    pub action_type: String,
    pub success: bool,
    pub details: String,
    pub created_at: i64,
}

/// Initializes the disaster recovery compliance audit table in SQLite.
pub fn init_dr_audit_schema(conn: &Connection) -> Result<(), ReplicaError> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS replica_dr_audit_logs (
            audit_id TEXT PRIMARY KEY,
            store_id TEXT NOT NULL,
            bundle_id TEXT NOT NULL,
            sequence_number INTEGER NOT NULL,
            action_type TEXT NOT NULL,
            success INTEGER NOT NULL,
            details TEXT NOT NULL,
            created_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_replica_dr_audit ON replica_dr_audit_logs(store_id, created_at);
        "#,
    )
    .map_err(|e| ReplicaError::DatabaseError(e.to_string()))?;

    Ok(())
}

/// Executes a safe non-destructive pre-flight dry run verifying cryptographic and structural integrity.
pub fn execute_dry_run_verification(
    bundle: &ChunkedSnapshotBundle,
    manifest: &ReplicaManifestHeader,
    passphrase: &str,
) -> DryRunReport {
    let start_time = std::time::Instant::now();
    let mut errors = Vec::new();

    // 1. Validate manifest internal signature seal
    let expected_seal = compute_manifest_seal(
        &manifest.bundle_id,
        &manifest.store_id,
        manifest.sequence_number,
        &manifest.previous_manifest_hash,
        &manifest.root_payload_sha256,
        manifest.total_chunks,
        manifest.uncompressed_bytes,
        manifest.timestamp_utc,
    );
    let seal_ok = manifest.manifest_signature_hash == expected_seal;
    if !seal_ok {
        errors.push("Αποτυχία επαλήθευσης ψηφιακής σφραγίδας manifest.".to_string());
    }

    // 2. Validate payload hash matches encrypted chunks in bundle
    let calculated_payload_hash = compute_bundle_payload_hash(bundle);
    let payload_ok = manifest.root_payload_sha256 == calculated_payload_hash;
    if !payload_ok {
        errors.push("Το root payload SHA-256 δεν ταυτίζεται με τα κρυπτογραφημένα chunks.".to_string());
    }

    // 3. Trial authenticated decryption
    let mut auth_tag_verified = false;
    let mut sqlite_header_verified = false;

    match decrypt_snapshot_bundle(bundle, passphrase) {
        Ok(raw_bytes) => {
            auth_tag_verified = true;

            // 4. Inspect SQLite magic header
            if raw_bytes.len() >= 16 && &raw_bytes[..16] == SQLITE_MAGIC_HEADER {
                sqlite_header_verified = true;
            } else {
                errors.push("Τα αποκρυπτογραφημένα δεδομένα δεν περιέχουν έγκυρη κεφαλίδα SQLite (SQLite format 3).".to_string());
            }

            if raw_bytes.len() as u64 != bundle.total_uncompressed_bytes {
                errors.push(format!(
                    "Το μέγεθος των αποκρυπτογραφημένων δεδομένων ({} bytes) διαφέρει από το δηλωθέν ({} bytes).",
                    raw_bytes.len(),
                    bundle.total_uncompressed_bytes
                ));
            }
        }
        Err(err) => {
            errors.push(format!("Αποτυχία δοκιμαστικής αποκρυπτογράφησης: {}", err));
        }
    }

    let is_valid = seal_ok && payload_ok && auth_tag_verified && sqlite_header_verified && errors.is_empty();
    let duration_ms = start_time.elapsed().as_millis() as u64;

    DryRunReport {
        is_valid,
        store_id: bundle.store_id.clone(),
        sequence_number: manifest.sequence_number,
        total_chunks: bundle.total_chunks,
        uncompressed_bytes: bundle.total_uncompressed_bytes,
        tamper_check_passed: seal_ok && payload_ok,
        auth_tag_verified,
        sqlite_header_verified,
        duration_ms,
        errors,
    }
}

/// Restores an encrypted bundle into an authentic on-disk SQLite database with PRAGMA integrity verification.
pub fn restore_snapshot_to_file(
    bundle: &ChunkedSnapshotBundle,
    manifest: &ReplicaManifestHeader,
    passphrase: &str,
    target_path: &Path,
) -> Result<DisasterRecoveryResult, ReplicaError> {
    // 1. Perform pre-flight dry-run check
    let dry_run = execute_dry_run_verification(bundle, manifest, passphrase);
    if !dry_run.is_valid {
        return Err(ReplicaError::InvalidBundle(format!(
            "Αποτυχία ελέγχου εγκυρότητας dry-run: {:?}",
            dry_run.errors
        )));
    }

    // 2. Decrypt raw bytes
    let raw_bytes = decrypt_snapshot_bundle(bundle, passphrase)?;

    // 3. Atomic file materialization via staging file
    let staging_path = target_path.with_extension(format!("staging.{}", Uuid::now_v7()));
    std::fs::write(&staging_path, &raw_bytes)
        .map_err(|e| ReplicaError::IoError(e.to_string()))?;

    // 4. Verify SQLite consistency via PRAGMA integrity_check
    let integrity_status = {
        let conn = Connection::open(&staging_path)
            .map_err(|e| ReplicaError::DatabaseError(e.to_string()))?;
        let status: String = conn
            .query_row("PRAGMA integrity_check", [], |r| r.get(0))
            .map_err(|e| ReplicaError::DatabaseError(e.to_string()))?;
        status
    };

    if integrity_status != "ok" {
        let _ = std::fs::remove_file(&staging_path);
        return Err(ReplicaError::DatabaseError(format!(
            "Αποτυχία PRAGMA integrity_check: {}",
            integrity_status
        )));
    }

    // 5. Query table count to confirm schema completeness
    let table_count: usize = {
        let conn = Connection::open(&staging_path)
            .map_err(|e| ReplicaError::DatabaseError(e.to_string()))?;
        let count: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        count as usize
    };

    // 6. Atomic replacement: rename staging to target
    if target_path.exists() {
        let _ = std::fs::remove_file(target_path);
    }
    std::fs::rename(&staging_path, target_path)
        .map_err(|e| ReplicaError::IoError(e.to_string()))?;

    Ok(DisasterRecoveryResult {
        restored_file_path: target_path.to_string_lossy().to_string(),
        table_count,
        total_bytes: raw_bytes.len() as u64,
        integrity_check_status: integrity_status,
    })
}

/// Logs a disaster recovery operation into the SQLite audit ledger.
pub fn log_dr_event(
    conn: &Connection,
    store_id: &str,
    bundle_id: &str,
    seq: u64,
    action: &str,
    success: bool,
    details: &str,
) -> Result<(), ReplicaError> {
    let audit_id = Uuid::now_v7().to_string();
    let now = chrono::Utc::now().timestamp_millis();

    conn.execute(
        r#"
        INSERT INTO replica_dr_audit_logs (
            audit_id, store_id, bundle_id, sequence_number,
            action_type, success, details, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        "#,
        params![
            audit_id,
            store_id,
            bundle_id,
            seq,
            action,
            if success { 1 } else { 0 },
            details,
            now
        ],
    )
    .map_err(|e| ReplicaError::DatabaseError(e.to_string()))?;

    Ok(())
}

/// Retrieves the most recent disaster recovery audit logs from SQLite.
pub fn list_dr_audit_logs(
    conn: &Connection,
    limit: usize,
) -> Result<Vec<DrAuditRecord>, ReplicaError> {
    let mut stmt = conn
        .prepare(
            r#"
            SELECT audit_id, store_id, bundle_id, sequence_number,
                   action_type, success, details, created_at
            FROM replica_dr_audit_logs
            ORDER BY created_at DESC, sequence_number DESC
            LIMIT ?1
            "#,
        )
        .map_err(|e| ReplicaError::DatabaseError(e.to_string()))?;

    let rows = stmt
        .query_map(params![limit as i64], |row| {
            let succ_int: i64 = row.get(5)?;
            Ok(DrAuditRecord {
                audit_id: row.get(0)?,
                store_id: row.get(1)?,
                bundle_id: row.get(2)?,
                sequence_number: row.get(3)?,
                action_type: row.get(4)?,
                success: succ_int == 1,
                details: row.get(6)?,
                created_at: row.get(7)?,
            })
        })
        .map_err(|e| ReplicaError::DatabaseError(e.to_string()))?;

    let mut logs = Vec::new();
    for row in rows {
        logs.push(row.map_err(|e| ReplicaError::DatabaseError(e.to_string()))?);
    }
    Ok(logs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cloud_replica::manifest::create_signed_manifest;
    use crate::cloud_replica::snapshot::{capture_sqlite_snapshot_bytes, encrypt_snapshot_bytes};

    fn create_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE inventory (sku TEXT PRIMARY KEY, qty INTEGER);
            INSERT INTO inventory (sku, qty) VALUES ('SKU-1001', 45);
            INSERT INTO inventory (sku, qty) VALUES ('SKU-1002', 12);
            CREATE TABLE receipts (id INTEGER PRIMARY KEY, total REAL);
            INSERT INTO receipts (total) VALUES (150.50);
            "#,
        )
        .unwrap();
        conn
    }

    #[test]
    fn test_dry_run_verify_bundle_success() {
        let src_conn = create_test_db();
        let snap_bytes = capture_sqlite_snapshot_bytes(&src_conn).unwrap();

        let bundle = encrypt_snapshot_bytes(&snap_bytes, "Passphrase123!", "STORE-ATH", 2048).unwrap();
        let manifest = create_signed_manifest("STORE-ATH", 1, "ROOT_GENESIS", &bundle);

        let report = execute_dry_run_verification(&bundle, &manifest, "Passphrase123!");
        assert!(report.is_valid);
        assert!(report.tamper_check_passed);
        assert!(report.auth_tag_verified);
        assert!(report.sqlite_header_verified);
        assert!(report.errors.is_empty());
    }

    #[test]
    fn test_dry_run_rejects_wrong_password() {
        let src_conn = create_test_db();
        let snap_bytes = capture_sqlite_snapshot_bytes(&src_conn).unwrap();

        let bundle = encrypt_snapshot_bytes(&snap_bytes, "CorrectPassword", "STORE-ATH", 2048).unwrap();
        let manifest = create_signed_manifest("STORE-ATH", 1, "ROOT_GENESIS", &bundle);

        let report = execute_dry_run_verification(&bundle, &manifest, "WrongPassword");
        assert!(!report.is_valid);
        assert!(!report.auth_tag_verified);
        assert!(!report.errors.is_empty());
    }

    #[test]
    fn test_live_restore_materialization_and_pragma_integrity() {
        let src_conn = create_test_db();
        let snap_bytes = capture_sqlite_snapshot_bytes(&src_conn).unwrap();

        let bundle = encrypt_snapshot_bytes(&snap_bytes, "RecoveryKey#2026", "STORE-01", 4096).unwrap();
        let manifest = create_signed_manifest("STORE-01", 1, "GENESIS", &bundle);

        let target_file = std::env::temp_dir().join(format!("proteus_dr_live_{}.db", Uuid::now_v7()));

        let result = restore_snapshot_to_file(&bundle, &manifest, "RecoveryKey#2026", &target_file).unwrap();
        assert_eq!(result.integrity_check_status, "ok");
        assert_eq!(result.table_count, 2);
        assert_eq!(result.total_bytes, snap_bytes.len() as u64);

        // Verify queries on restored file
        let restored_conn = Connection::open(&target_file).unwrap();
        let qty: i64 = restored_conn.query_row("SELECT qty FROM inventory WHERE sku = 'SKU-1001'", [], |r| r.get(0)).unwrap();
        assert_eq!(qty, 45);

        let _ = std::fs::remove_file(&target_file);
    }

    #[test]
    fn test_dr_audit_logs_sqlite_persistence() {
        let conn = Connection::open_in_memory().unwrap();
        init_dr_audit_schema(&conn).unwrap();

        log_dr_event(&conn, "STORE-01", "B-99", 5, "DRY_RUN", true, "Όλοι οι έλεγχοι πέρασαν επιτυχώς").unwrap();
        log_dr_event(&conn, "STORE-01", "B-100", 6, "LIVE_RESTORE", true, "Επιτυχής επαναφορά PRAGMA integrity ok").unwrap();

        let logs = list_dr_audit_logs(&conn, 10).unwrap();
        assert_eq!(logs.len(), 2);
        assert_eq!(logs[0].sequence_number, 6);
        assert_eq!(logs[0].action_type, "LIVE_RESTORE");
        assert!(logs[0].success);
        assert_eq!(logs[1].sequence_number, 5);
        assert_eq!(logs[1].action_type, "DRY_RUN");
    }
}
