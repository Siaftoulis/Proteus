//! Abstract Syntax Tree (AST) definitions for the Proteus Database Scripting DSL.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DslFieldType {
    Uuid,
    Text,
    Integer,
    Decimal,
    Boolean,
    Timestamp,
    Json,
}

impl DslFieldType {
    pub fn parse_type_name(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "uuid" => Some(Self::Uuid),
            "text" | "string" => Some(Self::Text),
            "integer" | "int" => Some(Self::Integer),
            "decimal" | "float" | "double" => Some(Self::Decimal),
            "bool" | "boolean" => Some(Self::Boolean),
            "timestamp" | "datetime" => Some(Self::Timestamp),
            "json" => Some(Self::Json),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DslField {
    pub name: String,
    pub field_type: DslFieldType,
    pub is_primary: bool,
    pub is_required: bool,
    pub is_unique: bool,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DslIndex {
    pub name: String,
    pub fields: Vec<String>,
    pub is_unique: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DslEntity {
    pub name: String,
    pub fields: Vec<DslField>,
    pub indexes: Vec<DslIndex>,
}

impl DslEntity {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            fields: Vec::new(),
            indexes: Vec::new(),
        }
    }

    pub fn get_primary_key(&self) -> Option<&DslField> {
        self.fields.iter().find(|f| f.is_primary)
    }

    pub fn find_field(&self, name: &str) -> Option<&DslField> {
        self.fields.iter().find(|f| f.name.eq_ignore_ascii_case(name))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationKind {
    OneToOne,
    OneToMany,
    ManyToMany,
}

impl RelationKind {
    pub fn parse_kind(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "one_to_one" | "1:1" => Some(Self::OneToOne),
            "one_to_many" | "1:n" | "1:m" => Some(Self::OneToMany),
            "many_to_many" | "m:n" | "n:m" => Some(Self::ManyToMany),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DslRelation {
    pub from_entity: String,
    pub from_field: String,
    pub to_entity: String,
    pub to_field: String,
    pub kind: RelationKind,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DslSchema {
    pub entities: Vec<DslEntity>,
    pub relations: Vec<DslRelation>,
}

impl DslSchema {
    pub fn find_entity(&self, name: &str) -> Option<&DslEntity> {
        self.entities.iter().find(|e| e.name.eq_ignore_ascii_case(name))
    }
}
