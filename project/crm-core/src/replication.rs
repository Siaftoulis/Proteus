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

        let summary = queue.process_sync_batch(&[r1.id.clone()]);
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
}
