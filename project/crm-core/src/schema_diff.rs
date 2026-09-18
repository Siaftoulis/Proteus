// Proteus Core — Visual Schema Diffing & Safety Verification Engine
// Designed from first principles. Zero copied third-party boilerplate.

use crate::drivers::ColumnMeta;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableMeta {
    pub name: String,
    pub columns: Vec<ColumnMeta>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffSafety {
    SafeAdditive,
    CautionTypeChange,
    DestructiveDrop,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColumnChangeKind {
    Added { data_type: String, is_nullable: bool },
    Modified { old_type: String, new_type: String },
    Unchanged { data_type: String },
    Dropped { old_type: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColumnDiff {
    pub column_name: String,
    pub kind: ColumnChangeKind,
    pub safety: DiffSafety,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TableChangeKind {
    Added,
    Modified,
    Unchanged,
    Dropped,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableDiff {
    pub table_name: String,
    pub kind: TableChangeKind,
    pub column_diffs: Vec<ColumnDiff>,
    pub safety: DiffSafety,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaDiff {
    pub table_diffs: Vec<TableDiff>,
    pub overall_safety: DiffSafety,
    pub new_tables_count: usize,
    pub new_columns_count: usize,
    pub modified_columns_count: usize,
    pub dropped_count: usize,
}

impl SchemaDiff {
    pub fn compute(live: &[TableMeta], target: &[TableMeta]) -> Self {
        let mut table_diffs = Vec::new();
        let mut new_tables_count = 0;
        let mut new_columns_count = 0;
        let mut modified_columns_count = 0;
        let mut dropped_count = 0;
        let mut worst_safety = DiffSafety::SafeAdditive;

        // Check tables in target against live
        for t_target in target {
            if let Some(t_live) = live.iter().find(|l| l.name.eq_ignore_ascii_case(&t_target.name)) {
                let mut col_diffs = Vec::new();
                let mut table_has_change = false;

                // Check target columns
                for c_target in &t_target.columns {
                    if let Some(c_live) = t_live.columns.iter().find(|cl| cl.name.eq_ignore_ascii_case(&c_target.name)) {
                        if c_live.data_type.to_uppercase() != c_target.data_type.to_uppercase() {
                            table_has_change = true;
                            modified_columns_count += 1;
                            worst_safety = max_safety(worst_safety, DiffSafety::CautionTypeChange);
                            col_diffs.push(ColumnDiff {
                                column_name: c_target.name.clone(),
                                kind: ColumnChangeKind::Modified {
                                    old_type: c_live.data_type.clone(),
                                    new_type: c_target.data_type.clone(),
                                },
                                safety: DiffSafety::CautionTypeChange,
                            });
                        } else {
                            col_diffs.push(ColumnDiff {
                                column_name: c_target.name.clone(),
                                kind: ColumnChangeKind::Unchanged { data_type: c_live.data_type.clone() },
                                safety: DiffSafety::SafeAdditive,
                            });
                        }
                    } else {
                        // Added column
                        table_has_change = true;
                        new_columns_count += 1;
                        col_diffs.push(ColumnDiff {
                            column_name: c_target.name.clone(),
                            kind: ColumnChangeKind::Added {
                                data_type: c_target.data_type.clone(),
                                is_nullable: c_target.is_nullable,
                            },
                            safety: DiffSafety::SafeAdditive,
                        });
                    }
                }

                // Check for live columns dropped in target
                for c_live in &t_live.columns {
                    if !t_target.columns.iter().any(|ct| ct.name.eq_ignore_ascii_case(&c_live.name)) {
                        table_has_change = true;
                        dropped_count += 1;
                        worst_safety = max_safety(worst_safety, DiffSafety::DestructiveDrop);
                        col_diffs.push(ColumnDiff {
                            column_name: c_live.name.clone(),
                            kind: ColumnChangeKind::Dropped { old_type: c_live.data_type.clone() },
                            safety: DiffSafety::DestructiveDrop,
                        });
                    }
                }

                let t_safety = col_diffs.iter().map(|c| c.safety).fold(DiffSafety::SafeAdditive, max_safety);
                let t_kind = if table_has_change { TableChangeKind::Modified } else { TableChangeKind::Unchanged };

                table_diffs.push(TableDiff {
                    table_name: t_target.name.clone(),
                    kind: t_kind,
                    column_diffs: col_diffs,
                    safety: t_safety,
                });
            } else {
                // Completely new table
                new_tables_count += 1;
                let mut col_diffs = Vec::new();
                for c in &t_target.columns {
                    new_columns_count += 1;
                    col_diffs.push(ColumnDiff {
                        column_name: c.name.clone(),
                        kind: ColumnChangeKind::Added {
                            data_type: c.data_type.clone(),
                            is_nullable: c.is_nullable,
                        },
                        safety: DiffSafety::SafeAdditive,
                    });
                }
                table_diffs.push(TableDiff {
                    table_name: t_target.name.clone(),
                    kind: TableChangeKind::Added,
                    column_diffs: col_diffs,
                    safety: DiffSafety::SafeAdditive,
                });
            }
        }

        // Check for live tables dropped in target
        for t_live in live {
            if !target.iter().any(|t| t.name.eq_ignore_ascii_case(&t_live.name)) {
                dropped_count += 1;
                worst_safety = max_safety(worst_safety, DiffSafety::DestructiveDrop);
                let col_diffs = t_live.columns.iter().map(|c| ColumnDiff {
                    column_name: c.name.clone(),
                    kind: ColumnChangeKind::Dropped { old_type: c.data_type.clone() },
                    safety: DiffSafety::DestructiveDrop,
                }).collect();

                table_diffs.push(TableDiff {
                    table_name: t_live.name.clone(),
                    kind: TableChangeKind::Dropped,
                    column_diffs: col_diffs,
                    safety: DiffSafety::DestructiveDrop,
                });
            }
        }

        Self {
            table_diffs,
            overall_safety: worst_safety,
            new_tables_count,
            new_columns_count,
            modified_columns_count,
            dropped_count,
        }
    }

    pub fn inspect_connection(conn: &Connection) -> Result<Vec<TableMeta>, String> {
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' AND name != '_proteus_migrations' ORDER BY name")
            .map_err(|e| e.to_string())?;
        let table_names: Vec<String> = stmt
            .query_map([], |r| r.get(0))
            .map_err(|e| e.to_string())?
            .filter_map(|r| r.ok())
            .collect();

        let mut tables = Vec::new();
        for tname in table_names {
            let pragma = format!("PRAGMA table_info(\"{}\")", tname.replace('"', ""));
            let mut col_stmt = conn.prepare(&pragma).map_err(|e| e.to_string())?;
            let cols = col_stmt
                .query_map([], |r| {
                    Ok(ColumnMeta {
                        name: r.get(1)?,
                        data_type: r.get(2)?,
                        is_nullable: r.get::<_, i32>(3)? == 0,
                        is_primary_key: r.get::<_, i32>(5)? > 0,
                    })
                })
                .map_err(|e| e.to_string())?
                .filter_map(|r| r.ok())
                .collect();

            tables.push(TableMeta {
                name: tname,
                columns: cols,
            });
        }
        Ok(tables)
    }

    pub fn from_ddl(live_conn: &Connection, ddl_statements: &[String]) -> Result<Self, String> {
        let live_tables = Self::inspect_connection(live_conn)?;

        // Replicate live schema into a temporary in-memory connection
        let sandbox = Connection::open_in_memory().map_err(|e| e.to_string())?;
        for t in &live_tables {
            if t.columns.is_empty() {
                continue;
            }
            let col_defs: Vec<String> = t.columns.iter().map(|c| {
                let pk = if c.is_primary_key { " PRIMARY KEY" } else { "" };
                let not_null = if !c.is_nullable { " NOT NULL" } else { "" };
                format!("\"{}\" {}{}{}", c.name, c.data_type, pk, not_null)
            }).collect();
            let sql = format!("CREATE TABLE \"{}\" ({});", t.name, col_defs.join(", "));
            let _ = sandbox.execute_batch(&sql);
        }

        // Apply pending DDL
        for ddl in ddl_statements {
            sandbox.execute_batch(ddl).map_err(|e| format!("DDL Simulation Error on '{}': {}", ddl, e))?;
        }

        let target_tables = Self::inspect_connection(&sandbox)?;
        Ok(Self::compute(&live_tables, &target_tables))
    }
}

fn max_safety(a: DiffSafety, b: DiffSafety) -> DiffSafety {
    match (a, b) {
        (DiffSafety::DestructiveDrop, _) | (_, DiffSafety::DestructiveDrop) => DiffSafety::DestructiveDrop,
        (DiffSafety::CautionTypeChange, _) | (_, DiffSafety::CautionTypeChange) => DiffSafety::CautionTypeChange,
        _ => DiffSafety::SafeAdditive,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_diff_new_table() {
        let live = vec![TableMeta {
            name: "customers".into(),
            columns: vec![ColumnMeta {
                name: "id".into(),
                data_type: "INTEGER".into(),
                is_nullable: false,
                is_primary_key: true,
            }],
        }];

        let target = vec![
            live[0].clone(),
            TableMeta {
                name: "loyalty_cards".into(),
                columns: vec![ColumnMeta {
                    name: "card_no".into(),
                    data_type: "TEXT".into(),
                    is_nullable: false,
                    is_primary_key: true,
                }],
            },
        ];

        let diff = SchemaDiff::compute(&live, &target);
        assert_eq!(diff.new_tables_count, 1);
        assert_eq!(diff.overall_safety, DiffSafety::SafeAdditive);
        assert_eq!(diff.table_diffs[1].kind, TableChangeKind::Added);
    }

    #[test]
    fn test_from_ddl_simulation() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE fleet (id TEXT PRIMARY KEY, plate TEXT NOT NULL);").unwrap();

        let ddl = vec![
            "ALTER TABLE fleet ADD COLUMN mileage REAL;".to_string(),
            "CREATE TABLE maintenance_records (id TEXT PRIMARY KEY, cost REAL);".to_string(),
        ];

        let diff = SchemaDiff::from_ddl(&conn, &ddl).expect("Simulation should succeed");
        assert_eq!(diff.new_tables_count, 1);
        assert_eq!(diff.new_columns_count, 3); // 1 on fleet + 2 on maintenance_records
        assert_eq!(diff.overall_safety, DiffSafety::SafeAdditive);
    }
}
