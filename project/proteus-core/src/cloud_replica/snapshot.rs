//! Sovereign Zero-Knowledge Encrypted SQLite WAL Snapshot Archiver.
//! Uses AES-256-GCM chunked authenticated encryption and Argon2id key derivation (Phase 14).
//! Ensures offsite cloud replicas remain strictly unreadable by cloud hosting providers (Zero-Knowledge).
//! Strict Rule 1 (100% Original Codebase), Rule 5 (Zero Mock Data), Rule 6 (Compliance & Security).

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::{Algorithm, Argon2, Params, Version};
use rand::RngCore;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Default chunk size for encrypted snapshot streams (64 KB).
pub const DEFAULT_CHUNK_SIZE_BYTES: usize = 64 * 1024;

/// Strongly typed errors for cloud replica snapshot operations.
#[derive(Debug, Error, PartialEq, Clone)]
pub enum ReplicaError {
    #[error("Argon2id key derivation error: {0}")]
    KeyDerivationFailed(String),
    #[error("AES-256-GCM encryption error: {0}")]
    EncryptionFailed(String),
    #[error("AES-256-GCM decryption failed (tampered data or wrong passphrase): {0}")]
    DecryptionFailed(String),
    #[error("Corrupted or missing chunk index {0}")]
    CorruptedChunk(u32),
    #[error("Invalid snapshot bundle format: {0}")]
    InvalidBundle(String),
    #[error("SQLite snapshot capture error: {0}")]
    DatabaseError(String),
    #[error("I/O error during snapshot creation: {0}")]
    IoError(String),
}

/// A single AES-256-GCM encrypted block within a chunked snapshot bundle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EncryptedChunk {
    pub chunk_index: u32,
    pub nonce: [u8; 12],
    pub ciphertext: Vec<u8>,
}

/// Zero-knowledge encrypted snapshot bundle ready for sovereign offsite replica storage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChunkedSnapshotBundle {
    pub format_version: u32,
    pub store_id: String,
    pub snapshot_id: String,
    pub salt: [u8; 16],
    pub total_uncompressed_bytes: u64,
    pub chunk_size_bytes: u32,
    pub total_chunks: u32,
    pub chunks: Vec<EncryptedChunk>,
    pub created_at: i64,
}

/// Derives a 256-bit AES encryption key from a master passphrase and 128-bit salt using Argon2id.
pub fn derive_replica_key(passphrase: &str, salt: &[u8; 16]) -> Result<[u8; 32], ReplicaError> {
    let params = Params::new(65536, 3, 4, Some(32))
        .map_err(|e| ReplicaError::KeyDerivationFailed(e.to_string()))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut key = [0u8; 32];
    argon2
        .hash_password_into(passphrase.as_bytes(), salt, &mut key)
        .map_err(|e| ReplicaError::KeyDerivationFailed(e.to_string()))?;

    Ok(key)
}

/// Encrypts raw database bytes into a chunked AES-256-GCM snapshot bundle.
pub fn encrypt_snapshot_bytes(
    raw_bytes: &[u8],
    passphrase: &str,
    store_id: &str,
    chunk_size: usize,
) -> Result<ChunkedSnapshotBundle, ReplicaError> {
    let chunk_size = if chunk_size == 0 {
        DEFAULT_CHUNK_SIZE_BYTES
    } else {
        chunk_size
    };

    let mut salt = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut salt);

    let key = derive_replica_key(passphrase, &salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| ReplicaError::EncryptionFailed(e.to_string()))?;

    let total_chunks = ((raw_bytes.len() + chunk_size - 1) / chunk_size).max(1);
    let mut chunks = Vec::with_capacity(total_chunks);

    for (index, slice) in raw_bytes.chunks(chunk_size).enumerate() {
        let mut nonce_bytes = [0u8; 12];
        rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, slice)
            .map_err(|e| ReplicaError::EncryptionFailed(e.to_string()))?;

        chunks.push(EncryptedChunk {
            chunk_index: index as u32,
            nonce: nonce_bytes,
            ciphertext,
        });
    }

    Ok(ChunkedSnapshotBundle {
        format_version: 1,
        store_id: store_id.to_string(),
        snapshot_id: Uuid::now_v7().to_string(),
        salt,
        total_uncompressed_bytes: raw_bytes.len() as u64,
        chunk_size_bytes: chunk_size as u32,
        total_chunks: chunks.len() as u32,
        chunks,
        created_at: chrono::Utc::now().timestamp_millis(),
    })
}

/// Decrypts a chunked AES-256-GCM snapshot bundle back into raw database bytes.
pub fn decrypt_snapshot_bundle(
    bundle: &ChunkedSnapshotBundle,
    passphrase: &str,
) -> Result<Vec<u8>, ReplicaError> {
    if bundle.chunks.len() != bundle.total_chunks as usize {
        return Err(ReplicaError::InvalidBundle(format!(
            "Αριθμός chunks ({}) διαφέρει από το δηλωθέν σύνολο ({})",
            bundle.chunks.len(),
            bundle.total_chunks
        )));
    }

    let key = derive_replica_key(passphrase, &bundle.salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| ReplicaError::DecryptionFailed(e.to_string()))?;

    let mut recovered_bytes = Vec::with_capacity(bundle.total_uncompressed_bytes as usize);

    for (expected_idx, chunk) in bundle.chunks.iter().enumerate() {
        if chunk.chunk_index != expected_idx as u32 {
            return Err(ReplicaError::CorruptedChunk(chunk.chunk_index));
        }

        let nonce = Nonce::from_slice(&chunk.nonce);
        let decrypted_slice = cipher
            .decrypt(nonce, chunk.ciphertext.as_ref())
            .map_err(|e| ReplicaError::DecryptionFailed(e.to_string()))?;

        recovered_bytes.extend_from_slice(&decrypted_slice);
    }

    Ok(recovered_bytes)
}

/// Captures a live consistent SQLite database snapshot without blocking concurrent writes.
pub fn capture_sqlite_snapshot_bytes(conn: &Connection) -> Result<Vec<u8>, ReplicaError> {
    let temp_snap_path = std::env::temp_dir().join(format!("proteus_snap_{}.db", Uuid::now_v7()));
    let path_str = temp_snap_path
        .to_str()
        .ok_or_else(|| ReplicaError::IoError("Invalid temporary path unicode".to_string()))?;

    conn.execute("VACUUM INTO ?1", params![path_str])
        .map_err(|e| ReplicaError::DatabaseError(e.to_string()))?;

    let bytes = std::fs::read(&temp_snap_path)
        .map_err(|e| ReplicaError::IoError(e.to_string()))?;

    let _ = std::fs::remove_file(&temp_snap_path);

    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_replica_key_deterministic() {
        let salt = [42u8; 16];
        let k1 = derive_replica_key("Passphrase123!", &salt).unwrap();
        let k2 = derive_replica_key("Passphrase123!", &salt).unwrap();
        assert_eq!(k1, k2);

        let k_diff = derive_replica_key("DifferentPassphrase!", &salt).unwrap();
        assert_ne!(k1, k_diff);
    }

    #[test]
    fn test_encrypt_and_decrypt_chunked_snapshot_roundtrip() {
        let sample_data = b"PR-SNAPSHOT: Master Ledger 2026. Consistent SQLite WAL replication chunk test."
            .repeat(500); // ~40 KB
        let passphrase = "SovereignStorePassword#2026";
        let store_id = "STORE-ATH-01";

        // Chunk size 4 KB so we get ~10 chunks
        let bundle = encrypt_snapshot_bytes(&sample_data, passphrase, store_id, 4096).unwrap();
        assert_eq!(bundle.store_id, store_id);
        assert!(bundle.total_chunks >= 9);
        assert_eq!(bundle.total_uncompressed_bytes, sample_data.len() as u64);

        // Decrypt
        let recovered = decrypt_snapshot_bundle(&bundle, passphrase).unwrap();
        assert_eq!(recovered, sample_data);
    }

    #[test]
    fn test_decrypt_with_wrong_password_fails() {
        let sample = b"Sensitive customer accounts payable and receivables data";
        let bundle = encrypt_snapshot_bytes(sample, "CorrectPassword", "STORE-01", 1024).unwrap();

        let res = decrypt_snapshot_bundle(&bundle, "WrongPassword");
        assert!(res.is_err());
    }

    #[test]
    fn test_decrypt_tampered_ciphertext_fails_auth_tag() {
        let sample = b"Original uncorrupted financial records";
        let mut bundle = encrypt_snapshot_bytes(sample, "Key#1", "STORE-01", 1024).unwrap();

        // Tamper with a single byte in the ciphertext of chunk 0
        if let Some(chunk) = bundle.chunks.get_mut(0) {
            let last = chunk.ciphertext.len() - 1;
            chunk.ciphertext[last] ^= 0xFF;
        }

        let res = decrypt_snapshot_bundle(&bundle, "Key#1");
        assert!(res.is_err());
    }

    #[test]
    fn test_sqlite_live_vacuum_snapshot_and_restore() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE customers (id INTEGER PRIMARY KEY, name TEXT);
            INSERT INTO customers (name) VALUES ('Γεώργιος Παπαδόπουλος');
            INSERT INTO customers (name) VALUES ('Μαρία Οικονόμου');
            "#,
        )
        .unwrap();

        let snap_bytes = capture_sqlite_snapshot_bytes(&conn).unwrap();
        assert!(!snap_bytes.is_empty());

        // Encrypt snapshot
        let bundle = encrypt_snapshot_bytes(&snap_bytes, "MasterBackupKey2026", "STORE-GR-01", 8192).unwrap();
        let decrypted_bytes = decrypt_snapshot_bundle(&bundle, "MasterBackupKey2026").unwrap();
        assert_eq!(decrypted_bytes, snap_bytes);

        // Verify restoration into new SQLite DB
        let temp_restore_path = std::env::temp_dir().join(format!("proteus_restore_{}.db", Uuid::now_v7()));
        std::fs::write(&temp_restore_path, &decrypted_bytes).unwrap();

        let restored_conn = Connection::open(&temp_restore_path).unwrap();
        let count: i64 = restored_conn.query_row("SELECT count(*) FROM customers", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 2);

        let _ = std::fs::remove_file(&temp_restore_path);
    }
}
