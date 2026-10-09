// Proteus Sovereign Business OS — Declarative Promotional Rules & Cart Engine
// 100% Original Codebase — Strictly adheres to Zero Mock Data & Decimal Cent Accounting

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiscountTarget {
    OrderTotal,
    SpecificItem(String),
    Category(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiscountType {
    /// Percentage in basis points (e.g. 1500 = 15.00%, 10000 = 100.00%)
    PercentageBasisPoints(u32),
    /// Fixed amount in integer cents (e.g. 500 = €5.00)
    FixedAmountCents(i64),
    /// Buy X get Y free on targeted items
    BuyXGetYFree { buy_qty: u32, free_qty: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PromoCondition {
    MinOrderAmountCents(i64),
    MinItemQuantity { target: DiscountTarget, min_qty: u32 },
    BundleRequiredItems(Vec<String>),
    DateRange { valid_from: String, valid_until: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromotionalRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub priority: u32,
    pub conditions: Vec<PromoCondition>,
    pub target: DiscountTarget,
    pub discount: DiscountType,
    pub is_stackable: bool,
    pub is_active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CartItem {
    pub sku: String,
    pub name: String,
    pub category: String,
    pub quantity: u32,
    pub unit_price_cents: i64,
}

impl CartItem {
    pub fn new(sku: &str, name: &str, category: &str, quantity: u32, unit_price_cents: i64) -> Self {
        Self {
            sku: sku.to_string(),
            name: name.to_string(),
            category: category.to_string(),
            quantity,
            unit_price_cents,
        }
    }

    pub fn total_cents(&self) -> i64 {
        self.unit_price_cents * (self.quantity as i64)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppliedPromo {
    pub rule_id: String,
    pub rule_name: String,
    pub discount_cents: i64,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CartEvaluationResult {
    pub original_gross_cents: i64,
    pub total_discount_cents: i64,
    pub final_gross_cents: i64,
    pub applied_promotions: Vec<AppliedPromo>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Coupon {
    pub id: String,
    pub code_hash: String,
    pub code_display: String,
    pub discount: DiscountType,
    pub min_spend_cents: i64,
    pub usage_limit_total: Option<u32>,
    pub usage_limit_per_customer: u32,
    pub usage_count: u32,
    pub valid_from: String,
    pub valid_until: Option<String>,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CouponRedemption {
    pub id: String,
    pub coupon_id: String,
    pub customer_identifier: String,
    pub receipt_id: Option<String>,
    pub discount_applied_cents: i64,
    pub redeemed_at: String,
}

#[derive(Debug, thiserror::Error)]
pub enum CouponError {
    #[error("Coupon not found")]
    NotFound,
    #[error("Coupon is inactive")]
    Inactive,
    #[error("Coupon is expired or not yet active")]
    Expired,
    #[error("Minimum order spend of {min_spend_cents} cents not met (cart has {actual_cents} cents)")]
    MinSpendNotMet { min_spend_cents: i64, actual_cents: i64 },
    #[error("Coupon total usage limit reached ({0} redemptions)")]
    TotalUsageLimitExceeded(u32),
    #[error("Customer has already used this coupon {used} times (limit {limit})")]
    CustomerUsageLimitExceeded { used: u32, limit: u32 },
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
}
