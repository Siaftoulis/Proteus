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

use std::path::{Path, PathBuf};

/// Manages automated daily local backups and rotation under %APPDATA%\Proteus\backups\
pub struct LocalBackupManager;

impl LocalBackupManager {
    /// Takes a timestamped, encrypted local snapshot of the SQLite database and saves it to backup_dir.
    pub fn create_local_snapshot(
        db_path: &Path,
        backup_dir: &Path,
        password: &str,
        salt: &str,
        retention_days: u32,
    ) -> Result<(BackupManifest, PathBuf), String> {
        if !db_path.exists() {
            return Err(format!("Database file not found: {}", db_path.display()));
        }

        let raw_bytes = std::fs::read(db_path)
            .map_err(|e| format!("Failed to read database file: {}", e))?;

        let (manifest, encrypted_payload) = CloudBackupEngine::create_encrypted_bundle(
            &raw_bytes,
            password,
            salt,
            StorageTier::LocalVault,
            retention_days,
        )?;

        std::fs::create_dir_all(backup_dir)
            .map_err(|e| format!("Failed to create backup directory: {}", e))?;

        let timestamp_str = Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let short_id = if manifest.id.len() >= 8 { &manifest.id[..8] } else { &manifest.id };
        let base_name = format!("proteus_backup_{}_{}", timestamp_str, short_id);
        let bak_path = backup_dir.join(format!("{}.bak", base_name));
        let manifest_path = backup_dir.join(format!("{}.manifest.json", base_name));

        std::fs::write(&bak_path, &encrypted_payload)
            .map_err(|e| format!("Failed to write backup file: {}", e))?;

        let manifest_json = serde_json::to_string_pretty(&manifest)
            .map_err(|e| format!("Failed to serialize manifest: {}", e))?;
        std::fs::write(&manifest_path, manifest_json)
            .map_err(|e| format!("Failed to write manifest file: {}", e))?;

        // Automatic pruning of older expired backups
        let _ = Self::prune_local_backups(backup_dir);

        Ok((manifest, bak_path))
    }

    /// Lists all valid local backups found in backup_dir, sorted newest first.
    pub fn list_local_backups(backup_dir: &Path) -> Result<Vec<(BackupManifest, PathBuf)>, String> {
        if !backup_dir.exists() {
            return Ok(Vec::new());
        }

        let entries = std::fs::read_dir(backup_dir)
            .map_err(|e| format!("Failed to read backup directory: {}", e))?;

        let mut backups = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json")
                && path.to_string_lossy().ends_with(".manifest.json")
            {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(manifest) = serde_json::from_str::<BackupManifest>(&content) {
                        let bak_filename = path.file_name().unwrap_or_default().to_string_lossy()
                            .replace(".manifest.json", ".bak");
                        let bak_path = path.parent().unwrap_or(backup_dir).join(bak_filename);
                        if bak_path.exists() {
                            backups.push((manifest, bak_path));
                        }
                    }
                }
            }
        }

        backups.sort_by(|a, b| b.0.created_at.cmp(&a.0.created_at));
        Ok(backups)
    }

    /// Prunes expired backups according to their individual retention policies.
    pub fn prune_local_backups(backup_dir: &Path) -> Result<usize, String> {
        let backups = Self::list_local_backups(backup_dir)?;
        let mut pruned = 0;

        for (manifest, bak_path) in backups {
            if manifest.is_expired() {
                let manifest_filename = bak_path.file_name().unwrap_or_default().to_string_lossy()
                    .replace(".bak", ".manifest.json");
                let manifest_path = bak_path.parent().unwrap_or(backup_dir).join(manifest_filename);
                let _ = std::fs::remove_file(&bak_path);
                let _ = std::fs::remove_file(&manifest_path);
                pruned += 1;
            }
        }

        Ok(pruned)
    }

    /// Triggers a scheduled backup only if more than interval_hours have passed since the last backup.
    pub fn trigger_scheduled_backup_if_needed(
        db_path: &Path,
        backup_dir: &Path,
        interval_hours: u64,
        password: &str,
        salt: &str,
    ) -> Result<Option<BackupManifest>, String> {
        let existing = Self::list_local_backups(backup_dir)?;
        if let Some((latest, _)) = existing.first() {
            if let Ok(created_dt) = chrono::DateTime::parse_from_rfc3339(&latest.created_at) {
                let elapsed = Utc::now() - created_dt.with_timezone(&Utc);
                if elapsed < Duration::hours(interval_hours as i64) {
                    return Ok(None);
                }
            }
        }

        let (manifest, _) = Self::create_local_snapshot(db_path, backup_dir, password, salt, 7)?;
        Ok(Some(manifest))
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

    #[test]
    fn test_local_backup_manager_lifecycle() {
        let temp_dir = std::env::temp_dir().join(format!("proteus_test_backup_{}", Uuid::now_v7()));
        let db_file = temp_dir.join("store.db");
        let backup_dir = temp_dir.join("backups");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let initial_db_bytes = b"SQLite format 3\0\x10\x00\x01\x01\x00@  real-db-content";
        std::fs::write(&db_file, initial_db_bytes).unwrap();

        // 1. Create snapshot
        let (manifest, bak_path) = LocalBackupManager::create_local_snapshot(
            &db_file,
            &backup_dir,
            "TestPass123!",
            "machine-salt",
            7,
        ).unwrap();

        assert!(bak_path.exists());
        assert_eq!(manifest.storage_tier, StorageTier::LocalVault);

        // 2. List backups
        let list = LocalBackupManager::list_local_backups(&backup_dir).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].0.id, manifest.id);

        // 3. Scheduled check: should NOT trigger because interval is 24h
        let triggered = LocalBackupManager::trigger_scheduled_backup_if_needed(
            &db_file,
            &backup_dir,
            24,
            "TestPass123!",
            "machine-salt",
        ).unwrap();
        assert!(triggered.is_none());

        // 4. Verify restoration of the backup
        let enc_bytes = std::fs::read(&bak_path).unwrap();
        let restored = CloudBackupEngine::restore_bundle(
            &enc_bytes,
            &manifest,
            "TestPass123!",
            "machine-salt",
        ).unwrap();
        assert_eq!(restored, initial_db_bytes);

        // Cleanup
        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
