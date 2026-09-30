//! Vendor-Managed Inventory (VMI) & Consignment Tracking Engine for Proteus BOS.
//! Manages consignment stock placed at partner locations, job-site containers, and branch workshops:
//! - Consignment partner accounts and locations
//! - Real-time stock on hand per consignment depot
//! - Automated VMI replenishment alerts triggered when stock falls below minimum thresholds
//! - Complete audit trail of consignment transfers, sales/consumption, and returns.
//! Strict Rule 1 (100% Original Codebase) and Rule 5 (Zero Mock Data).

use chrono::Utc;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type of stock movement within the consignment ecosystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsignmentMovementType {
    TransferToConsignment,  // Αποστολή σε Παρακαταθήκη
    ConsumptionSale,        // Κατανάλωση / Πώληση από Παρακαταθήκη
    ReturnToWarehouse,      // Επιστροφή στην Κεντρική Αποθήκη
    InventoryAdjustment,    // Απογραφή / Προσαρμογή
}

impl ConsignmentMovementType {
    pub fn code(&self) -> &'static str {
        match self {
            Self::TransferToConsignment => "TRANSFER_IN",
            Self::ConsumptionSale => "SALE_CONSUMPTION",
            Self::ReturnToWarehouse => "RETURN_OUT",
            Self::InventoryAdjustment => "ADJUSTMENT",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::TransferToConsignment => "Αποστολή σε Παρακαταθήκη",
            Self::ConsumptionSale => "Κατανάλωση / Πώληση",
            Self::ReturnToWarehouse => "Επιστροφή σε Αποθήκη",
            Self::InventoryAdjustment => "Προσαρμογή Απογραφής",
        }
    }

    pub fn from_code(code: &str) -> Self {
        match code {
            "TRANSFER_IN" => Self::TransferToConsignment,
            "SALE_CONSUMPTION" => Self::ConsumptionSale,
            "RETURN_OUT" => Self::ReturnToWarehouse,
            "ADJUSTMENT" => Self::InventoryAdjustment,
            _ => Self::TransferToConsignment,
        }
    }
}

/// Third-party partner holding consignment stock.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConsignmentPartner {
    pub id: String,
    pub name: String,
    pub afm: String,
    pub address: String,
    pub phone: String,
    pub created_at: String,
}

/// Serialized or batch item currently residing at a consignment depot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConsignmentStockItem {
    pub id: String,
    pub partner_id: String,
    pub sku: String,
    pub description: String,
    pub quantity_on_hand: f64,
    pub minimum_threshold: f64,
    pub unit_price: f64,
    pub last_replenished_at: String,
}

/// Individual immutable ledger entry for consignment movements.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConsignmentMovement {
    pub id: String,
    pub partner_id: String,
    pub sku: String,
    pub movement_type: ConsignmentMovementType,
    pub quantity: f64,
    pub reference_doc: Option<String>,
    pub timestamp: String,
    pub notes: Option<String>,
}

/// Initializes the SQLite tables and indexes for consignment tracking.
pub fn init_consignment_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS consignment_partners (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            afm TEXT NOT NULL UNIQUE,
            address TEXT NOT NULL,
            phone TEXT NOT NULL,
            created_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_consignment_afm ON consignment_partners(afm);

        CREATE TABLE IF NOT EXISTS consignment_stock (
            id TEXT PRIMARY KEY,
            partner_id TEXT NOT NULL,
            sku TEXT NOT NULL,
            description TEXT NOT NULL,
            quantity_on_hand REAL NOT NULL DEFAULT 0.0,
            minimum_threshold REAL NOT NULL DEFAULT 0.0,
            unit_price REAL NOT NULL DEFAULT 0.0,
            last_replenished_at TEXT NOT NULL,
            UNIQUE(partner_id, sku),
            FOREIGN KEY(partner_id) REFERENCES consignment_partners(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_consignment_stock_partner ON consignment_stock(partner_id);

        CREATE TABLE IF NOT EXISTS consignment_movements (
            id TEXT PRIMARY KEY,
            partner_id TEXT NOT NULL,
            sku TEXT NOT NULL,
            movement_type TEXT NOT NULL,
            quantity REAL NOT NULL,
            reference_doc TEXT,
            timestamp TEXT NOT NULL,
            notes TEXT,
            FOREIGN KEY(partner_id) REFERENCES consignment_partners(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_consignment_movements_sku ON consignment_movements(sku);
        CREATE INDEX IF NOT EXISTS idx_consignment_movements_partner ON consignment_movements(partner_id);",
    )
}

/// Registers a new consignment partner location.
pub fn register_consignment_partner(conn: &Connection, partner: &ConsignmentPartner) -> Result<()> {
    init_consignment_schema(conn)?;
    conn.execute(
        "INSERT INTO consignment_partners (id, name, afm, address, phone, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(afm) DO UPDATE SET
            name = excluded.name,
            address = excluded.address,
            phone = excluded.phone",
        params![
            partner.id,
            partner.name,
            partner.afm,
            partner.address,
            partner.phone,
            partner.created_at,
        ],
    )?;
    Ok(())
}

/// Lists all registered consignment partners.
pub fn list_consignment_partners(conn: &Connection) -> Result<Vec<ConsignmentPartner>> {
    init_consignment_schema(conn)?;
    let mut stmt = conn.prepare(
        "SELECT id, name, afm, address, phone, created_at FROM consignment_partners ORDER BY name ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(ConsignmentPartner {
            id: row.get(0)?,
            name: row.get(1)?,
            afm: row.get(2)?,
            address: row.get(3)?,
            phone: row.get(4)?,
            created_at: row.get(5)?,
        })
    })?;

    let mut partners = Vec::new();
    for row in rows {
        partners.push(row?);
    }
    Ok(partners)
}

/// Records a transfer of goods into a partner's consignment inventory.
pub fn transfer_to_consignment(
    conn: &Connection,
    partner_id: &str,
    sku: &str,
    description: &str,
    quantity: f64,
    unit_price: f64,
    minimum_threshold: f64,
    reference_doc: Option<&str>,
) -> Result<()> {
    init_consignment_schema(conn)?;
    let now = Utc::now().to_rfc3339();
    let stock_id = Uuid::new_v4().to_string();

    // 1. Update stock balance
    conn.execute(
        "INSERT INTO consignment_stock (
            id, partner_id, sku, description, quantity_on_hand, minimum_threshold, unit_price, last_replenished_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        ON CONFLICT(partner_id, sku) DO UPDATE SET
            quantity_on_hand = quantity_on_hand + excluded.quantity_on_hand,
            unit_price = excluded.unit_price,
            minimum_threshold = excluded.minimum_threshold,
            last_replenished_at = excluded.last_replenished_at",
        params![
            stock_id,
            partner_id,
            sku,
            description,
            quantity,
            minimum_threshold,
            unit_price,
            now,
        ],
    )?;

    // 2. Append movement record
    let movement_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO consignment_movements (
            id, partner_id, sku, movement_type, quantity, reference_doc, timestamp, notes
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            movement_id,
            partner_id,
            sku,
            ConsignmentMovementType::TransferToConsignment.code(),
            quantity,
            reference_doc,
            now,
            "Αποστολή αναπλήρωσης παρακαταθήκης",
        ],
    )?;

    Ok(())
}

/// Records customer consumption/sale from consignment stock.
pub fn record_consignment_sale(
    conn: &Connection,
    partner_id: &str,
    sku: &str,
    quantity: f64,
    reference_doc: Option<&str>,
) -> Result<()> {
    init_consignment_schema(conn)?;
    let now = Utc::now().to_rfc3339();

    // 1. Deduct stock balance
    conn.execute(
        "UPDATE consignment_stock SET
            quantity_on_hand = MAX(0.0, quantity_on_hand - ?1)
         WHERE partner_id = ?2 AND sku = ?3",
        params![quantity, partner_id, sku],
    )?;

    // 2. Append movement record
    let movement_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO consignment_movements (
            id, partner_id, sku, movement_type, quantity, reference_doc, timestamp, notes
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            movement_id,
            partner_id,
            sku,
            ConsignmentMovementType::ConsumptionSale.code(),
            quantity,
            reference_doc,
            now,
            "Πώληση / Ανάλωση από πελάτη",
        ],
    )?;

    Ok(())
}

/// Records return of unsold consignment goods back to central warehouse.
pub fn return_from_consignment(
    conn: &Connection,
    partner_id: &str,
    sku: &str,
    quantity: f64,
    reference_doc: Option<&str>,
) -> Result<()> {
    init_consignment_schema(conn)?;
    let now = Utc::now().to_rfc3339();

    // 1. Deduct from consignment stock
    conn.execute(
        "UPDATE consignment_stock SET
            quantity_on_hand = MAX(0.0, quantity_on_hand - ?1)
         WHERE partner_id = ?2 AND sku = ?3",
        params![quantity, partner_id, sku],
    )?;

    // 2. Append movement record
    let movement_id = Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO consignment_movements (
            id, partner_id, sku, movement_type, quantity, reference_doc, timestamp, notes
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            movement_id,
            partner_id,
            sku,
            ConsignmentMovementType::ReturnToWarehouse.code(),
            quantity,
            reference_doc,
            now,
            "Επιστροφή στην κεντρική αποθήκη",
        ],
    )?;

    Ok(())
}

/// Lists all stock items held at a consignment partner location.
pub fn list_consignment_stock(conn: &Connection, partner_id: &str) -> Result<Vec<ConsignmentStockItem>> {
    init_consignment_schema(conn)?;
    let mut stmt = conn.prepare(
        "SELECT id, partner_id, sku, description, quantity_on_hand, minimum_threshold, unit_price, last_replenished_at
         FROM consignment_stock WHERE partner_id = ?1 ORDER BY sku ASC",
    )?;

    let rows = stmt.query_map(params![partner_id], |row| {
        Ok(ConsignmentStockItem {
            id: row.get(0)?,
            partner_id: row.get(1)?,
            sku: row.get(2)?,
            description: row.get(3)?,
            quantity_on_hand: row.get(4)?,
            minimum_threshold: row.get(5)?,
            unit_price: row.get(6)?,
            last_replenished_at: row.get(7)?,
        })
    })?;

    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }
    Ok(items)
}

/// Automatically scans all consignment depots and returns VMI replenishment alerts
/// for items where stock on hand is at or below the minimum safety threshold.
/// Returns: (partner_id, sku, current_qty, min_threshold)
pub fn check_vmi_replenishment_alerts(
    conn: &Connection,
) -> Result<Vec<(String, String, f64, f64)>> {
    init_consignment_schema(conn)?;
    let mut stmt = conn.prepare(
        "SELECT partner_id, sku, quantity_on_hand, minimum_threshold
         FROM consignment_stock
         WHERE quantity_on_hand <= minimum_threshold AND minimum_threshold > 0.0
         ORDER BY partner_id ASC, sku ASC",
    )?;

    let rows = stmt.query_map([], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    })?;

    let mut alerts = Vec::new();
    for row in rows {
        alerts.push(row?);
    }
    Ok(alerts)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_consignment_schema(&conn).unwrap();
        conn
    }

    #[test]
    fn test_consignment_lifecycle_and_vmi_alert() {
        let conn = setup_test_db();
        let partner = ConsignmentPartner {
            id: "cp-001".to_string(),
            name: "PARTNER WORKSHOP ATE".to_string(),
            afm: "094014201".to_string(),
            address: "Πειραιάς 44".to_string(),
            phone: "+30 210 9988776".to_string(),
            created_at: Utc::now().to_rfc3339(),
        };
        register_consignment_partner(&conn, &partner).unwrap();

        let partners = list_consignment_partners(&conn).unwrap();
        assert_eq!(partners.len(), 1);
        assert_eq!(partners[0].afm, "094014201");

        // 1. Transfer stock to consignment
        transfer_to_consignment(
            &conn,
            &partner.id,
            "OIL-SYNTH-5W30",
            "Συνθετικό Λιπαντικό 5W30 4L",
            20.0,
            35.0,
            5.0, // Minimum threshold = 5.0
            Some("ΔΑ-2026-0001"),
        )
        .unwrap();

        let stock = list_consignment_stock(&conn, &partner.id).unwrap();
        assert_eq!(stock.len(), 1);
        assert_eq!(stock[0].quantity_on_hand, 20.0);

        // Check alerts - currently above threshold (20 > 5)
        let alerts = check_vmi_replenishment_alerts(&conn).unwrap();
        assert_eq!(alerts.len(), 0);

        // 2. Record consumption sale of 16 units (leaves 4.0 <= 5.0)
        record_consignment_sale(&conn, &partner.id, "OIL-SYNTH-5W30", 16.0, Some("INV-001")).unwrap();

        let stock_after = list_consignment_stock(&conn, &partner.id).unwrap();
        assert_eq!(stock_after[0].quantity_on_hand, 4.0);

        // Check alerts - now triggered! (4.0 <= 5.0)
        let alerts_triggered = check_vmi_replenishment_alerts(&conn).unwrap();
        assert_eq!(alerts_triggered.len(), 1);
        assert_eq!(alerts_triggered[0].1, "OIL-SYNTH-5W30");
        assert_eq!(alerts_triggered[0].2, 4.0);
        assert_eq!(alerts_triggered[0].3, 5.0);

        // 3. Return remaining 4 units back to warehouse
        return_from_consignment(&conn, &partner.id, "OIL-SYNTH-5W30", 4.0, Some("RET-001")).unwrap();
        let stock_final = list_consignment_stock(&conn, &partner.id).unwrap();
        assert_eq!(stock_final[0].quantity_on_hand, 0.0);
    }
}
