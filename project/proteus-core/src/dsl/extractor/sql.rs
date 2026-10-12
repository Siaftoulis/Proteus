//! SQL Formatting & Dialect Dumps for Database Extractor.
//! Strict Rule 1 (100% Original Codebase), Rule 3 (<400 lines), and Rule 5 (Zero Mock Data).

use super::DatabaseExportBundle;

pub struct SqlFormatter;

impl SqlFormatter {
    /// Generates full PostgreSQL SQL script with DDL and INSERT statements.
    pub fn generate_postgres_dump(bundle: &DatabaseExportBundle) -> String {
        let mut dump = format!(
            "-- Proteus Sovereign OS -> PostgreSQL Full Data & Schema Export\n\
             -- Generated: {}\n\
             -- Database: {}\n\n\
             SET statement_timeout = 0;\n\
             SET client_encoding = 'UTF8';\n\
             SET standard_conforming_strings = on;\n\n",
            bundle.exported_at_iso, bundle.database_id
        );

        for table in &bundle.tables {
            dump.push_str("-- -----------------------------------------------------\n");
            dump.push_str(&format!("-- Table structure for \"{}\"\n", table.table_name));
            dump.push_str("-- -----------------------------------------------------\n");
            dump.push_str(&format!("CREATE TABLE IF NOT EXISTS \"{}\" (\n", table.table_name));

            let mut col_lines = Vec::new();
            for col in &table.columns {
                let pg_type = Self::map_type_to_postgres(&col.data_type);
                let mut col_def = format!("    \"{}\" {}", col.name, pg_type);
                if col.pk {
                    col_def.push_str(" PRIMARY KEY");
                }
                if col.notnull && !col.pk {
                    col_def.push_str(" NOT NULL");
                }
                if let Some(ref dflt) = col.dflt_value {
                    col_def.push_str(&format!(" DEFAULT {}", dflt));
                }
                col_lines.push(col_def);
            }
            dump.push_str(&col_lines.join(",\n"));
            dump.push_str("\n);\n\n");

            if !table.rows.is_empty() {
                dump.push_str(&format!("-- Data records for \"{}\"\n", table.table_name));
                for row in &table.rows {
                    let col_names = table
                        .columns
                        .iter()
                        .map(|c| format!("\"{}\"", c.name))
                        .collect::<Vec<_>>()
                        .join(", ");

                    let col_vals = row
                        .iter()
                        .map(|v| Self::format_postgres_value(v))
                        .collect::<Vec<_>>()
                        .join(", ");

                    dump.push_str(&format!(
                        "INSERT INTO \"{}\" ({}) VALUES ({});\n",
                        table.table_name, col_names, col_vals
                    ));
                }
                dump.push('\n');
            }
        }
        dump
    }

    /// Generates full MySQL / MariaDB SQL script with DDL and INSERT statements.
    pub fn generate_mysql_dump(bundle: &DatabaseExportBundle) -> String {
        let mut dump = format!(
            "-- Proteus Sovereign OS -> MySQL Full Data & Schema Export\n\
             -- Generated: {}\n\
             -- Database: {}\n\n\
             SET NAMES utf8mb4;\n\
             SET FOREIGN_KEY_CHECKS = 0;\n\n",
            bundle.exported_at_iso, bundle.database_id
        );

        for table in &bundle.tables {
            dump.push_str("-- -----------------------------------------------------\n");
            dump.push_str(&format!("-- Table structure for `{}`\n", table.table_name));
            dump.push_str("-- -----------------------------------------------------\n");
            dump.push_str(&format!("CREATE TABLE IF NOT EXISTS `{}` (\n", table.table_name));

            let mut col_lines = Vec::new();
            for col in &table.columns {
                let mysql_type = Self::map_type_to_mysql(&col.data_type);
                let mut col_def = format!("    `{}` {}", col.name, mysql_type);
                if col.pk {
                    col_def.push_str(" PRIMARY KEY");
                }
                if col.notnull && !col.pk {
                    col_def.push_str(" NOT NULL");
                }
                if let Some(ref dflt) = col.dflt_value {
                    col_def.push_str(&format!(" DEFAULT {}", dflt));
                }
                col_lines.push(col_def);
            }
            dump.push_str(&col_lines.join(",\n"));
            dump.push_str("\n) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;\n\n");

            if !table.rows.is_empty() {
                dump.push_str(&format!("-- Data records for `{}`\n", table.table_name));
                for row in &table.rows {
                    let col_names = table
                        .columns
                        .iter()
                        .map(|c| format!("`{}`", c.name))
                        .collect::<Vec<_>>()
                        .join(", ");

                    let col_vals = row
                        .iter()
                        .map(|v| Self::format_mysql_value(v))
                        .collect::<Vec<_>>()
                        .join(", ");

                    dump.push_str(&format!(
                        "INSERT INTO `{}` ({}) VALUES ({});\n",
                        table.table_name, col_names, col_vals
                    ));
                }
                dump.push('\n');
            }
        }
        dump.push_str("SET FOREIGN_KEY_CHECKS = 1;\n");
        dump
    }

    /// Generates ANSI SQL / SQLite compliant script.
    pub fn generate_ansi_sql_dump(bundle: &DatabaseExportBundle) -> String {
        let mut dump = format!(
            "-- Proteus Sovereign OS -> ANSI SQL Standard Export\n\
             -- Generated: {}\n\
             -- Database: {}\n\n",
            bundle.exported_at_iso, bundle.database_id
        );

        for table in &bundle.tables {
            dump.push_str(&format!("CREATE TABLE IF NOT EXISTS [{}] (\n", table.table_name));
            let mut col_lines = Vec::new();
            for col in &table.columns {
                let mut col_def = format!("    [{}] {}", col.name, col.data_type);
                if col.pk {
                    col_def.push_str(" PRIMARY KEY");
                }
                if col.notnull && !col.pk {
                    col_def.push_str(" NOT NULL");
                }
                if let Some(ref dflt) = col.dflt_value {
                    col_def.push_str(&format!(" DEFAULT {}", dflt));
                }
                col_lines.push(col_def);
            }
            dump.push_str(&col_lines.join(",\n"));
            dump.push_str("\n);\n\n");

            if !table.rows.is_empty() {
                for row in &table.rows {
                    let col_names = table
                        .columns
                        .iter()
                        .map(|c| format!("[{}]", c.name))
                        .collect::<Vec<_>>()
                        .join(", ");

                    let col_vals = row
                        .iter()
                        .map(|v| Self::format_ansi_value(v))
                        .collect::<Vec<_>>()
                        .join(", ");

                    dump.push_str(&format!(
                        "INSERT INTO [{}] ({}) VALUES ({});\n",
                        table.table_name, col_names, col_vals
                    ));
                }
                dump.push('\n');
            }
        }
        dump
    }

    fn map_type_to_postgres(sqlite_type: &str) -> &'static str {
        let upper = sqlite_type.to_uppercase();
        if upper.contains("INT") {
            "BIGINT"
        } else if upper.contains("CHAR") || upper.contains("TEXT") || upper.contains("CLOB") {
            "TEXT"
        } else if upper.contains("REAL") || upper.contains("FLOA") || upper.contains("DOUB") {
            "DOUBLE PRECISION"
        } else if upper.contains("DECIMAL") || upper.contains("NUMERIC") {
            "NUMERIC(14,2)"
        } else if upper.contains("BLOB") {
            "BYTEA"
        } else if upper.contains("BOOL") {
            "BOOLEAN"
        } else {
            "TEXT"
        }
    }

    fn map_type_to_mysql(sqlite_type: &str) -> &'static str {
        let upper = sqlite_type.to_uppercase();
        if upper.contains("INT") {
            "BIGINT"
        } else if upper.contains("CHAR") || upper.contains("TEXT") || upper.contains("CLOB") {
            "LONGTEXT"
        } else if upper.contains("REAL") || upper.contains("FLOA") || upper.contains("DOUB") {
            "DOUBLE"
        } else if upper.contains("DECIMAL") || upper.contains("NUMERIC") {
            "DECIMAL(14,2)"
        } else if upper.contains("BLOB") {
            "LONGBLOB"
        } else if upper.contains("BOOL") {
            "TINYINT(1)"
        } else {
            "LONGTEXT"
        }
    }

    fn format_postgres_value(val: &serde_json::Value) -> String {
        match val {
            serde_json::Value::Null => "NULL".to_string(),
            serde_json::Value::Bool(b) => if *b { "TRUE".to_string() } else { "FALSE".to_string() },
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::String(s) => {
                if let Some(hex) = s.strip_prefix("hex:") {
                    format!("decode('{}', 'hex')", hex)
                } else {
                    format!("'{}'", s.replace('\'', "''"))
                }
            }
            _ => format!("'{}'", val.to_string().replace('\'', "''")),
        }
    }

    fn format_mysql_value(val: &serde_json::Value) -> String {
        match val {
            serde_json::Value::Null => "NULL".to_string(),
            serde_json::Value::Bool(b) => if *b { "1".to_string() } else { "0".to_string() },
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::String(s) => {
                if let Some(hex) = s.strip_prefix("hex:") {
                    format!("UNHEX('{}')", hex)
                } else {
                    format!("'{}'", s.replace('\'', "''").replace('\\', "\\\\"))
                }
            }
            _ => format!("'{}'", val.to_string().replace('\'', "''")),
        }
    }

    fn format_ansi_value(val: &serde_json::Value) -> String {
        match val {
            serde_json::Value::Null => "NULL".to_string(),
            serde_json::Value::Bool(b) => if *b { "1".to_string() } else { "0".to_string() },
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::String(s) => {
                if let Some(hex) = s.strip_prefix("hex:") {
                    format!("X'{}'", hex)
                } else {
                    format!("'{}'", s.replace('\'', "''"))
                }
            }
            _ => format!("'{}'", val.to_string().replace('\'', "''")),
        }
    }
}
