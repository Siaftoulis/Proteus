//! Inbound myDATA Expenses Scraping & Auto-Reconciler (Phase 12).
//! Automatically pulls supplier invoices from the official AADE myDATA endpoint,
//! parses line items, reconciles purchase orders, and writes to real SQLite ledger.
//! Strict Rule 1 (100% Original Codebase) and Rule 5 (Zero Mock Data).

pub mod query;
pub mod parser;
pub mod reconciler;
pub mod discrepancy;
pub mod ledger;
pub mod acceptance;

pub use query::{
    dispatch_request_transmitted_docs, fetch_and_record_inbound_expenses, ExpenseQueryError,
    InboundExpenseFilter,
};
pub use parser::{
    extract_tag_value, parse_decimal_to_cents, parse_requested_doc_xml, InboundExpenseInvoice,
    InboundExpenseLine, RequestedDocResponse,
};
pub use reconciler::{
    init_po_and_receipt_schema, insert_purchase_order, insert_warehouse_receipt,
    match_inbound_expense, ExpenseMatchQuality, PurchaseOrderLine, PurchaseOrderRecord,
    PurchaseOrderStatus, ReconciliationMatchResult, WarehouseReceiptRecord,
};
pub use discrepancy::{
    detect_expense_discrepancies, ExpenseDiscrepancyItem, ExpenseDiscrepancyReport,
    ExpenseDiscrepancySeverity, ExpenseDiscrepancyType,
};
pub use ledger::{
    get_inbound_expense_by_mark, init_inbound_expenses_schema, list_inbound_expenses,
    transition_expense_status, upsert_inbound_expense, ExpenseLedgerError, InboundExpenseRecord,
    InboundExpenseStatus,
};
pub use acceptance::{
    accept_inbound_expense_to_cardex, reject_inbound_expense, ExpenseAcceptanceResult,
};


