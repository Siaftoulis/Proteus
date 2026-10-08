//! SQLite persistence for Client Project Briefs.
//! Schema initialization, CRUD operations, and seed data.

use rusqlite::{params, Connection, Result};
use super::types::*;

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
