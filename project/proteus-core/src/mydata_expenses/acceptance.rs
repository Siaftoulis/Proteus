//! 1-Click Accounting Acceptance & Cardex Credit Settlement Engine.
//! Bridges approved myDATA expenses into the financial Cardex accounts payable ledger,
//! auto-provisions supplier commercial accounts, updates PO statuses, and maintains audit trails.
//! Strict Rule 1 (100% Original Codebase), Rule 5 (Zero Mock Data), Rule 6 (Compliance & Security).

use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::cardex::db::{get_cardex_entity, init_cardex_schema, post_cardex_movement, save_cardex_entity};
use crate::cardex::types::{CardexEntity, CardexMovementType, EntityType};
use crate::mydata_expenses::ledger::{
    get_inbound_expense_by_mark, transition_expense_status, ExpenseLedgerError, InboundExpenseStatus,
};
use crate::mydata_expenses::reconciler::PurchaseOrderStatus;

/// Summary outcome of a 1-click expense acceptance into the accounting Cardex.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpenseAcceptanceResult {
    pub mark: u64,
    pub uid: String,
    pub cardex_entry_id: String,
    pub supplier_afm: String,
    pub amount_credited_eur: f64,
    pub new_supplier_balance_eur: f64,
    pub matched_po_updated: Option<String>,
}

/// 1-Click Accounting Acceptance:
/// - Verifies expense status.
/// - Auto-provisions supplier entity in `cardex_entities` if missing.
/// - Posts financial CREDIT movement (Accounts Payable liability) into Cardex.
/// - Marks expense as `Accepted` in `mydata_inbound_expenses`.
/// - Updates matched Purchase Order status to `Invoiced`.
pub fn accept_inbound_expense_to_cardex(
    conn: &Connection,
    mark: u64,
    operator: &str,
) -> std::result::Result<ExpenseAcceptanceResult, ExpenseLedgerError> {
    init_cardex_schema(conn)?;

    let expense = get_inbound_expense_by_mark(conn, mark)?
        .ok_or(ExpenseLedgerError::ExpenseNotFound(mark))?;

    // Auto-provision supplier in Cardex if not existing
    let supplier_opt = get_cardex_entity(conn, &expense.supplier_afm)?;
    if supplier_opt.is_none() {
        let name = expense
            .supplier_name
            .clone()
            .unwrap_or_else(|| format!("Προμηθευτής ΑΦΜ {}", expense.supplier_afm));

        let now_millis = Utc::now().timestamp_millis();
        let entity = CardexEntity {
            entity_id: expense.supplier_afm.clone(),
            entity_type: EntityType::Supplier,
            name,
            phone: None,
            email: None,
            credit_limit_eur: 0.0,
            payment_terms_days: 30,
            current_balance_eur: 0.0,
            created_at: now_millis,
        };
        save_cardex_entity(conn, &entity)?;
    }

    // Convert gross cents to euros
    let gross_eur = (expense.gross_cents as f64) / 100.0;
    let doc_num: i64 = expense.aa.trim().parse().unwrap_or(0);
    let desc = format!(
        "myDATA Εισαγωγή MARK: {} ({} {})",
        expense.mark, expense.series, expense.aa
    );

    // Calculate due date (issue date + 30 days)
    let due_date = format!("{}+30d", expense.issue_date);

    // Post Credit to Supplier Cardex (Accounts Payable)
    let cardex_entry = post_cardex_movement(
        conn,
        &expense.supplier_afm,
        CardexMovementType::Credit,
        &expense.invoice_type,
        &expense.series,
        doc_num,
        &desc,
        gross_eur,
        &expense.issue_date,
        &due_date,
    )?;

    // Transition state in ledger
    let acceptance_reason = format!("Εγκρίθηκε από χειριστή: {}", operator);
    transition_expense_status(
        conn,
        mark,
        InboundExpenseStatus::Accepted,
        Some(&acceptance_reason),
    )?;

    // Update matched PO if present
    let mut po_updated = None;
    if let Some(ref po_id) = expense.matched_po_id {
        let rows = conn.execute(
            r#"
            UPDATE supplier_purchase_orders
            SET status = ?1
            WHERE order_id = ?2
            "#,
            params![PurchaseOrderStatus::Invoiced.as_str(), po_id],
        )?;
        if rows > 0 {
            po_updated = Some(po_id.clone());
        }
    }

    Ok(ExpenseAcceptanceResult {
        mark: expense.mark,
        uid: expense.uid,
        cardex_entry_id: cardex_entry.entry_id,
        supplier_afm: expense.supplier_afm,
        amount_credited_eur: gross_eur,
        new_supplier_balance_eur: cardex_entry.running_balance_eur,
        matched_po_updated: po_updated,
    })
}

/// Formally disputes and rejects an inbound expense (e.g. unrecorded invoice or billing fraud).
pub fn reject_inbound_expense(
    conn: &Connection,
    mark: u64,
    dispute_reason: &str,
    operator: &str,
) -> std::result::Result<(), ExpenseLedgerError> {
    let reason = format!("Απορρίφθηκε από {}: {}", operator, dispute_reason);
    transition_expense_status(
        conn,
        mark,
        InboundExpenseStatus::Rejected,
        Some(&reason),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mydata_expenses::ledger::{init_inbound_expenses_schema, upsert_inbound_expense, InboundExpenseRecord};
    use crate::mydata_expenses::reconciler::{init_po_and_receipt_schema, insert_purchase_order, PurchaseOrderLine, PurchaseOrderRecord};

    #[test]
    fn test_accept_inbound_expense_to_cardex_flow() {
        let conn = Connection::open_in_memory().unwrap();
        init_inbound_expenses_schema(&conn).unwrap();
        init_cardex_schema(&conn).unwrap();
        init_po_and_receipt_schema(&conn).unwrap();

        // 1. Create open PO
        let po = PurchaseOrderRecord {
            order_id: "PO-2026-BOSCH".to_string(),
            supplier_afm: "998877665".to_string(),
            supplier_name: "Bosch Hellas AE".to_string(),
            order_date: "2026-10-01".to_string(),
            expected_net_cents: 10000,
            expected_vat_cents: 2400,
            expected_gross_cents: 12400,
            status: PurchaseOrderStatus::PendingDelivery,
            lines: vec![PurchaseOrderLine {
                sku: "BOSCH-1".to_string(),
                description: "Drill".to_string(),
                quantity: 1,
                unit_cost_cents: 10000,
                vat_category: 1,
            }],
        };
        insert_purchase_order(&conn, &po).unwrap();

        // 2. Inbound Expense from AADE myDATA
        let mut expense = InboundExpenseRecord::new(
            9001234,
            "UID-9001234",
            "998877665",
            "1.1",
            "ΤΔΑ",
            "102",
            "2026-10-09",
            10000,
            2400,
            12400,
        );
        expense.supplier_name = Some("Bosch Hellas AE".to_string());
        expense.matched_po_id = Some("PO-2026-BOSCH".to_string());
        upsert_inbound_expense(&conn, &expense).unwrap();

        // 3. 1-Click Accept
        let res = accept_inbound_expense_to_cardex(&conn, 9001234, "CPA Admin").unwrap();
        assert_eq!(res.mark, 9001234);
        assert_eq!(res.amount_credited_eur, 124.0);
        assert_eq!(res.new_supplier_balance_eur, 124.0);
        assert_eq!(res.matched_po_updated, Some("PO-2026-BOSCH".to_string()));

        // 4. Verify supplier exists in Cardex and has balance 124.0
        let supplier = get_cardex_entity(&conn, "998877665").unwrap().unwrap();
        assert_eq!(supplier.entity_type, EntityType::Supplier);
        assert_eq!(supplier.current_balance_eur, 124.0);

        // 5. Verify expense is Accepted
        let exp = get_inbound_expense_by_mark(&conn, 9001234).unwrap().unwrap();
        assert_eq!(exp.status, InboundExpenseStatus::Accepted);
    }

    #[test]
    fn test_reject_inbound_expense_flow() {
        let conn = Connection::open_in_memory().unwrap();
        init_inbound_expenses_schema(&conn).unwrap();

        let expense = InboundExpenseRecord::new(
            9009999,
            "UID-FAKE",
            "111222333",
            "1.1",
            "ΤΔΑ",
            "99",
            "2026-10-09",
            5000,
            1200,
            6200,
        );
        upsert_inbound_expense(&conn, &expense).unwrap();

        reject_inbound_expense(&conn, 9009999, "Άγνωστος προμηθευτής / απάτη", "CPA Admin").unwrap();

        let rejected = get_inbound_expense_by_mark(&conn, 9009999).unwrap().unwrap();
        assert_eq!(rejected.status, InboundExpenseStatus::Rejected);
        assert!(rejected.discrepancy_summary.unwrap().contains("απάτη"));
    }
}
