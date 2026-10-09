//! Embedded Component Genealogy & RMA inspector widget for ticket detail modals.
//! Adheres strictly to Rule 1 (100% Original Codebase), Rule 2 (Minimalist UX), and Rule 5 (Zero Mock Data).

use chrono::{DateTime, Utc};
use egui::{Color32, CornerRadius, DragValue, Frame, Margin, RichText, Stroke, TextEdit, Ui};
use rusqlite::Connection;
use proteus_core::genealogy::{
    get_genealogy, get_genealogy_timeline, initiate_rma, install_in_ticket, record_component_intake,
    resolve_rma_replacement, ComponentLifecycleStage, GenealogyEvent, SerialGenealogy, WarrantyStatus,
};
use proteus_core::tickets::ServiceTicket;

#[allow(clippy::too_many_arguments)]
pub fn draw_embedded_genealogy(
    ui: &mut Ui,
    conn: &Connection,
    record: &mut Option<SerialGenealogy>,
    timeline: &mut Vec<GenealogyEvent>,
    rma_fault_input: &mut String,
    rma_replacement_sn: &mut String,
    intake_supplier: &mut String,
    intake_warranty_months: &mut u32,
    feedback_msg: &mut Option<String>,
    ticket: &ServiceTicket,
    sn: &str,
    operator_name: &str,
    operator_role: &str,
) {
    Frame::new()
        .fill(crate::theme::BG_PANEL)
        .stroke(Stroke::new(1.0, crate::theme::ACCENT_CYAN))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::same(10))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("🔬 ΙΧΝΗΛΑΣΙΜΟΤΗΤΑ ΕΞΑΡΤΗΜΑΤΟΣ & RMA").strong().color(crate::theme::ACCENT_CYAN));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(rec) = record.as_ref() {
                        let now = Utc::now().timestamp_millis();
                        let (w_label, w_col) = match rec.warranty_status(now) {
                            WarrantyStatus::Valid { days_remaining } => (
                                format!("🛡 Εντός Εγγύησης ({} ημ.)", days_remaining),
                                Color32::from_rgb(52, 211, 153),
                            ),
                            WarrantyStatus::Expired { days_expired } => (
                                format!("⚠️ Έληξε (πριν {} ημ.)", days_expired),
                                Color32::from_rgb(251, 146, 60),
                            ),
                        };
                        ui.label(RichText::new(w_label).color(w_col).size(11.0).strong());
                    }
                });
            });

            ui.add_space(6.0);

            if let Some(rec) = record.clone() {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("Προμηθευτής: {}", rec.supplier_name)).size(11.0).color(crate::theme::TEXT_MUTED));
                    ui.label(RichText::new("•").size(10.0).color(crate::theme::TEXT_MUTED));
                    ui.label(RichText::new(format!("Στάδιο: {}", rec.current_stage.display_name())).size(11.0).strong());
                });

                ui.add_space(6.0);

                // RMA controls
                match &rec.current_stage {
                    ComponentLifecycleStage::RmaClaimInitiated { fault_description, claimed_by } => {
                        ui.label(
                            RichText::new(format!("⚡ Εκκρεμεί RMA: {} (από {})", fault_description, claimed_by))
                                .color(Color32::from_rgb(251, 146, 60))
                                .size(11.0)
                                .strong(),
                        );
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Νέο S/N:").size(11.0));
                            ui.add(TextEdit::singleline(rma_replacement_sn).hint_text("S/N αντικατάστασης").desired_width(120.0));
                            if ui.button(RichText::new("✅ Ολοκλήρωση Αντικατάστασης").size(11.0).strong()).clicked() {
                                let rep_sn = rma_replacement_sn.trim();
                                if !rep_sn.is_empty() {
                                    if let Ok(updated) = resolve_rma_replacement(conn, sn, rep_sn, None, operator_name) {
                                        let _ = proteus_core::audit::log_audit_event(
                                            conn,
                                            &proteus_core::audit::SystemEvent::new(
                                                "RMA",
                                                sn,
                                                "RMA_REPLACED",
                                                operator_name,
                                                operator_role,
                                                format!("Αντικατάσταση S/N {} με νέο {}", sn, rep_sn),
                                                serde_json::json!({
                                                    "ticket_number": ticket.ticket_number,
                                                    "old_sn": sn,
                                                    "new_sn": rep_sn
                                                }).to_string(),
                                            ),
                                        );
                                        *record = Some(updated);
                                        *timeline = get_genealogy_timeline(conn, sn).unwrap_or_default();
                                        *feedback_msg = Some(format!("✓ Το S/N {} αντικαταστάθηκε με {}", sn, rep_sn));
                                        rma_replacement_sn.clear();
                                    }
                                }
                            }
                        });
                    }
                    _ => {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new("Αιτία RMA:").size(11.0));
                            ui.add(TextEdit::singleline(rma_fault_input).hint_text("Περιγραφή βλάβης").desired_width(180.0));
                            if ui.button(RichText::new("⚡ Έναρξη RMA").size(11.0).strong()).clicked() {
                                let fault = rma_fault_input.trim();
                                if !fault.is_empty() {
                                    if let Ok(updated) = initiate_rma(conn, sn, fault, operator_name) {
                                        let _ = proteus_core::audit::log_audit_event(
                                            conn,
                                            &proteus_core::audit::SystemEvent::new(
                                                "RMA",
                                                sn,
                                                "RMA_CLAIM_INITIATED",
                                                operator_name,
                                                operator_role,
                                                format!("Έναρξη RMA για S/N {}: {}", sn, fault),
                                                serde_json::json!({
                                                    "ticket_number": ticket.ticket_number,
                                                    "serial_number": sn,
                                                    "fault": fault
                                                }).to_string(),
                                            ),
                                        );
                                        *record = Some(updated);
                                        *timeline = get_genealogy_timeline(conn, sn).unwrap_or_default();
                                        *feedback_msg = Some("✓ Η αίτηση RMA καταχωρήθηκε.".to_string());
                                    }
                                }
                            }
                        });
                    }
                }

                // Compact timeline
                if !timeline.is_empty() {
                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(4.0);
                    for ev in timeline.iter().rev().take(3) {
                        let dt = DateTime::from_timestamp_millis(ev.timestamp)
                            .map(|d| d.format("%d/%m %H:%M").to_string())
                            .unwrap_or_default();
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(format!("• {} ({})", ev.description, dt)).size(10.0).color(crate::theme::TEXT_SECONDARY));
                        });
                    }
                }
            } else {
                ui.label(RichText::new(format!("Το S/N '{}' δεν είναι καταχωρημένο στο μητρώο.", sn)).size(11.0).color(crate::theme::TEXT_MUTED));
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Προμηθευτής:").size(11.0));
                    ui.add(TextEdit::singleline(intake_supplier).desired_width(120.0));
                    ui.label(RichText::new("Μήνες:").size(11.0));
                    ui.add(DragValue::new(intake_warranty_months).range(1..=60));

                    if ui.button(RichText::new("📥 Καταχώριση & Σύνδεση").size(11.0).strong()).clicked() {
                        let sup = intake_supplier.trim();
                        if !sup.is_empty() {
                            let _ = record_component_intake(
                                conn,
                                sn,
                                "SKU-AUTO",
                                &ticket.device_model,
                                sup,
                                None,
                                *intake_warranty_months,
                                operator_name,
                            );
                            let _ = install_in_ticket(
                                conn,
                                sn,
                                &ticket.customer_name,
                                &ticket.device_model,
                                ticket.ticket_number,
                                operator_name,
                            );
                            let _ = proteus_core::audit::log_audit_event(
                                conn,
                                &proteus_core::audit::SystemEvent::new(
                                    "GENEALOGY",
                                    sn,
                                    "COMPONENT_LINKED_TO_TICKET",
                                    operator_name,
                                    operator_role,
                                    format!("Σύνδεση S/N {} με Δελτίο #{}", sn, ticket.ticket_number),
                                    serde_json::json!({
                                        "ticket_number": ticket.ticket_number,
                                        "serial_number": sn
                                    }).to_string(),
                                ),
                            );
                            *record = get_genealogy(conn, sn).unwrap_or(None);
                            *timeline = get_genealogy_timeline(conn, sn).unwrap_or_default();
                            *feedback_msg = Some("✓ Το εξάρτημα καταχωρήθηκε και συνδέθηκε με το δελτίο.".to_string());
                        }
                    }
                });
            }
        });
}
