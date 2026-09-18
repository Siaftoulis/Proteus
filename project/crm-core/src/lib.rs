pub mod audit;
pub mod data;
pub mod drivers;
pub mod encryption;
pub mod event_bus;
pub mod export;
pub mod ffi;
pub mod flow;
pub mod gs1;
pub mod inference;
pub mod license;
pub mod mapping;
pub mod migrations;
pub mod package;
pub mod paths;
pub mod printer;
pub mod replication;
pub mod roles;
pub mod rules;
pub mod schema;
pub mod sync;
pub mod tickets;


use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use thiserror::Error;
use tracing::instrument;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("Database error: {0}")]
    Sql(#[from] rusqlite::Error),
    #[error("Lock error: {0}")]
    Lock(String),
    #[error("Not found")]
    NotFound,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub widgets: String,
    pub flows: String,
    pub created_at: String,
    pub updated_at: String,
}

pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    pub fn new(path: &str) -> Result<Self, DbError> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                widgets TEXT NOT NULL DEFAULT '[]',
                flows TEXT NOT NULL DEFAULT '[]',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS records (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                entity TEXT NOT NULL,
                fields TEXT NOT NULL DEFAULT '{}',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_records_project ON records(project_id);
            CREATE INDEX IF NOT EXISTS idx_records_entity ON records(entity);
            CREATE TABLE IF NOT EXISTS entity_schemas (
                project_id TEXT NOT NULL,
                entity TEXT NOT NULL,
                schema TEXT NOT NULL,
                PRIMARY KEY(project_id, entity)
            );",
        )?;
        Ok(Database {
            conn: Mutex::new(conn),
        })
    }

    #[instrument(skip(self), fields(project_name = %name))]
    pub fn create_project(&self, name: &str) -> Result<Project, DbError> {
        let conn = self.conn.lock().map_err(|e| DbError::Lock(e.to_string()))?;
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO projects (id, name, widgets, flows, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, name, "[]", "[]", now, now],
        )?;
        tracing::info!(project_id = %id, "Project created");
        Ok(Project {
            id,
            name: name.to_string(),
            widgets: "[]".to_string(),
            flows: "[]".to_string(),
            created_at: now.clone(),
            updated_at: now,
        })
    }

    #[instrument(skip(self))]
    pub fn list_projects(&self) -> Result<Vec<Project>, DbError> {
        let conn = self.conn.lock().map_err(|e| DbError::Lock(e.to_string()))?;
        let mut stmt = conn
            .prepare("SELECT id, name, widgets, flows, created_at, updated_at FROM projects ORDER BY updated_at DESC")?;
        let rows = stmt.query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                widgets: row.get(2)?,
                flows: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?;
        let mut projects = Vec::new();
        for row in rows {
            projects.push(row?);
        }
        tracing::debug!(count = projects.len(), "Projects listed");
        Ok(projects)
    }

    #[instrument(skip(self), fields(project_id = %id))]
    pub fn load_project(&self, id: &str) -> Result<Project, DbError> {
        let conn = self.conn.lock().map_err(|e| DbError::Lock(e.to_string()))?;
        let project = conn.query_row(
            "SELECT id, name, widgets, flows, created_at, updated_at FROM projects WHERE id = ?1",
            params![id],
            |row| {
                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    widgets: row.get(2)?,
                    flows: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            },
        )?;
        Ok(project)
    }

    #[instrument(skip(self), fields(project_id = %id))]
    pub fn save_project(&self, id: &str, widgets: &str, flows: &str) -> Result<(), DbError> {
        let conn = self.conn.lock().map_err(|e| DbError::Lock(e.to_string()))?;
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE projects SET widgets = ?1, flows = ?2, updated_at = ?3 WHERE id = ?4",
            params![widgets, flows, now, id],
        )?;
        tracing::info!("Project saved");
        Ok(())
    }

    pub fn upsert_project(&self, project: &Project) -> Result<(), DbError> {
        let conn = self.conn.lock().map_err(|e| DbError::Lock(e.to_string()))?;
        conn.execute(
            "INSERT INTO projects (id, name, widgets, flows, created_at, updated_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(id) DO UPDATE SET 
                name = excluded.name,
                widgets = excluded.widgets,
                flows = excluded.flows,
                updated_at = excluded.updated_at",
            params![project.id, project.name, project.widgets, project.flows, project.created_at, project.updated_at],
        )?;
        Ok(())
    }

    #[instrument(skip(self), fields(project_id = %id))]
    pub fn delete_project(&self, id: &str) -> Result<(), DbError> {
        let conn = self.conn.lock().map_err(|e| DbError::Lock(e.to_string()))?;
        conn.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
        tracing::info!("Project deleted");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_db() -> Database {
        Database::new(":memory:").expect("Failed to create in-memory DB")
    }

    #[test]
    fn test_create_project() {
        let db = test_db();
        let project = db.create_project("Test Project").unwrap();
        assert_eq!(project.name, "Test Project");
        assert_eq!(project.widgets, "[]");
        assert_eq!(project.flows, "[]");
        assert!(!project.id.is_empty());
        assert!(!project.created_at.is_empty());
    }

    #[test]
    fn test_list_projects_empty() {
        let db = test_db();
        let projects = db.list_projects().unwrap();
        assert!(projects.is_empty());
    }

    #[test]
    fn test_list_projects_multiple() {
        let db = test_db();
        db.create_project("Project A").unwrap();
        db.create_project("Project B").unwrap();
        let projects = db.list_projects().unwrap();
        assert_eq!(projects.len(), 2);
    }

    #[test]
    fn test_list_projects_ordered_by_updated_at_desc() {
        let db = test_db();
        db.create_project("First").unwrap();
        db.create_project("Second").unwrap();
        let projects = db.list_projects().unwrap();
        // Second was created after First, should appear first
        assert_eq!(projects[0].name, "Second");
        assert_eq!(projects[1].name, "First");
    }

    #[test]
    fn test_load_project() {
        let db = test_db();
        let created = db.create_project("Load Test").unwrap();
        let loaded = db.load_project(&created.id).unwrap();
        assert_eq!(loaded.id, created.id);
        assert_eq!(loaded.name, "Load Test");
    }

    #[test]
    fn test_load_project_not_found() {
        let db = test_db();
        let result = db.load_project("nonexistent-id");
        assert!(result.is_err());
    }

    #[test]
    fn test_save_project() {
        let db = test_db();
        let project = db.create_project("Save Test").unwrap();
        db.save_project(&project.id, r#"{"custom": true}"#, "[]").unwrap();
        let loaded = db.load_project(&project.id).unwrap();
        assert_eq!(loaded.widgets, r#"{"custom": true}"#);
        assert_ne!(loaded.updated_at, project.updated_at);
    }

    #[test]
    fn test_delete_project() {
        let db = test_db();
        let project = db.create_project("Delete Me").unwrap();
        db.delete_project(&project.id).unwrap();
        let projects = db.list_projects().unwrap();
        assert!(projects.is_empty());
    }

    #[test]
    fn test_create_project_empty_name() {
        let db = test_db();
        let project = db.create_project("").unwrap();
        assert_eq!(project.name, "");
    }

    #[test]
    fn test_save_project_updates_widgets_and_flows() {
        let db = test_db();
        let project = db.create_project("Full Update").unwrap();
        db.save_project(&project.id, "widgets_data", "flows_data").unwrap();
        let loaded = db.load_project(&project.id).unwrap();
        assert_eq!(loaded.widgets, "widgets_data");
        assert_eq!(loaded.flows, "flows_data");
    }

    #[test]
    fn test_delete_nonexistent_project() {
        let db = test_db();
        let result = db.delete_project("no-such-id");
        assert!(result.is_ok());
    }

    #[test]
    fn test_upsert_project_insert_new() {
        let db = test_db();
        let project = Project {
            id: "upsert-test-1".to_string(),
            name: "Upsert New".to_string(),
            widgets: r#"{"key":"value"}"#.to_string(),
            flows: "[]".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };
        let _ = db.delete_project("upsert-test-1");
        db.upsert_project(&project).unwrap();
        let loaded = db.load_project("upsert-test-1").unwrap();
        assert_eq!(loaded.name, "Upsert New");
        assert_eq!(loaded.widgets, r#"{"key":"value"}"#);
    }

    #[test]
    fn test_upsert_project_update_existing() {
        let db = test_db();
        let original = db.create_project("Update Me").unwrap();

        let updated = Project {
            id: original.id.clone(),
            name: "Updated Name".to_string(),
            widgets: "updated".to_string(),
            flows: "flows".to_string(),
            created_at: original.created_at.clone(),
            updated_at: "9999-12-31T23:59:59Z".to_string(),
        };
        db.upsert_project(&updated).unwrap();
        let loaded = db.load_project(&original.id).unwrap();
        assert_eq!(loaded.name, "Updated Name");
        assert_eq!(loaded.widgets, "updated");
        assert_eq!(loaded.created_at, original.created_at);
    }

    #[test]
    fn test_save_project_nonexistent_id() {
        let db = test_db();
        db.save_project("no-such-id", "{}", "[]").unwrap();
        let loaded = db.load_project("no-such-id");
        assert!(loaded.is_err());
    }
}
