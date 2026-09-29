//! Schema Inference & Universal Data Connector Engine for Proteus BOS.
//! Converts arbitrary, messy, and nested JSON or tabular data into normalized relational schemas.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InferredType {
    Integer,
    Real,
    Text,
    Boolean,
    Jsonb,
}

impl InferredType {
    pub fn to_sqlite_type(&self) -> &'static str {
        match self {
            Self::Integer => "INTEGER",
            Self::Real => "REAL",
            Self::Text => "TEXT",
            Self::Boolean => "INTEGER", // SQLite stores booleans as 0 or 1
            Self::Jsonb => "TEXT",
        }
    }

    pub fn to_postgres_type(&self) -> &'static str {
        match self {
            Self::Integer => "BIGINT",
            Self::Real => "DOUBLE PRECISION",
            Self::Text => "TEXT",
            Self::Boolean => "BOOLEAN",
            Self::Jsonb => "JSONB",
        }
    }

    pub fn merge(self, other: Self) -> Self {
        if self == other {
            return self;
        }
        match (self, other) {
            (Self::Integer, Self::Real) | (Self::Real, Self::Integer) => Self::Real,
            (Self::Text, _) | (_, Self::Text) => Self::Text,
            (Self::Jsonb, _) | (_, Self::Jsonb) => Self::Jsonb,
            _ => Self::Text,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferredColumn {
    pub name: String,
    pub col_type: InferredType,
    pub is_nullable: bool,
    pub is_primary_key: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferredTable {
    pub table_name: String,
    pub columns: Vec<InferredColumn>,
    pub primary_key: Option<String>,
    pub sample_rows_count: usize,
}

impl InferredTable {
    pub fn to_sqlite_ddl(&self) -> String {
        let mut ddl = format!("CREATE TABLE IF NOT EXISTS {} (\n", self.table_name);
        let col_defs: Vec<String> = self
            .columns
            .iter()
            .map(|col| {
                let mut def = format!("    {} {}", col.name, col.col_type.to_sqlite_type());
                if col.is_primary_key {
                    def.push_str(" PRIMARY KEY");
                } else if !col.is_nullable {
                    def.push_str(" NOT NULL");
                }
                def
            })
            .collect();
        ddl.push_str(&col_defs.join(",\n"));
        ddl.push_str("\n);");
        ddl
    }

    pub fn to_postgres_ddl(&self) -> String {
        let mut ddl = format!("CREATE TABLE IF NOT EXISTS {} (\n", self.table_name);
        let col_defs: Vec<String> = self
            .columns
            .iter()
            .map(|col| {
                let mut def = format!("    {} {}", col.name, col.col_type.to_postgres_type());
                if col.is_primary_key {
                    def.push_str(" PRIMARY KEY");
                } else if !col.is_nullable {
                    def.push_str(" NOT NULL");
                }
                def
            })
            .collect();
        ddl.push_str(&col_defs.join(",\n"));
        ddl.push_str("\n);");
        ddl
    }
}

pub struct SchemaInferer;

impl SchemaInferer {
    /// Infer relational schema from an array of JSON objects or a single JSON object.
    pub fn infer_from_json(table_name: &str, json_payload: &Value) -> Result<InferredTable, &'static str> {
        let records = match json_payload {
            Value::Array(arr) => {
                if arr.is_empty() {
                    return Err("Cannot infer schema from empty JSON array");
                }
                arr.as_slice()
            }
            Value::Object(_) => std::slice::from_ref(json_payload),
            _ => return Err("Payload must be a JSON object or array of objects"),
        };

        let mut column_types: BTreeMap<String, InferredType> = BTreeMap::new();
        let mut column_nullability: BTreeMap<String, bool> = BTreeMap::new();
        let total_records = records.len();

        for record in records {
            if let Value::Object(map) = record {
                let mut flattened = BTreeMap::new();
                flatten_json_object("", map, &mut flattened);

                for (col_name, val) in &flattened {
                    match val {
                        Value::Null => {
                            column_nullability.insert(col_name.clone(), true);
                        }
                        _ => {
                            let val_type = detect_value_type(val);
                            let merged_type = if let Some(existing) = column_types.get(col_name) {
                                existing.merge(val_type)
                            } else {
                                val_type
                            };
                            column_types.insert(col_name.clone(), merged_type);
                        }
                    }
                }

                // Check for columns missing in this specific record
                for col_name in column_types.keys() {
                    if !flattened.contains_key(col_name) {
                        column_nullability.insert(col_name.clone(), true);
                    }
                }
            }
        }

        // Primary key detection heuristic
        let pk_col = Self::detect_primary_key(table_name, &column_types);

        let columns: Vec<InferredColumn> = column_types
            .into_iter()
            .map(|(name, col_type)| {
                let is_pk = pk_col.as_deref() == Some(name.as_str());
                let is_nullable = if is_pk {
                    false
                } else {
                    column_nullability.get(&name).copied().unwrap_or(false)
                };
                InferredColumn {
                    name,
                    col_type,
                    is_nullable,
                    is_primary_key: is_pk,
                }
            })
            .collect();

        Ok(InferredTable {
            table_name: table_name.to_string(),
            columns,
            primary_key: pk_col,
            sample_rows_count: total_records,
        })
    }

    fn detect_primary_key(table_name: &str, columns: &BTreeMap<String, InferredType>) -> Option<String> {
        let candidates = ["id", "uuid", "_id", "code", "sku", "barcode", "key"];

        for candidate in candidates {
            if columns.contains_key(candidate) {
                return Some(candidate.to_string());
            }
        }

        // Check for {table}_id
        let table_id = format!("{}_id", table_name.trim_end_matches('s'));
        if columns.contains_key(&table_id) {
            return Some(table_id);
        }

        None
    }
}

fn detect_value_type(val: &Value) -> InferredType {
    match val {
        Value::Bool(_) => InferredType::Boolean,
        Value::Number(num) => {
            if num.is_i64() || num.is_u64() {
                InferredType::Integer
            } else {
                InferredType::Real
            }
        }
        Value::String(_) => InferredType::Text,
        Value::Array(_) | Value::Object(_) => InferredType::Jsonb,
        Value::Null => InferredType::Text,
    }
}

fn flatten_json_object(prefix: &str, map: &Map<String, Value>, output: &mut BTreeMap<String, Value>) {
    for (key, value) in map {
        let sanitized_key = key.trim().to_lowercase().replace([' ', '-'], "_");
        let full_key = if prefix.is_empty() {
            sanitized_key
        } else {
            format!("{}_{}", prefix, sanitized_key)
        };

        match value {
            Value::Object(inner_map) => {
                flatten_json_object(&full_key, inner_map, output);
            }
            _ => {
                output.insert(full_key, value.clone());
            }
        }
    }
}

/// Calculates Levenshtein edit distance between two strings.
pub fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let m = a_chars.len();
    let n = b_chars.len();

    if m == 0 { return n; }
    if n == 0 { return m; }

    let mut dp = vec![vec![0; n + 1]; m + 1];
    for (i, row) in dp.iter_mut().enumerate().take(m + 1) {
        row[0] = i;
    }
    for (j, cell) in dp[0].iter_mut().enumerate().take(n + 1) {
        *cell = j;
    }

    for i in 1..=m {
        for j in 1..=n {
            let cost = if a_chars[i - 1].eq_ignore_ascii_case(&b_chars[j - 1]) {
                0
            } else {
                1
            };
            dp[i][j] = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + cost);
        }
    }
    dp[m][n]
}

/// Computes normalized string similarity (0.0 to 1.0) between two field names.
pub fn field_similarity(source: &str, target: &str) -> f64 {
    let s = source.to_lowercase().replace(['_', '-'], "");
    let t = target.to_lowercase().replace(['_', '-'], "");
    if s == t {
        return 1.0;
    }
    if s.contains(&t) || t.contains(&s) {
        let min_len = s.len().min(t.len()) as f64;
        let max_len = s.len().max(t.len()) as f64;
        return (0.65 + 0.35 * (min_len / max_len)).min(1.0);
    }
    let max_len = s.len().max(t.len());
    if max_len == 0 {
        return 1.0;
    }
    let dist = levenshtein_distance(&s, &t);
    1.0 - (dist as f64 / max_len as f64)
}

/// Finds the best matching candidate from a list of schema fields.
pub fn find_best_field_match<'a>(source_field: &str, candidates: &'a [&'a str]) -> Option<(&'a str, f64)> {
    let mut best: Option<(&'a str, f64)> = None;
    for &candidate in candidates {
        let sim = field_similarity(source_field, candidate);
        if sim >= 0.45 {
            if let Some((_, best_sim)) = best {
                if sim > best_sim {
                    best = Some((candidate, sim));
                }
            } else {
                best = Some((candidate, sim));
            }
        }
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_flat_json_inference() {
        let payload = json!({
            "id": "item_001",
            "name": "Mechanical Keyboard",
            "price": 129.99,
            "stock_qty": 42,
            "is_active": true
        });

        let table = SchemaInferer::infer_from_json("products", &payload).unwrap();
        assert_eq!(table.table_name, "products");
        assert_eq!(table.primary_key.as_deref(), Some("id"));
        assert_eq!(table.columns.len(), 5);

        let ddl = table.to_sqlite_ddl();
        assert!(ddl.contains("id TEXT PRIMARY KEY"));
        assert!(ddl.contains("price REAL"));
        assert!(ddl.contains("stock_qty INTEGER"));
        assert!(ddl.contains("is_active INTEGER"));
    }

    #[test]
    fn test_nested_object_flattening() {
        let payload = json!({
            "customer": {
                "id": 1001,
                "profile": {
                    "full_name": "Elena P.",
                    "city": "Thessaloniki"
                }
            },
            "total_spent": 450.50
        });

        let table = SchemaInferer::infer_from_json("customers", &payload).unwrap();
        assert_eq!(table.primary_key.as_deref(), Some("customer_id"));

        let col_names: Vec<&str> = table.columns.iter().map(|c| c.name.as_str()).collect();
        assert!(col_names.contains(&"customer_id"));
        assert!(col_names.contains(&"customer_profile_full_name"));
        assert!(col_names.contains(&"customer_profile_city"));
        assert!(col_names.contains(&"total_spent"));
    }

    #[test]
    fn test_messy_batch_records_merging() {
        let batch = json!([
            { "id": "1", "score": 10 },
            { "id": "2", "score": 15.5, "note": "Promoted" },
            { "id": "3", "score": null, "extra_flag": false }
        ]);

        let table = SchemaInferer::infer_from_json("scores", &batch).unwrap();
        assert_eq!(table.sample_rows_count, 3);

        let score_col = table.columns.iter().find(|c| c.name == "score").unwrap();
        assert_eq!(score_col.col_type, InferredType::Real);
        assert!(score_col.is_nullable);

        let note_col = table.columns.iter().find(|c| c.name == "note").unwrap();
        assert!(note_col.is_nullable);
    }

    #[test]
    fn test_fuzzy_field_matching() {
        let candidates = &["customer_name", "email_address", "phone_number", "total_price"];
        let (matched, sim) = find_best_field_match("cust_name", candidates).unwrap();
        assert_eq!(matched, "customer_name");
        assert!(sim > 0.6);

        let (matched_phone, _) = find_best_field_match("phone", candidates).unwrap();
        assert_eq!(matched_phone, "phone_number");
    }
}
