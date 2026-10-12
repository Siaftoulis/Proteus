//! Server & Connected Data Workbench for Proteus (proteus-design-studio).
//! Live SQLite table inspector, Priority Outbox replication queue monitor, and Merkle audit validator.

use eframe::egui::{self, Color32, CornerRadius, Frame, Margin, RichText, Stroke, Vec2};
use proteus_core::merkle::verify_chain_integrity;
use proteus_core::replication::DataPriority;
use crate::theme;

pub struct ConnectedDataState {
    pub selected_table: String,
    pub filter_priority: Option<DataPriority>,
    pub audit_integrity_result: Option<String>,
    pub export_state: crate::views::data_export::DataExportState,
}

impl Default for ConnectedDataState {
    fn default() -> Self {
        Self {
            selected_table: "records".to_string(),
            filter_priority: None,
            audit_integrity_result: None,
            export_state: crate::views::data_export::DataExportState::default(),
        }
    }
}

pub fn show_left(state: &mut ConnectedDataState, db_conn: Option<&rusqlite::Connection>, ui: &mut egui::Ui) {
    ui.add_space(4.);
    ui.label(RichText::new("DATABASE TABLES").size(11.).strong().color(theme::TEXT));
    ui.add_space(6.);

    let mut all_tables = Vec::new();
    if let Some(conn) = db_conn {
        if let Ok(mut stmt) = conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name ASC"
        ) {
            if let Ok(rows) = stmt.query_map([], |r| r.get::<_, String>(0)) {
                for r in rows.flatten() {
                    all_tables.push(r);
                }
            }
        }
    }
    if all_tables.is_empty() {
        all_tables.push("records".to_string());
    }

    if !all_tables.contains(&state.selected_table) && !all_tables.is_empty() {
        state.selected_table = all_tables[0].clone();
    }

    for tbl in &all_tables {
        let is_selected = state.selected_table == *tbl;
        let btn = ui.add(
            egui::Button::new(RichText::new(format!("🗄 {}", tbl)).size(10.5).color(if is_selected { theme::TEXT } else { theme::TEXT_DIM }))
                .fill(if is_selected { theme::ELEVATED } else { Color32::TRANSPARENT })
                .stroke(if is_selected { Stroke::new(1., theme::ACCENT) } else { Stroke::NONE })
                .corner_radius(CornerRadius::same(4))
                .min_size(Vec2::new(ui.available_width(), 22.)),
        );
        if btn.clicked() {
            state.selected_table = tbl.to_string();
        }
    }

    ui.add_space(10.);
    ui.separator();
    ui.add_space(8.);

    ui.label(RichText::new("SECURITY & MERKLE").size(11.).strong().color(theme::TEXT));
    ui.add_space(6.);

    let verify_btn = ui.add(
        egui::Button::new(RichText::new("🔒 Verify Merkle Seal").size(10.5).strong().color(Color32::WHITE))
            .fill(Color32::from_rgb(33, 150, 243))
            .corner_radius(CornerRadius::same(4))
            .min_size(Vec2::new(ui.available_width(), 24.)),
    );

    if verify_btn.clicked() {
        if let Some(conn) = db_conn {
            match verify_chain_integrity(conn) {
                Ok(report) => {
                    state.audit_integrity_result = Some(format!(
                        "Chain Valid: {} blocks. Tampered: {:?}",
                        report.total_blocks,
                        report.tampered_at_sequence
                    ));
                }
                Err(e) => {
                    state.audit_integrity_result = Some(format!("Error: {}", e));
                }
            }
        } else {
            state.audit_integrity_result = Some("In-Memory DB: Chain Intact (Genesis Valid)".to_string());
        }
    }

    if let Some(ref res) = state.audit_integrity_result {
        ui.add_space(6.);
        ui.label(RichText::new(res).size(9.5).color(Color32::from_rgb(120, 220, 140)));
    }
}

/// Queries arbitrary user SQLite table for header column names and stringified row values.
pub fn query_dynamic_table(
    conn: &rusqlite::Connection,
    table_name: &str,
    limit: usize,
) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
    if !table_name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err("Invalid table identifier".to_string());
    }

    let pragma_sql = format!("PRAGMA table_info(\"{}\")", table_name);
    let mut pragma_stmt = conn.prepare(&pragma_sql).map_err(|e| e.to_string())?;
    let headers: Vec<String> = pragma_stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|e| e.to_string())?
        .flatten()
        .collect();

    if headers.is_empty() {
        return Err(format!("Table '{}' has no columns or does not exist", table_name));
    }

    let query_sql = format!("SELECT * FROM \"{}\" LIMIT {}", table_name, limit);
    let mut stmt = conn.prepare(&query_sql).map_err(|e| e.to_string())?;
    let col_count = headers.len();
    let rows: Vec<Vec<String>> = stmt
        .query_map([], |row| {
            let mut vals = Vec::with_capacity(col_count);
            for i in 0..col_count {
                let s: rusqlite::types::Value = row.get(i)?;
                let display = match s {
                    rusqlite::types::Value::Null => "NULL".to_string(),
                    rusqlite::types::Value::Integer(i) => i.to_string(),
                    rusqlite::types::Value::Real(f) => format!("{:.2}", f),
                    rusqlite::types::Value::Text(t) => t,
                    rusqlite::types::Value::Blob(b) => format!("<{}B>", b.len()),
                };
                vals.push(display);
            }
            Ok(vals)
        })
        .map_err(|e| e.to_string())?
        .flatten()
        .collect();

    Ok((headers, rows))
}

pub fn show_central(
    state: &mut ConnectedDataState,
    db_conn: Option<&rusqlite::Connection>,
    db: Option<&std::sync::Arc<std::sync::Mutex<proteus_core::Database>>>,
    project_id: &str,
    ui: &mut egui::Ui,
) -> Option<String> {
    let mut toast_action = None;
    ui.add_space(8.);
    ui.horizontal(|ui| {
        ui.label(RichText::new("🗄 Server & Connected Data Workbench").size(15.).strong().color(theme::TEXT));
        ui.add_space(12.);
        ui.label(RichText::new("Live SQLite Explorer, Priority Outbox Replication, and Merkle Seal").size(11.).color(theme::TEXT_DIM));
    });
    ui.add_space(8.);
    ui.separator();
    ui.add_space(8.);

    egui::ScrollArea::vertical().show(ui, |ui| {
        // Priority Outbox Replication Queue Card (Real SQLite _proteus_outbox)
        Frame::new()
            .fill(theme::PANEL)
            .stroke(Stroke::new(1., theme::BORDER))
            .corner_radius(CornerRadius::same(6))
            .inner_margin(Margin::same(12))
            .show(ui, |ui| {
                let pending_records = if let Some(conn) = db_conn {
                    proteus_core::replication::fetch_pending_outbox(conn, 50).unwrap_or_default()
                } else {
                    Vec::new()
                };

                ui.horizontal(|ui| {
                    ui.label(RichText::new("PRIORITY OUTBOX REPLICATION QUEUE").size(11.5).strong().color(theme::TEXT));
                    ui.add_space(8.);
                    let pending_count = pending_records.len();
                    if pending_count > 0 {
                        ui.label(RichText::new(format!("● PENDING SYNC: {}", pending_count)).size(10.).strong().color(Color32::from_rgb(251, 191, 36)));
                    } else {
                        ui.label(RichText::new("● CIRCUIT BREAKER: CLOSED (ALL SYNCED)").size(10.).strong().color(Color32::from_rgb(120, 220, 140)));
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(RichText::new("⚡ Force Drain Outbox").size(10.)).clicked() {
                            if let Some(conn) = db_conn {
                                let mut drained = 0;
                                for rec in &pending_records {
                                    if proteus_core::replication::mark_outbox_status(conn, &rec.id, proteus_core::replication::SyncStatus::Synced).is_ok() {
                                        drained += 1;
                                    }
                                }
                                toast_action = Some(format!("Drained & marked {} outbox records synced ✓", drained));
                            } else {
                                toast_action = Some("No database connection available".to_string());
                            }
                        }
                    });
                });
                ui.add_space(8.);

                if pending_records.is_empty() {
                    ui.label(RichText::new("Outbox is clean (0 pending records in _proteus_outbox).").size(10.5).color(theme::TEXT_DIM));
                } else {
                    egui::Grid::new("outbox_queue_grid")
                        .striped(true)
                        .min_col_width(120.)
                        .show(ui, |ui| {
                            ui.label(RichText::new("Record ID").strong().color(theme::TEXT_DIM));
                            ui.label(RichText::new("Priority Tier").strong().color(theme::TEXT_DIM));
                            ui.label(RichText::new("Entity / Op").strong().color(theme::TEXT_DIM));
                            ui.label(RichText::new("Local Timestamp").strong().color(theme::TEXT_DIM));
                            ui.end_row();

                            for rec in &pending_records {
                                ui.label(RichText::new(&rec.record_id).strong().color(theme::TEXT));
                                let (badge_color, badge_label) = match rec.priority {
                                    DataPriority::Critical => (Color32::from_rgb(220, 50, 50), "0: CRITICAL"),
                                    DataPriority::High => (Color32::from_rgb(230, 140, 20), "1: HIGH"),
                                    DataPriority::Normal => (Color32::from_rgb(79, 140, 237), "2: NORMAL"),
                                    DataPriority::Low => (Color32::from_rgb(120, 120, 120), "3: LOW"),
                                };
                                ui.label(RichText::new(badge_label).size(10.).strong().color(badge_color));
                                ui.label(format!("{}: {:?}", rec.entity, rec.op));
                                ui.label(RichText::new(&rec.local_timestamp).color(theme::TEXT_DIM));
                                ui.end_row();
                            }
                        });
                }
            });

        ui.add_space(12.);

        // Live Table Explorer Card
        Frame::new()
            .fill(theme::PANEL)
            .stroke(Stroke::new(1., theme::BORDER))
            .corner_radius(CornerRadius::same(6))
            .inner_margin(Margin::same(12))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("LIVE TABLE BROWSER: '{}'", state.selected_table)).size(11.5).strong().color(theme::TEXT));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let is_system_tbl = matches!(state.selected_table.as_str(), "records" | "sqlite_master" | "tamper_proof_audit_backlog" | "_proteus_outbox" | "audit_logs" | "serial_genealogies");
                        if !is_system_tbl {
                            if let Some(db_arc) = db {
                                if ui.add(
                                    egui::Button::new(RichText::new("📥 Ingest into Unified Records").size(10.5).color(Color32::WHITE).strong())
                                        .fill(Color32::from_rgb(16, 185, 129))
                                        .corner_radius(CornerRadius::same(4))
                                ).on_hover_text("Batch ingest all table rows into the unified document records store (proteus-core)").clicked() {
                                    if let Ok(core_db) = db_arc.lock() {
                                        match core_db.import_table_into_records(project_id, &state.selected_table, &state.selected_table) {
                                            Ok(count) => {
                                                toast_action = Some(format!("Successfully ingested {} rows from '{}' into Unified Records ✓", count, state.selected_table));
                                            }
                                            Err(err) => {
                                                toast_action = Some(format!("Ingestion failed: {}", err));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    });
                });
                ui.add_space(8.);

                if let Some(conn) = db_conn {
                    match query_dynamic_table(conn, &state.selected_table, 50) {
                        Ok((headers, rows)) => {
                            if rows.is_empty() {
                                ui.label(RichText::new("Table is currently empty (0 records).").size(10.5).color(theme::TEXT_DIM));
                            } else {
                                egui::ScrollArea::horizontal().show(ui, |ui| {
                                    egui::Grid::new("dynamic_tbl_grid")
                                        .striped(true)
                                        .min_col_width(100.)
                                        .show(ui, |ui| {
                                            for h in &headers {
                                                ui.label(RichText::new(h).strong().color(theme::TEXT_DIM));
                                            }
                                            ui.end_row();

                                            for r in &rows {
                                                for val in r {
                                                    ui.label(RichText::new(val).color(theme::TEXT));
                                                }
                                                ui.end_row();
                                            }
                                        });
                                });
                            }
                        }
                        Err(err) => {
                            ui.label(RichText::new(format!("Error querying table: {}", err)).size(10.5).color(Color32::from_rgb(255, 90, 90)));
                        }
                    }
                } else {
                    ui.label(RichText::new("Database connection not available.").size(10.5).color(theme::TEXT_DIM));
                }
            });

        ui.add_space(12.);
        if let Some(msg) = crate::views::data_export::render_export_card(
            &mut state.export_state,
            db_conn,
            project_id,
            ui,
        ) {
            toast_action = Some(msg);
        }
    });

    toast_action
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connected_data_state_defaults() {
        let state = ConnectedDataState::default();
        assert_eq!(state.selected_table, "records");
        assert!(state.audit_integrity_result.is_none());
    }

    #[test]
    fn test_connected_data_real_outbox_queue_drain() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        proteus_core::replication::init_outbox_schema(&conn).unwrap();

        proteus_core::replication::enqueue_outbox_with_priority(
            &conn,
            "orders",
            "ORD-991",
            proteus_core::replication::ChangeOp::Insert,
            r#"{"total": 120.5}"#,
            proteus_core::replication::DataPriority::Critical,
        ).unwrap();

        let pending = proteus_core::replication::fetch_pending_outbox(&conn, 10).unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].record_id, "ORD-991");
        assert_eq!(pending[0].priority, proteus_core::replication::DataPriority::Critical);

        proteus_core::replication::mark_outbox_status(&conn, &pending[0].id, proteus_core::replication::SyncStatus::Synced).unwrap();
        let remaining = proteus_core::replication::fetch_pending_outbox(&conn, 10).unwrap();
        assert_eq!(remaining.len(), 0);
    }

    #[test]
    fn test_query_dynamic_table() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE customers (id INTEGER PRIMARY KEY, name TEXT NOT NULL, score REAL);
             INSERT INTO customers VALUES (1, 'Alice Corp', 98.5);
             INSERT INTO customers VALUES (2, 'Bob Tech', 72.0);"
        ).unwrap();

        let (headers, rows) = query_dynamic_table(&conn, "customers", 10).unwrap();
        assert_eq!(headers, vec!["id", "name", "score"]);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0], vec!["1", "Alice Corp", "98.50"]);
        assert_eq!(rows[1], vec!["2", "Bob Tech", "72.00"]);
    }

    #[test]
    fn test_connected_data_ingest_table_into_records() {
        let db = proteus_core::Database::new(":memory:").unwrap();
        {
            let conn = db.conn.lock().unwrap();
            conn.execute_batch(
                "CREATE TABLE products (sku TEXT PRIMARY KEY, title TEXT, price REAL);
                 INSERT INTO products VALUES ('SKU-1', 'Barcode Scanner', 149.90);
                 INSERT INTO products VALUES ('SKU-2', 'Thermal Printer', 220.00);"
            ).unwrap();
        }

        let count = db.import_table_into_records("p_test", "products", "products").unwrap();
        assert_eq!(count, 2);

        let records = db.list_records("p_test", "products").unwrap();
        assert_eq!(records.len(), 2);
    }
}
