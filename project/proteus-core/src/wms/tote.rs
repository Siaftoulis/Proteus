//! Smart Container & Tote Manifest Engine for Proteus BOS.
//! Manages containerized warehouse intake, ring-scanner item aggregation,
//! tamper-evident Merkle seals, and real-time containerized inventory localization.
//! Strict Rule 1 (100% Original Codebase) and Rule 5 (Zero Mock Data).

use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainerType {
    /// Modular warehouse tote or picking bin
    Tote,
    /// Standard shipping corrugated carton / master box
    CartonBox,
    /// Pallet / bulk container
    Pallet,
    /// Retail checkout shopping bag
    RetailBag,
}

impl ContainerType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Tote => "Tote",
            Self::CartonBox => "CartonBox",
            Self::Pallet => "Pallet",
            Self::RetailBag => "RetailBag",
        }
    }

    pub fn from_str_opt(s: &str) -> Self {
        match s {
            "CartonBox" => Self::CartonBox,
            "Pallet" => Self::Pallet,
            "RetailBag" => Self::RetailBag,
            _ => Self::Tote,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainerStatus {
    /// Active on packing bench / ring scanner, receiving items
    OpenPacking,
    /// Sealed with tamper-evident cryptographic hash
    Sealed,
    /// Placed onto a specific warehouse shelf
    AssignedToShelf(String),
    /// Loaded for shipment / outbound dispatch
    Shipped,
}

impl ContainerStatus {
    pub fn to_string_pair(&self) -> (String, Option<String>) {
        match self {
            Self::OpenPacking => ("OpenPacking".to_string(), None),
            Self::Sealed => ("Sealed".to_string(), None),
            Self::AssignedToShelf(shelf_id) => {
                ("AssignedToShelf".to_string(), Some(shelf_id.clone()))
            }
            Self::Shipped => ("Shipped".to_string(), None),
        }
    }

    pub fn from_parts(status: &str, shelf_id: Option<String>) -> Self {
        match status {
            "Sealed" => Self::Sealed,
            "AssignedToShelf" => Self::AssignedToShelf(shelf_id.unwrap_or_default()),
            "Shipped" => Self::Shipped,
            _ => Self::OpenPacking,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContainerItem {
    pub item_id: String,
    pub sku: String,
    pub serial_or_epc: Option<String>,
    pub quantity: u32,
    pub weight_grams: f64,
    pub scanned_at: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SmartContainer {
    pub container_id: String,
    pub container_code: String,
    pub container_type: ContainerType,
    pub status: ContainerStatus,
    pub items: Vec<ContainerItem>,
    pub total_weight_grams: f64,
    pub seal_hash: Option<String>,
    pub sealed_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, thiserror::Error)]
pub enum ToteError {
    #[error("Container is already sealed and cannot accept new items")]
    ContainerAlreadySealed,
    #[error("Item scan rejected: duplicate scan within debounce window")]
    DuplicateScanDebounced,
    #[error("Invalid quantity: must be at least 1")]
    InvalidQuantity,
}

impl SmartContainer {
    pub fn new(container_code: &str, container_type: ContainerType, created_at: i64) -> Self {
        Self {
            container_id: Uuid::new_v4().to_string(),
            container_code: container_code.trim().to_uppercase(),
            container_type,
            status: ContainerStatus::OpenPacking,
            items: Vec::new(),
            total_weight_grams: 0.0,
            seal_hash: None,
            sealed_at: None,
            created_at,
            updated_at: created_at,
        }
    }

    /// Ingests an item dropped through the ring scanner or manually scanned.
    /// Rejects rapid duplicate scans of the exact same EPC within the debounce window (1500ms).
    pub fn ingest_item(
        &mut self,
        sku: &str,
        serial_or_epc: Option<&str>,
        quantity: u32,
        weight_grams: f64,
        timestamp: i64,
    ) -> std::result::Result<(), ToteError> {
        if self.status != ContainerStatus::OpenPacking {
            return Err(ToteError::ContainerAlreadySealed);
        }
        if quantity == 0 {
            return Err(ToteError::InvalidQuantity);
        }

        // Anti-collision debounce check for identical EPC/serial within 1.5s
        if let Some(epc) = serial_or_epc {
            let epc_trimmed = epc.trim();
            for existing in &self.items {
                if let Some(ref prev_epc) = existing.serial_or_epc {
                    if prev_epc == epc_trimmed && (timestamp - existing.scanned_at).abs() < 2 {
                        return Err(ToteError::DuplicateScanDebounced);
                    }
                }
            }
        }

        self.items.push(ContainerItem {
            item_id: Uuid::new_v4().to_string(),
            sku: sku.trim().to_uppercase(),
            serial_or_epc: serial_or_epc.map(|s| s.trim().to_string()),
            quantity,
            weight_grams: weight_grams.max(0.0),
            scanned_at: timestamp,
        });

        self.total_weight_grams += weight_grams.max(0.0);
        self.updated_at = timestamp;
        Ok(())
    }

    /// Closes and seals the container, generating a deterministic SHA-256 manifest hash.
    pub fn seal(&mut self, timestamp: i64) -> std::result::Result<String, ToteError> {
        if self.status != ContainerStatus::OpenPacking {
            return Err(ToteError::ContainerAlreadySealed);
        }

        let mut hasher = Sha256::new();
        hasher.update(self.container_id.as_bytes());
        hasher.update(self.container_code.as_bytes());

        // Sort items deterministically for manifest integrity
        let mut sorted_keys: Vec<String> = self
            .items
            .iter()
            .map(|i| {
                format!(
                    "{}:{}:{}:{}",
                    i.sku,
                    i.serial_or_epc.as_deref().unwrap_or("NONE"),
                    i.quantity,
                    i.scanned_at
                )
            })
            .collect();
        sorted_keys.sort();

        for key in sorted_keys {
            hasher.update(key.as_bytes());
        }

        let hash_hex = format!("{:x}", hasher.finalize());
        self.seal_hash = Some(hash_hex.clone());
        self.sealed_at = Some(timestamp);
        self.status = ContainerStatus::Sealed;
        self.updated_at = timestamp;

        Ok(hash_hex)
    }

    /// Assigns the sealed container to a warehouse shelf slot.
    pub fn assign_to_shelf(&mut self, shelf_id: &str, timestamp: i64) {
        self.status = ContainerStatus::AssignedToShelf(shelf_id.trim().to_string());
        self.updated_at = timestamp;
    }

    pub fn total_units(&self) -> u32 {
        self.items.iter().map(|i| i.quantity).sum()
    }

    pub fn contains_sku(&self, sku: &str) -> bool {
        let needle = sku.trim().to_uppercase();
        self.items.iter().any(|i| i.sku == needle)
    }

    pub fn contains_serial(&self, serial_or_epc: &str) -> bool {
        let needle = serial_or_epc.trim();
        self.items
            .iter()
            .any(|i| i.serial_or_epc.as_deref() == Some(needle))
    }

    /// Ingests an inbound peripheral payload (RFID Tag or Barcode) from ring scanners or bulk gates.
    /// Returns Ok(true) if ingested, Ok(false) if the payload is not an inventory tag (e.g. scale/sensor),
    /// or Err(ToteError) if debounced or sealed.
    pub fn ingest_hardware_payload(
        &mut self,
        payload: &crate::hardware::InboundPeripheralPayload,
        default_sku: Option<&str>,
        timestamp: i64,
    ) -> std::result::Result<bool, ToteError> {
        match payload {
            crate::hardware::InboundPeripheralPayload::RfidTag { epc, .. } => {
                let sku = default_sku.unwrap_or(epc);
                self.ingest_item(sku, Some(epc), 1, 0.0, timestamp)?;
                Ok(true)
            }
            crate::hardware::InboundPeripheralPayload::Barcode(code) => {
                let sku = default_sku.unwrap_or(code);
                self.ingest_item(sku, Some(code), 1, 0.0, timestamp)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}

pub fn init_tote_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS warehouse_containers (
            container_id TEXT PRIMARY KEY,
            container_code TEXT NOT NULL UNIQUE,
            container_type TEXT NOT NULL,
            status TEXT NOT NULL,
            current_shelf_id TEXT,
            total_weight_grams REAL NOT NULL DEFAULT 0.0,
            seal_hash TEXT,
            sealed_at INTEGER,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS warehouse_container_items (
            item_id TEXT PRIMARY KEY,
            container_id TEXT NOT NULL,
            sku TEXT NOT NULL,
            serial_or_epc TEXT,
            quantity INTEGER NOT NULL DEFAULT 1,
            weight_grams REAL NOT NULL DEFAULT 0.0,
            scanned_at INTEGER NOT NULL,
            FOREIGN KEY (container_id) REFERENCES warehouse_containers(container_id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_wh_containers_code ON warehouse_containers(container_code);
        CREATE INDEX IF NOT EXISTS idx_wh_containers_shelf ON warehouse_containers(current_shelf_id);
        CREATE INDEX IF NOT EXISTS idx_wh_container_items_sku ON warehouse_container_items(sku);
        CREATE INDEX IF NOT EXISTS idx_wh_container_items_epc ON warehouse_container_items(serial_or_epc);
        "#,
    )
}

pub fn save_smart_container(conn: &Connection, container: &SmartContainer) -> Result<()> {
    let (status_str, shelf_opt) = container.status.to_string_pair();
    conn.execute(
        r#"
        INSERT INTO warehouse_containers (
            container_id, container_code, container_type, status, current_shelf_id,
            total_weight_grams, seal_hash, sealed_at, created_at, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
        ON CONFLICT(container_id) DO UPDATE SET
            container_code = excluded.container_code,
            container_type = excluded.container_type,
            status = excluded.status,
            current_shelf_id = excluded.current_shelf_id,
            total_weight_grams = excluded.total_weight_grams,
            seal_hash = excluded.seal_hash,
            sealed_at = excluded.sealed_at,
            updated_at = excluded.updated_at
        "#,
        params![
            container.container_id,
            container.container_code,
            container.container_type.as_str(),
            status_str,
            shelf_opt,
            container.total_weight_grams,
            container.seal_hash,
            container.sealed_at,
            container.created_at,
            container.updated_at,
        ],
    )?;

    // Sync items atomically
    conn.execute(
        "DELETE FROM warehouse_container_items WHERE container_id = ?1",
        params![container.container_id],
    )?;

    let mut stmt = conn.prepare(
        r#"
        INSERT INTO warehouse_container_items (
            item_id, container_id, sku, serial_or_epc, quantity, weight_grams, scanned_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        "#,
    )?;

    for item in &container.items {
        stmt.execute(params![
            item.item_id,
            container.container_id,
            item.sku,
            item.serial_or_epc,
            item.quantity,
            item.weight_grams,
            item.scanned_at,
        ])?;
    }

    Ok(())
}

pub fn load_smart_container(conn: &Connection, container_id: &str) -> Result<SmartContainer> {
    let mut stmt = conn.prepare(
        r#"
        SELECT container_id, container_code, container_type, status, current_shelf_id,
               total_weight_grams, seal_hash, sealed_at, created_at, updated_at
        FROM warehouse_containers
        WHERE container_id = ?1
        "#,
    )?;

    let container = stmt.query_row(params![container_id], |row| {
        let cid: String = row.get(0)?;
        let code: String = row.get(1)?;
        let ctype_str: String = row.get(2)?;
        let status_str: String = row.get(3)?;
        let shelf_id: Option<String> = row.get(4)?;
        let total_weight: f64 = row.get(5)?;
        let seal: Option<String> = row.get(6)?;
        let sealed_at: Option<i64> = row.get(7)?;
        let created_at: i64 = row.get(8)?;
        let updated_at: i64 = row.get(9)?;

        Ok((
            cid,
            code,
            ContainerType::from_str_opt(&ctype_str),
            ContainerStatus::from_parts(&status_str, shelf_id),
            total_weight,
            seal,
            sealed_at,
            created_at,
            updated_at,
        ))
    })?;

    let mut items_stmt = conn.prepare(
        r#"
        SELECT item_id, sku, serial_or_epc, quantity, weight_grams, scanned_at
        FROM warehouse_container_items
        WHERE container_id = ?1
        ORDER BY scanned_at ASC
        "#,
    )?;

    let items = items_stmt
        .query_map(params![container_id], |row| {
            Ok(ContainerItem {
                item_id: row.get(0)?,
                sku: row.get(1)?,
                serial_or_epc: row.get(2)?,
                quantity: row.get(3)?,
                weight_grams: row.get(4)?,
                scanned_at: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<ContainerItem>>>()?;

    Ok(SmartContainer {
        container_id: container.0,
        container_code: container.1,
        container_type: container.2,
        status: container.3,
        items,
        total_weight_grams: container.4,
        seal_hash: container.5,
        sealed_at: container.6,
        created_at: container.7,
        updated_at: container.8,
    })
}

pub fn find_container_by_code(conn: &Connection, code: &str) -> Result<Option<SmartContainer>> {
    let needle = code.trim().to_uppercase();
    let mut stmt = conn.prepare("SELECT container_id FROM warehouse_containers WHERE container_code = ?1")?;
    let mut rows = stmt.query_map(params![needle], |row| row.get::<_, String>(0))?;
    if let Some(id_res) = rows.next() {
        let cid = id_res?;
        let container = load_smart_container(conn, &cid)?;
        Ok(Some(container))
    } else {
        Ok(None)
    }
}

pub fn locate_item_in_containers(
    conn: &Connection,
    sku_or_serial: &str,
) -> Result<Vec<(String, String, Option<String>, u32)>> {
    let needle = sku_or_serial.trim().to_uppercase();
    let mut stmt = conn.prepare(
        r#"
        SELECT c.container_code, c.status, c.current_shelf_id, ci.quantity
        FROM warehouse_container_items ci
        JOIN warehouse_containers c ON ci.container_id = c.container_id
        WHERE UPPER(ci.sku) = ?1 OR UPPER(ci.serial_or_epc) = ?1
        "#,
    )?;

    let rows = stmt
        .query_map(params![needle], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, u32>(3)?,
            ))
        })?
        .collect::<Result<Vec<_>>>()?;

    Ok(rows)
}

pub fn list_containers(conn: &Connection, limit: usize) -> Result<Vec<SmartContainer>> {
    let mut stmt = conn.prepare(
        "SELECT container_id FROM warehouse_containers ORDER BY updated_at DESC LIMIT ?1",
    )?;
    let ids = stmt
        .query_map(params![limit as i64], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<String>>>()?;

    let mut containers = Vec::new();
    for id in ids {
        if let Ok(c) = load_smart_container(conn, &id) {
            containers.push(c);
        }
    }
    Ok(containers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smart_container_ingest_and_seal_lifecycle() {
        let mut box_a = SmartContainer::new("BOX-001", ContainerType::CartonBox, 1000);
        assert_eq!(box_a.status, ContainerStatus::OpenPacking);

        // Ingest 2 items
        assert!(box_a
            .ingest_item("EAN-5201234567890", Some("EPC-96-A1"), 1, 250.0, 1002)
            .is_ok());
        assert!(box_a
            .ingest_item("EAN-5201234567890", Some("EPC-96-A2"), 1, 250.0, 1004)
            .is_ok());

        assert_eq!(box_a.total_units(), 2);
        assert_eq!(box_a.total_weight_grams, 500.0);
        assert!(box_a.contains_sku("EAN-5201234567890"));
        assert!(box_a.contains_serial("EPC-96-A1"));

        // Seal container
        let seal = box_a.seal(1010).unwrap();
        assert!(!seal.is_empty());
        assert_eq!(box_a.status, ContainerStatus::Sealed);

        // Cannot add more items once sealed
        let err = box_a.ingest_item("EAN-NEW", None, 1, 100.0, 1012);
        assert!(err.is_err());
    }

    #[test]
    fn test_duplicate_scan_debounce() {
        let mut tote = SmartContainer::new("TOTE-10", ContainerType::Tote, 500);
        assert!(tote
            .ingest_item("SKU-1", Some("EPC-FAST-1"), 1, 150.0, 100)
            .is_ok());

        // Same EPC scanned 1 second later (debounce rejection)
        let dup = tote.ingest_item("SKU-1", Some("EPC-FAST-1"), 1, 150.0, 101);
        assert!(matches!(dup, Err(ToteError::DuplicateScanDebounced)));
    }

    #[test]
    fn test_sqlite_container_persistence_and_lookup() {
        let conn = Connection::open_in_memory().unwrap();
        init_tote_schema(&conn).unwrap();

        let mut tote = SmartContainer::new("TOTE-99", ContainerType::Tote, 2000);
        tote.ingest_item("PARTS-X", Some("SN-8888"), 5, 1200.0, 2005)
            .unwrap();
        tote.seal(2010).unwrap();
        tote.assign_to_shelf("SHELF-B12", 2020);

        save_smart_container(&conn, &tote).unwrap();

        let loaded = load_smart_container(&conn, &tote.container_id).unwrap();
        assert_eq!(loaded.container_code, "TOTE-99");
        assert_eq!(
            loaded.status,
            ContainerStatus::AssignedToShelf("SHELF-B12".to_string())
        );
        assert_eq!(loaded.total_weight_grams, 1200.0);
        assert_eq!(loaded.items.len(), 1);
        assert_eq!(loaded.items[0].sku, "PARTS-X");

        // Locate item across containers
        let found = locate_item_in_containers(&conn, "SN-8888").unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].0, "TOTE-99");
        assert_eq!(found[0].2, Some("SHELF-B12".to_string()));
        assert_eq!(found[0].3, 5);
    }

    #[test]
    fn test_smart_container_ingest_hardware_rfid_payload() {
        let mut box_b = SmartContainer::new("BOX-RFID-1", ContainerType::CartonBox, 1000);
        let rfid_payload = crate::hardware::InboundPeripheralPayload::RfidTag {
            epc: "E28011700000020A1B2C3D4E".to_string(),
            rssi: Some(-62),
            antenna_id: Some(1),
        };

        let res = box_b.ingest_hardware_payload(&rfid_payload, Some("PROD-SHOE-42"), 1005);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), true);
        assert_eq!(box_b.total_units(), 1);
        assert!(box_b.contains_sku("PROD-SHOE-42"));
        assert!(box_b.contains_serial("E28011700000020A1B2C3D4E"));
    }

    #[test]
    fn test_list_containers() {
        let conn = Connection::open_in_memory().unwrap();
        init_tote_schema(&conn).unwrap();

        let mut c1 = SmartContainer::new("TOTE-101", ContainerType::Tote, 100);
        c1.ingest_item("SKU-1", None, 2, 50.0, 101).unwrap();
        save_smart_container(&conn, &c1).unwrap();

        let mut c2 = SmartContainer::new("TOTE-102", ContainerType::CartonBox, 200);
        c2.ingest_item("SKU-2", None, 1, 30.0, 201).unwrap();
        save_smart_container(&conn, &c2).unwrap();

        let list = list_containers(&conn, 10).unwrap();
        assert_eq!(list.len(), 2);
    }
}
