//! Additive-only Schema Migration Runner & Automated Snapshot Engine for Proteus.
//! Enforces zero-data-loss invariants:
//! - Strictly permits additive schema changes (CREATE TABLE, ALTER TABLE ADD COLUMN, CREATE INDEX).
//! - Statically and dynamically rejects destructive SQL operations (DROP TABLE, DROP COLUMN, TRUNCATE).
//! - Automatically creates a timestamped `.bak` SQLite snapshot before migration execution.
//! - Provides sandboxed dry-run validation with automatic rollback.

use chrono::Utc;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Instant;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum MigrationError {
    #[error("Destructive SQL operation rejected: '{0}'. Proteus allows only additive schema migrations.")]
    DestructiveSqlRejected(String),
    #[error("Database error during migration: {0}")]
    SqlError(String),
    #[error("IO error during backup snapshot: {0}")]
    IoError(String),
    #[error("Migration plan is empty")]
    EmptyPlan,
}

/// Represents an additive schema migration plan.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MigrationPlan {
    pub id: String,
    pub description: String,
    pub author_role: String,
    pub statements: Vec<String>,
}

impl MigrationPlan {
    pub fn new(id: impl Into<String>, desc: impl Into<String>, role: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            description: desc.into(),
            author_role: role.into(),
            statements: Vec::new(),
        }
    }

    pub fn add_statement(&mut self, sql: impl Into<String>) {
        self.statements.push(sql.into());
    }

    /// Verifies that all statements in this plan are strictly additive and non-destructive.
    pub fn validate_safety(&self) -> Result<(), MigrationError> {
        if self.statements.is_empty() {
            return Err(MigrationError::EmptyPlan);
        }

        let forbidden_patterns = [
            "DROP TABLE",
            "DROP COLUMN",
            "DROP VIEW",
            "DROP INDEX",
            "TRUNCATE",
            "DELETE FROM",
            "ALTER TABLE DROP",
        ];

        for stmt in &self.statements {
            let normalized = stmt.to_uppercase().replace(['\n', '\r', '\t'], " ");
            for forbidden in forbidden_patterns {
                if normalized.contains(forbidden) {
                    return Err(MigrationError::DestructiveSqlRejected(stmt.clone()));
                }
            }

            // Must begin with an allowed additive operation
            let trimmed = normalized.trim();
            let is_allowed = trimmed.starts_with("CREATE TABLE")
                || trimmed.starts_with("CREATE INDEX")
                || trimmed.starts_with("CREATE UNIQUE INDEX")
                || (trimmed.starts_with("ALTER TABLE") && trimmed.contains("ADD COLUMN"))
                || (trimmed.starts_with("ALTER TABLE") && trimmed.contains("ADD "));

            if !is_allowed {
                return Err(MigrationError::DestructiveSqlRejected(stmt.clone()));
            }
        }

        Ok(())
    }
}

/// Result of an executed migration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MigrationResult {
    pub migration_id: String,
    pub executed_statements_count: usize,
    pub snapshot_path: Option<String>,
    pub elapsed_ms: u64,
}

pub struct MigrationRunner;

impl MigrationRunner {
    /// Executes the migration inside a temporary transaction that is unconditionally rolled back.
    /// Used for pre-flight validation to ensure the SQL syntax is valid on SQLite.
    pub fn dry_run(conn: &mut Connection, plan: &MigrationPlan) -> Result<(), MigrationError> {
        plan.validate_safety()?;

        let tx = conn.transaction().map_err(|e| MigrationError::SqlError(e.to_string()))?;
        for stmt in &plan.statements {
            tx.execute_batch(stmt)
                .map_err(|e| MigrationError::SqlError(format!("Dry-run error on '{}': {}", stmt, e)))?;
        }
        // Rolling back by explicitly dropping or not committing
        drop(tx);
        Ok(())
    }

    /// Creates an atomic hot snapshot of the target SQLite database file.
    pub fn create_snapshot(db_path: &Path) -> Result<PathBuf, MigrationError> {
        if !db_path.exists() {
            return Err(MigrationError::IoError(format!("Database file '{}' does not exist", db_path.display())));
        }

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let bak_file_name = format!(
            "{}.bak_{}",
            db_path.file_name().and_then(|f| f.to_str()).unwrap_or("store.db"),
            timestamp
        );
        let bak_path = db_path.with_file_name(bak_file_name);

        std::fs::copy(db_path, &bak_path)
            .map_err(|e| MigrationError::IoError(format!("Failed to create backup snapshot: {}", e)))?;

        Ok(bak_path)
    }

    /// Executes the migration plan on the active SQLite database with an automated snapshot.
    pub fn execute(
        conn: &mut Connection,
        plan: &MigrationPlan,
        db_path: Option<&Path>,
    ) -> Result<MigrationResult, MigrationError> {
        plan.validate_safety()?;

        let start = Instant::now();

        // 1. Take snapshot if file path is provided and exists
        let snapshot_path_str = if let Some(path) = db_path {
            if path.exists() {
                let bak_path = Self::create_snapshot(path)?;
                Some(bak_path.to_string_lossy().to_string())
            } else {
                None
            }
        } else {
            None
        };

        // 2. Execute statements in a transactional block
        let tx = conn.transaction().map_err(|e| MigrationError::SqlError(e.to_string()))?;
        for stmt in &plan.statements {
            tx.execute_batch(stmt)
                .map_err(|e| MigrationError::SqlError(format!("Execution failed on '{}': {}", stmt, e)))?;
        }
        tx.commit().map_err(|e| MigrationError::SqlError(e.to_string()))?;

        let elapsed = start.elapsed().as_millis() as u64;

        Ok(MigrationResult {
            migration_id: plan.id.clone(),
            executed_statements_count: plan.statements.len(),
            snapshot_path: snapshot_path_str,
            elapsed_ms: elapsed,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_additive_plan_validation() {
        let mut plan = MigrationPlan::new("M-01", "Add Fleet Telematics", "Developer");
        plan.add_statement("CREATE TABLE IF NOT EXISTS fleet_vehicles (id TEXT PRIMARY KEY, plate TEXT NOT NULL);");
        plan.add_statement("ALTER TABLE fleet_vehicles ADD COLUMN battery_health REAL;");
        plan.add_statement("CREATE INDEX idx_fleet_plate ON fleet_vehicles(plate);");

        assert!(plan.validate_safety().is_ok());
    }

    #[test]
    fn test_rejects_drop_table() {
        let mut plan = MigrationPlan::new("M-MALICIOUS", "Drop Customers", "Attacker");
        plan.add_statement("DROP TABLE service_tickets;");

        let res = plan.validate_safety();
        assert!(matches!(res, Err(MigrationError::DestructiveSqlRejected(_))));
    }

    #[test]
    fn test_rejects_drop_column() {
        let mut plan = MigrationPlan::new("M-DROP-COL", "Remove column", "Dev");
        plan.add_statement("ALTER TABLE service_tickets DROP COLUMN customer_phone;");

        let res = plan.validate_safety();
        assert!(matches!(res, Err(MigrationError::DestructiveSqlRejected(_))));
    }

    #[test]
    fn test_rejects_truncate() {
        let mut plan = MigrationPlan::new("M-TRUNCATE", "Clear all data", "Operator");
        plan.add_statement("TRUNCATE service_tickets;");

        let res = plan.validate_safety();
        assert!(matches!(res, Err(MigrationError::DestructiveSqlRejected(_))));
    }

    #[test]
    fn test_dry_run_rolls_back_changes() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute("CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT);", []).unwrap();

        let mut plan = MigrationPlan::new("M-DRY", "Add email", "Developer");
        plan.add_statement("ALTER TABLE users ADD COLUMN email TEXT;");

        // Dry run must succeed
        assert!(MigrationRunner::dry_run(&mut conn, &plan).is_ok());

        // The column should NOT exist after dry-run
        let err = conn.execute("SELECT email FROM users;", []);
        assert!(err.is_err());
    }

    #[test]
    fn test_real_migration_execution() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute("CREATE TABLE inventory (sku TEXT PRIMARY KEY);", []).unwrap();

        let mut plan = MigrationPlan::new("M-REAL", "Expand Inventory", "Developer");
        plan.add_statement("ALTER TABLE inventory ADD COLUMN barcode TEXT;");
        plan.add_statement("CREATE TABLE suppliers (id TEXT PRIMARY KEY, name TEXT);");

        let res = MigrationRunner::execute(&mut conn, &plan, None).unwrap();
        assert_eq!(res.executed_statements_count, 2);

        // Verification: query the newly created column and table
        conn.execute("SELECT barcode FROM inventory;", []).unwrap();
        conn.execute("SELECT id, name FROM suppliers;", []).unwrap();
    }
}
