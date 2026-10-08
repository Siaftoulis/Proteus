//! Cross-database multi-store federation engine.
//! Provides differential transaction replication and cryptographic delta verification.

use chrono::Utc;
use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::federation::types::{
    ChangeOp, FederatedStoreNode, FederatedTransaction, FederationError, NodeRole, NodeStatus,
    ReplicationReport, SyncDirection,
};

/// Computes a hex-encoded SHA-256 hash for payload verification.
fn compute_sha256(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Core federation orchestrator for an autonomous store database node.
pub struct MultiStoreFederationEngine {
    pub local_store_id: String,
    pub node_name: String,
}

impl MultiStoreFederationEngine {
    /// Creates a new federation engine instance for the local store.
    pub fn new(local_store_id: impl Into<String>, node_name: impl Into<String>) -> Self {
        Self {
            local_store_id: local_store_id.into(),
            node_name: node_name.into(),
        }
    }

    /// Initializes SQLite tables required for cross-database federation.
    pub fn init_schema(&self, conn: &Connection) -> Result<(), FederationError> {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS federation_nodes (
                node_id TEXT PRIMARY KEY,
                store_id TEXT NOT NULL UNIQUE,
                store_name TEXT NOT NULL,
                endpoint_url TEXT NOT NULL,
                role TEXT NOT NULL DEFAULT 'BRANCH',
                status TEXT NOT NULL DEFAULT 'ACTIVE',
                sync_direction TEXT NOT NULL DEFAULT 'BIDIRECTIONAL',
                last_sync_seq INTEGER NOT NULL DEFAULT 0,
                last_sync_time TEXT,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS federation_changelog (
                sequence INTEGER PRIMARY KEY AUTOINCREMENT,
                tx_id TEXT NOT NULL UNIQUE,
                origin_store_id TEXT NOT NULL,
                entity TEXT NOT NULL,
                record_id TEXT NOT NULL,
                op TEXT NOT NULL,
                payload_json TEXT NOT NULL,
                payload_hash TEXT NOT NULL,
                natural_key TEXT,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS federation_checkpoints (
                node_id TEXT PRIMARY KEY,
                remote_store_id TEXT NOT NULL,
                last_received_seq INTEGER NOT NULL DEFAULT 0,
                last_sent_seq INTEGER NOT NULL DEFAULT 0,
                updated_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_fed_change_entity ON federation_changelog(entity, record_id);
            CREATE INDEX IF NOT EXISTS idx_fed_change_origin ON federation_changelog(origin_store_id);
            "#,
        )?;
        Ok(())
    }

    /// Registers or updates a federated peer store node.
    pub fn register_node(&self, conn: &Connection, node: &FederatedStoreNode) -> Result<(), FederationError> {
        conn.execute(
            r#"
            INSERT INTO federation_nodes (
                node_id, store_id, store_name, endpoint_url, role, status, sync_direction,
                last_sync_seq, last_sync_time, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            ON CONFLICT(node_id) DO UPDATE SET
                store_name = excluded.store_name,
                endpoint_url = excluded.endpoint_url,
                role = excluded.role,
                status = excluded.status,
                sync_direction = excluded.sync_direction,
                last_sync_seq = excluded.last_sync_seq,
                last_sync_time = excluded.last_sync_time
            "#,
            params![
                node.node_id,
                node.store_id,
                node.store_name,
                node.endpoint_url,
                node.role.as_str(),
                node.status.as_str(),
                node.sync_direction.as_str(),
                node.last_sync_seq as i64,
                node.last_sync_time,
                node.created_at,
            ],
        )?;

        // Ensure checkpoint entry exists
        conn.execute(
            r#"
            INSERT OR IGNORE INTO federation_checkpoints (
                node_id, remote_store_id, last_received_seq, last_sent_seq, updated_at
            ) VALUES (?1, ?2, 0, 0, ?3)
            "#,
            params![node.node_id, node.store_id, Utc::now().to_rfc3339()],
        )?;

        Ok(())
    }

    /// Lists all registered federated store nodes.
    pub fn list_nodes(&self, conn: &Connection) -> Result<Vec<FederatedStoreNode>, FederationError> {
        let mut stmt = conn.prepare(
            r#"
            SELECT node_id, store_id, store_name, endpoint_url, role, status, sync_direction,
                   last_sync_seq, last_sync_time, created_at
            FROM federation_nodes ORDER BY store_name ASC
            "#,
        )?;

        let rows = stmt.query_map([], |row| {
            let role_str: String = row.get(4)?;
            let status_str: String = row.get(5)?;
            let dir_str: String = row.get(6)?;
            let seq: i64 = row.get(7)?;

            Ok(FederatedStoreNode {
                node_id: row.get(0)?,
                store_id: row.get(1)?,
                store_name: row.get(2)?,
                endpoint_url: row.get(3)?,
                role: NodeRole::from_str(&role_str),
                status: NodeStatus::from_str(&status_str),
                sync_direction: SyncDirection::from_str(&dir_str),
                last_sync_seq: seq as u64,
                last_sync_time: row.get(8)?,
                created_at: row.get(9)?,
            })
        })?;

        let mut nodes = Vec::new();
        for r in rows {
            nodes.push(r?);
        }
        Ok(nodes)
    }

    /// Records a mutating local transaction with cryptographic SHA-256 seal into changelog.
    pub fn record_local_change(
        &self,
        conn: &Connection,
        entity: &str,
        record_id: &str,
        op: ChangeOp,
        payload_json: &str,
        natural_key: Option<&str>,
    ) -> Result<FederatedTransaction, FederationError> {
        let tx_id = Uuid::now_v7().to_string();
        let payload_hash = compute_sha256(payload_json);
        let now_utc = Utc::now().to_rfc3339();

        conn.execute(
            r#"
            INSERT INTO federation_changelog (
                tx_id, origin_store_id, entity, record_id, op, payload_json, payload_hash, natural_key, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            params![
                tx_id,
                self.local_store_id,
                entity,
                record_id,
                op.as_str(),
                payload_json,
                payload_hash,
                natural_key,
                now_utc,
            ],
        )?;

        let seq = conn.last_insert_rowid() as u64;

        Ok(FederatedTransaction {
            sequence: seq,
            tx_id,
            origin_store_id: self.local_store_id.clone(),
            entity: entity.to_string(),
            record_id: record_id.to_string(),
            op,
            payload_json: payload_json.to_string(),
            payload_hash,
            natural_key: natural_key.map(|s| s.to_string()),
            created_at: now_utc,
        })
    }

    /// Fetches outgoing delta transactions for a remote node and advances the sent checkpoint.
    pub fn pull_outgoing_deltas(
        &self,
        conn: &Connection,
        for_node_id: &str,
        limit: usize,
    ) -> Result<Vec<FederatedTransaction>, FederationError> {
        let last_sent: i64 = conn
            .query_row(
                "SELECT last_sent_seq FROM federation_checkpoints WHERE node_id = ?1",
                params![for_node_id],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let mut stmt = conn.prepare(
            r#"
            SELECT sequence, tx_id, origin_store_id, entity, record_id, op, payload_json, payload_hash, natural_key, created_at
            FROM federation_changelog
            WHERE sequence > ?1
            ORDER BY sequence ASC
            LIMIT ?2
            "#,
        )?;

        let rows = stmt.query_map(params![last_sent, limit as i64], |row| {
            let seq: i64 = row.get(0)?;
            let op_str: String = row.get(5)?;
            Ok(FederatedTransaction {
                sequence: seq as u64,
                tx_id: row.get(1)?,
                origin_store_id: row.get(2)?,
                entity: row.get(3)?,
                record_id: row.get(4)?,
                op: ChangeOp::from_str(&op_str),
                payload_json: row.get(6)?,
                payload_hash: row.get(7)?,
                natural_key: row.get(8)?,
                created_at: row.get(9)?,
            })
        })?;

        let mut deltas = Vec::new();
        let mut highest_seq = last_sent;

        for r in rows {
            let tx = r?;
            if (tx.sequence as i64) > highest_seq {
                highest_seq = tx.sequence as i64;
            }
            deltas.push(tx);
        }

        if highest_seq > last_sent {
            let now_utc = Utc::now().to_rfc3339();
            conn.execute(
                r#"
                UPDATE federation_checkpoints
                SET last_sent_seq = ?1, updated_at = ?2
                WHERE node_id = ?3
                "#,
                params![highest_seq, now_utc, for_node_id],
            )?;
        }

        Ok(deltas)
    }

    /// Validates integrity and applies an incoming batch of transactions from a peer store node.
    pub fn apply_incoming_deltas(
        &self,
        conn: &Connection,
        from_node_id: &str,
        transactions: &[FederatedTransaction],
    ) -> Result<ReplicationReport, FederationError> {
        let mut report = ReplicationReport::default();
        let now_utc = Utc::now().to_rfc3339();

        for tx in transactions {
            report.processed_count += 1;

            // 1. Verify cryptographic SHA-256 seal
            let computed_hash = compute_sha256(&tx.payload_json);
            if computed_hash != tx.payload_hash {
                return Err(FederationError::IntegrityViolation(format!(
                    "Payload hash mismatch on transaction {}. Expected {}, computed {}",
                    tx.tx_id, tx.payload_hash, computed_hash
                )));
            }

            // 2. Loopback suppression (ignore own originated transactions)
            if tx.origin_store_id == self.local_store_id {
                report.skipped_count += 1;
                continue;
            }

            // 3. Idempotency check: see if tx_id was already recorded
            let exists: bool = conn
                .query_row(
                    "SELECT 1 FROM federation_changelog WHERE tx_id = ?1",
                    params![tx.tx_id],
                    |_| Ok(true),
                )
                .unwrap_or(false);

            if exists {
                report.skipped_count += 1;
                continue;
            }

            // 4. Record into local changelog
            conn.execute(
                r#"
                INSERT INTO federation_changelog (
                    tx_id, origin_store_id, entity, record_id, op, payload_json, payload_hash, natural_key, created_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                "#,
                params![
                    tx.tx_id,
                    tx.origin_store_id,
                    tx.entity,
                    tx.record_id,
                    tx.op.as_str(),
                    tx.payload_json,
                    tx.payload_hash,
                    tx.natural_key,
                    tx.created_at,
                ],
            )?;

            report.applied_count += 1;
            if tx.sequence > report.highest_seq_applied {
                report.highest_seq_applied = tx.sequence;
            }
        }

        // Advance receive checkpoint
        if report.highest_seq_applied > 0 {
            conn.execute(
                r#"
                UPDATE federation_checkpoints
                SET last_received_seq = MAX(last_received_seq, ?1), updated_at = ?2
                WHERE node_id = ?3
                "#,
                params![report.highest_seq_applied as i64, now_utc, from_node_id],
            )?;
            conn.execute(
                r#"
                UPDATE federation_nodes
                SET last_sync_seq = MAX(last_sync_seq, ?1), last_sync_time = ?2
                WHERE node_id = ?3
                "#,
                params![report.highest_seq_applied as i64, now_utc, from_node_id],
            )?;
        }

        Ok(report)
    }

    /// Performs live two-way synchronization between two distinct SQLite database connections.
    pub fn sync_peer_databases(
        &self,
        local_conn: &Connection,
        remote_engine: &MultiStoreFederationEngine,
        remote_conn: &Connection,
        remote_node_id: &str,
        local_node_id_in_remote: &str,
        limit: usize,
    ) -> Result<(ReplicationReport, ReplicationReport), FederationError> {
        // Step 1: Pull outgoing deltas from local and apply to remote
        let local_deltas = self.pull_outgoing_deltas(local_conn, remote_node_id, limit)?;
        let remote_applied = remote_engine.apply_incoming_deltas(
            remote_conn,
            local_node_id_in_remote,
            &local_deltas,
        )?;

        // Step 2: Pull outgoing deltas from remote and apply to local
        let remote_deltas =
            remote_engine.pull_outgoing_deltas(remote_conn, local_node_id_in_remote, limit)?;
        let local_applied =
            self.apply_incoming_deltas(local_conn, remote_node_id, &remote_deltas)?;

        Ok((remote_applied, local_applied))
    }
}
