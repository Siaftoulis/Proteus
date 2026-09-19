//! Analyst Studio View for Proteus Certified Data/Business Analysts (PCDA).
//! Provides visual schema inference, GS1-128 barcode inspection, and business rules validation.

use crm_core::gs1::{Gs1BarcodeData, Gs1Parser};
use crm_core::inference::{InferredTable, SchemaInferer};
use crm_core::rules::{BusinessRule, BusinessRulesEngine, RuleAction, RuleCondition};
use eframe::egui::{self, Color32, CornerRadius, Frame, Margin, RichText, Stroke, Ui};
use serde_json::Value;
use crate::views::mapping_canvas::{render_mapping_canvas, MappingCanvasState};

pub struct AnalystStudioState {
    pub raw_json_input: String,
    pub inferred_table: Option<InferredTable>,
    pub gs1_input: String,
    pub parsed_gs1: Option<Gs1BarcodeData>,
    pub rules_simulation_result: Option<String>,
    pub status_message: Option<(String, bool)>, // (msg, is_error)
    pub mapping_canvas: MappingCanvasState,
}

impl Default for AnalystStudioState {
    fn default() -> Self {
        Self {
            raw_json_input: r#"{
  "item_sku": "TECH-9082",
  "name": "Sony Wireless Headphones",
  "price": 149.90,
  "stock_qty": 35,
  "specs": {
    "battery_hours": 30,
    "has_anc": true
  }
}"#.to_string(),
            inferred_table: None,
            gs1_input: "(01)08412345678905(10)LOT-2026-X(17)261231".to_string(),
            parsed_gs1: None,
            rules_simulation_result: None,
            status_message: None,
            mapping_canvas: MappingCanvasState::default(),
        }
    }
}

pub fn render_analyst_studio(ui: &mut Ui, state: &mut AnalystStudioState) {
    ui.vertical(|ui| {
        // Header
        ui.horizontal(|ui| {
            ui.label(RichText::new("📊 PCDA Studio").size(20.0).strong().color(crate::theme::TEXT_PRIMARY));
            ui.label(RichText::new("— Business Data Analysis & Schema Ingestion").size(14.0).color(crate::theme::TEXT_MUTED));
        });

        ui.label(RichText::new("Το πρώτο σημείο επαφής με την επιχείρηση. Κανονικοποίηση δεδομένων, ανάλυση schemas και επικύρωση κανόνων πριν την υλοποίηση UI από τον Designer.").size(12.0).color(crate::theme::TEXT_MUTED));

        ui.add_space(10.0);

        if let Some((msg, is_error)) = &state.status_message {
            let color = if *is_error { Color32::from_rgb(248, 113, 113) } else { Color32::from_rgb(52, 211, 153) };
            Frame::new()
                .fill(crate::theme::BG_CARD)
                .stroke(Stroke::new(1.0, color))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::symmetric(12, 6))
                .show(ui, |ui| {
                    ui.label(RichText::new(msg).color(color).strong());
                });
            ui.add_space(8.0);
        }

        // Section 1: Schema Inference & Universal Connector
        Frame::new()
            .fill(crate::theme::BG_CARD)
            .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(Margin::same(14))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("1. Universal Schema Ingestion (JSON / API / CSV)").strong().size(14.0).color(crate::theme::ACCENT_CYAN));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("⚡ Run Schema Inference").clicked() {
                            match serde_json::from_str::<Value>(&state.raw_json_input) {
                                Ok(val) => {
                                    match SchemaInferer::infer_from_json("imported_dataset", &val) {
                                        Ok(table) => {
                                            state.mapping_canvas.source_fields = table.columns.iter().map(|c| c.name.clone()).collect();
                                            if let Some(first) = state.mapping_canvas.source_fields.first() {
                                                state.mapping_canvas.selected_source = first.clone();
                                            }
                                            state.inferred_table = Some(table);
                                            state.status_message = Some(("Επιτυχές Schema Inference! Κανονικοποιήθηκαν τα πεδία.".to_string(), false));
                                        }
                                        Err(err) => {
                                            state.status_message = Some((format!("Σφάλμα Inference: {}", err), true));
                                        }
                                    }
                                }
                                Err(err) => {
                                    state.status_message = Some((format!("Μη έγκυρο JSON: {}", err), true));
                                }
                            }
                        }
                    });
                });

                ui.add_space(6.0);
                ui.add(
                    egui::TextEdit::multiline(&mut state.raw_json_input)
                        .font(egui::TextStyle::Monospace)
                        .desired_rows(6)
                        .desired_width(f32::INFINITY),
                );

                if let Some(table) = &state.inferred_table {
                    ui.add_space(10.0);
                    ui.label(RichText::new(format!("Κανονικοποιημένες Στήλες ({} πεδία, PK: {:?}):", table.columns.len(), table.primary_key)).strong().color(Color32::WHITE));

                    ui.horizontal_wrapped(|ui| {
                        for col in &table.columns {
                            let pk_str = if col.is_primary_key { " 🔑 PK" } else { "" };
                            let tag = format!("{}: {:?}{}", col.name, col.col_type, pk_str);
                            Frame::new()
                                .fill(crate::theme::BG_BASE)
                                .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
                                .corner_radius(CornerRadius::same(4))
                                .inner_margin(Margin::symmetric(8, 4))
                                .show(ui, |ui| {
                                    let col_color = if col.is_primary_key { Color32::from_rgb(251, 191, 36) } else { crate::theme::TEXT_SECONDARY };
                                    ui.label(RichText::new(tag).size(11.0).color(col_color).monospace());
                                });
                        }
                    });

                    ui.add_space(6.0);
                    ui.label(RichText::new("Generated DDL (SQLite Contract για τον Designer):").size(11.0).color(crate::theme::TEXT_MUTED));
                    Frame::new()
                        .fill(crate::theme::BG_BASE)
                        .corner_radius(CornerRadius::same(4))
                        .inner_margin(Margin::same(8))
                        .show(ui, |ui| {
                            ui.label(RichText::new(table.to_sqlite_ddl()).monospace().size(11.0).color(Color32::from_rgb(148, 163, 184)));
                        });

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button(RichText::new("📦 Εξαγωγή .pr Package").color(Color32::from_rgb(52, 211, 153)).strong()).clicked() {
                            let pkg = crm_core::package::PrPackage::from_inferred_table(table, "PCDA Analyst");
                            match pkg.to_bytes() {
                                Ok(bytes) => {
                                    state.status_message = Some((
                                        format!("✓ Το πακέτο '{}' εξήχθη επιτυχώς! ({} bytes, SHA-256 Verified)", pkg.manifest.name, bytes.len()),
                                        false,
                                    ));
                                }
                                Err(e) => {
                                    state.status_message = Some((format!("❌ Σφάλμα εξαγωγής πακέτου: {}", e), true));
                                }
                            }
                        }

                        if ui.add(egui::Button::new(RichText::new("🚀 Hot-Mount στο Κατάστημα (LAN)").color(Color32::WHITE).strong())
                            .fill(crate::theme::ACCENT_PRIMARY))
                            .clicked()
                        {
                            let pkg = crm_core::package::PrPackage::from_inferred_table(table, "PCDA Analyst");
                            match pkg.deploy_to_client("http://127.0.0.1:7443") {
                                Ok(summary) => {
                                    state.status_message = Some((
                                        format!("✓ Παραδόθηκε & Εγκαταστάθηκε στο τοπικό τερματικό: '{}'!", summary.package_name),
                                        false,
                                    ));
                                }
                                Err(e) => {
                                    state.status_message = Some((format!("❌ Σφάλμα αποστολής LAN (127.0.0.1:7443): {}", e), true));
                                }
                            }
                        }
                    });
                }
            });

        ui.add_space(12.0);

        // Section 2: GS1-128 Barcode Decoder
        Frame::new()
            .fill(crate::theme::BG_CARD)
            .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(Margin::same(14))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("2. GS1-128 Application Identifier (AI) Decoder").strong().size(14.0).color(Color32::from_rgb(52, 211, 153)));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("🏷 Decode Barcode").clicked() {
                            let parsed = Gs1Parser::parse(&state.gs1_input);
                            state.parsed_gs1 = Some(parsed);
                        }
                    });
                });

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.label("Barcode:");
                    ui.add(egui::TextEdit::singleline(&mut state.gs1_input).desired_width(400.0).font(egui::TextStyle::Monospace));
                });

                if let Some(data) = &state.parsed_gs1 {
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if let Some(gtin) = &data.gtin {
                            ui.label(RichText::new(format!("GTIN: {}", gtin)).color(Color32::from_rgb(56, 189, 248)).strong());
                        }
                        if let Some(lot) = &data.batch_lot {
                            ui.label(RichText::new(format!("Lot/Batch: {}", lot)).color(Color32::from_rgb(251, 191, 36)).strong());
                        }
                        if let Some(exp) = &data.expiry_date {
                            ui.label(RichText::new(format!("Expiry: {}", exp)).color(Color32::from_rgb(248, 113, 113)).strong());
                        }
                        if let Some(sn) = &data.serial_number {
                            ui.label(RichText::new(format!("Serial: {}", sn)).color(Color32::from_rgb(168, 85, 247)).strong());
                        }
                    });
                }
            });

        ui.add_space(12.0);

        // Section 3: Business Rules Simulator
        Frame::new()
            .fill(crate::theme::BG_CARD)
            .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(Margin::same(14))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("3. Business Rules Engine (\"IF X THEN Y\")").strong().size(14.0).color(Color32::from_rgb(249, 115, 22)));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("▶ Run Rules Simulator").clicked() {
                            let rule = BusinessRule {
                                id: "BR-01".to_string(),
                                name: "Low Stock Alert & 15% VIP Discount".to_string(),
                                entity_type: "item".to_string(),
                                condition: RuleCondition::FieldGreaterThan {
                                    field: "price".to_string(),
                                    value: 100.0,
                                },
                                actions: vec![
                                    RuleAction::ApplyDiscountPercentage {
                                        field: "price".to_string(),
                                        percentage: 15.0,
                                    },
                                    RuleAction::FlagForReview {
                                        reason: "VIP Pricing Triggered".to_string(),
                                    },
                                ],
                                is_active: true,
                            };

                            let engine = BusinessRulesEngine::new(vec![rule]);
                            if let Ok(mut val) = serde_json::from_str::<Value>(&state.raw_json_input) {
                                let outcome = engine.execute("item", &mut val);
                                state.rules_simulation_result = Some(format!(
                                    "Εφαρμοσμένοι Κανόνες: {:?} | Flags: {:?} | Νέα Δεδομένα:\n{}",
                                    outcome.applied_rules,
                                    outcome.flags,
                                    serde_json::to_string_pretty(&val).unwrap_or_default()
                                ));
                            }
                        }
                    });
                });

                if let Some(res) = &state.rules_simulation_result {
                    ui.add_space(8.0);
                    Frame::new()
                        .fill(crate::theme::BG_BASE)
                        .corner_radius(CornerRadius::same(4))
                        .inner_margin(Margin::same(8))
                        .show(ui, |ui| {
                            ui.label(RichText::new(res).monospace().size(11.0).color(Color32::from_rgb(167, 243, 208)));
                        });
                }
            });

        ui.add_space(12.0);

        // Section 4: Visual Field Mapping Canvas
        render_mapping_canvas(ui, &mut state.mapping_canvas);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyst_studio_defaults() {
        let state = AnalystStudioState::default();
        assert!(!state.raw_json_input.is_empty());
        assert!(state.gs1_input.contains("(01)"));
        assert!(state.inferred_table.is_none());
    }
}
