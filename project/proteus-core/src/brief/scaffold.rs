//! Canvas scaffold screen generation from a Client Project Brief.
//! Produces artboard specifications and UI element layouts for the design studio.

use super::types::*;

/// Generate canvas screen specifications based on a bespoke brief.
pub fn generate_scaffold_screens(brief: &ClientProjectBrief) -> Vec<ScaffoldScreenSpec> {
    let mut screens = Vec::new();
    let mut origin_x = 40.0;
    let origin_y = 60.0;
    let screen_spacing = 40.0;

    for (idx, screen_name) in brief.required_screens.iter().enumerate() {
        let screen_id = format!("frame-{}", idx + 1);
        let name_lower = screen_name.to_lowercase();

        let (width, height, elements) = if name_lower.contains("intake") || name_lower.contains("form") {
            (560.0, 520.0, build_intake_elements(brief, screen_name))
        } else if name_lower.contains("pos") || name_lower.contains("counter") || name_lower.contains("ticket") {
            (680.0, 500.0, build_pos_elements(brief, screen_name))
        } else if name_lower.contains("workflow") || name_lower.contains("kanban") {
            (720.0, 480.0, build_kanban_elements(brief, screen_name))
        } else if name_lower.contains("storefront") || name_lower.contains("catalog") {
            (640.0, 560.0, build_storefront_elements(brief, screen_name))
        } else {
            (560.0, 440.0, build_generic_elements(brief, screen_name))
        };

        screens.push(ScaffoldScreenSpec {
            screen_id,
            title: screen_name.clone(),
            width,
            height,
            pos_x: origin_x,
            pos_y: origin_y,
            elements,
        });

        origin_x += width + screen_spacing;
    }

    screens
}

fn build_intake_elements(brief: &ClientProjectBrief, title: &str) -> Vec<ScaffoldElement> {
    vec![
        ScaffoldElement {
            element_type: "Header".into(),
            label: format!("{} — {}", brief.business_name, title),
            width: 520.0,
            height: 40.0,
            x: 20.0,
            y: 20.0,
            shortcut: None,
            binding: None,
        },
        ScaffoldElement {
            element_type: "Input".into(),
            label: "Customer Name & Surname".into(),
            width: 250.0,
            height: 34.0,
            x: 20.0,
            y: 80.0,
            shortcut: None,
            binding: Some("customer_name".into()),
        },
        ScaffoldElement {
            element_type: "Input".into(),
            label: "Phone / Contact Mobile".into(),
            width: 250.0,
            height: 34.0,
            x: 290.0,
            y: 80.0,
            shortcut: None,
            binding: Some("customer_phone".into()),
        },
        ScaffoldElement {
            element_type: "Input".into(),
            label: "Equipment Model & Serial No".into(),
            width: 520.0,
            height: 34.0,
            x: 20.0,
            y: 130.0,
            shortcut: None,
            binding: Some("device_model".into()),
        },
        ScaffoldElement {
            element_type: "Input".into(),
            label: "Reported Fault & Work Instructions".into(),
            width: 520.0,
            height: 80.0,
            x: 20.0,
            y: 180.0,
            shortcut: None,
            binding: Some("fault_description".into()),
        },
        ScaffoldElement {
            element_type: "Input".into(),
            label: "Estimated Budget Quote (€)".into(),
            width: 250.0,
            height: 34.0,
            x: 20.0,
            y: 280.0,
            shortcut: None,
            binding: Some("estimated_cost".into()),
        },
        ScaffoldElement {
            element_type: "Button".into(),
            label: "Print Ticket & Save Intake (F1)".into(),
            width: 520.0,
            height: 44.0,
            x: 20.0,
            y: 340.0,
            shortcut: Some("F1".into()),
            binding: Some("action_save_and_print".into()),
        },
    ]
}

fn build_pos_elements(brief: &ClientProjectBrief, _title: &str) -> Vec<ScaffoldElement> {
    vec![
        ScaffoldElement {
            element_type: "Header".into(),
            label: format!("{} — POS Counter", brief.business_name),
            width: 640.0,
            height: 36.0,
            x: 20.0,
            y: 20.0,
            shortcut: None,
            binding: None,
        },
        ScaffoldElement {
            element_type: "Input".into(),
            label: "Scan Barcode or Search Item (F3)".into(),
            width: 380.0,
            height: 34.0,
            x: 20.0,
            y: 70.0,
            shortcut: Some("F3".into()),
            binding: Some("barcode_search".into()),
        },
        ScaffoldElement {
            element_type: "Table".into(),
            label: "Order Lines (SKU • Item • Qty • Unit Price • Total)".into(),
            width: 380.0,
            height: 280.0,
            x: 20.0,
            y: 120.0,
            shortcut: None,
            binding: Some("cart_items".into()),
        },
        ScaffoldElement {
            element_type: "Card".into(),
            label: "Total Payable: €0.00".into(),
            width: 240.0,
            height: 100.0,
            x: 420.0,
            y: 70.0,
            shortcut: None,
            binding: Some("total_amount".into()),
        },
        ScaffoldElement {
            element_type: "Button".into(),
            label: "Cash & Open Drawer (F1)".into(),
            width: 240.0,
            height: 40.0,
            x: 420.0,
            y: 190.0,
            shortcut: Some("F1".into()),
            binding: Some("pay_cash".into()),
        },
        ScaffoldElement {
            element_type: "Button".into(),
            label: "Card / POS Terminal (F2)".into(),
            width: 240.0,
            height: 40.0,
            x: 420.0,
            y: 240.0,
            shortcut: Some("F2".into()),
            binding: Some("pay_card".into()),
        },
        ScaffoldElement {
            element_type: "Button".into(),
            label: "Hold Ticket / Suspend (F4)".into(),
            width: 240.0,
            height: 36.0,
            x: 420.0,
            y: 290.0,
            shortcut: Some("F4".into()),
            binding: Some("hold_ticket".into()),
        },
    ]
}

fn build_kanban_elements(brief: &ClientProjectBrief, title: &str) -> Vec<ScaffoldElement> {
    vec![
        ScaffoldElement {
            element_type: "Header".into(),
            label: format!("{} — {}", brief.business_name, title),
            width: 680.0,
            height: 36.0,
            x: 20.0,
            y: 20.0,
            shortcut: None,
            binding: None,
        },
        ScaffoldElement {
            element_type: "Card".into(),
            label: "1. Received (Παραλαβή)".into(),
            width: 150.0,
            height: 380.0,
            x: 20.0,
            y: 70.0,
            shortcut: None,
            binding: Some("status_received".into()),
        },
        ScaffoldElement {
            element_type: "Card".into(),
            label: "2. In Progress (Σε Εξέλιξη)".into(),
            width: 150.0,
            height: 380.0,
            x: 190.0,
            y: 70.0,
            shortcut: None,
            binding: Some("status_in_progress".into()),
        },
        ScaffoldElement {
            element_type: "Card".into(),
            label: "3. Ready (Έτοιμο)".into(),
            width: 150.0,
            height: 380.0,
            x: 360.0,
            y: 70.0,
            shortcut: None,
            binding: Some("status_ready".into()),
        },
        ScaffoldElement {
            element_type: "Card".into(),
            label: "4. Delivered (Παραδόθηκε)".into(),
            width: 150.0,
            height: 380.0,
            x: 530.0,
            y: 70.0,
            shortcut: None,
            binding: Some("status_delivered".into()),
        },
    ]
}

fn build_storefront_elements(brief: &ClientProjectBrief, _title: &str) -> Vec<ScaffoldElement> {
    vec![
        ScaffoldElement {
            element_type: "Header".into(),
            label: format!("{} • Official Storefront", brief.business_name),
            width: 600.0,
            height: 48.0,
            x: 20.0,
            y: 20.0,
            shortcut: None,
            binding: None,
        },
        ScaffoldElement {
            element_type: "Card".into(),
            label: format!("Hero Banner • {}", brief.business_nature),
            width: 600.0,
            height: 140.0,
            x: 20.0,
            y: 80.0,
            shortcut: None,
            binding: None,
        },
        ScaffoldElement {
            element_type: "Table".into(),
            label: "Featured Catalog & Product Grid".into(),
            width: 600.0,
            height: 240.0,
            x: 20.0,
            y: 240.0,
            shortcut: None,
            binding: Some("featured_products".into()),
        },
        ScaffoldElement {
            element_type: "Button".into(),
            label: "View Cart & Checkout".into(),
            width: 200.0,
            height: 40.0,
            x: 420.0,
            y: 500.0,
            shortcut: None,
            binding: Some("open_checkout".into()),
        },
    ]
}

fn build_generic_elements(brief: &ClientProjectBrief, title: &str) -> Vec<ScaffoldElement> {
    vec![
        ScaffoldElement {
            element_type: "Header".into(),
            label: format!("{} — {}", brief.business_name, title),
            width: 520.0,
            height: 36.0,
            x: 20.0,
            y: 20.0,
            shortcut: None,
            binding: None,
        },
        ScaffoldElement {
            element_type: "Table".into(),
            label: format!("{} Data Records", title),
            width: 520.0,
            height: 300.0,
            x: 20.0,
            y: 70.0,
            shortcut: None,
            binding: Some("entity_records".into()),
        },
        ScaffoldElement {
            element_type: "Button".into(),
            label: "Add New Record".into(),
            width: 160.0,
            height: 34.0,
            x: 380.0,
            y: 390.0,
            shortcut: None,
            binding: Some("action_create".into()),
        },
    ]
}
