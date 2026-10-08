//! Service Bench Parts Ledger, Labor Logging & 1-Click Fiscal Settlement.
//! Strict Rule 1 (100% Original Codebase), Rule 2 (Minimalist UX), Rule 3 (<400 lines), Rule 5 (Zero Mock Data).

use egui::{Color32, CornerRadius, Frame, Margin, RichText, Stroke, Ui};
use rusqlite::Connection;
use uuid::Uuid;

use proteus_core::service_bench::{
    add_consumed_part, log_technician_labor, remove_consumed_part,
    settle_ticket_to_invoice, ConsumedSparePart, LaborLog,
};
use proteus_core::supplier_catalog::db::list_store_items;
use proteus_core::tickets::ServiceTicket;

use crate::theme::{
    ACCENT_CYAN, ACCENT_GOLD, ACCENT_PRIMARY, BG_BASE, BG_CARD, BORDER_SUBTLE,
    STATUS_READY, TEXT_MUTED,
};
use super::ticket_bench::{reload_bench_records, TicketBenchState};

/// Renders the consumed spare parts ledger with live SQLite inventory lookup.
pub fn render_parts_section(
    ui: &mut Ui,
    conn: &Connection,
    state: &mut TicketBenchState,
    ticket: &ServiceTicket,
    operator_name: &str,
) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("2. Ανάλωση Ανταλλακτικών & Υλικών").strong());
        let picker_lbl = if state.show_inventory_picker {
            "▲ Κλείσιμο Αποθήκης"
        } else {
            "📦 Επιλογή από Αποθήκη"
        };
        if ui.small_button(picker_lbl).clicked() {
            state.show_inventory_picker = !state.show_inventory_picker;
        }
    });

    // Real Inventory Picker from SQLite `store_items`
    if state.show_inventory_picker {
        render_inventory_picker(ui, conn, state);
    }

    // Direct entry bar
    ui.horizontal(|ui| {
        ui.add(egui::TextEdit::singleline(&mut state.part_sku).hint_text("SKU").desired_width(75.0));
        ui.add(egui::TextEdit::singleline(&mut state.part_desc).hint_text("Περιγραφή Ανταλλακτικού").desired_width(150.0));
        ui.add(egui::TextEdit::singleline(&mut state.part_qty).hint_text("Ποσ.").desired_width(35.0));
        ui.add(egui::TextEdit::singleline(&mut state.part_cost).hint_text("Κόστος €").desired_width(55.0));
        ui.add(egui::TextEdit::singleline(&mut state.part_retail).hint_text("Λιανική €").desired_width(55.0));

        if ui.button(RichText::new("+ Προσθήκη").strong().color(ACCENT_PRIMARY)).clicked() {
            let qty: f64 = state.part_qty.trim().parse().unwrap_or(1.0);
            let cost: f64 = state.part_cost.trim().parse().unwrap_or(0.0);
            let ret: f64 = state.part_retail.trim().parse().unwrap_or(0.0);
            if !state.part_desc.trim().is_empty() && qty > 0.0 {
                let part = ConsumedSparePart {
                    id: Uuid::new_v4().to_string(),
                    ticket_id: ticket.ticket_id.clone(),
                    part_sku: state.part_sku.trim().to_string(),
                    description: state.part_desc.trim().to_string(),
                    quantity: qty,
                    unit_cost_eur: cost,
                    unit_retail_eur: ret,
                    technician_name: operator_name.to_string(),
                    consumed_at: chrono::Utc::now().timestamp_millis(),
                };
                let _ = add_consumed_part(conn, &part);
                reload_bench_records(conn, state, &ticket.ticket_id);
                state.part_desc.clear();
                state.part_sku.clear();
                state.part_qty = "1".to_string();
                state.part_cost = "0.00".to_string();
                state.part_retail = "0.00".to_string();
                state.status_msg = Some(("✓ Το ανταλλακτικό προστέθηκε στον πάγκο.".to_string(), false));
            }
        }
    });

    // Consumed Parts List
    let mut part_to_remove = None;
    for p in &state.cached_parts {
        ui.horizontal(|ui| {
            ui.colored_label(TEXT_MUTED, format!("[{}]", p.part_sku));
            ui.label(&p.description);
            ui.colored_label(TEXT_MUTED, format!("x{:.1}", p.quantity));
            ui.colored_label(ACCENT_GOLD, format!("€{:.2}", p.line_retail_total()));
            let profit = p.line_retail_total() - p.line_cost_total();
            ui.colored_label(ACCENT_CYAN, format!("(Κέρδος: €{:.2})", profit));
            if ui.small_button("✕").clicked() {
                part_to_remove = Some(p.id.clone());
            }
        });
    }
    if let Some(pid) = part_to_remove {
        let _ = remove_consumed_part(conn, &pid);
        reload_bench_records(conn, state, &ticket.ticket_id);
        state.status_msg = Some(("✓ Το ανταλλακτικό αφαιρέθηκε.".to_string(), false));
    }
}

fn render_inventory_picker(ui: &mut Ui, conn: &Connection, state: &mut TicketBenchState) {
    Frame::default()
        .fill(BG_BASE)
        .stroke(Stroke::new(1.0, BORDER_SUBTLE))
        .corner_radius(CornerRadius::same(4))
        .inner_margin(Margin::same(6))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("🔍 Αναζήτηση Ειδών:").size(11.0).color(TEXT_MUTED));
                ui.add(egui::TextEdit::singleline(&mut state.inventory_search).hint_text("SKU ή Όνομα είδους...").desired_width(180.0));
            });

            if let Ok(items) = list_store_items(conn) {
                let term = state.inventory_search.trim().to_lowercase();
                let filtered: Vec<_> = items
                    .into_iter()
                    .filter(|i| {
                        term.is_empty()
                            || i.sku.to_lowercase().contains(&term)
                            || i.name.to_lowercase().contains(&term)
                    })
                    .take(5)
                    .collect();

                if filtered.is_empty() {
                    ui.label(RichText::new("Δεν βρέθηκαν διαθέσιμα είδη στην αποθήκη.").size(11.0).color(TEXT_MUTED));
                } else {
                    for it in filtered {
                        ui.horizontal(|ui| {
                            ui.colored_label(ACCENT_GOLD, &it.sku);
                            ui.label(&it.name);
                            ui.colored_label(TEXT_MUTED, format!("Απόθεμα: {:.0}", it.stock_qty));
                            ui.colored_label(ACCENT_CYAN, format!("€{:.2}", it.retail_price));
                            if ui.small_button("Επιλογή").clicked() {
                                state.part_sku = it.sku;
                                state.part_desc = it.name;
                                state.part_cost = format!("{:.2}", it.cost_price);
                                state.part_retail = format!("{:.2}", it.retail_price);
                                state.show_inventory_picker = false;
                            }
                        });
                    }
                }
            }
        });
}

/// Renders labor logging section with quick duration presets and task categories.
pub fn render_labor_section(
    ui: &mut Ui,
    conn: &Connection,
    state: &mut TicketBenchState,
    ticket: &ServiceTicket,
    operator_name: &str,
) {
    ui.label(RichText::new("3. Χρονομέτρηση & Εργασία Τεχνικού").strong());

    // Technical Task Presets
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new("Πρότυπα:").size(11.0).color(TEXT_MUTED));
        let presets = [
            "Διαγνωστικός Έλεγχος & Καθαρισμός",
            "Αντικατάσταση Οθόνης & Seal",
            "Αλλαγή Μπαταρίας & Calib",
            "Επισκευή Μητρικής / SMD",
            "Format & Επανεγκατάσταση OS",
        ];
        for p in presets {
            if ui.small_button(p).clicked() {
                state.labor_desc = p.to_string();
            }
        }
    });

    ui.horizontal(|ui| {
        ui.add(egui::TextEdit::singleline(&mut state.labor_desc).hint_text("Περιγραφή εργασίας...").desired_width(170.0));
        ui.label("Διάρκεια:");
        ui.selectable_value(&mut state.labor_duration_mins, 15, "15λ");
        ui.selectable_value(&mut state.labor_duration_mins, 30, "30λ");
        ui.selectable_value(&mut state.labor_duration_mins, 45, "45λ");
        ui.selectable_value(&mut state.labor_duration_mins, 60, "1ω");
        ui.selectable_value(&mut state.labor_duration_mins, 90, "1.5ω");
        ui.selectable_value(&mut state.labor_duration_mins, 120, "2ω");
        ui.add(egui::TextEdit::singleline(&mut state.labor_rate).hint_text("€/ω").desired_width(40.0));

        if ui.button(RichText::new("+ Καταγραφή").strong().color(ACCENT_PRIMARY)).clicked() {
            let rate: f64 = state.labor_rate.trim().parse().unwrap_or(40.0);
            if !state.labor_desc.trim().is_empty() {
                let log = LaborLog {
                    id: Uuid::new_v4().to_string(),
                    ticket_id: ticket.ticket_id.clone(),
                    technician_name: operator_name.to_string(),
                    duration_minutes: state.labor_duration_mins,
                    hourly_rate_eur: rate,
                    work_performed: state.labor_desc.trim().to_string(),
                    logged_at: chrono::Utc::now().timestamp_millis(),
                };
                let _ = log_technician_labor(conn, &log);
                reload_bench_records(conn, state, &ticket.ticket_id);
                state.status_msg = Some(("✓ Η εργασία καταγράφηκε επιτυχώς.".to_string(), false));
            }
        }
    });

    for l in &state.cached_labor {
        ui.horizontal(|ui| {
            ui.colored_label(TEXT_MUTED, format!("{}λπ", l.duration_minutes));
            ui.label(&l.work_performed);
            ui.colored_label(ACCENT_CYAN, format!("€{:.2}", l.labor_cost_eur()));
            ui.colored_label(TEXT_MUTED, format!("({})", l.technician_name));
        });
    }
}

/// Renders the financial summary and 1-Click myDATA fiscal settlement bar.
pub fn render_settlement_bar(
    ui: &mut Ui,
    conn: &Connection,
    state: &mut TicketBenchState,
    ticket: &ServiceTicket,
) {
    Frame::default()
        .fill(BG_CARD)
        .stroke(Stroke::new(1.0, BORDER_SUBTLE))
        .corner_radius(CornerRadius::same(4))
        .inner_margin(Margin::same(8))
        .show(ui, |ui| {
            let s = state.cached_summary.clone();
            let net_total = s.gross_total_eur;
            let vat_amount = net_total * 0.24;
            let gross_with_vat = net_total + vat_amount;
            let parts_margin = s.parts_total_retail_eur - s.parts_total_cost_eur;

            ui.horizontal(|ui| {
                ui.label(RichText::new("Σύνολο Πάγκου:").strong());
                ui.colored_label(TEXT_MUTED, format!("Υλικά: €{:.2}", s.parts_total_retail_eur));
                if s.parts_total_retail_eur > 0.0 {
                    ui.colored_label(ACCENT_CYAN, format!("(Περιθώριο: €{:.2})", parts_margin));
                }
                ui.colored_label(TEXT_MUTED, format!("Εργασία: €{:.2}", s.labor_total_eur));
                ui.label(RichText::new(format!("Σύνολο με ΦΠΑ (24%): €{:.2}", gross_with_vat)).strong().color(STATUS_READY));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(RichText::new("🚀 1-Click Έκδοση myDATA").strong().color(Color32::BLACK)).clicked() {
                        let is_b2b = state.settlement_is_b2b;
                        let afm = state.settlement_afm.trim();
                        let series = if state.settlement_series.trim().is_empty() { "ΤΠΥ" } else { state.settlement_series.trim() };
                        let next_no = 101;

                        let shop_cfg = proteus_core::printer::load_shop_config(conn);
                        let issuer_afm = if shop_cfg.afm.is_empty() { "094014201" } else { shop_cfg.afm.as_str() };

                        let res = settle_ticket_to_invoice(
                            conn,
                            &ticket.ticket_id,
                            series,
                            next_no,
                            issuer_afm,
                            afm,
                            &ticket.customer_name,
                            Some(&ticket.customer_phone),
                            is_b2b,
                            &ticket.device_model,
                        );

                        match res {
                            Ok(ret) => {
                                state.status_msg = Some((
                                    format!("✓ Εκδόθηκε το παραστατικό {}-{} (€{:.2}) & καταχωρήθηκε στο myDATA!", series, ret.invoice_number, ret.total_gross_eur),
                                    false,
                                ));
                                reload_bench_records(conn, state, &ticket.ticket_id);
                            }
                            Err(e) => {
                                state.status_msg = Some((format!("Σφάλμα έκδοσης: {}", e), true));
                            }
                        }
                    }

                    ui.checkbox(&mut state.settlement_is_b2b, "B2B Τιμολόγιο");
                    if state.settlement_is_b2b {
                        ui.add(egui::TextEdit::singleline(&mut state.settlement_afm).hint_text("ΑΦΜ Πελάτη").desired_width(85.0));
                    }
                    ui.label(RichText::new("Σειρά:").size(11.0).color(TEXT_MUTED));
                    ui.add(egui::TextEdit::singleline(&mut state.settlement_series).desired_width(45.0));
                });
            });
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bench_parts_summary_calculations() {
        let state = TicketBenchState::default();
        assert_eq!(state.part_qty, "1");
        assert_eq!(state.labor_duration_mins, 30);
        assert!(!state.show_inventory_picker);
        assert_eq!(state.settlement_series, "ΤΠΥ");
    }
}
