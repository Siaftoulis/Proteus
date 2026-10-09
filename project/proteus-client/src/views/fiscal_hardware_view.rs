//! Fiscal Hardware (ΦΗΜ) & EFT-POS Terminal Settings View (AADE A.1155/2023).
//! Direct configuration, TCP/IP LAN ping, A.1155 card handshake, and Batch Close.
//! Strict Rule 1 (100% Original Codebase), Rule 2 (Minimalist UX), Rule 3 (<400 lines), Rule 5 (Zero Mock Data).

use egui::{Color32, CornerRadius, Frame, Margin, RichText, Stroke, Ui};
use proteus_core::fiscal_pos::{
    format_a1155_receipt_slip, load_fiscal_pos_config, record_pos_card_transaction,
    save_fiscal_pos_config, EftPosDriver, FiscalDeviceType, FiscalHardwareConfig,
    PosInitiateRequest, PosTerminalResponse, PosTransactionStatus, PosTransactionType,
    TerminalBatchSummary,
};
use rusqlite::Connection;

#[derive(Default)]
pub struct FiscalHardwareViewState {
    pub config: FiscalHardwareConfig,
    pub loaded: bool,
    pub test_feedback: Option<(String, bool)>,
    pub save_feedback: Option<(String, bool)>,
    pub last_response: Option<PosTerminalResponse>,
    pub last_batch: Option<TerminalBatchSummary>,
    pub ping_status: Option<bool>,
}

pub fn draw_fiscal_hardware_panel(
    ui: &mut Ui,
    conn: &Connection,
    state: &mut FiscalHardwareViewState,
) {
    if !state.loaded {
        state.config = load_fiscal_pos_config(conn);
        state.loaded = true;
    }

    Frame::new()
        .fill(crate::theme::BG_CARD)
        .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::same(14))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("🏛 ΦΟΡΟΛΟΓΙΚΟΣ ΜΗΧΑΝΙΣΜΟΣ (ΦΗΜ) & EFT-POS TCP/IP (Α.1155/2023)")
                        .strong()
                        .color(crate::theme::ACCENT_CYAN),
                );
                if let Some(online) = state.ping_status {
                    let (p_txt, p_col) = if online { ("🟢 POS Online (LAN)", crate::theme::STATUS_READY) } else { ("🔴 POS Offline", crate::theme::STATUS_CANCELLED) };
                    ui.colored_label(p_col, p_txt);
                }
            });
            ui.add_space(4.0);

            if let Some((msg, is_ok)) = &state.test_feedback {
                let col = if *is_ok { Color32::from_rgb(52, 211, 153) } else { Color32::from_rgb(244, 63, 94) };
                ui.label(RichText::new(msg).color(col).strong().size(12.0));
                ui.add_space(4.0);
            }
            if let Some((msg, is_ok)) = &state.save_feedback {
                let col = if *is_ok { Color32::from_rgb(52, 211, 153) } else { Color32::from_rgb(244, 63, 94) };
                ui.label(RichText::new(msg).color(col).strong().size(12.0));
                ui.add_space(4.0);
            }

            ui.columns(2, |cols| {
                // Column 1: Fiscal Mechanism (ΦΗΜ)
                cols[0].vertical(|ui| {
                    ui.label(RichText::new("Τύπος Φορολογικού Μηχανισμού (ΦΗΜ):").strong());
                    egui::ComboBox::from_id_salt("fiscal_device_combo")
                        .selected_text(match state.config.device_type {
                            FiscalDeviceType::CashRegisterTypeA => "ΦΗΜ Τύπου Α (Ταμειακή RBS / Casio / Diga)",
                            FiscalDeviceType::SignatureMechanismTypeB => "ΦΗΜ Τύπου Β (ΕΑΦΔΣΣ / TaxSpooler)",
                            FiscalDeviceType::ModernTaxBoxTypeC => "ΦΗΜAS Τύπου Γ (Open TaxBox / REST)",
                            FiscalDeviceType::DirectCloudAade => "Direct Cloud myDATA (Χωρίς Φυσικό ΦΗΜ)",
                        })
                        .width(f32::INFINITY)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut state.config.device_type, FiscalDeviceType::CashRegisterTypeA, "ΦΗΜ Τύπου Α (Ταμειακή RBS / Casio / Diga)");
                            ui.selectable_value(&mut state.config.device_type, FiscalDeviceType::SignatureMechanismTypeB, "ΦΗΜ Τύπου Β (ΕΑΦΔΣΣ / TaxSpooler)");
                            ui.selectable_value(&mut state.config.device_type, FiscalDeviceType::ModernTaxBoxTypeC, "ΦΗΜAS Τύπου Γ (Open TaxBox / REST)");
                            ui.selectable_value(&mut state.config.device_type, FiscalDeviceType::DirectCloudAade, "Direct Cloud myDATA (Χωρίς Φυσικό ΦΗΜ)");
                        });

                    ui.add_space(6.0);
                    ui.checkbox(&mut state.config.eft_pos_interlock_enabled, "Υποχρεωτικό Interlock A.1155 (Κλείδωμα Ταμείου)");
                    ui.label(RichText::new("Μπλοκάρει την έκδοση απόδειξης χωρίς τραπεζική έγκριση.").size(11.0).color(crate::theme::TEXT_MUTED));

                    ui.add_space(8.0);
                    ui.label(RichText::new("A.1155 Κλειδί Ακεραιότητας (HMAC Secret):").size(11.0).color(crate::theme::TEXT_MUTED));
                    ui.add(egui::TextEdit::singleline(&mut state.config.a1155_secret_key).password(true).desired_width(f32::INFINITY));
                });

                // Column 2: EFT-POS Terminal Configuration & Actions
                cols[1].vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label("IP Τερματικού POS:");
                            ui.add(egui::TextEdit::singleline(&mut state.config.eft_pos_host).hint_text("192.168.1.150").desired_width(140.0));
                        });
                        ui.vertical(|ui| {
                            ui.label("TCP Port:");
                            let mut port_str = state.config.eft_pos_port.to_string();
                            if ui.add(egui::TextEdit::singleline(&mut port_str).desired_width(60.0)).changed() {
                                if let Ok(p) = port_str.parse::<u16>() { state.config.eft_pos_port = p; }
                            }
                        });
                    });

                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label("Terminal ID (TID):");
                            ui.add(egui::TextEdit::singleline(&mut state.config.eft_pos_terminal_id).desired_width(110.0));
                        });
                        ui.vertical(|ui| {
                            ui.label("Merchant ID (MID):");
                            ui.add(egui::TextEdit::singleline(&mut state.config.eft_pos_merchant_id).desired_width(110.0));
                        });
                    });

                    ui.add_space(8.0);
                    ui.horizontal_wrapped(|ui| {
                        // Action 1: LAN Ping
                        if ui.button(RichText::new("📡 Ping").strong()).clicked() {
                            let driver = EftPosDriver::new(state.config.clone());
                            match driver.ping() {
                                Ok(ok) => {
                                    state.ping_status = Some(ok);
                                    if ok {
                                        proteus_core::audio::play_barcode_chime();
                                        state.test_feedback = Some((format!("✓ Τερματικό POS ενεργό στο {}:{}", state.config.eft_pos_host, state.config.eft_pos_port), true));
                                    } else {
                                        proteus_core::audio::play_error_tone();
                                        state.test_feedback = Some(("❌ Το τερματικό δεν αποκρίθηκε στο Ping".to_string(), false));
                                    }
                                }
                                Err(e) => {
                                    state.ping_status = Some(false);
                                    proteus_core::audio::play_error_tone();
                                    state.test_feedback = Some((format!("❌ Σφάλμα σύνδεσης: {}", e), false));
                                }
                            }
                        }

                        // Action 2: Test Card Transaction (€1.00)
                        if ui.button(RichText::new("💳 Δοκιμή Συναλλαγής (1.00€)").strong().color(crate::theme::ACCENT_PRIMARY)).clicked() {
                            let driver = EftPosDriver::new(state.config.clone());
                            let test_req = PosInitiateRequest {
                                transaction_id: format!("TX-{}", chrono::Utc::now().timestamp_millis()),
                                operation: PosTransactionType::Sale,
                                amount_cents: 100,
                                currency_code: 978,
                                receipt_number: 9001,
                                invoice_type: "11.1".to_string(),
                                cashier_id: "TEST-LNK".to_string(),
                            };
                            match driver.initiate_card_transaction(&test_req) {
                                Ok(resp) => {
                                    if resp.status == PosTransactionStatus::Approved {
                                        proteus_core::audio::play_barcode_chime();
                                        let _ = record_pos_card_transaction(conn, &test_req, &resp);
                                        state.test_feedback = Some((
                                            format!("✓ ΕΓΚΡΙΘΗΚΕ: TID: {}, RRN: {}, Υπογραφή: {}", resp.terminal_id, resp.rrn, &resp.interconnection_signature[..22]),
                                            true,
                                        ));
                                        state.last_response = Some(resp);
                                    } else {
                                        proteus_core::audio::play_error_tone();
                                        state.test_feedback = Some((format!("❌ Απόρριψη: {:?}", resp.status), false));
                                    }
                                }
                                Err(e) => {
                                    proteus_core::audio::play_error_tone();
                                    state.test_feedback = Some((format!("❌ Σφάλμα: {}", e), false));
                                }
                            }
                        }

                        // Action 3: Batch Close
                        if ui.button(RichText::new("🏁 Κλείσιμο Πακέτου").strong()).clicked() {
                            let driver = EftPosDriver::new(state.config.clone());
                            match driver.close_batch() {
                                Ok(b) => {
                                    proteus_core::audio::play_affirm_tone();
                                    state.test_feedback = Some((
                                        format!("✓ Πακέτο #{} έκλεισε: {} συναλλαγές, €{:.2}", b.batch_number, b.transaction_count, b.total_amount_cents as f64 / 100.0),
                                        true,
                                    ));
                                    state.last_batch = Some(b);
                                }
                                Err(e) => {
                                    state.test_feedback = Some((format!("❌ Σφάλμα κλεισίματος: {}", e), false));
                                }
                            }
                        }

                        // Action 4: Save Config
                        if ui.button(RichText::new("💾 Αποθήκευση").strong().color(crate::theme::STATUS_READY)).clicked() {
                            match save_fiscal_pos_config(conn, &state.config) {
                                Ok(_) => {
                                    proteus_core::audio::play_affirm_tone();
                                    state.save_feedback = Some(("✓ Αποθηκεύτηκε στη SQLite!".to_string(), true));
                                }
                                Err(e) => {
                                    state.save_feedback = Some((format!("Σφάλμα: {}", e), false));
                                }
                            }
                        }
                    });
                });
            });

            // Receipt Thermal Slip Preview
            if let Some(resp) = &state.last_response {
                ui.add_space(8.0);
                Frame::default()
                    .fill(crate::theme::BG_BASE)
                    .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
                    .corner_radius(CornerRadius::same(4))
                    .inner_margin(Margin::same(8))
                    .show(ui, |ui| {
                        ui.label(RichText::new("🧾 ESC/POS Thermal Slip (AADE A.1155/2023 Footer):").size(11.0).color(crate::theme::TEXT_MUTED));
                        let slip_bytes = format_a1155_receipt_slip(resp, 42);
                        let slip_str = String::from_utf8_lossy(&slip_bytes);
                        ui.label(RichText::new(slip_str).monospace().size(10.0).color(crate::theme::ACCENT_CYAN));
                    });
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fiscal_hardware_view_defaults() {
        let state = FiscalHardwareViewState::default();
        assert!(!state.loaded);
        assert!(state.config.eft_pos_interlock_enabled);
        assert_eq!(state.config.eft_pos_port, 8080);
    }
}
