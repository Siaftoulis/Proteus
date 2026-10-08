//! Proteus Live Logistics Sync & Automated Stock Level Arbitrage Engine.
//! Designed from first principles for zero-rewrite database federation.

pub mod arbitrage;
pub mod db;
pub mod types;

pub use arbitrage::StockArbitrageEngine;
pub use types::{
    LogisticsError, StockTransferRecommendation, StoreStockLevel, TransferOrder, TransferPriority,
    TransferStatus,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::federation::ChangeOp;
    use crate::federation::FederatedTransaction;
    use rusqlite::Connection;

    #[test]
    fn test_stock_level_metrics() {
        let stock = StoreStockLevel {
            store_id: "STORE_ATH".to_string(),
            sku: "IPHONE-15-SCR".to_string(),
            barcode: Some("5201234567890".to_string()),
            quantity_on_hand: 25.0,
            quantity_reserved: 5.0,
            min_reorder_point: 10.0,
            max_target_capacity: 50.0,
            unit_cost: 45.0,
            last_updated_at: "2026-10-08T12:00:00Z".to_string(),
        };

        assert_eq!(stock.available_quantity(), 20.0);
        assert_eq!(stock.deficit_quantity(), 0.0);
        assert_eq!(stock.surplus_quantity(), 10.0);

        let deficit_stock = StoreStockLevel {
            quantity_on_hand: 4.0,
            quantity_reserved: 1.0,
            min_reorder_point: 10.0,
            ..stock
        };
        assert_eq!(deficit_stock.available_quantity(), 3.0);
        assert_eq!(deficit_stock.deficit_quantity(), 7.0);
        assert_eq!(deficit_stock.surplus_quantity(), 0.0);
    }

    #[test]
    fn test_arbitrage_discovery_and_transfer_order_lifecycle() {
        let conn = Connection::open_in_memory().unwrap();
        StockArbitrageEngine::init_schema(&conn).unwrap();

        // Store A (Athens Hub): 40 on hand, 0 reserved, min 10 -> 30 surplus
        let stock_ath = StoreStockLevel {
            store_id: "STORE_ATH_01".to_string(),
            sku: "THERMAL-ROLL-80MM".to_string(),
            barcode: Some("5209990001112".to_string()),
            quantity_on_hand: 40.0,
            quantity_reserved: 0.0,
            min_reorder_point: 10.0,
            max_target_capacity: 100.0,
            unit_cost: 0.85,
            last_updated_at: "2026-10-08T10:00:00Z".to_string(),
        };
        StockArbitrageEngine::upsert_stock(&conn, &stock_ath).unwrap();

        // Store B (Patras Branch): 2 on hand, 0 reserved, min 12 -> 10 deficit
        let stock_pat = StoreStockLevel {
            store_id: "STORE_PAT_01".to_string(),
            sku: "THERMAL-ROLL-80MM".to_string(),
            barcode: Some("5209990001112".to_string()),
            quantity_on_hand: 2.0,
            quantity_reserved: 0.0,
            min_reorder_point: 12.0,
            max_target_capacity: 50.0,
            unit_cost: 0.85,
            last_updated_at: "2026-10-08T10:00:00Z".to_string(),
        };
        StockArbitrageEngine::upsert_stock(&conn, &stock_pat).unwrap();

        // Compute arbitrage recommendations
        let recs = StockArbitrageEngine::compute_arbitrage_recommendations(&conn, None).unwrap();
        assert_eq!(recs.len(), 1);
        let rec = &recs[0];
        assert_eq!(rec.sku, "THERMAL-ROLL-80MM");
        assert_eq!(rec.deficit_store_id, "STORE_PAT_01");
        assert_eq!(rec.surplus_store_id, "STORE_ATH_01");
        assert_eq!(rec.recommended_transfer_qty, 10.0);

        // Convert recommendation into transfer order
        let order = StockArbitrageEngine::create_transfer_order_from_recommendation(&conn, rec).unwrap();
        assert_eq!(order.status, TransferStatus::Approved);
        assert_eq!(order.requested_qty, 10.0);

        // Check stock reservation in Athens
        let ath_after_order = StockArbitrageEngine::get_stock(&conn, "STORE_ATH_01", "THERMAL-ROLL-80MM")
            .unwrap()
            .unwrap();
        assert_eq!(ath_after_order.quantity_on_hand, 40.0);
        assert_eq!(ath_after_order.quantity_reserved, 10.0);
        assert_eq!(ath_after_order.available_quantity(), 30.0);

        // Complete the transfer
        StockArbitrageEngine::complete_transfer_order(&conn, &order.order_id).unwrap();

        // Verify balances after receipt
        let ath_final = StockArbitrageEngine::get_stock(&conn, "STORE_ATH_01", "THERMAL-ROLL-80MM")
            .unwrap()
            .unwrap();
        assert_eq!(ath_final.quantity_on_hand, 30.0);
        assert_eq!(ath_final.quantity_reserved, 0.0);

        let pat_final = StockArbitrageEngine::get_stock(&conn, "STORE_PAT_01", "THERMAL-ROLL-80MM")
            .unwrap()
            .unwrap();
        assert_eq!(pat_final.quantity_on_hand, 12.0);
    }

    #[test]
    fn test_federated_inventory_ingestion() {
        let conn = Connection::open_in_memory().unwrap();
        StockArbitrageEngine::init_schema(&conn).unwrap();

        let tx = FederatedTransaction {
            sequence: 42,
            tx_id: "tx-fed-inv-001".to_string(),
            origin_store_id: "STORE_THES_01".to_string(),
            entity: "inventory".to_string(),
            record_id: "SKU-SAMSUNG-BAT".to_string(),
            op: ChangeOp::Update,
            payload_json: r#"{"sku":"SKU-SAMSUNG-BAT","qty_on_hand":18.0,"min_reorder":5.0,"unit_cost":12.50}"#.to_string(),
            payload_hash: "abcd".to_string(),
            natural_key: Some("SKU-SAMSUNG-BAT".to_string()),
            created_at: "2026-10-08T12:00:00Z".to_string(),
        };

        let ingested = StockArbitrageEngine::ingest_federated_inventory_change(&conn, &tx).unwrap();
        assert!(ingested);

        let stock = StockArbitrageEngine::get_stock(&conn, "STORE_THES_01", "SKU-SAMSUNG-BAT")
            .unwrap()
            .unwrap();
        assert_eq!(stock.store_id, "STORE_THES_01");
        assert_eq!(stock.sku, "SKU-SAMSUNG-BAT");
        assert_eq!(stock.quantity_on_hand, 18.0);
        assert_eq!(stock.min_reorder_point, 5.0);
    }
}
