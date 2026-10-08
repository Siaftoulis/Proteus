//! Automatic Software Update & Release Verification Engine.
//!
//! Provides cryptographic SHA-256 binary and package checksum verification,
//! SemVer delta comparison, atomic staging with `.bak` rollback guarantees,
//! and persistent audit logging of applied updates.

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

/// Semantic version representation (Major.Minor.Patch).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SemVer {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl SemVer {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }

    pub fn parse(s: &str) -> Result<Self, String> {
        let clean = s.trim().trim_start_matches('v');
        let parts: Vec<&str> = clean.split('.').collect();
        if parts.len() != 3 {
            return Err(format!("Invalid SemVer string '{}': expected X.Y.Z", s));
        }
        let major = parts[0].parse().map_err(|e| format!("Invalid major: {}", e))?;
        let minor = parts[1].parse().map_err(|e| format!("Invalid minor: {}", e))?;
        let patch = parts[2].parse().map_err(|e| format!("Invalid patch: {}", e))?;
        Ok(Self { major, minor, patch })
    }

    pub fn is_newer_than(&self, other: &Self) -> bool {
        self > other
    }

    pub fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Release asset metadata containing binary URL and cryptographic signature.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseAsset {
    pub name: String,
    pub target_platform: String,
    pub download_url: String,
    pub sha256_checksum: String,
    pub size_bytes: u64,
}

/// Release manifest published on official sovereign update distribution servers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseManifest {
    pub version: String,
    pub min_supported_version: String,
    pub release_notes: String,
    pub mandatory: bool,
    pub released_at: String,
    pub assets: Vec<ReleaseAsset>,
}

impl ReleaseManifest {
    pub fn parse_json(json_str: &str) -> Result<Self, String> {
        serde_json::from_str(json_str).map_err(|e| format!("Invalid release manifest JSON: {}", e))
    }

    pub fn semver(&self) -> Result<SemVer, String> {
        SemVer::parse(&self.version)
    }

    pub fn find_asset_for_platform(&self, platform: &str) -> Option<&ReleaseAsset> {
        self.assets.iter().find(|a| a.target_platform.eq_ignore_ascii_case(platform))
    }
}

/// Audit log record of an applied or attempted software update.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAuditRecord {
    pub id: String,
    pub target_binary: String,
    pub previous_version: String,
    pub applied_version: String,
    pub sha256_verified: bool,
    pub status: String,
    pub error_message: Option<String>,
    pub timestamp: String,
}

/// Initializes SQLite schema for update audit logging.
pub fn ensure_updater_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS update_audit_log (
            id TEXT PRIMARY KEY,
            target_binary TEXT NOT NULL,
            previous_version TEXT NOT NULL,
            applied_version TEXT NOT NULL,
            sha256_verified INTEGER NOT NULL,
            status TEXT NOT NULL,
            error_message TEXT,
            timestamp TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_update_audit_timestamp ON update_audit_log(timestamp);",
    )
}

/// Records an update event into the SQLite audit log.
pub fn log_update_audit(conn: &Connection, record: &UpdateAuditRecord) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO update_audit_log (id, target_binary, previous_version, applied_version, sha256_verified, status, error_message, timestamp)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            record.id,
            record.target_binary,
            record.previous_version,
            record.applied_version,
            record.sha256_verified as i32,
            record.status,
            record.error_message,
            record.timestamp
        ],
    )?;
    Ok(())
}

/// Verifies that the raw byte payload matches the expected SHA-256 hex string.
pub fn verify_payload_sha256(payload: &[u8], expected_hex: &str) -> bool {
    let mut hasher = Sha256::new();
    hasher.update(payload);
    let result_hex = format!("{:x}", hasher.finalize());
    result_hex.eq_ignore_ascii_case(expected_hex.trim())
}

/// Atomically replaces target binary with staged update payload, creating a `.bak` file for rollback.
pub fn apply_staged_binary(
    target_path: &Path,
    staged_payload: &[u8],
    expected_sha256: &str,
) -> Result<PathBuf, String> {
    if !verify_payload_sha256(staged_payload, expected_sha256) {
        return Err("SHA-256 verification failed: Payload does not match manifest".to_string());
    }

    let backup_path = target_path.with_extension("bak");
    if target_path.exists() {
        fs::copy(target_path, &backup_path)
            .map_err(|e| format!("Failed to create backup at {:?}: {}", backup_path, e))?;
    }

    let staged_path = target_path.with_extension("staged");
    fs::write(&staged_path, staged_payload)
        .map_err(|e| format!("Failed to write staged payload: {}", e))?;

    // Atomic move staged over target
    if let Err(e) = fs::rename(&staged_path, target_path) {
        // Rollback attempt if backup exists
        if backup_path.exists() {
            let _ = fs::copy(&backup_path, target_path);
        }
        return Err(format!("Atomic update replace failed, restored from backup: {}", e));
    }

    Ok(backup_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_semver_parsing_and_comparison() {
        let v1 = SemVer::parse("0.1.0").unwrap();
        let v2 = SemVer::parse("v0.2.0").unwrap();
        let v3 = SemVer::parse("1.0.0").unwrap();

        assert!(v2.is_newer_than(&v1));
        assert!(v3.is_newer_than(&v2));
        assert!(!v1.is_newer_than(&v2));
        assert_eq!(v1.to_string(), "0.1.0");
    }

    #[test]
    fn test_release_manifest_parsing() {
        let json = r#"{
            "version": "1.0.4",
            "min_supported_version": "0.1.0",
            "release_notes": "Fiscal printer fix and POS acceleration",
            "mandatory": false,
            "released_at": "2026-10-06T12:00:00Z",
            "assets": [
                {
                    "name": "proteus-client.exe",
                    "target_platform": "windows",
                    "download_url": "https://cdn.proteus.gr/releases/v1.0.4/proteus-client.exe",
                    "sha256_checksum": "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08",
                    "size_bytes": 18450000
                }
            ]
        }"#;

        let manifest = ReleaseManifest::parse_json(json).unwrap();
        assert_eq!(manifest.version, "1.0.4");
        assert_eq!(manifest.semver().unwrap(), SemVer::new(1, 0, 4));

        let asset = manifest.find_asset_for_platform("windows").unwrap();
        assert_eq!(asset.name, "proteus-client.exe");
        assert_eq!(asset.size_bytes, 18450000);
    }

    #[test]
    fn test_payload_sha256_verification() {
        let payload = b"test";
        // sha256("test") = 9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08
        let valid_hash = "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08";
        assert!(verify_payload_sha256(payload, valid_hash));
        assert!(!verify_payload_sha256(payload, "invalid_hash_value"));
    }

    #[test]
    fn test_updater_audit_log_sqlite() {
        let conn = Connection::open_in_memory().unwrap();
        ensure_updater_schema(&conn).unwrap();

        let record = UpdateAuditRecord {
            id: "UPD-001".to_string(),
            target_binary: "proteus-client.exe".to_string(),
            previous_version: "0.1.0".to_string(),
            applied_version: "0.2.0".to_string(),
            sha256_verified: true,
            status: "SUCCESS".to_string(),
            error_message: None,
            timestamp: Utc::now().to_rfc3339(),
        };

        log_update_audit(&conn, &record).unwrap();

        let mut stmt = conn.prepare("SELECT count(*) FROM update_audit_log WHERE sha256_verified = 1").unwrap();
        let count: i64 = stmt.query_row([], |row| row.get(0)).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_apply_staged_binary_atomic_replace() {
        let temp_dir = std::env::temp_dir().join("proteus_update_test");
        let _ = fs::create_dir_all(&temp_dir);
        let target_file = temp_dir.join("test_app.exe");

        fs::write(&target_file, b"OLD_BINARY_V1").unwrap();

        let new_bytes = b"test";
        let valid_hash = "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08";

        let backup = apply_staged_binary(&target_file, new_bytes, valid_hash).unwrap();
        assert!(backup.exists());
        assert_eq!(fs::read(&backup).unwrap(), b"OLD_BINARY_V1");
        assert_eq!(fs::read(&target_file).unwrap(), b"test");

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
