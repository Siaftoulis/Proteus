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
    pub fiscal_hardware: crate::views::fiscal_hardware_view::FiscalHardwareViewState,
    pub pairing_modal: crate::views::pairing_modal::PairingModalState,
    pub security_guardian: crate::views::security_guardian::SecurityGuardianState,
}

impl Default for SettingsViewState {
    fn default() -> Self {
        let machine_id = proteus_core::security::HardwareIdentity::detect().machine_id;
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
            fiscal_hardware: crate::views::fiscal_hardware_view::FiscalHardwareViewState::default(),
            pairing_modal: crate::views::pairing_modal::PairingModalState::default(),
            security_guardian: crate::views::security_guardian::SecurityGuardianState::default(),
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
                ui.label(RichText::new("ΦΟΡΟΛΟΓΙΚΗ ΤΑΥΤΟΤΗΤΑ, ΣΤΟΙΧΕΙΑ ΕΠΙΧΕΙΡΗΣΗΣ & myDATA").strong().color(crate::theme::TEXT_MUTED));
                ui.add_space(8.0);

                ui.columns(2, |cols| {
                    cols[0].vertical(|ui| {
                        ui.label("Εμπορικός Τίτλος (Top Bar & Επισκευές):");
                        ui.add(egui::TextEdit::singleline(&mut config.shop_name).desired_width(f32::INFINITY));
                        ui.add_space(6.0);

                        ui.label("Επίσημη Επωνυμία Εταιρείας (myDATA & Τιμολόγια):");
                        ui.add(egui::TextEdit::singleline(&mut config.legal_name).desired_width(f32::INFINITY));
                        ui.add_space(6.0);

                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label("Α.Φ.Μ. (9 ψηφία):");
                                ui.add(egui::TextEdit::singleline(&mut config.afm).desired_width(140.0));
                            });
                            ui.add_space(8.0);
                            ui.vertical(|ui| {
                                ui.label("Δ.Ο.Υ.:");
                                ui.add(egui::TextEdit::singleline(&mut config.doy).desired_width(140.0));
                            });
                        });
                        ui.add_space(6.0);

                        ui.label("Διεύθυνση Έδρας / Πόλη:");
                        ui.add(egui::TextEdit::singleline(&mut config.address).desired_width(f32::INFINITY));
                        ui.add_space(6.0);

                        ui.label("Τηλέφωνο Επικοινωνίας:");
                        ui.add(egui::TextEdit::singleline(&mut config.phone).desired_width(f32::INFINITY));
                        ui.add_space(6.0);

                        ui.label("Λογότυπο / Εικονίδιο Κλάδου:");
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
                    });

                    cols[1].vertical(|ui| {
                        ui.label("Δραστηριότητα / ΚΑΔ:");
                        ui.add(egui::TextEdit::singleline(&mut config.activity_description).desired_width(f32::INFINITY));
                        ui.add_space(6.0);

                        ui.label("Αριθμός Εγκατάστασης (Υποκατάστημα):");
                        let mut branch_str = config.branch_code.to_string();
                        if ui.add(egui::TextEdit::singleline(&mut branch_str).desired_width(80.0)).changed() {
                            if let Ok(val) = branch_str.parse::<i32>() {
                                config.branch_code = val;
                            }
                        }
                        ui.add_space(6.0);

                        ui.label("AADE myDATA User ID:");
                        ui.add(egui::TextEdit::singleline(&mut config.mydata_user_id).hint_text("π.χ. username_aade").desired_width(f32::INFINITY));
                        ui.add_space(6.0);

                        ui.label("AADE myDATA Subscription Key:");
                        ui.add(egui::TextEdit::singleline(&mut config.mydata_subscription_key).password(true).desired_width(f32::INFINITY));
                        ui.add_space(6.0);

                        ui.checkbox(&mut config.mydata_sandbox, "myDATA Sandbox (Δοκιμαστικό Περιβάλλον AADE)");
                        ui.add_space(6.0);

                        ui.label("Μήνυμα Υποσέλιδου (Footer Αποδείξεων):");
                        ui.add(egui::TextEdit::singleline(&mut config.footer_message).desired_width(f32::INFINITY));
                        ui.add_space(8.0);

                        if ui.button(RichText::new("💾 Αποθήκευση Φορολογικών Στοιχείων Επιχείρησης").strong()).clicked() {
                            match proteus_core::printer::save_shop_config(conn, config) {
                                Ok(_) => {
                                    state.save_shop_msg = Some(("✓ Τα φορολογικά στοιχεία και το προφίλ της επιχείρησης αποθηκεύτηκαν επιτυχώς στη SQLite!".to_string(), true));
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

        // Section 2b: Fiscal Hardware (ΦΗΜ) & EFT-POS Settings (A.1155/2023)
        crate::views::fiscal_hardware_view::draw_fiscal_hardware_panel(ui, conn, &mut state.fiscal_hardware);

        ui.add_space(14.0);

        // Section 2c: Mobile Companion QR Pairing
        Frame::new()
            .fill(crate::theme::BG_CARD)
            .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(Margin::same(16))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("📱 ΣΥΖΕΥΞΗ ΦΟΡΗΤΩΝ ΣΥΣΚΕΥΩΝ (MOBILE COMPANION QR)").strong().color(crate::theme::TEXT_MUTED));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(RichText::new("📷 Έκδοση QR Σύζευξης").strong().color(crate::theme::ACCENT_CYAN)).clicked() {
                            state.pairing_modal.open_and_refresh(conn, &config.shop_name, &config.shop_name);
                        }
                    });
                });
                ui.add_space(4.0);
                ui.label(RichText::new("Δημιουργήστε κρυπτογραφικό QR token για άμεση σύνδεση του smartphone συνεργάτη (Outbox Sync, Van Sales, Παραλαβές).").size(11.5).color(crate::theme::TEXT_SECONDARY));
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
        crate::views::package_mount_view::draw_package_mount_section(ui, conn, &mut state.package_mount_msg);

        ui.add_space(14.0);

        // Section 5: License Management & Activation
        crate::views::license_view::draw_license_section(ui, config, state);

        ui.add_space(14.0);

        // Section 5b: Security Guardian & Hardware-Sealed Vault
        crate::views::security_guardian::draw_security_guardian_section(
            ui,
            conn,
            &mut state.security_guardian,
            &state.machine_id,
        );

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
