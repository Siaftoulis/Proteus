//! Proteus Supplier Price List & Catalog Reconciliation Engine.
//! Leverages SMLM and Universal Reconciler to ingest messy vendor CSV/TSV catalogs,
//! detect price hikes/reductions, calculate suggested retail margins, and update SQLite inventory.
//! 100% original bespoke code. Adheres to Rule 5 (Zero Mock Data).
//! Modularized under Rule 3 (<400 lines per file).

pub mod types;
pub mod db;
pub mod reconciler;

pub use types::*;
pub use db::*;
pub use reconciler::*;

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use rusqlite::Connection;

    #[test]
    fn test_csv_parsing_and_decimal_cleaning() {
        let raw = "ΚΩΔΙΚΟΣ;ΠΕΡΙΓΡΑΦΗ;ΤΙΜΗ;BARCODE;ΜΟΝΑΔΑ\n\
                   BOS-100;Δράπανο Κρουστικό;85,50 €;5201234567890;ΤΕΜ\n\
                   WUR-200;Σπρέι Σιλικόνης;4,20;4012345678901;ΤΕΜ\n";

        let (headers, rows) = parse_delimited_catalog(raw);
        assert_eq!(headers.len(), 5);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0][0], "BOS-100");
        assert_eq!(parse_euro_float(&rows[0][2]), 85.50);
        assert_eq!(parse_euro_float(&rows[1][2]), 4.20);
    }

    #[test]
    fn test_reconciliation_workflow_live_sqlite() {
        let conn = Connection::open_in_memory().unwrap();
        init_supplier_catalog_schema(&conn).unwrap();

        // Seed 1 existing item in store inventory
        let initial_item = StoreItem {
            sku: "BOS-100".into(),
            barcode: Some("5201234567890".into()),
            name: "Δράπανο Κρουστικό 18V".into(),
            cost_price: 80.00,
            retail_price: 110.00,
            unit: "ΤΕΜ".into(),
            category: "ΕΡΓΑΛΕΙΑ".into(),
            stock_qty: 5.0,
            updated_at: Utc::now().to_rfc3339(),
        };
        create_or_update_item(&conn, &initial_item).unwrap();

        // Incoming vendor CSV with price increase (+6.875%) and 1 new product
        let incoming_csv = "ΚΩΔΙΚΟΣ;ΠΕΡΙΓΡΑΦΗ;ΤΙΜΗ;BARCODE;ΜΟΝΑΔΑ\n\
                            BOS-100;Δράπανο Κρουστικό 18V;85,50;5201234567890;ΤΕΜ\n\
                            NEW-999;Νέο Κατσαβίδι Torx;3,50;5209999999999;ΤΕΜ\n";

        let report = reconcile_catalog(&conn, "BOSCH Hellas", incoming_csv, 35.0).unwrap();

        assert_eq!(report.total_rows, 2);
        assert_eq!(report.matched_existing, 1);
        assert_eq!(report.new_items, 1);
        assert_eq!(report.price_increases, 1);

        let bos = report.items.iter().find(|i| i.sku == "BOS-100").unwrap();
        assert_eq!(bos.old_cost, 80.00);
        assert_eq!(bos.new_cost, 85.50);
        assert_eq!(bos.cost_diff, 5.50);
        assert!(!bos.is_new_item);
        // With 35% markup on 85.50: 85.50 * 1.35 = 115.425 -> 115.43
        assert_eq!(bos.new_retail, 115.43);

        let new_prod = report.items.iter().find(|i| i.sku == "NEW-999").unwrap();
        assert!(new_prod.is_new_item);
        assert_eq!(new_prod.new_cost, 3.50);
        // With 35% markup on 3.50: 3.50 * 1.35 = 4.725 -> 4.73
        assert_eq!(new_prod.new_retail, 4.73);

        // Apply changes to SQLite
        let applied = apply_price_reconciliation(&conn, &report).unwrap();
        assert_eq!(applied, 2);

        // Verify SQLite table now contains both updated items
        let items = list_store_items(&conn).unwrap();
        assert_eq!(items.len(), 2);

        let updated_bos = items.iter().find(|i| i.sku == "BOS-100").unwrap();
        assert_eq!(updated_bos.cost_price, 85.50);
        assert_eq!(updated_bos.retail_price, 115.43);
    }
}
