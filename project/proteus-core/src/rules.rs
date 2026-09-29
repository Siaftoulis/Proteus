//! Declarative Business Rules Engine ("IF X THEN Y") for Proteus BOS.
//! Enables Business & Data Analysts (PCDA) to define visual-first validations,
//! transformations, automated discounts, and approval flags without modifying code.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RuleCondition {
    FieldEquals { field: String, value: Value },
    FieldGreaterThan { field: String, value: f64 },
    FieldLessThan { field: String, value: f64 },
    FieldContains { field: String, substring: String },
    FieldIsEmpty { field: String },
    And(Vec<RuleCondition>),
    Or(Vec<RuleCondition>),
}

impl RuleCondition {
    pub fn evaluate(&self, data: &Value) -> bool {
        match self {
            Self::FieldEquals { field, value } => {
                data.get(field) == Some(value)
            }
            Self::FieldGreaterThan { field, value } => {
                data.get(field)
                    .and_then(|v| v.as_f64())
                    .map(|v| v > *value)
                    .unwrap_or(false)
            }
            Self::FieldLessThan { field, value } => {
                data.get(field)
                    .and_then(|v| v.as_f64())
                    .map(|v| v < *value)
                    .unwrap_or(false)
            }
            Self::FieldContains { field, substring } => {
                data.get(field)
                    .and_then(|v| v.as_str())
                    .map(|s| s.contains(substring))
                    .unwrap_or(false)
            }
            Self::FieldIsEmpty { field } => match data.get(field) {
                None | Some(Value::Null) => true,
                Some(Value::String(s)) => s.trim().is_empty(),
                Some(Value::Array(arr)) => arr.is_empty(),
                Some(Value::Object(map)) => map.is_empty(),
                _ => false,
            },
            Self::And(conditions) => {
                !conditions.is_empty() && conditions.iter().all(|c| c.evaluate(data))
            }
            Self::Or(conditions) => {
                conditions.iter().any(|c| c.evaluate(data))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RuleAction {
    SetField { field: String, value: Value },
    ApplyDiscountPercentage { field: String, percentage: f64 },
    FlagForReview { reason: String },
    RejectTransaction { reason: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BusinessRule {
    pub id: String,
    pub name: String,
    pub entity_type: String,
    pub condition: RuleCondition,
    pub actions: Vec<RuleAction>,
    pub is_active: bool,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuleExecutionOutcome {
    pub applied_rules: Vec<String>,
    pub flags: Vec<String>,
    pub rejections: Vec<String>,
}

pub struct BusinessRulesEngine {
    rules: Vec<BusinessRule>,
}

impl BusinessRulesEngine {
    pub fn new(rules: Vec<BusinessRule>) -> Self {
        Self { rules }
    }

    /// Evaluates rules for a given entity type and applies actions directly to the data payload.
    pub fn execute(&self, entity_type: &str, data: &mut Value) -> RuleExecutionOutcome {
        let mut outcome = RuleExecutionOutcome::default();

        for rule in &self.rules {
            if !rule.is_active || rule.entity_type != entity_type {
                continue;
            }

            if rule.condition.evaluate(data) {
                outcome.applied_rules.push(rule.name.clone());

                for action in &rule.actions {
                    match action {
                        RuleAction::SetField { field, value } => {
                            if let Value::Object(map) = data {
                                map.insert(field.clone(), value.clone());
                            }
                        }
                        RuleAction::ApplyDiscountPercentage { field, percentage } => {
                            if let Value::Object(map) = data {
                                if let Some(current_val) = map.get(field).and_then(|v| v.as_f64()) {
                                    let factor = (100.0 - percentage) / 100.0;
                                    let new_val = (current_val * factor * 100.0).round() / 100.0;
                                    map.insert(field.clone(), Value::from(new_val));
                                }
                            }
                        }
                        RuleAction::FlagForReview { reason } => {
                            outcome.flags.push(reason.clone());
                        }
                        RuleAction::RejectTransaction { reason } => {
                            outcome.rejections.push(reason.clone());
                        }
                    }
                }
            }
        }

        outcome
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_discount_rule_execution() {
        let rule = BusinessRule {
            id: "R-001".to_string(),
            name: "VIP Discount 10%".to_string(),
            entity_type: "order".to_string(),
            condition: RuleCondition::FieldGreaterThan {
                field: "total_amount".to_string(),
                value: 100.0,
            },
            actions: vec![
                RuleAction::ApplyDiscountPercentage {
                    field: "total_amount".to_string(),
                    percentage: 10.0,
                },
                RuleAction::SetField {
                    field: "discount_applied".to_string(),
                    value: Value::Bool(true),
                },
            ],
            is_active: true,
        };

        let engine = BusinessRulesEngine::new(vec![rule]);
        let mut order = json!({
            "order_id": "ORD-99",
            "total_amount": 200.0
        });

        let outcome = engine.execute("order", &mut order);
        assert_eq!(outcome.applied_rules.len(), 1);
        assert_eq!(order["total_amount"].as_f64(), Some(180.0));
        assert_eq!(order["discount_applied"].as_bool(), Some(true));
    }

    #[test]
    fn test_validation_rejection_rule() {
        let rule = BusinessRule {
            id: "R-002".to_string(),
            name: "Require Phone Number".to_string(),
            entity_type: "ticket".to_string(),
            condition: RuleCondition::FieldIsEmpty {
                field: "customer_phone".to_string(),
            },
            actions: vec![RuleAction::RejectTransaction {
                reason: "Απαιτείται έγκυρο τηλέφωνο επικοινωνίας".to_string(),
            }],
            is_active: true,
        };

        let engine = BusinessRulesEngine::new(vec![rule]);
        let mut ticket = json!({
            "customer_name": "Nikos",
            "customer_phone": ""
        });

        let outcome = engine.execute("ticket", &mut ticket);
        assert_eq!(outcome.rejections.len(), 1);
        assert!(outcome.rejections[0].contains("Απαιτείται έγκυρο τηλέφωνο"));
    }

    #[test]
    fn test_compound_and_condition() {
        let cond = RuleCondition::And(vec![
            RuleCondition::FieldEquals {
                field: "category".to_string(),
                value: json!("electronics"),
            },
            RuleCondition::FieldGreaterThan {
                field: "price".to_string(),
                value: 50.0,
            },
        ]);

        let payload_matching = json!({ "category": "electronics", "price": 89.99 });
        let payload_non_matching = json!({ "category": "books", "price": 89.99 });

        assert!(cond.evaluate(&payload_matching));
        assert!(!cond.evaluate(&payload_non_matching));
    }
}
