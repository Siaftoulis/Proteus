// Proteus Sovereign Business OS — Declarative Promotional Rules & Cart Engine
// 100% Original Codebase — Strictly adheres to Zero Mock Data & Decimal Cent Accounting

pub mod coupons;
pub mod engine;
pub mod types;

pub use coupons::{
    create_coupon, generate_single_use_coupon, get_coupon_redemptions, hash_coupon_code,
    init_coupon_schema, lookup_coupon, normalize_coupon_code, redeem_coupon, validate_coupon,
};
pub use engine::{
    evaluate_cart, get_promotional_rule, init_promotions_schema, list_active_rules,
    save_promotional_rule,
};
pub use types::{
    AppliedPromo, CartEvaluationResult, CartItem, Coupon, CouponError, CouponRedemption,
    DiscountTarget, DiscountType, PromoCondition, PromotionalRule,
};
