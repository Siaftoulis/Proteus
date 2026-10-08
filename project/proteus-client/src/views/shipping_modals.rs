//! Modal Dialogs and Inspection Drawers for Shipping Notes.
//! Strict Rule 1 (100% Original Codebase), Rule 2 (Minimalist UX), Rule 5 (Zero Mock Data).

use egui::{Color32, RichText, Ui, Vec2};
use rusqlite::Connection;

use proteus_core::shipping_note::{
    add_shipping_note_item, create_shipping_note, list_shipping_note_items,
    record_delivery_completion, ShippingNote, ShippingNoteItem, TransportPurpose,
};
use proteus_core::smlm::ContentProfiler;

use super::shipping_notes::ShippingNotesViewState;

pub fn draw_new_shipping_note_modal(ui: &mut Ui, conn: &Connection, state: &mut ShippingNotesViewState) {
    egui::Window::new("Έκδοση Νέου Ψηφιακού Δελτίου Αποστολής")
        .collapsible(false)
        .resizable(false)
        .fixed_size(Vec2::new(560.0, 520.0))
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ui.ctx(), |ui| {
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                ui.label(RichText::new("Αριθμός Δελτίου:").strong());
                ui.text_edit_singleline(&mut state.note_number_input);
                ui.add_space(8.0);
                ui.label(RichText::new("Σκοπός:").strong());
                egui::ComboBox::from_id_salt("purpose_combo")
                    .selected_text(state.purpose.display_name())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut state.purpose, TransportPurpose::Sale, TransportPurpose::Sale.display_name());
                        ui.selectable_value(&mut state.purpose, TransportPurpose::Repair, TransportPurpose::Repair.display_name());
                        ui.selectable_value(&mut state.purpose, TransportPurpose::TransferBetweenBranches, TransportPurpose::TransferBetweenBranches.display_name());
                        ui.selectable_value(&mut state.purpose, TransportPurpose::ReturnToSupplier, TransportPurpose::ReturnToSupplier.display_name());
                        ui.selectable_value(&mut state.purpose, TransportPurpose::Consignment, TransportPurpose::Consignment.display_name());
                        ui.selectable_value(&mut state.purpose, TransportPurpose::Sample, TransportPurpose::Sample.display_name());
                    });
            });

            ui.separator();
            ui.label(RichText::new("Στοιχεία Παραλήπτη (Εντολέα / Πελάτη)").strong().color(Color32::from_rgb(96, 165, 250)));
            ui.horizontal(|ui| {
                ui.label("ΑΦΜ:");
                ui.text_edit_singleline(&mut state.recipient_afm);
                let is_afm_valid = ContentProfiler::is_greek_afm(&state.recipient_afm);
                if is_afm_valid {
                    ui.label(RichText::new("✓ Έγκυρο ΑΦΜ").color(Color32::GREEN));
                } else if !state.recipient_afm.is_empty() {
                    ui.label(RichText::new("⚠ Μη έγκυρο ΑΦΜ").color(Color32::YELLOW));
                }
            });
            ui.horizontal(|ui| {
                ui.label("Επωνυμία:");
                ui.text_edit_singleline(&mut state.recipient_name);
            });
            ui.horizontal(|ui| {
                ui.label("Διεύθυνση Παράδοσης:");
                ui.text_edit_singleline(&mut state.recipient_address);
            });

            ui.separator();
            ui.label(RichText::new("Στοιχεία Μεταφοράς").strong().color(Color32::from_rgb(96, 165, 250)));
            ui.horizontal(|ui| {
                ui.label("Πινακίδα:");
                ui.text_edit_singleline(&mut state.vehicle_plate);
                ui.add_space(8.0);
                ui.label("Οδηγός:");
                ui.text_edit_singleline(&mut state.driver_name);
            });
            ui.horizontal(|ui| {
                ui.label("Πλήθος Δεμάτων:");
                ui.add(egui::DragValue::new(&mut state.packages_count).speed(1).range(1..=999));
                ui.add_space(8.0);
                ui.label("Μικτό Βάρος (kg):");
                ui.text_edit_singleline(&mut state.gross_weight_input);
            });

            ui.separator();
            ui.label(RichText::new("Προσθήκη Ειδών Διακίνησης").strong().color(Color32::from_rgb(96, 165, 250)));
            ui.horizontal(|ui| {
                ui.label("SKU:");
                ui.add(egui::TextEdit::singleline(&mut state.item_sku).desired_width(70.0));
                ui.label("Περιγραφή:");
                ui.add(egui::TextEdit::singleline(&mut state.item_desc).desired_width(140.0));
                ui.label("Ποσ.:");
                ui.add(egui::DragValue::new(&mut state.item_qty).speed(1.0).range(0.1..=10000.0));
            });
            ui.horizontal(|ui| {
                ui.label("Σειριακοί Αριθμοί (S/N):");
                ui.add(egui::TextEdit::singleline(&mut state.item_serials).hint_text("SN-01, SN-02...").desired_width(200.0));
                if ui.button("+ Προσθήκη Είδους").clicked() && !state.item_desc.is_empty() {
                    state.pending_items.push((
                        state.item_sku.clone(),
                        state.item_desc.clone(),
                        state.item_unit,
                        state.item_qty,
                        state.item_serials.clone(),
                    ));
                    state.item_sku.clear();
                    state.item_desc.clear();
                    state.item_serials.clear();
                    state.item_qty = 1.0;
                }
            });

            if !state.pending_items.is_empty() {
                ui.label(RichText::new(format!("Προς αποστολή: {} είδη", state.pending_items.len())).color(Color32::from_rgb(203, 213, 225)).size(11.0));
            }

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.button("Ακύρωση").clicked() {
                    state.show_new_modal = false;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(RichText::new("💾 Έκδοση & Αποθήκευση").color(Color32::from_rgb(15, 23, 42)).strong()).clicked() {
                        let weight = state.gross_weight_input.trim().parse::<f64>().ok();
                        let note = ShippingNote::new(
                            state.note_number_input.clone(),
                            state.issuer_afm.clone(),
                            state.issuer_name.clone(),
                            state.issuer_address.clone(),
                            state.recipient_afm.clone(),
                            state.recipient_name.clone(),
                            state.recipient_address.clone(),
                            state.vehicle_plate.clone(),
                            state.driver_name.clone(),
                            state.purpose,
                            state.packages_count,
                            weight,
                        );

                        if let Err(e) = create_shipping_note(conn, &note) {
                            state.feedback_msg = Some((format!("Σφάλμα δημιουργίας: {}", e), true));
                        } else {
                            for (idx, (sku, desc, unit, qty, serials_raw)) in state.pending_items.drain(..).enumerate() {
                                let serials: Vec<String> = serials_raw.split(',')
                                    .map(|s| s.trim().to_string())
                                    .filter(|s| !s.is_empty())
                                    .collect();
                                let item = ShippingNoteItem {
                                    id: format!("item-{}-{}", note.id, idx + 1),
                                    note_id: note.id.clone(),
                                    line_number: (idx + 1) as u32,
                                    sku,
                                    description: desc,
                                    unit,
                                    quantity: qty,
                                    serial_numbers: serials,
                                    batch_lot: None,
                                    notes: None,
                                };
                                let _ = add_shipping_note_item(conn, &item);
                            }
                            state.feedback_msg = Some((format!("Το {} εκδόθηκε με επιτυχία.", note.note_number), false));
                            state.show_new_modal = false;
                        }
                    }
                });
            });
        });
}

pub fn draw_items_drawer_modal(ui: &mut Ui, conn: &Connection, state: &mut ShippingNotesViewState) {
    let note_id = state.active_items_note_id.clone().unwrap_or_default();
    let items = list_shipping_note_items(conn, &note_id).unwrap_or_default();

    egui::Window::new("Είδη & Σειριακοί Αριθμοί Δελτίου")
        .collapsible(false)
        .fixed_size(Vec2::new(480.0, 320.0))
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ui.ctx(), |ui| {
            if items.is_empty() {
                ui.label("Δεν έχουν καταχωρηθεί αναλυτικές γραμμές ειδών.");
            } else {
                egui::ScrollArea::vertical().max_height(240.0).show(ui, |ui| {
                    for (i, item) in items.iter().enumerate() {
                        ui.label(RichText::new(format!("{}. {} (x{:.0} {})", i + 1, item.description, item.quantity, item.unit.display_name())).strong());
                        if !item.sku.is_empty() {
                            ui.label(RichText::new(format!("   SKU: {}", item.sku)).color(Color32::from_rgb(148, 163, 184)).size(11.0));
                        }
                        if !item.serial_numbers.is_empty() {
                            ui.label(RichText::new(format!("   S/N: {}", item.serial_numbers.join(", "))).color(Color32::from_rgb(96, 165, 250)).size(11.0));
                        }
                        ui.separator();
                    }
                });
            }

            ui.add_space(8.0);
            if ui.button("Κλείσιμο").clicked() {
                state.active_items_note_id = None;
            }
        });
}

pub fn draw_delivery_sign_modal(ui: &mut Ui, conn: &Connection, state: &mut ShippingNotesViewState) {
    let note_id = state.active_delivery_note_id.clone().unwrap_or_default();

    egui::Window::new("Καταγραφή Ολοκλήρωσης Παράδοσης")
        .collapsible(false)
        .fixed_size(Vec2::new(420.0, 200.0))
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ui.ctx(), |ui| {
            ui.label(RichText::new("Σημείωση / Υπογραφή Παραλήπτη:").strong());
            ui.text_edit_multiline(&mut state.delivery_sign_input);
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                if ui.button("Ακύρωση").clicked() {
                    state.active_delivery_note_id = None;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(RichText::new("✔ Επιβεβαίωση Παράδοσης").color(Color32::from_rgb(15, 23, 42)).strong()).clicked() {
                        let now = chrono::Utc::now().to_rfc3339();
                        let sign = if state.delivery_sign_input.trim().is_empty() {
                            "Παρελήφθη ανεπιφύλακτα".to_string()
                        } else {
                            state.delivery_sign_input.trim().to_string()
                        };
                        let _ = record_delivery_completion(conn, &note_id, &sign, &now);
                        state.feedback_msg = Some(("Η παράδοση καταγράφηκε επιτυχώς.".to_string(), false));
                        state.active_delivery_note_id = None;
                    }
                });
            });
        });
}
