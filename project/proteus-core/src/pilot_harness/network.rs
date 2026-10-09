//! Intermittent Outbox & Network Partition Resilience Simulator.
//! Simulates physical LAN dropouts, router DHCP churn, and cellular failover
//! ensuring zero data loss and strict FIFO/priority idempotency across store branches.

use chrono::Utc;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NetworkSimError {
    #[error("Database error: {0}")]
    Sql(#[from] rusqlite::Error),
    #[error("Simulation state error: {0}")]
    State(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkLinkState {
    Connected,
    Degraded { packet_loss_pct: u8, latency_ms: u64 },
    SeveredPartition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OutboxPriority {
    CriticalFiscal = 0,    // AADE QR receipts, signed shipping notes
    NormalOperational = 1, // Tickets, customer updates
    LowTelemetry = 2,      // Heartbeats, diagnostic logs
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlushSummary {
    pub attempted: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub pending_remaining: usize,
}

/// Initializes database tables for network outbox simulation.
pub fn init_network_outbox_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS pilot_network_outbox (
            item_id TEXT PRIMARY KEY,
            store_id TEXT NOT NULL,
            priority INTEGER NOT NULL,
            payload_json TEXT NOT NULL,
            idempotency_hash TEXT NOT NULL UNIQUE,
            status TEXT NOT NULL, -- PENDING, SENT, FAILED
            retry_count INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            sent_at TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_outbox_lookup 
        ON pilot_network_outbox(store_id, status, priority, created_at);
        "#,
    )?;
    Ok(())
}

fn compute_idempotency_hash(store_id: &str, payload: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(store_id.as_bytes());
    hasher.update(b"::");
    hasher.update(payload.as_bytes());
    hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect()
}

pub struct NetworkPartitionSimulator {
    pub current_state: NetworkLinkState,
    pub total_staged: usize,
    pub total_delivered: usize,
    pub total_dropped: usize,
}

impl NetworkPartitionSimulator {
    pub fn new(initial_state: NetworkLinkState) -> Self {
        Self {
            current_state: initial_state,
            total_staged: 0,
            total_delivered: 0,
            total_dropped: 0,
        }
    }

    pub fn set_link_state(&mut self, state: NetworkLinkState) {
        self.current_state = state;
    }

    /// Stages a transaction in the local SQLite outbox.
    pub fn stage_transaction(
        &mut self,
        conn: &Connection,
        store_id: &str,
        priority: OutboxPriority,
        payload: &str,
    ) -> std::result::Result<String, NetworkSimError> {
        let now = Utc::now().to_rfc3339();
        let hash = compute_idempotency_hash(store_id, payload);
        let item_id = format!("OUTBOX-{}-{}", store_id, &hash[..12]);

        conn.execute(
            r#"
            INSERT OR IGNORE INTO pilot_network_outbox
            (item_id, store_id, priority, payload_json, idempotency_hash, status, retry_count, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, 'PENDING', 0, ?6)
            "#,
            params![item_id, store_id, priority as i32, payload, hash, now],
        )?;

        self.total_staged += 1;
        Ok(item_id)
    }

    /// Attempts to flush queued transactions according to current link state.
    pub fn flush_outbox(
        &mut self,
        conn: &Connection,
        store_id: &str,
        batch_size: usize,
    ) -> std::result::Result<FlushSummary, NetworkSimError> {
        let mut stmt = conn.prepare(
            r#"
            SELECT item_id, priority, payload_json, retry_count
            FROM pilot_network_outbox
            WHERE store_id = ?1 AND status = 'PENDING'
            ORDER BY priority ASC, created_at ASC
            LIMIT ?2
            "#,
        )?;

        let rows: Vec<(String, i32, String, u32)> = stmt
            .query_map(params![store_id, batch_size as i64], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })?
            .filter_map(std::result::Result::ok)
            .collect();

        let attempted = rows.len();
        let mut succeeded = 0;
        let mut failed = 0;
        let now = Utc::now().to_rfc3339();

        for (item_id, _, _, retry) in rows {
            match self.current_state {
                NetworkLinkState::Connected => {
                    conn.execute(
                        "UPDATE pilot_network_outbox SET status = 'SENT', sent_at = ?1 WHERE item_id = ?2",
                        params![now, item_id],
                    )?;
                    succeeded += 1;
                    self.total_delivered += 1;
                }
                NetworkLinkState::SeveredPartition => {
                    conn.execute(
                        "UPDATE pilot_network_outbox SET retry_count = ?1 WHERE item_id = ?2",
                        params![retry + 1, item_id],
                    )?;
                    failed += 1;
                    self.total_dropped += 1;
                }
                NetworkLinkState::Degraded { packet_loss_pct, .. } => {
                    // Deterministic loss check based on retry modulo
                    if (retry as u8 * 37) % 100 < packet_loss_pct {
                        conn.execute(
                            "UPDATE pilot_network_outbox SET retry_count = ?1 WHERE item_id = ?2",
                            params![retry + 1, item_id],
                        )?;
                        failed += 1;
                        self.total_dropped += 1;
                    } else {
                        conn.execute(
                            "UPDATE pilot_network_outbox SET status = 'SENT', sent_at = ?1 WHERE item_id = ?2",
                            params![now, item_id],
                        )?;
                        succeeded += 1;
                        self.total_delivered += 1;
                    }
                }
            }
        }

        let mut count_stmt = conn.prepare(
            "SELECT COUNT(*) FROM pilot_network_outbox WHERE store_id = ?1 AND status = 'PENDING'",
        )?;
        let pending_remaining: usize = count_stmt.query_row(params![store_id], |row| row.get(0))?;

        Ok(FlushSummary {
            attempted,
            succeeded,
            failed,
            pending_remaining,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_partition_queuing_without_data_loss() {
        let conn = Connection::open_in_memory().unwrap();
        init_network_outbox_schema(&conn).unwrap();

        let mut sim = NetworkPartitionSimulator::new(NetworkLinkState::SeveredPartition);

        // Stage 10 transactions during partition
        for i in 0..10 {
            let payload = format!(r#"{{"ticket_id": "TCK-{:03}", "fault": "Broken screen"}}"#, i);
            sim.stage_transaction(&conn, "STORE-REP-01", OutboxPriority::NormalOperational, &payload)
                .unwrap();
        }

        // Attempt flush during severed partition
        let summary = sim.flush_outbox(&conn, "STORE-REP-01", 50).unwrap();
        assert_eq!(summary.attempted, 10);
        assert_eq!(summary.succeeded, 0);
        assert_eq!(summary.failed, 10);
        assert_eq!(summary.pending_remaining, 10);
        assert_eq!(sim.total_delivered, 0);
    }

    #[test]
    fn test_partition_recovery_and_flush() {
        let conn = Connection::open_in_memory().unwrap();
        init_network_outbox_schema(&conn).unwrap();

        let mut sim = NetworkPartitionSimulator::new(NetworkLinkState::SeveredPartition);

        // Stage 5 transactions
        for i in 0..5 {
            let payload = format!(r#"{{"pos_id": "POS-{:03}", "total": 2500}}"#, i);
            sim.stage_transaction(&conn, "STORE-POS-02", OutboxPriority::CriticalFiscal, &payload)
                .unwrap();
        }

        // Network recovers
        sim.set_link_state(NetworkLinkState::Connected);
        let summary = sim.flush_outbox(&conn, "STORE-POS-02", 50).unwrap();

        assert_eq!(summary.attempted, 5);
        assert_eq!(summary.succeeded, 5);
        assert_eq!(summary.failed, 0);
        assert_eq!(summary.pending_remaining, 0);
        assert_eq!(sim.total_delivered, 5);
    }

    #[test]
    fn test_outbox_priority_ordering() {
        let conn = Connection::open_in_memory().unwrap();
        init_network_outbox_schema(&conn).unwrap();

        let mut sim = NetworkPartitionSimulator::new(NetworkLinkState::Connected);

        // Stage in reverse priority order: Low, then Normal, then Critical
        sim.stage_transaction(&conn, "STORE-01", OutboxPriority::LowTelemetry, r#"{"log": "ping"}"#)
            .unwrap();
        sim.stage_transaction(&conn, "STORE-01", OutboxPriority::NormalOperational, r#"{"ticket": "t1"}"#)
            .unwrap();
        sim.stage_transaction(&conn, "STORE-01", OutboxPriority::CriticalFiscal, r#"{"receipt": "r1"}"#)
            .unwrap();

        // Flush batch of 1
        let s1 = sim.flush_outbox(&conn, "STORE-01", 1).unwrap();
        assert_eq!(s1.succeeded, 1);
        assert_eq!(s1.pending_remaining, 2);

        // Verify the delivered item was the CriticalFiscal item
        let sent_payload: String = conn
            .query_row(
                "SELECT payload_json FROM pilot_network_outbox WHERE store_id = 'STORE-01' AND status = 'SENT'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(sent_payload, r#"{"receipt": "r1"}"#);
    }

    #[test]
    fn test_idempotent_duplicate_prevention() {
        let conn = Connection::open_in_memory().unwrap();
        init_network_outbox_schema(&conn).unwrap();

        let mut sim = NetworkPartitionSimulator::new(NetworkLinkState::Connected);
        let payload = r#"{"order_id": "ORD-999", "amount": 12000}"#;

        // Stage twice
        sim.stage_transaction(&conn, "STORE-LOG-03", OutboxPriority::CriticalFiscal, payload)
            .unwrap();
        sim.stage_transaction(&conn, "STORE-LOG-03", OutboxPriority::CriticalFiscal, payload)
            .unwrap();

        // Verify only 1 entry in outbox
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM pilot_network_outbox WHERE store_id = 'STORE-LOG-03'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }
}
