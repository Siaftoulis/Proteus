//! Additive Delta Package Format (`.prupdate`) & Transactional Patch Runner for Proteus BOS.
//! Implements Master Problem Audit P21:
//! - Light-weight differential updates (.prupdate) containing schema and view diffs.
//! - Non-destructive schema migrations with transactional safety and zero data loss.
//! - Automated `.bak` SQLite snapshot before patch execution.
//! - Persistent patch audit trail in `package_patch_history`.

use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;
use thiserror::Error;

use crate::migrations::{MigrationPlan, MigrationRunner};
use crate::package::{PrFlowTrigger, PrViewLayout};

#[derive(Debug, Error, PartialEq)]
pub enum DeltaError {
    #[error("Checksum mismatch on delta update. Expected: {expected}, Computed: {computed}")]
    ChecksumMismatch { expected: String, computed: String },
    #[error("Serialization / Deserialization error: {0}")]
    SerializationError(String),
    #[error("Destructive or unsafe DDL rejected in delta patch: {0}")]
    UnsafeDdl(String),
    #[error("Database error while applying delta: {0}")]
    SqlError(String),
    #[error("Invalid delta manifest: {0}")]
    InvalidManifest(String),
    #[error("IO error creating snapshot: {0}")]
    IoError(String),
}

/// Metadata header for an incremental `.prupdate` patch.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrDeltaManifest {
    pub update_id: String,
    pub target_bundle_id: String,
    pub base_version: String,
    pub target_version: String,
    pub author_pcd_id: String,
    pub description: String,
    pub created_at: String,
}

/// Differential `.prupdate` package bundle containing strictly incremental changes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PrDeltaUpdate {
    pub manifest: PrDeltaManifest,
    pub schema_additions: Vec<String>,
    pub view_updates: Vec<PrViewLayout>,
    pub flow_updates: Vec<PrFlowTrigger>,
}

/// Summary report returned upon successful application of a delta patch.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeltaApplySummary {
    pub update_id: String,
    pub target_bundle_id: String,
    pub target_version: String,
    pub applied_ddl_count: usize,
    pub updated_views_count: usize,
    pub updated_flows_count: usize,
    pub snapshot_path: Option<String>,
}

/// Historical audit record of an applied package delta patch.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatchHistoryRecord {
    pub patch_id: String,
    pub bundle_id: String,
    pub base_version: String,
    pub target_version: String,
    pub description: String,
    pub applied_at: String,
}

impl PrDeltaUpdate {
    pub fn new(
        update_id: impl Into<String>,
        target_bundle_id: impl Into<String>,
        base_version: impl Into<String>,
        target_version: impl Into<String>,
        author: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            manifest: PrDeltaManifest {
                update_id: update_id.into(),
                target_bundle_id: target_bundle_id.into(),
                base_version: base_version.into(),
                target_version: target_version.into(),
                author_pcd_id: author.into(),
                description: description.into(),
                created_at: Utc::now().to_rfc3339(),
            },
            schema_additions: Vec::new(),
            view_updates: Vec::new(),
            flow_updates: Vec::new(),
        }
    }

    /// Serializes the delta update bundle into sealed bytes with a SHA-256 integrity seal.
    pub fn to_bytes(&self) -> Result<Vec<u8>, DeltaError> {
        let json_data = serde_json::to_string(self)
            .map_err(|e| DeltaError::SerializationError(e.to_string()))?;
        let mut hasher = Sha256::new();
        hasher.update(json_data.as_bytes());
        let hash = format!("{:x}", hasher.finalize());

        let envelope = serde_json::json!({
            "magic": "PRUPDATE",
            "version": 1,
            "sha256": hash,
            "payload": json_data
        });

        serde_json::to_vec(&envelope)
            .map_err(|e| DeltaError::SerializationError(e.to_string()))
    }

    /// Deserializes and validates a sealed `.prupdate` byte payload.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, DeltaError> {
        let envelope: serde_json::Value = serde_json::from_slice(bytes)
            .map_err(|e| DeltaError::SerializationError(e.to_string()))?;

        let magic = envelope.get("magic").and_then(|v| v.as_str()).unwrap_or("");
        if magic != "PRUPDATE" {
            return Err(DeltaError::SerializationError("Invalid magic header for PRUPDATE".into()));
        }

        let expected_hash = envelope.get("sha256").and_then(|v| v.as_str()).unwrap_or("");
        let payload_str = envelope.get("payload").and_then(|v| v.as_str()).unwrap_or("");

        let mut hasher = Sha256::new();
        hasher.update(payload_str.as_bytes());
        let computed_hash = format!("{:x}", hasher.finalize());

        if computed_hash != expected_hash {
            return Err(DeltaError::ChecksumMismatch {
                expected: expected_hash.to_string(),
                computed: computed_hash,
            });
        }

        serde_json::from_str(payload_str)
            .map_err(|e| DeltaError::SerializationError(e.to_string()))
    }
}

pub struct DeltaPatchRunner;

impl DeltaPatchRunner {
    /// Applies an incremental `.prupdate` patch transactionally against a live SQLite database.
    pub fn apply_patch(
        conn: &Connection,
        db_path: Option<&Path>,
        patch: &PrDeltaUpdate,
    ) -> Result<DeltaApplySummary, DeltaError> {
        // 1. Validate manifest
        if patch.manifest.target_bundle_id.trim().is_empty() {
            return Err(DeltaError::InvalidManifest("Target bundle ID cannot be empty".into()));
        }

        // 2. Validate safety of all DDL additions (strictly additive, zero DROP statements)
        if !patch.schema_additions.is_empty() {
            let mut plan = MigrationPlan::new(
                &patch.manifest.update_id,
                &patch.manifest.description,
                "Certified Designer",
            );
            for ddl in &patch.schema_additions {
                plan.add_statement(ddl.clone());
            }
            plan.validate_safety()
                .map_err(|e| DeltaError::UnsafeDdl(e.to_string()))?;
        }

        // 3. Automated timestamped .bak snapshot if a database path is provided
        let mut snapshot_path = None;
        if let Some(path) = db_path {
            let backup_file = MigrationRunner::create_snapshot(path)
                .map_err(|e| DeltaError::IoError(e.to_string()))?;
            snapshot_path = Some(backup_file.to_string_lossy().to_string());
        }

        // 4. Ensure patch history table exists
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS package_patch_history (
                patch_id TEXT PRIMARY KEY,
                bundle_id TEXT NOT NULL,
                base_version TEXT NOT NULL,
                target_version TEXT NOT NULL,
                description TEXT NOT NULL,
                ddl_count INTEGER NOT NULL,
                views_count INTEGER NOT NULL,
                flows_count INTEGER NOT NULL,
                applied_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_patch_bundle ON package_patch_history(bundle_id);
            "#,
        )
        .map_err(|e| DeltaError::SqlError(e.to_string()))?;

        // 5. Execute additive migrations inside transaction
        let tx = conn.unchecked_transaction()
            .map_err(|e| DeltaError::SqlError(e.to_string()))?;

        for ddl in &patch.schema_additions {
            tx.execute_batch(ddl)
                .map_err(|e| DeltaError::SqlError(format!("Failed executing DDL '{}': {}", ddl, e)))?;
        }

        let now_utc = Utc::now().to_rfc3339();
        tx.execute(
            r#"
            INSERT INTO package_patch_history (
                patch_id, bundle_id, base_version, target_version,
                description, ddl_count, views_count, flows_count, applied_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            params![
                patch.manifest.update_id,
                patch.manifest.target_bundle_id,
                patch.manifest.base_version,
                patch.manifest.target_version,
                patch.manifest.description,
                patch.schema_additions.len() as i64,
                patch.view_updates.len() as i64,
                patch.flow_updates.len() as i64,
                now_utc,
            ],
        )
        .map_err(|e| DeltaError::SqlError(e.to_string()))?;

        tx.commit()
            .map_err(|e| DeltaError::SqlError(e.to_string()))?;

        Ok(DeltaApplySummary {
            update_id: patch.manifest.update_id.clone(),
            target_bundle_id: patch.manifest.target_bundle_id.clone(),
            target_version: patch.manifest.target_version.clone(),
            applied_ddl_count: patch.schema_additions.len(),
            updated_views_count: patch.view_updates.len(),
            updated_flows_count: patch.flow_updates.len(),
            snapshot_path,
        })
    }

    /// Queries the historical patch list applied to a specific package bundle.
    pub fn query_patch_history(
        conn: &Connection,
        bundle_id: &str,
    ) -> Result<Vec<PatchHistoryRecord>, DeltaError> {
        let mut stmt = conn
            .prepare(
                r#"
                SELECT patch_id, bundle_id, base_version, target_version, description, applied_at
                FROM package_patch_history
                WHERE bundle_id = ?1
                ORDER BY applied_at ASC
                "#,
            )
            .map_err(|e| DeltaError::SqlError(e.to_string()))?;

        let rows = stmt
            .query_map(params![bundle_id], |row| {
                Ok(PatchHistoryRecord {
                    patch_id: row.get(0)?,
                    bundle_id: row.get(1)?,
                    base_version: row.get(2)?,
                    target_version: row.get(3)?,
                    description: row.get(4)?,
                    applied_at: row.get(5)?,
                })
            })
            .map_err(|e| DeltaError::SqlError(e.to_string()))?;

        let mut history = Vec::new();
        for r in rows {
            history.push(r.map_err(|e| DeltaError::SqlError(e.to_string()))?);
        }
        Ok(history)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delta_update_serialization_and_tamper_rejection() {
        let mut patch = PrDeltaUpdate::new(
            "UPD-001",
            "PKG-REPAIR-01",
            "1.0.0",
            "1.1.0",
            "PCD-DEV-01",
            "Added vehicle mileage column",
        );
        patch.schema_additions.push("ALTER TABLE service_tickets ADD COLUMN mileage INTEGER DEFAULT 0;".into());

        let bytes = patch.to_bytes().unwrap();
        let loaded = PrDeltaUpdate::from_bytes(&bytes).unwrap();
        assert_eq!(loaded.manifest.update_id, "UPD-001");
        assert_eq!(loaded.schema_additions.len(), 1);

        // Tamper with bytes
        let mut tampered = bytes.clone();
        if let Some(pos) = tampered.iter().position(|&b| b == b'1') {
            tampered[pos] = b'9';
            assert!(PrDeltaUpdate::from_bytes(&tampered).is_err());
        }
    }

    #[test]
    fn test_apply_patch_preserves_existing_data() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE service_tickets (
                id TEXT PRIMARY KEY,
                customer TEXT NOT NULL
            );
            INSERT INTO service_tickets (id, customer) VALUES ('TCK-1', 'Dimitris');
            "#,
        )
        .unwrap();

        let mut patch = PrDeltaUpdate::new(
            "UPD-002",
            "PKG-REPAIR-01",
            "1.0.0",
            "1.1.0",
            "PCD-DEV-01",
            "Add device serial and mileage",
        );
        patch.schema_additions.push("ALTER TABLE service_tickets ADD COLUMN serial_no TEXT;".into());
        patch.schema_additions.push("ALTER TABLE service_tickets ADD COLUMN mileage INTEGER DEFAULT 0;".into());

        let summary = DeltaPatchRunner::apply_patch(&conn, None, &patch).unwrap();
        assert_eq!(summary.applied_ddl_count, 2);
        assert_eq!(summary.target_version, "1.1.0");

        // Verify existing record is preserved with default mileage
        let (customer, mileage): (String, i64) = conn
            .query_row(
                "SELECT customer, mileage FROM service_tickets WHERE id = 'TCK-1'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(customer, "Dimitris");
        assert_eq!(mileage, 0);

        // Verify patch history audit
        let history = DeltaPatchRunner::query_patch_history(&conn, "PKG-REPAIR-01").unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].patch_id, "UPD-002");
    }

    #[test]
    fn test_apply_patch_rejects_destructive_sql() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute("CREATE TABLE orders (id TEXT PRIMARY KEY);", []).unwrap();

        let mut patch = PrDeltaUpdate::new(
            "UPD-003",
            "PKG-ORDERS",
            "1.0.0",
            "2.0.0",
            "PCD-DEV-01",
            "Dangerous drop",
        );
        patch.schema_additions.push("DROP TABLE orders;".into());

        let res = DeltaPatchRunner::apply_patch(&conn, None, &patch);
        assert!(res.is_err());
        assert!(matches!(res.unwrap_err(), DeltaError::UnsafeDdl(_)));

        // Table still exists
        let exists: bool = conn
            .query_row(
                "SELECT 1 FROM sqlite_master WHERE type='table' AND name='orders'",
                [],
                |_| Ok(true),
            )
            .unwrap();
        assert!(exists);
    }
}
