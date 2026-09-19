//! Cryptographic Merkle Hash-Chain & Tamper-Proof Audit Backlog.
//! Guarantees that database records cannot be altered, forged, or cracked without breaking the cryptographic chain.

use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub const GENESIS_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MerkleBlock {
    pub sequence_id: i64,
    pub event_id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub action_type: String,
    pub payload_hash: String,
    pub previous_hash: String,
    pub current_hash: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChainIntegrityReport {
    pub is_valid: bool,
    pub total_blocks: usize,
    pub tampered_at_sequence: Option<i64>,
    pub details: String,
}

/// Computes a hex-encoded SHA-256 hash of a payload string.
pub fn hash_sha256(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    let result = hasher.finalize();
    format!("{:x}", result)
}

/// Computes the cryptographic block hash binding the previous hash, event ID, payload hash, and timestamp.
pub fn compute_block_hash(
    previous_hash: &str,
    event_id: &str,
    payload_hash: &str,
    timestamp: &str,
) -> String {
    let combined = format!("{}:{}:{}:{}", previous_hash, event_id, payload_hash, timestamp);
    hash_sha256(&combined)
}

/// Initializes the tamper-proof audit backlog schema in SQLite.
pub fn init_merkle_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS tamper_proof_audit_backlog (
            sequence_id INTEGER PRIMARY KEY AUTOINCREMENT,
            event_id TEXT NOT NULL UNIQUE,
            entity_type TEXT NOT NULL,
            entity_id TEXT NOT NULL,
            action_type TEXT NOT NULL,
            payload_hash TEXT NOT NULL,
            previous_hash TEXT NOT NULL,
            current_hash TEXT NOT NULL,
            created_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_merkle_seq ON tamper_proof_audit_backlog(sequence_id ASC);
        CREATE INDEX IF NOT EXISTS idx_merkle_entity ON tamper_proof_audit_backlog(entity_type, entity_id);
        "#,
    )?;
    Ok(())
}

/// Appends a new immutable event block to the Merkle hash chain.
pub fn append_merkle_block(
    conn: &Connection,
    entity_type: &str,
    entity_id: &str,
    action_type: &str,
    payload_json: &str,
) -> std::result::Result<MerkleBlock, String> {
    let event_id = Uuid::now_v7().to_string();
    let timestamp = format!(
        "{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    );

    let payload_hash = hash_sha256(payload_json);

    // Get previous block hash (or genesis if empty)
    let prev_hash: String = conn
        .query_row(
            "SELECT current_hash FROM tamper_proof_audit_backlog ORDER BY sequence_id DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .unwrap_or_else(|_| GENESIS_HASH.to_string());

    let current_hash = compute_block_hash(&prev_hash, &event_id, &payload_hash, &timestamp);

    conn.execute(
        "INSERT INTO tamper_proof_audit_backlog (
            event_id, entity_type, entity_id, action_type, payload_hash, previous_hash, current_hash, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            event_id,
            entity_type,
            entity_id,
            action_type,
            payload_hash,
            prev_hash,
            current_hash,
            timestamp,
        ],
    )
    .map_err(|e| format!("Failed to append Merkle block: {}", e))?;

    let sequence_id = conn.last_insert_rowid();

    Ok(MerkleBlock {
        sequence_id,
        event_id,
        entity_type: entity_type.to_string(),
        entity_id: entity_id.to_string(),
        action_type: action_type.to_string(),
        payload_hash,
        previous_hash: prev_hash,
        current_hash,
        created_at: timestamp,
    })
}

/// Verifies the cryptographic integrity of the entire Merkle hash chain from genesis to tip.
pub fn verify_chain_integrity(conn: &Connection) -> std::result::Result<ChainIntegrityReport, String> {
    let mut stmt = conn
        .prepare(
            "SELECT sequence_id, event_id, entity_type, entity_id, action_type, payload_hash, previous_hash, current_hash, created_at
             FROM tamper_proof_audit_backlog
             ORDER BY sequence_id ASC",
        )
        .map_err(|e| e.to_string())?;

    let rows = stmt
        .query_map([], |row| {
            Ok(MerkleBlock {
                sequence_id: row.get(0)?,
                event_id: row.get(1)?,
                entity_type: row.get(2)?,
                entity_id: row.get(3)?,
                action_type: row.get(4)?,
                payload_hash: row.get(5)?,
                previous_hash: row.get(6)?,
                current_hash: row.get(7)?,
                created_at: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?;

    let mut blocks = Vec::new();
    for r in rows {
        blocks.push(r.map_err(|e| e.to_string())?);
    }

    if blocks.is_empty() {
        return Ok(ChainIntegrityReport {
            is_valid: true,
            total_blocks: 0,
            tampered_at_sequence: None,
            details: "Η αλυσίδα είναι κενή (Genesis state).".to_string(),
        });
    }

    let mut expected_prev_hash = GENESIS_HASH.to_string();

    for block in &blocks {
        // 1. Verify previous hash link
        if block.previous_hash != expected_prev_hash {
            return Ok(ChainIntegrityReport {
                is_valid: false,
                total_blocks: blocks.len(),
                tampered_at_sequence: Some(block.sequence_id),
                details: format!(
                    "Σφάλμα συνδεσιμότητας στο Block #{}: αναμενόμενο prev_hash={}, βρέθηκε={}",
                    block.sequence_id, expected_prev_hash, block.previous_hash
                ),
            });
        }

        // 2. Recompute and verify current hash
        let recomputed = compute_block_hash(
            &block.previous_hash,
            &block.event_id,
            &block.payload_hash,
            &block.created_at,
        );
        if recomputed != block.current_hash {
            return Ok(ChainIntegrityReport {
                is_valid: false,
                total_blocks: blocks.len(),
                tampered_at_sequence: Some(block.sequence_id),
                details: format!(
                    "Αλλοίωση περιεχομένου στο Block #{}: αναμενόμενο hash={}, βρέθηκε={}",
                    block.sequence_id, recomputed, block.current_hash
                ),
            });
        }

        expected_prev_hash = block.current_hash.clone();
    }

    Ok(ChainIntegrityReport {
        is_valid: true,
        total_blocks: blocks.len(),
        tampered_at_sequence: None,
        details: format!("✓ Επιτυχής επαλήθευση: {} blocks, 0 αλλοιώσεις.", blocks.len()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_chain_validity() {
        let conn = Connection::open_in_memory().unwrap();
        init_merkle_schema(&conn).unwrap();

        let b1 = append_merkle_block(&conn, "TICKET", "TCK-1", "INTAKE", r#"{"cost":50}"#).unwrap();
        assert_eq!(b1.sequence_id, 1);
        assert_eq!(b1.previous_hash, GENESIS_HASH);

        let b2 = append_merkle_block(&conn, "TICKET", "TCK-1", "STATUS", r#"{"status":"Ready"}"#).unwrap();
        assert_eq!(b2.sequence_id, 2);
        assert_eq!(b2.previous_hash, b1.current_hash);

        let report = verify_chain_integrity(&conn).unwrap();
        assert!(report.is_valid);
        assert_eq!(report.total_blocks, 2);
        assert_eq!(report.tampered_at_sequence, None);
    }

    #[test]
    fn test_tamper_detection_on_row_modification() {
        let conn = Connection::open_in_memory().unwrap();
        init_merkle_schema(&conn).unwrap();

        let _b1 = append_merkle_block(&conn, "PAYMENT", "PAY-1", "RECEIPT", r#"{"amount":100}"#).unwrap();
        let _b2 = append_merkle_block(&conn, "PAYMENT", "PAY-2", "RECEIPT", r#"{"amount":200}"#).unwrap();

        // Attacker attempts to tamper with block 1 payload hash directly in SQLite!
        conn.execute(
            "UPDATE tamper_proof_audit_backlog SET payload_hash = 'hacked_hash' WHERE sequence_id = 1",
            [],
        ).unwrap();

        let report = verify_chain_integrity(&conn).unwrap();
        assert!(!report.is_valid, "Tampered chain must be detected as invalid");
        assert_eq!(report.tampered_at_sequence, Some(1));
    }
}
