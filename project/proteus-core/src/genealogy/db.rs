//! SQLite persistence for Serial Number Genealogy & RMA lifecycle tracking.
//! Schema initialization, CRUD operations, lifecycle transitions, and search queries.

use chrono::Utc;
use rusqlite::{params, Connection, Result};
use uuid::Uuid;
use super::types::*;

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
