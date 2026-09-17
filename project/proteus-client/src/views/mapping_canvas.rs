//! Visual Field Mapping Canvas for Proteus Certified Business Analysts (PCDA).
//! Provides drag-and-drop / connection mapping between external payloads and native entities,
//! fuzzy auto-matching, and real-time live preview.

use crm_core::inference::find_best_field_match;
use crm_core::mapping::{FieldMapping, FieldTransform, SchemaMappingContract};
use eframe::egui::{self, Color32, CornerRadius, Frame, Margin, RichText, Stroke, Ui};
use serde_json::Value;

pub struct MappingCanvasState {
    pub source_entity: String,
    pub target_entity: String,
    pub source_fields: Vec<String>,
    pub target_fields: Vec<String>,
    pub selected_source: String,
    pub selected_target: String,
    pub selected_transform_idx: usize,
    pub mappings: Vec<FieldMapping>,
    pub sample_json_payload: String,
    pub transformed_preview: Option<Value>,
    pub status_feedback: Option<String>,
}

impl Default for MappingCanvasState {
    fn default() -> Self {
        let default_source = vec![
            "cust_name".to_string(),
            "phone".to_string(),
            "device_brand".to_string(),
            "issue_notes".to_string(),
            "net_amount".to_string(),
            "order_date".to_string(),
        ];

        let default_target = vec![
            "customer_name".to_string(),
            "customer_phone".to_string(),
            "device_model".to_string(),
            "reported_fault".to_string(),
            "estimated_cost".to_string(),
            "created_at".to_string(),
        ];

        let mut mappings = Vec::new();
        mappings.push(FieldMapping::pass_through("cust_name", "customer_name"));
        mappings.push(FieldMapping::pass_through("phone", "customer_phone"));
        mappings.push(FieldMapping::pass_through("device_brand", "device_model"));
        mappings.push(FieldMapping::pass_through("issue_notes", "reported_fault"));
        mappings.push(FieldMapping::new(
            "net_amount",
            "estimated_cost",
            FieldTransform::MathMultiply { multiplier: 1.24 },
        ));
        mappings.push(FieldMapping::new(
            "order_date",
            "created_at",
            FieldTransform::DateFormat {
                input_format: "DD/MM/YYYY".to_string(),
                output_format: "YYYY-MM-DD".to_string(),
            },
        ));

        let sample_payload = r#"{
  "cust_name": "Γεώργιος Δημητρίου",
  "phone": "6987654321",
  "device_brand": "Samsung Galaxy S23",
  "issue_notes": "Σπασμένη οθόνη αφής μετά από πτώση",
  "net_amount": 120.00,
  "order_date": "18/09/2026"
}"#.to_string();

        Self {
            source_entity: "external_api_orders".to_string(),
            target_entity: "service_tickets".to_string(),
            selected_source: default_source.first().cloned().unwrap_or_default(),
            selected_target: default_target.first().cloned().unwrap_or_default(),
            source_fields: default_source,
            target_fields: default_target,
            selected_transform_idx: 0,
            mappings,
            sample_json_payload: sample_payload,
            transformed_preview: None,
            status_feedback: None,
        }
    }
}

pub fn render_mapping_canvas(ui: &mut Ui, state: &mut MappingCanvasState) {
    ui.vertical(|ui| {
        // Section Header
        ui.horizontal(|ui| {
            ui.label(RichText::new("⚡ Visual Field Mapping Canvas").strong().size(15.0).color(crate::theme::ACCENT_CYAN));
            ui.label(RichText::new(format!("({} ➔ {})", state.source_entity, state.target_entity)).size(12.0).color(crate::theme::TEXT_MUTED));
        });

        ui.label(RichText::new("Αντιστοίχιση εξωτερικών πεδίων (API/JSON) με εσωτερικές οντότητες Proteus με έξυπνους μετασχηματισμούς.").size(12.0).color(crate::theme::TEXT_MUTED));
        ui.add_space(8.0);

        // Control Toolbar
        ui.horizontal(|ui| {
            if ui.button("⚡ Fuzzy Auto-Match (AI)").clicked() {
                let targets_ref: Vec<&str> = state.target_fields.iter().map(|s| s.as_str()).collect();
                let mut matched_count = 0;
                for src in &state.source_fields {
                    if let Some((target, score)) = find_best_field_match(src, &targets_ref) {
                        if score >= 0.55 && !state.mappings.iter().any(|m| m.source_field == *src) {
                            state.mappings.push(FieldMapping::pass_through(src.clone(), target));
                            matched_count += 1;
                        }
                    }
                }
                state.status_feedback = Some(format!("Αυτόματη σύνδεση {} νέων πεδίων βάσει ομοιότητας!", matched_count));
            }

            if ui.button("👁 Preview Transform").clicked() {
                if let Ok(src_val) = serde_json::from_str::<Value>(&state.sample_json_payload) {
                    let mut contract = SchemaMappingContract::new("TMP-01", "Live Preview", &state.source_entity, &state.target_entity);
                    for m in &state.mappings {
                        contract.add_mapping(m.clone());
                    }
                    let res = contract.apply_record(&src_val);
                    state.transformed_preview = Some(res);
                    state.status_feedback = Some("Επιτυχής μετασχηματισμός δείγματος!".to_string());
                } else {
                    state.status_feedback = Some("Σφάλμα: Μη έγκυρο δείγμα JSON.".to_string());
                }
            }

            if ui.button("🗑 Καθαρισμός Όλων").clicked() {
                state.mappings.clear();
                state.transformed_preview = None;
                state.status_feedback = Some("Καθαρίστηκαν όλες οι συνδέσεις.".to_string());
            }
        });

        if let Some(fb) = &state.status_feedback {
            ui.add_space(4.0);
            ui.label(RichText::new(fb).color(Color32::from_rgb(52, 211, 153)).size(11.0));
        }

        ui.add_space(10.0);

        // Visual Mapping Grid: 3 columns (Source Fields | Connection & Transform | Target Fields)
        Frame::new()
            .fill(crate::theme::BG_CARD)
            .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(Margin::same(12))
            .show(ui, |ui| {
                ui.columns(3, |columns| {
                    // Column 1: Source Fields
                    columns[0].vertical(|ui| {
                        ui.label(RichText::new("📥 Εισερχόμενα Πεδία (Source)").strong().size(13.0).color(Color32::WHITE));
                        ui.add_space(4.0);
                        for src in &state.source_fields {
                            let is_mapped = state.mappings.iter().any(|m| m.source_field == *src);
                            let bg = if is_mapped { Color32::from_rgb(15, 23, 42) } else { crate::theme::BG_BASE };
                            let border = if is_mapped { Color32::from_rgb(56, 189, 248) } else { crate::theme::BORDER_SUBTLE };

                            Frame::new()
                                .fill(bg)
                                .stroke(Stroke::new(1.0, border))
                                .corner_radius(CornerRadius::same(4))
                                .inner_margin(Margin::symmetric(8, 5))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(src).monospace().size(11.0).color(Color32::WHITE));
                                        if is_mapped {
                                            ui.label(RichText::new("🔗").size(10.0));
                                        }
                                    });
                                });
                            ui.add_space(2.0);
                        }
                    });

                    // Column 2: Active Connections & Transforms
                    columns[1].vertical(|ui| {
                        ui.label(RichText::new(format!("⚙️ Ενεργές Συνδέσεις ({})", state.mappings.len())).strong().size(13.0).color(crate::theme::ACCENT_GOLD));
                        ui.add_space(4.0);

                        let mut to_remove = None;
                        for (idx, mapping) in state.mappings.iter().enumerate() {
                            Frame::new()
                                .fill(crate::theme::BG_BASE)
                                .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
                                .corner_radius(CornerRadius::same(4))
                                .inner_margin(Margin::symmetric(6, 4))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(&mapping.source_field).size(10.0).color(Color32::from_rgb(56, 189, 248)));
                                        ui.label(RichText::new("➔").size(10.0).color(crate::theme::TEXT_MUTED));
                                        ui.label(RichText::new(&mapping.target_field).size(10.0).color(Color32::from_rgb(52, 211, 153)));

                                        let transform_tag = match &mapping.transform {
                                            FieldTransform::PassThrough => "Direct",
                                            FieldTransform::MathMultiply { .. } => "× VAT",
                                            FieldTransform::DateFormat { .. } => "Date",
                                            FieldTransform::Concatenate { .. } => "Concat",
                                            FieldTransform::Lookup { .. } => "Lookup",
                                            FieldTransform::ToUpperCase => "UPPER",
                                            FieldTransform::ToLowerCase => "lower",
                                            FieldTransform::Prefix { .. } => "Prefix",
                                        };
                                        ui.label(RichText::new(format!("[{}]", transform_tag)).size(9.0).color(Color32::from_rgb(251, 191, 36)));

                                        if ui.small_button("✖").clicked() {
                                            to_remove = Some(idx);
                                        }
                                    });
                                });
                            ui.add_space(2.0);
                        }

                        if let Some(idx) = to_remove {
                            state.mappings.remove(idx);
                        }

                        // Add new mapping widget
                        ui.add_space(6.0);
                        ui.separator();
                        ui.horizontal(|ui| {
                            egui::ComboBox::from_id_salt("source_select")
                                .selected_text(&state.selected_source)
                                .show_ui(ui, |ui| {
                                    for s in &state.source_fields {
                                        ui.selectable_value(&mut state.selected_source, s.clone(), s);
                                    }
                                });

                            ui.label("➔");

                            egui::ComboBox::from_id_salt("target_select")
                                .selected_text(&state.selected_target)
                                .show_ui(ui, |ui| {
                                    for t in &state.target_fields {
                                        ui.selectable_value(&mut state.selected_target, t.clone(), t);
                                    }
                                });

                            let transform_names = ["Direct", "× 1.24 VAT", "DD/MM ➔ ISO", "Uppercase", "Lowercase"];
                            let curr_name = transform_names.get(state.selected_transform_idx).unwrap_or(&"Direct");
                            egui::ComboBox::from_id_salt("transform_select")
                                .selected_text(*curr_name)
                                .show_ui(ui, |ui| {
                                    for (idx, name) in transform_names.iter().enumerate() {
                                        ui.selectable_value(&mut state.selected_transform_idx, idx, *name);
                                    }
                                });

                            if ui.button("➕ Σύνδεση").clicked() {
                                if !state.mappings.iter().any(|m| m.source_field == state.selected_source && m.target_field == state.selected_target) {
                                    let transform = match state.selected_transform_idx {
                                        1 => FieldTransform::MathMultiply { multiplier: 1.24 },
                                        2 => FieldTransform::DateFormat {
                                            input_format: "DD/MM/YYYY".to_string(),
                                            output_format: "YYYY-MM-DD".to_string(),
                                        },
                                        3 => FieldTransform::ToUpperCase,
                                        4 => FieldTransform::ToLowerCase,
                                        _ => FieldTransform::PassThrough,
                                    };
                                    state.mappings.push(FieldMapping::new(&state.selected_source, &state.selected_target, transform));
                                }
                            }
                        });
                    });

                    // Column 3: Target Fields
                    columns[2].vertical(|ui| {
                        ui.label(RichText::new("🎯 Τοπική Οντότητα (Target)").strong().size(13.0).color(Color32::WHITE));
                        ui.add_space(4.0);
                        for tgt in &state.target_fields {
                            let is_mapped = state.mappings.iter().any(|m| m.target_field == *tgt);
                            let bg = if is_mapped { Color32::from_rgb(6, 78, 59) } else { crate::theme::BG_BASE };
                            let border = if is_mapped { Color32::from_rgb(52, 211, 153) } else { crate::theme::BORDER_SUBTLE };

                            Frame::new()
                                .fill(bg)
                                .stroke(Stroke::new(1.0, border))
                                .corner_radius(CornerRadius::same(4))
                                .inner_margin(Margin::symmetric(8, 5))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(tgt).monospace().size(11.0).color(Color32::WHITE));
                                        if is_mapped {
                                            ui.label(RichText::new("✔").size(10.0).color(Color32::from_rgb(52, 211, 153)));
                                        }
                                    });
                                });
                            ui.add_space(2.0);
                        }
                    });
                });
            });

        // Live Transformed Preview Box
        if let Some(preview) = &state.transformed_preview {
            ui.add_space(8.0);
            ui.label(RichText::new("📊 Live Transformed Record (Προεπισκόπηση σε πραγματικό χρόνο):").strong().size(12.0).color(Color32::WHITE));
            Frame::new()
                .fill(crate::theme::BG_CARD)
                .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::same(10))
                .show(ui, |ui| {
                    let pretty_json = serde_json::to_string_pretty(preview).unwrap_or_default();
                    ui.label(RichText::new(pretty_json).monospace().size(11.0).color(Color32::from_rgb(167, 243, 208)));
                });
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mapping_canvas_defaults() {
        let state = MappingCanvasState::default();
        assert_eq!(state.source_fields.len(), 6);
        assert_eq!(state.target_fields.len(), 6);
        assert_eq!(state.mappings.len(), 6);
    }

    #[test]
    fn test_mapping_canvas_fuzzy_matching() {
        let mut state = MappingCanvasState::default();
        state.mappings.clear();
        assert_eq!(state.mappings.len(), 0);

        let targets_ref: Vec<&str> = state.target_fields.iter().map(|s| s.as_str()).collect();
        for src in &state.source_fields {
            if let Some((target, score)) = find_best_field_match(src, &targets_ref) {
                if score >= 0.55 {
                    state.mappings.push(FieldMapping::pass_through(src.clone(), target));
                }
            }
        }
        assert!(state.mappings.len() >= 2);
    }
}
