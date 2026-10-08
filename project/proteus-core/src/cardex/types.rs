//! Types and Data Models for Customer & Supplier Financial Cardex.
//! Strict Rule 1 (100% Original Codebase) and Rule 5 (Zero Mock Data).

use serde::{Deserialize, Serialize};

/// Entity role type in financial ledgers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityType {
    Customer,
    Supplier,
}

impl EntityType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Customer => "CUSTOMER",
            Self::Supplier => "SUPPLIER",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_uppercase().as_str() {
            "CUSTOMER" => Some(Self::Customer),
            "SUPPLIER" => Some(Self::Supplier),
            _ => None,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Customer => "Πελάτης",
            Self::Supplier => "Προμηθευτής",
        }
    }
}

/// Nature of a financial transaction movement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CardexMovementType {
    Debit,  // Χρέωση
    Credit, // Πίστωση
}

impl CardexMovementType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Debit => "DEBIT",
            Self::Credit => "CREDIT",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_uppercase().as_str() {
            "DEBIT" => Some(Self::Debit),
            "CREDIT" => Some(Self::Credit),
            _ => None,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Debit => "Χρέωση",
            Self::Credit => "Πίστωση",
        }
    }
}

/// Registered commercial account profile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CardexEntity {
    pub entity_id: String, // AFM or sovereign unique code
    pub entity_type: EntityType,
    pub name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub credit_limit_eur: f64,
    pub payment_terms_days: u32,
    pub current_balance_eur: f64,
    pub created_at: i64,
}

/// Immutable ledger transaction in an entity's financial cardex.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CardexEntry {
    pub entry_id: String,
    pub entity_id: String,
    pub movement_type: CardexMovementType,
    pub document_type: String,
    pub document_series: String,
    pub document_number: i64,
    pub description: String,
    pub amount_eur: f64,
    pub running_balance_eur: f64,
    pub issue_date: String,
    pub due_date: String,
    pub created_at: i64,
}

/// Balance aging breakdown across statutory time brackets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BalanceAgingReport {
    pub entity_id: String,
    pub total_balance_eur: f64,
    pub current_0_30_eur: f64,
    pub overdue_31_60_eur: f64,
    pub overdue_61_90_eur: f64,
    pub overdue_91_120_eur: f64,
    pub overdue_120_plus_eur: f64,
}
