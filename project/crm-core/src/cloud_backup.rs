// Proteus Core — Zero-Knowledge Encrypted Cloud Backup & Cold Storage Engine
// Designed from first principles. Zero copied third-party boilerplate.

use crate::encryption;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageTier {
    ColdStorageR2,
    S3Glacier,
    LocalVault,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupManifest {
    pub id: String,
    pub created_at: String,
    pub sha256_checksum: String,
    pub raw_bytes_len: usize,
    pub encrypted_bytes_len: usize,
    pub storage_tier: StorageTier,
    pub retention_days: u32,
}

impl BackupManifest {
    pub fn is_expired(&self) -> bool {
        let created = match chrono::DateTime::parse_from_rfc3339(&self.created_at) {
            Ok(dt) => dt.with_timezone(&Utc),
            Err(_) => return false,
        };
        let cutoff = Utc::now() - Duration::days(self.retention_days as i64);
        created < cutoff
    }
}

pub struct CloudBackupEngine;

impl CloudBackupEngine {
    /// Creates an encrypted cold storage bundle from raw SQLite bytes using Argon2id + XChaCha20Poly1305.
    pub fn create_encrypted_bundle(
        raw_db_bytes: &[u8],
        password: &str,
        salt: &str,
        tier: StorageTier,
        retention_days: u32,
    ) -> Result<(BackupManifest, Vec<u8>), String> {
        if raw_db_bytes.is_empty() {
            return Err("Cannot create backup of empty database".into());
        }

        let mut hasher = Sha256::new();
        hasher.update(raw_db_bytes);
        let checksum = hex::encode(hasher.finalize());

        let key = encryption::derive_key(password, salt)?;
        let encrypted_payload = encryption::encrypt(raw_db_bytes, &key)?;

        let manifest = BackupManifest {
            id: Uuid::now_v7().to_string(),
            created_at: Utc::now().to_rfc3339(),
            sha256_checksum: checksum,
            raw_bytes_len: raw_db_bytes.len(),
            encrypted_bytes_len: encrypted_payload.len(),
            storage_tier: tier,
            retention_days,
        };

        Ok((manifest, encrypted_payload))
    }

    /// Decrypts and verifies the integrity of an encrypted backup bundle.
    pub fn restore_bundle(
        encrypted_bytes: &[u8],
        expected_manifest: &BackupManifest,
        password: &str,
        salt: &str,
    ) -> Result<Vec<u8>, String> {
        let key = encryption::derive_key(password, salt)?;
        let decrypted_bytes = encryption::decrypt(encrypted_bytes, &key)?;

        let mut hasher = Sha256::new();
        hasher.update(&decrypted_bytes);
        let calculated_checksum = hex::encode(hasher.finalize());

        if calculated_checksum != expected_manifest.sha256_checksum {
            return Err(format!(
                "Checksum mismatch during restore: expected {}, got {}",
                expected_manifest.sha256_checksum, calculated_checksum
            ));
        }

        Ok(decrypted_bytes)
    }

    /// Evaluates a list of manifests and returns IDs that should be pruned according to retention rules.
    pub fn prune_expired(manifests: &[BackupManifest]) -> Vec<String> {
        manifests
            .iter()
            .filter(|m| m.is_expired())
            .map(|m| m.id.clone())
            .collect()
    }
}

// Simple internal hex encoder to avoid extra dependencies
mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes.as_ref().iter().map(|b| format!("{:02x}", b)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_restore_backup_bundle() {
        let dummy_db = b"SQLite format 3\0\x10\x00\x01\x01\x00@  \x00\x00\x00\x01";
        let (manifest, enc) = CloudBackupEngine::create_encrypted_bundle(
            dummy_db,
            "MasterBackupPass!",
            "device-salt-999",
            StorageTier::ColdStorageR2,
            30,
        ).unwrap();

        assert_eq!(manifest.storage_tier, StorageTier::ColdStorageR2);
        assert_eq!(manifest.raw_bytes_len, dummy_db.len());
        assert!(!manifest.is_expired());

        let restored = CloudBackupEngine::restore_bundle(
            &enc,
            &manifest,
            "MasterBackupPass!",
            "device-salt-999",
        ).unwrap();

        assert_eq!(restored, dummy_db);
    }

    #[test]
    fn test_restore_tampered_fails() {
        let dummy_db = b"secret database content";
        let (manifest, mut enc) = CloudBackupEngine::create_encrypted_bundle(
            dummy_db,
            "pass",
            "salt",
            StorageTier::LocalVault,
            30,
        ).unwrap();

        if let Some(byte) = enc.last_mut() {
            *byte ^= 0xFF;
        }

        let res = CloudBackupEngine::restore_bundle(&enc, &manifest, "pass", "salt");
        assert!(res.is_err());
    }

    #[test]
    fn test_retention_prune_expired() {
        let old_manifest = BackupManifest {
            id: "old-1".into(),
            created_at: "2020-01-01T00:00:00Z".into(),
            sha256_checksum: "dummy".into(),
            raw_bytes_len: 100,
            encrypted_bytes_len: 140,
            storage_tier: StorageTier::ColdStorageR2,
            retention_days: 30,
        };
        let fresh_manifest = BackupManifest {
            id: "fresh-1".into(),
            created_at: Utc::now().to_rfc3339(),
            sha256_checksum: "dummy".into(),
            raw_bytes_len: 100,
            encrypted_bytes_len: 140,
            storage_tier: StorageTier::ColdStorageR2,
            retention_days: 30,
        };

        let to_prune = CloudBackupEngine::prune_expired(&[old_manifest, fresh_manifest]);
        assert_eq!(to_prune, vec!["old-1"]);
    }
}
