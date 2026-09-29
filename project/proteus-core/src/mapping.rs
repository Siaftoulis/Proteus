//! Visual Field Mapping Engine for Proteus Business Analysts (PCDA).
//! Maps arbitrary external schema payloads (API/JSON/CSV) to native Proteus entities
//! with configurable transformations and fuzzy auto-matching.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use crate::inference::find_best_field_match;

/// Transformations that can be applied to a source field during mapping.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FieldTransform {
    /// Copies source value directly to target.
    PassThrough,
    /// Concatenates this field with another source field using a separator.
    Concatenate { second_field: String, separator: String },
    /// Converts date format (e.g., "DD/MM/YYYY" to "YYYY-MM-DD").
    DateFormat { input_format: String, output_format: String },
    /// Replaces known keys using a dictionary lookup table.
    Lookup { dictionary: HashMap<String, String>, default_value: Option<String> },
    /// Multiplies numeric values by a factor (e.g. 1.24 for 24% VAT).
    MathMultiply { multiplier: f64 },
    /// Converts text to uppercase.
    ToUpperCase,
    /// Converts text to lowercase.
    ToLowerCase,
    /// Prepends a string prefix.
    Prefix { prefix: String },
}

/// A single mapping link between a source field and a target field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldMapping {
    pub source_field: String,
    pub target_field: String,
    pub transform: FieldTransform,
}

impl FieldMapping {
    pub fn new(source: impl Into<String>, target: impl Into<String>, transform: FieldTransform) -> Self {
        Self {
            source_field: source.into(),
            target_field: target.into(),
            transform,
        }
    }

    pub fn pass_through(source: impl Into<String>, target: impl Into<String>) -> Self {
        Self::new(source, target, FieldTransform::PassThrough)
    }
}

/// A certified schema mapping contract created by a PCDA.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SchemaMappingContract {
    pub id: String,
    pub name: String,
    pub source_entity: String,
    pub target_entity: String,
    pub mappings: Vec<FieldMapping>,
}

impl SchemaMappingContract {
    pub fn new(id: impl Into<String>, name: impl Into<String>, source: impl Into<String>, target: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            source_entity: source.into(),
            target_entity: target.into(),
            mappings: Vec::new(),
        }
    }

    pub fn add_mapping(&mut self, mapping: FieldMapping) {
        self.mappings.push(mapping);
    }

    /// Automatically suggests field mappings between source and target fields using fuzzy matching.
    pub fn auto_map_fuzzy(source_fields: &[String], target_fields: &[String], min_confidence: f64) -> Vec<FieldMapping> {
        let mut generated = Vec::new();
        let target_refs: Vec<&str> = target_fields.iter().map(|s| s.as_str()).collect();

        for source in source_fields {
            if let Some((best_target, score)) = find_best_field_match(source, &target_refs) {
                if score >= min_confidence {
                    generated.push(FieldMapping::pass_through(source.clone(), best_target));
                }
            }
        }
        generated
    }

    /// Transforms a single JSON record according to the contract mappings.
    pub fn apply_record(&self, source_record: &Value) -> Value {
        let empty_map = Map::new();
        let source_map = match source_record {
            Value::Object(m) => m,
            _ => &empty_map,
        };

        let mut output_map = Map::new();

        for mapping in &self.mappings {
            let raw_val = source_map.get(&mapping.source_field);
            let transformed_val = apply_transform(&mapping.transform, raw_val, source_map);
            output_map.insert(mapping.target_field.clone(), transformed_val);
        }

        Value::Object(output_map)
    }

    /// Transforms a batch of records (e.g. previewing the first 5 records).
    pub fn apply_batch(&self, records: &[Value]) -> Vec<Value> {
        records.iter().map(|r| self.apply_record(r)).collect()
    }
}

fn apply_transform(transform: &FieldTransform, val: Option<&Value>, all_fields: &Map<String, Value>) -> Value {
    match transform {
        FieldTransform::PassThrough => val.cloned().unwrap_or(Value::Null),
        FieldTransform::Concatenate { second_field, separator } => {
            let part1 = val.and_then(|v| v.as_str()).unwrap_or("");
            let part2 = all_fields.get(second_field).and_then(|v| v.as_str()).unwrap_or("");
            Value::String(format!("{}{}{}", part1, separator, part2).trim().to_string())
        }
        FieldTransform::DateFormat { input_format, output_format } => {
            if let Some(date_str) = val.and_then(|v| v.as_str()) {
                Value::String(convert_date_format(date_str, input_format, output_format))
            } else {
                Value::Null
            }
        }
        FieldTransform::Lookup { dictionary, default_value } => {
            if let Some(key_str) = val.and_then(|v| v.as_str()) {
                if let Some(mapped) = dictionary.get(key_str) {
                    Value::String(mapped.clone())
                } else if let Some(def) = default_value {
                    Value::String(def.clone())
                } else {
                    Value::String(key_str.to_string())
                }
            } else {
                Value::Null
            }
        }
        FieldTransform::MathMultiply { multiplier } => {
            if let Some(num) = val.and_then(|v| v.as_f64()) {
                let res = (num * multiplier * 100.0).round() / 100.0;
                serde_json::Number::from_f64(res)
                    .map(Value::Number)
                    .unwrap_or(Value::Null)
            } else {
                Value::Null
            }
        }
        FieldTransform::ToUpperCase => {
            val.and_then(|v| v.as_str())
                .map(|s| Value::String(s.to_uppercase()))
                .unwrap_or(Value::Null)
        }
        FieldTransform::ToLowerCase => {
            val.and_then(|v| v.as_str())
                .map(|s| Value::String(s.to_lowercase()))
                .unwrap_or(Value::Null)
        }
        FieldTransform::Prefix { prefix } => {
            val.and_then(|v| v.as_str())
                .map(|s| Value::String(format!("{}{}", prefix, s)))
                .unwrap_or(Value::Null)
        }
    }
}

/// Converts simple formatted dates (e.g. DD/MM/YYYY to YYYY-MM-DD or vice-versa).
fn convert_date_format(date_str: &str, in_fmt: &str, out_fmt: &str) -> String {
    let clean_str = date_str.trim();
    let parts: Vec<&str> = clean_str.split(['/', '-', '.']).collect();
    if parts.len() != 3 {
        return date_str.to_string();
    }

    if in_fmt == "DD/MM/YYYY" && out_fmt == "YYYY-MM-DD" {
        return format!("{}-{:0>2}-{:0>2}", parts[2], parts[1], parts[0]);
    }
    if in_fmt == "MM/DD/YYYY" && out_fmt == "YYYY-MM-DD" {
        return format!("{}-{:0>2}-{:0>2}", parts[2], parts[0], parts[1]);
    }
    if in_fmt == "YYYY-MM-DD" && out_fmt == "DD/MM/YYYY" {
        return format!("{:0>2}/{:0>2}/{}", parts[2], parts[1], parts[0]);
    }

    date_str.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_passthrough_and_concatenation() {
        let mut contract = SchemaMappingContract::new("C-01", "Customer Map", "legacy_crm", "proteus_contacts");
        contract.add_mapping(FieldMapping::pass_through("cust_id", "external_id"));
        contract.add_mapping(FieldMapping::new(
            "first_name",
            "full_name",
            FieldTransform::Concatenate {
                second_field: "last_name".to_string(),
                separator: " ".to_string(),
            },
        ));

        let source = json!({
            "cust_id": "ACC-99",
            "first_name": "Nikolaos",
            "last_name": "Papadopoulos"
        });

        let target = contract.apply_record(&source);
        assert_eq!(target["external_id"], "ACC-99");
        assert_eq!(target["full_name"], "Nikolaos Papadopoulos");
    }

    #[test]
    fn test_date_and_lookup_transform() {
        let mut lookup = HashMap::new();
        lookup.insert("P".to_string(), "Pending".to_string());
        lookup.insert("C".to_string(), "Completed".to_string());

        let mut contract = SchemaMappingContract::new("C-02", "Order Map", "api_orders", "orders");
        contract.add_mapping(FieldMapping::new(
            "order_date",
            "iso_date",
            FieldTransform::DateFormat {
                input_format: "DD/MM/YYYY".to_string(),
                output_format: "YYYY-MM-DD".to_string(),
            },
        ));
        contract.add_mapping(FieldMapping::new(
            "status_code",
            "status_name",
            FieldTransform::Lookup {
                dictionary: lookup,
                default_value: Some("Unknown".to_string()),
            },
        ));

        let source = json!({
            "order_date": "24/12/2026",
            "status_code": "C"
        });

        let target = contract.apply_record(&source);
        assert_eq!(target["iso_date"], "2026-12-24");
        assert_eq!(target["status_name"], "Completed");
    }

    #[test]
    fn test_math_multiply_and_case() {
        let mut contract = SchemaMappingContract::new("C-03", "Pricing Map", "items", "products");
        contract.add_mapping(FieldMapping::new(
            "net_price",
            "gross_price",
            FieldTransform::MathMultiply { multiplier: 1.24 },
        ));
        contract.add_mapping(FieldMapping::new(
            "sku",
            "sku_upper",
            FieldTransform::ToUpperCase,
        ));

        let source = json!({
            "net_price": 100.0,
            "sku": "prod-abc"
        });

        let target = contract.apply_record(&source);
        assert_eq!(target["gross_price"], 124.0);
        assert_eq!(target["sku_upper"], "PROD-ABC");
    }

    #[test]
    fn test_auto_map_fuzzy() {
        let sources = vec!["cust_name".to_string(), "phone".to_string(), "unknown_col".to_string()];
        let targets = vec!["customer_name".to_string(), "phone_number".to_string(), "address".to_string()];

        let mappings = SchemaMappingContract::auto_map_fuzzy(&sources, &targets, 0.6);
        assert_eq!(mappings.len(), 2);
        assert_eq!(mappings[0].source_field, "cust_name");
        assert_eq!(mappings[0].target_field, "customer_name");
        assert_eq!(mappings[1].source_field, "phone");
        assert_eq!(mappings[1].target_field, "phone_number");
    }
}
