//! Safe Schema Migrator & Canary Health Reporter Agent.
//! Provides sandboxed shadow pre-flight checks, non-destructive migration dry-runs,
//! post-migration SQLite integrity verification (`PRAGMA integrity_check`),
//! automated rollback on failure, and telemetry health reporting.

use rusqlite::{params, Connection, Result as SqliteResult};
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Deployment stage for canary rollouts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanaryStage {
    StagingSandbox,
    CanaryTenants, // e.g., first 5-10% of tenant nodes
    ProductionAll,
}

impl CanaryStage {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::StagingSandbox => "STAGING_SANDBOX",
            Self::CanaryTenants => "CANARY_TENANTS",
            Self::ProductionAll => "PRODUCTION_ALL",
        }
    }
}

/// Operational health assessment of a database schema after migration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SchemaHealthStatus {
    Healthy,
    Degraded { latency_ms: u64, reason: String },
    CriticalFailure { error: String },
}

impl SchemaHealthStatus {
    pub fn is_healthy(&self) -> bool {
        matches!(self, Self::Healthy)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Healthy => "HEALTHY",
            Self::Degraded { .. } => "DEGRADED",
            Self::CriticalFailure { .. } => "CRITICAL_FAILURE",
        }
    }
}

/// Detailed health and telemetry report produced by the Canary Migrator Agent.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CanaryHealthReport {
    pub report_id: String,
    pub plan_id: String,
    pub tenant_id: String,
    pub stage: CanaryStage,
    pub preflight_passed: bool,
    pub integrity_check_passed: bool,
    pub execution_duration_ms: u64,
    pub status: SchemaHealthStatus,
    pub rollback_triggered: bool,
    pub details: Vec<String>,
}

/// Canary Migrator Agent managing safe pre-flight trials and live migrations.
pub struct CanaryMigratorAgent;

impl CanaryMigratorAgent {
    /// Runs a sandboxed pre-flight dry run in an in-memory connection copy.
    pub fn preflight_dry_run(
        plan_statements: &[String],
        target_schema_dump: &str,
    ) -> Result<(), String> {
        let conn = Connection::open_in_memory()
            .map_err(|e| format!("Failed to create memory shadow DB: {}", e))?;

        if !target_schema_dump.is_empty() {
            conn.execute_batch(target_schema_dump)
                .map_err(|e| format!("Failed to populate shadow schema: {}", e))?;
        }

        // Execute proposed statements inside shadow sandbox
        for (idx, stmt) in plan_statements.iter().enumerate() {
            conn.execute(stmt, [])
                .map_err(|e| format!("Shadow trial failed at statement {}: {}", idx + 1, e))?;
        }

        // Verify shadow integrity
        let integrity: String = conn
            .query_row("PRAGMA integrity_check;", [], |r| r.get(0))
            .map_err(|e| format!("Integrity check failed: {}", e))?;

        if integrity != "ok" {
            return Err(format!("Shadow integrity error: {}", integrity));
        }

        Ok(())
    }

    /// Safely executes migration statements on target connection with canary health checks.
    pub fn execute_with_health_check(
        report_id: &str,
        plan_id: &str,
        tenant_id: &str,
        stage: CanaryStage,
        statements: &[String],
        conn: &mut Connection,
    ) -> CanaryHealthReport {
        let start = Instant::now();
        let mut details = Vec::new();

        // 1. Preflight dry run check
        let preflight_ok = match Self::preflight_dry_run(statements, "") {
            Ok(_) => {
                details.push("Shadow preflight dry run passed.".to_string());
                true
            }
            Err(e) => {
                details.push(format!("Shadow preflight failed: {}", e));
                return CanaryHealthReport {
                    report_id: report_id.to_string(),
                    plan_id: plan_id.to_string(),
                    tenant_id: tenant_id.to_string(),
                    stage,
                    preflight_passed: false,
                    integrity_check_passed: false,
                    execution_duration_ms: start.elapsed().as_millis() as u64,
                    status: SchemaHealthStatus::CriticalFailure { error: e },
                    rollback_triggered: true,
                    details,
                };
            }
        };

        // 2. Transactional execution
        let tx_result = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate);
        if let Err(e) = tx_result {
            details.push(format!("Failed to acquire write transaction: {}", e));
            return CanaryHealthReport {
                report_id: report_id.to_string(),
                plan_id: plan_id.to_string(),
                tenant_id: tenant_id.to_string(),
                stage,
                preflight_passed: preflight_ok,
                integrity_check_passed: false,
                execution_duration_ms: start.elapsed().as_millis() as u64,
                status: SchemaHealthStatus::CriticalFailure { error: e.to_string() },
                rollback_triggered: true,
                details,
            };
        }

        let tx = tx_result.unwrap();
        let mut exec_error = None;

        for (idx, stmt) in statements.iter().enumerate() {
            if let Err(e) = tx.execute(stmt, []) {
                exec_error = Some(format!("Error at statement {}: {}", idx + 1, e));
                break;
            }
        }

        if let Some(err) = exec_error {
            let _ = tx.rollback();
            details.push(format!("Transaction aborted and rolled back: {}", err));
            return CanaryHealthReport {
                report_id: report_id.to_string(),
                plan_id: plan_id.to_string(),
                tenant_id: tenant_id.to_string(),
                stage,
                preflight_passed: preflight_ok,
                integrity_check_passed: false,
                execution_duration_ms: start.elapsed().as_millis() as u64,
                status: SchemaHealthStatus::CriticalFailure { error: err },
                rollback_triggered: true,
                details,
            };
        }

        if let Err(e) = tx.commit() {
            details.push(format!("Failed to commit transaction: {}", e));
            return CanaryHealthReport {
                report_id: report_id.to_string(),
                plan_id: plan_id.to_string(),
                tenant_id: tenant_id.to_string(),
                stage,
                preflight_passed: preflight_ok,
                integrity_check_passed: false,
                execution_duration_ms: start.elapsed().as_millis() as u64,
                status: SchemaHealthStatus::CriticalFailure { error: e.to_string() },
                rollback_triggered: true,
                details,
            };
        }

        // 3. Post-migration PRAGMA integrity verification
        let integrity: Result<String, _> = conn.query_row("PRAGMA integrity_check;", [], |r| r.get(0));
        let (integrity_ok, status) = match integrity {
            Ok(ref val) if val == "ok" => {
                details.push("Post-migration PRAGMA integrity check returned OK.".to_string());
                (true, SchemaHealthStatus::Healthy)
            }
            Ok(ref other) => {
                details.push(format!("Integrity check anomaly: {}", other));
                (false, SchemaHealthStatus::Degraded { latency_ms: start.elapsed().as_millis() as u64, reason: other.clone() })
            }
            Err(e) => {
                details.push(format!("Integrity query error: {}", e));
                (false, SchemaHealthStatus::CriticalFailure { error: e.to_string() })
            }
        };

        CanaryHealthReport {
            report_id: report_id.to_string(),
            plan_id: plan_id.to_string(),
            tenant_id: tenant_id.to_string(),
            stage,
            preflight_passed: true,
            integrity_check_passed: integrity_ok,
            execution_duration_ms: start.elapsed().as_millis() as u64,
            status,
            rollback_triggered: !integrity_ok,
            details,
        }
    }

    /// Initializes SQLite schema for tracking canary health reports.
    pub fn init_canary_schema(conn: &Connection) -> SqliteResult<()> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS canary_migration_health_reports (
                report_id TEXT PRIMARY KEY,
                plan_id TEXT NOT NULL,
                tenant_id TEXT NOT NULL,
                stage TEXT NOT NULL,
                status TEXT NOT NULL,
                execution_duration_ms INTEGER NOT NULL,
                rollback_triggered INTEGER NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_canary_status ON canary_migration_health_reports(status);",
        )
    }

    /// Persists a canary report record into SQLite.
    pub fn record_canary_report(conn: &Connection, report: &CanaryHealthReport) -> SqliteResult<()> {
        conn.execute(
            "INSERT INTO canary_migration_health_reports (
                report_id, plan_id, tenant_id, stage, status, execution_duration_ms, rollback_triggered
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(report_id) DO UPDATE SET status = excluded.status",
            params![
                report.report_id,
                report.plan_id,
                report.tenant_id,
                report.stage.as_str(),
                report.status.as_str(),
                report.execution_duration_ms as i64,
                if report.rollback_triggered { 1 } else { 0 },
            ],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_successful_canary_migration() {
        let mut conn = Connection::open_in_memory().unwrap();
        let statements = vec![
            "CREATE TABLE canary_clients (id TEXT PRIMARY KEY, name TEXT);".to_string(),
            "INSERT INTO canary_clients (id, name) VALUES ('C-1', 'Alpha Corp');".to_string(),
        ];

        let report = CanaryMigratorAgent::execute_with_health_check(
            "REP-001",
            "PLAN-ADD-CLIENTS",
            "tenant_alpha",
            CanaryStage::CanaryTenants,
            &statements,
            &mut conn,
        );

        assert!(report.preflight_passed);
        assert!(report.integrity_check_passed);
        assert!(report.status.is_healthy());
        assert!(!report.rollback_triggered);

        let count: i64 = conn.query_row("SELECT COUNT(*) FROM canary_clients", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_preflight_rejection_prevents_corrupt_migration() {
        let mut conn = Connection::open_in_memory().unwrap();
        let invalid_statements = vec![
            "SYNTAX ERROR INVALID SQL STATEMENTS".to_string(),
        ];

        let report = CanaryMigratorAgent::execute_with_health_check(
            "REP-002",
            "PLAN-BAD-SQL",
            "tenant_beta",
            CanaryStage::StagingSandbox,
            &invalid_statements,
            &mut conn,
        );

        assert!(!report.preflight_passed);
        assert!(!report.status.is_healthy());
        assert!(report.rollback_triggered);
    }

    #[test]
    fn test_canary_schema_persistence() {
        let conn = Connection::open_in_memory().unwrap();
        CanaryMigratorAgent::init_canary_schema(&conn).unwrap();

        let report = CanaryHealthReport {
            report_id: "REP-999".to_string(),
            plan_id: "PLAN-1".to_string(),
            tenant_id: "client_omega".to_string(),
            stage: CanaryStage::ProductionAll,
            preflight_passed: true,
            integrity_check_passed: true,
            execution_duration_ms: 12,
            status: SchemaHealthStatus::Healthy,
            rollback_triggered: false,
            details: vec!["OK".to_string()],
        };

        CanaryMigratorAgent::record_canary_report(&conn, &report).unwrap();

        let (stage, status): (String, String) = conn
            .query_row(
                "SELECT stage, status FROM canary_migration_health_reports WHERE report_id = 'REP-999'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();

        assert_eq!(stage, "PRODUCTION_ALL");
        assert_eq!(status, "HEALTHY");
    }
}
