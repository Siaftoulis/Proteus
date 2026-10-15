//! End-to-End Sovereign Tripartite Pipeline Integration Harness
//!
//! Authentically verifies the complete sequence connecting:
//! 1. Designer Layer: Visual Component Bindings & UI Trigger Events.
//! 2. Data Structurer Layer: Entity Schemas, Field Types & Required Constraints.
//! 3. Business Engine Layer: Reactive Business Rules, Dynamic Discounts & Rejection Gates.
//! 4. Sovereign SQLite Persistence: Atomic insertion and audit traceability in `tripartite_records`.

use rusqlite::Connection;
use serde_json::json;

use proteus_core::rules::{BusinessRule, RuleAction, RuleCondition};
use proteus_core::schema::{EntitySchema, FieldDefinition, FieldType};
use proteus_core::tripartite::{ComponentBinding, TripartitePipeline, UiTriggerEvent};

#[test]
fn test_tripartite_omnichannel_retail_checkout_pipeline() {
    let conn = Connection::open_in_memory().expect("Must initialize in-memory SQLite store");
    proteus_core::apply_storage_tuning(&conn).expect("Storage tuning must succeed");

    let mut pipeline = TripartitePipeline::new();

    // ─────────────────────────────────────────────────────────────────────────
    // 1. Data Structurer: Register "retail_invoices" Entity Schema
    // ─────────────────────────────────────────────────────────────────────────
    let invoice_schema = EntitySchema::new("retail_invoices", "Τιμολόγια Λιανικής Πώλησης")
        .with_field(FieldDefinition {
            name: "invoice_number".into(),
            label: "Αριθμός Παραστατικού".into(),
            field_type: FieldType::Text,
            required: true,
            default_value: None,
        })
        .with_field(FieldDefinition {
            name: "customer_name".into(),
            label: "Ονοματεπώνυμο Πελάτη".into(),
            field_type: FieldType::Text,
            required: true,
            default_value: None,
        })
        .with_field(FieldDefinition {
            name: "gross_amount".into(),
            label: "Συνολική Αξία (€)".into(),
            field_type: FieldType::Number,
            required: true,
            default_value: None,
        })
        .with_field(FieldDefinition {
            name: "payment_method".into(),
            label: "Τρόπος Πληρωμής".into(),
            field_type: FieldType::Select(vec!["Cash".into(), "Card_POS".into(), "Bank_Transfer".into()]),
            required: true,
            default_value: None,
        })
        .with_field(FieldDefinition {
            name: "mydata_status".into(),
            label: "Κατάσταση myDATA".into(),
            field_type: FieldType::Text,
            required: false,
            default_value: Some(json!("PENDING")),
        });
    pipeline.register_schema(invoice_schema);

    // ─────────────────────────────────────────────────────────────────────────
    // 2. Designer: Bind Touch POS "Pay Now" Button to "retail_invoices"
    // ─────────────────────────────────────────────────────────────────────────
    let checkout_btn_binding = ComponentBinding::new("btn_pos_pay_now", "retail_invoices", UiTriggerEvent::OnButtonClick)
        .with_field("ui_doc_num", "invoice_number")
        .with_field("ui_client", "customer_name")
        .with_field("ui_total", "gross_amount")
        .with_field("ui_payment", "payment_method");
    pipeline.bind_component(checkout_btn_binding);

    // ─────────────────────────────────────────────────────────────────────────
    // 3. Business Engine: Attach Reactive Business Rules
    // Rule A: VIP Volume Discount (5% if gross_amount > 200.0)
    // Rule B: High-Value Statutory Flag (Set mydata_status = "IMMEDIATE_DISPATCH" if gross_amount > 500.0)
    // ─────────────────────────────────────────────────────────────────────────
    let volume_discount_rule = BusinessRule {
        id: "RULE-POS-DISC-5".into(),
        name: "Εμπορική Έκπτωση 5%".into(),
        entity_type: "retail_invoices".into(),
        condition: RuleCondition::FieldGreaterThan {
            field: "gross_amount".into(),
            value: 200.0,
        },
        actions: vec![RuleAction::ApplyDiscountPercentage {
            field: "gross_amount".into(),
            percentage: 5.0,
        }],
        is_active: true,
    };
    pipeline.add_business_rule(volume_discount_rule);

    let high_value_rule = BusinessRule {
        id: "RULE-MYDATA-HIGH".into(),
        name: "Άμεση Διαβίβαση myDATA".into(),
        entity_type: "retail_invoices".into(),
        condition: RuleCondition::FieldGreaterThan {
            field: "gross_amount".into(),
            value: 500.0,
        },
        actions: vec![RuleAction::SetField {
            field: "mydata_status".into(),
            value: json!("IMMEDIATE_DISPATCH"),
        }],
        is_active: true,
    };
    pipeline.add_business_rule(high_value_rule);

    // ─────────────────────────────────────────────────────────────────────────
    // 4. Execution: Valid High-Value Checkout Interaction
    // ─────────────────────────────────────────────────────────────────────────
    let raw_ui_input = json!({
        "ui_doc_num": "INV-2026-0042",
        "ui_client": "Γεώργιος Παπαδόπουλος",
        "ui_total": 600.0,
        "ui_payment": "Card_POS"
    });

    let outcome = pipeline.process_interaction("btn_pos_pay_now", &raw_ui_input, Some(&conn));

    assert!(outcome.success, "Tripartite execution must succeed for valid checkout");
    assert_eq!(outcome.component_id, "btn_pos_pay_now");
    assert_eq!(outcome.target_entity, "retail_invoices");
    assert!(outcome.validation_errors.is_empty(), "Schema validation must pass with zero errors");

    // Verify Business Rule Transformations
    // 600.0 with 5% discount -> 570.0 (still > 500, so high-value rule also applies)
    let payload = outcome.transformed_payload.as_object().expect("Payload must be a JSON object");
    let calculated_gross = payload.get("gross_amount").and_then(|v| v.as_f64()).expect("Gross must be f64");
    assert!((calculated_gross - 570.0).abs() < 0.01, "Expected discounted gross 570.0, got {}", calculated_gross);

    let mydata_status = payload.get("mydata_status").and_then(|v| v.as_str()).expect("Status must be str");
    assert_eq!(mydata_status, "IMMEDIATE_DISPATCH", "Statutory rule must set immediate dispatch");

    assert!(outcome.applied_rules.contains(&"Εμπορική Έκπτωση 5%".into()));
    assert!(outcome.applied_rules.contains(&"Άμεση Διαβίβαση myDATA".into()));

    // ─────────────────────────────────────────────────────────────────────────
    // 5. SQLite Persistence Verification
    // ─────────────────────────────────────────────────────────────────────────
    let persisted_id = outcome.persisted_record_id.expect("Record ID must be generated");
    let mut stmt = conn.prepare(
        "SELECT entity_name, payload_json FROM tripartite_records WHERE record_id = ?1",
    ).expect("Query prepare must succeed");

    let (saved_entity, saved_json): (String, String) = stmt.query_row(
        [persisted_id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    ).expect("Record must be saved in tripartite_records table");

    assert_eq!(saved_entity, "retail_invoices");
    assert!(saved_json.contains("INV-2026-0042"));
    assert!(saved_json.contains("570"));
    assert!(saved_json.contains("IMMEDIATE_DISPATCH"));
}

#[test]
fn test_tripartite_schema_validation_rejection_prevents_db_write() {
    let conn = Connection::open_in_memory().expect("Must initialize in-memory SQLite store");
    let mut pipeline = TripartitePipeline::new();

    let task_schema = EntitySchema::new("tasks", "Εργασίες Συνεργείου")
        .with_field(FieldDefinition {
            name: "title".into(),
            label: "Τίτλος Εργασίας".into(),
            field_type: FieldType::Text,
            required: true,
            default_value: None,
        })
        .with_field(FieldDefinition {
            name: "estimated_hours".into(),
            label: "Εκτιμώμενες Ώρες".into(),
            field_type: FieldType::Number,
            required: true,
            default_value: None,
        });
    pipeline.register_schema(task_schema);

    let binding = ComponentBinding::new("form_new_task", "tasks", UiTriggerEvent::OnSubmit)
        .with_field("ui_task_title", "title")
        .with_field("ui_task_hours", "estimated_hours");
    pipeline.bind_component(binding);

    // Missing required field: "estimated_hours" provided as string instead of number
    let invalid_input = json!({
        "ui_task_title": "Αλλαγή Ιμάντα Χρονισμού",
        "ui_task_hours": "three_hours" // Type mismatch!
    });

    let outcome = pipeline.process_interaction("form_new_task", &invalid_input, Some(&conn));

    assert!(!outcome.success, "Pipeline must fail when schema types are invalid");
    assert!(outcome.persisted_record_id.is_none(), "Database write must be aborted on validation failure");
    assert!(outcome.validation_errors.iter().any(|e| e.contains("type mismatch")), "Error must explain type mismatch");

    // Verify SQLite table remains completely untouched
    let table_exists: bool = conn.query_row(
        "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='tripartite_records'",
        [],
        |r| Ok(r.get::<_, i64>(0)? > 0),
    ).unwrap_or(false);

    if table_exists {
        let count: i64 = conn.query_row("SELECT count(*) FROM tripartite_records", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 0, "No records should be written on schema validation failure");
    }
}
