//! Audit Log & Activity Timeline View (Ιστορικό Κινήσεων).
//! Provides full transparency into every movement, state change, and operational action.

use crm_core::audit::{list_audit_events, SystemEvent};
use egui::{Color32, CornerRadius, Frame, Margin, RichText, ScrollArea, Stroke, Ui};
use rusqlite::Connection;

pub struct AuditLogViewState {
    pub search_query: String,
    pub selected_role_filter: Option<String>,
    pub selected_event_type: Option<String>,
    pub expanded_event_id: Option<String>,
}

impl Default for AuditLogViewState {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            selected_role_filter: None,
            selected_event_type: None,
            expanded_event_id: None,
        }
    }
}

pub fn draw_audit_log_view(
    ui: &mut Ui,
    conn: &Connection,
    state: &mut AuditLogViewState,
) {
    ui.vertical(|ui| {
        // Header
        ui.heading(RichText::new("📜 Ιστορικό Κινήσεων & Audit Trail (Activity Timeline)").strong().size(22.0));
        ui.add_space(4.0);
        ui.label(RichText::new("Πλήρης διαφάνεια ενεργειών: Πότε έγινε, τι έγινε, από ποιον χειριστή/ρόλο, και σε ποιο υποσύστημα.")
            .size(12.0)
            .color(crate::theme::TEXT_MUTED));
        ui.add_space(8.0);
        ui.separator();
        ui.add_space(10.0);

        // Filter Bar
        Frame::new()
            .fill(crate::theme::BG_CARD)
            .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(Margin::symmetric(14, 10))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("🔍 Αναζήτηση:");
                    ui.add(egui::TextEdit::singleline(&mut state.search_query).hint_text("Χειριστής, περιγραφή, ID..."));

                    ui.add_space(12.0);
                    ui.label("Ρόλος:");
                    let current_role_text = state.selected_role_filter.as_deref().unwrap_or("Όλοι οι Ρόλοι");
                    egui::ComboBox::from_id_salt("role_filter_combo")
                        .selected_text(current_role_text)
                        .show_ui(ui, |ui| {
                            if ui.selectable_label(state.selected_role_filter.is_none(), "Όλοι οι Ρόλοι").clicked() {
                                state.selected_role_filter = None;
                            }
                            for r in &["Ceo", "CustomerService", "Technician", "SalesConsultant", "Developer", "BusinessAnalyst"] {
                                let is_sel = state.selected_role_filter.as_deref() == Some(*r);
                                if ui.selectable_label(is_sel, *r).clicked() {
                                    state.selected_role_filter = Some(r.to_string());
                                }
                            }
                        });

                    ui.add_space(12.0);
                    if ui.button("⟲ Καθαρισμός Φίλτρων").clicked() {
                        state.search_query.clear();
                        state.selected_role_filter = None;
                        state.selected_event_type = None;
                    }
                });
            });

        ui.add_space(12.0);

        // Query events from SQLite
        let raw_events = list_audit_events(conn, 100, 0, state.selected_role_filter.as_deref()).unwrap_or_default();

        let query = state.search_query.trim().to_lowercase();
        let filtered_events: Vec<&SystemEvent> = raw_events
            .iter()
            .filter(|e| {
                if query.is_empty() {
                    return true;
                }
                e.description.to_lowercase().contains(&query)
                    || e.operator_name.to_lowercase().contains(&query)
                    || e.entity_id.to_lowercase().contains(&query)
                    || e.event_type.to_lowercase().contains(&query)
            })
            .collect();

        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("Σύνολο Κινήσεων: {}", filtered_events.len())).size(12.0).color(crate::theme::TEXT_MUTED));
        });
        ui.add_space(6.0);

        if filtered_events.is_empty() {
            Frame::new()
                .fill(crate::theme::BG_CARD)
                .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
                .corner_radius(CornerRadius::same(8))
                .inner_margin(Margin::same(24))
                .show(ui, |ui| {
                    ui.label(RichText::new("Δεν βρέθηκαν καταγεγραμμένες κινήσεις για τα επιλεγμένα κριτήρια.").color(crate::theme::TEXT_MUTED));
                });
            return;
        }

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                for evt in filtered_events {
                    draw_timeline_event_card(ui, evt, state);
                    ui.add_space(6.0);
                }
            });
    });
}

fn draw_timeline_event_card(ui: &mut Ui, evt: &SystemEvent, state: &mut AuditLogViewState) {
    let role_color = match evt.operator_role.as_str() {
        "Ceo" => Color32::from_rgb(251, 191, 36), // Gold
        "CustomerService" => Color32::from_rgb(56, 189, 248), // Sky Blue
        "Technician" => Color32::from_rgb(52, 211, 153), // Emerald Green
        "SalesConsultant" => Color32::from_rgb(249, 115, 22), // Orange
        "Developer" => Color32::from_rgb(168, 85, 247), // Purple
        "BusinessAnalyst" => Color32::from_rgb(14, 165, 233), // Electric Cyan
        _ => Color32::from_rgb(156, 163, 175),
    };

    let time_str = format_relative_time(evt.created_at);

    Frame::new()
        .fill(crate::theme::BG_CARD)
        .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::symmetric(14, 10))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Role pill
                Frame::new()
                    .fill(crate::theme::BG_BASE)
                    .stroke(Stroke::new(1.0, role_color))
                    .corner_radius(CornerRadius::same(4))
                    .inner_margin(Margin::symmetric(6, 2))
                    .show(ui, |ui| {
                        ui.label(RichText::new(&evt.operator_role).size(11.0).color(role_color).strong());
                    });

                ui.add_space(6.0);
                ui.label(RichText::new(&evt.operator_name).strong().color(crate::theme::TEXT_PRIMARY));

                ui.add_space(6.0);
                // Action tag
                ui.label(RichText::new(format!("[{}]", evt.event_type)).size(11.0).color(crate::theme::ACCENT_CYAN));

                ui.add_space(6.0);
                ui.label(RichText::new(format!("({})", evt.entity_id)).size(11.0).color(crate::theme::TEXT_MUTED));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(time_str).size(11.0).color(crate::theme::TEXT_MUTED));
                });
            });

            ui.add_space(4.0);
            ui.label(RichText::new(&evt.description).size(13.0).color(crate::theme::TEXT_SECONDARY));

            // Optional Payload Expand
            let is_expanded = state.expanded_event_id.as_deref() == Some(&evt.event_id);
            if is_expanded {
                ui.add_space(6.0);
                Frame::new()
                    .fill(crate::theme::BG_BASE)
                    .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
                    .corner_radius(CornerRadius::same(4))
                    .inner_margin(Margin::same(8))
                    .show(ui, |ui| {
                        ui.label(RichText::new(&evt.payload_json).monospace().size(11.0).color(crate::theme::TEXT_MUTED));
                    });
            }

            ui.horizontal(|ui| {
                let expand_btn_text = if is_expanded { "▴ Απόκρυψη λεπτομερειών" } else { "▾ Προβολή JSON Payload" };
                if ui.small_button(expand_btn_text).clicked() {
                    if is_expanded {
                        state.expanded_event_id = None;
                    } else {
                        state.expanded_event_id = Some(evt.event_id.clone());
                    }
                }
            });
        });
}

fn format_relative_time(timestamp_ms: i64) -> String {
    let now = chrono::Utc::now().timestamp_millis();
    let diff_sec = (now - timestamp_ms) / 1000;

    if diff_sec < 60 {
        "μόλις τώρα".to_string()
    } else if diff_sec < 3600 {
        format!("πριν {} λεπτά", diff_sec / 60)
    } else if diff_sec < 86400 {
        format!("πριν {} ώρες", diff_sec / 3600)
    } else {
        format!("πριν {} μέρες", diff_sec / 86400)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relative_time_formatting() {
        let now = chrono::Utc::now().timestamp_millis();
        assert_eq!(format_relative_time(now - 10_000), "μόλις τώρα");
        assert_eq!(format_relative_time(now - 180_000), "πριν 3 λεπτά");
        assert_eq!(format_relative_time(now - 7200_000), "πριν 2 ώρες");
    }
}
