//! SQLite persistence for store items and supplier catalog history.

use rusqlite::{params, Connection};
use super::types::*;

/// Initializes database tables for store items and supplier catalog history.
pub fn init_supplier_catalog_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS store_items (
            sku TEXT PRIMARY KEY,
            barcode TEXT,
            name TEXT NOT NULL,
            cost_price REAL NOT NULL,
            retail_price REAL NOT NULL,
            unit TEXT NOT NULL DEFAULT 'ΤΕΜ',
            category TEXT NOT NULL DEFAULT 'ΓΕΝΙΚΑ',
            stock_qty REAL NOT NULL DEFAULT 0.0,
            updated_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_store_items_barcode ON store_items(barcode);

        CREATE TABLE IF NOT EXISTS supplier_catalog_history (
            id TEXT PRIMARY KEY,
            supplier_name TEXT NOT NULL,
            imported_at TEXT NOT NULL,
            total_items INTEGER NOT NULL,
            increases INTEGER NOT NULL,
            decreases INTEGER NOT NULL,
            avg_change_pct REAL NOT NULL
        );",
    )?;
    Ok(())
}

/// Inserts or updates an individual store inventory item in SQLite.
pub fn create_or_update_item(conn: &Connection, item: &StoreItem) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO store_items (sku, barcode, name, cost_price, retail_price, unit, category, stock_qty, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
         ON CONFLICT(sku) DO UPDATE SET
            barcode = excluded.barcode,
            name = excluded.name,
            cost_price = excluded.cost_price,
            retail_price = excluded.retail_price,
            unit = excluded.unit,
            category = excluded.category,
            stock_qty = excluded.stock_qty,
            updated_at = excluded.updated_at",
        params![
            item.sku,
            item.barcode,
            item.name,
            item.cost_price,
            item.retail_price,
            item.unit,
            item.category,
            item.stock_qty,
            item.updated_at,
        ],
    )?;
    Ok(())
}

/// Queries all items from SQLite.
pub fn list_store_items(conn: &Connection) -> Result<Vec<StoreItem>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT sku, barcode, name, cost_price, retail_price, unit, category, stock_qty, updated_at
         FROM store_items ORDER BY name ASC",
    )?;

    let rows = stmt.query_map([], |r| {
        Ok(StoreItem {
            sku: r.get(0)?,
            barcode: r.get(1)?,
            name: r.get(2)?,
            cost_price: r.get(3)?,
            retail_price: r.get(4)?,
            unit: r.get(5)?,
            category: r.get(6)?,
            stock_qty: r.get(7)?,
            updated_at: r.get(8)?,
        })
    })?;

    let mut items = Vec::new();
    for r in rows {
        items.push(r?);
    }
    Ok(items)
}
