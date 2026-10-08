//! Screen 1: New Service Intake Form (Νέα Παραλαβή).
//! Fast operational data entry with instant ESC/POS receipt generation.

use proteus_core::audio::{play_barcode_chime, play_error_tone};
use proteus_core::printer::{generate_intake_receipt, print_raw_bytes, ShopReceiptConfig};
use proteus_core::tickets::{create_ticket, ServiceTicket};
use egui::{Color32, CornerRadius, Frame, Margin, RichText, Stroke, Ui};
use rusqlite::Connection;

#[derive(Clone)]
pub struct IntakeFormState {
    pub customer_name: String,
    pub customer_phone: String,
    pub device_model: String,
    pub serial_number: String,
    pub reported_fault: String,
    pub estimated_cost_str: String,
    pub gdpr_consent: bool,
    pub status_message: Option<(String, bool)>, // (message, is_success)
}

impl Default for IntakeFormState {
    fn default() -> Self {
        Self {
            customer_name: String::new(),
            customer_phone: String::new(),
            device_model: String::new(),
            serial_number: String::new(),
            reported_fault: String::new(),
            estimated_cost_str: String::new(),
            gdpr_consent: true,
            status_message: None,
        }
    }
}

impl IntakeFormState {
    pub fn reset(&mut self) {
        self.customer_name.clear();
        self.customer_phone.clear();
        self.device_model.clear();
        self.serial_number.clear();
        self.reported_fault.clear();
        self.estimated_cost_str.clear();
        self.gdpr_consent = true;
        self.status_message = None;
    }
}

pub fn draw_intake_view(
    ui: &mut Ui,
    conn: &Connection,
    state: &mut IntakeFormState,
    printer_config: &ShopReceiptConfig,
    printer_name: &str,
    operator_name: &str,
    operator_role: &str,
) {
    ui.vertical(|ui| {
        // Header
        ui.horizontal(|ui| {
            ui.heading(RichText::new("Νέα Παραλαβή Συσκευής").strong().size(20.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Καθαρισμός").clicked() {
                    state.reset();
                }
            });
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(12.0);

        // Status Toast Message
        if let Some((msg, is_success)) = &state.status_message {
            let bg_col = if *is_success {
                Color32::from_rgb(16, 80, 50)
            } else {
                Color32::from_rgb(90, 20, 30)
            };
            let text_col = if *is_success {
                Color32::from_rgb(52, 211, 153)
            } else {
                Color32::from_rgb(244, 63, 94)
            };

            Frame::new()
                .fill(bg_col)
                .stroke(Stroke::new(1.0, text_col))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::symmetric(14, 10))
                .show(ui, |ui| {
                    ui.label(RichText::new(msg).color(text_col).strong());
                });
            ui.add_space(12.0);
        }

        // Form Two-Column Grid
        Frame::new()
            .fill(crate::theme::BG_CARD)
            .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(Margin::same(16))
            .show(ui, |ui| {
                ui.columns(2, |cols| {
                    // Column 1: Customer Details
                    cols[0].vertical(|ui| {
                        ui.label(RichText::new("ΣΤΟΙΧΕΙΑ ΠΕΛΑΤΗ").size(13.0).color(crate::theme::TEXT_MUTED).strong());
                        ui.add_space(6.0);

                        ui.label("Ονοματεπώνυμο *");
                        let name_resp = ui.add(egui::TextEdit::singleline(&mut state.customer_name).hint_text("π.χ. Νίκος Παπαδόπουλος").desired_width(f32::INFINITY));
                        if name_resp.has_focus() {
                            name_resp.scroll_to_me(Some(egui::Align::Center));
                        }
                        ui.add_space(8.0);

                        ui.label("Τηλέφωνο Επικοινωνίας *");
                        let phone_resp = ui.add(egui::TextEdit::singleline(&mut state.customer_phone).hint_text("π.χ. 6971234567").desired_width(f32::INFINITY));
                        if phone_resp.has_focus() {
                            phone_resp.scroll_to_me(Some(egui::Align::Center));
                        }
                        ui.add_space(8.0);

                        ui.label("Εκτίμηση Κόστους (€)");
                        let cost_resp = ui.add(egui::TextEdit::singleline(&mut state.estimated_cost_str).hint_text("π.χ. 45.00").desired_width(f32::INFINITY));
                        if cost_resp.has_focus() {
                            cost_resp.scroll_to_me(Some(egui::Align::Center));
                        }
                    });

                    // Column 2: Device & Fault Details
                    cols[1].vertical(|ui| {
                        ui.label(RichText::new("ΣΤΟΙΧΕΙΑ ΣΥΣΚΕΥΗΣ & ΒΛΑΒΗ").size(13.0).color(crate::theme::TEXT_MUTED).strong());
                        ui.add_space(6.0);

                        ui.label("Μοντέλο Συσκευής / Οχήματος *");
                        let model_resp = ui.add(egui::TextEdit::singleline(&mut state.device_model).hint_text("π.χ. Samsung Galaxy S22 ή Crypton X").desired_width(f32::INFINITY));
                        if model_resp.has_focus() {
                            model_resp.scroll_to_me(Some(egui::Align::Center));
                        }
                        ui.add_space(8.0);

                        ui.label("Σειριακός Αριθμός / IMEI / Πινακίδα");
                        let serial_resp = ui.add(egui::TextEdit::singleline(&mut state.serial_number).hint_text("Προαιρετικό").desired_width(f32::INFINITY));
                        if serial_resp.has_focus() {
                            serial_resp.scroll_to_me(Some(egui::Align::Center));
                        }
                        ui.add_space(8.0);

                        ui.label("Περιγραφή Βλάβης *");
                        let fault_resp = ui.add(egui::TextEdit::multiline(&mut state.reported_fault).hint_text("Αναφορά συμπτωμάτων από τον πελάτη...").desired_rows(3).desired_width(f32::INFINITY));
                        if fault_resp.has_focus() {
                            fault_resp.scroll_to_me(Some(egui::Align::Center));
                        }
                    });
                });
            });

        ui.add_space(8.0);
        let consent_col = if state.gdpr_consent { Color32::from_rgb(148, 163, 184) } else { Color32::from_rgb(244, 63, 94) };
        ui.checkbox(
            &mut state.gdpr_consent,
            RichText::new("✓ Συγκατάθεση GDPR (Ν. 4624/2019): Ο πελάτης συναινεί στην καταγραφή στοιχείων για την επισκευή.")
                .size(11.0)
                .color(consent_col),
        );
        ui.add_space(8.0);

        // Actions Row
        ui.horizontal(|ui| {
            // Save & Print Button
            let save_print_btn = ui.add_sized(
                [220.0, 42.0],
                egui::Button::new(RichText::new("🖨 Αποθήκευση & Εκτύπωση").strong().size(15.0).color(Color32::WHITE))
                    .fill(crate::theme::ACCENT_PRIMARY),
            );

            // Save Only Button
            let save_only_btn = ui.add_sized(
                [160.0, 42.0],
                egui::Button::new(RichText::new("💾 Αποθήκευση Μόνο").size(14.0))
                    .fill(crate::theme::BG_CARD),
            );

            let mut trigger_save = false;
            let mut trigger_print = false;

            if save_print_btn.clicked() {
                trigger_save = true;
                trigger_print = true;
            } else if save_only_btn.clicked() {
                trigger_save = true;
                trigger_print = false;
            }

            if trigger_save {
                if !state.gdpr_consent {
                    play_error_tone();
                    state.status_message = Some(("Απαιτείται επιβεβαίωση συγκατάθεσης GDPR του πελάτη.".to_string(), false));
                } else if state.customer_name.trim().is_empty()
                    || state.customer_phone.trim().is_empty()
                    || state.device_model.trim().is_empty()
                    || state.reported_fault.trim().is_empty()
                {
                    play_error_tone();
                    state.status_message = Some(("Παρακαλώ συμπληρώστε όλα τα απαραίτητα πεδία με (*).".to_string(), false));
                } else {
                    let cost: f64 = state.estimated_cost_str.trim().parse().unwrap_or(0.0);
                    let mut ticket = ServiceTicket::new(
                        state.customer_name.trim(),
                        state.customer_phone.trim(),
                        state.device_model.trim(),
                        state.reported_fault.trim(),
                    );
                    if !state.serial_number.trim().is_empty() {
                        ticket.serial_number = Some(state.serial_number.trim().to_string());
                    }
                    ticket.estimated_cost = cost;

                    match create_ticket(conn, &mut ticket) {
                        Ok(_) => {
                            play_barcode_chime();
                            let ticket_num = ticket.ticket_number;
                            let payload = serde_json::to_string(&ticket).unwrap_or_default();
                            let _ = proteus_core::replication::enqueue_outbox(
                                conn,
                                "tickets",
                                &ticket_num.to_string(),
                                proteus_core::replication::ChangeOp::Insert,
                                &payload,
                            );
                            let _ = proteus_core::audit::log_audit_event(
                                conn,
                                &proteus_core::audit::SystemEvent::new(
                                    "TICKET",
                                    &ticket.ticket_id,
                                    "INTAKE_CREATED",
                                    operator_name,
                                    operator_role,
                                    format!("Νέα παραλαβή συσκευής {} ({}) - #{}", ticket.device_model, ticket.customer_name, ticket_num),
                                    &payload,
                                ),
                            );

                            let mut msg = format!("✓ Το Δελτίο #{} καταχωρήθηκε επιτυχώς!", ticket_num);

                            if trigger_print {
                                let receipt_bytes = generate_intake_receipt(&ticket, printer_config);
                                match print_raw_bytes(printer_name, &format!("Ticket #{}", ticket_num), &receipt_bytes) {
                                    Ok(_) => {
                                        msg.push_str(" Η εκτύπωση στάλθηκε.");
                                    }
                                    Err(err) => {
                                        msg.push_str(&format!(" (Σφάλμα εκτύπωσης: {})", err));
                                    }
                                }
                            }

                            state.reset();
                            state.status_message = Some((msg, true));
                        }
                        Err(err) => {
                            play_error_tone();
                            state.status_message = Some((format!("Σφάλμα βάσης δεδομένων: {}", err), false));
                        }
                    }
                }
            }
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intake_form_reset() {
        let mut state = IntakeFormState {
            customer_name: "John Doe".to_string(),
            customer_phone: "123456789".to_string(),
            device_model: "Phone X".to_string(),
            serial_number: "SN123".to_string(),
            reported_fault: "Broken screen".to_string(),
            estimated_cost_str: "50.00".to_string(),
            gdpr_consent: false,
            status_message: Some(("Success".to_string(), true)),
        };

        state.reset();

        assert!(state.gdpr_consent);

        assert!(state.customer_name.is_empty());
        assert!(state.customer_phone.is_empty());
        assert!(state.device_model.is_empty());
        assert!(state.serial_number.is_empty());
        assert!(state.reported_fault.is_empty());
        assert!(state.estimated_cost_str.is_empty());
        assert!(state.status_message.is_none());
    }
}

