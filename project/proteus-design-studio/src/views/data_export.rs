//! Sovereign Data Export Workbench for Proteus (proteus-design-studio).
//! Strict Rule 1 (100% Original Codebase), Rule 3 (<400 lines), and Rule 5 (Zero Mock Data).
//! Renders multi-format database extraction controls (PostgreSQL, MySQL, ANSI SQL, JSON, CSV).

use eframe::egui::{self, Color32, CornerRadius, Frame, Margin, RichText, Stroke, Ui, Vec2};
use proteus_core::dsl::{ExportFormat, MultiFormatDatabaseExtractor};
use crate::theme;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataExportState {
    pub format: ExportFormat,
    pub preview_snippet: Option<String>,
    pub last_status: Option<String>,
    pub total_tables: usize,
    pub total_rows: usize,
}

impl Default for DataExportState {
    fn default() -> Self {
        Self {
            format: ExportFormat::Postgres,
            preview_snippet: None,
            last_status: None,
            total_tables: 0,
            total_rows: 0,
        }
    }
}

/// Executes data extraction from live SQLite connection and formats to target dialect.
pub fn execute_export(
    conn: &rusqlite::Connection,
    database_id: &str,
    format: ExportFormat,
) -> Result<(String, usize, usize), String> {
    let bundle = MultiFormatDatabaseExtractor::extract_all_tables(conn, database_id)?;
    let total_tables = bundle.tables.len();
    let total_rows = bundle.tables.iter().map(|t| t.rows.len()).sum();
    let formatted = MultiFormatDatabaseExtractor::export_bundle(&bundle, format)?;
    Ok((formatted, total_tables, total_rows))
}

/// Renders the Sovereign Data Export Card inside the Connected Data workbench.
pub fn render_export_card(
    state: &mut DataExportState,
    db_conn: Option<&rusqlite::Connection>,
    database_id: &str,
    ui: &mut Ui,
) -> Option<String> {
    let mut toast_action = None;

    Frame::new()
        .fill(theme::PANEL)
        .stroke(Stroke::new(1., theme::BORDER))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("🌐 SOVEREIGN DATA EXPORT (ZERO VENDOR LOCK-IN)").size(11.5).strong().color(theme::TEXT));
                ui.add_space(8.);
                ui.label(RichText::new("Export live data to standard database engines").size(10.).color(theme::TEXT_DIM));
            });
            ui.add_space(8.);

            // Format selector pills
            ui.horizontal(|ui| {
                ui.label(RichText::new("Target Dialect:").size(10.5).color(theme::TEXT_DIM));
                ui.add_space(4.);

                let formats = [
                    (ExportFormat::Postgres, "PostgreSQL"),
                    (ExportFormat::Mysql, "MySQL / MariaDB"),
                    (ExportFormat::AnsiSql, "ANSI SQL / SQLite"),
                    (ExportFormat::Json, "JSON Bundle"),
                    (ExportFormat::Csv, "RFC CSV"),
                ];

                for (fmt, label) in formats {
                    let is_active = state.format == fmt;
                    let btn = ui.add(
                        egui::Button::new(RichText::new(label).size(10.).color(if is_active { Color32::WHITE } else { theme::TEXT_DIM }))
                            .fill(if is_active { theme::ACCENT } else { theme::ELEVATED })
                            .stroke(if is_active { Stroke::new(1., theme::ACCENT) } else { Stroke::new(1., theme::BORDER) })
                            .corner_radius(CornerRadius::same(4))
                    );
                    if btn.clicked() {
                        state.format = fmt;
                    }
                }
            });

            ui.add_space(8.);

            // Action buttons row
            ui.horizontal(|ui| {
                let generate_btn = ui.add(
                    egui::Button::new(RichText::new("⚡ Generate & Copy Export").size(10.5).strong().color(Color32::WHITE))
                        .fill(Color32::from_rgb(16, 185, 129))
                        .corner_radius(CornerRadius::same(4))
                        .min_size(Vec2::new(180., 24.)),
                );

                if generate_btn.clicked() {
                    if let Some(conn) = db_conn {
                        match execute_export(conn, database_id, state.format) {
                            Ok((content, tables, rows)) => {
                                ui.ctx().copy_text(content.clone());
                                state.total_tables = tables;
                                state.total_rows = rows;
                                state.preview_snippet = Some(content);
                                let msg = format!("Exported {} tables ({} rows) to {:?} ✓", tables, rows, state.format);
                                state.last_status = Some(msg.clone());
                                toast_action = Some(msg);
                            }
                            Err(e) => {
                                let err_msg = format!("Export error: {}", e);
                                state.last_status = Some(err_msg.clone());
                                toast_action = Some(err_msg);
                            }
                        }
                    } else {
                        toast_action = Some("No database connection available".to_string());
                    }
                }

                let save_btn = ui.add(
                    egui::Button::new(RichText::new("💾 Save to Disk (vault/exports)").size(10.5).color(theme::TEXT))
                        .fill(theme::WIDGET_BG)
                        .stroke(Stroke::new(1., theme::BORDER))
                        .corner_radius(CornerRadius::same(4))
                        .min_size(Vec2::new(180., 24.)),
                );

                if save_btn.clicked() {
                    if let Some(conn) = db_conn {
                        match execute_export(conn, database_id, state.format) {
                            Ok((content, tables, rows)) => {
                                let ext = match state.format {
                                    ExportFormat::Postgres | ExportFormat::Mysql | ExportFormat::AnsiSql => "sql",
                                    ExportFormat::Json => "json",
                                    ExportFormat::Csv => "csv",
                                };
                                let dir = std::path::Path::new("vault").join("exports");
                                let _ = std::fs::create_dir_all(&dir);
                                let filename = format!("proteus_{}_{}.{}", database_id, chrono::Utc::now().format("%Y%m%d_%H%M%S"), ext);
                                let file_path = dir.join(filename);
                                match std::fs::write(&file_path, &content) {
                                    Ok(_) => {
                                        let msg = format!("Saved {} tables ({} rows) to {} ✓", tables, rows, file_path.display());
                                        state.last_status = Some(msg.clone());
                                        toast_action = Some(msg);
                                    }
                                    Err(e) => {
                                        let err_msg = format!("File write error: {}", e);
                                        state.last_status = Some(err_msg.clone());
                                        toast_action = Some(err_msg);
                                    }
                                }
                            }
                            Err(e) => {
                                let err_msg = format!("Export error: {}", e);
                                state.last_status = Some(err_msg.clone());
                                toast_action = Some(err_msg);
                            }
                        }
                    } else {
                        toast_action = Some("No database connection available".to_string());
                    }
                }
            });

            if let Some(ref status) = state.last_status {
                ui.add_space(6.);
                ui.label(RichText::new(status).size(9.5).color(theme::ACCENT_GREEN));
            }

            // Preview pane
            if let Some(ref preview) = state.preview_snippet {
                ui.add_space(8.);
                ui.separator();
                ui.add_space(4.);
                ui.label(RichText::new("EXPORT PREVIEW:").size(9.5).strong().color(theme::TEXT_DIM));
                ui.add_space(2.);

                let preview_lines = preview.lines().take(25).collect::<Vec<_>>().join("\n");
                egui::ScrollArea::vertical()
                    .max_height(140.)
                    .show(ui, |ui| {
                        ui.monospace(RichText::new(preview_lines).size(9.).color(Color32::from_rgb(180, 210, 240)));
                    });
            }
        });

    toast_action
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_export_live_sqlite() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE test_items (id TEXT PRIMARY KEY, qty INT, price REAL);
             INSERT INTO test_items VALUES ('ITEM-1', 10, 9.99);
             INSERT INTO test_items VALUES ('ITEM-2', 5, 14.50);"
        ).unwrap();

        let (pg_sql, tables, rows) = execute_export(&conn, "demo_db", ExportFormat::Postgres).unwrap();
        assert_eq!(tables, 1);
        assert_eq!(rows, 2);
        assert!(pg_sql.contains("CREATE TABLE IF NOT EXISTS \"test_items\""));
        assert!(pg_sql.contains("INSERT INTO \"test_items\""));

        let (csv_data, _, _) = execute_export(&conn, "demo_db", ExportFormat::Csv).unwrap();
        assert!(csv_data.contains("\"id\",\"qty\",\"price\""));
        assert!(csv_data.contains("\"ITEM-1\",10,9.99"));
    }
}
