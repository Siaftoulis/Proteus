//! Screen 3: Service Ticket Detail Modal / Inspector.
//! Detailed technician notes, cost editing, status switching, and thermal reprint.

use proteus_core::genealogy::{get_genealogy, get_genealogy_timeline, GenealogyEvent, SerialGenealogy};
use proteus_core::printer::{generate_intake_receipt, print_raw_bytes, ShopReceiptConfig};
use proteus_core::tickets::{get_ticket, update_ticket_details, update_ticket_status, ServiceTicket, TicketStatus};
use egui::{Color32, CornerRadius, Frame, Margin, RichText, Stroke, Ui};
use rusqlite::Connection;

#[derive(Default)]
pub struct TicketDetailState {
    pub loaded_id: Option<String>,
    pub ticket: Option<ServiceTicket>,
    pub notes_edit: String,
    pub cost_edit: String,
    pub feedback_msg: Option<String>,
    pub show_genealogy: bool,
    pub genealogy_record: Option<SerialGenealogy>,
    pub genealogy_timeline: Vec<GenealogyEvent>,
    pub rma_fault_input: String,
    pub rma_replacement_sn: String,
    pub intake_supplier: String,
    pub intake_warranty_months: u32,
    pub show_bench_panel: bool,
    pub bench_state: crate::views::ticket_bench::TicketBenchState,
}

#[allow(clippy::too_many_arguments)]
pub fn draw_ticket_detail_modal(
    ctx: &egui::Context,
    conn: &Connection,
    selected_id: &mut Option<String>,
    state: &mut TicketDetailState,
    printer_config: &ShopReceiptConfig,
    printer_name: &str,
    operator_name: &str,
    operator_role: &str,
) {
    if let Some(target_id) = selected_id.clone() {
        // Load ticket if not already loaded
        if state.loaded_id.as_deref() != Some(&target_id) {
            if let Ok(Some(t)) = get_ticket(conn, &target_id) {
                state.notes_edit = t.internal_notes.clone();
                state.cost_edit = format!("{:.2}", t.estimated_cost);
                if let Some(sn) = &t.serial_number {
                    state.genealogy_record = get_genealogy(conn, sn).unwrap_or(None);
                    state.genealogy_timeline = get_genealogy_timeline(conn, sn).unwrap_or_default();
                    state.rma_fault_input = t.reported_fault.clone();
                } else {
                    state.genealogy_record = None;
                    state.genealogy_timeline.clear();
                    state.rma_fault_input.clear();
                }
                state.show_genealogy = false;
                state.show_bench_panel = false;
                state.bench_state = crate::views::ticket_bench::TicketBenchState::default();
                state.rma_replacement_sn.clear();
                state.intake_supplier = "Επίσημη Αντιπροσωπεία".to_string();
                state.intake_warranty_months = 24;
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
                draw_modal_content(ui, conn, state, printer_config, printer_name, operator_name, operator_role);
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
    operator_name: &str,
    operator_role: &str,
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
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(sn).strong().color(crate::theme::ACCENT_CYAN));
                            let gene_btn_txt = if state.show_genealogy { "▲ Απόκρυψη S/N" } else { "🔍 Ιστορικό S/N & RMA" };
                            if ui.small_button(gene_btn_txt).clicked() {
                                state.show_genealogy = !state.show_genealogy;
                                if state.show_genealogy {
                                    state.genealogy_record = get_genealogy(conn, sn).unwrap_or(None);
                                    state.genealogy_timeline = get_genealogy_timeline(conn, sn).unwrap_or_default();
                                }
                            }
                        });
                        ui.end_row();
                    }
                }

                ui.label(RichText::new("Βλάβη:").color(crate::theme::TEXT_MUTED));
                ui.label(RichText::new(&ticket.reported_fault).italics());
                ui.end_row();
            });

        if state.show_genealogy {
            if let Some(sn) = &ticket.serial_number {
                ui.add_space(8.0);
                crate::views::embedded_genealogy::draw_embedded_genealogy(
                    ui,
                    conn,
                    &mut state.genealogy_record,
                    &mut state.genealogy_timeline,
                    &mut state.rma_fault_input,
                    &mut state.rma_replacement_sn,
                    &mut state.intake_supplier,
                    &mut state.intake_warranty_months,
                    &mut state.feedback_msg,
                    &ticket,
                    sn,
                    operator_name,
                    operator_role,
                );
            }
        }
        ui.add_space(8.0);

        let bench_btn_txt = if state.show_bench_panel {
            "▲ Απόκρυψη Πάγκου Τεχνικού"
        } else {
            "🛠️ Πάγκος Τεχνικού (Υλικά, Εργασία & myDATA)"
        };
        if ui.button(RichText::new(bench_btn_txt).strong().color(crate::theme::ACCENT_PRIMARY)).clicked() {
            state.show_bench_panel = !state.show_bench_panel;
        }

        if state.show_bench_panel {
            ui.add_space(8.0);
            crate::views::ticket_bench::draw_ticket_bench_panel(
                ui,
                conn,
                &mut state.bench_state,
                &ticket,
                operator_name,
            );
        }

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
                    let _ = proteus_core::audit::log_audit_event(
                        conn,
                        &proteus_core::audit::SystemEvent::new(
                            "TICKET",
                            &ticket.ticket_id,
                            "STATUS_CHANGED",
                            operator_name,
                            operator_role,
                            format!("Αλλαγή σταδίου σε {} (#{})", s.display_name(), ticket.ticket_number),
                            serde_json::json!({
                                "from_status": ticket.current_status.display_name(),
                                "to_status": s.display_name(),
                                "ticket_number": ticket.ticket_number,
                            }).to_string(),
                        ),
                    );
                    let mut updated_ticket = ticket.clone();
                    updated_ticket.current_status = s;
                    let _ = proteus_core::replication::enqueue_outbox(
                        conn,
                        "tickets",
                        &ticket.ticket_number.to_string(),
                        proteus_core::replication::ChangeOp::Update,
                        &serde_json::to_string(&updated_ticket).unwrap_or_default(),
                    );
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
                            let _ = proteus_core::audit::log_audit_event(
                                conn,
                                &proteus_core::audit::SystemEvent::new(
                                    "TICKET",
                                    &ticket.ticket_id,
                                    "DETAILS_SAVED",
                                    operator_name,
                                    operator_role,
                                    format!("Ενημέρωση σημειώσεων/κόστους (#{})", ticket.ticket_number),
                                    serde_json::json!({
                                        "ticket_number": ticket.ticket_number,
                                        "cost": cost,
                                        "notes": state.notes_edit.trim(),
                                    }).to_string(),
                                ),
                            );
                            let mut updated = ticket.clone();
                            updated.internal_notes = state.notes_edit.trim().to_string();
                            updated.estimated_cost = cost;
                            let _ = proteus_core::replication::enqueue_outbox(
                                conn,
                                "tickets",
                                &ticket.ticket_number.to_string(),
                                proteus_core::replication::ChangeOp::Update,
                                &serde_json::to_string(&updated).unwrap_or_default(),
                            );
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

