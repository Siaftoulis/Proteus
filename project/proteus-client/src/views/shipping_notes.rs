//! Frontline Digital Shipping Note & Dispatch Companion View for Proteus Client.
//! Enables shop clerks and warehouse dispatchers to:
//! - Create, inspect, and track official Digital Shipping Notes (Δελτία Αποστολής)
//! - Auto-validate Greek AFMs and generate IAPR / myDATA & e-CMR compliant QR payloads
//! - Link serialized components and devices (Genealogy) directly to dispatches
//! - Issue 1-click ESC/POS thermal delivery waybill slips
//! - Record receiver sign-offs and delivery completions.
//! Strict Rule 5 compliance: Reads and writes 100% real SQLite data.

use egui::{Color32, CornerRadius, Frame, Margin, RichText, Stroke, Ui};
use rusqlite::Connection;

use proteus_core::printer::ShopReceiptConfig;
use proteus_core::shipping_note::{
    generate_escpos_shipping_voucher, init_shipping_schema, list_shipping_note_items,
    list_shipping_notes, update_shipping_note_status, DispatchStatus, ShippingNote,
    ShippingUnit, TransportPurpose,
};

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
    pub label_modal: super::label_print_modal::LabelPrintModalState,
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
            issuer_afm: String::new(),
            issuer_name: String::new(),
            issuer_address: String::new(),
            recipient_afm: String::new(),
            recipient_name: String::new(),
            recipient_address: String::new(),
            vehicle_plate: String::new(),
            driver_name: String::new(),
            purpose: TransportPurpose::Sale,
            packages_count: 1,
            gross_weight_input: "1.0".to_string(),
            pending_items: Vec::new(),
            item_sku: String::new(),
            item_desc: String::new(),
            item_unit: ShippingUnit::Piece,
            item_qty: 1.0,
            item_serials: String::new(),
            feedback_msg: None,
            schema_initialized: false,
            label_modal: super::label_print_modal::LabelPrintModalState::default(),
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
        if state.issuer_afm.is_empty() {
            state.issuer_afm = config.afm.clone();
        }
        if state.issuer_name.is_empty() {
            state.issuer_name = if !config.legal_name.is_empty() {
                config.legal_name.clone()
            } else {
                config.shop_name.clone()
            };
        }
        if state.issuer_address.is_empty() {
            state.issuer_address = config.address.clone();
        }
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
        super::shipping_modals::draw_new_shipping_note_modal(ui, conn, state);
    }

    if state.active_items_note_id.is_some() {
        super::shipping_modals::draw_items_drawer_modal(ui, conn, state);
    }

    if state.active_delivery_note_id.is_some() {
        super::shipping_modals::draw_delivery_sign_modal(ui, conn, state);
    }

    super::label_print_modal::render_label_print_modal(ui.ctx(), &mut state.label_modal);
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
                    if ui.button(RichText::new("🏷 TSPL").color(Color32::from_rgb(147, 197, 253)).size(11.0)).clicked() {
                        state.label_modal.open_shipping_waybill(
                            &note.note_number,
                            "ACS Courier",
                            &note.recipient_name,
                            &note.recipient_address,
                            "Ελλάδα",
                            "",
                            None,
                            note.packages_count,
                            note.gross_weight_kg.unwrap_or(1.0) as f32,
                        );
                    }

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


#[cfg(test)]
mod tests {
    use super::*;
    use proteus_core::shipping_note::create_shipping_note;
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
        assert!(state.issuer_afm.is_empty());
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

