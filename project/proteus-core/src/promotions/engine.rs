// Proteus Sovereign Business OS — Declarative Promotional Rules & Cart Engine
// 100% Original Codebase — Strictly adheres to Zero Mock Data & Decimal Cent Accounting

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};

use super::types::{
    AppliedPromo, CartEvaluationResult, CartItem, DiscountTarget, DiscountType, PromoCondition,
    PromotionalRule,
};

pub fn init_promotions_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS promotional_rules (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT NOT NULL,
            priority INTEGER NOT NULL,
            rule_json TEXT NOT NULL,
            is_active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_promotions_priority ON promotional_rules(priority DESC, is_active);
        "#,
    )?;
    Ok(())
}

pub fn save_promotional_rule(conn: &Connection, rule: &PromotionalRule) -> rusqlite::Result<()> {
    let now = Utc::now().to_rfc3339();
    let json_str = serde_json::to_string(rule).unwrap_or_default();
    conn.execute(
        r#"
        INSERT INTO promotional_rules (id, name, description, priority, rule_json, is_active, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            description = excluded.description,
            priority = excluded.priority,
            rule_json = excluded.rule_json,
            is_active = excluded.is_active,
            updated_at = excluded.updated_at
        "#,
        params![
            rule.id,
            rule.name,
            rule.description,
            rule.priority,
            json_str,
            if rule.is_active { 1 } else { 0 },
            now,
            now,
        ],
    )?;
    Ok(())
}

pub fn get_promotional_rule(conn: &Connection, id: &str) -> rusqlite::Result<Option<PromotionalRule>> {
    conn.query_row(
        "SELECT rule_json FROM promotional_rules WHERE id = ?1",
        params![id],
        |row| {
            let json_str: String = row.get(0)?;
            let rule: PromotionalRule = serde_json::from_str(&json_str)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?;
            Ok(rule)
        },
    )
    .optional()
}

pub fn list_active_rules(conn: &Connection) -> rusqlite::Result<Vec<PromotionalRule>> {
    let mut stmt = conn.prepare(
        "SELECT rule_json FROM promotional_rules WHERE is_active = 1 ORDER BY priority DESC",
    )?;
    let rules = stmt
        .query_map([], |row| {
            let json_str: String = row.get(0)?;
            let rule: PromotionalRule = serde_json::from_str(&json_str)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))?;
            Ok(rule)
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rules)
}

pub fn evaluate_cart(
    items: &[CartItem],
    rules: &[PromotionalRule],
    current_time_iso: &str,
) -> CartEvaluationResult {
    let original_gross: i64 = items.iter().map(|it| it.total_cents()).sum();
    let mut current_gross = original_gross;
    let mut applied_promotions = Vec::new();
    let mut applied_non_stackable = false;

    let mut sorted_rules = rules.to_vec();
    sorted_rules.sort_by_key(|b| std::cmp::Reverse(b.priority));

    for rule in sorted_rules {
        if !rule.is_active {
            continue;
        }
        if applied_non_stackable || (!rule.is_stackable && !applied_promotions.is_empty()) {
            continue;
        }

        let conditions_met = rule.conditions.iter().all(|c| match c {
            PromoCondition::MinOrderAmountCents(min) => original_gross >= *min,
            PromoCondition::MinItemQuantity { target, min_qty } => {
                let count: u32 = items
                    .iter()
                    .filter(|it| match target {
                        DiscountTarget::OrderTotal => true,
                        DiscountTarget::SpecificItem(sku) => &it.sku == sku,
                        DiscountTarget::Category(cat) => &it.category == cat,
                    })
                    .map(|it| it.quantity)
                    .sum();
                count >= *min_qty
            }
            PromoCondition::BundleRequiredItems(required) => {
                required.iter().all(|req_sku| items.iter().any(|it| &it.sku == req_sku && it.quantity > 0))
            }
            PromoCondition::DateRange { valid_from, valid_until } => {
                current_time_iso >= valid_from.as_str() && current_time_iso <= valid_until.as_str()
            }
        });

        if !conditions_met {
            continue;
        }

        let discount_cents = match &rule.target {
            DiscountTarget::OrderTotal => match rule.discount {
                DiscountType::PercentageBasisPoints(bp) => (current_gross * bp as i64) / 10000,
                DiscountType::FixedAmountCents(amt) => amt.min(current_gross),
                DiscountType::BuyXGetYFree { .. } => 0,
            },
            DiscountTarget::SpecificItem(sku) => {
                if let Some(it) = items.iter().find(|i| &i.sku == sku) {
                    match rule.discount {
                        DiscountType::PercentageBasisPoints(bp) => (it.total_cents() * bp as i64) / 10000,
                        DiscountType::FixedAmountCents(amt) => amt.min(it.total_cents()),
                        DiscountType::BuyXGetYFree { buy_qty, free_qty } => {
                            let cycle = buy_qty + free_qty;
                            it.quantity
                                .checked_div(cycle)
                                .map(|free_batches| (free_batches * free_qty) as i64 * it.unit_price_cents)
                                .unwrap_or(0)
                        }
                    }
                } else {
                    0
                }
            }
            DiscountTarget::Category(cat) => {
                let cat_total: i64 = items
                    .iter()
                    .filter(|it| &it.category == cat)
                    .map(|it| it.total_cents())
                    .sum();
                match rule.discount {
                    DiscountType::PercentageBasisPoints(bp) => (cat_total * bp as i64) / 10000,
                    DiscountType::FixedAmountCents(amt) => amt.min(cat_total),
                    DiscountType::BuyXGetYFree { .. } => 0,
                }
            }
        };

        if discount_cents > 0 {
            let actual_discount = discount_cents.min(current_gross);
            current_gross -= actual_discount;
            if !rule.is_stackable {
                applied_non_stackable = true;
            }
            applied_promotions.push(AppliedPromo {
                rule_id: rule.id.clone(),
                rule_name: rule.name.clone(),
                discount_cents: actual_discount,
                note: rule.description.clone(),
            });
        }
    }

    let total_discount_cents = original_gross - current_gross;
    CartEvaluationResult {
        original_gross_cents: original_gross,
        total_discount_cents,
        final_gross_cents: current_gross,
        applied_promotions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_promotions_schema_and_crud() {
        let conn = Connection::open_in_memory().unwrap();
        init_promotions_schema(&conn).unwrap();

        let rule = PromotionalRule {
            id: "PROMO-SUMMER".to_string(),
            name: "Summer Sale 10%".to_string(),
            description: "10% off entire order".to_string(),
            priority: 100,
            conditions: vec![PromoCondition::MinOrderAmountCents(5000)],
            target: DiscountTarget::OrderTotal,
            discount: DiscountType::PercentageBasisPoints(1000),
            is_stackable: true,
            is_active: true,
        };

        save_promotional_rule(&conn, &rule).unwrap();
        let loaded = get_promotional_rule(&conn, "PROMO-SUMMER").unwrap().unwrap();
        assert_eq!(loaded.name, "Summer Sale 10%");

        let active = list_active_rules(&conn).unwrap();
        assert_eq!(active.len(), 1);
    }

    #[test]
    fn test_cart_evaluation_discounts_and_bogo() {
        let items = vec![
            CartItem::new("SKU-SCR", "Screen Protector", "Accessories", 3, 1000), // 3 @ €10.00 = €30.00
            CartItem::new("SKU-REP", "Screen Repair Service", "Service", 1, 8000), // 1 @ €80.00 = €80.00
        ]; // Total = 11,000 cents (€110.00)

        // Rule 1: Buy 2 Get 1 Free on Screen Protectors (SKU-SCR)
        let bogo_rule = PromotionalRule {
            id: "BOGO-SCR".to_string(),
            name: "BOGO Screen Protector".to_string(),
            description: "Buy 2 get 1 free".to_string(),
            priority: 200,
            conditions: vec![],
            target: DiscountTarget::SpecificItem("SKU-SCR".to_string()),
            discount: DiscountType::BuyXGetYFree { buy_qty: 2, free_qty: 1 },
            is_stackable: true,
            is_active: true,
        };

        // Rule 2: 10% off order if order >= €100.00
        let order_rule = PromotionalRule {
            id: "ORDER-10".to_string(),
            name: "10% VIP Order".to_string(),
            description: "10% off big orders".to_string(),
            priority: 100,
            conditions: vec![PromoCondition::MinOrderAmountCents(10000)],
            target: DiscountTarget::OrderTotal,
            discount: DiscountType::PercentageBasisPoints(1000),
            is_stackable: true,
            is_active: true,
        };

        let result = evaluate_cart(&items, &[bogo_rule, order_rule], "2026-10-09T00:00:00Z");
        assert_eq!(result.original_gross_cents, 11000);
        // BOGO saves 1000 cents (€10.00). Remaining gross = 10000.
        // Then 10% on remaining 10000 saves 1000 cents (€10.00).
        // Total discount = 2000 cents (€20.00). Final gross = 9000 cents (€90.00).
        assert_eq!(result.total_discount_cents, 2000);
        assert_eq!(result.final_gross_cents, 9000);
        assert_eq!(result.applied_promotions.len(), 2);
    }
}
