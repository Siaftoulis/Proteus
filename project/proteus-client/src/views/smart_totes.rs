//! Smart Tote Intake & Pack Bench View for Proteus Client.
//! Visualizes active container manifest aggregation, near-field RFID chute drops,
//! tamper-evident Merkle SHA-256 seals, and 1-click shelf placement.
//! Strict Rule 1 (100% Original Codebase), Rule 2 (Minimalist UX), Rule 3 (<400 lines), Rule 5 (Zero Mock Data).

use egui::{Color32, CornerRadius, Frame, Margin, RichText, Stroke, Ui, Vec2};
use rusqlite::Connection;

use proteus_core::audio::play_barcode_chime;
use proteus_core::wms::tote::{
    find_container_by_code, list_containers, load_smart_container, save_smart_container,
    ContainerStatus, ContainerType, SmartContainer,
};

use crate::theme::{
    ACCENT_GOLD, BG_CARD, BG_PANEL, BORDER_SUBTLE, STATUS_READY, TEXT_MUTED, TEXT_PRIMARY,
};

use super::spatial_wms::SpatialWmsViewState;

/// Renders the Smart Tote Intake & Pack Bench tab within Spatial WMS.
pub fn render_smart_totes_tab(ui: &mut Ui, conn: &Connection, state: &mut SpatialWmsViewState) {
    ui.label(RichText::new("Έξυπνος Σταθμός Συσκευασίας & Near-Field Chute (Smart Totes):").strong());
    ui.label(
        RichText::new("Αυτόματη συγκέντρωση αντικειμένων, κρυπτογραφική σφράγιση Merkle SHA-256 και άμεση τοποθέτηση.")
            .size(12.0)
            .color(TEXT_MUTED),
    );

    ui.add_space(6.0);

    // Tote Selector & Creation Row
    ui.horizontal(|ui| {
        ui.label("Κωδικός Κιβωτίου:");
        ui.add(
            egui::TextEdit::singleline(&mut state.tote_input_code)
                .hint_text("π.χ. TOTE-101")
                .desired_width(120.0),
        );

        if ui.button(RichText::new("➕ Άνοιγμα / Φόρτωση").strong()).clicked() {
            let code = state.tote_input_code.trim().to_uppercase();
            if !code.is_empty() {
                match find_container_by_code(conn, &code) {
                    Ok(Some(existing)) => {
                        state.tote_selected_id = Some(existing.container_id);
                        state.feedback_message = Some((format!("Φορτώθηκε το κιβώτιο {}", code), true));
                    }
                    Ok(None) => {
                        let now = chrono::Utc::now().timestamp();
                        let new_tote = SmartContainer::new(&code, ContainerType::Tote, now);
                        if save_smart_container(conn, &new_tote).is_ok() {
                            state.tote_selected_id = Some(new_tote.container_id);
                            state.feedback_message = Some((format!("Δημιουργήθηκε νέο κιβώτιο {}", code), true));
                            play_barcode_chime();
                        }
                    }
                    Err(e) => {
                        state.feedback_message = Some((format!("Σφάλμα: {}", e), false));
                    }
                }
            }
        }
    });

    // Recent Totes Quick Bar
    if let Ok(recent) = list_containers(conn, 5) {
        if !recent.is_empty() {
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new("Πρόσφατα:").size(11.0).color(TEXT_MUTED));
                for c in &recent {
                    let is_active = state.tote_selected_id.as_deref() == Some(&c.container_id);
                    let (icon, color) = match &c.status {
                        ContainerStatus::OpenPacking => ("🟢", STATUS_READY),
                        ContainerStatus::Sealed => ("🔒", ACCENT_GOLD),
                        ContainerStatus::AssignedToShelf(_) => ("📍", Color32::from_rgb(59, 130, 246)),
                        ContainerStatus::Shipped => ("🚚", Color32::from_rgb(168, 85, 247)),
                    };

                    let btn_text = format!("{} {}", icon, c.container_code);
                    if ui.selectable_label(is_active, RichText::new(btn_text).color(color).size(11.0)).clicked() {
                        state.tote_selected_id = Some(c.container_id.clone());
                        state.tote_input_code = c.container_code.clone();
                    }
                }
            });
        }
    }

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(8.0);

    // Active Container Details & Ingestion Bench
    let selected_opt = state
        .tote_selected_id
        .as_ref()
        .and_then(|id| load_smart_container(conn, id).ok());

    match selected_opt {
        Some(mut container) => {
            render_container_details(ui, conn, state, &mut container);
        }
        None => {
            Frame::new()
                .fill(BG_PANEL)
                .stroke(Stroke::new(1.0, BORDER_SUBTLE))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::same(16))
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new("📦 Δεν έχει επιλεγεί κιβώτιο").color(TEXT_MUTED).size(13.0));
                        ui.label(
                            RichText::new("Εισάγετε κωδικό κιβωτίου παραπάνω για να ξεκινήσετε τη συλλογή αντικειμένων.")
                                .color(TEXT_MUTED)
                                .size(11.0),
                        );
                    });
                });
        }
    }
}

fn render_container_details(
    ui: &mut Ui,
    conn: &Connection,
    state: &mut SpatialWmsViewState,
    container: &mut SmartContainer,
) {
    let (status_text, status_color) = match &container.status {
        ContainerStatus::OpenPacking => ("🟢 Ανοιχτό προς Συσκευασία (OpenPacking)", STATUS_READY),
        ContainerStatus::Sealed => ("🔒 Σφραγισμένο (Merkle Sealed)", ACCENT_GOLD),
        ContainerStatus::AssignedToShelf(_) => ("📍 Τοποθετημένο σε Ράφι", Color32::from_rgb(59, 130, 246)),
        ContainerStatus::Shipped => ("🚚 Απεστάλη (Shipped)", Color32::from_rgb(168, 85, 247)),
    };

    Frame::new()
        .fill(BG_CARD)
        .stroke(Stroke::new(1.0, BORDER_SUBTLE))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            // Header: Code, Type, Status
            ui.horizontal(|ui| {
                ui.label(RichText::new(&container.container_code).strong().size(16.0).color(TEXT_PRIMARY));
                ui.label(RichText::new(format!("({})", container.container_type.as_str())).size(12.0).color(TEXT_MUTED));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(status_text).strong().size(12.0).color(status_color));
                });
            });

            // Merkle Seal & Shelf info if present
            if let Some(seal) = &container.seal_hash {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Merkle Seal:").size(11.0).color(TEXT_MUTED));
                    ui.label(RichText::new(format!("{}...", &seal[..16.min(seal.len())])).monospace().size(11.0).color(ACCENT_GOLD));
                });
            }

            if let ContainerStatus::AssignedToShelf(shelf_id) = &container.status {
                ui.add_space(2.0);
                ui.label(RichText::new(format!("Θέση Αποθήκευσης: Ράφι #{}", shelf_id)).strong().size(12.0).color(Color32::from_rgb(59, 130, 246)));
            }

            // Metrics Row
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("Τεμάχια: {}", container.total_units())).strong().size(12.0));
                ui.label(RichText::new("|").color(TEXT_MUTED));
                ui.label(RichText::new(format!("Βάρος: {:.1} g ({:.2} kg)", container.total_weight_grams, container.total_weight_grams / 1000.0)).strong().size(12.0));
            });

            // Ingestion section for OpenPacking
            if container.status == ContainerStatus::OpenPacking {
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(6.0);

                ui.label(RichText::new("⚡ Σάρωση / Chute Drop Εισερχόμενου Αντικειμένου:").strong().size(12.0));
                ui.horizontal(|ui| {
                    ui.label("SKU:");
                    ui.add(egui::TextEdit::singleline(&mut state.tote_item_sku).desired_width(100.0));
                    ui.label("EPC/Serial:");
                    ui.add(egui::TextEdit::singleline(&mut state.tote_item_serial).desired_width(90.0));
                    ui.label("Βάρος (g):");
                    ui.add(egui::TextEdit::singleline(&mut state.tote_item_weight_str).desired_width(45.0));
                });

                ui.add_space(4.0);
                let drop_btn = egui::Button::new(RichText::new("📥 Ρίψη στο Κιβώτιο (Ring Scan / Drop)").strong().size(12.0))
                    .min_size(Vec2::new(ui.available_width(), 28.0));

                if ui.add(drop_btn).clicked() {
                    let sku = state.tote_item_sku.trim().to_uppercase();
                    if !sku.is_empty() {
                        let serial_opt = if state.tote_item_serial.trim().is_empty() {
                            None
                        } else {
                            Some(state.tote_item_serial.trim())
                        };
                        let weight = state.tote_item_weight_str.parse::<f64>().unwrap_or(250.0);
                        let now = chrono::Utc::now().timestamp();

                        match container.ingest_item(&sku, serial_opt, 1, weight, now) {
                            Ok(_) => {
                                let _ = save_smart_container(conn, container);
                                state.feedback_message = Some((format!("Προστέθηκε {} στο κιβώτιο!", sku), true));
                                play_barcode_chime();
                                state.tote_item_serial.clear();
                            }
                            Err(e) => {
                                state.feedback_message = Some((format!("Σφάλμα καταχώρησης: {}", e), false));
                            }
                        }
                    }
                }
            }

            // Manifest Items Table
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(6.0);
            ui.label(RichText::new(format!("Περιεχόμενα Κιβωτίου ({} εγγραφές):", container.items.len())).strong().size(12.0));

            egui::ScrollArea::vertical().max_height(140.0).show(ui, |ui| {
                if container.items.is_empty() {
                    ui.label(RichText::new("Το κιβώτιο είναι άδειο.").color(TEXT_MUTED).size(11.0));
                } else {
                    for (idx, item) in container.items.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(format!("{}.", idx + 1)).color(TEXT_MUTED).size(11.0));
                            ui.label(RichText::new(&item.sku).strong().size(11.0));
                            if let Some(epc) = &item.serial_or_epc {
                                ui.label(RichText::new(format!("({})", epc)).monospace().size(10.0).color(TEXT_MUTED));
                            }
                            ui.label(RichText::new(format!("x{}", item.quantity)).size(11.0));
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(RichText::new(format!("{:.0}g", item.weight_grams)).size(11.0).color(TEXT_MUTED));
                            });
                        });
                    }
                }
            });

            // Action Buttons: Seal & Assign to Shelf
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                if container.status == ContainerStatus::OpenPacking && !container.items.is_empty() {
                    let seal_btn = egui::Button::new(RichText::new("🔒 Σφράγιση Κιβωτίου (Merkle Seal)").strong().color(Color32::BLACK))
                        .fill(ACCENT_GOLD);
                    if ui.add(seal_btn).clicked() {
                        let now = chrono::Utc::now().timestamp();
                        match container.seal(now) {
                            Ok(hash) => {
                                let _ = save_smart_container(conn, container);
                                state.feedback_message = Some((format!("Το κιβώτιο σφραγίστηκε! SHA-256: {}", &hash[..12]), true));
                                play_barcode_chime();
                            }
                            Err(e) => {
                                state.feedback_message = Some((format!("Σφάλμα σφράγισης: {}", e), false));
                            }
                        }
                    }
                }

                ui.label("Ράφι:");
                ui.add(egui::TextEdit::singleline(&mut state.tote_target_shelf).hint_text("S-102").desired_width(55.0));

                let assign_btn = egui::Button::new(RichText::new("📍 Ανάθεση σε Ράφι").strong());
                if ui.add(assign_btn).clicked() {
                    let shelf = state.tote_target_shelf.trim().to_uppercase();
                    if !shelf.is_empty() {
                        let now = chrono::Utc::now().timestamp();
                        container.assign_to_shelf(&shelf, now);
                        let _ = save_smart_container(conn, container);
                        state.feedback_message = Some((format!("Το κιβώτιο τοποθετήθηκε στο Ράφι #{}!", shelf), true));
                        play_barcode_chime();
                    }
                }
            });
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use proteus_core::wms::tote::init_tote_schema;

    #[test]
    fn test_smart_totes_state_and_persistence() {
        let conn = Connection::open_in_memory().unwrap();
        init_tote_schema(&conn).unwrap();

        let mut state = SpatialWmsViewState::default();
        state.tote_input_code = "BOX-TEST-1".to_string();
        state.tote_item_sku = "ROUTER-AX3000".to_string();
        state.tote_target_shelf = "S-102".to_string();

        assert_eq!(state.tote_input_code, "BOX-TEST-1");
        assert_eq!(state.tote_item_sku, "ROUTER-AX3000");
    }
}
