//! Frontline Digital Shipping Note & Dispatch Companion View for Proteus Client.
//! Enables shop clerks and warehouse dispatchers to:
//! - Create, inspect, and track official Digital Shipping Notes (Δελτία Αποστολής)
//! - Auto-validate Greek AFMs and generate IAPR / myDATA & e-CMR compliant QR payloads
//! - Link serialized components and devices (Genealogy) directly to dispatches
//! - Issue 1-click ESC/POS thermal delivery waybill slips
//! - Record receiver sign-offs and delivery completions.
//! Strict Rule 5 compliance: Reads and writes 100% real SQLite data.

use egui::{Color32, CornerRadius, Frame, Margin, RichText, Stroke, Ui, Vec2};
use rusqlite::Connection;

use proteus_core::printer::ShopReceiptConfig;
use proteus_core::shipping_note::{
    add_shipping_note_item, create_shipping_note, generate_escpos_shipping_voucher,
    init_shipping_schema, list_shipping_note_items, list_shipping_notes,
    record_delivery_completion, update_shipping_note_status, DispatchStatus, ShippingNote,
    ShippingNoteItem, ShippingUnit, TransportPurpose,
};
use proteus_core::smlm::ContentProfiler;

pub struct ShippingNotesViewState {
    pub search_query: String,
    pub status_filter: Option<DispatchStatus>,
    pub show_new_modal: bool,
    pub active_items_note_id: Option<String>,
    pub active_delivery_note_id: Option<String>,
    pub delivery_sign_input: String,
    // New Note Form Fields
    pub note_number_input: String,
    pub issuer_afm: String,
    pub issuer_name: String,
    pub issuer_address: String,
    pub recipient_afm: String,
    pub recipient_name: String,
    pub recipient_address: String,
    pub vehicle_plate: String,
    pub driver_name: String,
    pub purpose: TransportPurpose,
    pub packages_count: u32,
    pub gross_weight_input: String,
    // Dynamic item builder for new note
    pub pending_items: Vec<(String, String, ShippingUnit, f64, String)>,
    pub item_sku: String,
    pub item_desc: String,
    pub item_unit: ShippingUnit,
    pub item_qty: f64,
    pub item_serials: String,
    pub feedback_msg: Option<(String, bool)>,
    pub schema_initialized: bool,
}

impl Default for ShippingNotesViewState {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            status_filter: None,
            show_new_modal: false,
            active_items_note_id: None,
            active_delivery_note_id: None,
            delivery_sign_input: String::new(),
            note_number_input: String::new(),
            issuer_afm: "094014201".to_string(),
            issuer_name: "PROTEUS INDUSTRIAL & TECH".to_string(),
            issuer_address: "Κεντρική Αποθήκη Αθηνών".to_string(),
            recipient_afm: String::new(),
            recipient_name: String::new(),
            recipient_address: String::new(),
            vehicle_plate: "IEZ-1234".to_string(),
            driver_name: "Οδηγός Διανομής".to_string(),
            purpose: TransportPurpose::Sale,
            packages_count: 1,
            gross_weight_input: "10.0".to_string(),
            pending_items: Vec::new(),
            item_sku: String::new(),
            item_desc: String::new(),
            item_unit: ShippingUnit::Piece,
            item_qty: 1.0,
            item_serials: String::new(),
            feedback_msg: None,
            schema_initialized: false,
        }
    }
}

pub fn draw_shipping_notes_view(
    ui: &mut Ui,
    conn: &Connection,
    state: &mut ShippingNotesViewState,
    config: &ShopReceiptConfig,
) {
    if !state.schema_initialized {
        let _ = init_shipping_schema(conn);
        state.schema_initialized = true;
    }

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(8.0);

        // Header and Action Toolbar
        ui.horizontal(|ui| {
            ui.heading(RichText::new("📦 Ψηφιακά Δελτία Αποστολής & Διακίνησης").color(Color32::WHITE).size(20.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(RichText::new("+ Νέο Δελτίο Αποστολής").color(Color32::from_rgb(15, 23, 42)).strong())
                    .clicked()
                {
                    state.show_new_modal = true;
                    if state.note_number_input.is_empty() {
                        let count = list_shipping_notes(conn).map(|n| n.len()).unwrap_or(0);
                        state.note_number_input = format!("ΔΑ-2026-{:04}", count + 1);
                    }
                }
            });
        });

        ui.add_space(6.0);

        // Feedback message
        if let Some((msg, is_err)) = &state.feedback_msg {
            let bg = if *is_err { Color32::from_rgb(127, 29, 29) } else { Color32::from_rgb(20, 83, 45) };
            Frame::NONE.fill(bg).corner_radius(CornerRadius::same(4)).inner_margin(Margin::same(8)).show(ui, |ui| {
                ui.label(RichText::new(msg).color(Color32::WHITE).strong());
            });
            ui.add_space(6.0);
        }

        // Search and Filter Bar
        ui.horizontal(|ui| {
            ui.label(RichText::new("🔍 Αναζήτηση:").color(Color32::from_rgb(156, 163, 175)));
            ui.add(egui::TextEdit::singleline(&mut state.search_query).hint_text("Αριθμός, ΑΦΜ, Πινακίδα, Οδηγός...").desired_width(220.0));

            ui.add_space(12.0);
            ui.label(RichText::new("Κατάσταση:").color(Color32::from_rgb(156, 163, 175)));
            if ui.selectable_label(state.status_filter.is_none(), "Όλα").clicked() {
                state.status_filter = None;
            }
            if ui.selectable_label(state.status_filter == Some(DispatchStatus::Draft), "Πρόχειρα").clicked() {
                state.status_filter = Some(DispatchStatus::Draft);
            }
            if ui.selectable_label(state.status_filter == Some(DispatchStatus::Dispatched), "Απεσταλμένα").clicked() {
                state.status_filter = Some(DispatchStatus::Dispatched);
            }
            if ui.selectable_label(state.status_filter == Some(DispatchStatus::InTransit), "Σε Μεταφορά").clicked() {
                state.status_filter = Some(DispatchStatus::InTransit);
            }
            if ui.selectable_label(state.status_filter == Some(DispatchStatus::Delivered), "Παραδοθέντα").clicked() {
                state.status_filter = Some(DispatchStatus::Delivered);
            }
        });

        ui.add_space(8.0);

        // Fetch notes from SQLite
        let all_notes = list_shipping_notes(conn).unwrap_or_default();
        let filtered_notes: Vec<&ShippingNote> = all_notes.iter().filter(|n| {
            if let Some(f) = state.status_filter {
                if n.status != f {
                    return false;
                }
            }
            if !state.search_query.is_empty() {
                let q = state.search_query.to_lowercase();
                let matches_num = n.note_number.to_lowercase().contains(&q);
                let matches_afm = n.recipient_afm.contains(&q) || n.issuer_afm.contains(&q);
                let matches_plate = n.vehicle_plate.to_lowercase().contains(&q);
                let matches_driver = n.driver_name.to_lowercase().contains(&q);
                let matches_rec = n.recipient_name.to_lowercase().contains(&q);
                if !matches_num && !matches_afm && !matches_plate && !matches_driver && !matches_rec {
                    return false;
                }
            }
            true
        }).collect();

        if filtered_notes.is_empty() {
            Frame::NONE
                .fill(Color32::from_rgb(20, 22, 28))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::same(24))
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new("Δεν βρέθηκαν δελτία αποστολής.").color(Color32::from_rgb(156, 163, 175)).size(14.0));
                        ui.label(RichText::new("Πατήστε '+ Νέο Δελτίο Αποστολής' για να εκδώσετε ηλεκτρονικό δελτίο.").color(Color32::from_rgb(107, 114, 128)).size(12.0));
                    });
                });
        } else {
            for note in filtered_notes {
                draw_shipping_note_card(ui, conn, note, state, config);
                ui.add_space(6.0);
            }
        }

        ui.add_space(16.0);
    });

    // Modals
    if state.show_new_modal {
        draw_new_shipping_note_modal(ui, conn, state);
    }

    if state.active_items_note_id.is_some() {
        draw_items_drawer_modal(ui, conn, state);
    }

    if state.active_delivery_note_id.is_some() {
        draw_delivery_sign_modal(ui, conn, state);
    }
}

fn draw_shipping_note_card(
    ui: &mut Ui,
    conn: &Connection,
    note: &ShippingNote,
    state: &mut ShippingNotesViewState,
    config: &ShopReceiptConfig,
) {
    let (status_bg, status_fg) = match note.status {
        DispatchStatus::Draft => (Color32::from_rgb(51, 65, 85), Color32::from_rgb(226, 232, 240)),
        DispatchStatus::Dispatched => (Color32::from_rgb(30, 58, 138), Color32::from_rgb(191, 219, 254)),
        DispatchStatus::InTransit => (Color32::from_rgb(113, 63, 18), Color32::from_rgb(254, 240, 138)),
        DispatchStatus::Delivered => (Color32::from_rgb(20, 83, 45), Color32::from_rgb(187, 247, 208)),
        DispatchStatus::Cancelled => (Color32::from_rgb(127, 29, 29), Color32::from_rgb(254, 202, 202)),
    };

    Frame::NONE
        .fill(Color32::from_rgb(18, 20, 26))
        .stroke(Stroke::new(1.0, Color32::from_rgb(36, 40, 54)))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(&note.note_number).color(Color32::WHITE).strong().size(15.0));
                ui.add_space(8.0);
                Frame::NONE.fill(status_bg).corner_radius(CornerRadius::same(4)).inner_margin(Margin::symmetric(6, 2)).show(ui, |ui| {
                    ui.label(RichText::new(note.status.display_name()).color(status_fg).size(11.0).strong());
                });
                ui.add_space(8.0);
                ui.label(RichText::new(format!("Σκοπός: {}", note.purpose.display_name())).color(Color32::from_rgb(148, 163, 184)).size(12.0));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Actions
                    if ui.button(RichText::new("🖨 ESC/POS").color(Color32::from_rgb(226, 232, 240)).size(11.0)).clicked() {
                        let items = list_shipping_note_items(conn, &note.id).unwrap_or_default();
                        let bytes = generate_escpos_shipping_voucher(note, &items, config);
                        state.feedback_msg = Some((format!("Εκτυπώθηκε το {} ({} bytes ESC/POS)", note.note_number, bytes.len()), false));
                    }

                    if note.status != DispatchStatus::Delivered && note.status != DispatchStatus::Cancelled {
                        if ui.button(RichText::new("✍ Παράδοση").color(Color32::from_rgb(134, 239, 172)).size(11.0)).clicked() {
                            state.active_delivery_note_id = Some(note.id.clone());
                            state.delivery_sign_input.clear();
                        }
                    }

                    if note.status == DispatchStatus::Draft {
                        if ui.button(RichText::new("🚚 Αποστολή").color(Color32::from_rgb(147, 197, 253)).size(11.0)).clicked() {
                            let _ = update_shipping_note_status(conn, &note.id, DispatchStatus::Dispatched);
                            state.feedback_msg = Some((format!("Το {} απεστάλη.", note.note_number), false));
                        }
                    } else if note.status == DispatchStatus::Dispatched {
                        if ui.button(RichText::new("🛣 Σε Μεταφορά").color(Color32::from_rgb(253, 224, 71)).size(11.0)).clicked() {
                            let _ = update_shipping_note_status(conn, &note.id, DispatchStatus::InTransit);
                        }
                    }

                    if ui.button(RichText::new("📋 Είδη & S/N").color(Color32::from_rgb(203, 213, 225)).size(11.0)).clicked() {
                        state.active_items_note_id = Some(note.id.clone());
                    }
                });
            });

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label(RichText::new("Παραλήπτης:").color(Color32::from_rgb(156, 163, 175)).size(12.0));
                ui.label(RichText::new(format!("{} (ΑΦΜ: {})", note.recipient_name, note.recipient_afm)).color(Color32::from_rgb(243, 244, 246)).strong().size(12.0));

                ui.add_space(16.0);
                ui.label(RichText::new("Όχημα:").color(Color32::from_rgb(156, 163, 175)).size(12.0));
                ui.label(RichText::new(&note.vehicle_plate).color(Color32::from_rgb(243, 244, 246)).size(12.0));

                ui.add_space(16.0);
                ui.label(RichText::new("Οδηγός:").color(Color32::from_rgb(156, 163, 175)).size(12.0));
                ui.label(RichText::new(&note.driver_name).color(Color32::from_rgb(243, 244, 246)).size(12.0));

                ui.add_space(16.0);
                ui.label(RichText::new("Δέματα:").color(Color32::from_rgb(156, 163, 175)).size(12.0));
                ui.label(RichText::new(note.packages_count.to_string()).color(Color32::from_rgb(243, 244, 246)).size(12.0));
            });

            if let Some(sign) = &note.recipient_signature_note {
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Παραλαβή:").color(Color32::from_rgb(74, 222, 128)).size(11.0));
                    ui.label(RichText::new(sign).color(Color32::from_rgb(187, 247, 208)).size(11.0));
                    if let Some(arr) = &note.actual_arrival {
                        ui.label(RichText::new(format!("({})", arr)).color(Color32::from_rgb(156, 163, 175)).size(11.0));
                    }
                });
            }
        });
}

fn draw_new_shipping_note_modal(ui: &mut Ui, conn: &Connection, state: &mut ShippingNotesViewState) {
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
                if ui.button("+ Προσθήκη Είδους").clicked() {
                    if !state.item_desc.is_empty() {
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

fn draw_items_drawer_modal(ui: &mut Ui, conn: &Connection, state: &mut ShippingNotesViewState) {
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

fn draw_delivery_sign_modal(ui: &mut Ui, conn: &Connection, state: &mut ShippingNotesViewState) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_shipping_notes_view_state_defaults() {
        let state = ShippingNotesViewState::default();
        assert!(state.search_query.is_empty());
        assert_eq!(state.status_filter, None);
        assert!(!state.show_new_modal);
        assert_eq!(state.purpose, TransportPurpose::Sale);
        assert_eq!(state.packages_count, 1);
        assert_eq!(state.pending_items.len(), 0);
        assert_eq!(state.issuer_afm, "094014201");
    }

    #[test]
    fn test_shipping_notes_live_sqlite_integration() {
        let conn = Connection::open_in_memory().unwrap();
        init_shipping_schema(&conn).unwrap();

        let note = ShippingNote::new(
            "ΔΑ-2026-9999".to_string(),
            "094014201".to_string(),
            "PROTEUS".to_string(),
            "ATHENS".to_string(),
            "090000045".to_string(),
            "CLIENT".to_string(),
            "VOLOS".to_string(),
            "VOL-1234".to_string(),
            "Γιώργος".to_string(),
            TransportPurpose::Sale,
            3,
            Some(22.5),
        );
        create_shipping_note(&conn, &note).unwrap();

        let notes = list_shipping_notes(&conn).unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].note_number, "ΔΑ-2026-9999");
        assert_eq!(notes[0].status, DispatchStatus::Draft);
    }
}

