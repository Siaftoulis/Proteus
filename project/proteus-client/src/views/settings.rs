//! Screen 4: Shop Profile & Hardware Printer Settings.
//! Printer name configuration, paper width selection (58mm/80mm), and test print.

use crm_core::paths::get_database_path;
use crm_core::printer::{generate_intake_receipt, print_raw_bytes, PaperWidth, ShopReceiptConfig};
use crm_core::tickets::ServiceTicket;
use egui::{Color32, CornerRadius, Frame, Margin, RichText, Stroke, Ui};
use rusqlite::Connection;

pub struct SettingsViewState {
    pub printer_name: String,
    pub test_print_msg: Option<(String, bool)>,
    pub package_mount_msg: Option<(String, bool)>,
}

impl Default for SettingsViewState {
    fn default() -> Self {
        Self {
            printer_name: "POS-80".to_string(),
            test_print_msg: None,
            package_mount_msg: None,
        }
    }
}

pub fn draw_settings_view(
    ui: &mut Ui,
    conn: &mut Connection,
    config: &mut ShopReceiptConfig,
    state: &mut SettingsViewState,
) {
    ui.vertical(|ui| {
        ui.heading(RichText::new("⚙ Ρυθμίσεις Καταστήματος & Εκτυπωτή").strong().size(22.0));
        ui.add_space(8.0);
        ui.separator();
        ui.add_space(12.0);

        if let Some((msg, is_ok)) = &state.test_print_msg {
            let col = if *is_ok {
                Color32::from_rgb(52, 211, 153)
            } else {
                Color32::from_rgb(244, 63, 94)
            };
            ui.label(RichText::new(msg).color(col).strong());
            ui.add_space(8.0);
        }

        // Section 1: Shop Profile
        Frame::new()
            .fill(crate::theme::BG_CARD)
            .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(Margin::same(16))
            .show(ui, |ui| {
                ui.label(RichText::new("ΣΤΟΙΧΕΙΑ ΕΠΙΧΕΙΡΗΣΗΣ (ΚΕΦΑΛΙΔΑ ΑΠΟΔΕΙΞΕΩΝ)").strong().color(crate::theme::TEXT_MUTED));
                ui.add_space(8.0);

                ui.columns(2, |cols| {
                    cols[0].vertical(|ui| {
                        ui.label("Επωνυμία Καταστήματος:");
                        ui.add(egui::TextEdit::singleline(&mut config.shop_name).desired_width(f32::INFINITY));
                        ui.add_space(8.0);

                        ui.label("Διεύθυνση / Πόλη:");
                        ui.add(egui::TextEdit::singleline(&mut config.address).desired_width(f32::INFINITY));
                    });

                    cols[1].vertical(|ui| {
                        ui.label("Τηλέφωνο Επικοινωνίας:");
                        ui.add(egui::TextEdit::singleline(&mut config.phone).desired_width(f32::INFINITY));
                        ui.add_space(8.0);

                        ui.label("Μήνυμα Υποσέλιδου (Footer):");
                        ui.add(egui::TextEdit::singleline(&mut config.footer_message).desired_width(f32::INFINITY));
                    });
                });
            });

        ui.add_space(14.0);

        // Section 2: Thermal Printer Setup
        Frame::new()
            .fill(crate::theme::BG_CARD)
            .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(Margin::same(16))
            .show(ui, |ui| {
                ui.label(RichText::new("ΡΥΘΜΙΣΕΙΣ ΘΕΡΜΙΚΟΥ ΕΚΤΥΠΩΤΗ (ESC/POS)").strong().color(crate::theme::TEXT_MUTED));
                ui.add_space(8.0);

                ui.columns(2, |cols| {
                    cols[0].vertical(|ui| {
                        ui.label("Όνομα Εκτυπωτή Windows (Spooler):");
                        ui.add(egui::TextEdit::singleline(&mut state.printer_name).hint_text("π.χ. POS-80 ή Generic / Text Only").desired_width(f32::INFINITY));
                        ui.label(RichText::new("Εισάγετε το ακριβές όνομα όπως φαίνεται στους Εκτυπωτές των Windows.").size(11.0).color(crate::theme::TEXT_MUTED));
                    });

                    cols[1].vertical(|ui| {
                        ui.label("Πλάτος Χαρτιού:");
                        ui.horizontal(|ui| {
                            ui.radio_value(&mut config.paper_width, PaperWidth::Width58mm, "58mm (32 στήλες)");
                            ui.radio_value(&mut config.paper_width, PaperWidth::Width80mm, "80mm (48 στήλες)");
                        });

                        ui.add_space(12.0);

                        if ui.button(RichText::new("🖨 Δοκιμαστική Εκτύπωση (Test Print)").strong()).clicked() {
                            let mut sample = ServiceTicket::new("ΔΟΚΙΜΗ ΕΚΤΥΠΩΤΗ", "210 0000000", "Δοκιμαστική Συσκευή", "Έλεγχος εκτύπωσης δελτίου");
                            sample.ticket_number = 999;
                            sample.estimated_cost = 25.0;

                            let receipt = generate_intake_receipt(&sample, config);
                            match print_raw_bytes(&state.printer_name, "Δοκιμαστική Εκτύπωση", &receipt) {
                                Ok(_) => {
                                    state.test_print_msg = Some(("✓ Η δοκιμαστική εκτύπωση εστάλη επιτυχώς!".to_string(), true));
                                }
                                Err(e) => {
                                    state.test_print_msg = Some((format!("Σφάλμα εκτύπωσης: {}", e), false));
                                }
                            }
                        }
                    });
                });
            });

        ui.add_space(14.0);

        // Section 3: Storage Info
        Frame::new()
            .fill(crate::theme::BG_CARD)
            .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(Margin::same(16))
            .show(ui, |ui| {
                ui.label(RichText::new("ΤΟΠΙΚΗ ΒΑΣΗ ΔΕΔΟΜΕΝΩΝ (ZERO-PRIVILEGE)").strong().color(crate::theme::TEXT_MUTED));
                ui.add_space(6.0);

                let path = get_database_path();
                ui.label(RichText::new(format!("Διαδρομή SQLite: {}", path.display())).size(12.0).color(crate::theme::TEXT_SECONDARY));
                ui.label(RichText::new("100% Offline-First. Όλα τα δεδομένα των πελατών και των επισκευών αποθηκεύονται τοπικά.").size(11.0).color(crate::theme::TEXT_MUTED));
            });

        ui.add_space(14.0);

        // Section 4: Declarative .pr Package Mount & Templates
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

                if let Some((msg, is_ok)) = &state.package_mount_msg {
                    let col = if *is_ok { Color32::from_rgb(52, 211, 153) } else { Color32::from_rgb(244, 63, 94) };
                    ui.label(RichText::new(msg).color(col).strong().size(12.0));
                    ui.add_space(6.0);
                }

                ui.horizontal(|ui| {
                    if ui.button("📥 Εγκατάσταση Sample Template (Επισκευές Μοτοσυκλετών)").clicked() {
                        let mut pkg = crm_core::package::PrPackage::new(
                            "PKG-MOTO-PRO",
                            "Πρότυπο Επισκευών Μοτοσυκλετών & Συνεργείου",
                            "PCD-ELITE-01",
                        );
                        pkg.schema.ddl_statements.push(
                            "CREATE TABLE IF NOT EXISTS moto_service_inspections (
                                id TEXT PRIMARY KEY,
                                vin_number TEXT NOT NULL,
                                engine_cc INTEGER,
                                tire_condition TEXT,
                                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
                            );".to_string()
                        );
                        let db_path = get_database_path();
                        match pkg.mount(conn, Some(&db_path), "Admin") {
                            Ok(summary) => {
                                state.package_mount_msg = Some((
                                    format!("✓ Επιτυχής εγκατάσταση προτύπου '{}'! Εφαρμόστηκαν {} DDL εντολές.", summary.package_name, summary.applied_ddl_count),
                                    true,
                                ));
                            }
                            Err(e) => {
                                state.package_mount_msg = Some((format!("Σφάλμα εγκατάστασης πακέτου: {}", e), false));
                            }
                        }
                    }
                });
            });
    });
}
