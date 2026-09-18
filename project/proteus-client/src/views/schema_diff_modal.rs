//! Visual Schema Diffing & Migration Confirmation Modal for Proteus Client.
//! Designed from first principles for zero-friction review of schema modifications.

use crm_core::migrations::MigrationRunner;
use crm_core::package::PrPackage;
use crm_core::paths::get_database_path;
use crm_core::schema_diff::{ColumnChangeKind, DiffSafety, SchemaDiff, TableChangeKind};
use egui::{Color32, CornerRadius, Frame, Margin, RichText, Stroke, Ui};
use rusqlite::Connection;

#[derive(Debug, Clone)]
pub struct PendingMigrationReview {
    pub source_title: String,
    pub author: String,
    pub version: String,
    pub diff: SchemaDiff,
    pub ddl_statements: Vec<String>,
    pub package_to_mount: Option<PrPackage>,
    pub dry_run_ok: bool,
    pub dry_run_error: Option<String>,
}

impl PendingMigrationReview {
    pub fn from_package(conn: &Connection, pkg: PrPackage) -> Result<Self, String> {
        let diff = SchemaDiff::from_ddl(conn, &pkg.schema.ddl_statements)?;
        let dry_run_conn = Connection::open_in_memory().map_err(|e| e.to_string())?;

        // Replicate live tables for dry-run
        let live_tables = SchemaDiff::inspect_connection(conn)?;
        for t in &live_tables {
            if t.columns.is_empty() {
                continue;
            }
            let col_defs: Vec<String> = t
                .columns
                .iter()
                .map(|c| {
                    let pk = if c.is_primary_key { " PRIMARY KEY" } else { "" };
                    let not_null = if !c.is_nullable { " NOT NULL" } else { "" };
                    format!("\"{}\" {}{}{}", c.name, c.data_type, pk, not_null)
                })
                .collect();
            let sql = format!("CREATE TABLE \"{}\" ({});", t.name, col_defs.join(", "));
            let _ = dry_run_conn.execute_batch(&sql);
        }

        let mut dry_run_ok = true;
        let mut dry_run_error = None;
        for ddl in &pkg.schema.ddl_statements {
            if let Err(e) = dry_run_conn.execute_batch(ddl) {
                dry_run_ok = false;
                dry_run_error = Some(format!("Dry-run error on '{}': {}", ddl, e));
                break;
            }
        }

        Ok(Self {
            source_title: pkg.manifest.name.clone(),
            author: pkg.manifest.author_pcd_id.clone(),
            version: pkg.manifest.version.clone(),
            diff,
            ddl_statements: pkg.schema.ddl_statements.clone(),
            package_to_mount: Some(pkg),
            dry_run_ok,
            dry_run_error,
        })
    }
}

pub fn draw_schema_diff_modal(
    ctx: &egui::Context,
    conn: &mut Connection,
    pending: &mut Option<PendingMigrationReview>,
    status_msg: &mut Option<(String, bool)>,
) {
    let review = match pending {
        Some(r) => r,
        None => return,
    };

    let mut should_apply = false;
    let mut should_dismiss = false;

    egui::Window::new("🔍 Έλεγχος & Επιβεβαίωση Μετάβασης Σχήματος (Schema Diff)")
        .collapsible(false)
        .resizable(true)
        .default_width(620.0)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .frame(
            Frame::new()
                .fill(crate::theme::BG_PANEL)
                .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
                .corner_radius(CornerRadius::same(10))
                .inner_margin(Margin::same(18)),
        )
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                // Header Info
                ui.horizontal(|ui| {
                    ui.label(RichText::new("📦 ΠΑΚΕΤΟ:").strong().size(11.0).color(crate::theme::TEXT_MUTED));
                    ui.label(RichText::new(&review.source_title).strong().size(14.0).color(Color32::WHITE));
                    ui.label(RichText::new(format!("v{}", review.version)).size(12.0).color(crate::theme::ACCENT_CYAN));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(format!("Συντάκτης: {}", review.author)).size(11.0).color(crate::theme::TEXT_MUTED));
                    });
                });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                // Safety Status Banner
                match review.diff.overall_safety {
                    DiffSafety::SafeAdditive => {
                        Frame::new()
                            .fill(Color32::from_rgb(16, 50, 35))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(52, 211, 153)))
                            .corner_radius(CornerRadius::same(6))
                            .inner_margin(Margin::symmetric(12, 8))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("🛡 100% ΑΣΦΑΛΗΣ ΜΕΤΑΒΑΣΗ (Strictly Additive)").strong().color(Color32::from_rgb(52, 211, 153)));
                                    ui.label(RichText::new("— Μηδενικός κίνδυνος απώλειας δεδομένων").size(11.0).color(Color32::from_rgb(167, 243, 208)));
                                });
                            });
                    }
                    DiffSafety::CautionTypeChange => {
                        Frame::new()
                            .fill(Color32::from_rgb(55, 35, 10))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(251, 146, 60)))
                            .corner_radius(CornerRadius::same(6))
                            .inner_margin(Margin::symmetric(12, 8))
                            .show(ui, |ui| {
                                ui.label(RichText::new("⚡ ΠΡΟΣΟΧΗ: Περιλαμβάνει τροποποίηση τύπου δεδομένων").strong().color(Color32::from_rgb(251, 146, 60)));
                            });
                    }
                    DiffSafety::DestructiveDrop => {
                        Frame::new()
                            .fill(Color32::from_rgb(60, 20, 20))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(239, 68, 68)))
                            .corner_radius(CornerRadius::same(6))
                            .inner_margin(Margin::symmetric(12, 8))
                            .show(ui, |ui| {
                                ui.label(RichText::new("⚠️ ΚΙΝΔΥΝΟΣ: Εντοπίστηκαν διαγραφές πινάκων ή πεδίων!").strong().color(Color32::from_rgb(239, 68, 68)));
                            });
                    }
                }

                ui.add_space(8.0);

                // Verification & Snapshot Pills
                ui.horizontal(|ui| {
                    if review.dry_run_ok {
                        ui.label(RichText::new("✓ Dry-Run Verified").size(11.0).color(Color32::from_rgb(52, 211, 153)));
                    } else if let Some(ref err) = review.dry_run_error {
                        ui.label(RichText::new(format!("❌ Dry-Run Error: {}", err)).size(11.0).color(Color32::from_rgb(239, 68, 68)));
                    }
                    ui.separator();
                    ui.label(RichText::new("💾 Αυτόματο Snapshot (.bak) ενεργό").size(11.0).color(crate::theme::TEXT_MUTED));
                });

                ui.add_space(8.0);

                // Summary Numbers
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("+{} Νέοι Πίνακες", review.diff.new_tables_count)).strong().color(Color32::from_rgb(52, 211, 153)));
                    ui.label(RichText::new("•").color(crate::theme::TEXT_MUTED));
                    ui.label(RichText::new(format!("+{} Νέες Στήλες", review.diff.new_columns_count)).strong().color(crate::theme::ACCENT_CYAN));
                    if review.diff.dropped_count > 0 {
                        ui.label(RichText::new("•").color(crate::theme::TEXT_MUTED));
                        ui.label(RichText::new(format!("-{} Διαγραφές", review.diff.dropped_count)).strong().color(Color32::from_rgb(239, 68, 68)));
                    }
                });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                // Scrollable Diff Tree
                egui::ScrollArea::vertical().max_height(240.0).show(ui, |ui| {
                    render_diff_table(ui, &review.diff);
                });

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(10.0);

                // Action Buttons
                ui.horizontal(|ui| {
                    let can_apply = review.dry_run_ok && review.diff.overall_safety != DiffSafety::DestructiveDrop;
                    let apply_btn = egui::Button::new(RichText::new("🚀 Επιβεβαίωση & Εφαρμογή (Apply Migration)").strong().color(Color32::WHITE))
                        .fill(if can_apply { crate::theme::ACCENT_PRIMARY } else { Color32::from_rgb(80, 80, 80) })
                        .min_size(egui::vec2(240.0, 32.0));

                    if ui.add_enabled(can_apply, apply_btn).clicked() {
                        should_apply = true;
                    }

                    if ui.add(egui::Button::new(RichText::new("✕ Απόρριψη (Reject)").color(crate::theme::TEXT_SECONDARY))
                        .fill(crate::theme::BG_CARD)
                        .min_size(egui::vec2(100.0, 32.0))).clicked()
                    {
                        should_dismiss = true;
                    }
                });
            });
        });

    if should_dismiss {
        *pending = None;
    } else if should_apply {
        if let Some(mut rev) = pending.take() {
            let db_path = get_database_path();
            if let Some(pkg) = rev.package_to_mount.take() {
                match pkg.mount(conn, Some(&db_path), &rev.author) {
                    Ok(summary) => {
                        *status_msg = Some((
                            format!("✓ Επιτυχής αναβάθμιση σχήματος: '{}' ({} DDL)", summary.package_name, summary.applied_ddl_count),
                            true,
                        ));
                    }
                    Err(e) => {
                        *status_msg = Some((format!("❌ Σφάλμα εγκατάστασης: {}", e), false));
                    }
                }
            } else if !rev.ddl_statements.is_empty() {
                let mut plan = crm_core::migrations::MigrationPlan::new(
                    "MANUAL-DIFF-APPLY",
                    rev.source_title.clone(),
                    "Developer",
                );
                for ddl in rev.ddl_statements {
                    plan.add_statement(ddl);
                }
                match MigrationRunner::execute(conn, &plan, Some(&db_path)) {
                    Ok(res) => {
                        *status_msg = Some((
                            format!("✓ Το σχήμα εφαρμόστηκε επιτυχώς ({}ms)", res.elapsed_ms),
                            true,
                        ));
                    }
                    Err(e) => {
                        *status_msg = Some((format!("❌ Σφάλμα εκτέλεσης: {}", e), false));
                    }
                }
            }
        }
    }
}

fn render_diff_table(ui: &mut Ui, diff: &SchemaDiff) {
    for t_diff in &diff.table_diffs {
        let (icon, label_color) = match t_diff.kind {
            TableChangeKind::Added => ("+ ПΙΝΑΚΑΣ", Color32::from_rgb(52, 211, 153)),
            TableChangeKind::Modified => ("⚡ ΠΙΝΑΚΑΣ", Color32::from_rgb(251, 146, 60)),
            TableChangeKind::Unchanged => ("● ΠΙΝΑΚΑΣ", crate::theme::TEXT_MUTED),
            TableChangeKind::Dropped => ("- ΠΙΝΑΚΑΣ", Color32::from_rgb(239, 68, 68)),
        };

        ui.horizontal(|ui| {
            ui.label(RichText::new(icon).size(10.0).color(label_color).strong());
            ui.label(RichText::new(&t_diff.table_name).strong().color(Color32::WHITE));
        });

        ui.indent(format!("col_{}", t_diff.table_name), |ui| {
            for c in &t_diff.column_diffs {
                match &c.kind {
                    ColumnChangeKind::Added { data_type, is_nullable } => {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("+").size(11.0).color(Color32::from_rgb(52, 211, 153)).strong());
                            ui.label(RichText::new(&c.column_name).color(Color32::from_rgb(52, 211, 153)).strong());
                            ui.label(RichText::new(data_type).size(10.0).color(crate::theme::TEXT_MUTED));
                            if !*is_nullable {
                                ui.label(RichText::new("NOT NULL").size(9.0).color(Color32::from_rgb(251, 146, 60)));
                            }
                        });
                    }
                    ColumnChangeKind::Modified { old_type, new_type } => {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("⚡").size(11.0).color(Color32::from_rgb(251, 146, 60)).strong());
                            ui.label(RichText::new(&c.column_name).color(Color32::from_rgb(251, 146, 60)));
                            ui.label(RichText::new(format!("{} → {}", old_type, new_type)).size(10.0).color(crate::theme::ACCENT_CYAN));
                        });
                    }
                    ColumnChangeKind::Dropped { old_type } => {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("—").size(11.0).color(Color32::from_rgb(239, 68, 68)).strong());
                            ui.label(RichText::new(&c.column_name).color(Color32::from_rgb(239, 68, 68)));
                            ui.label(RichText::new(old_type).size(10.0).color(crate::theme::TEXT_MUTED));
                        });
                    }
                    ColumnChangeKind::Unchanged { data_type } => {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(" ").size(11.0));
                            ui.label(RichText::new(&c.column_name).color(crate::theme::TEXT_MUTED));
                            ui.label(RichText::new(data_type).size(10.0).color(Color32::from_rgb(100, 100, 100)));
                        });
                    }
                }
            }
        });
        ui.add_space(4.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pending_migration_review_from_package() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE inventory (id TEXT PRIMARY KEY, qty INTEGER);").unwrap();

        let mut pkg = PrPackage::new("PKG-TEST", "Inventory Extension", "Analyst");
        pkg.schema.ddl_statements.push("ALTER TABLE inventory ADD COLUMN min_threshold INTEGER;".to_string());
        pkg.schema.ddl_statements.push("CREATE TABLE suppliers (id TEXT PRIMARY KEY, name TEXT);".to_string());

        let review = PendingMigrationReview::from_package(&conn, pkg).expect("Review creation should succeed");
        assert_eq!(review.source_title, "Inventory Extension");
        assert!(review.dry_run_ok);
        assert_eq!(review.diff.new_tables_count, 1);
        assert_eq!(review.diff.new_columns_count, 3);
        assert_eq!(review.diff.overall_safety, DiffSafety::SafeAdditive);
    }
}

