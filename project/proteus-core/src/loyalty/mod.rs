// Proteus Sovereign Business OS — Bespoke Customer Loyalty & Rewards Engine
// 100% Original Codebase — Strictly adheres to Zero Mock Data & Pure Decimal Cent Accounting

pub mod points;
pub mod types;

pub use points::{
    accrue_points, create_or_get_account, find_account_by_phone_or_card, get_account,
    get_account_ledger, init_loyalty_schema, redeem_points,
};
pub use types::{
    LoyaltyAccount, LoyaltyEntryType, LoyaltyError, LoyaltyPointEntry, LoyaltyPolicy, LoyaltyTier,
};
