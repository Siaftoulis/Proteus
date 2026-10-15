//! Tripartite Pipeline Architecture: Designer <-> Data Structurer <-> Business Engine
//!
//! Unites the three fundamental pillars of the Proteus Sovereign BOS:
//! 1. Designer Layer: Visual UI components, inputs, and form bindings.
//! 2. Data Structurer Layer: Entity schemas, field types, constraints, and validation.
//! 3. Business Engine Layer: Reactive business rules, automated actions, and compliance gates.

use std::collections::HashMap;
use rusqlite::{params, Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::rules::{BusinessRule, BusinessRulesEngine, RuleExecutionOutcome};
use crate::schema::{EntitySchema, FieldType};

/// Trigger events emitted by Designer UI components.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UiTriggerEvent {
    OnChange,
    OnSubmit,
    OnButtonClick,
    OnFormBlur,
}

/// Binding between a visual Designer component and a Data Structurer entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentBinding {
    pub component_id: String,
    pub target_entity: String,
    pub field_mappings: HashMap<String, String>, // UI element key -> Entity column name
    pub trigger_event: UiTriggerEvent,
}

impl ComponentBinding {
    pub fn new(
        component_id: impl Into<String>,
        target_entity: impl Into<String>,
        trigger_event: UiTriggerEvent,
    ) -> Self {
        Self {
            component_id: component_id.into(),
            target_entity: target_entity.into(),
            field_mappings: HashMap::new(),
            trigger_event,
        }
    }

    pub fn with_field(mut self, ui_key: impl Into<String>, entity_col: impl Into<String>) -> Self {
        self.field_mappings.insert(ui_key.into(), entity_col.into());
        self
    }
}

/// Execution outcome returned by the Tripartite Pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TripartiteResult {
    pub success: bool,
    pub component_id: String,
    pub target_entity: String,
    pub transformed_payload: Value,
    pub validation_errors: Vec<String>,
    pub applied_rules: Vec<String>,
    pub business_rejections: Vec<String>,
    pub persisted_record_id: Option<String>,
}

/// Tripartite Pipeline Coordinator linking Designer, Data Structurer, and Business Engine.
pub struct TripartitePipeline {
    schemas: HashMap<String, EntitySchema>,
    bindings: HashMap<String, ComponentBinding>,
    rules: Vec<BusinessRule>,
}

impl Default for TripartitePipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl TripartitePipeline {
    pub fn new() -> Self {
        Self {
            schemas: HashMap::new(),
            bindings: HashMap::new(),
            rules: Vec::new(),
        }
    }

    /// Registers a Data Structurer entity schema.
    pub fn register_schema(&mut self, schema: EntitySchema) {
        self.schemas.insert(schema.entity.clone(), schema);
    }

    /// Binds a visual Designer component to an entity schema.
    pub fn bind_component(&mut self, binding: ComponentBinding) {
        self.bindings.insert(binding.component_id.clone(), binding);
    }

    /// Attaches a Business Rule to the pipeline.
    pub fn add_business_rule(&mut self, rule: BusinessRule) {
        self.rules.push(rule);
    }

    /// Executes the full tripartite sequence:
    /// 1. Designer Mapping: extracts UI inputs into target entity fields.
    /// 2. Data Structurer Validation: enforces types and required fields against EntitySchema.
    /// 3. Business Rule Execution: evaluates IF/THEN rules, discounts, and rejection gates.
    /// 4. Optional Persistence: writes to local SQLite store if connection is provided.
    pub fn process_interaction(
        &self,
        component_id: &str,
        raw_ui_input: &Value,
        conn: Option<&Connection>,
    ) -> TripartiteResult {
        // 1. Resolve Designer Component Binding
        let binding = match self.bindings.get(component_id) {
            Some(b) => b,
            None => {
                return TripartiteResult {
                    success: false,
                    component_id: component_id.to_string(),
                    target_entity: String::new(),
                    transformed_payload: Value::Null,
                    validation_errors: vec![format!("Unbound designer component: '{}'", component_id)],
                    applied_rules: vec![],
                    business_rejections: vec![],
                    persisted_record_id: None,
                };
            }
        };

        // Map UI field keys to Entity column keys
        let mut mapped_entity_payload = Map::new();
        if let Some(input_map) = raw_ui_input.as_object() {
            for (ui_key, val) in input_map {
                let entity_col = binding.field_mappings.get(ui_key).unwrap_or(ui_key);
                mapped_entity_payload.insert(entity_col.clone(), val.clone());
            }
        }

        // 2. Data Structurer Schema Validation
        let mut validation_errors = Vec::new();
        if let Some(schema) = self.schemas.get(&binding.target_entity) {
            for field in &schema.fields {
                match mapped_entity_payload.get(&field.name) {
                    Some(val) if !val.is_null() => {
                        // Type verification
                        let type_valid = match &field.field_type {
                            FieldType::Text | FieldType::Email | FieldType::Phone | FieldType::Date => val.is_string(),
                            FieldType::Number => val.is_number(),
                            FieldType::Boolean => val.is_boolean(),
                            FieldType::Select(opts) => val.as_str().map(|s| opts.iter().any(|o| o == s)).unwrap_or(false),
                        };
                        if !type_valid {
                            validation_errors.push(format!("Field '{}' type mismatch", field.name));
                        }
                    }
                    _ => {
                        if field.required {
                            validation_errors.push(format!("Missing required field: '{}'", field.name));
                        } else if let Some(ref def) = field.default_value {
                            mapped_entity_payload.insert(field.name.clone(), def.clone());
                        }
                    }
                }
            }
        } else {
            validation_errors.push(format!("Missing schema for entity '{}'", binding.target_entity));
        }

        if !validation_errors.is_empty() {
            return TripartiteResult {
                success: false,
                component_id: component_id.to_string(),
                target_entity: binding.target_entity.clone(),
                transformed_payload: Value::Object(mapped_entity_payload),
                validation_errors,
                applied_rules: vec![],
                business_rejections: vec![],
                persisted_record_id: None,
            };
        }

        // 3. Business Logic Engine Execution
        let mut payload_value = Value::Object(mapped_entity_payload);
        let rules_engine = BusinessRulesEngine::new(self.rules.clone());
        let outcome: RuleExecutionOutcome = rules_engine.execute(&binding.target_entity, &mut payload_value);

        if !outcome.rejections.is_empty() {
            return TripartiteResult {
                success: false,
                component_id: component_id.to_string(),
                target_entity: binding.target_entity.clone(),
                transformed_payload: payload_value,
                validation_errors: vec![],
                applied_rules: outcome.applied_rules,
                business_rejections: outcome.rejections,
                persisted_record_id: None,
            };
        }

        // 4. Persistence to SQLite Store
        let mut persisted_id = None;
        if let Some(c) = conn {
            if let Ok(id) = self.persist_entity_record(c, &binding.target_entity, &payload_value) {
                persisted_id = Some(id);
            }
        }

        TripartiteResult {
            success: true,
            component_id: component_id.to_string(),
            target_entity: binding.target_entity.clone(),
            transformed_payload: payload_value,
            validation_errors: vec![],
            applied_rules: outcome.applied_rules,
            business_rejections: vec![],
            persisted_record_id: persisted_id,
        }
    }

    /// Persists the validated, rule-processed record to the SQLite store.
    fn persist_entity_record(
        &self,
        conn: &Connection,
        entity: &str,
        payload: &Value,
    ) -> SqlResult<String> {
        let record_id = uuid::Uuid::new_v4().to_string();
        let payload_str = payload.to_string();

        // Create entity table if not existing (generic entities table)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tripartite_records (
                record_id TEXT PRIMARY KEY,
                entity_name TEXT NOT NULL,
                payload_json TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "INSERT INTO tripartite_records (record_id, entity_name, payload_json, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![record_id, entity, payload_str, chrono::Utc::now().timestamp()],
        )?;

        Ok(record_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::{RuleAction, RuleCondition};
    use crate::schema::FieldDefinition;
    use serde_json::json;

    #[test]
    fn test_tripartite_pipeline_end_to_end_success() {
        let mut pipeline = TripartitePipeline::new();

        // 1. Data Structurer: Register "products" Entity Schema
        let product_schema = EntitySchema::new("products", "Είδη Αποθήκης")
            .with_field(FieldDefinition {
                name: "sku".to_string(),
                label: "Κωδικός SKU".to_string(),
                field_type: FieldType::Text,
                required: true,
                default_value: None,
            })
            .with_field(FieldDefinition {
                name: "price".to_string(),
                label: "Τιμή Χονδρικής".to_string(),
                field_type: FieldType::Number,
                required: true,
                default_value: None,
            })
            .with_field(FieldDefinition {
                name: "stock".to_string(),
                label: "Απόθεμα".to_string(),
                field_type: FieldType::Number,
                required: false,
                default_value: Some(json!(0)),
            });
        pipeline.register_schema(product_schema);

        // 2. Designer: Bind visual form inputs to the "products" entity
        let form_binding = ComponentBinding::new("designer_product_card", "products", UiTriggerEvent::OnSubmit)
            .with_field("ui_item_code", "sku")
            .with_field("ui_retail_price", "price");
        pipeline.bind_component(form_binding);

        // 3. Business Engine: Attach a business rule (10% discount if price > 100)
        let discount_rule = BusinessRule {
            id: "RULE-BULK-DISC".to_string(),
            name: "Εμπορική Έκπτωση 10%".to_string(),
            entity_type: "products".to_string(),
            condition: RuleCondition::FieldGreaterThan {
                field: "price".to_string(),
                value: 100.0,
            },
            actions: vec![RuleAction::ApplyDiscountPercentage {
                field: "price".to_string(),
                percentage: 10.0,
            }],
            is_active: true,
        };
        pipeline.add_business_rule(discount_rule);

        // 4. Execution: Simulate Designer Form Submit
        let conn = Connection::open_in_memory().unwrap();
        let ui_input = json!({
            "ui_item_code": "PROD-101",
            "ui_retail_price": 200.0
        });

        let result = pipeline.process_interaction("designer_product_card", &ui_input, Some(&conn));

        assert!(result.success, "Pipeline execution should succeed");
        assert_eq!(result.target_entity, "products");
        assert_eq!(result.applied_rules.len(), 1);
        assert_eq!(result.applied_rules[0], "Εμπορική Έκπτωση 10%");

        // Price discounted from 200.0 by 10% -> 180.0
        let final_price = result.transformed_payload.get("price").and_then(|v| v.as_f64()).unwrap();
        assert!((final_price - 180.0).abs() < 0.01);

        // Default stock applied
        assert_eq!(result.transformed_payload.get("stock").unwrap(), &json!(0));
        assert!(result.persisted_record_id.is_some(), "Record should be stored in SQLite");
    }

    #[test]
    fn test_tripartite_pipeline_schema_validation_rejection() {
        let mut pipeline = TripartitePipeline::new();

        let schema = EntitySchema::new("customers", "Πελάτες")
            .with_field(FieldDefinition {
                name: "tin".to_string(),
                label: "ΑΦΜ".to_string(),
                field_type: FieldType::Text,
                required: true,
                default_value: None,
            });
        pipeline.register_schema(schema);

        let binding = ComponentBinding::new("customer_modal", "customers", UiTriggerEvent::OnSubmit)
            .with_field("ui_tax_id", "tin");
        pipeline.bind_component(binding);

        // Provide number instead of string for Text field
        let ui_input = json!({ "ui_tax_id": 123456789 });
        let result = pipeline.process_interaction("customer_modal", &ui_input, None);

        assert!(!result.success, "Type mismatch must fail schema validation");
        assert_eq!(result.validation_errors.len(), 1);
        assert!(result.validation_errors[0].contains("type mismatch"));
    }
}
