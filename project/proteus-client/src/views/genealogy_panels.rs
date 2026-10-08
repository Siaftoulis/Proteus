//! Companion Panels & Intake Drawers for Component Genealogy & RMA Hub.
//! Strict Rule 1 (100% Original Codebase), Rule 2 (Minimalist UX), Rule 5 (Zero Mock Data).

use egui::{Color32, CornerRadius, Frame, Margin, RichText, Stroke, Ui};
use rusqlite::Connection;

use proteus_core::genealogy::{
    get_genealogy_timeline, record_component_intake, ComponentLifecycleStage,
};

use super::genealogy_rma::GenealogyRmaState;

pub fn draw_active_rma_queue(
    ui: &mut Ui,
    conn: &Connection,
    state: &mut GenealogyRmaState,
    _operator_name: &str,
    _operator_role: &str,
) {
    Frame::new()
        .fill(crate::theme::BG_CARD)
        .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::same(14))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("ΕΚΚΡΕΜΕΙΣ ΑΙΤΗΣΕΙΣ RMA ΠΡΟΜΗΘΕΥΤΩΝ").strong().color(crate::theme::TEXT_MUTED));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new(format!("{} Ενεργές", state.active_claims.len())).size(11.0).color(Color32::from_rgb(251, 146, 60)));
                });
            });

            ui.add_space(8.0);

            if state.active_claims.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);
                    ui.label(RichText::new("✓").size(24.0).color(Color32::from_rgb(52, 211, 153)));
                    ui.label(RichText::new("Δεν υπάρχουν εκκρεμείς αιτήσεις RMA.").size(12.0).color(crate::theme::TEXT_MUTED));
                    ui.add_space(20.0);
                });
            } else {
                for claim in state.active_claims.clone() {
                    Frame::new()
                        .fill(crate::theme::BG_PANEL)
                        .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
                        .corner_radius(CornerRadius::same(6))
                        .inner_margin(Margin::same(10))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(&claim.name).strong().color(crate::theme::TEXT_PRIMARY));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.small_button("🔍 Προβολή").clicked() {
                                        state.search_query = claim.serial_number.clone();
                                        state.selected_genealogy = Some(claim.clone());
                                        state.timeline_events = get_genealogy_timeline(conn, &claim.serial_number).unwrap_or_default();
                                    }
                                });
                            });

                            ui.horizontal(|ui| {
                                ui.label(RichText::new(format!("S/N: {}", claim.serial_number)).size(11.0).color(crate::theme::ACCENT_CYAN));
                                ui.label(RichText::new("•").size(10.0).color(crate::theme::TEXT_MUTED));
                                ui.label(RichText::new(format!("Προμηθευτής: {}", claim.supplier_name)).size(11.0).color(crate::theme::TEXT_SECONDARY));
                            });

                            if let ComponentLifecycleStage::RmaClaimInitiated { fault_description, claimed_by } = &claim.current_stage {
                                ui.add_space(2.0);
                                ui.label(RichText::new(format!("Βλάβη: {}", fault_description)).size(11.0).italics());
                                ui.label(RichText::new(format!("Καταχωρήθηκε από: {}", claimed_by)).size(10.0).color(crate::theme::TEXT_MUTED));
                            }
                        });
                    ui.add_space(6.0);
                }
            }
        });
}

pub fn draw_intake_drawer(
    ui: &mut Ui,
    conn: &Connection,
    state: &mut GenealogyRmaState,
    operator_name: &str,
    operator_role: &str,
) {
    Frame::new()
        .fill(crate::theme::BG_PANEL)
        .stroke(Stroke::new(1.0, crate::theme::ACCENT_CYAN))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::same(14))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("📥 ΚΑΤΑΧΩΡΙΣΗ ΝΕΟΥ ΣΕΙΡΙΑΚΟΥ ΕΞΑΡΤΗΜΑΤΟΣ (SUPPLIER INTAKE)").strong().color(crate::theme::ACCENT_CYAN));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("✖ Κλείσιμο").clicked() {
                        state.show_intake_drawer = false;
                    }
                });
            });

            ui.add_space(8.0);

            egui::Grid::new("intake_form_grid")
                .num_columns(2)
                .spacing([14.0, 6.0])
                .show(ui, |ui| {
                    ui.label(RichText::new("Σειριακός Αριθμός (S/N):*").size(12.0));
                    ui.add(egui::TextEdit::singleline(&mut state.intake_sn).hint_text("π.χ. SN-499102"));
                    ui.end_row();

                    ui.label(RichText::new("Κωδικός SKU:*").size(12.0));
                    ui.add(egui::TextEdit::singleline(&mut state.intake_sku).hint_text("π.x. DISP-OLED-IP15"));
                    ui.end_row();

                    ui.label(RichText::new("Περιγραφή Είδους:*").size(12.0));
                    ui.add(egui::TextEdit::singleline(&mut state.intake_name).hint_text("π.χ. Οθόνη Super Retina XDR"));
                    ui.end_row();

                    ui.label(RichText::new("Προμηθευτής:*").size(12.0));
                    ui.add(egui::TextEdit::singleline(&mut state.intake_supplier).hint_text("π.χ. Westnet / Quest / CPI"));
                    ui.end_row();

                    ui.label(RichText::new("Παραστατικό Αγοράς:").size(12.0));
                    ui.add(egui::TextEdit::singleline(&mut state.intake_invoice).hint_text("π.χ. ΤΙΜ-2026-0819"));
                    ui.end_row();
                });

            ui.add_space(8.0);

            if ui.button(RichText::new("📥 Αποθήκευση Παραλαβής στη Βάση").strong()).clicked() {
                let sn = state.intake_sn.trim();
                let sku = state.intake_sku.trim();
                let name = state.intake_name.trim();
                let supplier = state.intake_supplier.trim();
                let inv = state.intake_invoice.trim();
                let inv_opt = if inv.is_empty() { None } else { Some(inv) };

                if !sn.is_empty() && !sku.is_empty() && !name.is_empty() && !supplier.is_empty() {
                    match record_component_intake(
                        conn,
                        sn,
                        sku,
                        name,
                        supplier,
                        inv_opt,
                        state.intake_warranty_months,
                        operator_name,
                    ) {
                        Ok(g) => {
                            let _ = proteus_core::audit::log_audit_event(
                                conn,
                                &proteus_core::audit::SystemEvent::new(
                                    "GENEALOGY",
                                    sn,
                                    "COMPONENT_INTAKE",
                                    operator_name,
                                    operator_role,
                                    format!("Παραλαβή S/N {} ({}) από {}", sn, name, supplier),
                                    serde_json::json!({
                                        "serial_number": sn,
                                        "sku": sku,
                                        "supplier": supplier,
                                    }).to_string(),
                                ),
                            );
                            state.selected_genealogy = Some(g);
                            state.timeline_events = get_genealogy_timeline(conn, sn).unwrap_or_default();
                            state.feedback_msg = Some(format!("✓ Το εξάρτημα με S/N '{}' καταχωρήθηκε επιτυχώς.", sn));
                            state.intake_sn.clear();
                            state.intake_sku.clear();
                            state.intake_name.clear();
                            state.intake_supplier.clear();
                            state.intake_invoice.clear();
                            state.show_intake_drawer = false;
                        }
                        Err(e) => {
                            state.feedback_msg = Some(format!("Σφάλμα καταχώρισης: {}", e));
                        }
                    }
                } else {
                    state.feedback_msg = Some("Παρακαλώ συμπληρώστε όλα τα υποχρεωτικά πεδία.".to_string());
                }
            }
        });
}
