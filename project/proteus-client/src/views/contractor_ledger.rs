//! Frontline Hardware Store & Contractor Job-Site Sub-ledger View for Proteus Client.
//! Provides Touch Quick-Pills for bulk non-barcoded fasteners/materials,
//! dual unit-of-measure conversions (pieces, packs, boxes, meters),
//! and project-based running ledgers for professional contractors (Καρτέλα Μάστορα ανά Έργο).
//! Strict Rule 5 compliance: Reads and writes 100% real SQLite data.

use proteus_core::contractor::{
    add_contractor_entry, create_contractor, init_contractor_schema, list_contractor_entries,
    list_contractors, ContractorLedgerEntry, QuickHardwarePill, UnitOfMeasure,
    DEFAULT_HARDWARE_PILLS,
};
use egui::{Color32, CornerRadius, FontId, Frame, Margin, RichText, Stroke, Ui, Vec2};
use rusqlite::Connection;

pub struct ContractorLedgerState {
    pub selected_contractor_id: Option<String>,
    pub selected_pill: Option<QuickHardwarePill>,
    pub item_quantity: f64,
    pub selected_uom: UnitOfMeasure,
    pub current_worksite: String,
    #[allow(dead_code)]
    pub transaction_notes: String,
    pub payment_amount: String,
    pub new_contractor_name: String,
    pub new_contractor_phone: String,
    pub new_contractor_trade: String,
    pub show_new_contractor_modal: bool,
    pub feedback_msg: Option<(String, bool)>,
    pub schema_initialized: bool,
}

impl Default for ContractorLedgerState {
    fn default() -> Self {
        Self {
            selected_contractor_id: None,
            selected_pill: None,
            item_quantity: 1.0,
            selected_uom: UnitOfMeasure::Piece,
            current_worksite: "Οικοδομή / Έργο".to_string(),
            transaction_notes: String::new(),
            payment_amount: String::new(),
            new_contractor_name: String::new(),
            new_contractor_phone: String::new(),
            new_contractor_trade: "Ηλεκτρολόγος".to_string(),
            show_new_contractor_modal: false,
            feedback_msg: None,
            schema_initialized: false,
        }
    }
}

pub fn draw_contractor_ledger_view(
    ui: &mut Ui,
    conn: &Connection,
    state: &mut ContractorLedgerState,
) {
    if !state.schema_initialized {
        let _ = init_contractor_schema(conn);
        state.schema_initialized = true;
    }

    let contractors = list_contractors(conn).unwrap_or_default();

    // Top Title & Quick Actions
    ui.horizontal(|ui| {
        ui.label(
            RichText::new("🔩 ΤΑΜΕΙΟ ΥΛΙΚΩΝ & ΚΑΡΤΕΛΑ ΜΑΣΤΟΡΑ")
                .font(FontId::proportional(16.0))
                .strong()
                .color(crate::theme::TEXT_PRIMARY),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add(
                    egui::Button::new(
                        RichText::new("➕ Νέος Μάστορας")
                            .font(FontId::proportional(11.0))
                            .color(crate::theme::ICY_MIST),
                    )
                    .fill(crate::theme::BG_CARD)
                    .stroke(Stroke::new(1.0, crate::theme::STEEL_SLATE))
                    .corner_radius(CornerRadius::same(5)),
                )
                .clicked()
            {
                state.show_new_contractor_modal = !state.show_new_contractor_modal;
            }
        });
    });

    if let Some((msg, is_ok)) = &state.feedback_msg {
        let col = if *is_ok { Color32::from_rgb(56, 161, 105) } else { Color32::from_rgb(229, 62, 62) };
        Frame::new()
            .fill(crate::theme::BG_PANEL)
            .inner_margin(Margin::same(8))
            .corner_radius(CornerRadius::same(4))
            .show(ui, |ui| {
                ui.label(RichText::new(msg).color(col).font(FontId::proportional(12.0)));
            });
        ui.add_space(4.0);
    }

    // Modal: New Contractor
    if state.show_new_contractor_modal {
        Frame::new()
            .fill(crate::theme::BG_PANEL)
            .stroke(Stroke::new(1.0, crate::theme::ACCENT_GOLD))
            .corner_radius(CornerRadius::same(6))
            .inner_margin(Margin::same(12))
            .show(ui, |ui| {
                ui.label(RichText::new("Εγγραφή Νέου Επαγγελματία / Μάστορα").strong().color(crate::theme::ACCENT_GOLD));
                ui.horizontal(|ui| {
                    ui.label("Ονοματεπώνυμο:");
                    ui.add(egui::TextEdit::singleline(&mut state.new_contractor_name).desired_width(180.0));
                    ui.label("Τηλέφωνο:");
                    ui.add(egui::TextEdit::singleline(&mut state.new_contractor_phone).desired_width(110.0));
                    ui.label("Ειδικότητα:");
                    ui.add(egui::TextEdit::singleline(&mut state.new_contractor_trade).desired_width(120.0));

                    if ui.button("Αποθήκευση").clicked() {
                        if !state.new_contractor_name.trim().is_empty() {
                            match create_contractor(conn, &state.new_contractor_name, &state.new_contractor_phone, &state.new_contractor_trade) {
                                Ok(c) => {
                                    state.selected_contractor_id = Some(c.id);
                                    state.new_contractor_name.clear();
                                    state.new_contractor_phone.clear();
                                    state.show_new_contractor_modal = false;
                                    state.feedback_msg = Some(("✓ Ο μάστορας καταχωρήθηκε επιτυχώς στη SQLite!".into(), true));
                                }
                                Err(e) => {
                                    state.feedback_msg = Some((format!("Σφάλμα SQLite: {}", e), false));
                                }
                            }
                        }
                    }
                });
            });
        ui.add_space(8.0);
    }

    ui.add_space(6.0);

    // ── Two Columns: Left (Touch Quick-Pills & Fast Dispense), Right (Job-Site Sub-ledger) ──
    ui.columns(2, |cols| {
        // ── LEFT COLUMN: Touch Quick-Pills & Material Calculator ──
        cols[0].vertical(|ui| {
            ui.label(
                RichText::new("⚡ TOUCH QUICK-PILLS (ΧΥΜΑ & ΜΗ-BARCODED ΥΛΙΚΑ)")
                    .font(FontId::proportional(12.0))
                    .strong()
                    .color(crate::theme::ACCENT_GOLD),
            );
            ui.label(
                RichText::new("Άμεση επιλογή βιδών, καλωδίων, εξαρτημάτων με μετατροπή μονάδων.")
                    .font(FontId::proportional(10.5))
                    .color(crate::theme::TEXT_SECONDARY),
            );
            ui.add_space(6.0);

            // Pills Grid
            egui::ScrollArea::vertical().max_height(240.0).show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(6.0, 6.0);
                    for pill in DEFAULT_HARDWARE_PILLS {
                        let is_sel = state.selected_pill.as_ref().map(|p| p.id) == Some(pill.id);
                        let text = format!("{} {}\n{:.2}€/τεμ", pill.icon, pill.name, pill.price_cents_per_piece as f64 / 100.0);
                        let btn = ui.add_sized(
                            Vec2::new(135.0, 50.0),
                            egui::Button::new(
                                RichText::new(text)
                                    .font(FontId::proportional(10.5))
                                    .color(if is_sel { crate::theme::ICY_MIST } else { crate::theme::TEXT_PRIMARY }),
                            )
                            .fill(if is_sel { crate::theme::STEEL_SLATE } else { crate::theme::BG_CARD })
                            .stroke(Stroke::new(1.0, if is_sel { crate::theme::ACCENT_GOLD } else { crate::theme::BORDER_SUBTLE }))
                            .corner_radius(CornerRadius::same(6)),
                        );

                        if btn.clicked() {
                            state.selected_pill = Some(pill.clone());
                        }
                    }
                });
            });

            ui.add_space(10.0);

            // Unit of Measure & Quantity Converter Box
            if let Some(pill) = &state.selected_pill {
                Frame::new()
                    .fill(crate::theme::BG_PANEL)
                    .stroke(Stroke::new(1.0, crate::theme::BORDER_FOCUS))
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(Margin::same(12))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(format!("{} Επιλεγμένο: {}", pill.icon, pill.name)).strong().color(crate::theme::TEXT_PRIMARY));
                        });
                        ui.add_space(6.0);

                        ui.horizontal(|ui| {
                            ui.label("Ποσότητα:");
                            ui.add(egui::DragValue::new(&mut state.item_quantity).speed(1.0).range(0.1..=10000.0));

                            ui.label("Μονάδα:");
                            egui::ComboBox::from_id_salt("uom_selector")
                                .selected_text(state.selected_uom.display_name())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut state.selected_uom, UnitOfMeasure::Piece, "Τεμάχια (τεμ)");
                                    ui.selectable_value(&mut state.selected_uom, UnitOfMeasure::Pack, format!("Πακέτο ({} τεμ)", pill.pieces_per_pack));
                                    ui.selectable_value(&mut state.selected_uom, UnitOfMeasure::Box, format!("Κουτί ({} τεμ)", pill.pieces_per_box));
                                    ui.selectable_value(&mut state.selected_uom, UnitOfMeasure::Meter, "Μέτρα (m)");
                                });
                        });

                        // Calculate total in pieces and total price
                        let total_pieces = UnitOfMeasure::convert_quantity(
                            state.item_quantity,
                            state.selected_uom,
                            UnitOfMeasure::Piece,
                            pill.pieces_per_pack as f64,
                            pill.pieces_per_box as f64,
                        );
                        let total_cents = (total_pieces * pill.price_cents_per_piece as f64).round() as i64;
                        let total_eur = total_cents as f64 / 100.0;

                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(format!("Ισοδύναμο: {:.0} τεμ | Σύνολο: {:.2} €", total_pieces, total_eur)).strong().color(crate::theme::ACCENT_GOLD));
                        });

                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            ui.label("Έργο/Οικοδομή:");
                            ui.add(egui::TextEdit::singleline(&mut state.current_worksite).desired_width(140.0));

                            let can_charge = state.selected_contractor_id.is_some() && !state.current_worksite.trim().is_empty();
                            if ui.add_enabled(
                                can_charge,
                                egui::Button::new(RichText::new("📥 Χρέωση στην Καρτέλα").strong().color(crate::theme::ICY_MIST))
                                    .fill(crate::theme::STEEL_SLATE),
                            ).clicked() {
                                if let Some(cid) = &state.selected_contractor_id {
                                    let desc = format!("{:.0} {} {} ({:.2}€)", state.item_quantity, state.selected_uom.display_name(), pill.name, total_eur);
                                    let entry = ContractorLedgerEntry::new_debit(
                                        cid.clone(),
                                        state.current_worksite.trim(),
                                        desc,
                                        total_cents,
                                    );
                                    match add_contractor_entry(conn, &entry) {
                                        Ok(()) => {
                                            state.feedback_msg = Some((format!("✓ Χρεώθηκαν {:.2}€ στην καρτέλα!", total_eur), true));
                                        }
                                        Err(e) => {
                                            state.feedback_msg = Some((format!("Σφάλμα SQLite: {}", e), false));
                                        }
                                    }
                                }
                            }
                        });
                    });
            }
        });

        // ── RIGHT COLUMN: Contractor Job-Site Sub-ledger ──
        cols[1].vertical(|ui| {
            ui.label(
                RichText::new("📋 ΚΑΡΤΕΛΑ ΜΑΣΤΟΡΑ ΑΝΑ ΕΡΓΟ/ΟΙΚΟΔΟΜΗ")
                    .font(FontId::proportional(12.0))
                    .strong()
                    .color(crate::theme::TEXT_PRIMARY),
            );

            // Contractor Dropdown
            let cur_name = contractors
                .iter()
                .find(|c| Some(&c.id) == state.selected_contractor_id.as_ref())
                .map(|c| format!("{} ({} - υπόλοιπο: {:.2}€)", c.contractor_name, c.trade, c.current_balance_cents as f64 / 100.0))
                .unwrap_or_else(|| "— Επιλέξτε Μάστορα / Επαγγελματία —".to_string());

            egui::ComboBox::from_id_salt("contractor_picker")
                .selected_text(cur_name)
                .width(ui.available_width() - 20.0)
                .show_ui(ui, |ui| {
                    for c in &contractors {
                        let text = format!("{} ({}) | {:.2}€", c.contractor_name, c.trade, c.current_balance_cents as f64 / 100.0);
                        if ui.selectable_label(Some(&c.id) == state.selected_contractor_id.as_ref(), text).clicked() {
                            state.selected_contractor_id = Some(c.id.clone());
                        }
                    }
                });

            ui.add_space(8.0);

            if let Some(cid) = &state.selected_contractor_id {
                let entries = list_contractor_entries(conn, cid).unwrap_or_default();
                let active_contractor = contractors.iter().find(|c| &c.id == cid);

                if let Some(c) = active_contractor {
                    let bal_col = if c.current_balance_cents > 0 { crate::theme::ACCENT_GOLD } else { Color32::from_rgb(56, 161, 105) };
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!("Τηλ: {}", c.phone)).color(crate::theme::TEXT_SECONDARY));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new(format!("Υπόλοιπο: {:.2} €", c.current_balance_cents as f64 / 100.0)).strong().size(13.0).color(bal_col));
                        });
                    });
                }

                // Quick Payment Row
                ui.add_space(4.0);
                Frame::new().fill(crate::theme::BG_PANEL).inner_margin(Margin::same(8)).corner_radius(CornerRadius::same(4)).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Πληρωμή έναντι (€):");
                        ui.add(egui::TextEdit::singleline(&mut state.payment_amount).desired_width(70.0));
                        if ui.button("Καταχώρηση Πληρωμής").clicked() {
                            if let Ok(eur) = state.payment_amount.trim().parse::<f64>() {
                                let cents = (eur * 100.0).round() as i64;
                                let entry = ContractorLedgerEntry::new_credit(
                                    cid.clone(),
                                    "Ταμείο Καταστήματος",
                                    "Πληρωμή έναντι μετρητά/POS",
                                    cents,
                                );
                                let _ = add_contractor_entry(conn, &entry);
                                state.payment_amount.clear();
                                state.feedback_msg = Some((format!("✓ Καταχωρήθηκε πληρωμή {:.2}€!", eur), true));
                            }
                        }
                    });
                });

                ui.add_space(6.0);

                // Entries Table
                egui::ScrollArea::vertical().max_height(280.0).show(ui, |ui| {
                    if entries.is_empty() {
                        ui.label(RichText::new("Δεν υπάρχουν ακόμη κινήσεις για τον συγκεκριμένο μάστορα.").color(crate::theme::TEXT_SECONDARY));
                    } else {
                        for e in entries {
                            Frame::new()
                                .fill(crate::theme::BG_CARD)
                                .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
                                .corner_radius(CornerRadius::same(5))
                                .inner_margin(Margin::same(8))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(&e.worksite).strong().color(crate::theme::ICY_MIST));
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if e.debit_cents > 0 {
                                                ui.label(RichText::new(format!("+{:.2} €", e.debit_cents as f64 / 100.0)).color(Color32::from_rgb(229, 62, 62)));
                                            } else {
                                                ui.label(RichText::new(format!("-{:.2} €", e.credit_cents as f64 / 100.0)).color(Color32::from_rgb(56, 161, 105)));
                                            }
                                        });
                                    });
                                    ui.label(RichText::new(&e.description).size(11.0).color(crate::theme::TEXT_SECONDARY));
                                });
                            ui.add_space(4.0);
                        }
                    }
                });
            } else {
                ui.label(RichText::new("Επιλέξτε έναν μάστορα για να δείτε το υπόλοιπο και τις χρεώσεις ανά έργο.").color(crate::theme::TEXT_SECONDARY));
            }
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contractor_ledger_state_defaults() {
        let state = ContractorLedgerState::default();
        assert!(state.selected_contractor_id.is_none());
        assert_eq!(state.item_quantity, 1.0);
        assert_eq!(state.selected_uom, UnitOfMeasure::Piece);
        assert!(!DEFAULT_HARDWARE_PILLS.is_empty());
    }
}
