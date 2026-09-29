//! Screen 4: Shop Profile & Hardware Printer Settings.
//! Printer name configuration, paper width selection (58mm/80mm), and test print.

use proteus_core::paths::get_database_path;
use proteus_core::printer::{generate_intake_receipt, print_raw_bytes, PaperWidth, ShopReceiptConfig};
use proteus_core::tickets::ServiceTicket;
use egui::{Color32, CornerRadius, Frame, Margin, RichText, Stroke, Ui};
use rusqlite::Connection;

pub struct SettingsViewState {
    pub printer_name: String,
    pub test_print_msg: Option<(String, bool)>,
    pub package_mount_msg: Option<(String, bool)>,
    pub license_key: String,
    pub machine_id: String,
    pub license_info: Option<proteus_core::license::LicenseInfo>,
    pub license_msg: Option<(String, bool)>,
    pub save_shop_msg: Option<(String, bool)>,
}

impl Default for SettingsViewState {
    fn default() -> Self {
        let machine_id = format!("MACHINE-{:x}", {
            use std::hash::{Hash, Hasher};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            std::env::var("COMPUTERNAME").unwrap_or_else(|_| "DEFAULT-HOST".into()).hash(&mut hasher);
            hasher.finish()
        });
        let cached = proteus_core::license::get_license_info(None, &machine_id);
        Self {
            printer_name: "POS-80".to_string(),
            test_print_msg: None,
            package_mount_msg: None,
            license_key: String::new(),
            machine_id,
            license_info: if cached.valid { Some(cached) } else { None },
            license_msg: None,
            save_shop_msg: None,
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

        if let Some((msg, is_ok)) = &state.save_shop_msg {
            let col = if *is_ok {
                Color32::from_rgb(52, 211, 153)
            } else {
                Color32::from_rgb(244, 63, 94)
            };
            ui.label(RichText::new(msg).color(col).strong());
            ui.add_space(8.0);
        }

        // Section 1: Shop Profile & Store Branding
        Frame::new()
            .fill(crate::theme::BG_CARD)
            .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(Margin::same(16))
            .show(ui, |ui| {
                ui.label(RichText::new("ΣΤΟΙΧΕΙΑ ΕΠΙΧΕΙΡΗΣΗΣ & CUSTOM LOGO (ΤΑΥΤΟΤΗΤΑ ΚΑΤΑΣΤΗΜΑΤΟΣ)").strong().color(crate::theme::TEXT_MUTED));
                ui.add_space(8.0);

                ui.columns(2, |cols| {
                    cols[0].vertical(|ui| {
                        ui.label("Επωνυμία Καταστήματος (Εμφάνιση στο Top Bar & Αποδείξεις):");
                        ui.add(egui::TextEdit::singleline(&mut config.shop_name).desired_width(f32::INFINITY));
                        ui.add_space(8.0);

                        ui.label("Διεύθυνση / Πόλη:");
                        ui.add(egui::TextEdit::singleline(&mut config.address).desired_width(f32::INFINITY));
                        ui.add_space(8.0);

                        ui.label("Λογότυπο / Εικονίδιο Κλάδου Επιχείρησης:");
                        egui::ComboBox::from_id_salt("shop_logo_combo")
                            .selected_text(match config.logo_icon.as_str() {
                                "wrench" => "Service & Workshop / Επισκευές",
                                "hardware" => "Hardware & Fasteners / Σιδηρικά",
                                "anchor" => "Marine & Nautical / Ναυτιλιακά",
                                "auto" => "Auto Garage / Συνεργείο Αυτοκινήτων",
                                "store" => "General Store & Trades / Εμπορικό",
                                "default" => "King Proteus Emblem (Official)",
                                other => other,
                            })
                            .width(f32::INFINITY)
                            .show_ui(ui, |ui| {
                                for (val, label) in crate::views::logo::STORE_ICONS {
                                    ui.selectable_value(&mut config.logo_icon, val.to_string(), *label);
                                }
                            });
                        ui.label(RichText::new("Ή πληκτρολογήστε 2 γράμματα για αυτόματο Monogram αρχικών:").size(11.0).color(crate::theme::TEXT_MUTED));
                        ui.add(egui::TextEdit::singleline(&mut config.logo_icon).hint_text("π.χ. AL ή wrench").desired_width(120.0));
                    });

                    cols[1].vertical(|ui| {
                        ui.label("Τηλέφωνο Επικοινωνίας:");
                        ui.add(egui::TextEdit::singleline(&mut config.phone).desired_width(f32::INFINITY));
                        ui.add_space(8.0);

                        ui.label("Μήνυμα Υποσέλιδου (Footer Αποδείξεων):");
                        ui.add(egui::TextEdit::singleline(&mut config.footer_message).desired_width(f32::INFINITY));
                        ui.add_space(10.0);

                        ui.label(RichText::new("ΠΡΟΕΠΙΣΚΟΠΗΣΗ ΤΑΥΤΟΤΗΤΑΣ:").size(11.0).color(crate::theme::TEXT_MUTED).strong());
                        Frame::new()
                            .fill(crate::theme::BG_PANEL)
                            .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
                            .corner_radius(CornerRadius::same(6))
                            .inner_margin(Margin::same(10))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    crate::views::logo::render_store_logo_widget(ui, egui::vec2(28.0, 28.0), &config.logo_icon, &config.shop_name, true);
                                    ui.add_space(8.0);
                                    ui.vertical(|ui| {
                                        ui.label(RichText::new(&config.shop_name).strong().size(14.0).color(crate::theme::TEXT_PRIMARY));
                                        ui.label(RichText::new(&config.phone).size(11.0).color(crate::theme::TEXT_MUTED));
                                    });
                                });
                            });

                        ui.add_space(10.0);
                        if ui.button(RichText::new("💾 Αποθήκευση Στοιχείων Καταστήματος").strong()).clicked() {
                            match proteus_core::printer::save_shop_config(conn, config) {
                                Ok(_) => {
                                    state.save_shop_msg = Some(("✓ Τα στοιχεία και το λογότυπο του καταστήματος αποθηκεύτηκαν επιτυχώς στη SQLite!".to_string(), true));
                                }
                                Err(e) => {
                                    state.save_shop_msg = Some((format!("Σφάλμα αποθήκευσης: {}", e), false));
                                }
                            }
                        }
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
                                                state.package_mount_msg = Some((
                                                    format!("✓ Επιτυχής εγκατάσταση '{}'! DDL εντολές: {}.", summary.package_name, summary.applied_ddl_count),
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
                    ui.add_space(4.0);
                }
            });

        ui.add_space(14.0);

        // Section 5: License Management & Activation
        Frame::new()
            .fill(crate::theme::BG_CARD)
            .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(Margin::same(16))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("🔑 ΑΔΕΙΑ ΧΡΗΣΗΣ & ΕΝΕΡΓΟΠΟΙΗΣΗ").strong().color(crate::theme::ACCENT_CYAN));
                    ui.label(RichText::new("(Hardware Lock & Offline Fallback)").size(11.0).color(crate::theme::TEXT_MUTED));
                });
                ui.add_space(6.0);
                ui.label(RichText::new("Διαχείριση επιχειρησιακής άδειας χρήσης και επαλήθευση ενεργοποίησης έναντι του License Server.").size(12.0).color(crate::theme::TEXT_MUTED));
                ui.add_space(8.0);

                if let Some((msg, is_ok)) = &state.license_msg {
                    let col = if *is_ok { Color32::from_rgb(52, 211, 153) } else { Color32::from_rgb(244, 63, 94) };
                    ui.label(RichText::new(msg).color(col).strong().size(12.0));
                    ui.add_space(6.0);
                }

                ui.columns(2, |cols| {
                    cols[0].vertical(|ui| {
                        ui.label("Κλειδί Άδειας (License Key):");
                        ui.add(egui::TextEdit::singleline(&mut state.license_key).hint_text("π.χ. PRO-XXXX-XXXX-XXXX").desired_width(f32::INFINITY));
                        ui.add_space(6.0);

                        ui.label("Αναγνωριστικό Μηχανήματος (Machine ID):");
                        ui.add(egui::TextEdit::singleline(&mut state.machine_id).desired_width(f32::INFINITY));
                    });

                    cols[1].vertical(|ui| {
                        ui.label("Τρέχουσα Κατάσταση Άδειας:");
                        if let Some(ref info) = state.license_info {
                            if info.valid {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("● ΕΝΕΡΓΗ ΑΔΕΙΑ").color(Color32::from_rgb(52, 211, 153)).strong());
                                    ui.label(format!("(Έως {} χρήστες)", info.max_users));
                                });
                                if !info.expires_at.is_empty() {
                                    ui.label(RichText::new(format!("Ημερομηνία Λήξης: {}", info.expires_at)).size(11.0).color(crate::theme::TEXT_SECONDARY));
                                }
                            } else {
                                ui.label(RichText::new("○ ΔΩΡΕΑΝ ΕΚΔΟΣΗ (Community — Έως 3 χρήστες)").color(crate::theme::TEXT_MUTED).strong());
                            }
                        } else {
                            ui.label(RichText::new("○ ΔΩΡΕΑΝ ΕΚΔΟΣΗ (Community — Έως 3 χρήστες)").color(crate::theme::TEXT_MUTED).strong());
                        }

                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            let activate_btn = egui::Button::new(RichText::new("🔑 Ενεργοποίηση").strong().color(Color32::WHITE))
                                .fill(crate::theme::ACCENT_PRIMARY);
                            if ui.add(activate_btn).clicked() {
                                let key = state.license_key.trim();
                                if key.is_empty() {
                                    state.license_msg = Some(("❌ Εισάγετε έγκυρο κλειδί άδειας ή offline token.".into(), false));
                                } else if key.starts_with("PROT-LIC-") {
                                    match proteus_core::license::install_offline_token(key, &state.machine_id) {
                                        Ok(info) => {
                                            proteus_core::audio::play_barcode_chime();
                                            state.license_info = Some(info.clone());
                                            state.license_msg = Some((format!("✓ Επιτυχής εγκατάσταση offline άδειας ({} χρήστες)! Λήξη: {}", info.max_users, info.expires_at), true));
                                        }
                                        Err(e) => {
                                            proteus_core::audio::play_error_tone();
                                            state.license_msg = Some((format!("❌ Σφάλμα offline άδειας: {}", e), false));
                                        }
                                    }
                                } else {
                                    match proteus_core::license::verify_online(key, &state.machine_id) {
                                        Ok(info) => {
                                            let is_val = info.valid;
                                            state.license_info = Some(info.clone());
                                            if is_val {
                                                proteus_core::audio::play_barcode_chime();
                                                state.license_msg = Some((format!("✓ Επιτυχής ενεργοποίηση άδειας! Μέγιστοι χρήστες: {}", info.max_users), true));
                                            } else {
                                                proteus_core::audio::play_error_tone();
                                                state.license_msg = Some(("❌ Το κλειδί δεν είναι έγκυρο ή έχει εξαντληθεί το όριο ενεργοποιήσεων.".into(), false));
                                            }
                                        }
                                        Err(e) => {
                                            proteus_core::audio::play_error_tone();
                                            state.license_msg = Some((format!("❌ Αποτυχία σύνδεσης με τον License Server: {}", e), false));
                                        }
                                    }
                                }
                            }

                            if ui.button("⚡ Έκδοση Pilot Token (30 Ημέρες)").clicked() {
                                let shop = if config.shop_name.is_empty() { "Pilot Workshop" } else { &config.shop_name };
                                match proteus_core::license::issue_pilot_token(shop, "000000000", Some(&state.machine_id), 30) {
                                    Ok(token) => {
                                        proteus_core::audio::play_affirm_tone();
                                        state.license_key = token;
                                        state.license_msg = Some(("✓ Παρήχθη νέο υπογεγραμμένο Pilot Token 30 ημερών. Πατήστε 'Ενεργοποίηση'.".into(), true));
                                    }
                                    Err(e) => {
                                        proteus_core::audio::play_error_tone();
                                        state.license_msg = Some((format!("❌ Σφάλμα παραγωγής: {}", e), false));
                                    }
                                }
                            }

                            if ui.button("🔄 Έλεγχος Cache").clicked() {
                                let info = proteus_core::license::get_license_info(None, &state.machine_id);
                                let is_val = info.valid;
                                state.license_info = Some(info.clone());
                                if is_val {
                                    state.license_msg = Some((format!("✓ Φορτώθηκε έγκυρη άδεια από την τοπική μνήμη ({} χρήστες).", info.max_users), true));
                                } else {
                                    state.license_msg = Some(("Δεν βρέθηκε αποθηκευμένη έγκυρη άδεια στην τοπική μνήμη.".into(), false));
                                }
                            }
                        });
                    });
                });
            });

        ui.add_space(14.0);

        // Section 6: Sovereign Brand & System Identity Card
        Frame::new()
            .fill(crate::theme::BG_CARD)
            .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(Margin::same(16))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    crate::views::logo::render_store_logo_widget(ui, egui::vec2(44.0, 44.0), "default", "Proteus Business OS", true);
                    ui.add_space(12.0);
                    ui.vertical(|ui| {
                        ui.label(RichText::new("PROTEUS BUSINESS OS // SYSTEM IDENTITY").strong().color(crate::theme::TEXT_PRIMARY).size(13.0));
                        ui.label(RichText::new("King Proteus Sovereign Core Engine • Local SQLite Store • 100% Bespoke Architecture").size(11.0).color(crate::theme::TEXT_MUTED));
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Core: v2.2.0-sovereign").size(10.0).color(crate::theme::ACCENT_CYAN));
                            ui.label(RichText::new("•").size(10.0).color(crate::theme::TEXT_MUTED));
                            ui.label(RichText::new("Status: Live Real SQLite Connected").size(10.0).color(Color32::from_rgb(52, 211, 153)));
                        });
                    });
                });
            });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_view_state_defaults() {
        let state = SettingsViewState::default();
        assert_eq!(state.printer_name, "POS-80");
        assert!(!state.machine_id.is_empty());
        assert!(state.license_key.is_empty());
        assert!(state.save_shop_msg.is_none());
        assert!(!crate::views::logo::STORE_ICONS.is_empty());
    }
}
