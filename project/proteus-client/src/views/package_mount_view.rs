//! Declarative .pr Package Mount & Store Industry Templates Component.
//! Extracted for clean modularity (<400 lines per file).

use egui::{Color32, CornerRadius, Frame, Margin, RichText, Stroke, Ui};
use proteus_core::paths::get_database_path;
use rusqlite::Connection;

pub fn draw_package_mount_section(
    ui: &mut Ui,
    conn: &mut Connection,
    package_mount_msg: &mut Option<(String, bool)>,
) {
    Frame::new()
        .fill(crate::theme::BG_CARD)
        .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::same(16))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("📦 ΕΓΚΑΤΑΣΤΑΣΗ ΠΡΟΤΥΠΩΝ ΚΑΤΑΣΤΗΜΑΤΟΣ (.PR PACKAGES)").strong().color(crate::theme::ACCENT_CYAN));
                ui.label(RichText::new("(4-Step Ingestion & Tamper-Proof)").size(11.0).color(crate::theme::TEXT_MUTED));
            });
            ui.add_space(6.0);
            ui.label(RichText::new("Εγκατάσταση επαληθευμένων επιχειρησιακών templates σχεδιασμένων από πιστοποιημένους Designers (PCD).").size(12.0).color(crate::theme::TEXT_MUTED));
            ui.add_space(8.0);

            if let Some((msg, is_ok)) = package_mount_msg {
                let col = if *is_ok { Color32::from_rgb(52, 211, 153) } else { Color32::from_rgb(244, 63, 94) };
                ui.label(RichText::new(msg.as_str()).color(col).strong().size(12.0));
                ui.add_space(6.0);
            }

            ui.horizontal(|ui| {
                ui.label(RichText::new("Επαληθευμένα Επιχειρησιακά Πρότυπα (.pr) έτοιμα για άμεση προσάρτηση:").size(12.0).color(crate::theme::TEXT_MUTED));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("🌐 Web Marketplace Hub (Port 8080)").clicked() {
                        ui.ctx().open_url(egui::OpenUrl::new_tab("http://localhost:8080"));
                    }
                });
            });
            ui.add_space(6.0);

            let templates = [
                (
                    "PKG-MOTO-PRO",
                    "Επισκευές Μοτοσυκλετών & Συνεργείο",
                    "PCD Specialist",
                    "Automotive",
                    "CREATE TABLE IF NOT EXISTS moto_service_inspections (id TEXT PRIMARY KEY, vin_number TEXT NOT NULL, engine_cc INTEGER, tire_condition TEXT, created_at DATETIME DEFAULT CURRENT_TIMESTAMP);",
                ),
                (
                    "PKG-SERVICE-AUTO",
                    "Συνεργείο Αυτοκινήτων & Ανταλλακτικά",
                    "PCD Senior Partner",
                    "Automotive",
                    "CREATE TABLE IF NOT EXISTS auto_service_jobs (id TEXT PRIMARY KEY, license_plate TEXT NOT NULL, mileage INTEGER, mechanic_notes TEXT, created_at DATETIME DEFAULT CURRENT_TIMESTAMP);",
                ),
                (
                    "PKG-RETAIL-POS",
                    "Λιανική & Ταμειακή Διαχείριση POS",
                    "PCDA Analyst Group",
                    "Retail",
                    "CREATE TABLE IF NOT EXISTS retail_inventory_sync (id TEXT PRIMARY KEY, sku TEXT NOT NULL, barcode TEXT NOT NULL, stock_qty INTEGER, reorder_level INTEGER, last_scanned DATETIME);",
                ),
                (
                    "PKG-CLINIC-HEALTH",
                    "Ιατρεία & Οδοντιατρική Διαχείριση",
                    "PCSS Systems Architect",
                    "Healthcare",
                    "CREATE TABLE IF NOT EXISTS clinic_patient_records (id TEXT PRIMARY KEY, amka TEXT NOT NULL, full_name TEXT NOT NULL, diagnosis_notes TEXT, consent_gdpr INTEGER DEFAULT 1, created_at DATETIME DEFAULT CURRENT_TIMESTAMP);",
                ),
            ];

            for (bundle_id, title, author, category, ddl) in templates {
                Frame::new()
                    .fill(crate::theme::BG_PANEL)
                    .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(Margin::same(10))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(title).strong().size(12.0).color(Color32::WHITE));
                            ui.label(RichText::new(format!("({})", category)).size(10.0).color(crate::theme::ACCENT_CYAN));
                            ui.label(RichText::new(format!("by {}", author)).size(10.0).color(crate::theme::TEXT_MUTED));
                            
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button(format!("📥 Προσάρτηση {}", bundle_id)).clicked() {
                                    let mut pkg = proteus_core::package::PrPackage::new(bundle_id, title, author);
                                    pkg.schema.ddl_statements.push(ddl.to_string());
                                    let db_path = get_database_path();
                                    match pkg.mount(conn, Some(&db_path), "Admin") {
                                        Ok(summary) => {
                                            *package_mount_msg = Some((
                                                format!("✓ Επιτυχής εγκατάσταση '{}'! DDL εντολές: {}.", summary.package_name, summary.applied_ddl_count),
                                                true,
                                            ));
                                        }
                                        Err(e) => {
                                            *package_mount_msg = Some((format!("Σφάλμα εγκατάστασης πακέτου: {}", e), false));
                                        }
                                    }
                                }
                            });
                        });
                    });
                ui.add_space(4.0);
            }
        });
}
