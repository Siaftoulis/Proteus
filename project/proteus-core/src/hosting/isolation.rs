//! Multi-Tenant Data Isolation Engine & Storage Quota Enforcement.
//! Provides segregated per-client SQLite database containers, strict sandbox pathing,
//! zero cross-tenant data leakage, and 50GB storage quota enforcement.

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Default storage quota per client tenant: 50 Gigabytes.
pub const DEFAULT_TENANT_QUOTA_BYTES: u64 = 50 * 1024 * 1024 * 1024;

#[derive(Debug, Error)]
pub enum StorageQuotaError {
    #[error("Invalid tenant identifier: '{0}'. Must be 3-64 chars [a-zA-Z0-9_-]")]
    InvalidTenantIdentifier(String),

    #[error("Path traversal attack detected: '{0}'")]
    PathTraversalDetected(String),

    #[error("Storage quota exceeded for tenant '{tenant_id}': current {current_bytes}B + requested {requested_bytes}B > quota {quota_bytes}B")]
    QuotaExceeded {
        tenant_id: String,
        current_bytes: u64,
        requested_bytes: u64,
        quota_bytes: u64,
    },

    #[error("Storage I/O error: {0}")]
    StorageIoError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),
}

/// Tenant storage metrics and quota consumption report.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TenantStorageUsage {
    pub tenant_id: String,
    pub database_bytes: u64,
    pub wal_bytes: u64,
    pub total_bytes: u64,
    pub quota_bytes: u64,
    pub usage_percent: f64,
    pub is_exceeded: bool,
}

/// Isolated container context for a single tenant.
#[derive(Debug, Clone)]
pub struct TenantContainer {
    pub tenant_id: String,
    pub container_dir: PathBuf,
    pub db_path: PathBuf,
    pub quota_bytes: u64,
}

impl TenantContainer {
    /// Validates a tenant identifier against path traversal and special characters.
    pub fn validate_tenant_id(id: &str) -> Result<(), StorageQuotaError> {
        let trimmed = id.trim();
        if trimmed.len() < 3 || trimmed.len() > 64 {
            return Err(StorageQuotaError::InvalidTenantIdentifier(id.to_string()));
        }

        if trimmed.contains('/') || trimmed.contains('\\') || trimmed.contains("..") {
            return Err(StorageQuotaError::PathTraversalDetected(id.to_string()));
        }

        for ch in trimmed.chars() {
            if !ch.is_ascii_alphanumeric() && ch != '_' && ch != '-' {
                return Err(StorageQuotaError::InvalidTenantIdentifier(id.to_string()));
            }
        }

        Ok(())
    }

    /// Measures the exact on-disk size of the tenant's database and WAL files.
    pub fn get_storage_usage(&self) -> Result<TenantStorageUsage, StorageQuotaError> {
        let db_bytes = if self.db_path.exists() {
            fs::metadata(&self.db_path)
                .map_err(|e| StorageQuotaError::StorageIoError(e.to_string()))?
                .len()
        } else {
            0
        };

        let wal_path = self.container_dir.join(format!("{}-wal", self.db_path.file_name().unwrap().to_string_lossy()));
        let wal_bytes = if wal_path.exists() {
            fs::metadata(&wal_path)
                .map_err(|e| StorageQuotaError::StorageIoError(e.to_string()))?
                .len()
        } else {
            0
        };

        let total_bytes = db_bytes + wal_bytes;
        let usage_percent = if self.quota_bytes > 0 {
            (total_bytes as f64 / self.quota_bytes as f64) * 100.0
        } else {
            0.0
        };

        Ok(TenantStorageUsage {
            tenant_id: self.tenant_id.clone(),
            database_bytes: db_bytes,
            wal_bytes,
            total_bytes,
            quota_bytes: self.quota_bytes,
            usage_percent: (usage_percent * 100.0).round() / 100.0,
            is_exceeded: total_bytes > self.quota_bytes,
        })
    }

    /// Verifies if the container has sufficient headroom for incoming data write.
    pub fn check_headroom(&self, incoming_bytes: u64) -> Result<(), StorageQuotaError> {
        let usage = self.get_storage_usage()?;
        if usage.total_bytes + incoming_bytes > self.quota_bytes {
            return Err(StorageQuotaError::QuotaExceeded {
                tenant_id: self.tenant_id.clone(),
                current_bytes: usage.total_bytes,
                requested_bytes: incoming_bytes,
                quota_bytes: self.quota_bytes,
            });
        }
        Ok(())
    }

    /// Opens an isolated SQLite connection locked into the tenant's container path.
    pub fn open_isolated_connection(&self) -> Result<Connection, StorageQuotaError> {
        if !self.container_dir.exists() {
            fs::create_dir_all(&self.container_dir)
                .map_err(|e| StorageQuotaError::StorageIoError(e.to_string()))?;
        }

        let conn = Connection::open(&self.db_path)
            .map_err(|e| StorageQuotaError::DatabaseError(e.to_string()))?;

        // Enforce WAL mode and busy timeout for high-concurrency cloud isolation
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA busy_timeout = 5000;
             PRAGMA foreign_keys = ON;
             PRAGMA synchronous = NORMAL;",
        )
        .map_err(|e| StorageQuotaError::DatabaseError(e.to_string()))?;

        Ok(conn)
    }
}

/// Multi-tenant storage manager managing isolated customer databases.
#[derive(Debug, Clone)]
pub struct TenantIsolationManager {
    pub storage_root: PathBuf,
    pub default_quota_bytes: u64,
}

impl TenantIsolationManager {
    pub fn new(storage_root: impl AsRef<Path>, default_quota_bytes: u64) -> Self {
        Self {
            storage_root: storage_root.as_ref().to_path_buf(),
            default_quota_bytes,
        }
    }

    /// Provisions or gets an isolated container for a tenant.
    pub fn provision_tenant(
        &self,
        tenant_id: &str,
        custom_quota_bytes: Option<u64>,
    ) -> Result<TenantContainer, StorageQuotaError> {
        TenantContainer::validate_tenant_id(tenant_id)?;

        let container_dir = self.storage_root.join("tenants").join(tenant_id);
        if !container_dir.exists() {
            fs::create_dir_all(&container_dir)
                .map_err(|e| StorageQuotaError::StorageIoError(e.to_string()))?;
        }

        let db_path = container_dir.join(format!("store_{}.db", tenant_id));
        let quota_bytes = custom_quota_bytes.unwrap_or(self.default_quota_bytes);

        Ok(TenantContainer {
            tenant_id: tenant_id.to_string(),
            container_dir,
            db_path,
            quota_bytes,
        })
    }

    /// Creates the global tenant isolation tracking table in the master registry DB.
    pub fn init_registry_schema(conn: &Connection) -> rusqlite::Result<()> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS tenant_isolation_registry (
                tenant_id TEXT PRIMARY KEY,
                quota_bytes INTEGER NOT NULL,
                storage_root_path TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'ACTIVE',
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_tenant_registry_status ON tenant_isolation_registry(status);",
        )
    }

    /// Registers a tenant provisioning record in the registry table.
    pub fn register_tenant(
        conn: &Connection,
        tenant_id: &str,
        quota_bytes: u64,
        storage_root_path: &str,
    ) -> rusqlite::Result<()> {
        conn.execute(
            "INSERT INTO tenant_isolation_registry (tenant_id, quota_bytes, storage_root_path, status)
             VALUES (?1, ?2, ?3, 'ACTIVE')
             ON CONFLICT(tenant_id) DO UPDATE SET quota_bytes = excluded.quota_bytes",
            params![tenant_id, quota_bytes as i64, storage_root_path],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tenant_id_validation_prevents_traversal() {
        assert!(TenantContainer::validate_tenant_id("acme_corp").is_ok());
        assert!(TenantContainer::validate_tenant_id("tenant-1234").is_ok());

        // Path traversal attempts
        assert!(TenantContainer::validate_tenant_id("../etc").is_err());
        assert!(TenantContainer::validate_tenant_id("..\\passwd").is_err());
        assert!(TenantContainer::validate_tenant_id("foo/bar").is_err());
        assert!(TenantContainer::validate_tenant_id("ab").is_err()); // Too short
        assert!(TenantContainer::validate_tenant_id("invalid!char").is_err());
    }

    #[test]
    fn test_quota_headroom_check() {
        let temp_dir = std::env::temp_dir().join("proteus_quota_test_headroom");
        let _ = fs::remove_dir_all(&temp_dir);

        let manager = TenantIsolationManager::new(&temp_dir, 1024 * 1024); // 1 MB quota
        let container = manager.provision_tenant("client_test", None).unwrap();

        // 500 KB fits
        assert!(container.check_headroom(500 * 1024).is_ok());

        // 2 MB exceeds 1 MB quota
        let err = container.check_headroom(2 * 1024 * 1024).unwrap_err();
        match err {
            StorageQuotaError::QuotaExceeded { tenant_id, quota_bytes, .. } => {
                assert_eq!(tenant_id, "client_test");
                assert_eq!(quota_bytes, 1024 * 1024);
            }
            _ => panic!("Expected QuotaExceeded error"),
        }

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_isolated_database_connections() {
        let temp_dir = std::env::temp_dir().join("proteus_tenant_isolation_db");
        let _ = fs::remove_dir_all(&temp_dir);

        let manager = TenantIsolationManager::new(&temp_dir, DEFAULT_TENANT_QUOTA_BYTES);
        let tenant_a = manager.provision_tenant("client_alpha", None).unwrap();
        let tenant_b = manager.provision_tenant("client_beta", None).unwrap();

        // Connect to Tenant A and create a table
        let conn_a = tenant_a.open_isolated_connection().unwrap();
        conn_a.execute("CREATE TABLE alpha_secrets (id INTEGER PRIMARY KEY, secret TEXT);", []).unwrap();
        conn_a.execute("INSERT INTO alpha_secrets (secret) VALUES ('secret_a');", []).unwrap();

        // Connect to Tenant B and ensure alpha_secrets does NOT exist
        let conn_b = tenant_b.open_isolated_connection().unwrap();
        let table_exists: bool = conn_b
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='alpha_secrets'",
                [],
                |r| r.get::<_, i64>(0),
            )
            .map(|count| count > 0)
            .unwrap();

        assert!(!table_exists, "Tenant B must not see Tenant A's tables (zero leakage)");

        // Check storage usage report for Tenant A
        let usage_a = tenant_a.get_storage_usage().unwrap();
        assert_eq!(usage_a.tenant_id, "client_alpha");
        assert!(usage_a.database_bytes > 0);
        assert_eq!(usage_a.quota_bytes, DEFAULT_TENANT_QUOTA_BYTES);
        assert!(!usage_a.is_exceeded);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_tenant_registry_schema_and_registration() {
        let conn = Connection::open_in_memory().unwrap();
        TenantIsolationManager::init_registry_schema(&conn).unwrap();

        TenantIsolationManager::register_tenant(
            &conn,
            "tenant_omega",
            DEFAULT_TENANT_QUOTA_BYTES,
            "/var/proteus/storage/tenants/tenant_omega",
        )
        .unwrap();

        let (quota, status): (i64, String) = conn
            .query_row(
                "SELECT quota_bytes, status FROM tenant_isolation_registry WHERE tenant_id = 'tenant_omega'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();

        assert_eq!(quota, DEFAULT_TENANT_QUOTA_BYTES as i64);
        assert_eq!(status, "ACTIVE");
    }
}
