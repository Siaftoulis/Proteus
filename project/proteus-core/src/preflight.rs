//! Proteus Sovereign BOS — Pre-Flight Diagnostic & Self-Healing Engine
//!
//! Autonomous runtime environment validation enforcing the "Average Joe" principle:
//! - SQLite integrity checks (WAL mode, PRAGMA integrity_check, foreign keys).
//! - Required schema table verification.
//! - Self-healing default configuration generation if proteus.toml is missing.
//! - Network port availability diagnostics for LAN mesh and local services.

use chrono::Utc;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::fs;
use std::net::TcpListener;
use std::path::Path;

/// Embedded default configuration used for zero-configuration self-healing on first launch.
pub const EMBEDDED_DEFAULT_CONFIG: &str = r#"# Proteus Sovereign BOS — Default Self-Healed Configuration
[store]
company_name = "Ελληνική Εμπορική Επιχείρηση"
tin = "000000000"
branch_id = 0
address = "Ερμού 1, Αθήνα"
postal_code = "10563"
phone = "+30 210 0000000"

[database]
path = "data/store.db"
wal_mode = true
busy_timeout_ms = 5000

[mydata]
environment = "development"
auto_transmit = true
retry_interval_secs = 300

[pos]
default_payment_method = "Card"
require_shift_clockin = true
enforce_eftpos_pairing = false

[mesh]
enabled = true
branch_name = "Branch-Central"
bind_port = 7474
"#;

/// Essential database tables required for core sovereign operations.
pub const REQUIRED_SYSTEM_TABLES: &[&str] = &[
    "shop_settings",
    "audit_logs",
    "invoices",
    "cardex_entities",
    "work_card_events",
    "pos_quick_items",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqliteCheckResult {
    pub integrity_ok: bool,
    pub integrity_message: String,
    pub journal_mode: String,
    pub foreign_keys_enabled: bool,
    pub verified_tables: Vec<String>,
    pub missing_tables: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigCheckResult {
    pub exists: bool,
    pub path: String,
    pub self_healed: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortCheckResult {
    pub port: u16,
    pub available: bool,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreflightReport {
    pub timestamp: String,
    pub overall_pass: bool,
    pub sqlite: SqliteCheckResult,
    pub config: ConfigCheckResult,
    pub ports: Vec<PortCheckResult>,
    pub remediation_notes: Vec<String>,
}

/// Validates SQLite database integrity, WAL journal mode, and essential tables.
pub fn check_sqlite_integrity(conn: &Connection) -> Result<SqliteCheckResult, rusqlite::Error> {
    // 1. PRAGMA integrity_check
    let integrity_message: String = conn
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .unwrap_or_else(|e| format!("Error checking integrity: {}", e));
    let integrity_ok = integrity_message.trim().eq_ignore_ascii_case("ok");

    // 2. PRAGMA journal_mode
    let journal_mode: String = conn
        .query_row("PRAGMA journal_mode", [], |row| row.get(0))
        .unwrap_or_else(|_| "unknown".to_string());

    // 3. PRAGMA foreign_keys
    let foreign_keys_enabled: bool = conn
        .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
        .unwrap_or(false);

    // 4. Required tables verification
    let mut verified_tables = Vec::new();
    let mut missing_tables = Vec::new();

    for table in REQUIRED_SYSTEM_TABLES {
        let exists: bool = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name=?1",
                [table],
                |row| row.get::<_, i64>(0).map(|c| c > 0),
            )
            .unwrap_or(false);

        if exists {
            verified_tables.push(table.to_string());
        } else {
            missing_tables.push(table.to_string());
        }
    }

    Ok(SqliteCheckResult {
        integrity_ok,
        integrity_message,
        journal_mode,
        foreign_keys_enabled,
        verified_tables,
        missing_tables,
    })
}

/// Verifies presence of proteus.toml, auto-generating it if missing (self-healing).
pub fn check_and_heal_configuration(config_path: &Path) -> Result<ConfigCheckResult, std::io::Error> {
    let path_str = config_path.to_string_lossy().to_string();

    if config_path.exists() {
        let metadata = fs::metadata(config_path)?;
        if metadata.len() > 0 {
            return Ok(ConfigCheckResult {
                exists: true,
                path: path_str,
                self_healed: false,
                message: "Configuration file exists and is readable.".to_string(),
            });
        }
    }

    // Auto-create parent directory and file if missing
    if let Some(parent) = config_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    fs::write(config_path, EMBEDDED_DEFAULT_CONFIG)?;

    Ok(ConfigCheckResult {
        exists: true,
        path: path_str,
        self_healed: true,
        message: "Configuration was missing and has been automatically self-healed.".to_string(),
    })
}

/// Checks TCP port availability for system services and mesh coordination.
pub fn check_tcp_port_available(port: u16, description: &str) -> PortCheckResult {
    let available = match TcpListener::bind(("127.0.0.1", port)) {
        Ok(listener) => {
            drop(listener);
            true
        }
        Err(_) => false,
    };

    PortCheckResult {
        port,
        available,
        description: description.to_string(),
    }
}

/// Comprehensive pre-flight health diagnostic executing all checks and compiling remediation actions.
pub fn run_preflight_diagnostics(conn: &Connection, config_path: &Path) -> PreflightReport {
    let mut remediation_notes = Vec::new();

    // 1. SQLite Check
    let sqlite_res = check_sqlite_integrity(conn).unwrap_or_else(|e| SqliteCheckResult {
        integrity_ok: false,
        integrity_message: format!("Connection error: {}", e),
        journal_mode: "error".to_string(),
        foreign_keys_enabled: false,
        verified_tables: vec![],
        missing_tables: vec!["all".to_string()],
    });

    if !sqlite_res.integrity_ok {
        remediation_notes.push(format!("SQLite Integrity Check Failed: {}", sqlite_res.integrity_message));
    }
    if !sqlite_res.missing_tables.is_empty() {
        remediation_notes.push(format!(
            "Missing {} critical database tables: {:?}",
            sqlite_res.missing_tables.len(),
            sqlite_res.missing_tables
        ));
    }

    // 2. Config Check & Self-Healing
    let config_res = check_and_heal_configuration(config_path).unwrap_or_else(|e| ConfigCheckResult {
        exists: false,
        path: config_path.to_string_lossy().to_string(),
        self_healed: false,
        message: format!("Failed to access or self-heal config: {}", e),
    });

    if config_res.self_healed {
        remediation_notes.push("Default configuration proteus.toml was generated automatically.".to_string());
    }

    // 3. Port Diagnostics (Mesh: 7474, EFT/POS: 8080)
    let ports = vec![
        check_tcp_port_available(7474, "Mesh Peer Discovery TCP"),
        check_tcp_port_available(8080, "EFT/POS Terminal TCP Link"),
    ];

    let overall_pass = sqlite_res.integrity_ok && sqlite_res.missing_tables.is_empty() && config_res.exists;

    PreflightReport {
        timestamp: Utc::now().to_rfc3339(),
        overall_pass,
        sqlite: sqlite_res,
        config: config_res,
        ports,
        remediation_notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preflight_sqlite_integrity_and_tables() {
        let conn = Connection::open_in_memory().unwrap();
        let _ = crate::apply_storage_tuning(&conn);
        let _ = crate::bootstrap::bootstrap_store_database(&conn);
        let _ = crate::cardex::init_cardex_schema(&conn);
        let _ = crate::work_card::init_work_card_schema(&conn);

        let res = check_sqlite_integrity(&conn).unwrap();
        assert!(res.integrity_ok, "Integrity check must pass");
        assert_eq!(res.missing_tables.len(), 0, "All required tables should exist after bootstrap");
        assert!(res.verified_tables.contains(&"shop_settings".to_string()));
        assert!(res.verified_tables.contains(&"invoices".to_string()));
        assert!(res.verified_tables.contains(&"cardex_entities".to_string()));
        assert!(res.verified_tables.contains(&"work_card_events".to_string()));
        assert!(res.verified_tables.contains(&"pos_quick_items".to_string()));
    }

    #[test]
    fn test_config_self_healing_missing_file() {
        let temp_dir = std::env::temp_dir().join(format!("proteus_test_preflight_{}", uuid::Uuid::new_v4()));
        let config_file = temp_dir.join("config").join("proteus.toml");

        assert!(!config_file.exists());
        let res = check_and_heal_configuration(&config_file).unwrap();
        assert!(res.exists);
        assert!(res.self_healed, "Should report self-healed when missing");
        assert!(config_file.exists(), "File should be created by self-healing");

        // Second run should report already existing without self-healing
        let second_run = check_and_heal_configuration(&config_file).unwrap();
        assert!(second_run.exists);
        assert!(!second_run.self_healed, "Already exists, no self-healing needed");

        // Clean up
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_preflight_full_diagnostics_report() {
        let conn = Connection::open_in_memory().unwrap();
        let _ = crate::apply_storage_tuning(&conn);
        let _ = crate::bootstrap::bootstrap_store_database(&conn);
        let _ = crate::cardex::init_cardex_schema(&conn);
        let _ = crate::work_card::init_work_card_schema(&conn);

        let temp_dir = std::env::temp_dir().join(format!("proteus_report_test_{}", uuid::Uuid::new_v4()));
        let config_file = temp_dir.join("proteus.toml");

        let report = run_preflight_diagnostics(&conn, &config_file);
        assert!(report.overall_pass, "Overall pass should be true when DB is bootstrapped");
        assert!(report.sqlite.integrity_ok);
        assert_eq!(report.ports.len(), 2);
        assert!(report.config.self_healed);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
