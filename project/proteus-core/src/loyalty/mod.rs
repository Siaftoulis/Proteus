// Proteus Sovereign Business OS — Bespoke Customer Loyalty & Rewards Engine
// 100% Original Codebase — Strictly adheres to Zero Mock Data & Pure Decimal Cent Accounting

pub mod gift_cards;
pub mod points;
pub mod types;

pub use gift_cards::{
    generate_secure_gift_code, get_card_ledger, hash_gift_code, init_gift_card_schema,
    issue_gift_card, lookup_gift_card, normalize_code, redeem_gift_card, top_up_gift_card,
};
pub use points::{
    accrue_points, create_or_get_account, find_account_by_phone_or_card, get_account,
    get_account_ledger, init_loyalty_schema, redeem_points,
};
pub use types::{
    GiftCard, GiftCardError, GiftCardStatus, GiftCardTransaction, GiftCardTxType, LoyaltyAccount,
    LoyaltyEntryType, LoyaltyError, LoyaltyPointEntry, LoyaltyPolicy, LoyaltyTier,
};
