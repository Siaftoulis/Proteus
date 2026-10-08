//! Starter Package Generator from Business Questionnaire.
//! Converts questionnaire answers into fully compliant, sealed `.pr` packages
//! containing domain DDL tables, declarative views, and reactive flow triggers.

use crate::package::{PrFlowTrigger, PrManifest, PrPackage, PrSchemaBundle, PrViewLayout};
use crate::questionnaire::{BusinessQuestionnaire, TopIndustryCategory};
use chrono::Utc;

/// Generates a turnkey `.pr` starter package from questionnaire responses.
pub fn generate_starter_package_from_answers(q: &BusinessQuestionnaire) -> PrPackage {
    let now = Utc::now().to_rfc3339();
    let bundle_id = format!(
        "PRK-STARTER-{}",
        q.sub_category_id.to_uppercase().replace('_', "-")
    );

    let manifest = PrManifest {
        bundle_id: bundle_id.clone(),
        name: format!("{} Starter BOS", q.business_name),
        version: "1.0.0".to_string(),
        author_pcd_id: "PCD-AUTOSCAFFOLD-SYSTEM".to_string(),
        target_client_license: None,
        created_at: now,
    };

    let mut ddl_statements = Vec::new();
    let mut views = Vec::new();
    let mut flows = Vec::new();

    // 1. Sector-Specific Domain Schemas & Views
    match q.top_category {
        TopIndustryCategory::TechnologyAndRepairs => {
            ddl_statements.push(
                "CREATE TABLE IF NOT EXISTS starter_service_tickets (
                    id TEXT PRIMARY KEY,
                    ticket_number INTEGER,
                    customer_name TEXT NOT NULL,
                    customer_phone TEXT NOT NULL,
                    device_model TEXT NOT NULL,
                    fault_description TEXT NOT NULL,
                    status TEXT NOT NULL,
                    estimated_cost REAL NOT NULL,
                    created_at TEXT NOT NULL
                );"
                .to_string(),
            );
            views.push(PrViewLayout {
                view_id: "view_service_kanban".into(),
                name: "Ροή Επισκευών (Kanban)".into(),
                view_type: "kanban_pipeline".into(),
                layout_json: r#"{"columns":["received","in_progress","waiting_parts","ready","delivered"]}"#.into(),
            });
        }
        TopIndustryCategory::RetailAndCommerce => {
            ddl_statements.push(
                "CREATE TABLE IF NOT EXISTS starter_retail_items (
                    id TEXT PRIMARY KEY,
                    barcode TEXT UNIQUE,
                    name TEXT NOT NULL,
                    price REAL NOT NULL,
                    stock INTEGER NOT NULL,
                    vat_rate REAL NOT NULL
                );"
                .to_string(),
            );
            views.push(PrViewLayout {
                view_id: "view_retail_pos".into(),
                name: "Ταμείο POS & Barcode".into(),
                view_type: "pos_cash_desk".into(),
                layout_json: r#"{"layout":"touch_grid_with_numpad","barcode_listener":true}"#.into(),
            });
        }
        TopIndustryCategory::HealthcareAndMedical => {
            ddl_statements.push(
                "CREATE TABLE IF NOT EXISTS starter_medical_patients (
                    id TEXT PRIMARY KEY,
                    amka TEXT UNIQUE,
                    full_name TEXT NOT NULL,
                    phone TEXT NOT NULL,
                    medical_notes TEXT,
                    created_at TEXT NOT NULL
                );"
                .to_string(),
            );
            views.push(PrViewLayout {
                view_id: "view_patient_records".into(),
                name: "Καρτέλες Ασθενών".into(),
                view_type: "table_grid".into(),
                layout_json: r#"{"table":"starter_medical_patients","search_fields":["amka","full_name"]}"#.into(),
            });
        }
        TopIndustryCategory::HospitalityAndFood => {
            ddl_statements.push(
                "CREATE TABLE IF NOT EXISTS starter_hospitality_orders (
                    id TEXT PRIMARY KEY,
                    table_number INTEGER,
                    order_items TEXT NOT NULL,
                    total_amount REAL NOT NULL,
                    status TEXT NOT NULL,
                    created_at TEXT NOT NULL
                );"
                .to_string(),
            );
            views.push(PrViewLayout {
                view_id: "view_table_service".into(),
                name: "Παραγγελίες & Τραπέζια".into(),
                view_type: "table_map".into(),
                layout_json: r#"{"mode":"tables_with_takeaway"}"#.into(),
            });
        }
        TopIndustryCategory::ServicesAndOffices => {
            ddl_statements.push(
                "CREATE TABLE IF NOT EXISTS starter_client_matters (
                    id TEXT PRIMARY KEY,
                    client_name TEXT NOT NULL,
                    afm TEXT,
                    matter_type TEXT NOT NULL,
                    status TEXT NOT NULL,
                    retainer_fee REAL NOT NULL,
                    created_at TEXT NOT NULL
                );"
                .to_string(),
            );
            views.push(PrViewLayout {
                view_id: "view_matters_dossier".into(),
                name: "Φάκελοι Υποθέσεων".into(),
                view_type: "table_grid".into(),
                layout_json: r#"{"table":"starter_client_matters","filterable":true}"#.into(),
            });
        }
        TopIndustryCategory::PersonalCareAndWellness => {
            ddl_statements.push(
                "CREATE TABLE IF NOT EXISTS starter_wellness_appointments (
                    id TEXT PRIMARY KEY,
                    client_name TEXT NOT NULL,
                    client_phone TEXT NOT NULL,
                    service_type TEXT NOT NULL,
                    scheduled_at TEXT NOT NULL,
                    duration_minutes INTEGER NOT NULL,
                    price REAL NOT NULL,
                    status TEXT NOT NULL
                );"
                .to_string(),
            );
            views.push(PrViewLayout {
                view_id: "view_appointments_calendar".into(),
                name: "Ραντεβού & Πελατολόγιο".into(),
                view_type: "calendar_schedule".into(),
                layout_json: r#"{"mode":"day_week_agenda"}"#.into(),
            });
        }
        TopIndustryCategory::CraftsAndManufacturing => {
            ddl_statements.push(
                "CREATE TABLE IF NOT EXISTS starter_production_orders (
                    id TEXT PRIMARY KEY,
                    order_code TEXT UNIQUE,
                    item_description TEXT NOT NULL,
                    quantity INTEGER NOT NULL,
                    raw_materials_cost REAL NOT NULL,
                    target_delivery_date TEXT NOT NULL,
                    stage TEXT NOT NULL
                );"
                .to_string(),
            );
            views.push(PrViewLayout {
                view_id: "view_production_pipeline".into(),
                name: "Εντολές Παραγωγής & Στάδια".into(),
                view_type: "kanban_pipeline".into(),
                layout_json: r#"{"columns":["pending","cutting","assembly","finishing","delivered"]}"#.into(),
            });
        }
    }

    // 2. Auxiliary Operational Features
    if q.workflow.has_spare_parts_inventory {
        ddl_statements.push(
            "CREATE TABLE IF NOT EXISTS starter_inventory_parts (
                id TEXT PRIMARY KEY,
                part_sku TEXT UNIQUE,
                description TEXT NOT NULL,
                quantity INTEGER NOT NULL,
                unit_cost REAL NOT NULL
            );"
            .to_string(),
        );
        views.push(PrViewLayout {
            view_id: "view_inventory_parts".into(),
            name: "Αποθήκη & Ανταλλακτικά".into(),
            view_type: "table_grid".into(),
            layout_json: r#"{"table":"starter_inventory_parts"}"#.into(),
        });
    }

    if q.workflow.has_intake_voucher {
        flows.push(PrFlowTrigger {
            id: "flow_print_intake_voucher".into(),
            trigger_event: "on_record_intake".into(),
            action_type: "escpos_print_voucher".into(),
            config_json: r#"{"printer_width_mm":80,"cut_paper":true}"#.into(),
        });
    }

    if q.workflow.has_mydata_invoicing {
        flows.push(PrFlowTrigger {
            id: "flow_mydata_transmit".into(),
            trigger_event: "on_payment_complete".into(),
            action_type: "aade_mydata_transmit".into(),
            config_json: r#"{"document_type":"11.1","auto_retry":true}"#.into(),
        });
    }

    if q.workflow.has_customer_sms_notifications {
        flows.push(PrFlowTrigger {
            id: "flow_sms_alert".into(),
            trigger_event: "on_status_ready".into(),
            action_type: "send_sms_notification".into(),
            config_json: r#"{"template":"Η επισκευή σας είναι έτοιμη για παραλαβή!"}"#.into(),
        });
    }

    PrPackage {
        manifest,
        schema: PrSchemaBundle {
            ddl_statements,
            entity_schemas: Vec::new(),
        },
        views,
        flows,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::questionnaire::{BusinessScale, DesignStrategy, WorkflowRequirements};

    #[test]
    fn test_generate_starter_package_repair_shop() {
        let mut q = BusinessQuestionnaire::new(
            "TechFix Patras",
            TopIndustryCategory::TechnologyAndRepairs,
            "tech_smartphones_pc",
        );
        q.workflow = WorkflowRequirements {
            has_intake_voucher: true,
            has_serial_imei_tracking: true,
            has_spare_parts_inventory: true,
            has_technician_labor_rates: true,
            has_customer_sms_notifications: true,
            has_mydata_invoicing: true,
            has_eftpos_interlock: true,
            has_ergani_work_card: true,
            custom_notes: "Επισκευές αυθημερόν".into(),
        };
        q.scale = BusinessScale {
            employee_count: 3,
            workstation_count: 2,
            monthly_volume_bracket: "100-300".into(),
            branch_count: 1,
        };
        q.strategy = DesignStrategy::SoloFreeStarter;

        let pkg = generate_starter_package_from_answers(&q);

        assert_eq!(pkg.manifest.bundle_id, "PRK-STARTER-TECH-SMARTPHONES-PC");
        assert_eq!(pkg.manifest.name, "TechFix Patras Starter BOS");
        assert_eq!(pkg.schema.ddl_statements.len(), 2); // service tickets + inventory parts
        assert!(pkg.schema.ddl_statements[0].contains("starter_service_tickets"));
        assert!(pkg.schema.ddl_statements[1].contains("starter_inventory_parts"));
        assert_eq!(pkg.views.len(), 2); // kanban + inventory parts
        assert_eq!(pkg.flows.len(), 3); // voucher print + mydata + sms alert

        let bytes = pkg.to_bytes().unwrap();
        assert!(bytes.starts_with(b"PRPK"));
    }

    #[test]
    fn test_generate_starter_package_retail_pos() {
        let q = BusinessQuestionnaire::new(
            "Athens Boutique",
            TopIndustryCategory::RetailAndCommerce,
            "retail_fashion",
        );

        let pkg = generate_starter_package_from_answers(&q);
        assert_eq!(pkg.manifest.bundle_id, "PRK-STARTER-RETAIL-FASHION");
        assert_eq!(pkg.schema.ddl_statements.len(), 1);
        assert!(pkg.schema.ddl_statements[0].contains("starter_retail_items"));
        assert_eq!(pkg.views.len(), 1);
        assert_eq!(pkg.views[0].view_type, "pos_cash_desk");
    }
}
