//! Serial Number Tracking, Component Genealogy & RMA Hub for Proteus BOS.
//! Manages component lifecycle from Supplier Intake -> Warehouse -> Customer Ticket -> RMA Claim -> Supplier Replacement.
//! Adheres strictly to Rule 1 (100% Original Codebase), Rule 2 (Minimalist UX), and Rule 5 (Zero Mock Data).

use chrono::{DateTime, Utc};
use egui::{Color32, CornerRadius, Frame, Margin, RichText, Stroke, Ui};
use rusqlite::Connection;
use proteus_core::genealogy::{
    get_genealogy, get_genealogy_timeline, initiate_rma, list_active_rma_claims,
    record_component_intake, resolve_rma_replacement, ComponentLifecycleStage,
    GenealogyEvent, SerialGenealogy, WarrantyStatus,
};

pub struct GenealogyRmaState {
    pub search_query: String,
    pub active_claims: Vec<SerialGenealogy>,
    pub selected_genealogy: Option<SerialGenealogy>,
    pub timeline_events: Vec<GenealogyEvent>,
    pub feedback_msg: Option<String>,

    // RMA Resolution form inputs
    pub replacement_sn_input: String,
    pub credit_invoice_input: String,

    // RMA Initiation form inputs
    pub rma_serial_target: Option<String>,
    pub rma_fault_input: String,

    // New component intake form
    pub intake_sn: String,
    pub intake_sku: String,
    pub intake_name: String,
    pub intake_supplier: String,
    pub intake_invoice: String,
    pub intake_warranty_months: u32,
    pub show_intake_drawer: bool,
}

impl Default for GenealogyRmaState {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            active_claims: Vec::new(),
            selected_genealogy: None,
            timeline_events: Vec::new(),
            feedback_msg: None,
            replacement_sn_input: String::new(),
            credit_invoice_input: String::new(),
            rma_serial_target: None,
            rma_fault_input: String::new(),
            intake_sn: String::new(),
            intake_sku: String::new(),
            intake_name: String::new(),
            intake_supplier: String::new(),
            intake_invoice: String::new(),
            intake_warranty_months: 24,
            show_intake_drawer: false,
        }
    }
}

pub fn draw_genealogy_rma_view(
    ui: &mut Ui,
    conn: &Connection,
    state: &mut GenealogyRmaState,
    operator_name: &str,
    operator_role: &str,
) {
    // Refresh active claims
    if state.active_claims.is_empty() {
        if let Ok(claims) = list_active_rma_claims(conn) {
            state.active_claims = claims;
        }
    }

    ui.vertical(|ui| {
        // Header
        ui.horizontal(|ui| {
            ui.heading(RichText::new("🔬 Ιχνηλασιμότητα S/N & Διαχείριση Εγγυήσεων RMA").strong().size(22.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(RichText::new("📥 Νέα Παραλαβή S/N").strong()).clicked() {
                    state.show_intake_drawer = !state.show_intake_drawer;
                }
            });
        });

        ui.add_space(4.0);
        ui.label(
            RichText::new("Πλήρες ιστορικό ζωής εξαρτημάτων: Προμηθευτής ➔ Αποθήκη ➔ Δελτίο Πελάτη ➔ Αίτηση RMA ➔ Αντικατάσταση.")
                .size(12.0)
                .color(crate::theme::TEXT_MUTED),
        );
        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        // Feedback notification
        if let Some(msg) = &state.feedback_msg {
            Frame::new()
                .fill(Color32::from_rgb(16, 50, 35))
                .stroke(Stroke::new(1.0, Color32::from_rgb(52, 211, 153)))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::symmetric(14, 8))
                .show(ui, |ui| {
                    ui.label(RichText::new(msg).color(Color32::from_rgb(52, 211, 153)).strong());
                });
            ui.add_space(8.0);
        }

        // Intake Drawer (if toggled)
        if state.show_intake_drawer {
            draw_intake_drawer(ui, conn, state, operator_name, operator_role);
            ui.add_space(10.0);
        }

        // Top Section: Search & Quick Lookup
        ui.horizontal(|ui| {
            ui.label(RichText::new("Αναζήτηση S/N:").strong());
            let res = ui.add(
                egui::TextEdit::singleline(&mut state.search_query)
                    .hint_text("Εισάγετε σειριακό αριθμό (π.χ. SN-998822)")
                    .desired_width(260.0),
            );

            if ui.button(RichText::new("🔍 Έλεγχος Ιστορικού").strong()).clicked() || (res.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                let trimmed = state.search_query.trim();
                if !trimmed.is_empty() {
                    match get_genealogy(conn, trimmed) {
                        Ok(Some(g)) => {
                            state.selected_genealogy = Some(g);
                            state.timeline_events = get_genealogy_timeline(conn, trimmed).unwrap_or_default();
                            state.feedback_msg = None;
                        }
                        Ok(None) => {
                            state.selected_genealogy = None;
                            state.timeline_events.clear();
                            state.feedback_msg = Some(format!("Δεν βρέθηκε εξάρτημα με S/N '{}'", trimmed));
                        }
                        Err(e) => {
                            state.feedback_msg = Some(format!("Σφάλμα βάσης: {}", e));
                        }
                    }
                }
            }

            if state.selected_genealogy.is_some() && ui.button("✖ Καθαρισμός").clicked() {
                state.selected_genealogy = None;
                state.timeline_events.clear();
                state.search_query.clear();
            }
        });

        ui.add_space(10.0);

        // Main Body: 2 Columns (Left: Inspector / Timeline, Right: Active RMA Queue)
        ui.columns(2, |cols| {
            // Left Column: Selected Component Details & Timeline
            cols[0].vertical(|ui| {
                if let Some(item) = &state.selected_genealogy {
                    draw_component_inspector(ui, conn, state, item.clone(), operator_name, operator_role);
                } else {
                    Frame::new()
                        .fill(crate::theme::BG_CARD)
                        .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
                        .corner_radius(CornerRadius::same(8))
                        .inner_margin(Margin::same(18))
                        .show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.add_space(20.0);
                                ui.label(RichText::new("🔍").size(32.0));
                                ui.add_space(8.0);
                                ui.label(RichText::new("Επιλέξτε ή αναζητήστε σειριακό αριθμό (S/N)").strong().size(14.0));
                                ui.label(RichText::new("για να δείτε την πλήρη γενεαλογία και κατάσταση εγγύησης.").color(crate::theme::TEXT_MUTED));
                                ui.add_space(20.0);
                            });
                        });
                }
            });

            // Right Column: Active RMA Claims Queue
            cols[1].vertical(|ui| {
                draw_active_rma_queue(ui, conn, state, operator_name, operator_role);
            });
        });
    });
}

fn draw_component_inspector(
    ui: &mut Ui,
    conn: &Connection,
    state: &mut GenealogyRmaState,
    item: SerialGenealogy,
    operator_name: &str,
    operator_role: &str,
) {
    Frame::new()
        .fill(crate::theme::BG_CARD)
        .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::same(14))
        .show(ui, |ui| {
            // Header: Name & S/N
            ui.horizontal(|ui| {
                ui.label(RichText::new(&item.name).strong().size(16.0));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let now = Utc::now().timestamp_millis();
                    let warranty = item.warranty_status(now);
                    let (w_label, w_col) = match warranty {
                        WarrantyStatus::Valid { days_remaining } => (
                            format!("🛡 Εντός Εγγύησης ({} ημ.)", days_remaining),
                            Color32::from_rgb(52, 211, 153),
                        ),
                        WarrantyStatus::Expired { days_expired } => (
                            format!("⚠️ Έληξε η Εγγύηση (πριν {} ημ.)", days_expired),
                            Color32::from_rgb(251, 146, 60),
                        ),
                    };

                    Frame::new()
                        .fill(w_col.linear_multiply(0.2))
                        .stroke(Stroke::new(1.0, w_col))
                        .corner_radius(CornerRadius::same(4))
                        .inner_margin(Margin::symmetric(8, 3))
                        .show(ui, |ui| {
                            ui.label(RichText::new(w_label).color(w_col).size(11.0).strong());
                        });
                });
            });

            ui.add_space(6.0);

            // Metadata Grid
            egui::Grid::new("genealogy_meta_grid")
                .num_columns(2)
                .spacing([16.0, 6.0])
                .show(ui, |ui| {
                    ui.label(RichText::new("S/N:").color(crate::theme::TEXT_MUTED));
                    ui.label(RichText::new(&item.serial_number).strong().color(crate::theme::ACCENT_CYAN));
                    ui.end_row();

                    ui.label(RichText::new("SKU:").color(crate::theme::TEXT_MUTED));
                    ui.label(RichText::new(&item.sku));
                    ui.end_row();

                    ui.label(RichText::new("Προμηθευτής:").color(crate::theme::TEXT_MUTED));
                    ui.label(RichText::new(&item.supplier_name).strong());
                    ui.end_row();

                    let intake_dt = DateTime::from_timestamp_millis(item.intake_date)
                        .map(|d| d.format("%d/%m/%Y").to_string())
                        .unwrap_or_else(|| "N/A".to_string());
                    ui.label(RichText::new("Ημ/νία Εισαγωγής:").color(crate::theme::TEXT_MUTED));
                    ui.label(RichText::new(intake_dt));
                    ui.end_row();

                    ui.label(RichText::new("Τρέχον Στάδιο:").color(crate::theme::TEXT_MUTED));
                    ui.label(RichText::new(item.current_stage.display_name()).strong());
                    ui.end_row();
                });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // RMA Action Buttons
            match &item.current_stage {
                ComponentLifecycleStage::RmaClaimInitiated { fault_description, claimed_by } => {
                    Frame::new()
                        .fill(Color32::from_rgb(45, 25, 20))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(251, 146, 60)))
                        .corner_radius(CornerRadius::same(6))
                        .inner_margin(Margin::same(10))
                        .show(ui, |ui| {
                            ui.label(RichText::new("⚡ ΕΚΚΡΕΜΕΙ ΑΙΤΗΣΗ RMA ΣΤΟΝ ΠΡΟΜΗΘΕΥΤΗ").strong().color(Color32::from_rgb(251, 146, 60)));
                            ui.label(RichText::new(format!("Βλάβη: {}", fault_description)).size(11.0));
                            ui.label(RichText::new(format!("Από: {}", claimed_by)).size(11.0).color(crate::theme::TEXT_MUTED));
                            ui.add_space(6.0);

                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Νέο S/N:").size(11.0));
                                ui.add(egui::TextEdit::singleline(&mut state.replacement_sn_input).hint_text("S/N νέου εξαρτήματος").desired_width(130.0));
                                ui.label(RichText::new("Πιστωτικό:").size(11.0));
                                ui.add(egui::TextEdit::singleline(&mut state.credit_invoice_input).hint_text("π.χ. ΠΙΣΤ-2026").desired_width(90.0));
                                if ui.button(RichText::new("✅ Ολοκλήρωση").strong()).clicked() {
                                    let rep_sn = state.replacement_sn_input.trim();
                                    let cred = state.credit_invoice_input.trim();
                                    let cred_opt = if cred.is_empty() { None } else { Some(cred) };
                                    if !rep_sn.is_empty() {
                                        match resolve_rma_replacement(conn, &item.serial_number, rep_sn, cred_opt, operator_name) {
                                            Ok(updated) => {
                                                let _ = proteus_core::audit::log_audit_event(
                                                    conn,
                                                    &proteus_core::audit::SystemEvent::new(
                                                        "RMA",
                                                        &item.serial_number,
                                                        "RMA_REPLACED",
                                                        operator_name,
                                                        operator_role,
                                                        format!("Αντικατάσταση S/N {} με νέο {}", item.serial_number, rep_sn),
                                                        serde_json::json!({
                                                            "original_sn": item.serial_number,
                                                            "replacement_sn": rep_sn,
                                                        }).to_string(),
                                                    ),
                                                );
                                                state.selected_genealogy = Some(updated);
                                                state.timeline_events = get_genealogy_timeline(conn, &item.serial_number).unwrap_or_default();
                                                state.active_claims = list_active_rma_claims(conn).unwrap_or_default();
                                                state.feedback_msg = Some(format!("✓ Η αντικατάσταση ολοκληρώθηκε με νέο S/N '{}'.", rep_sn));
                                                state.replacement_sn_input.clear();
                                            }
                                            Err(e) => {
                                                state.feedback_msg = Some(format!("Σφάλμα αντικατάστασης: {}", e));
                                            }
                                        }
                                    }
                                }
                            });
                        });
                }
                _ => {
                    ui.horizontal(|ui| {
                        if ui.button(RichText::new("⚡ Έναρξη Αίτησης RMA").strong()).clicked() {
                            state.rma_serial_target = Some(item.serial_number.clone());
                        }
                    });

                    if state.rma_serial_target.as_deref() == Some(&item.serial_number) {
                        ui.add_space(6.0);
                        Frame::new()
                            .fill(crate::theme::BG_PANEL)
                            .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
                            .corner_radius(CornerRadius::same(6))
                            .inner_margin(Margin::same(10))
                            .show(ui, |ui| {
                                ui.label(RichText::new("Περιγραφή Ελαττώματος / Βλάβης:").strong().size(12.0));
                                ui.add(egui::TextEdit::multiline(&mut state.rma_fault_input).desired_rows(2).desired_width(f32::INFINITY));
                                ui.add_space(6.0);
                                ui.horizontal(|ui| {
                                    if ui.button(RichText::new("📤 Υποβολή στον Προμηθευτή").strong()).clicked() {
                                        let fault = state.rma_fault_input.trim();
                                        if !fault.is_empty() {
                                            match initiate_rma(conn, &item.serial_number, fault, operator_name) {
                                                Ok(updated) => {
                                                    let _ = proteus_core::audit::log_audit_event(
                                                        conn,
                                                        &proteus_core::audit::SystemEvent::new(
                                                            "RMA",
                                                            &item.serial_number,
                                                            "RMA_CLAIM_INITIATED",
                                                            operator_name,
                                                            operator_role,
                                                            format!("Έναρξη RMA για S/N {}: {}", item.serial_number, fault),
                                                            serde_json::json!({
                                                                "serial_number": item.serial_number,
                                                                "fault": fault,
                                                            }).to_string(),
                                                        ),
                                                    );
                                                    state.selected_genealogy = Some(updated);
                                                    state.timeline_events = get_genealogy_timeline(conn, &item.serial_number).unwrap_or_default();
                                                    state.active_claims = list_active_rma_claims(conn).unwrap_or_default();
                                                    state.feedback_msg = Some("✓ Η αίτηση RMA καταχωρήθηκε επιτυχώς.".to_string());
                                                    state.rma_serial_target = None;
                                                    state.rma_fault_input.clear();
                                                }
                                                Err(e) => {
                                                    state.feedback_msg = Some(format!("Σφάλμα RMA: {}", e));
                                                }
                                            }
                                        }
                                    }
                                    if ui.button("Ακύρωση").clicked() {
                                        state.rma_serial_target = None;
                                    }
                                });
                            });
                    }
                }
            }

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(8.0);

            // Timeline View
            ui.label(RichText::new("ΧΡΟΝΟΛΟΓΙΚΟ ΙΣΤΟΡΙΚΟ ΓΕΝΕΑΛΟΓΙΑΣ").strong().size(12.0).color(crate::theme::TEXT_MUTED));
            ui.add_space(6.0);

            if state.timeline_events.is_empty() {
                ui.label(RichText::new("Δεν υπάρχουν καταγεγραμμένα συμβάντα.").color(crate::theme::TEXT_MUTED).size(12.0));
            } else {
                for ev in &state.timeline_events {
                    ui.horizontal(|ui| {
                        let dt = DateTime::from_timestamp_millis(ev.timestamp)
                            .map(|d| d.format("%d/%m/%Y %H:%M").to_string())
                            .unwrap_or_else(|| "N/A".to_string());
                        ui.label(RichText::new("●").color(crate::theme::ACCENT_CYAN).size(10.0));
                        ui.label(RichText::new(dt).size(11.0).color(crate::theme::TEXT_MUTED));
                        ui.label(RichText::new(&ev.actor).size(11.0).color(crate::theme::TEXT_SECONDARY));
                    });
                    ui.horizontal(|ui| {
                        ui.add_space(14.0);
                        ui.label(RichText::new(&ev.description).size(12.0).color(crate::theme::TEXT_PRIMARY));
                    });
                    ui.add_space(4.0);
                }
            }
        });
}

fn draw_active_rma_queue(
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

fn draw_intake_drawer(
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
