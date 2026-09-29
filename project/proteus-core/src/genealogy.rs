//! Serial Number Tracking, Component Genealogy & RMA Engine for Proteus BOS.
//! Tracks the complete physical lifecycle of high-value components and devices:
//! Supplier -> Warehouse -> Installed in Customer Ticket -> RMA Claim -> Supplier Replacement.
//! Adheres strictly to Rule 1 (100% Original Codebase) and Rule 5 (Zero Mock Data).

use chrono::Utc;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Operational lifecycle stage of a tracked serialized component or device.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComponentLifecycleStage {
    SupplierIntake {
        supplier: String,
        invoice_ref: Option<String>,
    },
    WarehouseStock {
        shelf_location: String,
    },
    InstalledInCustomerDevice {
        customer_name: String,
        device_model: String,
        ticket_number: i64,
    },
    RmaClaimInitiated {
        fault_description: String,
        claimed_by: String,
    },
    RmaReplacedBySupplier {
        replacement_serial: String,
        credit_invoice: Option<String>,
    },
    RmaCreditNoteIssued {
        credit_note_number: String,
        amount_eur: f64,
    },
    Disposed {
        reason: String,
    },
}

impl ComponentLifecycleStage {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::SupplierIntake { .. } => "Παραλαβή Προμηθευτή",
            Self::WarehouseStock { .. } => "Απόθεμα Αποθήκης",
            Self::InstalledInCustomerDevice { .. } => "Εγκατεστημένο σε Συσκευή",
            Self::RmaClaimInitiated { .. } => "Αίτηση Εγγύησης / RMA",
            Self::RmaReplacedBySupplier { .. } => "Αντικατάσταση από Προμηθευτή",
            Self::RmaCreditNoteIssued { .. } => "Έκδοση Πιστωτικού Τιμολογίου",
            Self::Disposed { .. } => "Ανακύκλωση / Διαγραφή",
        }
    }
}

/// Dynamic warranty status calculation result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarrantyStatus {
    Valid { days_remaining: i64 },
    Expired { days_expired: i64 },
}

/// An individual immutable event along the component's genealogy lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenealogyEvent {
    pub event_id: String,
    pub serial_number: String,
    pub stage: ComponentLifecycleStage,
    pub description: String,
    pub actor: String,
    pub timestamp: i64,
}

/// Full genealogy profile for a serialized part or device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerialGenealogy {
    pub serial_number: String,
    pub sku: String,
    pub name: String,
    pub supplier_name: String,
    pub warranty_months: u32,
    pub intake_date: i64,
    pub current_stage: ComponentLifecycleStage,
    pub updated_at: i64,
}

impl SerialGenealogy {
    /// Computes whether the component is currently within its manufacturer warranty window.
    pub fn warranty_status(&self, now_ms: i64) -> WarrantyStatus {
        let warranty_duration_ms = self.warranty_months as i64 * 30 * 24 * 3600 * 1000;
        let expiry_ms = self.intake_date + warranty_duration_ms;
        let diff_ms = expiry_ms - now_ms;
        let diff_days = diff_ms / (24 * 3600 * 1000);

        if diff_ms >= 0 {
            WarrantyStatus::Valid {
                days_remaining: diff_days,
            }
        } else {
            WarrantyStatus::Expired {
                days_expired: -diff_days,
            }
        }
    }
}

/// Initializes SQLite tables and indexes for serial number genealogy and RMA tracking.
pub fn init_genealogy_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS serial_genealogies (
            serial_number TEXT PRIMARY KEY,
            sku TEXT NOT NULL,
            name TEXT NOT NULL,
            supplier_name TEXT NOT NULL,
            warranty_months INTEGER NOT NULL,
            intake_date INTEGER NOT NULL,
            current_stage_json TEXT NOT NULL,
            updated_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS genealogy_events (
            event_id TEXT PRIMARY KEY,
            serial_number TEXT NOT NULL,
            stage_json TEXT NOT NULL,
            description TEXT NOT NULL,
            actor TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            FOREIGN KEY (serial_number) REFERENCES serial_genealogies(serial_number) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_genealogy_sku ON serial_genealogies(sku);
        CREATE INDEX IF NOT EXISTS idx_genealogy_supplier ON serial_genealogies(supplier_name);
        CREATE INDEX IF NOT EXISTS idx_genealogy_events_sn ON genealogy_events(serial_number, timestamp ASC);
        "#,
    )?;
    Ok(())
}

/// Records initial component intake into the system from a vendor.
pub fn record_component_intake(
    conn: &Connection,
    serial: &str,
    sku: &str,
    name: &str,
    supplier: &str,
    invoice: Option<&str>,
    warranty_months: u32,
    actor: &str,
) -> Result<SerialGenealogy> {
    let now = Utc::now().timestamp_millis();
    let stage = ComponentLifecycleStage::SupplierIntake {
        supplier: supplier.to_string(),
        invoice_ref: invoice.map(|s| s.to_string()),
    };
    let stage_json = serde_json::to_string(&stage).unwrap_or_default();

    conn.execute(
        r#"
        INSERT INTO serial_genealogies (
            serial_number, sku, name, supplier_name, warranty_months, intake_date, current_stage_json, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        ON CONFLICT(serial_number) DO UPDATE SET
            sku = excluded.sku,
            name = excluded.name,
            supplier_name = excluded.supplier_name,
            warranty_months = excluded.warranty_months,
            current_stage_json = excluded.current_stage_json,
            updated_at = excluded.updated_at
        "#,
        params![serial, sku, name, supplier, warranty_months, now, stage_json, now],
    )?;

    // Record initial event
    let event = GenealogyEvent {
        event_id: Uuid::now_v7().to_string(),
        serial_number: serial.to_string(),
        stage: stage.clone(),
        description: format!(
            "Παραλαβή εξαρτήματος από '{}' (Τιμολόγιο: {})",
            supplier,
            invoice.unwrap_or("Χωρίς ένδειξη")
        ),
        actor: actor.to_string(),
        timestamp: now,
    };
    record_genealogy_event(conn, &event)?;

    Ok(SerialGenealogy {
        serial_number: serial.to_string(),
        sku: sku.to_string(),
        name: name.to_string(),
        supplier_name: supplier.to_string(),
        warranty_months,
        intake_date: now,
        current_stage: stage,
        updated_at: now,
    })
}

/// Transitions a component to a new lifecycle stage, appending an immutable event.
pub fn transition_stage(
    conn: &Connection,
    serial: &str,
    new_stage: ComponentLifecycleStage,
    description: &str,
    actor: &str,
) -> Result<SerialGenealogy> {
    let now = Utc::now().timestamp_millis();
    let stage_json = serde_json::to_string(&new_stage).unwrap_or_default();

    conn.execute(
        "UPDATE serial_genealogies SET current_stage_json = ?1, updated_at = ?2 WHERE serial_number = ?3",
        params![stage_json, now, serial],
    )?;

    let event = GenealogyEvent {
        event_id: Uuid::now_v7().to_string(),
        serial_number: serial.to_string(),
        stage: new_stage,
        description: description.to_string(),
        actor: actor.to_string(),
        timestamp: now,
    };
    record_genealogy_event(conn, &event)?;

    match get_genealogy(conn, serial)? {
        Some(g) => Ok(g),
        None => Err(rusqlite::Error::QueryReturnedNoRows),
    }
}

/// Marks a component as installed into a customer's repair ticket.
pub fn install_in_ticket(
    conn: &Connection,
    serial: &str,
    customer_name: &str,
    device_model: &str,
    ticket_number: i64,
    actor: &str,
) -> Result<SerialGenealogy> {
    let stage = ComponentLifecycleStage::InstalledInCustomerDevice {
        customer_name: customer_name.to_string(),
        device_model: device_model.to_string(),
        ticket_number,
    };
    let desc = format!(
        "Εγκατάσταση στη συσκευή '{}' του πελάτη '{}' (Δελτίο #{})",
        device_model, customer_name, ticket_number
    );
    transition_stage(conn, serial, stage, &desc, actor)
}

/// Initiates an RMA warranty claim against the original supplier.
pub fn initiate_rma(
    conn: &Connection,
    serial: &str,
    fault_description: &str,
    actor: &str,
) -> Result<SerialGenealogy> {
    let stage = ComponentLifecycleStage::RmaClaimInitiated {
        fault_description: fault_description.to_string(),
        claimed_by: actor.to_string(),
    };
    let desc = format!("Έναρξη διαδικασίας RMA: {}", fault_description);
    transition_stage(conn, serial, stage, &desc, actor)
}

/// Resolves an RMA claim with a replacement unit from the supplier.
pub fn resolve_rma_replacement(
    conn: &Connection,
    serial: &str,
    replacement_serial: &str,
    credit_invoice: Option<&str>,
    actor: &str,
) -> Result<SerialGenealogy> {
    let stage = ComponentLifecycleStage::RmaReplacedBySupplier {
        replacement_serial: replacement_serial.to_string(),
        credit_invoice: credit_invoice.map(|s| s.to_string()),
    };
    let desc = format!(
        "Αντικατάσταση ελαττωματικού με νέο S/N '{}' (Παραστατικό: {})",
        replacement_serial,
        credit_invoice.unwrap_or("Εντός Εγγύησης")
    );
    transition_stage(conn, serial, stage, &desc, actor)
}

/// Fetches full genealogy metadata for a given serial number.
pub fn get_genealogy(conn: &Connection, serial: &str) -> Result<Option<SerialGenealogy>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT serial_number, sku, name, supplier_name, warranty_months, intake_date, current_stage_json, updated_at
        FROM serial_genealogies WHERE serial_number = ?1
        "#,
    )?;

    let mut rows = stmt.query(params![serial])?;
    if let Some(row) = rows.next()? {
        let stage_str: String = row.get(6)?;
        let current_stage: ComponentLifecycleStage =
            serde_json::from_str(&stage_str).unwrap_or(ComponentLifecycleStage::Disposed {
                reason: "Corrupted stage data".to_string(),
            });

        Ok(Some(SerialGenealogy {
            serial_number: row.get(0)?,
            sku: row.get(1)?,
            name: row.get(2)?,
            supplier_name: row.get(3)?,
            warranty_months: row.get(4)?,
            intake_date: row.get(5)?,
            current_stage,
            updated_at: row.get(7)?,
        }))
    } else {
        Ok(None)
    }
}

/// Retrieves the complete historical timeline for a component ordered chronologically.
pub fn get_genealogy_timeline(conn: &Connection, serial: &str) -> Result<Vec<GenealogyEvent>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT event_id, serial_number, stage_json, description, actor, timestamp
        FROM genealogy_events
        WHERE serial_number = ?1
        ORDER BY timestamp ASC
        "#,
    )?;

    let event_rows = stmt.query_map(params![serial], |row| {
        let stage_str: String = row.get(2)?;
        let stage: ComponentLifecycleStage =
            serde_json::from_str(&stage_str).unwrap_or(ComponentLifecycleStage::Disposed {
                reason: "Corrupted event stage".to_string(),
            });

        Ok(GenealogyEvent {
            event_id: row.get(0)?,
            serial_number: row.get(1)?,
            stage,
            description: row.get(3)?,
            actor: row.get(4)?,
            timestamp: row.get(5)?,
        })
    })?;

    let mut events = Vec::new();
    for ev in event_rows {
        events.push(ev?);
    }
    Ok(events)
}

/// Lists all components currently in an active RMA warranty claim status.
pub fn list_active_rma_claims(conn: &Connection) -> Result<Vec<SerialGenealogy>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT serial_number, sku, name, supplier_name, warranty_months, intake_date, current_stage_json, updated_at
        FROM serial_genealogies
        WHERE current_stage_json LIKE '%RmaClaimInitiated%'
        ORDER BY updated_at DESC
        "#,
    )?;

    let rows = stmt.query_map([], |row| {
        let stage_str: String = row.get(6)?;
        let current_stage: ComponentLifecycleStage =
            serde_json::from_str(&stage_str).unwrap_or(ComponentLifecycleStage::Disposed {
                reason: "Corrupted stage data".to_string(),
            });

        Ok(SerialGenealogy {
            serial_number: row.get(0)?,
            sku: row.get(1)?,
            name: row.get(2)?,
            supplier_name: row.get(3)?,
            warranty_months: row.get(4)?,
            intake_date: row.get(5)?,
            current_stage,
            updated_at: row.get(7)?,
        })
    })?;

    let mut claims = Vec::new();
    for r in rows {
        claims.push(r?);
    }
    Ok(claims)
}

/// Searches serial genealogies by query substring across serial, sku, name, or supplier.
pub fn search_genealogies(conn: &Connection, query: &str) -> Result<Vec<SerialGenealogy>> {
    let pattern = format!("%{}%", query.trim());
    let mut stmt = conn.prepare(
        r#"
        SELECT serial_number, sku, name, supplier_name, warranty_months, intake_date, current_stage_json, updated_at
        FROM serial_genealogies
        WHERE serial_number LIKE ?1 OR sku LIKE ?1 OR name LIKE ?1 OR supplier_name LIKE ?1
        ORDER BY updated_at DESC
        LIMIT 50
        "#,
    )?;

    let rows = stmt.query_map(params![pattern], |row| {
        let stage_str: String = row.get(6)?;
        let current_stage: ComponentLifecycleStage =
            serde_json::from_str(&stage_str).unwrap_or(ComponentLifecycleStage::Disposed {
                reason: "Corrupted stage data".to_string(),
            });

        Ok(SerialGenealogy {
            serial_number: row.get(0)?,
            sku: row.get(1)?,
            name: row.get(2)?,
            supplier_name: row.get(3)?,
            warranty_months: row.get(4)?,
            intake_date: row.get(5)?,
            current_stage,
            updated_at: row.get(7)?,
        })
    })?;

    let mut results = Vec::new();
    for r in rows {
        results.push(r?);
    }
    Ok(results)
}

fn record_genealogy_event(conn: &Connection, event: &GenealogyEvent) -> Result<()> {
    let stage_json = serde_json::to_string(&event.stage).unwrap_or_default();
    conn.execute(
        r#"
        INSERT INTO genealogy_events (event_id, serial_number, stage_json, description, actor, timestamp)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        "#,
        params![
            event.event_id,
            event.serial_number,
            stage_json,
            event.description,
            event.actor,
            event.timestamp
        ],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_mem_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_genealogy_schema(&conn).unwrap();
        conn
    }

    #[test]
    fn test_genealogy_intake_and_fetch() {
        let conn = setup_mem_db();
        let g = record_component_intake(
            &conn,
            "SN-BOSCH-99120",
            "BSH-ALT-01",
            "Δυναμό Αυτοκινήτου 12V 90A",
            "Bosch Hellas",
            Some("INV-2026-441"),
            24,
            "Νίκος (Αποθηκάριος)",
        )
        .unwrap();

        assert_eq!(g.serial_number, "SN-BOSCH-99120");
        assert_eq!(g.sku, "BSH-ALT-01");
        assert_eq!(g.warranty_months, 24);
        assert_eq!(g.current_stage.display_name(), "Παραλαβή Προμηθευτή");

        let fetched = get_genealogy(&conn, "SN-BOSCH-99120").unwrap().unwrap();
        assert_eq!(fetched.name, "Δυναμό Αυτοκινήτου 12V 90A");

        let timeline = get_genealogy_timeline(&conn, "SN-BOSCH-99120").unwrap();
        assert_eq!(timeline.len(), 1);
        assert!(timeline[0].description.contains("INV-2026-441"));
    }

    #[test]
    fn test_lifecycle_and_rma_workflow() {
        let conn = setup_mem_db();
        record_component_intake(
            &conn,
            "SN-SCR-4040",
            "SCR-OLED-14",
            "Οθόνη OLED 14.1\"",
            "Global Parts EU",
            None,
            12,
            "Μαρία (Παραλαβές)",
        )
        .unwrap();

        // 1. Install in customer device ticket #1042
        install_in_ticket(
            &conn,
            "SN-SCR-4040",
            "Γιώργος Αντωνίου",
            "ThinkPad T14 Gen 3",
            1042,
            "Κώστας (Τεχνικός)",
        )
        .unwrap();

        let installed = get_genealogy(&conn, "SN-SCR-4040").unwrap().unwrap();
        assert_eq!(installed.current_stage.display_name(), "Εγκατεστημένο σε Συσκευή");

        // 2. Customer returns device with fault -> initiate RMA
        initiate_rma(
            &conn,
            "SN-SCR-4040",
            "Κάθετες πράσινες γραμμές μετά από 2 εβδομάδες χρήσης",
            "Κώστας (Τεχνικός)",
        )
        .unwrap();

        let rma_active = list_active_rma_claims(&conn).unwrap();
        assert_eq!(rma_active.len(), 1);
        assert_eq!(rma_active[0].serial_number, "SN-SCR-4040");

        // 3. Supplier sends replacement part
        resolve_rma_replacement(
            &conn,
            "SN-SCR-4040",
            "SN-SCR-5099",
            Some("RMA-CR-8812"),
            "Νίκος (Αποθήκη)",
        )
        .unwrap();

        let resolved = get_genealogy(&conn, "SN-SCR-4040").unwrap().unwrap();
        assert_eq!(resolved.current_stage.display_name(), "Αντικατάσταση από Προμηθευτή");

        let active_after = list_active_rma_claims(&conn).unwrap();
        assert!(active_after.is_empty());

        let timeline = get_genealogy_timeline(&conn, "SN-SCR-4040").unwrap();
        assert_eq!(timeline.len(), 4);
    }

    #[test]
    fn test_warranty_status_calculation() {
        let conn = setup_mem_db();
        let g = record_component_intake(
            &conn,
            "SN-WARR-TEST",
            "PART-01",
            "Μίζα Αυτοκινήτου",
            "Valeo Direct",
            None,
            12,
            "Admin",
        )
        .unwrap();

        // Same time as intake: should have ~360 days remaining
        let status = g.warranty_status(g.intake_date);
        match status {
            WarrantyStatus::Valid { days_remaining } => {
                assert!(days_remaining >= 355 && days_remaining <= 365);
            }
            _ => panic!("Expected valid warranty"),
        }

        // 400 days in the future: should be expired
        let future_time = g.intake_date + 400 * 24 * 3600 * 1000;
        let expired_status = g.warranty_status(future_time);
        match expired_status {
            WarrantyStatus::Expired { days_expired } => {
                assert!(days_expired >= 35 && days_expired <= 45);
            }
            _ => panic!("Expected expired warranty"),
        }
    }

    #[test]
    fn test_search_genealogies() {
        let conn = setup_mem_db();
        record_component_intake(&conn, "SN-ALPHA-01", "SKU-A", "Alternator", "Denso", None, 24, "User").unwrap();
        record_component_intake(&conn, "SN-BETA-02", "SKU-B", "Brake Pad", "Brembo", None, 12, "User").unwrap();

        let res_a = search_genealogies(&conn, "ALPHA").unwrap();
        assert_eq!(res_a.len(), 1);
        assert_eq!(res_a[0].serial_number, "SN-ALPHA-01");

        let res_b = search_genealogies(&conn, "Brembo").unwrap();
        assert_eq!(res_b.len(), 1);
        assert_eq!(res_b[0].name, "Brake Pad");
    }
}
