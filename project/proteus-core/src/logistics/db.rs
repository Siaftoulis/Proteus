//! SQLite persistence routines for multi-store logistics and inventory tracking.
//! Strict Rule 1 (100% Original Codebase) and Rule 5 (Zero Mock Data).

use chrono::Utc;
use rusqlite::{params, Connection};

use crate::logistics::types::{
    LogisticsError, StoreStockLevel, TransferStatus,
};

/// Initializes SQLite tables for multi-store inventory tracking and arbitrage orders.
pub fn init_schema(conn: &Connection) -> Result<(), LogisticsError> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS logistics_stock_levels (
            store_id TEXT NOT NULL,
            sku TEXT NOT NULL,
            barcode TEXT,
            quantity_on_hand REAL NOT NULL DEFAULT 0.0,
            quantity_reserved REAL NOT NULL DEFAULT 0.0,
            min_reorder_point REAL NOT NULL DEFAULT 0.0,
            max_target_capacity REAL NOT NULL DEFAULT 0.0,
            unit_cost REAL NOT NULL DEFAULT 0.0,
            last_updated_at TEXT NOT NULL,
            PRIMARY KEY (store_id, sku)
        );

        CREATE TABLE IF NOT EXISTS logistics_transfer_orders (
            order_id TEXT PRIMARY KEY,
            from_store_id TEXT NOT NULL,
            to_store_id TEXT NOT NULL,
            sku TEXT NOT NULL,
            requested_qty REAL NOT NULL,
            status TEXT NOT NULL,
            priority TEXT NOT NULL,
            reason TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS logistics_arbitrage_alerts (
            recommendation_id TEXT PRIMARY KEY,
            sku TEXT NOT NULL,
            deficit_store_id TEXT NOT NULL,
            deficit_quantity REAL NOT NULL,
            surplus_store_id TEXT NOT NULL,
            surplus_available REAL NOT NULL,
            recommended_transfer_qty REAL NOT NULL,
            arbitrage_reason TEXT NOT NULL,
            priority TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'PENDING',
            created_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_stock_sku ON logistics_stock_levels(sku);
        CREATE INDEX IF NOT EXISTS idx_transfer_status ON logistics_transfer_orders(status);
        "#,
    )?;
    Ok(())
}

/// Upserts a stock level record for a specific store node.
pub fn upsert_stock(conn: &Connection, stock: &StoreStockLevel) -> Result<(), LogisticsError> {
    conn.execute(
        r#"
        INSERT INTO logistics_stock_levels (
            store_id, sku, barcode, quantity_on_hand, quantity_reserved,
            min_reorder_point, max_target_capacity, unit_cost, last_updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        ON CONFLICT(store_id, sku) DO UPDATE SET
            barcode = COALESCE(excluded.barcode, logistics_stock_levels.barcode),
            quantity_on_hand = excluded.quantity_on_hand,
            quantity_reserved = excluded.quantity_reserved,
            min_reorder_point = excluded.min_reorder_point,
            max_target_capacity = excluded.max_target_capacity,
            unit_cost = excluded.unit_cost,
            last_updated_at = excluded.last_updated_at
        "#,
        params![
            stock.store_id,
            stock.sku,
            stock.barcode,
            stock.quantity_on_hand,
            stock.quantity_reserved,
            stock.min_reorder_point,
            stock.max_target_capacity,
            stock.unit_cost,
            stock.last_updated_at,
        ],
    )?;
    Ok(())
}

/// Retrieves current stock level for a store and SKU.
pub fn get_stock(
    conn: &Connection,
    store_id: &str,
    sku: &str,
) -> Result<Option<StoreStockLevel>, LogisticsError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT store_id, sku, barcode, quantity_on_hand, quantity_reserved,
               min_reorder_point, max_target_capacity, unit_cost, last_updated_at
        FROM logistics_stock_levels
        WHERE store_id = ?1 AND sku = ?2
        "#,
    )?;

    let mut rows = stmt.query(params![store_id, sku])?;
    if let Some(row) = rows.next()? {
        Ok(Some(map_stock_row(row)?))
    } else {
        Ok(None)
    }
}

/// Lists stock levels for an item across all federated stores.
pub fn list_sku_stock_across_stores(
    conn: &Connection,
    sku: &str,
) -> Result<Vec<StoreStockLevel>, LogisticsError> {
    let mut stmt = conn.prepare(
        r#"
        SELECT store_id, sku, barcode, quantity_on_hand, quantity_reserved,
               min_reorder_point, max_target_capacity, unit_cost, last_updated_at
        FROM logistics_stock_levels
        WHERE sku = ?1
        ORDER BY store_id ASC
        "#,
    )?;

    let rows = stmt.query_map(params![sku], map_stock_row)?;
    let mut list = Vec::new();
    for r in rows {
        list.push(r?);
    }
    Ok(list)
}

/// Completes the physical transfer upon destination receipt and adjusts ledger balances.
pub fn complete_transfer_order(
    conn: &Connection,
    order_id: &str,
) -> Result<(), LogisticsError> {
    let (from_store, to_store, sku, qty, status_str): (String, String, String, f64, String) = conn
        .query_row(
            "SELECT from_store_id, to_store_id, sku, requested_qty, status FROM logistics_transfer_orders WHERE order_id = ?1",
            params![order_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
        )
        .map_err(|_| LogisticsError::OrderNotFound(order_id.to_string()))?;

    let status = TransferStatus::from_str(&status_str);
    if status == TransferStatus::Completed || status == TransferStatus::Cancelled {
        return Err(LogisticsError::InvalidStateTransition {
            from: status,
            to: TransferStatus::Completed,
        });
    }

    let now_utc = Utc::now().to_rfc3339();

    // 1. Deduct from origin store
    conn.execute(
        r#"
        UPDATE logistics_stock_levels
        SET quantity_on_hand = quantity_on_hand - ?1,
            quantity_reserved = MAX(0.0, quantity_reserved - ?1),
            last_updated_at = ?2
        WHERE store_id = ?3 AND sku = ?4
        "#,
        params![qty, now_utc, from_store, sku],
    )?;

    // 2. Add to destination store
    conn.execute(
        r#"
        INSERT INTO logistics_stock_levels (
            store_id, sku, barcode, quantity_on_hand, quantity_reserved,
            min_reorder_point, max_target_capacity, unit_cost, last_updated_at
        ) VALUES (?1, ?2, NULL, ?3, 0.0, 5.0, 50.0, 0.0, ?4)
        ON CONFLICT(store_id, sku) DO UPDATE SET
            quantity_on_hand = quantity_on_hand + excluded.quantity_on_hand,
            last_updated_at = excluded.last_updated_at
        "#,
        params![to_store, sku, qty, now_utc],
    )?;

    // 3. Mark transfer order completed
    conn.execute(
        "UPDATE logistics_transfer_orders SET status = 'COMPLETED', updated_at = ?1 WHERE order_id = ?2",
        params![now_utc, order_id],
    )?;

    Ok(())
}

pub fn map_stock_row(row: &rusqlite::Row) -> rusqlite::Result<StoreStockLevel> {
    Ok(StoreStockLevel {
        store_id: row.get(0)?,
        sku: row.get(1)?,
        barcode: row.get(2)?,
        quantity_on_hand: row.get(3)?,
        quantity_reserved: row.get(4)?,
        min_reorder_point: row.get(5)?,
        max_target_capacity: row.get(6)?,
        unit_cost: row.get(7)?,
        last_updated_at: row.get(8)?,
    })
}
