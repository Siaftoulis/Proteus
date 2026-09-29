//! Bespoke EAV & Document Data Engine for Proteus CRM Core.
//! Standardizes the `records` table for all dynamic business entities:
//! - High-speed CRUD operations with timestamping and UUIDs.
//! - In-memory and SQL-assisted filtering, search, and pagination.
//! - Schema validation bridge through EntitySchema.
//! - High-performance batched bulk imports wrapped in atomic transactions.

use crate::schema::EntitySchema;
use crate::{Database, DbError};
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Record {
    pub id: String,
    pub project_id: String,
    pub entity: String,
    pub fields: String,
    pub created_at: String,
    pub updated_at: String,
}

impl Record {
    /// Helper to parse fields as a `serde_json::Value`.
    pub fn parse_fields(&self) -> Option<serde_json::Value> {
        serde_json::from_str(&self.fields).ok()
    }

    /// Retrieve a specific string field from the JSON document.
    pub fn get_str(&self, key: &str) -> Option<String> {
        let v = self.parse_fields()?;
        v.get(key).and_then(|val| val.as_str()).map(|s| s.to_string())
    }
}

impl Database {
    pub fn create_record(&self, project_id: &str, entity: &str, fields: &str) -> Result<Record, DbError> {
        let conn = self.conn.lock().map_err(|e| DbError::Lock(e.to_string()))?;
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO records (id, project_id, entity, fields, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, project_id, entity, fields, now, now],
        )?;
        Ok(Record {
            id,
            project_id: project_id.to_string(),
            entity: entity.to_string(),
            fields: fields.to_string(),
            created_at: now.clone(),
            updated_at: now,
        })
    }

    /// Validates fields against schema before creating record.
    pub fn create_record_validated(
        &self,
        project_id: &str,
        entity: &str,
        fields: &str,
        schema: &EntitySchema,
    ) -> Result<Record, DbError> {
        schema
            .validate_json(fields)
            .map_err(|e| DbError::Validation(e.to_string()))?;
        self.create_record(project_id, entity, fields)
    }

    pub fn get_record(&self, id: &str) -> Result<Record, DbError> {
        let conn = self.conn.lock().map_err(|e| DbError::Lock(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, project_id, entity, fields, created_at, updated_at FROM records WHERE id = ?1",
        )?;
        stmt.query_row(params![id], |row| {
            Ok(Record {
                id: row.get(0)?,
                project_id: row.get(1)?,
                entity: row.get(2)?,
                fields: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => DbError::NotFound,
            other => DbError::Sql(other),
        })
    }

    pub fn list_records(&self, project_id: &str, entity: &str) -> Result<Vec<Record>, DbError> {
        let conn = self.conn.lock().map_err(|e| DbError::Lock(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, project_id, entity, fields, created_at, updated_at FROM records WHERE project_id = ?1 AND entity = ?2 ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map(params![project_id, entity], |row| {
            Ok(Record {
                id: row.get(0)?,
                project_id: row.get(1)?,
                entity: row.get(2)?,
                fields: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?;
        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }
        Ok(records)
    }

    pub fn count_records(&self, project_id: &str, entity: &str) -> Result<usize, DbError> {
        let conn = self.conn.lock().map_err(|e| DbError::Lock(e.to_string()))?;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM records WHERE project_id = ?1 AND entity = ?2",
            params![project_id, entity],
            |r| r.get(0),
        )?;
        Ok(count as usize)
    }

    pub fn paginate_records(
        &self,
        project_id: &str,
        entity: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Record>, DbError> {
        let conn = self.conn.lock().map_err(|e| DbError::Lock(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, project_id, entity, fields, created_at, updated_at FROM records WHERE project_id = ?1 AND entity = ?2 ORDER BY created_at DESC LIMIT ?3 OFFSET ?4",
        )?;
        let rows = stmt.query_map(params![project_id, entity, limit as i64, offset as i64], |row| {
            Ok(Record {
                id: row.get(0)?,
                project_id: row.get(1)?,
                entity: row.get(2)?,
                fields: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?;
        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }
        Ok(records)
    }

    pub fn search_records(&self, project_id: &str, entity: &str, query: &str) -> Result<Vec<Record>, DbError> {
        let pattern = format!("%{}%", query);
        let conn = self.conn.lock().map_err(|e| DbError::Lock(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT id, project_id, entity, fields, created_at, updated_at FROM records WHERE project_id = ?1 AND entity = ?2 AND fields LIKE ?3 ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map(params![project_id, entity, pattern], |row| {
            Ok(Record {
                id: row.get(0)?,
                project_id: row.get(1)?,
                entity: row.get(2)?,
                fields: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?;
        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }
        Ok(records)
    }

    pub fn filter_records_by_field(
        &self,
        project_id: &str,
        entity: &str,
        key: &str,
        value: &str,
    ) -> Result<Vec<Record>, DbError> {
        let all = self.list_records(project_id, entity)?;
        let filtered = all
            .into_iter()
            .filter(|rec| {
                if let Some(val) = rec.get_str(key) {
                    val.eq_ignore_ascii_case(value)
                } else {
                    false
                }
            })
            .collect();
        Ok(filtered)
    }

    pub fn update_record(&self, id: &str, fields: &str) -> Result<(), DbError> {
        let conn = self.conn.lock().map_err(|e| DbError::Lock(e.to_string()))?;
        let now = chrono::Utc::now().to_rfc3339();
        let updated = conn.execute(
            "UPDATE records SET fields = ?1, updated_at = ?2 WHERE id = ?3",
            params![fields, now, id],
        )?;
        if updated == 0 {
            return Err(DbError::NotFound);
        }
        Ok(())
    }

    pub fn update_record_validated(
        &self,
        id: &str,
        fields: &str,
        schema: &EntitySchema,
    ) -> Result<(), DbError> {
        schema
            .validate_json(fields)
            .map_err(|e| DbError::Validation(e.to_string()))?;
        self.update_record(id, fields)
    }

    pub fn delete_record(&self, id: &str) -> Result<(), DbError> {
        let conn = self.conn.lock().map_err(|e| DbError::Lock(e.to_string()))?;
        let deleted = conn.execute("DELETE FROM records WHERE id = ?1", params![id])?;
        if deleted == 0 {
            return Err(DbError::NotFound);
        }
        Ok(())
    }

    pub fn list_record_entities(&self, project_id: &str) -> Result<Vec<String>, DbError> {
        let conn = self.conn.lock().map_err(|e| DbError::Lock(e.to_string()))?;
        let mut stmt = conn.prepare(
            "SELECT DISTINCT entity FROM records WHERE project_id = ?1 ORDER BY entity",
        )?;
        let rows = stmt.query_map(params![project_id], |row| row.get::<_, String>(0))?;
        let mut entities = Vec::new();
        for row in rows {
            entities.push(row?);
        }
        Ok(entities)
    }

    pub fn batch_import_records(
        &self,
        project_id: &str,
        entity: &str,
        records_json: &[String],
    ) -> Result<usize, DbError> {
        let mut conn = self.conn.lock().map_err(|e| DbError::Lock(e.to_string()))?;
        let tx = conn.transaction()?;
        let now = chrono::Utc::now().to_rfc3339();
        let mut count = 0;
        {
            let mut stmt = tx.prepare(
                "INSERT INTO records (id, project_id, entity, fields, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )?;
            for fields in records_json {
                let id = uuid::Uuid::new_v4().to_string();
                stmt.execute(params![id, project_id, entity, fields, now, now])?;
                count += 1;
            }
        }
        tx.commit()?;
        Ok(count)
    }

    /// Access the high-level DataEngine wrapper.
    pub fn engine(&self) -> DataEngine<'_> {
        DataEngine { db: self }
    }
}

/// High-level DataEngine facade for workspace runtimes.
pub struct DataEngine<'a> {
    pub db: &'a Database,
}

impl<'a> DataEngine<'a> {
    pub fn find(&self, id: &str) -> Result<Record, DbError> {
        self.db.get_record(id)
    }

    pub fn all(&self, project_id: &str, entity: &str) -> Result<Vec<Record>, DbError> {
        self.db.list_records(project_id, entity)
    }

    pub fn search(&self, project_id: &str, entity: &str, query: &str) -> Result<Vec<Record>, DbError> {
        self.db.search_records(project_id, entity, query)
    }

    pub fn count(&self, project_id: &str, entity: &str) -> Result<usize, DbError> {
        self.db.count_records(project_id, entity)
    }

    pub fn paginate(&self, project_id: &str, entity: &str, limit: usize, offset: usize) -> Result<Vec<Record>, DbError> {
        self.db.paginate_records(project_id, entity, limit, offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{FieldDefinition, FieldType};

    fn test_db() -> Database {
        Database::new(":memory:").expect("in-memory db")
    }

    #[test]
    fn test_create_and_get_record() {
        let db = test_db();
        let r = db.create_record("p1", "contact", r#"{"name":"John","phone":"2101234567"}"#).unwrap();
        assert_eq!(r.entity, "contact");
        assert_eq!(r.get_str("name").as_deref(), Some("John"));
        assert_eq!(r.get_str("phone").as_deref(), Some("2101234567"));

        let fetched = db.get_record(&r.id).unwrap();
        assert_eq!(fetched.id, r.id);
        assert_eq!(fetched.fields, r.fields);
    }

    #[test]
    fn test_get_nonexistent_returns_not_found() {
        let db = test_db();
        let res = db.get_record("nonexistent_id");
        match res {
            Err(DbError::NotFound) => {}
            _ => panic!("Expected DbError::NotFound"),
        }
    }

    #[test]
    fn test_list_and_count_and_paginate() {
        let db = test_db();
        for i in 1..=10 {
            db.create_record("p1", "item", &format!(r#"{{"sku":"SKU_{}","stock":{}}}"#, i, i)).unwrap();
        }

        assert_eq!(db.count_records("p1", "item").unwrap(), 10);
        let page1 = db.paginate_records("p1", "item", 4, 0).unwrap();
        assert_eq!(page1.len(), 4);

        let page2 = db.paginate_records("p1", "item", 4, 4).unwrap();
        assert_eq!(page2.len(), 4);

        let page3 = db.paginate_records("p1", "item", 4, 8).unwrap();
        assert_eq!(page3.len(), 2);
    }

    #[test]
    fn test_search_records() {
        let db = test_db();
        db.create_record("p1", "contact", r#"{"name":"Dimitris","city":"Athens"}"#).unwrap();
        db.create_record("p1", "contact", r#"{"name":"Eleni","city":"Thessaloniki"}"#).unwrap();
        db.create_record("p1", "contact", r#"{"name":"Kostas","city":"Athens"}"#).unwrap();

        let athens = db.search_records("p1", "contact", "Athens").unwrap();
        assert_eq!(athens.len(), 2);

        let dim = db.search_records("p1", "contact", "Dimitris").unwrap();
        assert_eq!(dim.len(), 1);
    }

    #[test]
    fn test_filter_records_by_field() {
        let db = test_db();
        db.create_record("p1", "ticket", r#"{"status":"Open","priority":"High"}"#).unwrap();
        db.create_record("p1", "ticket", r#"{"status":"Closed","priority":"Low"}"#).unwrap();
        db.create_record("p1", "ticket", r#"{"status":"Open","priority":"Medium"}"#).unwrap();

        let open = db.filter_records_by_field("p1", "ticket", "status", "Open").unwrap();
        assert_eq!(open.len(), 2);

        let closed = db.filter_records_by_field("p1", "ticket", "status", "Closed").unwrap();
        assert_eq!(closed.len(), 1);
    }

    #[test]
    fn test_validated_create_and_update() {
        let db = test_db();
        let schema = EntitySchema::new("client", "Clients")
            .with_field(FieldDefinition {
                name: "company".to_string(),
                label: "Company Name".to_string(),
                field_type: FieldType::Text,
                required: true,
                default_value: None,
            });

        // Missing required field -> fails
        let fail = db.create_record_validated("p1", "client", r#"{"phone":"123"}"#, &schema);
        assert!(matches!(fail, Err(DbError::Validation(_))));

        // Valid -> succeeds
        let ok = db.create_record_validated("p1", "client", r#"{"company":"Acme Corp"}"#, &schema).unwrap();
        assert_eq!(ok.get_str("company").as_deref(), Some("Acme Corp"));

        // Valid update
        let upd_ok = db.update_record_validated(&ok.id, r#"{"company":"Acme Global"}"#, &schema);
        assert!(upd_ok.is_ok());

        // Invalid update -> fails
        let upd_fail = db.update_record_validated(&ok.id, r#"{"other":"val"}"#, &schema);
        assert!(matches!(upd_fail, Err(DbError::Validation(_))));
    }

    #[test]
    fn test_batch_import_records() {
        let db = test_db();
        let batch = vec![
            r#"{"name":"Alpha"}"#.to_string(),
            r#"{"name":"Beta"}"#.to_string(),
            r#"{"name":"Gamma"}"#.to_string(),
        ];

        let count = db.batch_import_records("p1", "partner", &batch).unwrap();
        assert_eq!(count, 3);
        assert_eq!(db.count_records("p1", "partner").unwrap(), 3);
    }

    #[test]
    fn test_data_engine_facade() {
        let db = test_db();
        db.create_record("p1", "lead", r#"{"name":"Lead 1"}"#).unwrap();
        db.create_record("p1", "lead", r#"{"name":"Lead 2"}"#).unwrap();

        let engine = db.engine();
        assert_eq!(engine.count("p1", "lead").unwrap(), 2);
        assert_eq!(engine.all("p1", "lead").unwrap().len(), 2);
        assert_eq!(engine.search("p1", "lead", "Lead 1").unwrap().len(), 1);
    }
}
