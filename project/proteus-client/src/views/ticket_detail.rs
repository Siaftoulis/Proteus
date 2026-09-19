//! Screen 3: Service Ticket Detail Modal / Inspector.
//! Detailed technician notes, cost editing, status switching, and thermal reprint.

use crm_core::printer::{generate_intake_receipt, print_raw_bytes, ShopReceiptConfig};
use crm_core::tickets::{get_ticket, update_ticket_details, update_ticket_status, ServiceTicket, TicketStatus};
use egui::{Color32, CornerRadius, Frame, Margin, RichText, Stroke, Ui};
use rusqlite::Connection;

#[derive(Default)]
pub struct TicketDetailState {
    pub loaded_id: Option<String>,
    pub ticket: Option<ServiceTicket>,
    pub notes_edit: String,
    pub cost_edit: String,
    pub feedback_msg: Option<String>,
}

pub fn draw_ticket_detail_modal(
    ctx: &egui::Context,
    conn: &Connection,
    selected_id: &mut Option<String>,
    state: &mut TicketDetailState,
    printer_config: &ShopReceiptConfig,
    printer_name: &str,
) {
    if let Some(target_id) = selected_id.clone() {
        // Load ticket if not already loaded
        if state.loaded_id.as_deref() != Some(&target_id) {
            if let Ok(Some(t)) = get_ticket(conn, &target_id) {
                state.notes_edit = t.internal_notes.clone();
                state.cost_edit = format!("{:.2}", t.estimated_cost);
                state.ticket = Some(t);
                state.loaded_id = Some(target_id);
                state.feedback_msg = None;
            } else {
                *selected_id = None;
                return;
            }
        }

        let mut is_open = true;
        let title = if let Some(t) = &state.ticket {
            format!("Δελτίο Επισκευής #{} — {}", t.ticket_number, t.device_model)
        } else {
            "Καρτέλα Επισκευής".to_string()
        };

        egui::Window::new(RichText::new(title).strong().size(18.0))
            .open(&mut is_open)
            .collapsible(false)
            .resizable(true)
            .default_size([580.0, 520.0])
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                draw_modal_content(ui, conn, state, printer_config, printer_name);
            });

        if !is_open {
            *selected_id = None;
            state.loaded_id = None;
        }
    }
}

fn draw_modal_content(
    ui: &mut Ui,
    conn: &Connection,
    state: &mut TicketDetailState,
    printer_config: &ShopReceiptConfig,
    printer_name: &str,
) {
    let ticket = match state.ticket.as_ref() {
        Some(t) => t.clone(),
        None => return,
    };

    ui.vertical(|ui| {
        // Top status pill & date
        ui.horizontal(|ui| {
            let col = crate::theme::status_color(ticket.current_status);
            Frame::new()
                .fill(col.linear_multiply(0.2))
                .stroke(Stroke::new(1.0, col))
                .corner_radius(CornerRadius::same(4))
                .inner_margin(Margin::symmetric(10, 4))
                .show(ui, |ui| {
                    ui.label(RichText::new(ticket.current_status.display_name()).color(col).strong());
                });

            let dt = chrono::DateTime::from_timestamp_millis(ticket.created_at)
                .map(|d| d.format("%d/%m/%Y %H:%M").to_string())
                .unwrap_or_else(|| "N/A".to_string());
            ui.label(RichText::new(format!("Εισαγωγή: {}", dt)).size(12.0).color(crate::theme::TEXT_MUTED));
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        // Grid of customer & device info
        egui::Grid::new("ticket_info_grid")
            .num_columns(2)
            .spacing([20.0, 8.0])
            .show(ui, |ui| {
                ui.label(RichText::new("Πελάτης:").color(crate::theme::TEXT_MUTED));
                ui.label(RichText::new(&ticket.customer_name).strong());
                ui.end_row();

                ui.label(RichText::new("Τηλέφωνο:").color(crate::theme::TEXT_MUTED));
                ui.label(RichText::new(&ticket.customer_phone).strong());
                ui.end_row();

                ui.label(RichText::new("Συσκευή:").color(crate::theme::TEXT_MUTED));
                ui.label(RichText::new(&ticket.device_model).strong());
                ui.end_row();

                if let Some(sn) = &ticket.serial_number {
                    if !sn.is_empty() {
                        ui.label(RichText::new("S/N:").color(crate::theme::TEXT_MUTED));
                        ui.label(RichText::new(sn).strong());
                        ui.end_row();
                    }
                }

                ui.label(RichText::new("Βλάβη:").color(crate::theme::TEXT_MUTED));
                ui.label(RichText::new(&ticket.reported_fault).italics());
                ui.end_row();
            });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        // Status Changer Buttons Row
        ui.label(RichText::new("ΑΛΛΑΓΗ ΣΤΑΔΙΟΥ ΕΠΙΣΚΕΥΗΣ").size(12.0).color(crate::theme::TEXT_MUTED).strong());
        ui.horizontal_wrapped(|ui| {
            for &s in TicketStatus::all() {
                let is_active = s == ticket.current_status;
                let btn = if is_active {
                    egui::Button::new(RichText::new(format!("✓ {}", s.display_name())).strong())
                        .fill(crate::theme::status_color(s))
                } else {
                    egui::Button::new(s.display_name())
                        .fill(crate::theme::BG_CARD)
                };

                if ui.add(btn).clicked() && !is_active {
                    let _ = update_ticket_status(conn, &ticket.ticket_id, s);
                    if let Ok(Some(refreshed)) = get_ticket(conn, &ticket.ticket_id) {
                        state.ticket = Some(refreshed);
                        state.feedback_msg = Some("✓ Το στάδιο ενημερώθηκε.".to_string());
                    }
                }
            }
        });

        ui.add_space(10.0);

        // Editable Notes & Cost
        ui.columns(2, |cols| {
            cols[0].vertical(|ui| {
                ui.label(RichText::new("Τεχνικές Σημειώσεις Εργαστηρίου").strong());
                ui.add(egui::TextEdit::multiline(&mut state.notes_edit).desired_rows(4).desired_width(f32::INFINITY));
            });

            cols[1].vertical(|ui| {
                ui.label(RichText::new("Κόστος Επισκευής (€)").strong());
                ui.add(egui::TextEdit::singleline(&mut state.cost_edit).hint_text("0.00").desired_width(120.0));

                ui.add_space(12.0);

                if ui.button(RichText::new("💾 Αποθήκευση Σημειώσεων").strong()).clicked() {
                    let cost: f64 = state.cost_edit.trim().parse().unwrap_or(ticket.estimated_cost);
                    match update_ticket_details(conn, &ticket.ticket_id, state.notes_edit.trim(), cost) {
                        Ok(_) => {
                            if let Ok(Some(refreshed)) = get_ticket(conn, &ticket.ticket_id) {
                                state.ticket = Some(refreshed);
                                state.feedback_msg = Some("✓ Οι σημειώσεις αποθηκεύτηκαν.".to_string());
                            }
                        }
                        Err(e) => {
                            state.feedback_msg = Some(format!("Σφάλμα: {}", e));
                        }
                    }
                }
            });
        });

        ui.add_space(12.0);

        // Feedback message
        if let Some(msg) = &state.feedback_msg {
            ui.label(RichText::new(msg).color(Color32::from_rgb(52, 211, 153)).strong());
            ui.add_space(6.0);
        }

        ui.separator();
        ui.add_space(8.0);

        // Actions: Reprint
        ui.horizontal(|ui| {
            if ui.button(RichText::new("🖨 Επανεκτύπωση Δελτίου").size(14.0)).clicked() {
                let receipt_bytes = generate_intake_receipt(&ticket, printer_config);
                match print_raw_bytes(printer_name, &format!("Ticket #{}", ticket.ticket_number), &receipt_bytes) {
                    Ok(_) => {
                        state.feedback_msg = Some("✓ Η επανεκτύπωση στάλθηκε στον εκτυπωτή.".to_string());
                    }
                    Err(e) => {
                        state.feedback_msg = Some(format!("Σφάλμα εκτυπωτή: {}", e));
                    }
                }
            }
        });
    });
}
