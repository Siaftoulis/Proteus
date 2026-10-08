//! Multi-Dialect SQL & NoSQL Transpiler for Proteus Database DSL.
//! Transpiles declarative DslSchema AST into idiomatic SQLite, PostgreSQL, MySQL, and MongoDB.

use crate::dsl::ast::{DslFieldType, DslSchema};
use serde::{Deserialize, Serialize};

/// Target database dialects supported by the transpiler engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetDialect {
    Sqlite,
    Postgres,
    Mysql,
    MongoDb,
}

pub struct DslTranspiler;

impl DslTranspiler {
    /// Transpiles a DslSchema into the chosen target dialect.
    pub fn transpile(schema: &DslSchema, dialect: TargetDialect) -> String {
        match dialect {
            TargetDialect::Sqlite => Self::transpile_sqlite(schema),
            TargetDialect::Postgres => Self::transpile_postgres(schema),
            TargetDialect::Mysql => Self::transpile_mysql(schema),
            TargetDialect::MongoDb => Self::transpile_mongodb(schema),
        }
    }

    /// Transpiles to SQLite DDL with strict types and indexes.
    pub fn transpile_sqlite(schema: &DslSchema) -> String {
        let mut ddl = String::new();
        ddl.push_str("-- Proteus Database DSL -> SQLite Generated DDL\n\n");

        for entity in &schema.entities {
            ddl.push_str(&format!("CREATE TABLE IF NOT EXISTS {} (\n", entity.name));
            let mut col_defs = Vec::new();

            for field in &entity.fields {
                let type_name = match field.field_type {
                    DslFieldType::Uuid | DslFieldType::Text | DslFieldType::Timestamp | DslFieldType::Json => "TEXT",
                    DslFieldType::Integer => "INTEGER",
                    DslFieldType::Decimal => "REAL",
                    DslFieldType::Boolean => "INTEGER",
                };

                let mut col_str = format!("    {} {}", field.name, type_name);
                if field.is_primary {
                    col_str.push_str(" PRIMARY KEY");
                }
                if field.is_required && !field.is_primary {
                    col_str.push_str(" NOT NULL");
                }
                if field.is_unique && !field.is_primary {
                    col_str.push_str(" UNIQUE");
                }
                if let Some(ref def) = field.default_value {
                    col_str.push_str(&format!(" DEFAULT {}", def));
                }
                col_defs.push(col_str);
            }

            // Foreign keys from relations
            for rel in &schema.relations {
                if rel.to_entity.eq_ignore_ascii_case(&entity.name) {
                    col_defs.push(format!(
                        "    FOREIGN KEY ({}) REFERENCES {}({}) ON DELETE CASCADE",
                        rel.to_field, rel.from_entity, rel.from_field
                    ));
                }
            }

            ddl.push_str(&col_defs.join(",\n"));
            ddl.push_str("\n);\n\n");

            // Indexes
            for idx in &entity.indexes {
                let unique_clause = if idx.is_unique { "UNIQUE " } else { "" };
                ddl.push_str(&format!(
                    "CREATE {}INDEX IF NOT EXISTS {} ON {}({});\n",
                    unique_clause,
                    idx.name,
                    entity.name,
                    idx.fields.join(", ")
                ));
            }
            if !entity.indexes.is_empty() {
                ddl.push('\n');
            }
        }

        ddl
    }

    /// Transpiles to PostgreSQL DDL with UUID, JSONB, and TIMESTAMPTZ.
    pub fn transpile_postgres(schema: &DslSchema) -> String {
        let mut ddl = String::new();
        ddl.push_str("-- Proteus Database DSL -> PostgreSQL Generated DDL\n\n");

        for entity in &schema.entities {
            ddl.push_str(&format!("CREATE TABLE IF NOT EXISTS \"{}\" (\n", entity.name));
            let mut col_defs = Vec::new();

            for field in &entity.fields {
                let type_name = match field.field_type {
                    DslFieldType::Uuid => "UUID",
                    DslFieldType::Text => "TEXT",
                    DslFieldType::Integer => "BIGINT",
                    DslFieldType::Decimal => "NUMERIC(14, 4)",
                    DslFieldType::Boolean => "BOOLEAN",
                    DslFieldType::Timestamp => "TIMESTAMPTZ",
                    DslFieldType::Json => "JSONB",
                };

                let mut col_str = format!("    \"{}\" {}", field.name, type_name);
                if field.is_primary {
                    col_str.push_str(" PRIMARY KEY");
                }
                if field.is_required && !field.is_primary {
                    col_str.push_str(" NOT NULL");
                }
                if field.is_unique && !field.is_primary {
                    col_str.push_str(" UNIQUE");
                }
                if let Some(ref def) = field.default_value {
                    col_str.push_str(&format!(" DEFAULT {}", def));
                }
                col_defs.push(col_str);
            }

            // Foreign keys
            for rel in &schema.relations {
                if rel.to_entity.eq_ignore_ascii_case(&entity.name) {
                    col_defs.push(format!(
                        "    CONSTRAINT \"fk_{}_{}\" FOREIGN KEY (\"{}\") REFERENCES \"{}\"(\"{}\") ON DELETE CASCADE",
                        entity.name, rel.to_field, rel.to_field, rel.from_entity, rel.from_field
                    ));
                }
            }

            ddl.push_str(&col_defs.join(",\n"));
            ddl.push_str("\n);\n\n");

            // Indexes
            for idx in &entity.indexes {
                let unique_clause = if idx.is_unique { "UNIQUE " } else { "" };
                let fields_quoted = idx.fields.iter().map(|f| format!("\"{}\"", f)).collect::<Vec<_>>().join(", ");
                ddl.push_str(&format!(
                    "CREATE {}INDEX IF NOT EXISTS \"{}\" ON \"{}\" ({});\n",
                    unique_clause, idx.name, entity.name, fields_quoted
                ));
            }
            if !entity.indexes.is_empty() {
                ddl.push('\n');
            }
        }

        ddl
    }

    /// Transpiles to MySQL / MariaDB DDL with engine and charset options.
    pub fn transpile_mysql(schema: &DslSchema) -> String {
        let mut ddl = String::new();
        ddl.push_str("-- Proteus Database DSL -> MySQL / MariaDB Generated DDL\n\n");

        for entity in &schema.entities {
            ddl.push_str(&format!("CREATE TABLE IF NOT EXISTS `{}` (\n", entity.name));
            let mut col_defs = Vec::new();

            for field in &entity.fields {
                let type_name = match field.field_type {
                    DslFieldType::Uuid => "VARCHAR(36)",
                    DslFieldType::Text => "TEXT",
                    DslFieldType::Integer => "BIGINT",
                    DslFieldType::Decimal => "DECIMAL(14, 4)",
                    DslFieldType::Boolean => "TINYINT(1)",
                    DslFieldType::Timestamp => "DATETIME",
                    DslFieldType::Json => "JSON",
                };

                let mut col_str = format!("    `{}` {}", field.name, type_name);
                if field.is_primary {
                    col_str.push_str(" PRIMARY KEY");
                }
                if field.is_required && !field.is_primary {
                    col_str.push_str(" NOT NULL");
                }
                if field.is_unique && !field.is_primary {
                    col_str.push_str(" UNIQUE");
                }
                if let Some(ref def) = field.default_value {
                    col_str.push_str(&format!(" DEFAULT {}", def));
                }
                col_defs.push(col_str);
            }

            for rel in &schema.relations {
                if rel.to_entity.eq_ignore_ascii_case(&entity.name) {
                    col_defs.push(format!(
                        "    CONSTRAINT `fk_{}_{}` FOREIGN KEY (`{}`) REFERENCES `{}`(`{}`) ON DELETE CASCADE",
                        entity.name, rel.to_field, rel.to_field, rel.from_entity, rel.from_field
                    ));
                }
            }

            ddl.push_str(&col_defs.join(",\n"));
            ddl.push_str("\n) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;\n\n");

            for idx in &entity.indexes {
                let unique_clause = if idx.is_unique { "UNIQUE " } else { "" };
                let fields_quoted = idx.fields.iter().map(|f| format!("`{}`", f)).collect::<Vec<_>>().join(", ");
                ddl.push_str(&format!(
                    "CREATE {}INDEX `{}` ON `{}` ({});\n",
                    unique_clause, idx.name, entity.name, fields_quoted
                ));
            }
            if !entity.indexes.is_empty() {
                ddl.push('\n');
            }
        }

        ddl
    }

    /// Transpiles to MongoDB JSON Schema validator and collection creation script.
    pub fn transpile_mongodb(schema: &DslSchema) -> String {
        let mut script = String::new();
        script.push_str("// Proteus Database DSL -> MongoDB Collection Initialization & Schema Validator\n\n");

        for entity in &schema.entities {
            let mut required_fields = Vec::new();
            let mut properties = serde_json::Map::new();

            for field in &entity.fields {
                if field.is_required || field.is_primary {
                    required_fields.push(serde_json::Value::String(field.name.clone()));
                }

                let bson_type = match field.field_type {
                    DslFieldType::Uuid | DslFieldType::Text => "string",
                    DslFieldType::Integer => "long",
                    DslFieldType::Decimal => "double",
                    DslFieldType::Boolean => "bool",
                    DslFieldType::Timestamp => "date",
                    DslFieldType::Json => "object",
                };

                let mut prop = serde_json::Map::new();
                prop.insert("bsonType".to_string(), serde_json::Value::String(bson_type.to_string()));
                prop.insert(
                    "description".to_string(),
                    serde_json::Value::String(format!("Field {} ({:?})", field.name, field.field_type)),
                );

                properties.insert(field.name.clone(), serde_json::Value::Object(prop));
            }

            let mut validator = serde_json::Map::new();
            let mut json_schema = serde_json::Map::new();
            json_schema.insert("bsonType".to_string(), serde_json::Value::String("object".to_string()));
            json_schema.insert("required".to_string(), serde_json::Value::Array(required_fields));
            json_schema.insert("properties".to_string(), serde_json::Value::Object(properties));
            validator.insert("$jsonSchema".to_string(), serde_json::Value::Object(json_schema));

            let validator_json = serde_json::to_string_pretty(&validator).unwrap_or_default();

            script.push_str(&format!(
                "db.createCollection(\"{}\", {{\n    validator: {}\n}});\n\n",
                entity.name, validator_json
            ));

            // Indexes
            for idx in &entity.indexes {
                let mut index_spec = serde_json::Map::new();
                for f in &idx.fields {
                    index_spec.insert(f.clone(), serde_json::Value::Number(1.into()));
                }
                let spec_str = serde_json::to_string(&index_spec).unwrap_or_default();
                script.push_str(&format!(
                    "db.{}.createIndex({}, {{ name: \"{}\", unique: {} }});\n",
                    entity.name, spec_str, idx.name, idx.is_unique
                ));
            }
            if !entity.indexes.is_empty() {
                script.push('\n');
            }
        }

        script
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::parser::parse_dsl;

    #[test]
    fn test_transpile_sqlite() {
        let script = r#"
            entity Customer {
                id: uuid primary,
                name: text required,
                balance: decimal default(0.0),
                index idx_cust_name (name)
            }
            entity Order {
                id: uuid primary,
                customer_id: uuid required
            }
            relation Customer.id -> Order.customer_id [one_to_many]
        "#;
        let schema = parse_dsl(script).unwrap();
        let sqlite_ddl = DslTranspiler::transpile(&schema, TargetDialect::Sqlite);

        assert!(sqlite_ddl.contains("CREATE TABLE IF NOT EXISTS Customer"));
        assert!(sqlite_ddl.contains("id TEXT PRIMARY KEY"));
        assert!(sqlite_ddl.contains("name TEXT NOT NULL"));
        assert!(sqlite_ddl.contains("balance REAL DEFAULT 0.0"));
        assert!(sqlite_ddl.contains("CREATE INDEX IF NOT EXISTS idx_cust_name ON Customer(name);"));
        assert!(sqlite_ddl.contains("FOREIGN KEY (customer_id) REFERENCES Customer(id) ON DELETE CASCADE"));
    }

    #[test]
    fn test_transpile_postgres() {
        let script = r#"
            entity Inventory {
                id: uuid primary,
                sku: text unique,
                quantity: integer default(0),
                metadata: json
            }
        "#;
        let schema = parse_dsl(script).unwrap();
        let pg_ddl = DslTranspiler::transpile(&schema, TargetDialect::Postgres);

        assert!(pg_ddl.contains("CREATE TABLE IF NOT EXISTS \"Inventory\""));
        assert!(pg_ddl.contains("\"id\" UUID PRIMARY KEY"));
        assert!(pg_ddl.contains("\"sku\" TEXT UNIQUE"));
        assert!(pg_ddl.contains("\"quantity\" BIGINT DEFAULT 0"));
        assert!(pg_ddl.contains("\"metadata\" JSONB"));
    }

    #[test]
    fn test_transpile_mysql() {
        let script = r#"
            entity User {
                id: uuid primary,
                username: text required,
                is_admin: bool default(false)
            }
        "#;
        let schema = parse_dsl(script).unwrap();
        let mysql_ddl = DslTranspiler::transpile(&schema, TargetDialect::Mysql);

        assert!(mysql_ddl.contains("CREATE TABLE IF NOT EXISTS `User`"));
        assert!(mysql_ddl.contains("`id` VARCHAR(36) PRIMARY KEY"));
        assert!(mysql_ddl.contains("`is_admin` TINYINT(1) DEFAULT false"));
        assert!(mysql_ddl.contains("ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;"));
    }

    #[test]
    fn test_transpile_mongodb() {
        let script = r#"
            entity Device {
                id: uuid primary,
                serial_number: text unique,
                index idx_serial (serial_number) unique
            }
        "#;
        let schema = parse_dsl(script).unwrap();
        let mongo_script = DslTranspiler::transpile(&schema, TargetDialect::MongoDb);

        assert!(mongo_script.contains("db.createCollection(\"Device\""));
        assert!(mongo_script.contains("\"bsonType\": \"string\""));
        assert!(mongo_script.contains("\"required\": ["));
        assert!(mongo_script.contains("db.Device.createIndex"));
        assert!(mongo_script.contains("unique: true"));
    }
}
