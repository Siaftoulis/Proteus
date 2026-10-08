//! Modal dialogs for Financial Cardex (New Entity & Payment Receipts).
//! Strict Rule 1 (100% Original Codebase) and Rule 3 (Modular files <400 lines).

use egui::{Color32, CornerRadius, Frame, Margin, Stroke, Ui};
use rusqlite::Connection;

use proteus_core::cardex::{
    record_payment_receipt, save_cardex_entity, CardexEntity, EntityType,
};

use crate::theme::{BG_BASE, TEXT_MUTED};
use crate::views::cardex::{reload_entities, reload_selected_entity, CardexViewState};

/// Renders the modal window for creating a new Customer or Supplier cardex profile.
pub fn render_new_entity_modal(ui: &mut Ui, conn: &Connection, state: &mut CardexViewState) {
    egui::Window::new("➕ Δημιουργία Νέας Καρτέλας")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ui.ctx(), |ui| {
            ui.horizontal(|ui| {
                ui.label("Τύπος:");
                ui.radio_value(&mut state.new_entity_type, EntityType::Customer, "Πελάτης");
                ui.radio_value(&mut state.new_entity_type, EntityType::Supplier, "Προμηθευτής");
            });

            ui.horizontal(|ui| {
                ui.label("Επωνυμία / Όνομα:");
                ui.text_edit_singleline(&mut state.new_entity_name);
            });

            ui.horizontal(|ui| {
                ui.label("ΑΦΜ:");
                ui.text_edit_singleline(&mut state.new_entity_afm);
            });

            ui.horizontal(|ui| {
                ui.label("Πιστωτικό Όριο (€):");
                ui.text_edit_singleline(&mut state.new_entity_credit_limit);
            });

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("Αποθήκευση").clicked() {
                    let clean_afm = state.new_entity_afm.trim().to_string();
                    let clean_name = state.new_entity_name.trim().to_string();
                    let ent_type = state.new_entity_type;
                    let limit: f64 = state.new_entity_credit_limit.trim().parse().unwrap_or(0.0);

                    if !clean_afm.is_empty() && !clean_name.is_empty() {
                        let entity = CardexEntity {
                            entity_id: clean_afm.clone(),
                            entity_type: ent_type,
                            name: clean_name.clone(),
                            phone: None,
                            email: None,
                            credit_limit_eur: limit,
                            payment_terms_days: 30,
                            current_balance_eur: 0.0,
                            created_at: chrono::Utc::now().timestamp_millis(),
                        };
                        let _ = save_cardex_entity(conn, &entity);
                        reload_entities(conn, state);
                        state.selected_entity_id = Some(clean_afm.clone());
                        reload_selected_entity(conn, state, &clean_afm);
                        state.show_new_entity_modal = false;
                        state.status_message = Some((format!("Δημιουργήθηκε η καρτέλα {}", clean_name), false));
                    }
                }

                if ui.button("Ακύρωση").clicked() {
                    state.show_new_entity_modal = false;
                }
            });
        });
}

/// Renders the rapid payment receipt collection modal dialog.
pub fn render_payment_receipt_modal(ui: &mut Ui, conn: &Connection, state: &mut CardexViewState) {
    let entity_id = match state.selected_entity_id {
        Some(ref id) => id.clone(),
        None => {
            state.show_receipt_modal = false;
            return;
        }
    };

    egui::Window::new("💰 Καταχώρηση Είσπραξης / Πληρωμής")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ui.ctx(), |ui| {
            ui.horizontal(|ui| {
                ui.label("Ποσό (€):");
                ui.text_edit_singleline(&mut state.receipt_amount_input);
            });

            ui.horizontal(|ui| {
                ui.label("Τρόπος Πληρωμής:");
                egui::ComboBox::from_label("")
                    .selected_text(&state.receipt_payment_method)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut state.receipt_payment_method, "Μετρητά".to_string(), "Μετρητά");
                        ui.selectable_value(&mut state.receipt_payment_method, "Τραπεζική Κατάθεση".to_string(), "Τραπεζική Κατάθεση");
                        ui.selectable_value(&mut state.receipt_payment_method, "Κάρτα POS".to_string(), "Κάρτα POS");
                        ui.selectable_value(&mut state.receipt_payment_method, "Επιταγή".to_string(), "Επιταγή");
                    });
            });

            ui.horizontal(|ui| {
                ui.label("Σημειώσεις / Αιτιολογία:");
                ui.text_edit_singleline(&mut state.receipt_notes_input);
            });

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("Επιβεβαίωση & Έκδοση").clicked() {
                    let amt: f64 = state.receipt_amount_input.trim().parse().unwrap_or(0.0);
                    let method = state.receipt_payment_method.clone();
                    let notes = state.receipt_notes_input.clone();

                    if amt > 0.0 {
                        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
                        let res = record_payment_receipt(
                            conn,
                            &entity_id,
                            "E",
                            1,
                            amt,
                            &method,
                            &notes,
                            &today,
                        );
                        if res.is_ok() {
                            reload_entities(conn, state);
                            reload_selected_entity(conn, state, &entity_id);
                            state.show_receipt_modal = false;
                            state.status_message = Some((format!("Εκδόθηκε απόδειξη είσπραξης €{:.2}", amt), false));
                        }
                    }
                }

                if ui.button("Ακύρωση").clicked() {
                    state.show_receipt_modal = false;
                }
            });
        });
}

/// Renders a color-coded aging bucket badge with currency amount.
pub fn render_aging_badge(ui: &mut Ui, label: &str, amt: f64, col: Color32) {
    Frame::default()
        .fill(BG_BASE)
        .stroke(Stroke::new(1.0, col))
        .corner_radius(CornerRadius::same(4))
        .inner_margin(Margin::same(6))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.colored_label(TEXT_MUTED, label);
                ui.colored_label(col, egui::RichText::new(format!("€{:.0}", amt)).strong());
            });
        });
    ui.add_space(4.0);
}
