//! Dropshipping Automation & Supplier Catalog Ingestion Engine for Proteus.
//! Strict Rule 1 (100% Original Codebase), Rule 3 (<400 lines), and Rule 5 (Zero Mock Data).
//! Enables direct wholesale feed ingestion, rule-based retail margin calculations,
//! live stock/price delta synchronization, and automated order fulfillment routing.

pub mod pricing;
pub mod ingest;
pub mod dispatch;

pub use pricing::{
    init_dropship_pricing_tables, insert_margin_rule, insert_supplier, resolve_optimal_rule,
    upsert_dropship_item, DropshipItem, DropshipSupplier, MarginCalculator, MarginRule,
};
pub use ingest::{
    DropshipRawJsonItem, FeedColumnMapping, FeedIngestionReport, SupplierFeedIngestor,
};
pub use dispatch::{
    generate_fulfillment_slip, init_dropship_dispatch_tables, record_tracking_update,
    route_inbound_order_to_suppliers, DispatchItem, DispatchStatus, SupplierDispatchRecord,
    SupplierTrackingCallback,
};
