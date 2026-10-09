// Proteus Sovereign Business OS — Declarative Promotional Rules & Cart Engine
// 100% Original Codebase — Strictly adheres to Zero Mock Data & Decimal Cent Accounting

pub mod engine;
pub mod types;

pub use engine::{
    evaluate_cart, get_promotional_rule, init_promotions_schema, list_active_rules,
    save_promotional_rule,
};
pub use types::{
    AppliedPromo, CartEvaluationResult, CartItem, DiscountTarget, DiscountType, PromoCondition,
    PromotionalRule,
};
