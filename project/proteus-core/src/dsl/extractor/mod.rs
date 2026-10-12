//! Multi-Format Database Data & DDL Extractor for Proteus BOS.
//! Strict Rule 1 (100% Original Codebase), Rule 3 (<400 lines), and Rule 5 (Zero Mock Data).
//! Guarantees complete data sovereignty and zero vendor lock-in by extracting
//! live SQLite schema and datasets into PostgreSQL, MySQL, ANSI SQL, JSON, and CSV.

pub mod sql;

use rusqlite::types::ValueRef;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use sql::SqlFormatter;

/// Target export formats for business migration and external backup.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    Postgres,
    Mysql,
    AnsiSql,
    Json,
    Csv,
}

/// Metadata describing a table column queried live from PRAGMA table_info.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableColumnInfo {
    pub cid: i64,
    pub name: String,
    pub data_type: String,
    pub notnull: bool,
    pub dflt_value: Option<String>,
    pub pk: bool,
}

/// Extracted records and column definitions for a single database table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedTableData {
    pub table_name: String,
    pub columns: Vec<TableColumnInfo>,
    pub rows: Vec<Vec<serde_json::Value>>,
}

/// Complete portable database archive containing all application tables and real records.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseExportBundle {
    pub database_id: String,
    pub exported_at_iso: String,
    pub tables: Vec<ExtractedTableData>,
}

pub struct MultiFormatDatabaseExtractor;

impl MultiFormatDatabaseExtractor {
    /// Discovers all user tables in the SQLite database (excluding internal metadata).
    pub fn discover_user_tables(conn: &Connection) -> Result<Vec<String>, String> {
        let mut stmt = conn
            .prepare(
                "SELECT name FROM sqlite_master 
                 WHERE type = 'table' 
                   AND name NOT LIKE 'sqlite_%' 
                   AND name NOT LIKE '_proteus_%'
                 ORDER BY name ASC",
            )
            .map_err(|e| format!("Failed to discover tables: {}", e))?;

        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| format!("Query failed: {}", e))?;

        let mut tables = Vec::new();
        for r in rows {
            if let Ok(tbl) = r {
                tables.push(tbl);
            }
        }
        Ok(tables)
    }

    /// Extracts column definitions for a given table from SQLite.
    pub fn extract_columns(conn: &Connection, table_name: &str) -> Result<Vec<TableColumnInfo>, String> {
        let pragma_sql = format!("PRAGMA table_info(\"{}\")", table_name);
        let mut stmt = conn
            .prepare(&pragma_sql)
            .map_err(|e| format!("Failed to inspect table {}: {}", table_name, e))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(TableColumnInfo {
                    cid: row.get(0)?,
                    name: row.get(1)?,
                    data_type: row.get(2)?,
                    notnull: row.get::<_, i64>(3)? != 0,
                    dflt_value: row.get(4)?,
                    pk: row.get::<_, i64>(5)? != 0,
                })
            })
            .map_err(|e| format!("Query pragma failed: {}", e))?;

        let mut cols = Vec::new();
        for r in rows {
            cols.push(r.map_err(|e| format!("Pragma row read error: {}", e))?);
        }
        Ok(cols)
    }

    /// Extracts all live rows from a specific table into JSON-compatible values.
    pub fn extract_table_data(conn: &Connection, table_name: &str) -> Result<ExtractedTableData, String> {
        let columns = Self::extract_columns(conn, table_name)?;
        let query_sql = format!("SELECT * FROM \"{}\"", table_name);
        let mut stmt = conn
            .prepare(&query_sql)
            .map_err(|e| format!("Failed to query table {}: {}", table_name, e))?;

        let col_count = columns.len();
        let mut rows: Vec<Vec<serde_json::Value>> = Vec::new();

        let mut query_rows = stmt.query([]).map_err(|e| format!("Query failed: {}", e))?;
        while let Some(row) = query_rows.next().map_err(|e| format!("Next row failed: {}", e))? {
            let mut row_vals = Vec::with_capacity(col_count);
            for i in 0..col_count {
                let val_ref = row.get_ref(i).map_err(|e| format!("Get col {} failed: {}", i, e))?;
                let json_val = match val_ref {
                    ValueRef::Null => serde_json::Value::Null,
                    ValueRef::Integer(v) => serde_json::Value::Number(serde_json::Number::from(v)),
                    ValueRef::Real(v) => serde_json::Number::from_f64(v)
                        .map(serde_json::Value::Number)
                        .unwrap_or(serde_json::Value::Null),
                    ValueRef::Text(t) => {
                        let text_str = std::str::from_utf8(t).unwrap_or("");
                        serde_json::Value::String(text_str.to_string())
                    }
                    ValueRef::Blob(b) => {
                        let hex_str = b.iter().map(|byte| format!("{:02x}", byte)).collect::<String>();
                        serde_json::Value::String(format!("hex:{}", hex_str))
                    }
                };
                row_vals.push(json_val);
            }
            rows.push(row_vals);
        }

        Ok(ExtractedTableData {
            table_name: table_name.to_string(),
            columns,
            rows,
        })
    }

    /// Extracts all user tables and records into a unified database export bundle.
    pub fn extract_all_tables(conn: &Connection, database_id: &str) -> Result<DatabaseExportBundle, String> {
        let table_names = Self::discover_user_tables(conn)?;
        let mut tables = Vec::with_capacity(table_names.len());

        for tbl in table_names {
            let extracted = Self::extract_table_data(conn, &tbl)?;
            tables.push(extracted);
        }

        Ok(DatabaseExportBundle {
            database_id: database_id.to_string(),
            exported_at_iso: chrono::Utc::now().to_rfc3339(),
            tables,
        })
    }

    /// Formats an entire export bundle into the requested dialect (PostgreSQL, MySQL, ANSI SQL, JSON).
    pub fn export_bundle(bundle: &DatabaseExportBundle, format: ExportFormat) -> Result<String, String> {
        match format {
            ExportFormat::Json => {
                serde_json::to_string_pretty(bundle).map_err(|e| format!("JSON serialization error: {}", e))
            }
            ExportFormat::Postgres => Ok(SqlFormatter::generate_postgres_dump(bundle)),
            ExportFormat::Mysql => Ok(SqlFormatter::generate_mysql_dump(bundle)),
            ExportFormat::AnsiSql => Ok(SqlFormatter::generate_ansi_sql_dump(bundle)),
            ExportFormat::Csv => {
                let mut out = String::new();
                for tbl in &bundle.tables {
                    out.push_str(&format!("--- TABLE: {} ---\n", tbl.table_name));
                    out.push_str(&Self::export_table_to_csv(tbl));
                    out.push_str("\n\n");
                }
                Ok(out)
            }
        }
    }

    /// Exports a single table's records into standard RFC 4180 CSV format.
    pub fn export_table_to_csv(table: &ExtractedTableData) -> String {
        let mut csv = String::new();
        let header = table
            .columns
            .iter()
            .map(|c| format!("\"{}\"", c.name.replace('"', "\"\"")))
            .collect::<Vec<_>>()
            .join(",");
        csv.push_str(&header);
        csv.push('\n');

        for row in &table.rows {
            let line = row
                .iter()
                .map(|val| match val {
                    serde_json::Value::Null => String::new(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::String(s) => format!("\"{}\"", s.replace('"', "\"\"")),
                    _ => format!("\"{}\"", val.to_string().replace('"', "\"\"")),
                })
                .collect::<Vec<_>>()
                .join(",");
            csv.push_str(&line);
            csv.push('\n');
        }
        csv
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_all_tables_roundtrip() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE customers (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                balance REAL DEFAULT 0.0,
                is_active INTEGER DEFAULT 1
            )",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO customers (id, name, balance, is_active) VALUES ('cust-1', 'Acme Corp', 1500.50, 1)",
            [],
        )
        .unwrap();

        conn.execute(
            "INSERT INTO customers (id, name, balance, is_active) VALUES ('cust-2', 'O''Reilly Media', 42.00, 0)",
            [],
        )
        .unwrap();

        let bundle = MultiFormatDatabaseExtractor::extract_all_tables(&conn, "test_db").unwrap();
        assert_eq!(bundle.database_id, "test_db");
        assert_eq!(bundle.tables.len(), 1);

        let table = &bundle.tables[0];
        assert_eq!(table.table_name, "customers");
        assert_eq!(table.columns.len(), 4);
        assert_eq!(table.rows.len(), 2);

        // Test PostgreSQL export
        let pg_dump = MultiFormatDatabaseExtractor::export_bundle(&bundle, ExportFormat::Postgres).unwrap();
        assert!(pg_dump.contains("CREATE TABLE IF NOT EXISTS \"customers\""));
        assert!(pg_dump.contains("INSERT INTO \"customers\""));
        assert!(pg_dump.contains("'O''Reilly Media'"));

        // Test MySQL export
        let mysql_dump = MultiFormatDatabaseExtractor::export_bundle(&bundle, ExportFormat::Mysql).unwrap();
        assert!(mysql_dump.contains("CREATE TABLE IF NOT EXISTS `customers`"));
        assert!(mysql_dump.contains("INSERT INTO `customers`"));

        // Test ANSI SQL export
        let ansi_dump = MultiFormatDatabaseExtractor::export_bundle(&bundle, ExportFormat::AnsiSql).unwrap();
        assert!(ansi_dump.contains("CREATE TABLE IF NOT EXISTS [customers]"));

        // Test JSON export
        let json_dump = MultiFormatDatabaseExtractor::export_bundle(&bundle, ExportFormat::Json).unwrap();
        assert!(json_dump.contains("\"database_id\": \"test_db\""));

        // Test CSV export
        let csv = MultiFormatDatabaseExtractor::export_table_to_csv(table);
        assert!(csv.contains("\"id\",\"name\",\"balance\",\"is_active\""));
        assert!(csv.contains("\"O'Reilly Media\""));
    }
}
