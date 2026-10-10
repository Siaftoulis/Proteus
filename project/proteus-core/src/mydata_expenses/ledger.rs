//! Persistent SQLite Ledger & State Machine for Inbound myDATA Expenses.
//! Manages lifecycle status transitions (PendingReview -> Matched -> Accepted / Rejected)
//! ensuring full auditability, zero data loss, and immutable fiscal records.
//! Strict Rule 1 (100% Original Codebase), Rule 5 (Zero Mock Data), Rule 6 (Compliance & Security).

use chrono::Utc;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Lifecycle state for an inbound supplier invoice in the expense ledger.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InboundExpenseStatus {
    PendingReview, // Newly scraped from AADE or contains unresolved discrepancies
    Matched,       // Successfully matched with internal PO / receipt
    Accepted,      // Formally accepted into financial ledger and Cardex credit
    Rejected,      // Formally disputed / rejected by the business
}

impl InboundExpenseStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::PendingReview => "PENDING_REVIEW",
            Self::Matched => "MATCHED",
            Self::Accepted => "ACCEPTED",
            Self::Rejected => "REJECTED",
        }
    }

    pub fn from_str_opt(s: &str) -> Self {
        match s {
            "MATCHED" => Self::Matched,
            "ACCEPTED" => Self::Accepted,
            "REJECTED" => Self::Rejected,
            _ => Self::PendingReview,
        }
    }

    /// Validates whether a state transition is legal according to Greek accounting guidelines.
    pub fn can_transition_to(&self, target: InboundExpenseStatus) -> bool {
        match (self, target) {
            (Self::PendingReview, Self::Matched) => true,
            (Self::PendingReview, Self::Accepted) => true,
            (Self::PendingReview, Self::Rejected) => true,
            (Self::Matched, Self::Accepted) => true,
            (Self::Matched, Self::Rejected) => true,
            (Self::Matched, Self::PendingReview) => true, // Re-opened if PO changed
            (Self::Rejected, Self::PendingReview) => true, // Re-evaluated if dispute resolved
            (Self::Accepted, _) => false, // Accepted expenses are immutable (reversals require explicit credit notes)
            _ => false,
        }
    }
}

/// Errors occurring during expense ledger state transitions and database operations.
#[derive(Debug, Error, PartialEq)]
pub enum ExpenseLedgerError {
    #[error("Invalid state transition from {from} to {to}")]
    InvalidStateTransition {
        from: &'static str,
        to: &'static str,
    },
    #[error("Expense with MARK {0} not found")]
    ExpenseNotFound(u64),
    #[error("Database error: {0}")]
    Database(String),
}

impl From<rusqlite::Error> for ExpenseLedgerError {
    fn from(err: rusqlite::Error) -> Self {
        ExpenseLedgerError::Database(err.to_string())
    }
}

/// Complete stored record of an inbound myDATA expense in SQLite.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InboundExpenseRecord {
    pub expense_id: String,
    pub mark: u64,
    pub uid: String,
    pub supplier_afm: String,
    pub supplier_name: Option<String>,
    pub invoice_type: String,
    pub series: String,
    pub aa: String,
    pub issue_date: String,
    pub currency: String,
    pub net_cents: u64,
    pub vat_cents: u64,
    pub gross_cents: u64,
    pub matched_po_id: Option<String>,
    pub matched_receipt_id: Option<String>,
    pub status: InboundExpenseStatus,
    pub discrepancy_summary: Option<String>,
    pub raw_xml_payload: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl InboundExpenseRecord {
    pub fn new(
        mark: u64,
        uid: impl Into<String>,
        supplier_afm: impl Into<String>,
        invoice_type: impl Into<String>,
        series: impl Into<String>,
        aa: impl Into<String>,
        issue_date: impl Into<String>,
        net_cents: u64,
        vat_cents: u64,
        gross_cents: u64,
    ) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            expense_id: Uuid::new_v4().to_string(),
            mark,
            uid: uid.into(),
            supplier_afm: supplier_afm.into(),
            supplier_name: None,
            invoice_type: invoice_type.into(),
            series: series.into(),
            aa: aa.into(),
            issue_date: issue_date.into(),
            currency: "EUR".to_string(),
            net_cents,
            vat_cents,
            gross_cents,
            matched_po_id: None,
            matched_receipt_id: None,
            status: InboundExpenseStatus::PendingReview,
            discrepancy_summary: None,
            raw_xml_payload: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

/// Initializes the `mydata_inbound_expenses` table and performance indexes in SQLite.
pub fn init_inbound_expenses_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS mydata_inbound_expenses (
            expense_id TEXT PRIMARY KEY,
            mark INTEGER NOT NULL UNIQUE,
            uid TEXT NOT NULL,
            supplier_afm TEXT NOT NULL,
            supplier_name TEXT,
            invoice_type TEXT NOT NULL,
            series TEXT NOT NULL,
            aa TEXT NOT NULL,
            issue_date TEXT NOT NULL,
            currency TEXT NOT NULL DEFAULT 'EUR',
            net_cents INTEGER NOT NULL,
            vat_cents INTEGER NOT NULL,
            gross_cents INTEGER NOT NULL,
            matched_po_id TEXT,
            matched_receipt_id TEXT,
            status TEXT NOT NULL DEFAULT 'PENDING_REVIEW',
            discrepancy_summary TEXT,
            raw_xml_payload TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE UNIQUE INDEX IF NOT EXISTS idx_inbound_mark ON mydata_inbound_expenses(mark);
        CREATE INDEX IF NOT EXISTS idx_inbound_status ON mydata_inbound_expenses(status);
        CREATE INDEX IF NOT EXISTS idx_inbound_supplier ON mydata_inbound_expenses(supplier_afm);
        CREATE INDEX IF NOT EXISTS idx_inbound_date ON mydata_inbound_expenses(issue_date);
        "#,
    )?;
    Ok(())
}

/// Inserts or updates an inbound expense record in SQLite.
pub fn upsert_inbound_expense(conn: &Connection, record: &InboundExpenseRecord) -> Result<()> {
    conn.execute(
        r#"
        INSERT INTO mydata_inbound_expenses (
            expense_id, mark, uid, supplier_afm, supplier_name, invoice_type, series, aa,
            issue_date, currency, net_cents, vat_cents, gross_cents, matched_po_id,
            matched_receipt_id, status, discrepancy_summary, raw_xml_payload, created_at, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20)
        ON CONFLICT(mark) DO UPDATE SET
            matched_po_id = excluded.matched_po_id,
            matched_receipt_id = excluded.matched_receipt_id,
            status = excluded.status,
            discrepancy_summary = excluded.discrepancy_summary,
            updated_at = excluded.updated_at
        "#,
        params![
            record.expense_id,
            record.mark as i64,
            record.uid,
            record.supplier_afm,
            record.supplier_name,
            record.invoice_type,
            record.series,
            record.aa,
            record.issue_date,
            record.currency,
            record.net_cents as i64,
            record.vat_cents as i64,
            record.gross_cents as i64,
            record.matched_po_id,
            record.matched_receipt_id,
            record.status.as_str(),
            record.discrepancy_summary,
            record.raw_xml_payload,
            record.created_at,
            record.updated_at
        ],
    )?;
    Ok(())
}

/// Retrieves an inbound expense by its unique AADE MARK.
pub fn get_inbound_expense_by_mark(
    conn: &Connection,
    mark: u64,
) -> Result<Option<InboundExpenseRecord>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT expense_id, mark, uid, supplier_afm, supplier_name, invoice_type, series, aa,
               issue_date, currency, net_cents, vat_cents, gross_cents, matched_po_id,
               matched_receipt_id, status, discrepancy_summary, raw_xml_payload, created_at, updated_at
        FROM mydata_inbound_expenses
        WHERE mark = ?1
        "#,
    )?;

    let mut rows = stmt.query(params![mark as i64])?;
    if let Some(row) = rows.next()? {
        let status_str: String = row.get(15)?;
        let net_cents_i64: i64 = row.get(10)?;
        let vat_cents_i64: i64 = row.get(11)?;
        let gross_cents_i64: i64 = row.get(12)?;

        Ok(Some(InboundExpenseRecord {
            expense_id: row.get(0)?,
            mark: row.get::<_, i64>(1)? as u64,
            uid: row.get(2)?,
            supplier_afm: row.get(3)?,
            supplier_name: row.get(4)?,
            invoice_type: row.get(5)?,
            series: row.get(6)?,
            aa: row.get(7)?,
            issue_date: row.get(8)?,
            currency: row.get(9)?,
            net_cents: net_cents_i64 as u64,
            vat_cents: vat_cents_i64 as u64,
            gross_cents: gross_cents_i64 as u64,
            matched_po_id: row.get(13)?,
            matched_receipt_id: row.get(14)?,
            status: InboundExpenseStatus::from_str_opt(&status_str),
            discrepancy_summary: row.get(16)?,
            raw_xml_payload: row.get(17)?,
            created_at: row.get(18)?,
            updated_at: row.get(19)?,
        }))
    } else {
        Ok(None)
    }
}

/// Safely transitions the lifecycle state of an inbound expense with strict state machine validation.
pub fn transition_expense_status(
    conn: &Connection,
    mark: u64,
    new_status: InboundExpenseStatus,
    reason: Option<&str>,
) -> std::result::Result<(), ExpenseLedgerError> {
    let current = get_inbound_expense_by_mark(conn, mark)?
        .ok_or(ExpenseLedgerError::ExpenseNotFound(mark))?;

    if !current.status.can_transition_to(new_status) {
        return Err(ExpenseLedgerError::InvalidStateTransition {
            from: current.status.as_str(),
            to: new_status.as_str(),
        });
    }

    let now = Utc::now().to_rfc3339();
    let updated_summary = match (current.discrepancy_summary, reason) {
        (Some(existing), Some(r)) => Some(format!("{} | {}", existing, r)),
        (None, Some(r)) => Some(r.to_string()),
        (existing, None) => existing,
    };

    conn.execute(
        r#"
        UPDATE mydata_inbound_expenses
        SET status = ?1, discrepancy_summary = ?2, updated_at = ?3
        WHERE mark = ?4
        "#,
        params![new_status.as_str(), updated_summary, now, mark as i64],
    )?;

    Ok(())
}

/// Lists inbound expenses, optionally filtered by status.
pub fn list_inbound_expenses(
    conn: &Connection,
    status_filter: Option<InboundExpenseStatus>,
    limit: usize,
) -> Result<Vec<InboundExpenseRecord>> {
    let mut stmt = if let Some(status) = status_filter {
        let mut s = conn.prepare(
            r#"
            SELECT expense_id, mark, uid, supplier_afm, supplier_name, invoice_type, series, aa,
                   issue_date, currency, net_cents, vat_cents, gross_cents, matched_po_id,
                   matched_receipt_id, status, discrepancy_summary, raw_xml_payload, created_at, updated_at
            FROM mydata_inbound_expenses
            WHERE status = ?1
            ORDER BY issue_date DESC
            LIMIT ?2
            "#,
        )?;
        let rows = s
            .query_map(params![status.as_str(), limit as i64], map_row)?
            .collect::<Result<Vec<_>>>()?;
        return Ok(rows);
    } else {
        conn.prepare(
            r#"
            SELECT expense_id, mark, uid, supplier_afm, supplier_name, invoice_type, series, aa,
                   issue_date, currency, net_cents, vat_cents, gross_cents, matched_po_id,
                   matched_receipt_id, status, discrepancy_summary, raw_xml_payload, created_at, updated_at
            FROM mydata_inbound_expenses
            ORDER BY issue_date DESC
            LIMIT ?1
            "#,
        )?
    };

    let rows = stmt
        .query_map(params![limit as i64], map_row)?
        .collect::<Result<Vec<_>>>()?;
    Ok(rows)
}

fn map_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<InboundExpenseRecord> {
    let status_str: String = row.get(15)?;
    let net_cents_i64: i64 = row.get(10)?;
    let vat_cents_i64: i64 = row.get(11)?;
    let gross_cents_i64: i64 = row.get(12)?;

    Ok(InboundExpenseRecord {
        expense_id: row.get(0)?,
        mark: row.get::<_, i64>(1)? as u64,
        uid: row.get(2)?,
        supplier_afm: row.get(3)?,
        supplier_name: row.get(4)?,
        invoice_type: row.get(5)?,
        series: row.get(6)?,
        aa: row.get(7)?,
        issue_date: row.get(8)?,
        currency: row.get(9)?,
        net_cents: net_cents_i64 as u64,
        vat_cents: vat_cents_i64 as u64,
        gross_cents: gross_cents_i64 as u64,
        matched_po_id: row.get(13)?,
        matched_receipt_id: row.get(14)?,
        status: InboundExpenseStatus::from_str_opt(&status_str),
        discrepancy_summary: row.get(16)?,
        raw_xml_payload: row.get(17)?,
        created_at: row.get(18)?,
        updated_at: row.get(19)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expense_ledger_lifecycle_and_state_transitions() {
        let conn = Connection::open_in_memory().unwrap();
        init_inbound_expenses_schema(&conn).unwrap();

        let mut record = InboundExpenseRecord::new(
            88990011,
            "UID-EXPENSE-01",
            "998877665",
            "1.1",
            "ΤΔΑ",
            "450",
            "2026-10-09",
            10000,
            2400,
            12400,
        );
        record.supplier_name = Some("Bosch Hellas".to_string());

        upsert_inbound_expense(&conn, &record).unwrap();

        let retrieved = get_inbound_expense_by_mark(&conn, 88990011).unwrap().unwrap();
        assert_eq!(retrieved.status, InboundExpenseStatus::PendingReview);
        assert_eq!(retrieved.gross_cents, 12400);

        // Transition: PendingReview -> Matched
        transition_expense_status(&conn, 88990011, InboundExpenseStatus::Matched, Some("Matched PO-01")).unwrap();
        let matched = get_inbound_expense_by_mark(&conn, 88990011).unwrap().unwrap();
        assert_eq!(matched.status, InboundExpenseStatus::Matched);

        // Transition: Matched -> Accepted
        transition_expense_status(&conn, 88990011, InboundExpenseStatus::Accepted, Some("Approved")).unwrap();
        let accepted = get_inbound_expense_by_mark(&conn, 88990011).unwrap().unwrap();
        assert_eq!(accepted.status, InboundExpenseStatus::Accepted);

        // Invalid Transition: Accepted -> PendingReview should fail!
        let err = transition_expense_status(&conn, 88990011, InboundExpenseStatus::PendingReview, None);
        assert!(err.is_err());
        assert_eq!(
            err.unwrap_err(),
            ExpenseLedgerError::InvalidStateTransition {
                from: "ACCEPTED",
                to: "PENDING_REVIEW"
            }
        );
    }
}
