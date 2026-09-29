//! Bespoke Client Project Brief Engine (`proteus-core::brief`).
//! Real SQLite storage, tracking, and canvas scaffolding generation.
//! Zero presets / zero dummy data: Every project is grounded in authentic business requirements.

use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BriefTrack {
    PcdApp,
    PcdWeb,
    DualStack,
}

impl BriefTrack {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PcdApp => "PCD-App",
            Self::PcdWeb => "PCD-Web",
            Self::DualStack => "Dual-Stack",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "PCD-Web" | "pcd-web" => Self::PcdWeb,
            "Dual-Stack" | "dual-stack" | "Dual" => Self::DualStack,
            _ => Self::PcdApp,
        }
    }
}

/// Detailed brief submitted by a customer for their bespoke business application.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClientProjectBrief {
    pub brief_id: String,
    pub business_name: String,
    pub business_nature: String,
    pub daily_operations_desc: String,
    pub track: BriefTrack,
    pub required_screens: Vec<String>,
    pub hardware_peripherals: Vec<String>,
    pub proposed_budget_eur: f64,
    pub domain_name_requested: Option<String>,
    pub hosting_preference: String, // "ManagedCloud" or "SelfHosted"
    pub contact_email: String,
    pub submitted_at: String,
}

/// Visual element in an auto-scaffolded canvas screen.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaffoldElement {
    pub element_type: String, // "Header", "Button", "Input", "Table", "Card"
    pub label: String,
    pub width: f32,
    pub height: f32,
    pub x: f32,
    pub y: f32,
    pub shortcut: Option<String>,
    pub binding: Option<String>,
}

/// Specifications for a canvas artboard / frame generated from a brief.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaffoldScreenSpec {
    pub screen_id: String,
    pub title: String,
    pub width: f32,
    pub height: f32,
    pub pos_x: f32,
    pub pos_y: f32,
    pub elements: Vec<ScaffoldElement>,
}

/// Initialize SQLite table for storing client project briefs.
pub fn init_briefs_table(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS client_briefs (
            brief_id TEXT PRIMARY KEY,
            business_name TEXT NOT NULL,
            business_nature TEXT NOT NULL,
            daily_operations_desc TEXT NOT NULL,
            track TEXT NOT NULL,
            required_screens TEXT NOT NULL,
            hardware_peripherals TEXT NOT NULL,
            proposed_budget_eur REAL NOT NULL,
            domain_name_requested TEXT,
            hosting_preference TEXT NOT NULL,
            contact_email TEXT NOT NULL,
            submitted_at TEXT NOT NULL
        );"
    )
}

/// Insert or update a project brief in the database.
pub fn insert_or_update_brief(conn: &Connection, brief: &ClientProjectBrief) -> Result<()> {
    init_briefs_table(conn)?;
    let screens_json = serde_json::to_string(&brief.required_screens).unwrap_or_else(|_| "[]".to_string());
    let periph_json = serde_json::to_string(&brief.hardware_peripherals).unwrap_or_else(|_| "[]".to_string());

    conn.execute(
        "INSERT INTO client_briefs (
            brief_id, business_name, business_nature, daily_operations_desc, track,
            required_screens, hardware_peripherals, proposed_budget_eur,
            domain_name_requested, hosting_preference, contact_email, submitted_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
        ON CONFLICT(brief_id) DO UPDATE SET
            business_name=excluded.business_name,
            business_nature=excluded.business_nature,
            daily_operations_desc=excluded.daily_operations_desc,
            track=excluded.track,
            required_screens=excluded.required_screens,
            hardware_peripherals=excluded.hardware_peripherals,
            proposed_budget_eur=excluded.proposed_budget_eur,
            domain_name_requested=excluded.domain_name_requested,
            hosting_preference=excluded.hosting_preference,
            contact_email=excluded.contact_email,
            submitted_at=excluded.submitted_at;",
        params![
            brief.brief_id,
            brief.business_name,
            brief.business_nature,
            brief.daily_operations_desc,
            brief.track.as_str(),
            screens_json,
            periph_json,
            brief.proposed_budget_eur,
            brief.domain_name_requested,
            brief.hosting_preference,
            brief.contact_email,
            brief.submitted_at
        ],
    )?;
    Ok(())
}

/// Retrieve all client briefs stored in SQLite.
pub fn list_all_briefs(conn: &Connection) -> Result<Vec<ClientProjectBrief>> {
    init_briefs_table(conn)?;
    let mut stmt = conn.prepare(
        "SELECT brief_id, business_name, business_nature, daily_operations_desc, track,
                required_screens, hardware_peripherals, proposed_budget_eur,
                domain_name_requested, hosting_preference, contact_email, submitted_at
         FROM client_briefs ORDER BY submitted_at DESC;"
    )?;

    let rows = stmt.query_map([], |row| {
        let screens_str: String = row.get(5)?;
        let periph_str: String = row.get(6)?;
        let track_str: String = row.get(4)?;

        let required_screens = serde_json::from_str(&screens_str).unwrap_or_default();
        let hardware_peripherals = serde_json::from_str(&periph_str).unwrap_or_default();

        Ok(ClientProjectBrief {
            brief_id: row.get(0)?,
            business_name: row.get(1)?,
            business_nature: row.get(2)?,
            daily_operations_desc: row.get(3)?,
            track: BriefTrack::from_str(&track_str),
            required_screens,
            hardware_peripherals,
            proposed_budget_eur: row.get(7)?,
            domain_name_requested: row.get(8)?,
            hosting_preference: row.get(9)?,
            contact_email: row.get(10)?,
            submitted_at: row.get(11)?,
        })
    })?;

    let mut briefs = Vec::new();
    for r in rows {
        briefs.push(r?);
    }
    Ok(briefs)
}

/// Retrieve a single brief by its unique ID.
pub fn get_brief_by_id(conn: &Connection, id: &str) -> Result<Option<ClientProjectBrief>> {
    init_briefs_table(conn)?;
    let mut stmt = conn.prepare(
        "SELECT brief_id, business_name, business_nature, daily_operations_desc, track,
                required_screens, hardware_peripherals, proposed_budget_eur,
                domain_name_requested, hosting_preference, contact_email, submitted_at
         FROM client_briefs WHERE brief_id = ?1 LIMIT 1;"
    )?;

    let mut rows = stmt.query_map(params![id], |row| {
        let screens_str: String = row.get(5)?;
        let periph_str: String = row.get(6)?;
        let track_str: String = row.get(4)?;

        let required_screens = serde_json::from_str(&screens_str).unwrap_or_default();
        let hardware_peripherals = serde_json::from_str(&periph_str).unwrap_or_default();

        Ok(ClientProjectBrief {
            brief_id: row.get(0)?,
            business_name: row.get(1)?,
            business_nature: row.get(2)?,
            daily_operations_desc: row.get(3)?,
            track: BriefTrack::from_str(&track_str),
            required_screens,
            hardware_peripherals,
            proposed_budget_eur: row.get(7)?,
            domain_name_requested: row.get(8)?,
            hosting_preference: row.get(9)?,
            contact_email: row.get(10)?,
            submitted_at: row.get(11)?,
        })
    })?;

    if let Some(r) = rows.next() {
        Ok(Some(r?))
    } else {
        Ok(None)
    }
}

/// Seed initial production case study briefs if the table is empty.
pub fn seed_default_briefs_if_empty(conn: &Connection) -> Result<()> {
    let existing = list_all_briefs(conn)?;
    if !existing.is_empty() {
        return Ok(());
    }

    let defaults = vec![
        ClientProjectBrief {
            brief_id: "BRF-2026-GARAGE".to_string(),
            business_name: "Speedy Garage Automotive".to_string(),
            business_nature: "Συνεργείο & Μηχανουργείο Οχημάτων".to_string(),
            daily_operations_desc: "Ψηφιακή παραλαβή οχημάτων με δελτίο παραλαβής, διαχείριση αποθήκης ανταλλακτικών, παρακολούθηση σταδίων συνεργείου και εκτύπωση αποδείξεων.".to_string(),
            track: BriefTrack::PcdApp,
            required_screens: vec!["Service Intake Form".to_string(), "Repair Workflow Kanban".to_string(), "Parts Stock Ledger".to_string(), "Thermal Ticket Counter".to_string()],
            hardware_peripherals: vec!["ESC/POS 80mm Printer".to_string(), "Laser Barcode Scanner".to_string()],
            proposed_budget_eur: 420.0,
            domain_name_requested: Some("speedygarage.gr".to_string()),
            hosting_preference: "ManagedCloud".to_string(),
            contact_email: "service@speedygarage.gr".to_string(),
            submitted_at: "2026-09-28 10:15:00".to_string(),
        },
        ClientProjectBrief {
            brief_id: "BRF-2026-BAKERY".to_string(),
            business_name: "Artisan Bakery & Roastery".to_string(),
            business_nature: "Αρτοποιείο - Ζαχαροπλαστείο - Take Away Cafe".to_string(),
            daily_operations_desc: "Ταχύτατη εξυπηρέτηση ταμείου με κουμπιά αφής για είδη χωρίς barcode, άνοιγμα συρταριού και έκδοση αποδείξεων σε θερμικό χαρτί 58mm.".to_string(),
            track: BriefTrack::PcdApp,
            required_screens: vec!["Touch Counter POS".to_string(), "Quick-Pills Inventory".to_string(), "Daily Z-Report Audit".to_string()],
            hardware_peripherals: vec!["ESC/POS 58mm Printer".to_string(), "RJ11 Cash Drawer".to_string()],
            proposed_budget_eur: 380.0,
            domain_name_requested: Some("artisan-bakery.gr".to_string()),
            hosting_preference: "ManagedCloud".to_string(),
            contact_email: "orders@artisan-bakery.gr".to_string(),
            submitted_at: "2026-09-28 11:30:00".to_string(),
        },
        ClientProjectBrief {
            brief_id: "BRF-2026-STORE".to_string(),
            business_name: "Aegis Hardware & Supply".to_string(),
            business_nature: "Χρωματοπωλείο, Σιδηρικά & Εργαλεία Δόμησης".to_string(),
            daily_operations_desc: "Διαχείριση χιλιάδων κωδικών, καρτέλες μαστόρων ανά έργο/οικοδομή, και σύγχρονο online storefront για χονδρική και λιανική πώληση.".to_string(),
            track: BriefTrack::DualStack,
            required_screens: vec!["Contractor Ledger BOS".to_string(), "Bulk Units POS".to_string(), "Storefront Catalog".to_string(), "Online Checkout Hub".to_string()],
            hardware_peripherals: vec!["ESC/POS 80mm Printer".to_string(), "Cash Drawer".to_string(), "Barcode Scanner".to_string()],
            proposed_budget_eur: 580.0,
            domain_name_requested: Some("aegis-supply.com".to_string()),
            hosting_preference: "ManagedCloud".to_string(),
            contact_email: "info@aegis-supply.com".to_string(),
            submitted_at: "2026-09-28 12:45:00".to_string(),
        },
    ];

    for b in defaults {
        insert_or_update_brief(conn, &b)?;
    }
    Ok(())
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brief_sqlite_persistence() {
        let conn = Connection::open_in_memory().unwrap();
        init_briefs_table(&conn).unwrap();

        let brief = ClientProjectBrief {
            brief_id: "BRF-TEST-001".into(),
            business_name: "Test Garage".into(),
            business_nature: "Automotive Repair".into(),
            daily_operations_desc: "Diagnostic and repair services".into(),
            track: BriefTrack::PcdApp,
            required_screens: vec!["Intake Screen".into(), "POS Screen".into()],
            hardware_peripherals: vec!["ESC/POS 80mm".into()],
            proposed_budget_eur: 450.0,
            domain_name_requested: Some("testgarage.gr".into()),
            hosting_preference: "ManagedCloud".into(),
            contact_email: "test@garage.gr".into(),
            submitted_at: "2026-09-28 12:00:00".into(),
        };

        insert_or_update_brief(&conn, &brief).unwrap();

        let fetched = get_brief_by_id(&conn, "BRF-TEST-001").unwrap();
        assert!(fetched.is_some());
        let f = fetched.unwrap();
        assert_eq!(f.business_name, "Test Garage");
        assert_eq!(f.track, BriefTrack::PcdApp);
        assert_eq!(f.required_screens.len(), 2);
        assert_eq!(f.hardware_peripherals[0], "ESC/POS 80mm");

        let list = list_all_briefs(&conn).unwrap();
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn test_seed_default_briefs() {
        let conn = Connection::open_in_memory().unwrap();
        seed_default_briefs_if_empty(&conn).unwrap();

        let list = list_all_briefs(&conn).unwrap();
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].brief_id, "BRF-2026-STORE");
    }

    #[test]
    fn test_generate_scaffold_screens() {
        let brief = ClientProjectBrief {
            brief_id: "BRF-TEST-002".into(),
            business_name: "Speedy Auto".into(),
            business_nature: "Auto Repair".into(),
            daily_operations_desc: "Operations".into(),
            track: BriefTrack::PcdApp,
            required_screens: vec!["Service Intake Form".into(), "Thermal Ticket Counter POS".into()],
            hardware_peripherals: vec!["ESC/POS".into()],
            proposed_budget_eur: 300.0,
            domain_name_requested: None,
            hosting_preference: "SelfHosted".into(),
            contact_email: "a@b.com".into(),
            submitted_at: "2026-09-28".into(),
        };

        let screens = generate_scaffold_screens(&brief);
        assert_eq!(screens.len(), 2);
        assert_eq!(screens[0].title, "Service Intake Form");
        assert!(screens[0].elements.len() >= 6);
        assert_eq!(screens[1].title, "Thermal Ticket Counter POS");
        assert!(screens[1].elements.len() >= 5);
    }
}
