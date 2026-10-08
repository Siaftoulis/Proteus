//! Technician Workshop Bench View: Visual Diagnostics, Quality Gate & Orchestration.
//! Strict Rule 1 (100% Original Codebase), Rule 2 (Minimalist UX), Rule 3 (<400 lines), Rule 5 (Zero Mock Data).

use egui::{CornerRadius, Frame, Margin, RichText, Stroke, Ui};
use rusqlite::Connection;

use proteus_core::service_bench::{
    compute_bench_summary, get_diagnostic_checklist, init_service_bench_schema,
    list_consumed_parts, list_labor_logs, save_diagnostic_checklist, BenchSummary,
    ConsumedSparePart, DeviceDiagnosticChecklist, LaborLog,
};
use proteus_core::tickets::ServiceTicket;

use crate::theme::{
    ACCENT_GOLD, ACCENT_PRIMARY, BG_BASE, STATUS_CANCELLED, STATUS_READY, TEXT_MUTED,
};
use super::ticket_bench_parts::{render_labor_section, render_parts_section, render_settlement_bar};

/// UI state for technician bench operations on a ticket.
pub struct TicketBenchState {
    pub diag_loaded: bool,
    pub powers_on: bool,
    pub display_functional: bool,
    pub touch_responsive: bool,
    pub liquid_ingress_detected: bool,
    pub audio_functional: bool,
    pub camera_functional: bool,
    pub has_battery: bool,
    pub battery_health_pct: u8,
    pub cosmetic_condition: String,
    pub customer_accessories: String,
    pub diag_saved_at: Option<i64>,
    pub inspector_name: String,

    // Spare parts consumption state
    pub part_sku: String,
    pub part_desc: String,
    pub part_qty: String,
    pub part_cost: String,
    pub part_retail: String,
    pub inventory_search: String,
    pub show_inventory_picker: bool,

    // Labor logging state
    pub labor_desc: String,
    pub labor_duration_mins: u32,
    pub labor_rate: String,

    // Settlement state
    pub settlement_series: String,
    pub settlement_is_b2b: bool,
    pub settlement_afm: String,

    // Cached ledger data
    pub cached_parts: Vec<ConsumedSparePart>,
    pub cached_labor: Vec<LaborLog>,
    pub cached_summary: BenchSummary,
    pub status_msg: Option<(String, bool)>,
}

impl Default for TicketBenchState {
    fn default() -> Self {
        Self {
            diag_loaded: false,
            powers_on: true,
            display_functional: true,
            touch_responsive: true,
            liquid_ingress_detected: false,
            audio_functional: true,
            camera_functional: true,
            has_battery: true,
            battery_health_pct: 100,
            cosmetic_condition: "Καλή".to_string(),
            customer_accessories: "Μόνο συσκευή".to_string(),
            diag_saved_at: None,
            inspector_name: String::new(),
            part_sku: String::new(),
            part_desc: String::new(),
            part_qty: "1".to_string(),
            part_cost: "0.00".to_string(),
            part_retail: "0.00".to_string(),
            inventory_search: String::new(),
            show_inventory_picker: false,
            labor_desc: "Τεχνική επισκευή & έλεγχος".to_string(),
            labor_duration_mins: 30,
            labor_rate: "40.00".to_string(),
            settlement_series: "ΤΠΥ".to_string(),
            settlement_is_b2b: false,
            settlement_afm: String::new(),
            cached_parts: Vec::new(),
            cached_labor: Vec::new(),
            cached_summary: BenchSummary::default(),
            status_msg: None,
        }
    }
}

/// Renders the technician workshop bench panel inside ticket detail.
pub fn draw_ticket_bench_panel(
    ui: &mut Ui,
    conn: &Connection,
    state: &mut TicketBenchState,
    ticket: &ServiceTicket,
    operator_name: &str,
) {
    let _ = init_service_bench_schema(conn);

    if !state.diag_loaded {
        load_bench_data(conn, state, &ticket.ticket_id);
    }

    Frame::default()
        .fill(BG_BASE)
        .stroke(Stroke::new(1.0, ACCENT_PRIMARY.linear_multiply(0.4)))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("🛠️ Πάγκος Τεχνικού & Ανάλωση Υλικών").strong().color(ACCENT_PRIMARY));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("🔄 Ανανέωση").clicked() {
                        reload_bench_records(conn, state, &ticket.ticket_id);
                    }
                });
            });

            if let Some((ref msg, is_err)) = state.status_msg {
                let col = if is_err { STATUS_CANCELLED } else { STATUS_READY };
                ui.colored_label(col, msg);
            }

            ui.add_space(6.0);

            // 1. Diagnostics Checklist & Quality Gate
            render_diagnostics_section(ui, conn, state, ticket, operator_name);

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // 2. Consumed Spare Parts Ledger
            render_parts_section(ui, conn, state, ticket, operator_name);

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // 3. Labor Logging Section
            render_labor_section(ui, conn, state, ticket, operator_name);

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // 4. 1-Click Fiscal Settlement Bar
            render_settlement_bar(ui, conn, state, ticket);
        });
}

fn render_diagnostics_section(
    ui: &mut Ui,
    conn: &Connection,
    state: &mut TicketBenchState,
    ticket: &ServiceTicket,
    operator_name: &str,
) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("1. Διαγνωστικός Έλεγχος Παραλαβής (ISO 9001 Quality Gate)").strong());
        if let Some(ts) = state.diag_saved_at {
            let dt = chrono::DateTime::from_timestamp_millis(ts)
                .map(|t| t.format("%d/%m/%Y %H:%M").to_string())
                .unwrap_or_else(|| "Καταγεγραμμένο".to_string());
            ui.colored_label(STATUS_READY, format!("✓ Ελεγμένο ({}, από {})", dt, state.inspector_name));
        } else {
            ui.colored_label(ACCENT_GOLD, "⚠️ Εκκρεμεί αρχική καταγραφή");
        }
    });

    // Hardware Inspection Toggles Grid
    ui.horizontal_wrapped(|ui| {
        let p_txt = if state.powers_on { "⚡ Ανάβει" } else { "❌ Δεν ανάβει" };
        ui.checkbox(&mut state.powers_on, p_txt);

        let d_txt = if state.display_functional { "🖥️ Οθόνη ΟΚ" } else { "❌ Βλάβη Οθόνης" };
        ui.checkbox(&mut state.display_functional, d_txt);

        let t_txt = if state.touch_responsive { "👆 Αφή ΟΚ" } else { "❌ Χωρίς Αφή" };
        ui.checkbox(&mut state.touch_responsive, t_txt);

        let a_txt = if state.audio_functional { "🔊 Ήχος ΟΚ" } else { "❌ Βλάβη Ήχου" };
        ui.checkbox(&mut state.audio_functional, a_txt);

        let c_txt = if state.camera_functional { "📷 Κάμερα ΟΚ" } else { "❌ Βλάβη Κάμερας" };
        ui.checkbox(&mut state.camera_functional, c_txt);

        let liq_col = if state.liquid_ingress_detected { STATUS_CANCELLED } else { TEXT_MUTED };
        let liq_txt = if state.liquid_ingress_detected { "💧 ⚠️ ΥΓΡΑΣΙΑ/ΟΞΕΙΔΩΣΗ" } else { "💧 Χωρίς Υγρασία" };
        if ui.checkbox(&mut state.liquid_ingress_detected, RichText::new(liq_txt).color(liq_col)).changed()
            && state.liquid_ingress_detected {
            state.status_msg = Some(("⚠️ Προσοχή: Εντοπίστηκε υγρασία. Απαιτείται αποξείδωση.".to_string(), true));
        }
    });

    // Battery Health Diagnostic
    ui.horizontal(|ui| {
        ui.checkbox(&mut state.has_battery, "Μπαταρία:");
        if state.has_battery {
            ui.add(egui::Slider::new(&mut state.battery_health_pct, 1..=100).text("% Υγεία"));
            let (badge_txt, badge_col) = if state.battery_health_pct >= 80 {
                ("🟢 Φυσιολογική", STATUS_READY)
            } else if state.battery_health_pct >= 70 {
                ("🟡 Υποβαθμισμένη", ACCENT_GOLD)
            } else {
                ("🔴 Σέρβις Μπαταρίας", STATUS_CANCELLED)
            };
            ui.colored_label(badge_col, badge_txt);

            if ui.small_button("100%").clicked() { state.battery_health_pct = 100; }
            if ui.small_button("85%").clicked() { state.battery_health_pct = 85; }
            if ui.small_button("65%").clicked() { state.battery_health_pct = 65; }
        } else {
            ui.colored_label(TEXT_MUTED, "(Σταθερή συσκευή / Χωρίς μπαταρία)");
        }
    });

    // Cosmetic Condition with Quick Presets
    ui.horizontal(|ui| {
        ui.label(RichText::new("Κατάσταση:").size(11.0).color(TEXT_MUTED));
        ui.add(egui::TextEdit::singleline(&mut state.cosmetic_condition).desired_width(120.0));
        let cond_presets = ["Άριστη", "Καλή", "Γρατζουνιές", "Σπασμένη"];
        for p in cond_presets {
            if ui.small_button(p).clicked() {
                state.cosmetic_condition = p.to_string();
            }
        }
    });

    // Accessories with Quick Presets
    ui.horizontal(|ui| {
        ui.label(RichText::new("Αξεσουάρ:").size(11.0).color(TEXT_MUTED));
        ui.add(egui::TextEdit::singleline(&mut state.customer_accessories).desired_width(120.0));
        let acc_presets = ["Μόνο συσκευή", "+Φορτιστής", "+Θήκη", "+Κουτί"];
        for a in acc_presets {
            if ui.small_button(a).clicked() {
                state.customer_accessories = a.to_string();
            }
        }

        if ui.button(RichText::new("💾 Αποθήκευση Ελέγχου").strong().color(ACCENT_PRIMARY)).clicked() {
            let batt = if state.has_battery { Some(state.battery_health_pct) } else { None };
            let now = chrono::Utc::now().timestamp_millis();
            let diag = DeviceDiagnosticChecklist {
                ticket_id: ticket.ticket_id.clone(),
                powers_on: state.powers_on,
                display_functional: state.display_functional,
                touch_responsive: state.touch_responsive,
                liquid_ingress_detected: state.liquid_ingress_detected,
                audio_functional: state.audio_functional,
                camera_functional: state.camera_functional,
                battery_health_pct: batt,
                cosmetic_condition: state.cosmetic_condition.clone(),
                customer_accessories: state.customer_accessories.clone(),
                inspected_by: operator_name.to_string(),
                inspected_at: now,
            };
            let _ = save_diagnostic_checklist(conn, &diag);
            state.diag_saved_at = Some(now);
            state.inspector_name = operator_name.to_string();
            state.status_msg = Some(("✓ Ο διαγνωστικός έλεγχος αποθηκεύτηκε με επιτυχία.".to_string(), false));
        }
    });
}

fn load_bench_data(conn: &Connection, state: &mut TicketBenchState, ticket_id: &str) {
    if let Ok(Some(d)) = get_diagnostic_checklist(conn, ticket_id) {
        state.powers_on = d.powers_on;
        state.display_functional = d.display_functional;
        state.touch_responsive = d.touch_responsive;
        state.liquid_ingress_detected = d.liquid_ingress_detected;
        state.audio_functional = d.audio_functional;
        state.camera_functional = d.camera_functional;
        state.has_battery = d.battery_health_pct.is_some();
        state.battery_health_pct = d.battery_health_pct.unwrap_or(100);
        state.cosmetic_condition = d.cosmetic_condition;
        state.customer_accessories = d.customer_accessories;
        state.diag_saved_at = Some(d.inspected_at);
        state.inspector_name = d.inspected_by;
    }
    reload_bench_records(conn, state, ticket_id);
    state.diag_loaded = true;
}

pub fn reload_bench_records(conn: &Connection, state: &mut TicketBenchState, ticket_id: &str) {
    state.cached_parts = list_consumed_parts(conn, ticket_id).unwrap_or_default();
    state.cached_labor = list_labor_logs(conn, ticket_id).unwrap_or_default();
    state.cached_summary = compute_bench_summary(conn, ticket_id).unwrap_or_default();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bench_state_defaults() {
        let state = TicketBenchState::default();
        assert!(state.powers_on);
        assert!(state.display_functional);
        assert!(!state.liquid_ingress_detected);
        assert_eq!(state.battery_health_pct, 100);
        assert_eq!(state.cosmetic_condition, "Καλή");
        assert_eq!(state.labor_duration_mins, 30);
    }
}
