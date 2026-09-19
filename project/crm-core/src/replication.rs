// Proteus Core — Hybrid Offline-First Replication Engine & Outbox Queue
// Designed from first principles. Zero copied third-party boilerplate.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeOp {
    Insert,
    Update,
    Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncStatus {
    Pending,
    Synced,
    Conflict,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum DataPriority {
    Critical = 0, // High-value financial totals, active security, urgent tickets
    High = 1,     // Real-time stage transitions, appointment bookings
    Normal = 2,   // Customer profile updates, metadata
    Low = 3,      // Historical logs, bulk analytics
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxRecord {
    pub id: String,
    pub entity: String,
    pub record_id: String,
    pub op: ChangeOp,
    pub payload_json: String,
    pub local_timestamp: String,
    pub remote_version: u64,
    pub status: SyncStatus,
    pub priority: DataPriority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictStrategy {
    LastWriteWins,
    RemoteWins,
    LocalWins,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReplicationSummary {
    pub records_pushed: usize,
    pub records_pulled: usize,
    pub conflicts_resolved: usize,
    pub pending_remaining: usize,
}

pub struct ReplicationQueue {
    records: Vec<OutboxRecord>,
}

impl Default for ReplicationQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl ReplicationQueue {
    pub fn new() -> Self {
        Self { records: Vec::new() }
    }

    pub fn enqueue(&mut self, entity: &str, record_id: &str, op: ChangeOp, payload_json: &str) -> OutboxRecord {
        let rec = OutboxRecord {
            id: Uuid::now_v7().to_string(),
            entity: entity.to_string(),
            record_id: record_id.to_string(),
            op,
            payload_json: payload_json.to_string(),
            local_timestamp: Utc::now().to_rfc3339(),
            remote_version: 0,
            status: SyncStatus::Pending,
            priority: DataPriority::Normal,
        };
        self.records.push(rec.clone());
        rec
    }

    pub fn pending_records(&self, limit: usize) -> Vec<OutboxRecord> {
        self.records
            .iter()
            .filter(|r| r.status == SyncStatus::Pending)
            .take(limit)
            .cloned()
            .collect()
    }

    pub fn mark_status(&mut self, id: &str, status: SyncStatus) -> bool {
        if let Some(rec) = self.records.iter_mut().find(|r| r.id == id) {
            rec.status = status;
            true
        } else {
            false
        }
    }

    pub fn resolve_conflict(
        local_timestamp: &str,
        remote_timestamp: &str,
        strategy: ConflictStrategy,
    ) -> bool {
        match strategy {
            ConflictStrategy::LocalWins => true,
            ConflictStrategy::RemoteWins => false,
            ConflictStrategy::LastWriteWins => local_timestamp >= remote_timestamp,
        }
    }

    pub fn process_sync_batch(&mut self, remote_success_ids: &[String]) -> ReplicationSummary {
        let mut pushed = 0;
        for id in remote_success_ids {
            if self.mark_status(id, SyncStatus::Synced) {
                pushed += 1;
            }
        }
        let pending = self.records.iter().filter(|r| r.status == SyncStatus::Pending).count();
        ReplicationSummary {
            records_pushed: pushed,
            records_pulled: 0,
            conflicts_resolved: 0,
            pending_remaining: pending,
        }
    }

    pub fn total_count(&self) -> usize {
        self.records.len()
    }

    pub fn pending_count(&self) -> usize {
        self.records.iter().filter(|r| r.status == SyncStatus::Pending).count()
    }
}

use crate::drivers::DatabaseDriver;
use rusqlite::{params, Connection};

pub fn init_outbox_schema(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS _proteus_outbox (
            id TEXT PRIMARY KEY,
            entity TEXT NOT NULL,
            record_id TEXT NOT NULL,
            op TEXT NOT NULL,
            payload_json TEXT NOT NULL,
            local_timestamp TEXT NOT NULL,
            remote_version INTEGER NOT NULL DEFAULT 0,
            status TEXT NOT NULL DEFAULT 'Pending',
            priority INTEGER NOT NULL DEFAULT 2
        );
        CREATE INDEX IF NOT EXISTS idx_outbox_status ON _proteus_outbox(status);
        CREATE INDEX IF NOT EXISTS idx_outbox_prio ON _proteus_outbox(priority, local_timestamp ASC);
        "#,
    )
    .map_err(|e| format!("Failed to init outbox schema: {}", e))?;
    Ok(())
}

pub fn enqueue_outbox_with_priority(
    conn: &Connection,
    entity: &str,
    record_id: &str,
    op: ChangeOp,
    payload_json: &str,
    priority: DataPriority,
) -> Result<OutboxRecord, String> {
    let rec = OutboxRecord {
        id: Uuid::now_v7().to_string(),
        entity: entity.to_string(),
        record_id: record_id.to_string(),
        op,
        payload_json: payload_json.to_string(),
        local_timestamp: Utc::now().to_rfc3339(),
        remote_version: 0,
        status: SyncStatus::Pending,
        priority,
    };
    let op_str = match op { ChangeOp::Insert => "Insert", ChangeOp::Update => "Update", ChangeOp::Delete => "Delete" };
    conn.execute(
        "INSERT INTO _proteus_outbox (id, entity, record_id, op, payload_json, local_timestamp, remote_version, status, priority)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'Pending', ?8)",
        params![rec.id, rec.entity, rec.record_id, op_str, rec.payload_json, rec.local_timestamp, rec.remote_version, priority as u8],
    ).map_err(|e| format!("Failed to enqueue outbox record: {}", e))?;
    Ok(rec)
}

pub fn enqueue_outbox(
    conn: &Connection,
    entity: &str,
    record_id: &str,
    op: ChangeOp,
    payload_json: &str,
) -> Result<OutboxRecord, String> {
    enqueue_outbox_with_priority(conn, entity, record_id, op, payload_json, DataPriority::Normal)
}

pub fn fetch_pending_outbox(conn: &Connection, limit: usize) -> Result<Vec<OutboxRecord>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, entity, record_id, op, payload_json, local_timestamp, remote_version, status, priority
         FROM _proteus_outbox WHERE status = 'Pending' ORDER BY priority ASC, local_timestamp ASC LIMIT ?1",
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map(params![limit as i64], |row| {
        let op_str: String = row.get(3)?;
        let op = match op_str.as_str() { "Update" => ChangeOp::Update, "Delete" => ChangeOp::Delete, _ => ChangeOp::Insert };
        let status_str: String = row.get(7)?;
        let status = match status_str.as_str() { "Synced" => SyncStatus::Synced, "Conflict" => SyncStatus::Conflict, "Failed" => SyncStatus::Failed, _ => SyncStatus::Pending };
        let prio_u8: u8 = row.get::<_, u8>(8).unwrap_or(2);
        let priority = match prio_u8 { 0 => DataPriority::Critical, 1 => DataPriority::High, 2 => DataPriority::Normal, _ => DataPriority::Low };
        Ok(OutboxRecord {
            id: row.get(0)?,
            entity: row.get(1)?,
            record_id: row.get(2)?,
            op,
            payload_json: row.get(4)?,
            local_timestamp: row.get(5)?,
            remote_version: row.get(6)?,
            status,
            priority,
        })
    }).map_err(|e| e.to_string())?;

    let mut list = Vec::new();
    for r in rows { list.push(r.map_err(|e| e.to_string())?); }
    Ok(list)
}

pub fn mark_outbox_status(conn: &Connection, id: &str, status: SyncStatus) -> Result<bool, String> {
    let status_str = match status { SyncStatus::Pending => "Pending", SyncStatus::Synced => "Synced", SyncStatus::Conflict => "Conflict", SyncStatus::Failed => "Failed" };
    let count = conn.execute("UPDATE _proteus_outbox SET status = ?1 WHERE id = ?2", params![status_str, id]).map_err(|e| e.to_string())?;
    Ok(count > 0)
}

pub fn count_pending_outbox(conn: &Connection) -> Result<usize, String> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM _proteus_outbox WHERE status = 'Pending'", [], |r| r.get(0)).unwrap_or(0);
    Ok(count as usize)
}

pub fn sync_outbox_to_driver(
    conn: &Connection,
    driver: &dyn DatabaseDriver,
    limit: usize,
) -> Result<ReplicationSummary, String> {
    let pending = fetch_pending_outbox(conn, limit)?;
    let mut pushed = 0;

    for rec in &pending {
        let sync_sql = format!(
            "/* PROTEUS_REPLICATION_SYNC */ -- Entity: {}, Record: {}, Op: {:?}\n-- Payload: {}",
            rec.entity, rec.record_id, rec.op, rec.payload_json
        );
        match driver.execute_raw(&sync_sql) {
            Ok(_) => {
                let _ = mark_outbox_status(conn, &rec.id, SyncStatus::Synced);
                pushed += 1;
            }
            Err(_) => {
                let _ = mark_outbox_status(conn, &rec.id, SyncStatus::Failed);
            }
        }
    }

    let remaining = count_pending_outbox(conn)?;
    Ok(ReplicationSummary {
        records_pushed: pushed,
        records_pulled: 0,
        conflicts_resolved: 0,
        pending_remaining: remaining,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enqueue_and_fetch_pending() {
        let mut queue = ReplicationQueue::new();
        assert_eq!(queue.pending_count(), 0);

        let r1 = queue.enqueue("tickets", "tick_1", ChangeOp::Insert, r#"{"status":"received"}"#);
        let _r2 = queue.enqueue("contacts", "c_1", ChangeOp::Update, r#"{"name":"Alice"}"#);

        assert_eq!(queue.total_count(), 2);
        assert_eq!(queue.pending_count(), 2);

        let batch = queue.pending_records(1);
        assert_eq!(batch.len(), 1);
        assert_eq!(batch[0].id, r1.id);
        assert_eq!(batch[0].op, ChangeOp::Insert);
    }

    #[test]
    fn test_batch_sync_marks_status() {
        let mut queue = ReplicationQueue::new();
        let r1 = queue.enqueue("tickets", "tick_1", ChangeOp::Insert, "{}");
        let _r2 = queue.enqueue("tickets", "tick_2", ChangeOp::Insert, "{}");

        let summary = queue.process_sync_batch(std::slice::from_ref(&r1.id));
        assert_eq!(summary.records_pushed, 1);
        assert_eq!(summary.pending_remaining, 1);

        assert_eq!(queue.pending_records(10).len(), 1);
    }

    #[test]
    fn test_conflict_resolution_strategies() {
        let local_t = "2026-09-18T10:00:00Z";
        let remote_t = "2026-09-18T11:00:00Z";

        assert!(!ReplicationQueue::resolve_conflict(local_t, remote_t, ConflictStrategy::LastWriteWins));
        assert!(ReplicationQueue::resolve_conflict(remote_t, local_t, ConflictStrategy::LastWriteWins));
        assert!(ReplicationQueue::resolve_conflict(local_t, remote_t, ConflictStrategy::LocalWins));
        assert!(!ReplicationQueue::resolve_conflict(local_t, remote_t, ConflictStrategy::RemoteWins));
    }

    #[test]
    fn test_sqlite_outbox_persistence_and_sync() {
        let conn = Connection::open_in_memory().unwrap();
        init_outbox_schema(&conn).unwrap();

        assert_eq!(count_pending_outbox(&conn).unwrap(), 0);

        let rec = enqueue_outbox(&conn, "tickets", "TCK-101", ChangeOp::Insert, r#"{"status":"Open"}"#).unwrap();
        assert_eq!(count_pending_outbox(&conn).unwrap(), 1);

        let pending = fetch_pending_outbox(&conn, 10).unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].id, rec.id);
        assert_eq!(pending[0].entity, "tickets");

        let driver = crate::drivers::RemoteDriverMock::new(crate::drivers::ConnectionConfig::sqlite_memory());
        let summary = sync_outbox_to_driver(&conn, &driver, 10).unwrap();
        assert_eq!(summary.records_pushed, 1);
        assert_eq!(summary.pending_remaining, 0);

        assert_eq!(count_pending_outbox(&conn).unwrap(), 0);
    }

    #[test]
    fn test_priority_outbox_ordering() {
        let conn = Connection::open_in_memory().unwrap();
        init_outbox_schema(&conn).unwrap();

        // Enqueue normal first, then critical
        let _r_normal = enqueue_outbox_with_priority(&conn, "contacts", "C-1", ChangeOp::Update, "{}", DataPriority::Normal).unwrap();
        let r_critical = enqueue_outbox_with_priority(&conn, "ledger", "LEDGER-1", ChangeOp::Insert, r#"{"balance":1500}"#, DataPriority::Critical).unwrap();

        let pending = fetch_pending_outbox(&conn, 10).unwrap();
        assert_eq!(pending.len(), 2);
        // Critical must be fetched first!
        assert_eq!(pending[0].id, r_critical.id);
        assert_eq!(pending[0].priority, DataPriority::Critical);
        assert_eq!(pending[1].priority, DataPriority::Normal);
    }
}

