//! Tripartite Pipeline Studio Inspector (Designer <-> Data Structurer <-> Business Engine).
//!
//! Provides visual component binding inspection, entity schema verification (field types,
//! required constraints), reactive business rule attachment, and real-time dry-run execution.

use eframe::egui::{self, Color32, CornerRadius, RichText, Vec2};
use proteus_core::rules::{BusinessRule, RuleAction, RuleCondition};
use proteus_core::schema::{EntitySchema, FieldDefinition, FieldType};
use proteus_core::tripartite::{ComponentBinding, TripartitePipeline, TripartiteResult, UiTriggerEvent};
use std::time::Instant;

use crate::theme;

/// Active state for the Tripartite Pipeline Inspector panel.
#[derive(Debug, Clone)]
pub struct TripartiteInspectorState {
    pub selected_entity: String,
    pub bound_rule_id: String,
    pub rule_condition_val: f64,
    pub rule_discount_pct: f64,
    pub last_result: Option<TripartiteResult>,
    pub last_latency_us: Option<u128>,
    pub status_message: Option<String>,
}

impl Default for TripartiteInspectorState {
    fn default() -> Self {
        Self {
            selected_entity: "deals".to_string(),
            bound_rule_id: "bulk_discount_10".to_string(),
            rule_condition_val: 10.0,
            rule_discount_pct: 15.0,
            last_result: None,
            last_latency_us: None,
            status_message: None,
        }
    }
}

impl TripartiteInspectorState {
    /// Executes a live end-to-end dry-run test through the Tripartite Pipeline.
    pub fn execute_dry_run(&mut self, component_id: &str, trigger: UiTriggerEvent) {
        let start = Instant::now();
        let entity = self.selected_entity.clone();
        let schema = standard_entity_schema(&entity);

        // Build sample test payload conforming to schema
        let mut sample_payload = serde_json::Map::new();
        for field in &schema.fields {
            let val = match &field.field_type {
                FieldType::Text => serde_json::Value::String(format!("Sample {}", field.label)),
                FieldType::Number => serde_json::json!(self.rule_condition_val + 5.0),
                FieldType::Boolean => serde_json::Value::Bool(true),
                FieldType::Email => serde_json::Value::String("demo@proteus.gr".into()),
                FieldType::Phone => serde_json::Value::String("+302101234567".into()),
                FieldType::Date => serde_json::Value::String("2026-10-15".into()),
                FieldType::Select(opts) => {
                    serde_json::Value::String(opts.first().cloned().unwrap_or_else(|| "Default".into()))
                }
            };
            sample_payload.insert(field.name.clone(), val);
        }

        // Setup TripartitePipeline with in-memory SQLite store
        let conn = match rusqlite::Connection::open_in_memory() {
            Ok(c) => c,
            Err(_) => {
                self.status_message = Some("Failed to initialize in-memory SQLite runner".into());
                return;
            }
        };

        let mut pipeline = TripartitePipeline::new();
        pipeline.register_schema(schema);

        let binding = ComponentBinding::new(component_id, &entity, trigger);
        pipeline.bind_component(binding);

        // Attach selected business rule if configured
        if !self.bound_rule_id.is_empty() {
            let rule = BusinessRule {
                id: self.bound_rule_id.clone(),
                name: self.bound_rule_id.clone(),
                entity_type: entity.clone(),
                condition: RuleCondition::FieldGreaterThan {
                    field: "value".to_string(),
                    value: self.rule_condition_val,
                },
                actions: vec![RuleAction::ApplyDiscountPercentage {
                    field: "value".to_string(),
                    percentage: self.rule_discount_pct,
                }],
                is_active: true,
            };
            pipeline.add_business_rule(rule);
        }

        let result = pipeline.process_interaction(
            component_id,
            &serde_json::Value::Object(sample_payload),
            Some(&conn),
        );
        let elapsed = start.elapsed().as_micros();

        self.last_latency_us = Some(elapsed);
        self.last_result = Some(result);
    }
}

/// Provides standard entity schemas for core business domains.
pub fn standard_entity_schema(entity_name: &str) -> EntitySchema {
    match entity_name {
        "contacts" => EntitySchema::new("contacts", "Customer Contacts")
            .with_field(FieldDefinition {
                name: "name".into(),
                label: "Full Name".into(),
                field_type: FieldType::Text,
                required: true,
                default_value: None,
            })
            .with_field(FieldDefinition {
                name: "email".into(),
                label: "Email Address".into(),
                field_type: FieldType::Email,
                required: true,
                default_value: None,
            })
            .with_field(FieldDefinition {
                name: "phone".into(),
                label: "Phone Number".into(),
                field_type: FieldType::Phone,
                required: false,
                default_value: None,
            }),
        "deals" => EntitySchema::new("deals", "Sales Deals & Orders")
            .with_field(FieldDefinition {
                name: "title".into(),
                label: "Deal Title".into(),
                field_type: FieldType::Text,
                required: true,
                default_value: None,
            })
            .with_field(FieldDefinition {
                name: "value".into(),
                label: "Deal Amount (€)".into(),
                field_type: FieldType::Number,
                required: true,
                default_value: None,
            })
            .with_field(FieldDefinition {
                name: "stage".into(),
                label: "Pipeline Stage".into(),
                field_type: FieldType::Select(vec!["New".into(), "Qualified".into(), "Won".into(), "Lost".into()]),
                required: true,
                default_value: None,
            }),
        "tickets" => EntitySchema::new("tickets", "Support Tickets")
            .with_field(FieldDefinition {
                name: "title".into(),
                label: "Issue Subject".into(),
                field_type: FieldType::Text,
                required: true,
                default_value: None,
            })
            .with_field(FieldDefinition {
                name: "priority".into(),
                label: "Severity".into(),
                field_type: FieldType::Select(vec!["Low".into(), "Normal".into(), "High".into(), "Critical".into()]),
                required: true,
                default_value: None,
            }),
        "inventory" => EntitySchema::new("inventory", "Inventory Items")
            .with_field(FieldDefinition {
                name: "sku".into(),
                label: "SKU Code".into(),
                field_type: FieldType::Text,
                required: true,
                default_value: None,
            })
            .with_field(FieldDefinition {
                name: "price".into(),
                label: "Unit Retail Price".into(),
                field_type: FieldType::Number,
                required: true,
                default_value: None,
            })
            .with_field(FieldDefinition {
                name: "stock".into(),
                label: "Available Stock".into(),
                field_type: FieldType::Number,
                required: true,
                default_value: None,
            }),
        _ => EntitySchema::new(entity_name, entity_name)
            .with_field(FieldDefinition {
                name: "id".into(),
                label: "Identifier".into(),
                field_type: FieldType::Text,
                required: true,
                default_value: None,
            })
            .with_field(FieldDefinition {
                name: "title".into(),
                label: "Title".into(),
                field_type: FieldType::Text,
                required: false,
                default_value: None,
            }),
    }
}

/// Draws the unified Tripartite Pipeline section in the Inspector.
pub fn draw_tripartite_section(
    ui: &mut egui::Ui,
    state: &mut TripartiteInspectorState,
    component_id: &str,
    trigger: UiTriggerEvent,
) {
    ui.add_space(8.);
    ui.separator();
    ui.add_space(6.);

    ui.horizontal(|ui| {
        ui.label(RichText::new("⚡ TRIPARTITE PIPELINE").size(9.5).color(theme::ACCENT).strong());
        ui.label(RichText::new("(UI ➔ Schema ➔ Rules ➔ DB)").size(8.0).color(theme::TEXT_DIM));
    });
    ui.add_space(4.);

    // ── Stage 1: Designer Component Trigger ──
    ui.label(RichText::new("1. DESIGNER TRIGGER:").size(8.5).color(Color32::from_rgb(56, 189, 248)));
    ui.horizontal(|ui| {
        ui.label(RichText::new(format!("Element: {}", component_id)).size(9.0).color(theme::TEXT));
        ui.label(RichText::new(format!("Event: {:?}", trigger)).size(8.5).color(theme::TEXT_MUTED));
    });

    // ── Stage 2: Data Structurer Schema ──
    ui.add_space(4.);
    ui.label(RichText::new("2. DATA STRUCTURER SCHEMA:").size(8.5).color(Color32::from_rgb(245, 158, 11)));
    
    ui.horizontal(|ui| {
        ui.label(RichText::new("Target Entity:").size(9.0));
        egui::ComboBox::from_id_salt(format!("trip_ent_{}", component_id))
            .selected_text(&state.selected_entity)
            .width(130.)
            .show_ui(ui, |ui| {
                for ent in &["deals", "contacts", "tickets", "inventory"] {
                    ui.selectable_value(&mut state.selected_entity, ent.to_string(), *ent);
                }
            });
    });

    let schema = standard_entity_schema(&state.selected_entity);
    ui.horizontal_wrapped(|ui| {
        for fld in &schema.fields {
            let req_str = if fld.required { " *" } else { "" };
            let pill_text = format!("{}{}: {:?}", fld.name, req_str, fld.field_type);
            ui.label(
                RichText::new(pill_text)
                    .size(8.0)
                    .color(if fld.required { Color32::from_rgb(234, 179, 8) } else { theme::TEXT_DIM }),
            );
        }
    });

    // ── Stage 3: Reactive Business Engine Rules ──
    ui.add_space(4.);
    ui.label(RichText::new("3. BUSINESS ENGINE RULE:").size(8.5).color(Color32::from_rgb(34, 197, 94)));
    
    ui.horizontal(|ui| {
        ui.label(RichText::new("Rule ID:").size(9.0));
        ui.add(egui::TextEdit::singleline(&mut state.bound_rule_id).hint_text("rule_name").desired_width(120.));
    });

    ui.horizontal(|ui| {
        ui.label(RichText::new("Condition:").size(8.5).color(theme::TEXT_MUTED));
        ui.label(RichText::new(format!("value >= {:.0}", state.rule_condition_val)).size(8.5).color(theme::TEXT));
        ui.label(RichText::new(format!("➔ -{:.0}%", state.rule_discount_pct)).size(8.5).color(Color32::from_rgb(34, 197, 94)));
    });

    // ── Stage 4: Real-time Dry-Run Execution ──
    ui.add_space(6.);
    let run_btn = ui.add(
        egui::Button::new(RichText::new("⚡ Run Tripartite Dry-Run Test").size(9.5).strong().color(Color32::WHITE))
            .fill(Color32::from_rgb(16, 185, 129))
            .corner_radius(CornerRadius::same(3))
            .min_size(Vec2::new(ui.available_width(), 22.)),
    );

    if run_btn.clicked() {
        state.execute_dry_run(component_id, trigger);
    }

    if let Some(ref res) = state.last_result {
        ui.add_space(4.);
        let badge_bg = if res.success { Color32::from_rgb(20, 83, 45) } else { Color32::from_rgb(127, 29, 29) };
        let badge_text = if res.success {
            format!("✓ SUCCESS ({} µs)", state.last_latency_us.unwrap_or(0))
        } else {
            "✕ FAILED".to_string()
        };

        ui.horizontal(|ui| {
            ui.label(RichText::new(badge_text).size(9.0).color(Color32::WHITE).background_color(badge_bg));
            if !res.applied_rules.is_empty() {
                ui.label(RichText::new(format!("Applied: {:?}", res.applied_rules)).size(8.0).color(Color32::from_rgb(34, 197, 94)));
            }
        });

        if !res.validation_errors.is_empty() {
            for err in &res.validation_errors {
                ui.label(RichText::new(format!("Schema Error: {}", err)).size(8.0).color(Color32::from_rgb(239, 68, 68)));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_schemas_completeness() {
        for ent in &["deals", "contacts", "tickets", "inventory"] {
            let schema = standard_entity_schema(ent);
            assert_eq!(schema.entity, *ent);
            assert!(!schema.fields.is_empty(), "Schema for '{}' must have fields", ent);
            assert!(schema.fields.iter().any(|f| f.required), "Schema for '{}' must have at least one required field", ent);
        }
    }

    #[test]
    fn test_tripartite_dry_run_success() {
        let mut state = TripartiteInspectorState::default();
        state.selected_entity = "deals".to_string();
        state.bound_rule_id = "test_bulk_discount".to_string();
        state.rule_condition_val = 10.0;
        state.rule_discount_pct = 15.0;

        state.execute_dry_run("btn_submit_deal", UiTriggerEvent::OnButtonClick);

        let res = state.last_result.as_ref().expect("Execution result must exist");
        assert!(res.success, "Pipeline dry run must succeed");
        assert_eq!(res.component_id, "btn_submit_deal");
        assert_eq!(res.target_entity, "deals");
        assert!(res.applied_rules.contains(&"test_bulk_discount".to_string()));
        assert!(state.last_latency_us.is_some());
    }
}
