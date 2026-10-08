//! License Management & Hardware-Lock Activation Section for Proteus BOS.
//! Strict Rule 1 (100% Original Codebase) and Rule 3 (<400 lines).

use egui::{Color32, CornerRadius, Frame, Margin, RichText, Stroke, Ui};
use proteus_core::printer::ShopReceiptConfig;
use crate::views::settings::SettingsViewState;

pub fn draw_license_section(
    ui: &mut Ui,
    config: &ShopReceiptConfig,
    state: &mut SettingsViewState,
) {
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
}
