//! Core types for the Supplier Price List & Catalog Reconciliation Engine.

use serde::{Deserialize, Serialize};

/// Authentic store inventory item tracked in SQLite.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoreItem {
    pub sku: String,
    pub barcode: Option<String>,
    pub name: String,
    pub cost_price: f64,
    pub retail_price: f64,
    pub unit: String,
    pub category: String,
    pub stock_qty: f64,
    pub updated_at: String,
}

/// Variance record between incoming supplier catalog price and current store cost.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PriceVariance {
    pub sku: String,
    pub barcode: Option<String>,
    pub name: String,
    pub old_cost: f64,
    pub new_cost: f64,
    pub cost_diff: f64,
    pub cost_diff_pct: f64,
    pub old_retail: f64,
    pub new_retail: f64,
    pub unit: String,
    pub category: String,
    pub is_new_item: bool,
}

/// Full reconciliation report comparing supplier invoice/catalog against local store records.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReconciliationReport {
    pub supplier_name: String,
    pub total_rows: usize,
    pub matched_existing: usize,
    pub new_items: usize,
    pub price_increases: usize,
    pub price_decreases: usize,
    pub price_unchanged: usize,
    pub avg_cost_change_pct: f64,
    pub items: Vec<PriceVariance>,
}
