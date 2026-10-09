//! Customer & Supplier Financial Cardex & Balance Aging View for Proteus Client.
//! Implements sovereign commercial cardex interface:
//! - Rolling balances (Χρέωση / Πίστωση / Υπόλοιπο)
//! - 5-Tier statutory balance aging analysis (0-30d, 31-60d, 61-90d, 91-120d, 120+ days)
//! - Rapid payment collection receipts with automated debt offset
//! - Credit limit enforcement and visual debt capacity gauges
//!
//! Strict Rule 1 (100% Original Codebase) and Rule 5 (Zero Mock Data).

use egui::{Color32, CornerRadius, Frame, Margin, Stroke, Ui};
use rusqlite::Connection;

use proteus_core::cardex::{
    calculate_balance_aging, get_cardex_entity, init_cardex_schema, list_cardex_entries,
    BalanceAgingReport, CardexEntity, CardexEntry, CardexMovementType, EntityType,
};

use crate::views::cardex_modals::{
    render_aging_badge, render_new_entity_modal, render_payment_receipt_modal,
};

use crate::theme::{
    ACCENT_CYAN, ACCENT_GOLD, ACCENT_PRIMARY, BG_BASE, BG_CARD, BG_PANEL, BORDER_SUBTLE,
    STATUS_CANCELLED, STATUS_READY, TEXT_MUTED, TEXT_PRIMARY, TEXT_SECONDARY,
};

/// UI state for Financial Cardex & Debt Manager.
pub struct CardexViewState {
    pub search_query: String,
    pub selected_entity_id: Option<String>,
    pub filter_entity_type: Option<EntityType>,
    pub cached_entities: Vec<CardexEntity>,
    pub cached_entries: Vec<CardexEntry>,
    pub cached_aging: Option<BalanceAgingReport>,
    pub show_receipt_modal: bool,
    pub receipt_amount_input: String,
    pub receipt_payment_method: String,
    pub receipt_notes_input: String,
    pub status_message: Option<(String, bool)>,
    pub show_new_entity_modal: bool,
    pub new_entity_name: String,
    pub new_entity_afm: String,
    pub new_entity_type: EntityType,
    pub new_entity_credit_limit: String,
}

impl Default for CardexViewState {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            selected_entity_id: None,
            filter_entity_type: None,
            cached_entities: Vec::new(),
            cached_entries: Vec::new(),
            cached_aging: None,
            show_receipt_modal: false,
            receipt_amount_input: String::new(),
            receipt_payment_method: "Μετρητά".to_string(),
            receipt_notes_input: String::new(),
            status_message: None,
            show_new_entity_modal: false,
            new_entity_name: String::new(),
            new_entity_afm: String::new(),
            new_entity_type: EntityType::Customer,
            new_entity_credit_limit: "1000.00".to_string(),
        }
    }
}

/// Renders the complete Financial Cardex View.
pub fn draw_cardex_view(ui: &mut Ui, conn: &Connection, state: &mut CardexViewState) {
    let _ = init_cardex_schema(conn);

    // Initial load of entities if cache empty
    if state.cached_entities.is_empty() {
        reload_entities(conn, state);
    }

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(8.0);

        // Header and Actions Toolbar
        Frame::default()
            .fill(BG_PANEL)
            .stroke(Stroke::new(1.0, BORDER_SUBTLE))
            .corner_radius(CornerRadius::same(6))
            .inner_margin(Margin::same(12))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("👥 Οικονομικές Καρτέλες (Cardex CRM)");

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("+ Νέα Καρτέλα").clicked() {
                            state.show_new_entity_modal = true;
                        }

                        if ui.button("🔄").clicked() {
                            reload_entities(conn, state);
                            if let Some(ref id) = state.selected_entity_id.clone() {
                                reload_selected_entity(conn, state, id);
                            }
                        }

                        ui.add(
                            egui::TextEdit::singleline(&mut state.search_query)
                                .hint_text("Αναζήτηση ΑΦΜ / Επωνυμία...")
                                .desired_width(180.0),
                        );
                    });
                });
            });

        ui.add_space(8.0);

        // Status Notification banner
        if let Some((ref msg, is_err)) = state.status_message {
            let col = if is_err { STATUS_CANCELLED } else { STATUS_READY };
            Frame::default()
                .fill(col.linear_multiply(0.15))
                .stroke(Stroke::new(1.0, col))
                .corner_radius(CornerRadius::same(4))
                .inner_margin(Margin::same(8))
                .show(ui, |ui| {
                    ui.colored_label(col, msg);
                });
            ui.add_space(8.0);
        }

        // Two-column Split View: Entity Directory & Detailed Cardex
        ui.columns(2, |cols| {
            // Left Column: Directory of Entities
            render_entities_list(&mut cols[0], conn, state);

            // Right Column: Selected Entity Ledger & Aging
            render_selected_cardex(&mut cols[1], conn, state);
        });

        // Modals
        if state.show_new_entity_modal {
            render_new_entity_modal(ui, conn, state);
        }

        if state.show_receipt_modal {
            render_payment_receipt_modal(ui, conn, state);
        }
    });
}

pub fn reload_entities(conn: &Connection, state: &mut CardexViewState) {
    let sql = "SELECT entity_id, entity_type, name, phone, email, credit_limit_eur, payment_terms_days, current_balance_eur, created_at FROM cardex_entities ORDER BY current_balance_eur DESC, name ASC";
    let mut stmt = match conn.prepare(sql) {
        Ok(s) => s,
        Err(_) => return,
    };

    let rows = stmt.query_map([], |r| {
        let type_str: String = r.get(1)?;
        let ent_type = EntityType::from_str(&type_str).unwrap_or(EntityType::Customer);
        Ok(CardexEntity {
            entity_id: r.get(0)?,
            entity_type: ent_type,
            name: r.get(2)?,
            phone: r.get(3)?,
            email: r.get(4)?,
            credit_limit_eur: r.get(5)?,
            payment_terms_days: r.get(6)?,
            current_balance_eur: r.get(7)?,
            created_at: r.get(8)?,
        })
    });

    if let Ok(iter) = rows {
        state.cached_entities = iter.filter_map(|r| r.ok()).collect();
    }
}

pub fn reload_selected_entity(conn: &Connection, state: &mut CardexViewState, entity_id: &str) {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    state.cached_entries = list_cardex_entries(conn, entity_id).unwrap_or_default();
    state.cached_aging = calculate_balance_aging(conn, entity_id, &today).ok();
}

fn render_entities_list(ui: &mut Ui, conn: &Connection, state: &mut CardexViewState) {
    Frame::default()
        .fill(BG_BASE)
        .stroke(Stroke::new(1.0, BORDER_SUBTLE))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::same(10))
        .show(ui, |ui| {
            ui.label(egui::RichText::new("Κατάλογος Λογαριασμών").strong());
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                if ui.selectable_label(state.filter_entity_type.is_none(), "Όλοι").clicked() {
                    state.filter_entity_type = None;
                }
                if ui.selectable_label(state.filter_entity_type == Some(EntityType::Customer), "Πελάτες").clicked() {
                    state.filter_entity_type = Some(EntityType::Customer);
                }
                if ui.selectable_label(state.filter_entity_type == Some(EntityType::Supplier), "Προμηθευτές").clicked() {
                    state.filter_entity_type = Some(EntityType::Supplier);
                }
            });
            ui.add_space(4.0);

            let filter = state.search_query.trim().to_lowercase();
            let mut selected_to_set = None;

            egui::ScrollArea::vertical()
                .max_height(450.0)
                .show(ui, |ui| {
                    for entity in &state.cached_entities {
                        if let Some(target_type) = state.filter_entity_type {
                            if entity.entity_type != target_type {
                                continue;
                            }
                        }
                        if !filter.is_empty()
                            && !entity.name.to_lowercase().contains(&filter)
                            && !entity.entity_id.contains(&filter)
                        {
                            continue;
                        }

                        let is_selected = state.selected_entity_id.as_deref() == Some(&entity.entity_id);
                        let bg = if is_selected { BG_PANEL } else { BG_CARD };

                        Frame::default()
                            .fill(bg)
                            .stroke(Stroke::new(
                                if is_selected { 1.5 } else { 1.0 },
                                if is_selected { ACCENT_PRIMARY } else { BORDER_SUBTLE },
                            ))
                            .corner_radius(CornerRadius::same(4))
                            .inner_margin(Margin::same(8))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    let badge_col = match entity.entity_type {
                                        EntityType::Customer => ACCENT_CYAN,
                                        EntityType::Supplier => ACCENT_GOLD,
                                    };
                                    ui.colored_label(badge_col, format!("[{}]", entity.entity_type.display_name()));
                                    ui.strong(&entity.name);

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        let bal_col = if entity.current_balance_eur > 0.0 {
                                            STATUS_CANCELLED
                                        } else {
                                            STATUS_READY
                                        };
                                        ui.colored_label(bal_col, format!("€{:.2}", entity.current_balance_eur));
                                    });
                                });

                                ui.horizontal(|ui| {
                                    ui.colored_label(TEXT_MUTED, format!("ΑΦΜ: {}", entity.entity_id));
                                    if entity.credit_limit_eur > 0.0 {
                                        ui.colored_label(TEXT_SECONDARY, format!("Όριο: €{:.0}", entity.credit_limit_eur));
                                    }
                                    if ui.button("Προβολή").clicked() {
                                        selected_to_set = Some(entity.entity_id.clone());
                                    }
                                });
                            });
                        ui.add_space(4.0);
                    }
                });

            if let Some(id) = selected_to_set {
                state.selected_entity_id = Some(id.clone());
                reload_selected_entity(conn, state, &id);
            }
        });
}

fn render_selected_cardex(ui: &mut Ui, conn: &Connection, state: &mut CardexViewState) {
    let entity_id = match state.selected_entity_id {
        Some(ref id) => id.clone(),
        None => {
            Frame::default()
                .fill(BG_BASE)
                .stroke(Stroke::new(1.0, BORDER_SUBTLE))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::same(20))
                .show(ui, |ui| {
                    ui.colored_label(TEXT_MUTED, "Επιλέξτε λογαριασμό από τον κατάλογο για προβολή κινήσεων και ενηλικίωσης.");
                });
            return;
        }
    };

    let entity = match get_cardex_entity(conn, &entity_id) {
        Ok(Some(e)) => e,
        _ => return,
    };

    Frame::default()
        .fill(BG_CARD)
        .stroke(Stroke::new(1.0, BORDER_SUBTLE))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            // Header summary
            ui.horizontal(|ui| {
                ui.heading(&entity.name);
                ui.colored_label(ACCENT_CYAN, format!("(ΑΦΜ: {})", entity.entity_id));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("💰 Είσπραξη / Πληρωμή").clicked() {
                        state.receipt_amount_input = format!("{:.2}", entity.current_balance_eur);
                        state.show_receipt_modal = true;
                    }
                });
            });

            ui.add_space(6.0);

            // Balance & Credit Limit bar
            ui.horizontal(|ui| {
                ui.label(format!("Τρέχον Υπόλοιπο: €{:.2}", entity.current_balance_eur));
                if entity.credit_limit_eur > 0.0 {
                    let ratio = (entity.current_balance_eur / entity.credit_limit_eur).clamp(0.0, 1.0) as f32;
                    ui.label(format!("Πιστωτικό Όριο: €{:.2}", entity.credit_limit_eur));
                    ui.add(egui::ProgressBar::new(ratio).desired_width(120.0).text(format!("{:.0}%", ratio * 100.0)));
                }
            });

            ui.add_space(10.0);

            // 5-Tier Aging Bar
            if let Some(ref aging) = state.cached_aging {
                ui.label(egui::RichText::new("Ενηλικίωση Υπολοίπων").strong());
                ui.horizontal(|ui| {
                    render_aging_badge(ui, "0-30ημ", aging.current_0_30_eur, STATUS_READY);
                    render_aging_badge(ui, "31-60ημ", aging.overdue_31_60_eur, ACCENT_PRIMARY);
                    render_aging_badge(ui, "61-90ημ", aging.overdue_61_90_eur, ACCENT_GOLD);
                    render_aging_badge(ui, "91-120ημ", aging.overdue_91_120_eur, Color32::from_rgb(220, 120, 50));
                    render_aging_badge(ui, "120+ημ", aging.overdue_120_plus_eur, STATUS_CANCELLED);
                });
            }

            ui.add_space(10.0);

            // Movement Ledger Table
            ui.label(egui::RichText::new("Κινήσεις Καρτέλας (Χρέωση / Πίστωση)").strong());
            ui.add_space(4.0);

            egui::ScrollArea::vertical()
                .max_height(250.0)
                .show(ui, |ui| {
                    if state.cached_entries.is_empty() {
                        ui.colored_label(TEXT_MUTED, "Δεν υπάρχουν καταχωρημένες κινήσεις στην καρτέλα.");
                    } else {
                        for entry in state.cached_entries.iter().rev() {
                            Frame::default()
                                .fill(BG_BASE)
                                .stroke(Stroke::new(1.0, BORDER_SUBTLE))
                                .corner_radius(CornerRadius::same(4))
                                .inner_margin(Margin::same(6))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.colored_label(TEXT_MUTED, &entry.issue_date);
                                        ui.label(format!("{}-{}", entry.document_series, entry.document_number));
                                        ui.colored_label(TEXT_SECONDARY, &entry.description);

                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            ui.colored_label(TEXT_PRIMARY, format!("Υπόλ: €{:.2}", entry.running_balance_eur));
                                            match entry.movement_type {
                                                CardexMovementType::Debit => {
                                                    ui.colored_label(STATUS_CANCELLED, format!("+€{:.2}", entry.amount_eur));
                                                }
                                                CardexMovementType::Credit => {
                                                    ui.colored_label(STATUS_READY, format!("-€{:.2}", entry.amount_eur));
                                                }
                                            }
                                        });
                                    });
                                });
                            ui.add_space(2.0);
                        }
                    }
                });
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cardex_view_state_defaults() {
        let state = CardexViewState::default();
        assert!(state.search_query.is_empty());
        assert!(state.selected_entity_id.is_none());
        assert!(state.cached_entities.is_empty());
        assert_eq!(state.receipt_payment_method, "Μετρητά");
    }
}
