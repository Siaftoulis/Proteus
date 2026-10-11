//! Cryptographic Manifest Header & Monotonic Time-Lock Sequence Engine (Phase 14).
//! Enforces immutable SHA-256 chain linking, payload hashing, and monotonic sequence
//! progression to prevent cloud snapshot rollbacks, replays, and state truncation.
//! Strict Rule 1 (100% Original Codebase), Rule 5 (Zero Mock Data), Rule 6 (Compliance & Security).

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::cloud_replica::snapshot::{ChunkedSnapshotBundle, ReplicaError};

/// Immutable cryptographic header sealing a zero-knowledge cloud snapshot bundle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplicaManifestHeader {
    pub bundle_id: String,
    pub store_id: String,
    pub sequence_number: u64,
    pub previous_manifest_hash: String,
    pub root_payload_sha256: String,
    pub total_chunks: u32,
    pub uncompressed_bytes: u64,
    pub timestamp_utc: i64,
    pub manifest_signature_hash: String,
}

/// Initializes the local SQLite journal for tracking outbound and inbound replica manifests.
pub fn init_replica_ledger_schema(conn: &Connection) -> Result<(), ReplicaError> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS replica_manifest_ledger (
            bundle_id TEXT PRIMARY KEY,
            store_id TEXT NOT NULL,
            sequence_number INTEGER NOT NULL,
            previous_manifest_hash TEXT NOT NULL,
            root_payload_sha256 TEXT NOT NULL,
            manifest_signature_hash TEXT NOT NULL,
            total_chunks INTEGER NOT NULL,
            uncompressed_bytes INTEGER NOT NULL,
            timestamp_utc INTEGER NOT NULL,
            status TEXT NOT NULL,
            created_at INTEGER NOT NULL
        );
        CREATE UNIQUE INDEX IF NOT EXISTS idx_replica_seq ON replica_manifest_ledger(store_id, sequence_number);
        CREATE INDEX IF NOT EXISTS idx_replica_timestamp ON replica_manifest_ledger(timestamp_utc);
        "#,
    )
    .map_err(|e| ReplicaError::DatabaseError(e.to_string()))?;

    Ok(())
}

/// Calculates the deterministic SHA-256 hash over all encrypted chunks in order.
pub fn compute_bundle_payload_hash(bundle: &ChunkedSnapshotBundle) -> String {
    let mut hasher = Sha256::new();
    for chunk in &bundle.chunks {
        hasher.update(&chunk.chunk_index.to_be_bytes());
        hasher.update(&chunk.nonce);
        hasher.update(&(chunk.ciphertext.len() as u64).to_be_bytes());
        hasher.update(&chunk.ciphertext);
    }
    format!("{:x}", hasher.finalize())
}

/// Computes the cryptographic signature hash sealing all fields of a manifest header.
pub fn compute_manifest_seal(
    bundle_id: &str,
    store_id: &str,
    sequence_number: u64,
    prev_hash: &str,
    payload_hash: &str,
    total_chunks: u32,
    uncompressed_bytes: u64,
    timestamp_utc: i64,
) -> String {
    let mut hasher = Sha256::new();
    let record = format!(
        "{}:{}:{}:{}:{}:{}:{}:{}",
        bundle_id,
        store_id,
        sequence_number,
        prev_hash,
        payload_hash,
        total_chunks,
        uncompressed_bytes,
        timestamp_utc
    );
    hasher.update(record.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Generates a signed cryptographic manifest header for a freshly encrypted snapshot bundle.
pub fn create_signed_manifest(
    store_id: &str,
    sequence_number: u64,
    previous_manifest_hash: &str,
    bundle: &ChunkedSnapshotBundle,
) -> ReplicaManifestHeader {
    let bundle_id = bundle.snapshot_id.clone();
    let root_payload_sha256 = compute_bundle_payload_hash(bundle);
    let timestamp_utc = bundle.created_at;

    let manifest_signature_hash = compute_manifest_seal(
        &bundle_id,
        store_id,
        sequence_number,
        previous_manifest_hash,
        &root_payload_sha256,
        bundle.total_chunks,
        bundle.total_uncompressed_bytes,
        timestamp_utc,
    );

    ReplicaManifestHeader {
        bundle_id,
        store_id: store_id.to_string(),
        sequence_number,
        previous_manifest_hash: previous_manifest_hash.to_string(),
        root_payload_sha256,
        total_chunks: bundle.total_chunks,
        uncompressed_bytes: bundle.total_uncompressed_bytes,
        timestamp_utc,
        manifest_signature_hash,
    }
}

/// Verifies that a manifest header passes internal seal validation and monotonic progression.
pub fn verify_manifest_integrity(
    manifest: &ReplicaManifestHeader,
    last_known: Option<&ReplicaManifestHeader>,
) -> Result<(), ReplicaError> {
    // 1. Verify internal seal
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

    if manifest.manifest_signature_hash != expected_seal {
        return Err(ReplicaError::InvalidBundle(format!(
            "Αποτυχία επαλήθευσης ψηφιακής σφραγίδας manifest για bundle {}",
            manifest.bundle_id
        )));
    }

    // 2. If there is a predecessor, enforce monotonic sequence, timestamp, and chain continuity
    if let Some(prev) = last_known {
        if manifest.sequence_number != prev.sequence_number + 1 {
            return Err(ReplicaError::InvalidBundle(format!(
                "Παραβίαση μονοτονικής αλληλουχίας: αναμενόταν αύξων αριθμός {}, ελήφθη {}",
                prev.sequence_number + 1,
                manifest.sequence_number
            )));
        }

        if manifest.timestamp_utc <= prev.timestamp_utc {
            return Err(ReplicaError::InvalidBundle(format!(
                "Παραβίαση time-lock: η χρονική σήμανση ({}) είναι παλαιότερη ή ίση από το προηγούμενο snapshot ({})",
                manifest.timestamp_utc, prev.timestamp_utc
            )));
        }

        if manifest.previous_manifest_hash != prev.manifest_signature_hash {
            return Err(ReplicaError::InvalidBundle(format!(
                "Διακοπή αλυσίδας SHA-256: το previous_manifest_hash δεν ταυτίζεται με τη σφραγίδα του προηγούμενου snapshot"
            )));
        }
    } else if manifest.sequence_number != 1 {
        // First snapshot in an uninitialized store must have sequence_number == 1
        return Err(ReplicaError::InvalidBundle(format!(
            "Το αρχικό snapshot πρέπει να έχει sequence number 1 (ελήφθη {})",
            manifest.sequence_number
        )));
    }

    Ok(())
}

/// Retrieves the latest committed manifest header for a given store from SQLite.
pub fn get_latest_manifest(
    conn: &Connection,
    store_id: &str,
) -> Result<Option<ReplicaManifestHeader>, ReplicaError> {
    let mut stmt = conn
        .prepare(
            r#"
            SELECT bundle_id, store_id, sequence_number, previous_manifest_hash,
                   root_payload_sha256, manifest_signature_hash, total_chunks,
                   uncompressed_bytes, timestamp_utc
            FROM replica_manifest_ledger
            WHERE store_id = ?1
            ORDER BY sequence_number DESC
            LIMIT 1
            "#,
        )
        .map_err(|e| ReplicaError::DatabaseError(e.to_string()))?;

    let row = stmt
        .query_row(params![store_id], |r| {
            Ok(ReplicaManifestHeader {
                bundle_id: r.get(0)?,
                store_id: r.get(1)?,
                sequence_number: r.get(2)?,
                previous_manifest_hash: r.get(3)?,
                root_payload_sha256: r.get(4)?,
                manifest_signature_hash: r.get(5)?,
                total_chunks: r.get(6)?,
                uncompressed_bytes: r.get(7)?,
                timestamp_utc: r.get(8)?,
            })
        })
        .optional()
        .map_err(|e| ReplicaError::DatabaseError(e.to_string()))?;

    Ok(row)
}

/// Validates and records a new manifest header in the local SQLite ledger.
pub fn record_and_verify_manifest(
    conn: &Connection,
    manifest: &ReplicaManifestHeader,
) -> Result<(), ReplicaError> {
    let latest = get_latest_manifest(conn, &manifest.store_id)?;
    verify_manifest_integrity(manifest, latest.as_ref())?;

    conn.execute(
        r#"
        INSERT INTO replica_manifest_ledger (
            bundle_id, store_id, sequence_number, previous_manifest_hash,
            root_payload_sha256, manifest_signature_hash, total_chunks,
            uncompressed_bytes, timestamp_utc, status, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'COMMITTED', ?10)
        "#,
        params![
            manifest.bundle_id,
            manifest.store_id,
            manifest.sequence_number,
            manifest.previous_manifest_hash,
            manifest.root_payload_sha256,
            manifest.manifest_signature_hash,
            manifest.total_chunks,
            manifest.uncompressed_bytes,
            manifest.timestamp_utc,
            chrono::Utc::now().timestamp_millis(),
        ],
    )
    .map_err(|e| ReplicaError::DatabaseError(e.to_string()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cloud_replica::snapshot::encrypt_snapshot_bytes;

    #[test]
    fn test_create_and_verify_signed_manifest_success() {
        let sample = b"Live retail database WAL records for store replication";
        let bundle = encrypt_snapshot_bytes(sample, "Password123!", "STORE-ATH", 1024).unwrap();

        let manifest = create_signed_manifest("STORE-ATH", 1, "0000000000000000000000000000000000000000000000000000000000000000", &bundle);
        assert_eq!(manifest.sequence_number, 1);
        assert_eq!(manifest.store_id, "STORE-ATH");
        assert_eq!(manifest.manifest_signature_hash.len(), 64);
        assert_eq!(manifest.root_payload_sha256.len(), 64);

        // Verification of initial manifest succeeds
        assert!(verify_manifest_integrity(&manifest, None).is_ok());
    }

    #[test]
    fn test_rejects_tampered_manifest_seal() {
        let sample = b"Sensitive transactional ledger bytes";
        let bundle = encrypt_snapshot_bytes(sample, "Password123!", "STORE-ATH", 1024).unwrap();

        let mut manifest = create_signed_manifest("STORE-ATH", 1, "GENESIS", &bundle);
        // Tamper with payload hash
        manifest.root_payload_sha256 = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_string();

        let res = verify_manifest_integrity(&manifest, None);
        assert!(res.is_err());
    }

    #[test]
    fn test_enforces_monotonic_sequence_and_timelock() {
        let sample = b"Incremental checkpoint payload";
        let mut bundle1 = encrypt_snapshot_bytes(sample, "Key", "STORE-01", 1024).unwrap();
        bundle1.created_at = 1728000000000;
        let m1 = create_signed_manifest("STORE-01", 1, "GENESIS", &bundle1);

        // Successor 2 with later timestamp and matching prev hash
        let mut bundle2 = encrypt_snapshot_bytes(sample, "Key", "STORE-01", 1024).unwrap();
        bundle2.created_at = 1728000005000;
        let m2 = create_signed_manifest("STORE-01", 2, &m1.manifest_signature_hash, &bundle2);

        assert!(verify_manifest_integrity(&m2, Some(&m1)).is_ok());

        // Rejection 1: Non-monotonic sequence (e.g. sequence 1 again or sequence 3 skipping 2)
        let m_bad_seq = create_signed_manifest("STORE-01", 3, &m1.manifest_signature_hash, &bundle2);
        assert!(verify_manifest_integrity(&m_bad_seq, Some(&m1)).is_err());

        // Rejection 2: Rollback timestamp (earlier or equal timestamp)
        let mut bundle_past = encrypt_snapshot_bytes(sample, "Key", "STORE-01", 1024).unwrap();
        bundle_past.created_at = 1727999999000; // In the past!
        let m_bad_time = create_signed_manifest("STORE-01", 2, &m1.manifest_signature_hash, &bundle_past);
        assert!(verify_manifest_integrity(&m_bad_time, Some(&m1)).is_err());

        // Rejection 3: Broken hash chain
        let m_broken_chain = create_signed_manifest("STORE-01", 2, "broken_fake_hash", &bundle2);
        assert!(verify_manifest_integrity(&m_broken_chain, Some(&m1)).is_err());
    }

    #[test]
    fn test_sqlite_manifest_ledger_persistence_and_query() {
        let conn = Connection::open_in_memory().unwrap();
        init_replica_ledger_schema(&conn).unwrap();

        let sample = b"SQLite database checkpoint";
        let mut bundle1 = encrypt_snapshot_bytes(sample, "Secret", "STORE-02", 512).unwrap();
        bundle1.created_at = 1728100000000;
        let m1 = create_signed_manifest("STORE-02", 1, "GENESIS_ROOT", &bundle1);

        record_and_verify_manifest(&conn, &m1).unwrap();

        let latest = get_latest_manifest(&conn, "STORE-02").unwrap();
        assert!(latest.is_some());
        let fetched1 = latest.unwrap();
        assert_eq!(fetched1.sequence_number, 1);
        assert_eq!(fetched1.manifest_signature_hash, m1.manifest_signature_hash);

        // Record next in chain
        let mut bundle2 = encrypt_snapshot_bytes(sample, "Secret", "STORE-02", 512).unwrap();
        bundle2.created_at = 1728100005000;
        let m2 = create_signed_manifest("STORE-02", 2, &m1.manifest_signature_hash, &bundle2);

        record_and_verify_manifest(&conn, &m2).unwrap();

        let latest2 = get_latest_manifest(&conn, "STORE-02").unwrap().unwrap();
        assert_eq!(latest2.sequence_number, 2);
        assert_eq!(latest2.previous_manifest_hash, m1.manifest_signature_hash);
    }
}
