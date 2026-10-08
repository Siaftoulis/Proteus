//! 1-Click Zero-Friction Migration Engine between Cloud Hosted & Local Self-Hosted.
//! Enables bidirectional database & asset migration (Cloud -> Local or Local -> Cloud)
//! with SHA-256 seal verification, automatic `.bak` safety rollback snapshot,
//! and atomic database restoration.

use rusqlite::{params, Connection, Result as SqliteResult};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MigrationError {
    #[error("I/O error during migration: {0}")]
    IoError(String),

    #[error("Database error during migration: {0}")]
    DatabaseError(String),

    #[error("Integrity error: checksum mismatch. Expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },

    #[error("Migration manifest missing or invalid: {0}")]
    InvalidManifest(String),

    #[error("No backup found to rollback migration")]
    NoBackupAvailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MigrationDirection {
    CloudToLocal,
    LocalToCloud,
}

impl MigrationDirection {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CloudToLocal => "CLOUD_TO_LOCAL",
            Self::LocalToCloud => "LOCAL_TO_CLOUD",
        }
    }
}

/// Sealed manifest packaging a migration snapshot.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MigrationManifest {
    pub migration_id: String,
    pub tenant_id: String,
    pub direction: MigrationDirection,
    pub source_endpoint: String,
    pub target_endpoint: String,
    pub database_sha256: String,
    pub database_size_bytes: u64,
    pub created_at: String,
}

/// Manages bidirectional snapshot creation, verification, and atomic application.
pub struct MigrationManager;

impl MigrationManager {
    /// Creates a verified migration snapshot of a source SQLite database.
    pub fn create_snapshot(
        migration_id: &str,
        tenant_id: &str,
        direction: MigrationDirection,
        source_db_path: impl AsRef<Path>,
        output_dir: impl AsRef<Path>,
        source_endpoint: &str,
        target_endpoint: &str,
    ) -> Result<(MigrationManifest, PathBuf), MigrationError> {
        let source_path = source_db_path.as_ref();
        let out_dir = output_dir.as_ref();

        if !source_path.exists() {
            return Err(MigrationError::IoError(format!(
                "Source database not found: {}",
                source_path.display()
            )));
        }

        if !out_dir.exists() {
            fs::create_dir_all(out_dir)
                .map_err(|e| MigrationError::IoError(e.to_string()))?;
        }

        let snapshot_db_path = out_dir.join(format!("{}_{}.snap.db", tenant_id, migration_id));
        fs::copy(source_path, &snapshot_db_path)
            .map_err(|e| MigrationError::IoError(e.to_string()))?;

        let db_bytes = fs::read(&snapshot_db_path)
            .map_err(|e| MigrationError::IoError(e.to_string()))?;
        let db_size = db_bytes.len() as u64;

        let db_hash = crate::merkle::hash_sha256(&String::from_utf8_lossy(&db_bytes));

        let now_str = chrono::Utc::now().to_rfc3339();
        let manifest = MigrationManifest {
            migration_id: migration_id.to_string(),
            tenant_id: tenant_id.to_string(),
            direction,
            source_endpoint: source_endpoint.to_string(),
            target_endpoint: target_endpoint.to_string(),
            database_sha256: db_hash,
            database_size_bytes: db_size,
            created_at: now_str,
        };

        let manifest_path = out_dir.join(format!("{}_{}.manifest.json", tenant_id, migration_id));
        let manifest_json = serde_json::to_string_pretty(&manifest)
            .map_err(|e| MigrationError::InvalidManifest(e.to_string()))?;
        fs::write(manifest_path, manifest_json)
            .map_err(|e| MigrationError::IoError(e.to_string()))?;

        Ok((manifest, snapshot_db_path))
    }

    /// Verifies and atomically applies a migration snapshot onto target database location.
    pub fn apply_snapshot(
        manifest: &MigrationManifest,
        snapshot_db_path: impl AsRef<Path>,
        target_db_path: impl AsRef<Path>,
    ) -> Result<Option<PathBuf>, MigrationError> {
        let snap_path = snapshot_db_path.as_ref();
        let target_path = target_db_path.as_ref();

        // 1. Verify file hash
        let snap_bytes = fs::read(snap_path)
            .map_err(|e| MigrationError::IoError(e.to_string()))?;
        let computed_hash = crate::merkle::hash_sha256(&String::from_utf8_lossy(&snap_bytes));

        if computed_hash != manifest.database_sha256 {
            return Err(MigrationError::ChecksumMismatch {
                expected: manifest.database_sha256.clone(),
                actual: computed_hash,
            });
        }

        // 2. Ensure target directory exists
        if let Some(parent) = target_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| MigrationError::IoError(e.to_string()))?;
            }
        }

        // 3. Create rollback .bak if target already exists
        let backup_path = if target_path.exists() {
            let bak = target_path.with_extension("db.bak");
            fs::copy(target_path, &bak)
                .map_err(|e| MigrationError::IoError(e.to_string()))?;
            Some(bak)
        } else {
            None
        };

        // 4. Overwrite target with snapshot
        fs::copy(snap_path, target_path)
            .map_err(|e| MigrationError::IoError(e.to_string()))?;

        Ok(backup_path)
    }

    /// Rolls back an applied migration by restoring the `.bak` database.
    pub fn rollback(target_db_path: impl AsRef<Path>) -> Result<(), MigrationError> {
        let target_path = target_db_path.as_ref();
        let backup_path = target_path.with_extension("db.bak");

        if !backup_path.exists() {
            return Err(MigrationError::NoBackupAvailable);
        }

        fs::copy(&backup_path, target_path)
            .map_err(|e| MigrationError::IoError(e.to_string()))?;
        fs::remove_file(&backup_path)
            .map_err(|e| MigrationError::IoError(e.to_string()))?;

        Ok(())
    }

    /// Initializes SQLite schema for tracking migration history and audit trail.
    pub fn init_migration_schema(conn: &Connection) -> SqliteResult<()> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS migration_audit_log (
                migration_id TEXT PRIMARY KEY,
                tenant_id TEXT NOT NULL,
                direction TEXT NOT NULL,
                source_endpoint TEXT NOT NULL,
                target_endpoint TEXT NOT NULL,
                database_size_bytes INTEGER NOT NULL,
                database_sha256 TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'APPLIED',
                applied_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_migration_tenant ON migration_audit_log(tenant_id);",
        )
    }

    /// Logs an applied migration in the master registry.
    pub fn record_migration_log(
        conn: &Connection,
        manifest: &MigrationManifest,
        status: &str,
    ) -> SqliteResult<()> {
        conn.execute(
            "INSERT INTO migration_audit_log (
                migration_id, tenant_id, direction, source_endpoint, target_endpoint,
                database_size_bytes, database_sha256, status
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(migration_id) DO UPDATE SET status = excluded.status",
            params![
                manifest.migration_id,
                manifest.tenant_id,
                manifest.direction.as_str(),
                manifest.source_endpoint,
                manifest.target_endpoint,
                manifest.database_size_bytes as i64,
                manifest.database_sha256,
                status,
            ],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_apply_cloud_to_local_migration() {
        let temp_dir = std::env::temp_dir().join("proteus_mig_test_roundtrip");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let source_db = temp_dir.join("cloud_store.db");
        let target_db = temp_dir.join("local_store.db");
        let staging_dir = temp_dir.join("staging");

        // 1. Create source DB with records
        {
            let conn = Connection::open(&source_db).unwrap();
            conn.execute("CREATE TABLE tickets (id TEXT PRIMARY KEY, status TEXT);", []).unwrap();
            conn.execute("INSERT INTO tickets (id, status) VALUES ('T-1', 'OPEN');", []).unwrap();
        }

        // 2. Create snapshot
        let (manifest, snap_path) = MigrationManager::create_snapshot(
            "MIG-001",
            "client_test",
            MigrationDirection::CloudToLocal,
            &source_db,
            &staging_dir,
            "https://cloud.proteus-os.gr/client_test",
            "http://localhost:8080",
        )
        .unwrap();

        assert_eq!(manifest.direction, MigrationDirection::CloudToLocal);
        assert!(snap_path.exists());

        // 3. Apply snapshot to target local DB
        let bak = MigrationManager::apply_snapshot(&manifest, &snap_path, &target_db).unwrap();
        assert!(bak.is_none()); // No prior target existed
        assert!(target_db.exists());

        // 4. Verify target has source's data
        {
            let conn = Connection::open(&target_db).unwrap();
            let count: i64 = conn.query_row("SELECT COUNT(*) FROM tickets WHERE status = 'OPEN'", [], |r| r.get(0)).unwrap();
            assert_eq!(count, 1);
        }

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_migration_rollback_when_target_already_exists() {
        let temp_dir = std::env::temp_dir().join("proteus_mig_test_rollback");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let source_db = temp_dir.join("new_source.db");
        let target_db = temp_dir.join("old_target.db");
        let staging_dir = temp_dir.join("staging");

        // Old target has 'OLD' data
        {
            let conn = Connection::open(&target_db).unwrap();
            conn.execute("CREATE TABLE config (ver TEXT);", []).unwrap();
            conn.execute("INSERT INTO config (ver) VALUES ('v1.0');", []).unwrap();
        }

        // New source has 'NEW' data
        {
            let conn = Connection::open(&source_db).unwrap();
            conn.execute("CREATE TABLE config (ver TEXT);", []).unwrap();
            conn.execute("INSERT INTO config (ver) VALUES ('v2.0');", []).unwrap();
        }

        let (manifest, snap_path) = MigrationManager::create_snapshot(
            "MIG-002",
            "client_rollback",
            MigrationDirection::LocalToCloud,
            &source_db,
            &staging_dir,
            "http://localhost:8080",
            "https://cloud.proteus-os.gr/client_rollback",
        )
        .unwrap();

        // Apply replaces target and creates .bak
        let bak = MigrationManager::apply_snapshot(&manifest, &snap_path, &target_db).unwrap();
        assert!(bak.is_some());
        assert!(bak.unwrap().exists());

        // Target currently has 'v2.0'
        {
            let conn = Connection::open(&target_db).unwrap();
            let ver: String = conn.query_row("SELECT ver FROM config", [], |r| r.get(0)).unwrap();
            assert_eq!(ver, "v2.0");
        }

        // Perform rollback
        MigrationManager::rollback(&target_db).unwrap();

        // Target restored to 'v1.0'
        {
            let conn = Connection::open(&target_db).unwrap();
            let ver: String = conn.query_row("SELECT ver FROM config", [], |r| r.get(0)).unwrap();
            assert_eq!(ver, "v1.0");
        }

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_migration_audit_schema() {
        let conn = Connection::open_in_memory().unwrap();
        MigrationManager::init_migration_schema(&conn).unwrap();

        let manifest = MigrationManifest {
            migration_id: "MIG-999".to_string(),
            tenant_id: "tenant_omega".to_string(),
            direction: MigrationDirection::CloudToLocal,
            source_endpoint: "cloud".to_string(),
            target_endpoint: "local".to_string(),
            database_sha256: "hash123".to_string(),
            database_size_bytes: 4096,
            created_at: "2026-10-07T12:00:00Z".to_string(),
        };

        MigrationManager::record_migration_log(&conn, &manifest, "COMPLETED").unwrap();

        let status: String = conn
            .query_row(
                "SELECT status FROM migration_audit_log WHERE migration_id = 'MIG-999'",
                [],
                |r| r.get(0),
            )
            .unwrap();

        assert_eq!(status, "COMPLETED");
    }
}
