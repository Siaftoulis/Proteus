//! Comprehensive Audit Trail & Event-Sourcing Engine for Proteus Ecosystem.
//! Tracks every movement, state change, and operational action across all subsystems.

use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemEvent {
    pub event_id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub event_type: String,
    pub operator_name: String,
    pub operator_role: String,
    pub description: String,
    pub payload_json: String,
    pub created_at: i64,
}

impl SystemEvent {
    pub fn new(
        entity_type: impl Into<String>,
        entity_id: impl Into<String>,
        event_type: impl Into<String>,
        operator_name: impl Into<String>,
        operator_role: impl Into<String>,
        description: impl Into<String>,
        payload_json: impl Into<String>,
    ) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        Self {
            event_id: uuid::Uuid::now_v7().to_string(),
            entity_type: entity_type.into(),
            entity_id: entity_id.into(),
            event_type: event_type.into(),
            operator_name: operator_name.into(),
            operator_role: operator_role.into(),
            description: description.into(),
            payload_json: payload_json.into(),
            created_at: now,
        }
    }
}

/// Initializes the audit log table and indexes in SQLite.
pub fn init_audit_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS audit_logs (
            event_id TEXT PRIMARY KEY,
            entity_type TEXT NOT NULL,
            entity_id TEXT NOT NULL,
            event_type TEXT NOT NULL,
            operator_name TEXT NOT NULL,
            operator_role TEXT NOT NULL,
            description TEXT NOT NULL,
            payload_json TEXT NOT NULL,
            created_at INTEGER NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_audit_created ON audit_logs(created_at DESC);
        CREATE INDEX IF NOT EXISTS idx_audit_role ON audit_logs(operator_role);
        CREATE INDEX IF NOT EXISTS idx_audit_entity ON audit_logs(entity_type, entity_id);
        "#,
    )?;
    Ok(())
}

/// Records a system movement event into the audit trail.
pub fn log_audit_event(conn: &Connection, event: &SystemEvent) -> Result<()> {
    conn.execute(
        r#"
        INSERT INTO audit_logs (
            event_id, entity_type, entity_id, event_type,
            operator_name, operator_role, description, payload_json, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        "#,
        params![
            event.event_id,
            event.entity_type,
            event.entity_id,
            event.event_type,
            event.operator_name,
            event.operator_role,
            event.description,
            event.payload_json,
            event.created_at,
        ],
    )?;
    Ok(())
}

/// Lists recent audit events with optional role filtering.
pub fn list_audit_events(
    conn: &Connection,
    limit: usize,
    offset: usize,
    role_filter: Option<&str>,
) -> Result<Vec<SystemEvent>> {
    let mut sql = "SELECT event_id, entity_type, entity_id, event_type, operator_name, operator_role, description, payload_json, created_at FROM audit_logs".to_string();

    let events = if let Some(role) = role_filter {
        sql.push_str(" WHERE operator_role = ?1 ORDER BY created_at DESC, rowid DESC LIMIT ?2 OFFSET ?3");
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params![role, limit as i64, offset as i64], |row| {
            Ok(SystemEvent {
                event_id: row.get(0)?,
                entity_type: row.get(1)?,
                entity_id: row.get(2)?,
                event_type: row.get(3)?,
                operator_name: row.get(4)?,
                operator_role: row.get(5)?,
                description: row.get(6)?,
                payload_json: row.get(7)?,
                created_at: row.get(8)?,
            })
        })?;
        rows.collect::<Result<Vec<SystemEvent>>>()?
    } else {
        sql.push_str(" ORDER BY created_at DESC, rowid DESC LIMIT ?1 OFFSET ?2");
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params![limit as i64, offset as i64], |row| {
            Ok(SystemEvent {
                event_id: row.get(0)?,
                entity_type: row.get(1)?,
                entity_id: row.get(2)?,
                event_type: row.get(3)?,
                operator_name: row.get(4)?,
                operator_role: row.get(5)?,
                description: row.get(6)?,
                payload_json: row.get(7)?,
                created_at: row.get(8)?,
            })
        })?;
        rows.collect::<Result<Vec<SystemEvent>>>()?
    };

    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_and_list_audit_events() {
        let conn = Connection::open_in_memory().unwrap();
        init_audit_schema(&conn).unwrap();

        let evt1 = SystemEvent::new(
            "TICKET",
            "TCK-1001",
            "CREATED",
            "Μαρία (Reception)",
            "CustomerService",
            "Νέα παραλαβή συσκευής iPhone 14 Pro Max",
            r#"{"customer":"Γιώργος Αλεξίου"}"#,
        );
        log_audit_event(&conn, &evt1).unwrap();

        let mut evt2 = SystemEvent::new(
            "TICKET",
            "TCK-1001",
            "STATUS_CHANGED",
            "Νίκος (Τεχνικός)",
            "Technician",
            "Αλλαγή κατάστασης: Σε Εξέλιξη",
            r#"{"from":"Received","to":"InProgress"}"#,
        );
        evt2.created_at = evt1.created_at + 10;
        log_audit_event(&conn, &evt2).unwrap();

        let all = list_audit_events(&conn, 10, 0, None).unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].event_type, "STATUS_CHANGED");
        assert_eq!(all[1].event_type, "CREATED");

        // Filter by role
        let tech_only = list_audit_events(&conn, 10, 0, Some("Technician")).unwrap();
        assert_eq!(tech_only.len(), 1);
        assert_eq!(tech_only[0].operator_name, "Νίκος (Τεχνικός)");

        let cs_only = list_audit_events(&conn, 10, 0, Some("CustomerService")).unwrap();
        assert_eq!(cs_only.len(), 1);
        assert_eq!(cs_only[0].operator_name, "Μαρία (Reception)");
    }
}
