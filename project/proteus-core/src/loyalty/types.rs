// Proteus Sovereign Business OS — Bespoke Customer Loyalty & Rewards Engine
// 100% Original Codebase — Strictly adheres to Zero Mock Data & Decimal Cent Accounting

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoyaltyTier {
    Bronze,
    Silver,
    Gold,
    Platinum,
}

impl LoyaltyTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            LoyaltyTier::Bronze => "Bronze",
            LoyaltyTier::Silver => "Silver",
            LoyaltyTier::Gold => "Gold",
            LoyaltyTier::Platinum => "Platinum",
        }
    }

    pub fn parse_str(s: &str) -> Self {
        match s {
            "Silver" => LoyaltyTier::Silver,
            "Gold" => LoyaltyTier::Gold,
            "Platinum" => LoyaltyTier::Platinum,
            _ => LoyaltyTier::Bronze,
        }
    }

    pub fn multiplier_basis_points(&self) -> u32 {
        match self {
            LoyaltyTier::Bronze => 10000,    // 1.00x
            LoyaltyTier::Silver => 12500,    // 1.25x
            LoyaltyTier::Gold => 15000,      // 1.50x
            LoyaltyTier::Platinum => 20000,  // 2.00x
        }
    }

    pub fn from_lifetime_points(points: i64, policy: &LoyaltyPolicy) -> Self {
        if points >= policy.tier_threshold_platinum {
            LoyaltyTier::Platinum
        } else if points >= policy.tier_threshold_gold {
            LoyaltyTier::Gold
        } else if points >= policy.tier_threshold_silver {
            LoyaltyTier::Silver
        } else {
            LoyaltyTier::Bronze
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoyaltyPolicy {
    pub cents_per_point: u32,
    pub redemption_cents_per_point: u32,
    pub tier_threshold_silver: i64,
    pub tier_threshold_gold: i64,
    pub tier_threshold_platinum: i64,
}

impl Default for LoyaltyPolicy {
    fn default() -> Self {
        Self {
            cents_per_point: 100,            // 1 point per €1.00 spent
            redemption_cents_per_point: 5,   // 100 points = €5.00 discount (5c/pt)
            tier_threshold_silver: 500,
            tier_threshold_gold: 2000,
            tier_threshold_platinum: 5000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoyaltyAccount {
    pub id: String,
    pub customer_phone: String,
    pub card_number: String,
    pub customer_name: String,
    pub tier: LoyaltyTier,
    pub points_balance: i64,
    pub lifetime_points_earned: i64,
    pub lifetime_spend_cents: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoyaltyEntryType {
    Accrual,
    Redemption,
    ManualAdjustment,
    Expiration,
}

impl LoyaltyEntryType {
    pub fn as_str(&self) -> &'static str {
        match self {
            LoyaltyEntryType::Accrual => "Accrual",
            LoyaltyEntryType::Redemption => "Redemption",
            LoyaltyEntryType::ManualAdjustment => "ManualAdjustment",
            LoyaltyEntryType::Expiration => "Expiration",
        }
    }

    pub fn parse_str(s: &str) -> Self {
        match s {
            "Redemption" => LoyaltyEntryType::Redemption,
            "ManualAdjustment" => LoyaltyEntryType::ManualAdjustment,
            "Expiration" => LoyaltyEntryType::Expiration,
            _ => LoyaltyEntryType::Accrual,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoyaltyPointEntry {
    pub id: String,
    pub account_id: String,
    pub entry_type: LoyaltyEntryType,
    pub points: i64,
    pub reference_id: Option<String>,
    pub qualifying_amount_cents: i64,
    pub multiplier_basis_points: u32,
    pub balance_after: i64,
    pub note: String,
    pub created_at: String,
}

#[derive(Debug, thiserror::Error)]
pub enum LoyaltyError {
    #[error("Loyalty account not found: {0}")]
    AccountNotFound(String),
    #[error("Insufficient loyalty points: requested {requested}, available {available}")]
    InsufficientPoints { requested: i64, available: i64 },
    #[error("Invalid points parameter: {0}")]
    InvalidParameter(String),
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
}
