//! Spatial WMS & Retail Shelf Optimizer SQLite Persistence Engine.
//! Manages warehouse zones, racks, shelves, inventory allocations, and audit trails.
//! Strict Rule 1 (100% Original Codebase) and Rule 5 (Zero Mock Data).

use crate::wms::types::{
    Dimensions, ShelfAuditProof, WarehouseRack, WarehouseShelf, WarehouseZone, WmsItem,
};
use rusqlite::{params, Connection, Result};
use uuid::Uuid;

pub fn init_wms_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS warehouse_zones (
            zone_id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            code TEXT NOT NULL UNIQUE,
            temperature_class TEXT NOT NULL,
            description TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS warehouse_racks (
            rack_id TEXT PRIMARY KEY,
            zone_id TEXT NOT NULL,
            rack_code TEXT NOT NULL UNIQUE,
            aisle_number INTEGER NOT NULL,
            x_pos REAL NOT NULL DEFAULT 0.0,
            y_pos REAL NOT NULL DEFAULT 0.0,
            FOREIGN KEY (zone_id) REFERENCES warehouse_zones(zone_id)
        );

        CREATE TABLE IF NOT EXISTS warehouse_shelves (
            shelf_id TEXT PRIMARY KEY,
            rack_id TEXT NOT NULL,
            shelf_level INTEGER NOT NULL,
            height_from_floor_cm REAL NOT NULL,
            width_cm REAL NOT NULL,
            height_cm REAL NOT NULL,
            depth_cm REAL NOT NULL,
            max_weight_kg REAL NOT NULL,
            current_weight_kg REAL NOT NULL DEFAULT 0.0,
            FOREIGN KEY (rack_id) REFERENCES warehouse_racks(rack_id)
        );

        CREATE TABLE IF NOT EXISTS warehouse_shelf_items (
            id TEXT PRIMARY KEY,
            shelf_id TEXT NOT NULL,
            sku TEXT NOT NULL,
            item_name TEXT NOT NULL,
            quantity INTEGER NOT NULL DEFAULT 1,
            unit_width_cm REAL NOT NULL,
            unit_height_cm REAL NOT NULL,
            unit_depth_cm REAL NOT NULL,
            unit_weight_kg REAL NOT NULL,
            storage_form TEXT NOT NULL,
            turnover_velocity TEXT NOT NULL,
            is_upright_only INTEGER NOT NULL DEFAULT 0,
            updated_at INTEGER NOT NULL,
            FOREIGN KEY (shelf_id) REFERENCES warehouse_shelves(shelf_id)
        );

        CREATE TABLE IF NOT EXISTS warehouse_shelf_audits (
            audit_id TEXT PRIMARY KEY,
            shelf_id TEXT NOT NULL,
            scanned_sku TEXT NOT NULL,
            is_misplaced INTEGER NOT NULL,
            expected_sku TEXT,
            photo_sha256 TEXT,
            audit_timestamp INTEGER NOT NULL,
            FOREIGN KEY (shelf_id) REFERENCES warehouse_shelves(shelf_id)
        );

        CREATE INDEX IF NOT EXISTS idx_wh_shelves_rack ON warehouse_shelves(rack_id);
        CREATE INDEX IF NOT EXISTS idx_wh_items_shelf ON warehouse_shelf_items(shelf_id);
        CREATE INDEX IF NOT EXISTS idx_wh_items_sku ON warehouse_shelf_items(sku);
        "#,
    )
}

pub fn save_zone(conn: &Connection, zone: &WarehouseZone) -> Result<()> {
    conn.execute(
        r#"
        INSERT INTO warehouse_zones (zone_id, name, code, temperature_class, description)
        VALUES (?1, ?2, ?3, ?4, ?5)
        ON CONFLICT(zone_id) DO UPDATE SET
            name = excluded.name,
            code = excluded.code,
            temperature_class = excluded.temperature_class,
            description = excluded.description
        "#,
        params![
            zone.zone_id,
            zone.name,
            zone.code,
            zone.temperature_class,
            zone.description
        ],
    )?;
    Ok(())
}

pub fn save_rack(conn: &Connection, rack: &WarehouseRack) -> Result<()> {
    conn.execute(
        r#"
        INSERT INTO warehouse_racks (rack_id, zone_id, rack_code, aisle_number, x_pos, y_pos)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        ON CONFLICT(rack_id) DO UPDATE SET
            zone_id = excluded.zone_id,
            rack_code = excluded.rack_code,
            aisle_number = excluded.aisle_number,
            x_pos = excluded.x_pos,
            y_pos = excluded.y_pos
        "#,
        params![
            rack.rack_id,
            rack.zone_id,
            rack.rack_code,
            rack.aisle_number,
            rack.x_pos,
            rack.y_pos
        ],
    )?;
    Ok(())
}

pub fn save_shelf(conn: &Connection, shelf: &WarehouseShelf) -> Result<()> {
    conn.execute(
        r#"
        INSERT INTO warehouse_shelves (
            shelf_id, rack_id, shelf_level, height_from_floor_cm,
            width_cm, height_cm, depth_cm, max_weight_kg, current_weight_kg
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        ON CONFLICT(shelf_id) DO UPDATE SET
            rack_id = excluded.rack_id,
            shelf_level = excluded.shelf_level,
            height_from_floor_cm = excluded.height_from_floor_cm,
            width_cm = excluded.width_cm,
            height_cm = excluded.height_cm,
            depth_cm = excluded.depth_cm,
            max_weight_kg = excluded.max_weight_kg,
            current_weight_kg = excluded.current_weight_kg
        "#,
        params![
            shelf.shelf_id,
            shelf.rack_id,
            shelf.shelf_level,
            shelf.height_from_floor_cm,
            shelf.dimensions.width_cm,
            shelf.dimensions.height_cm,
            shelf.dimensions.depth_cm,
            shelf.max_weight_kg,
            shelf.current_weight_kg
        ],
    )?;
    Ok(())
}

pub fn get_shelf(conn: &Connection, shelf_id: &str) -> Result<WarehouseShelf> {
    conn.query_row(
        r#"
        SELECT shelf_id, rack_id, shelf_level, height_from_floor_cm,
               width_cm, height_cm, depth_cm, max_weight_kg, current_weight_kg
        FROM warehouse_shelves
        WHERE shelf_id = ?1
        "#,
        params![shelf_id],
        |row| {
            Ok(WarehouseShelf {
                shelf_id: row.get(0)?,
                rack_id: row.get(1)?,
                shelf_level: row.get(2)?,
                height_from_floor_cm: row.get(3)?,
                dimensions: Dimensions::new(row.get(4)?, row.get(5)?, row.get(6)?),
                max_weight_kg: row.get(7)?,
                current_weight_kg: row.get(8)?,
            })
        },
    )
}

pub fn list_all_shelves(conn: &Connection) -> Result<Vec<WarehouseShelf>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT shelf_id, rack_id, shelf_level, height_from_floor_cm,
               width_cm, height_cm, depth_cm, max_weight_kg, current_weight_kg
        FROM warehouse_shelves
        ORDER BY rack_id, shelf_level ASC
        "#,
    )?;

    let iter = stmt.query_map([], |row| {
        Ok(WarehouseShelf {
            shelf_id: row.get(0)?,
            rack_id: row.get(1)?,
            shelf_level: row.get(2)?,
            height_from_floor_cm: row.get(3)?,
            dimensions: Dimensions::new(row.get(4)?, row.get(5)?, row.get(6)?),
            max_weight_kg: row.get(7)?,
            current_weight_kg: row.get(8)?,
        })
    })?;

    let mut list = Vec::new();
    for s in iter {
        list.push(s?);
    }
    Ok(list)
}

pub fn allocate_item_to_shelf(
    conn: &Connection,
    shelf_id: &str,
    item: &WmsItem,
    qty: u32,
) -> Result<()> {
    let id = Uuid::now_v7().to_string();
    let now = chrono::Utc::now().timestamp_millis();
    let added_weight = item.weight_kg * (qty as f64);

    conn.execute(
        r#"
        INSERT INTO warehouse_shelf_items (
            id, shelf_id, sku, item_name, quantity, unit_width_cm, unit_height_cm,
            unit_depth_cm, unit_weight_kg, storage_form, turnover_velocity,
            is_upright_only, updated_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
        "#,
        params![
            id,
            shelf_id,
            item.sku,
            item.name,
            qty,
            item.dimensions.width_cm,
            item.dimensions.height_cm,
            item.dimensions.depth_cm,
            item.weight_kg,
            format!("{:?}", item.storage_form),
            format!("{:?}", item.turnover_velocity),
            if item.is_upright_only { 1 } else { 0 },
            now,
        ],
    )?;

    // Update loaded weight on shelf
    conn.execute(
        r#"
        UPDATE warehouse_shelves
        SET current_weight_kg = current_weight_kg + ?1
        WHERE shelf_id = ?2
        "#,
        params![added_weight, shelf_id],
    )?;

    Ok(())
}

pub fn record_shelf_audit(conn: &Connection, audit: &ShelfAuditProof) -> Result<()> {
    conn.execute(
        r#"
        INSERT INTO warehouse_shelf_audits (
            audit_id, shelf_id, scanned_sku, is_misplaced, expected_sku, photo_sha256, audit_timestamp
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        "#,
        params![
            audit.audit_id,
            audit.shelf_id,
            audit.scanned_sku,
            if audit.is_misplaced { 1 } else { 0 },
            audit.expected_sku,
            audit.photo_sha256,
            audit.audit_timestamp,
        ],
    )?;
    Ok(())
}

pub fn get_shelf_occupancy(conn: &Connection, shelf_id: &str) -> Result<(f64, f64)> {
    let shelf = get_shelf(conn, shelf_id)?;
    let shelf_vol = shelf.dimensions.volume_cm3();

    let items_vol: f64 = conn.query_row(
        r#"
        SELECT coalesce(sum(quantity * unit_width_cm * unit_height_cm * unit_depth_cm), 0.0)
        FROM warehouse_shelf_items
        WHERE shelf_id = ?1
        "#,
        params![shelf_id],
        |r| r.get(0),
    )?;

    let vol_pct = if shelf_vol > 0.0 {
        ((items_vol / shelf_vol) * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };

    let weight_pct = if shelf.max_weight_kg > 0.0 {
        ((shelf.current_weight_kg / shelf.max_weight_kg) * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };

    Ok((vol_pct, weight_pct))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wms::types::{StorageForm, TurnoverVelocity};

    #[test]
    fn test_wms_sqlite_schema_and_occupancy() {
        let conn = Connection::open_in_memory().unwrap();
        init_wms_schema(&conn).unwrap();

        let zone = WarehouseZone {
            zone_id: "Z-RETAIL".to_string(),
            name: "Main Retail Floor".to_string(),
            code: "Z-1".to_string(),
            temperature_class: "Ambient".to_string(),
            description: "Storefront shelves".to_string(),
        };
        save_zone(&conn, &zone).unwrap();

        let rack = WarehouseRack {
            rack_id: "R-10".to_string(),
            zone_id: "Z-RETAIL".to_string(),
            rack_code: "R10".to_string(),
            aisle_number: 1,
            x_pos: 10.0,
            y_pos: 20.0,
        };
        save_rack(&conn, &rack).unwrap();

        let shelf = WarehouseShelf {
            shelf_id: "S-101".to_string(),
            rack_id: "R-10".to_string(),
            shelf_level: 1,
            height_from_floor_cm: 110.0,
            dimensions: Dimensions::new(100.0, 40.0, 40.0), // 160,000 cm3
            max_weight_kg: 50.0,
            current_weight_kg: 0.0,
        };
        save_shelf(&conn, &shelf).unwrap();

        let item = WmsItem {
            sku: "SKU-TEST-1".to_string(),
            name: "Router Box".to_string(),
            dimensions: Dimensions::new(20.0, 20.0, 10.0), // 4,000 cm3 each, 1kg
            weight_kg: 1.0,
            storage_form: StorageForm::Cuboid,
            turnover_velocity: TurnoverVelocity::FastMover,
            is_upright_only: false,
        };

        // Allocate 10 units -> 40,000 cm3 (25% vol), 10kg (20% weight)
        allocate_item_to_shelf(&conn, "S-101", &item, 10).unwrap();

        let (vol_pct, weight_pct) = get_shelf_occupancy(&conn, "S-101").unwrap();
        assert!((vol_pct - 25.0).abs() < 0.1);
        assert!((weight_pct - 20.0).abs() < 0.1);

        // Audit entry
        let audit = ShelfAuditProof {
            audit_id: "AUD-1".to_string(),
            shelf_id: "S-101".to_string(),
            scanned_sku: "SKU-TEST-1".to_string(),
            is_misplaced: false,
            expected_sku: Some("SKU-TEST-1".to_string()),
            photo_sha256: Some("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".to_string()),
            audit_timestamp: 1727956800000,
        };
        record_shelf_audit(&conn, &audit).unwrap();
    }
}
