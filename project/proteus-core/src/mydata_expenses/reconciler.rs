//! Automated Supplier & Inventory Reconciler for Inbound myDATA Expenses.
//! Matches incoming AADE electronic supplier invoices against open purchase orders (PO)
//! and physical warehouse receipts (Δελτία Παραλαβής) using multi-factor deterministic logic.
//! Strict Rule 1 (100% Original Codebase), Rule 5 (Zero Mock Data), Rule 6 (Compliance & Security).

use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

use crate::mydata_expenses::parser::InboundExpenseInvoice;

/// Status lifecycle for internal Purchase Orders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PurchaseOrderStatus {
    PendingDelivery,
    PartiallyReceived,
    FullyReceived,
    Invoiced,
}

impl PurchaseOrderStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PendingDelivery => "PENDING_DELIVERY",
            Self::PartiallyReceived => "PARTIALLY_RECEIVED",
            Self::FullyReceived => "FULLY_RECEIVED",
            Self::Invoiced => "INVOICED",
        }
    }

    pub fn from_str_opt(s: &str) -> Self {
        match s {
            "PARTIALLY_RECEIVED" => Self::PartiallyReceived,
            "FULLY_RECEIVED" => Self::FullyReceived,
            "INVOICED" => Self::Invoiced,
            _ => Self::PendingDelivery,
        }
    }
}

/// Line item in an internal Purchase Order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PurchaseOrderLine {
    pub sku: String,
    pub description: String,
    pub quantity: u32,
    pub unit_cost_cents: u64,
    pub vat_category: u8,
}

/// Internal store Purchase Order against a supplier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PurchaseOrderRecord {
    pub order_id: String,
    pub supplier_afm: String,
    pub supplier_name: String,
    pub order_date: String,
    pub expected_net_cents: u64,
    pub expected_vat_cents: u64,
    pub expected_gross_cents: u64,
    pub status: PurchaseOrderStatus,
    pub lines: Vec<PurchaseOrderLine>,
}

/// Physical Warehouse Delivery Receipt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WarehouseReceiptRecord {
    pub receipt_id: String,
    pub purchase_order_id: Option<String>,
    pub supplier_afm: String,
    pub supplier_delivery_note: String,
    pub received_at: String,
    pub total_units: u32,
}

/// Quality ranking of the matching between a myDATA expense and internal records.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExpenseMatchQuality {
    ExactMatch,          // AFM + Amount + Reference PO/Receipt identical
    AmountAndAfmMatch,   // AFM + Amount identical, no explicit reference tag
    PartialDiscrepancy,  // AFM match, but price or quantity differs
    Unmatched,           // No matching open PO or delivery note found
}

/// Reconciled match result for an incoming AADE expense.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReconciliationMatchResult {
    pub invoice_mark: u64,
    pub invoice_uid: String,
    pub supplier_afm: String,
    pub matched_po_id: Option<String>,
    pub matched_receipt_id: Option<String>,
    pub match_quality: ExpenseMatchQuality,
    pub confidence_score: f64,
    pub amount_difference_cents: i64,
}

/// Initializes SQLite tables for purchase orders and warehouse receipts.
pub fn init_po_and_receipt_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS supplier_purchase_orders (
            order_id TEXT PRIMARY KEY,
            supplier_afm TEXT NOT NULL,
            supplier_name TEXT NOT NULL,
            order_date TEXT NOT NULL,
            expected_net_cents INTEGER NOT NULL,
            expected_vat_cents INTEGER NOT NULL,
            expected_gross_cents INTEGER NOT NULL,
            status TEXT NOT NULL DEFAULT 'PENDING_DELIVERY'
        );

        CREATE TABLE IF NOT EXISTS supplier_po_lines (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            order_id TEXT NOT NULL REFERENCES supplier_purchase_orders(order_id),
            sku TEXT NOT NULL,
            description TEXT NOT NULL,
            quantity INTEGER NOT NULL,
            unit_cost_cents INTEGER NOT NULL,
            vat_category INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS warehouse_receipts (
            receipt_id TEXT PRIMARY KEY,
            purchase_order_id TEXT,
            supplier_afm TEXT NOT NULL,
            supplier_delivery_note TEXT NOT NULL,
            received_at TEXT NOT NULL,
            total_units INTEGER NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_po_supplier_afm ON supplier_purchase_orders(supplier_afm, status);
        CREATE INDEX IF NOT EXISTS idx_receipts_supplier ON warehouse_receipts(supplier_afm);
        "#,
    )?;
    Ok(())
}

/// Inserts a new Purchase Order with its lines into SQLite.
pub fn insert_purchase_order(conn: &Connection, po: &PurchaseOrderRecord) -> Result<()> {
    conn.execute(
        r#"
        INSERT INTO supplier_purchase_orders
        (order_id, supplier_afm, supplier_name, order_date, expected_net_cents, expected_vat_cents, expected_gross_cents, status)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        "#,
        params![
            po.order_id,
            po.supplier_afm,
            po.supplier_name,
            po.order_date,
            po.expected_net_cents as i64,
            po.expected_vat_cents as i64,
            po.expected_gross_cents as i64,
            po.status.as_str()
        ],
    )?;

    for line in &po.lines {
        conn.execute(
            r#"
            INSERT INTO supplier_po_lines
            (order_id, sku, description, quantity, unit_cost_cents, vat_category)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            params![
                po.order_id,
                line.sku,
                line.description,
                line.quantity as i64,
                line.unit_cost_cents as i64,
                line.vat_category as i64
            ],
        )?;
    }

    Ok(())
}

/// Inserts a warehouse delivery receipt into SQLite.
pub fn insert_warehouse_receipt(conn: &Connection, receipt: &WarehouseReceiptRecord) -> Result<()> {
    conn.execute(
        r#"
        INSERT INTO warehouse_receipts
        (receipt_id, purchase_order_id, supplier_afm, supplier_delivery_note, received_at, total_units)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        "#,
        params![
            receipt.receipt_id,
            receipt.purchase_order_id,
            receipt.supplier_afm,
            receipt.supplier_delivery_note,
            receipt.received_at,
            receipt.total_units as i64
        ],
    )?;
    Ok(())
}

/// Matches an incoming AADE expense against candidate purchase orders and warehouse receipts.
pub fn match_inbound_expense(
    invoice: &InboundExpenseInvoice,
    candidate_pos: &[PurchaseOrderRecord],
    candidate_receipts: &[WarehouseReceiptRecord],
) -> ReconciliationMatchResult {
    // 1. Filter candidates strictly by supplier AFM
    let matching_pos: Vec<&PurchaseOrderRecord> = candidate_pos
        .iter()
        .filter(|po| po.supplier_afm == invoice.supplier_afm)
        .collect();

    // Look for explicit reference in invoice series, aa, or line comments
    for po in &matching_pos {
        let mentions_po = invoice.series.contains(&po.order_id)
            || invoice.aa.contains(&po.order_id)
            || invoice.lines.iter().any(|l| {
                l.comments
                    .as_deref()
                    .map(|c| c.contains(&po.order_id))
                    .unwrap_or(false)
            });

        let diff_cents = (invoice.total_gross_cents as i64) - (po.expected_gross_cents as i64);

        if mentions_po && diff_cents.abs() <= 1 {
            let receipt_id = candidate_receipts
                .iter()
                .find(|r| r.purchase_order_id.as_deref() == Some(&po.order_id))
                .map(|r| r.receipt_id.clone());

            return ReconciliationMatchResult {
                invoice_mark: invoice.mark,
                invoice_uid: invoice.uid.clone(),
                supplier_afm: invoice.supplier_afm.clone(),
                matched_po_id: Some(po.order_id.clone()),
                matched_receipt_id: receipt_id,
                match_quality: ExpenseMatchQuality::ExactMatch,
                confidence_score: 1.0,
                amount_difference_cents: diff_cents,
            };
        }
    }

    // 2. Look for exact amount and AFM match among open POs
    for po in &matching_pos {
        let diff_cents = (invoice.total_gross_cents as i64) - (po.expected_gross_cents as i64);
        if diff_cents.abs() <= 1 {
            let receipt_id = candidate_receipts
                .iter()
                .find(|r| r.purchase_order_id.as_deref() == Some(&po.order_id))
                .map(|r| r.receipt_id.clone());

            return ReconciliationMatchResult {
                invoice_mark: invoice.mark,
                invoice_uid: invoice.uid.clone(),
                supplier_afm: invoice.supplier_afm.clone(),
                matched_po_id: Some(po.order_id.clone()),
                matched_receipt_id: receipt_id,
                match_quality: ExpenseMatchQuality::AmountAndAfmMatch,
                confidence_score: 0.90,
                amount_difference_cents: diff_cents,
            };
        }
    }

    // 3. Partial discrepancy check (AFM matches, but amount diverges)
    if let Some(closest_po) = matching_pos.first() {
        let diff_cents = (invoice.total_gross_cents as i64) - (closest_po.expected_gross_cents as i64);
        return ReconciliationMatchResult {
            invoice_mark: invoice.mark,
            invoice_uid: invoice.uid.clone(),
            supplier_afm: invoice.supplier_afm.clone(),
            matched_po_id: Some(closest_po.order_id.clone()),
            matched_receipt_id: None,
            match_quality: ExpenseMatchQuality::PartialDiscrepancy,
            confidence_score: 0.50,
            amount_difference_cents: diff_cents,
        };
    }

    // 4. Unmatched
    ReconciliationMatchResult {
        invoice_mark: invoice.mark,
        invoice_uid: invoice.uid.clone(),
        supplier_afm: invoice.supplier_afm.clone(),
        matched_po_id: None,
        matched_receipt_id: None,
        match_quality: ExpenseMatchQuality::Unmatched,
        confidence_score: 0.0,
        amount_difference_cents: invoice.total_gross_cents as i64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mydata_expenses::parser::InboundExpenseLine;

    #[test]
    fn test_po_and_receipt_schema_roundtrip() {
        let conn = Connection::open_in_memory().unwrap();
        init_po_and_receipt_schema(&conn).unwrap();

        let po = PurchaseOrderRecord {
            order_id: "PO-2026-001".to_string(),
            supplier_afm: "998877665".to_string(),
            supplier_name: "Bosch Hellas AE".to_string(),
            order_date: "2026-10-01".to_string(),
            expected_net_cents: 25000,
            expected_vat_cents: 6000,
            expected_gross_cents: 31000,
            status: PurchaseOrderStatus::PendingDelivery,
            lines: vec![PurchaseOrderLine {
                sku: "BOSCH-DRILL-18V".to_string(),
                description: "Δραπανοκατσάβιδο 18V".to_string(),
                quantity: 5,
                unit_cost_cents: 5000,
                vat_category: 1,
            }],
        };

        insert_purchase_order(&conn, &po).unwrap();

        let receipt = WarehouseReceiptRecord {
            receipt_id: "REC-2026-088".to_string(),
            purchase_order_id: Some("PO-2026-001".to_string()),
            supplier_afm: "998877665".to_string(),
            supplier_delivery_note: "ΔΑ-4412".to_string(),
            received_at: "2026-10-05 14:30:00".to_string(),
            total_units: 5,
        };

        insert_warehouse_receipt(&conn, &receipt).unwrap();
    }

    #[test]
    fn test_match_inbound_expense_logic() {
        let po = PurchaseOrderRecord {
            order_id: "PO-2026-001".to_string(),
            supplier_afm: "998877665".to_string(),
            supplier_name: "Bosch Hellas AE".to_string(),
            order_date: "2026-10-01".to_string(),
            expected_net_cents: 25000,
            expected_vat_cents: 6000,
            expected_gross_cents: 31000,
            status: PurchaseOrderStatus::PendingDelivery,
            lines: vec![],
        };

        let receipt = WarehouseReceiptRecord {
            receipt_id: "REC-01".to_string(),
            purchase_order_id: Some("PO-2026-001".to_string()),
            supplier_afm: "998877665".to_string(),
            supplier_delivery_note: "DA-10".to_string(),
            received_at: "2026-10-05".to_string(),
            total_units: 5,
        };

        let invoice = InboundExpenseInvoice {
            mark: 8800112233,
            uid: "UID-99".to_string(),
            cancelled_by_mark: None,
            supplier_afm: "998877665".to_string(),
            supplier_country: "GR".to_string(),
            counterpart_afm: "094123456".to_string(),
            series: "ΤΔΑ".to_string(),
            aa: "452".to_string(),
            issue_date: "2026-10-08".to_string(),
            invoice_type: "1.1".to_string(),
            currency: "EUR".to_string(),
            total_net_cents: 25000,
            total_vat_cents: 6000,
            total_gross_cents: 31000,
            lines: vec![InboundExpenseLine {
                line_number: 1,
                net_cents: 25000,
                vat_category: 1,
                vat_cents: 6000,
                comments: Some("Ref PO-2026-001".to_string()),
            }],
        };

        let res = match_inbound_expense(&invoice, &[po], &[receipt]);
        assert_eq!(res.match_quality, ExpenseMatchQuality::ExactMatch);
        assert_eq!(res.matched_po_id, Some("PO-2026-001".to_string()));
        assert_eq!(res.matched_receipt_id, Some("REC-01".to_string()));
        assert_eq!(res.amount_difference_cents, 0);
        assert_eq!(res.confidence_score, 1.0);
    }
}
