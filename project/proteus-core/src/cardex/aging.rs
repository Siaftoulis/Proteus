//! Statutory Balance Aging & Credit Limit Engine.
//! Implements 5-tier aging calculations and credit ceiling enforcement.
//! Strict Rule 1 (100% Original Codebase) and Rule 5 (Zero Mock Data).

use chrono::NaiveDate;
use rusqlite::{params, Connection, Result};

use crate::cardex::db::get_cardex_entity;
use crate::cardex::types::{BalanceAgingReport, CardexMovementType, EntityType};

/// Checks whether an additional purchase would exceed the customer's credit limit.
pub fn check_credit_limit_ok(
    conn: &Connection,
    entity_id: &str,
    additional_amount_eur: f64,
) -> Result<bool> {
    if let Some(entity) = get_cardex_entity(conn, entity_id)? {
        if entity.credit_limit_eur <= 0.0 {
            // 0.0 means no limit enforced / unlimited credit
            return Ok(true);
        }
        let prospective = entity.current_balance_eur + additional_amount_eur;
        Ok(prospective <= entity.credit_limit_eur)
    } else {
        Ok(true)
    }
}

/// Computes the 5-tier aging report for an entity's balance as of a given reference date.
pub fn calculate_balance_aging(
    conn: &Connection,
    entity_id: &str,
    as_of_date: &str,
) -> Result<BalanceAgingReport> {
    let entity = get_cardex_entity(conn, entity_id)?
        .ok_or_else(|| rusqlite::Error::QueryReturnedNoRows)?;

    let as_of = NaiveDate::parse_from_str(as_of_date, "%Y-%m-%d")
        .unwrap_or_else(|_| chrono::Local::now().date_naive());

    let mut stmt = conn.prepare(
        r#"
        SELECT movement_type, amount_eur, issue_date
        FROM cardex_entries
        WHERE entity_id = ?1
        ORDER BY issue_date ASC
        "#,
    )?;

    let mut current_0_30 = 0.0;
    let mut overdue_31_60 = 0.0;
    let mut overdue_61_90 = 0.0;
    let mut overdue_91_120 = 0.0;
    let mut overdue_120_plus = 0.0;

    let rows = stmt.query_map(params![entity_id], |r| {
        let m_str: String = r.get(0)?;
        let m_type = CardexMovementType::from_str(&m_str).unwrap_or(CardexMovementType::Debit);
        let amt: f64 = r.get(1)?;
        let date_str: String = r.get(2)?;
        Ok((m_type, amt, date_str))
    })?;

    for row in rows {
        let (m_type, amt, date_str) = row?;
        let is_positive = match (entity.entity_type, m_type) {
            (EntityType::Customer, CardexMovementType::Debit) => true,
            (EntityType::Customer, CardexMovementType::Credit) => false,
            (EntityType::Supplier, CardexMovementType::Credit) => true,
            (EntityType::Supplier, CardexMovementType::Debit) => false,
        };

        let signed_amt = if is_positive { amt } else { -amt };

        if let Ok(entry_date) = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
            let age_days = (as_of - entry_date).num_days();
            if age_days <= 30 {
                current_0_30 += signed_amt;
            } else if age_days <= 60 {
                overdue_31_60 += signed_amt;
            } else if age_days <= 90 {
                overdue_61_90 += signed_amt;
            } else if age_days <= 120 {
                overdue_91_120 += signed_amt;
            } else {
                overdue_120_plus += signed_amt;
            }
        } else {
            current_0_30 += signed_amt;
        }
    }

    Ok(BalanceAgingReport {
        entity_id: entity_id.to_string(),
        total_balance_eur: entity.current_balance_eur,
        current_0_30_eur: ((current_0_30 * 100.0).round() / 100.0).max(0.0),
        overdue_31_60_eur: ((overdue_31_60 * 100.0).round() / 100.0).max(0.0),
        overdue_61_90_eur: ((overdue_61_90 * 100.0).round() / 100.0).max(0.0),
        overdue_91_120_eur: ((overdue_91_120 * 100.0).round() / 100.0).max(0.0),
        overdue_120_plus_eur: ((overdue_120_plus * 100.0).round() / 100.0).max(0.0),
    })
}
