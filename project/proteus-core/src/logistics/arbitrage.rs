//! Core multi-store stock level arbitrage and live logistics synchronization engine.
//! Evaluates inter-store inventory imbalances and proposes automated rebalancing transfers.

use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::federation::FederatedTransaction;
use crate::logistics::db;
use crate::logistics::types::{
    LogisticsError, StockTransferRecommendation, StoreStockLevel, TransferOrder, TransferPriority,
    TransferStatus,
};

#[derive(Debug, Serialize, Deserialize)]
struct FederatedStockPayload {
    pub sku: String,
    pub barcode: Option<String>,
    pub qty_on_hand: f64,
    pub qty_reserved: Option<f64>,
    pub min_reorder: Option<f64>,
    pub max_capacity: Option<f64>,
    pub unit_cost: Option<f64>,
}

pub struct StockArbitrageEngine;

impl StockArbitrageEngine {
    /// Initializes SQLite tables for multi-store inventory tracking and arbitrage orders.
    pub fn init_schema(conn: &Connection) -> Result<(), LogisticsError> {
        db::init_schema(conn)
    }

    /// Upserts a stock level record for a specific store node.
    pub fn upsert_stock(conn: &Connection, stock: &StoreStockLevel) -> Result<(), LogisticsError> {
        db::upsert_stock(conn, stock)
    }

    /// Retrieves current stock level for a store and SKU.
    pub fn get_stock(
        conn: &Connection,
        store_id: &str,
        sku: &str,
    ) -> Result<Option<StoreStockLevel>, LogisticsError> {
        db::get_stock(conn, store_id, sku)
    }

    /// Lists stock levels for an item across all federated stores.
    pub fn list_sku_stock_across_stores(
        conn: &Connection,
        sku: &str,
    ) -> Result<Vec<StoreStockLevel>, LogisticsError> {
        db::list_sku_stock_across_stores(conn, sku)
    }

    /// Evaluates inventory deficits and surpluses across stores to compute arbitrage transfer proposals.
    pub fn compute_arbitrage_recommendations(
        conn: &Connection,
        sku_filter: Option<&str>,
    ) -> Result<Vec<StockTransferRecommendation>, LogisticsError> {
        let mut query = String::from(
            r#"
            SELECT store_id, sku, barcode, quantity_on_hand, quantity_reserved,
                   min_reorder_point, max_target_capacity, unit_cost, last_updated_at
            FROM logistics_stock_levels
            "#,
        );
        if sku_filter.is_some() {
            query.push_str(" WHERE sku = ?1");
        }
        query.push_str(" ORDER BY sku ASC, store_id ASC");

        let mut stmt = conn.prepare(&query)?;
        let all_stock: Vec<StoreStockLevel> = if let Some(sku) = sku_filter {
            stmt.query_map(params![sku], db::map_stock_row)?
                .collect::<rusqlite::Result<Vec<_>>>()?
        } else {
            stmt.query_map([], db::map_stock_row)?
                .collect::<rusqlite::Result<Vec<_>>>()?
        };

        // Group by SKU
        let mut sku_map: std::collections::HashMap<String, Vec<StoreStockLevel>> =
            std::collections::HashMap::new();
        for item in all_stock {
            sku_map.entry(item.sku.clone()).or_default().push(item);
        }

        let mut recommendations = Vec::new();
        let now_utc = Utc::now().to_rfc3339();

        for (sku, items) in sku_map {
            let mut deficits: Vec<(String, f64, f64)> = items
                .iter()
                .filter(|i| i.deficit_quantity() > 0.0)
                .map(|i| (i.store_id.clone(), i.deficit_quantity(), i.available_quantity()))
                .collect();
            deficits.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

            let mut surpluses: Vec<(String, f64)> = items
                .iter()
                .filter(|i| i.surplus_quantity() > 0.0)
                .map(|i| (i.store_id.clone(), i.surplus_quantity()))
                .collect();
            surpluses.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

            for (deficit_store, mut deficit_qty, avail) in deficits {
                for (surplus_store, surplus_avail) in surpluses.iter_mut() {
                    if deficit_qty <= 0.0 || *surplus_avail <= 0.0 {
                        continue;
                    }
                    if deficit_store == *surplus_store {
                        continue;
                    }

                    let transfer_qty = deficit_qty.min(*surplus_avail);
                    *surplus_avail -= transfer_qty;
                    deficit_qty -= transfer_qty;

                    let priority = if avail == 0.0 {
                        TransferPriority::Urgent
                    } else if avail < (deficit_qty / 2.0) {
                        TransferPriority::High
                    } else {
                        TransferPriority::Normal
                    };

                    let rec_id = format!("ARB-{}", Uuid::now_v7());
                    let reason = format!(
                        "Inter-store rebalancing: {} transferred from {} to eliminate stockout risk",
                        transfer_qty, surplus_store
                    );

                    let rec = StockTransferRecommendation {
                        recommendation_id: rec_id.clone(),
                        sku: sku.clone(),
                        deficit_store_id: deficit_store.clone(),
                        deficit_quantity: deficit_qty + transfer_qty,
                        surplus_store_id: surplus_store.clone(),
                        surplus_available: *surplus_avail + transfer_qty,
                        recommended_transfer_qty: transfer_qty,
                        arbitrage_reason: reason.clone(),
                        priority,
                    };

                    conn.execute(
                        r#"
                        INSERT OR REPLACE INTO logistics_arbitrage_alerts (
                            recommendation_id, sku, deficit_store_id, deficit_quantity,
                            surplus_store_id, surplus_available, recommended_transfer_qty,
                            arbitrage_reason, priority, status, created_at
                        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'PENDING', ?10)
                        "#,
                        params![
                            rec.recommendation_id,
                            rec.sku,
                            rec.deficit_store_id,
                            rec.deficit_quantity,
                            rec.surplus_store_id,
                            rec.surplus_available,
                            rec.recommended_transfer_qty,
                            rec.arbitrage_reason,
                            rec.priority.as_str(),
                            now_utc,
                        ],
                    )?;

                    recommendations.push(rec);
                }
            }
        }

        Ok(recommendations)
    }

    /// Converts an approved arbitrage recommendation into a formal TransferOrder with inventory reservation.
    pub fn create_transfer_order_from_recommendation(
        conn: &Connection,
        rec: &StockTransferRecommendation,
    ) -> Result<TransferOrder, LogisticsError> {
        let origin_stock = Self::get_stock(conn, &rec.surplus_store_id, &rec.sku)?
            .ok_or_else(|| LogisticsError::OrderNotFound(format!("Store {} stock not found", rec.surplus_store_id)))?;

        if origin_stock.available_quantity() < rec.recommended_transfer_qty {
            return Err(LogisticsError::InsufficientStock {
                available: origin_stock.available_quantity(),
                requested: rec.recommended_transfer_qty,
            });
        }

        let order_id = format!("XFER-{}", Uuid::now_v7());
        let now_utc = Utc::now().to_rfc3339();

        conn.execute(
            r#"
            INSERT INTO logistics_transfer_orders (
                order_id, from_store_id, to_store_id, sku, requested_qty,
                status, priority, reason, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            params![
                order_id,
                rec.surplus_store_id,
                rec.deficit_store_id,
                rec.sku,
                rec.recommended_transfer_qty,
                TransferStatus::Approved.as_str(),
                rec.priority.as_str(),
                rec.arbitrage_reason,
                now_utc,
                now_utc,
            ],
        )?;

        // Reserve stock in source store
        conn.execute(
            r#"
            UPDATE logistics_stock_levels
            SET quantity_reserved = quantity_reserved + ?1, last_updated_at = ?2
            WHERE store_id = ?3 AND sku = ?4
            "#,
            params![rec.recommended_transfer_qty, now_utc, rec.surplus_store_id, rec.sku],
        )?;

        // Mark alert as approved
        conn.execute(
            "UPDATE logistics_arbitrage_alerts SET status = 'APPROVED' WHERE recommendation_id = ?1",
            params![rec.recommendation_id],
        )?;

        Ok(TransferOrder {
            order_id,
            from_store_id: rec.surplus_store_id.clone(),
            to_store_id: rec.deficit_store_id.clone(),
            sku: rec.sku.clone(),
            requested_qty: rec.recommended_transfer_qty,
            status: TransferStatus::Approved,
            priority: rec.priority,
            reason: rec.arbitrage_reason.clone(),
            created_at: now_utc.clone(),
            updated_at: now_utc,
        })
    }

    /// Completes the physical transfer upon destination receipt and adjusts ledger balances.
    pub fn complete_transfer_order(
        conn: &Connection,
        order_id: &str,
    ) -> Result<(), LogisticsError> {
        db::complete_transfer_order(conn, order_id)
    }

    /// Ingests incoming federated inventory transactions directly into local stock levels.
    pub fn ingest_federated_inventory_change(
        conn: &Connection,
        tx: &FederatedTransaction,
    ) -> Result<bool, LogisticsError> {
        if tx.entity != "inventory" && tx.entity != "stock" {
            return Ok(false);
        }

        let payload: FederatedStockPayload = serde_json::from_str(&tx.payload_json)?;
        let now_utc = Utc::now().to_rfc3339();

        let stock = StoreStockLevel {
            store_id: tx.origin_store_id.clone(),
            sku: payload.sku,
            barcode: payload.barcode,
            quantity_on_hand: payload.qty_on_hand,
            quantity_reserved: payload.qty_reserved.unwrap_or(0.0),
            min_reorder_point: payload.min_reorder.unwrap_or(5.0),
            max_target_capacity: payload.max_capacity.unwrap_or(100.0),
            unit_cost: payload.unit_cost.unwrap_or(0.0),
            last_updated_at: now_utc,
        };

        db::upsert_stock(conn, &stock)?;
        Ok(true)
    }
}
