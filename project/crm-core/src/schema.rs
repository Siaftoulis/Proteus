// Bespoke Schema Definition and Validation Engine for Proteus CRM Core
use crate::{Database, DbError};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FieldType {
    Text,
    Number,
    Boolean,
    Email,
    Phone,
    Date,
    Select(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldDefinition {
    pub name: String,
    pub label: String,
    pub field_type: FieldType,
    pub required: bool,
    pub default_value: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntitySchema {
    pub entity: String,
    pub label: String,
    pub fields: Vec<FieldDefinition>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    MissingRequiredField(String),
    TypeMismatch { field: String, expected: String },
    InvalidOption { field: String, value: String },
    InvalidJson(String),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::MissingRequiredField(name) => write!(f, "Missing required field: '{}'", name),
            ValidationError::TypeMismatch { field, expected } => write!(f, "Field '{}' expected type {}", field, expected),
            ValidationError::InvalidOption { field, value } => write!(f, "Field '{}' has invalid option '{}'", field, value),
            ValidationError::InvalidJson(err) => write!(f, "Invalid JSON data: {}", err),
        }
    }
}

impl std::error::Error for ValidationError {}

impl EntitySchema {
    pub fn new(entity: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            entity: entity.into(),
            label: label.into(),
            fields: Vec::new(),
        }
    }

    pub fn with_field(mut self, field: FieldDefinition) -> Self {
        self.fields.push(field);
        self
    }

    /// Validates a raw JSON fields payload against this entity schema.
    pub fn validate_json(&self, json_str: &str) -> Result<(), ValidationError> {
        let value: Value = serde_json::from_str(json_str)
            .map_err(|e| ValidationError::InvalidJson(e.to_string()))?;
        
        let map = match value.as_object() {
            Some(m) => m,
            None => return Err(ValidationError::InvalidJson("Root must be a JSON object".into())),
        };

        for field in &self.fields {
            match map.get(&field.name) {
                Some(val) if !val.is_null() => {
                    match &field.field_type {
                        FieldType::Text | FieldType::Email | FieldType::Phone | FieldType::Date => {
                            if !val.is_string() {
                                return Err(ValidationError::TypeMismatch {
                                    field: field.name.clone(),
                                    expected: "string".into(),
                                });
                            }
                        }
                        FieldType::Number => {
                            if !val.is_number() {
                                return Err(ValidationError::TypeMismatch {
                                    field: field.name.clone(),
                                    expected: "number".into(),
                                });
                            }
                        }
                        FieldType::Boolean => {
                            if !val.is_boolean() {
                                return Err(ValidationError::TypeMismatch {
                                    field: field.name.clone(),
                                    expected: "boolean".into(),
                                });
                            }
                        }
                        FieldType::Select(options) => {
                            if let Some(s) = val.as_str() {
                                if !options.iter().any(|opt| opt == s) {
                                    return Err(ValidationError::InvalidOption {
                                        field: field.name.clone(),
                                        value: s.to_string(),
                                    });
                                }
                            } else {
                                return Err(ValidationError::TypeMismatch {
                                    field: field.name.clone(),
                                    expected: "string (select option)".into(),
                                });
                            }
                        }
                    }
                }
                _ => {
                    if field.required {
                        return Err(ValidationError::MissingRequiredField(field.name.clone()));
                    }
                }
            }
        }
        Ok(())
    }
}

// Database extensions for Entity Schema storage & retrieval
impl Database {
    pub fn save_schema(&self, project_id: &str, schema: &EntitySchema) -> Result<(), DbError> {
        let conn = self.conn.lock().map_err(|e| DbError::Lock(e.to_string()))?;
        let json = serde_json::to_string(schema).map_err(|e| DbError::Lock(e.to_string()))?;
        conn.execute(
            "INSERT INTO entity_schemas (project_id, entity, schema) VALUES (?1, ?2, ?3)
             ON CONFLICT(project_id, entity) DO UPDATE SET schema = excluded.schema",
            params![project_id, schema.entity, json],
        )?;
        Ok(())
    }

    pub fn get_schema(&self, project_id: &str, entity: &str) -> Result<Option<EntitySchema>, DbError> {
        let conn = self.conn.lock().map_err(|e| DbError::Lock(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT schema FROM entity_schemas WHERE project_id = ?1 AND entity = ?2",
        )?;
        let mut rows = stmt.query(params![project_id, entity])?;
        if let Some(row) = rows.next()? {
            let json: String = row.get(0)?;
            let schema: EntitySchema = serde_json::from_str(&json)
                .map_err(|e| DbError::Lock(e.to_string()))?;
            Ok(Some(schema))
        } else {
            Ok(None)
        }
    }

    pub fn list_schemas(&self, project_id: &str) -> Result<Vec<EntitySchema>, DbError> {
        let conn = self.conn.lock().map_err(|e| DbError::Lock(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT schema FROM entity_schemas WHERE project_id = ?1 ORDER BY entity ASC",
        )?;
        let rows = stmt.query_map(params![project_id], |row| {
            let json: String = row.get(0)?;
            Ok(json)
        })?;
        let mut schemas = Vec::new();
        for r in rows {
            let json = r?;
            if let Ok(schema) = serde_json::from_str::<EntitySchema>(&json) {
                schemas.push(schema);
            }
        }
        Ok(schemas)
    }

    pub fn delete_schema(&self, project_id: &str, entity: &str) -> Result<(), DbError> {
        let conn = self.conn.lock().map_err(|e| DbError::Lock(e.to_string()))?;
        conn.execute(
            "DELETE FROM entity_schemas WHERE project_id = ?1 AND entity = ?2",
            params![project_id, entity],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_validation_success() {
        let schema = EntitySchema::new("lead", "Sales Lead")
            .with_field(FieldDefinition {
                name: "company".into(),
                label: "Company".into(),
                field_type: FieldType::Text,
                required: true,
                default_value: None,
            })
            .with_field(FieldDefinition {
                name: "deal_size".into(),
                label: "Deal Size".into(),
                field_type: FieldType::Number,
                required: false,
                default_value: None,
            })
            .with_field(FieldDefinition {
                name: "status".into(),
                label: "Status".into(),
                field_type: FieldType::Select(vec!["New".into(), "Won".into(), "Lost".into()]),
                required: true,
                default_value: Some(serde_json::json!("New")),
            });

        let valid_json = r#"{"company": "Acme Corp", "deal_size": 15000, "status": "Won"}"#;
        assert!(schema.validate_json(valid_json).is_ok());

        let missing_required = r#"{"deal_size": 15000, "status": "Won"}"#;
        assert_eq!(
            schema.validate_json(missing_required),
            Err(ValidationError::MissingRequiredField("company".into()))
        );

        let type_mismatch = r#"{"company": "Acme Corp", "deal_size": "lots of money", "status": "Won"}"#;
        assert_eq!(
            schema.validate_json(type_mismatch),
            Err(ValidationError::TypeMismatch { field: "deal_size".into(), expected: "number".into() })
        );

        let invalid_select = r#"{"company": "Acme Corp", "status": "Pending"}"#;
        assert_eq!(
            schema.validate_json(invalid_select),
            Err(ValidationError::InvalidOption { field: "status".into(), value: "Pending".into() })
        );
    }

    #[test]
    fn test_database_schema_persistence() {
        let db = Database::new(":memory:").expect("in-memory db");
        let schema = EntitySchema::new("contacts", "Contacts")
            .with_field(FieldDefinition {
                name: "email".into(),
                label: "Email Address".into(),
                field_type: FieldType::Email,
                required: true,
                default_value: None,
            });

        db.save_schema("proj-1", &schema).unwrap();

        let loaded = db.get_schema("proj-1", "contacts").unwrap().expect("should find schema");
        assert_eq!(loaded.entity, "contacts");
        assert_eq!(loaded.fields.len(), 1);
        assert_eq!(loaded.fields[0].name, "email");

        let all = db.list_schemas("proj-1").unwrap();
        assert_eq!(all.len(), 1);

        db.delete_schema("proj-1", "contacts").unwrap();
        assert!(db.get_schema("proj-1", "contacts").unwrap().is_none());
    }
}
