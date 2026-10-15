//! Mobile Companion Tickets Tab View.
//! Renders searchable service ticket list or interactive on-site field service workbench.

use crate::{MobileAppState, ACCENT_GOLD, BG_CARD, BORDER_LINE, MIN_TOUCH_TARGET, TEXT_BODY, TEXT_TITLE};
use egui::{CornerRadius, Frame, Margin, RichText, Stroke, Ui, Vec2};
use proteus_core::tickets::{list_tickets, ServiceTicket};

pub fn render_tickets_tab(ui: &mut Ui, state: &mut MobileAppState) {
    if state.field_service_state.active_ticket.is_some() {
        crate::field_service::render_field_service_view(ui, &mut state.field_service_state, &state.conn);
        return;
    }

    ui.horizontal(|ui| {
        ui.add_sized(
            Vec2::new(ui.available_width(), 38.0),
            egui::TextEdit::singleline(&mut state.search_query).hint_text("🔍 Αναζήτηση πελάτη ή συσκευής..."),
        );
    });
    ui.add_space(8.0);

    let tickets = list_tickets(&state.conn).unwrap_or_default();
    let q = state.search_query.to_lowercase();
    let filtered: Vec<&ServiceTicket> = tickets
        .iter()
        .filter(|t| {
            q.is_empty()
                || t.customer_name.to_lowercase().contains(&q)
                || t.device_model.to_lowercase().contains(&q)
                || t.ticket_id.to_lowercase().contains(&q)
        })
        .collect();

    if filtered.is_empty() {
        ui.label(RichText::new("Δεν βρέθηκαν ενεργές εντολές στη βάση SQLite.").color(TEXT_BODY));
        return;
    }

    let mut selected_to_open: Option<ServiceTicket> = None;

    for t in filtered {
        Frame::new()
            .fill(BG_CARD)
            .stroke(Stroke::new(1.0, BORDER_LINE))
            .corner_radius(CornerRadius::same(6))
            .inner_margin(Margin::same(10))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&t.customer_name).strong().color(TEXT_TITLE));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(t.current_status.display_name()).size(11.0).color(ACCENT_GOLD));
                    });
                });
                ui.label(RichText::new(format!("{} | {}", t.device_model, t.reported_fault)).size(11.5).color(TEXT_BODY));
                ui.label(RichText::new(format!("#{}: {}", t.ticket_number, t.ticket_id)).size(9.5).color(BORDER_LINE));
                ui.add_space(4.0);
                if ui.add_sized(
                    Vec2::new(ui.available_width(), MIN_TOUCH_TARGET - 6.0),
                    egui::Button::new(RichText::new("🔧 Επιτόπιο Service Bench").size(11.0).strong().color(egui::Color32::WHITE))
                        .fill(egui::Color32::from_rgb(37, 99, 235))
                        .corner_radius(CornerRadius::same(4)),
                ).clicked() {
                    selected_to_open = Some((*t).clone());
                }
            });
        ui.add_space(6.0);
    }

    if let Some(t) = selected_to_open {
        state.field_service_state.open_ticket(&state.conn, t);
    }
}
