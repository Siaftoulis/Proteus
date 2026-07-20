// ponytail: minimal data model — JSON fields in SQLite, no schema engine yet
use crate::{Database, DbError};
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Record {
    pub id: String,
    pub project_id: String,
    pub entity: String,
    pub fields: String,
    pub created_at: String,
    pub updated_at: String,
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
        Ok(Record { id, project_id: project_id.to_string(), entity: entity.to_string(), fields: fields.to_string(), created_at: now.clone(), updated_at: now })
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

    pub fn update_record(&self, id: &str, fields: &str) -> Result<(), DbError> {
        let conn = self.conn.lock().map_err(|e| DbError::Lock(e.to_string()))?;
        let now = chrono::Utc::now().to_rfc3339();
        let updated = conn.execute(
            "UPDATE records SET fields = ?1, updated_at = ?2 WHERE id = ?3",
            params![fields, now, id],
        )?;
        if updated == 0 { return Err(DbError::NotFound); }
        Ok(())
    }

    pub fn delete_record(&self, id: &str) -> Result<(), DbError> {
        let conn = self.conn.lock().map_err(|e| DbError::Lock(e.to_string()))?;
        let deleted = conn.execute("DELETE FROM records WHERE id = ?1", params![id])?;
        if deleted == 0 { return Err(DbError::NotFound); }
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
}

#[cfg(test)]
mod tests {
    use crate::Database;

    fn test_db() -> Database {
        Database::new(":memory:").expect("in-memory db")
    }

    #[test]
    fn test_create_record() {
        let db = test_db();
        let r = db.create_record("p1", "contact", r#"{"name":"John"}"#).unwrap();
        assert_eq!(r.entity, "contact");
        assert!(r.fields.contains("John"));
    }

    #[test]
    fn test_list_records_empty() {
        let db = test_db();
        let records = db.list_records("p1", "contact").unwrap();
        assert!(records.is_empty());
    }

    #[test]
    fn test_list_records_by_entity() {
        let db = test_db();
        db.create_record("p1", "contact", "{}").unwrap();
        db.create_record("p1", "contact", "{}").unwrap();
        db.create_record("p1", "company", "{}").unwrap();
        assert_eq!(db.list_records("p1", "contact").unwrap().len(), 2);
        assert_eq!(db.list_records("p1", "company").unwrap().len(), 1);
    }

    #[test]
    fn test_update_record() {
        let db = test_db();
        let r = db.create_record("p1", "contact", r#"{"name":"Old"}"#).unwrap();
        db.update_record(&r.id, r#"{"name":"New"}"#).unwrap();
        let records = db.list_records("p1", "contact").unwrap();
        assert_eq!(records[0].fields, r#"{"name":"New"}"#);
    }

    #[test]
    fn test_delete_record() {
        let db = test_db();
        let r = db.create_record("p1", "contact", "{}").unwrap();
        db.delete_record(&r.id).unwrap();
        assert!(db.list_records("p1", "contact").unwrap().is_empty());
    }

    #[test]
    fn test_delete_nonexistent_fails() {
        let db = test_db();
        let result = db.delete_record("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_list_entities() {
        let db = test_db();
        db.create_record("p1", "contact", "{}").unwrap();
        db.create_record("p1", "company", "{}").unwrap();
        let entities = db.list_record_entities("p1").unwrap();
        assert_eq!(entities.len(), 2);
        assert!(entities.contains(&"contact".to_string()));
    }
}
