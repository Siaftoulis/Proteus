//! Universal Database Scripting DSL & Visual Scratch Data Blocks.
//! Provides a declarative database specification language and recursive descent AST parser.

pub mod ast;
pub mod lexer;
pub mod parser;
pub mod transpiler;

pub use ast::{
    DslEntity, DslField, DslFieldType, DslIndex, DslRelation, DslSchema, RelationKind,
};
pub use lexer::{DslParserError, Lexer, Token};
pub use parser::{parse_dsl, DslParser};
pub use transpiler::{DslTranspiler, TargetDialect};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_entity() {
        let script = r#"
            // Customer Table Definition
            entity Customer {
                id: uuid primary,
                name: text required,
                email: text unique,
                balance: decimal default(0.0),
                is_active: bool default(true),
                created_at: timestamp
            }
        "#;

        let schema = parse_dsl(script).unwrap();
        assert_eq!(schema.entities.len(), 1);

        let customer = &schema.entities[0];
        assert_eq!(customer.name, "Customer");
        assert_eq!(customer.fields.len(), 6);

        let id = customer.find_field("id").unwrap();
        assert_eq!(id.field_type, DslFieldType::Uuid);
        assert!(id.is_primary);
        assert!(id.is_required);

        let email = customer.find_field("email").unwrap();
        assert_eq!(email.field_type, DslFieldType::Text);
        assert!(email.is_unique);

        let balance = customer.find_field("balance").unwrap();
        assert_eq!(balance.field_type, DslFieldType::Decimal);
        assert_eq!(balance.default_value.as_deref(), Some("0.0"));
    }

    #[test]
    fn test_parse_entities_with_indexes_and_relations() {
        let script = r#"
            entity Customer {
                id: uuid primary,
                company_afm: text unique,
                index idx_afm (company_afm) unique
            }

            entity Order {
                id: uuid primary,
                customer_id: uuid required,
                total_cents: integer default(0),
                index idx_customer_order (customer_id)
            }

            relation Customer.id -> Order.customer_id [one_to_many]
        "#;

        let schema = parse_dsl(script).unwrap();
        assert_eq!(schema.entities.len(), 2);
        assert_eq!(schema.relations.len(), 1);

        let customer = schema.find_entity("Customer").unwrap();
        assert_eq!(customer.indexes.len(), 1);
        assert_eq!(customer.indexes[0].name, "idx_afm");
        assert!(customer.indexes[0].is_unique);

        let order = schema.find_entity("Order").unwrap();
        assert_eq!(order.indexes.len(), 1);
        assert_eq!(order.indexes[0].fields, vec!["customer_id"]);

        let rel = &schema.relations[0];
        assert_eq!(rel.from_entity, "Customer");
        assert_eq!(rel.from_field, "id");
        assert_eq!(rel.to_entity, "Order");
        assert_eq!(rel.to_field, "customer_id");
        assert_eq!(rel.kind, RelationKind::OneToMany);
    }

    #[test]
    fn test_unknown_type_error() {
        let script = r#"
            entity BadEntity {
                id: unknown_alien_type
            }
        "#;

        let err = parse_dsl(script).unwrap_err();
        match err {
            DslParserError::UnknownType { type_name, .. } => {
                assert_eq!(type_name, "unknown_alien_type");
            }
            _ => panic!("Expected UnknownType error"),
        }
    }

    #[test]
    fn test_invalid_relation_entity_validation() {
        let script = r#"
            entity Account {
                id: uuid primary
            }

            relation Account.id -> NonExistentEntity.account_id [one_to_many]
        "#;

        let err = parse_dsl(script).unwrap_err();
        match err {
            DslParserError::ValidationError { message } => {
                assert!(message.contains("NonExistentEntity"));
            }
            _ => panic!("Expected ValidationError"),
        }
    }
}
