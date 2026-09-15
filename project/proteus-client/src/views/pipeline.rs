//! Screen 2: 6-Stage Kanban Pipeline (Ροή Επισκευών).
//! Visual stage tracking, instant search, and drag/click stage advancement.

use crm_core::tickets::{search_tickets, update_ticket_status, ServiceTicket, TicketStatus};
use egui::{Color32, CornerRadius, Frame, Margin, RichText, Stroke, Ui, Vec2};
use rusqlite::Connection;

pub fn draw_pipeline_view(
    ui: &mut Ui,
    conn: &Connection,
    search_query: &mut String,
    selected_ticket_id: &mut Option<String>,
) {
    ui.vertical(|ui| {
        // Header & Search Bar
        ui.horizontal(|ui| {
            ui.heading(RichText::new("📋 Ροή Επισκευών (Kanban)").strong().size(22.0));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add(
                    egui::TextEdit::singleline(search_query)
                        .hint_text("🔍 Αναζήτηση με Όνομα, Τηλέφωνο, Συσκευή, #...")
                        .desired_width(320.0),
                );
            });
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        // Fetch filtered tickets
        let tickets = search_tickets(conn, search_query).unwrap_or_default();

        // 6-Lane Columns
        let stages = TicketStatus::all();

        ui.horizontal(|ui| {
            egui::ScrollArea::horizontal()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.horizontal_top(|ui| {
                        for &stage in stages {
                            let stage_tickets: Vec<&ServiceTicket> = tickets
                                .iter()
                                .filter(|t| t.current_status == stage)
                                .collect();

                            draw_kanban_column(
                                ui,
                                conn,
                                stage,
                                &stage_tickets,
                                selected_ticket_id,
                            );
                            ui.add_space(8.0);
                        }
                    });
                });
        });
    });
}

fn draw_kanban_column(
    ui: &mut Ui,
    conn: &Connection,
    stage: TicketStatus,
    tickets: &[&ServiceTicket],
    selected_ticket_id: &mut Option<String>,
) {
    let col_width = 240.0;
    let col_color = crate::theme::status_color(stage);

    ui.vertical(|ui| {
        ui.set_width(col_width);

        // Column Header Box
        Frame::new()
            .fill(crate::theme::BG_CARD)
            .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
            .corner_radius(CornerRadius::same(6))
            .inner_margin(Margin::symmetric(10, 8))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Small status color bullet
                    let (rect, _) = ui.allocate_exact_size(Vec2::splat(10.0), egui::Sense::hover());
                    ui.painter().circle_filled(rect.center(), 5.0, col_color);

                    ui.label(RichText::new(stage.display_name()).strong().size(13.0));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Badge count
                        let badge_text = format!("{}", tickets.len());
                        Frame::new()
                            .fill(crate::theme::BG_BASE)
                            .corner_radius(CornerRadius::same(4))
                            .inner_margin(Margin::symmetric(6, 2))
                            .show(ui, |ui| {
                                ui.label(RichText::new(badge_text).size(11.0).strong().color(crate::theme::TEXT_SECONDARY));
                            });
                    });
                });
            });

        ui.add_space(6.0);

        // Column Cards Scroll Area
        egui::ScrollArea::vertical()
            .id_salt(format!("col_{}", stage.as_str()))
            .max_height(f32::INFINITY)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    for ticket in tickets {
                        draw_ticket_card(ui, conn, ticket, selected_ticket_id);
                        ui.add_space(6.0);
                    }
                });
            });
    });
}

fn draw_ticket_card(
    ui: &mut Ui,
    conn: &Connection,
    ticket: &ServiceTicket,
    selected_ticket_id: &mut Option<String>,
) {
    let card_response = Frame::new()
        .fill(crate::theme::BG_CARD)
        .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::same(10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());

            // Header: #Ticket Number & Price
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("#{}", ticket.ticket_number)).strong().size(13.0).color(crate::theme::ACCENT_CYAN));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ticket.estimated_cost > 0.0 {
                        ui.label(RichText::new(format!("{:.0}€", ticket.estimated_cost)).strong().size(12.0).color(Color32::from_rgb(52, 211, 153)));
                    }
                });
            });

            ui.add_space(2.0);

            // Customer Name
            ui.label(RichText::new(&ticket.customer_name).strong().size(13.0).color(crate::theme::TEXT_PRIMARY));

            // Device Model & Phone
            ui.label(RichText::new(&ticket.device_model).size(12.0).color(crate::theme::TEXT_SECONDARY));
            ui.label(RichText::new(&ticket.customer_phone).size(11.0).color(crate::theme::TEXT_MUTED));

            // Truncated fault
            if !ticket.reported_fault.is_empty() {
                ui.add_space(2.0);
                let fault_summary = if ticket.reported_fault.len() > 38 {
                    format!("{}...", &ticket.reported_fault[..35])
                } else {
                    ticket.reported_fault.clone()
                };
                ui.label(RichText::new(fault_summary).italics().size(11.0).color(crate::theme::TEXT_MUTED));
            }

            ui.add_space(4.0);

            // Quick Stage-Advance Bar
            ui.horizontal(|ui| {
                if let Some(next_stage) = get_next_stage(ticket.current_status) {
                    if ui.small_button(format!("→ {}", next_stage.display_name())).clicked() {
                        let _ = update_ticket_status(conn, &ticket.ticket_id, next_stage);
                    }
                }
            });
        });

    if card_response.response.interact(egui::Sense::click()).clicked() {
        *selected_ticket_id = Some(ticket.ticket_id.clone());
    }
}

pub(crate) fn get_next_stage(current: TicketStatus) -> Option<TicketStatus> {
    match current {
        TicketStatus::Received => Some(TicketStatus::InProgress),
        TicketStatus::InProgress => Some(TicketStatus::Ready),
        TicketStatus::WaitingParts => Some(TicketStatus::InProgress),
        TicketStatus::Ready => Some(TicketStatus::Delivered),
        TicketStatus::Delivered => None,
        TicketStatus::Cancelled => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stage_progression() {
        assert_eq!(get_next_stage(TicketStatus::Received), Some(TicketStatus::InProgress));
        assert_eq!(get_next_stage(TicketStatus::InProgress), Some(TicketStatus::Ready));
        assert_eq!(get_next_stage(TicketStatus::WaitingParts), Some(TicketStatus::InProgress));
        assert_eq!(get_next_stage(TicketStatus::Ready), Some(TicketStatus::Delivered));
        assert_eq!(get_next_stage(TicketStatus::Delivered), None);
        assert_eq!(get_next_stage(TicketStatus::Cancelled), None);
    }
}

