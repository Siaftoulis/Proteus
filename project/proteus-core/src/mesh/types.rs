//! Peer-to-Peer Branch Mesh Discovery Protocol Types & SQLite DDL.
//! Native multi-branch LAN mesh synchronization for Proteus Sovereign BOS.
//! 100% Original Bespoke Implementation. Zero third-party boilerplate.

use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MeshError {
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Database error: {0}")]
    Database(String),
    #[error("Invalid packet format: {0}")]
    InvalidPacket(String),
    #[error("Peer not found: {0}")]
    PeerNotFound(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MeshRole {
    Coordinator,
    PeerNode,
    MobileRelay,
}

impl MeshRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Coordinator => "COORDINATOR",
            Self::PeerNode => "PEER_NODE",
            Self::MobileRelay => "MOBILE_RELAY",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "COORDINATOR" => Self::Coordinator,
            "MOBILE_RELAY" => Self::MobileRelay,
            _ => Self::PeerNode,
        }
    }
}

/// UDP broadcast beacon advertised on the local network segment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BranchBeacon {
    pub protocol_version: u8,
    pub branch_id: String,
    pub node_id: String,
    pub branch_name: String,
    pub listen_port: u16,
    pub sync_epoch: u64,
    pub catalog_checksum: String,
    pub timestamp_utc: i64,
    pub role: MeshRole,
}

impl BranchBeacon {
    pub const PROTOCOL_MAGIC: u8 = 0x50; // 'P' for Proteus

    pub fn new(
        branch_id: impl Into<String>,
        node_id: impl Into<String>,
        branch_name: impl Into<String>,
        listen_port: u16,
        sync_epoch: u64,
        catalog_checksum: impl Into<String>,
        role: MeshRole,
    ) -> Self {
        Self {
            protocol_version: Self::PROTOCOL_MAGIC,
            branch_id: branch_id.into(),
            node_id: node_id.into(),
            branch_name: branch_name.into(),
            listen_port,
            sync_epoch,
            catalog_checksum: catalog_checksum.into(),
            timestamp_utc: Utc::now().timestamp_millis(),
            role,
        }
    }

    pub fn to_packet_bytes(&self) -> Result<Vec<u8>, MeshError> {
        serde_json::to_vec(self).map_err(|e| MeshError::Serialization(e.to_string()))
    }

    pub fn from_packet_bytes(bytes: &[u8]) -> Result<Self, MeshError> {
        let beacon: Self = serde_json::from_slice(bytes)
            .map_err(|e| MeshError::InvalidPacket(e.to_string()))?;
        if beacon.protocol_version != Self::PROTOCOL_MAGIC {
            return Err(MeshError::InvalidPacket("Magic byte mismatch".to_string()));
        }
        Ok(beacon)
    }
}

/// Live peer node tracked in the local branch ledger.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerNode {
    pub node_id: String,
    pub branch_id: String,
    pub branch_name: String,
    pub ip_address: String,
    pub port: u16,
    pub last_seen_utc: i64,
    pub sync_epoch: u64,
    pub catalog_checksum: String,
    pub is_online: bool,
    pub latency_ms: u32,
    pub role: MeshRole,
}

impl PeerNode {
    pub fn is_stale(&self, current_ts: i64, ttl_secs: i64) -> bool {
        let age_ms = current_ts.saturating_sub(self.last_seen_utc);
        age_ms > (ttl_secs * 1000)
    }
}

/// Computes a deterministic SHA-256 fingerprint for catalog verification.
pub fn compute_catalog_checksum(items: &[(&str, f64, &str)]) -> String {
    let mut hasher = Sha256::new();
    for (sku, price, name) in items {
        let line = format!("{}:{:.2}:{}", sku, price, name);
        hasher.update(line.as_bytes());
        hasher.update(b"\n");
    }
    format!("{:x}", hasher.finalize())
}

/// Initializes mesh discovery and vector sync tables in SQLite.
pub fn init_mesh_schema(conn: &Connection) -> Result<(), MeshError> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS mesh_peers (
            node_id TEXT PRIMARY KEY,
            branch_id TEXT NOT NULL,
            branch_name TEXT NOT NULL,
            ip_address TEXT NOT NULL,
            port INTEGER NOT NULL,
            last_seen_utc INTEGER NOT NULL,
            sync_epoch INTEGER NOT NULL,
            catalog_checksum TEXT NOT NULL,
            is_online INTEGER NOT NULL DEFAULT 1,
            latency_ms INTEGER NOT NULL DEFAULT 0,
            role TEXT NOT NULL DEFAULT 'PEER_NODE'
        );

        CREATE TABLE IF NOT EXISTS mesh_sync_epochs (
            branch_id TEXT PRIMARY KEY,
            current_epoch INTEGER NOT NULL DEFAULT 0,
            catalog_checksum TEXT NOT NULL,
            updated_at_utc INTEGER NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_mesh_peers_branch ON mesh_peers(branch_id);
        CREATE INDEX IF NOT EXISTS idx_mesh_peers_seen ON mesh_peers(last_seen_utc);
        "#,
    )
    .map_err(|e| MeshError::Database(e.to_string()))?;
    Ok(())
}

/// Inserts or updates an active peer node in SQLite.
pub fn upsert_peer_node(conn: &Connection, peer: &PeerNode) -> Result<(), MeshError> {
    conn.execute(
        r#"
        INSERT INTO mesh_peers (
            node_id, branch_id, branch_name, ip_address, port,
            last_seen_utc, sync_epoch, catalog_checksum, is_online, latency_ms, role
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
        ON CONFLICT(node_id) DO UPDATE SET
            branch_id = excluded.branch_id,
            branch_name = excluded.branch_name,
            ip_address = excluded.ip_address,
            port = excluded.port,
            last_seen_utc = excluded.last_seen_utc,
            sync_epoch = excluded.sync_epoch,
            catalog_checksum = excluded.catalog_checksum,
            is_online = excluded.is_online,
            latency_ms = excluded.latency_ms,
            role = excluded.role
        "#,
        params![
            peer.node_id,
            peer.branch_id,
            peer.branch_name,
            peer.ip_address,
            peer.port,
            peer.last_seen_utc,
            peer.sync_epoch,
            peer.catalog_checksum,
            if peer.is_online { 1 } else { 0 },
            peer.latency_ms,
            peer.role.as_str(),
        ],
    )
    .map_err(|e| MeshError::Database(e.to_string()))?;
    Ok(())
}

/// Returns all active non-stale peers.
pub fn list_active_peers(
    conn: &Connection,
    current_ts: i64,
    ttl_secs: i64,
) -> Result<Vec<PeerNode>, MeshError> {
    let cutoff_ts = current_ts.saturating_sub(ttl_secs * 1000);
    let mut stmt = conn
        .prepare(
            r#"
            SELECT node_id, branch_id, branch_name, ip_address, port,
                   last_seen_utc, sync_epoch, catalog_checksum, is_online, latency_ms, role
            FROM mesh_peers
            WHERE last_seen_utc >= ?1 AND is_online = 1
            ORDER BY last_seen_utc DESC
            "#,
        )
        .map_err(|e| MeshError::Database(e.to_string()))?;

    let rows = stmt
        .query_map(params![cutoff_ts], |row| {
            let role_str: String = row.get(10)?;
            Ok(PeerNode {
                node_id: row.get(0)?,
                branch_id: row.get(1)?,
                branch_name: row.get(2)?,
                ip_address: row.get(3)?,
                port: row.get(4)?,
                last_seen_utc: row.get(5)?,
                sync_epoch: row.get(6)?,
                catalog_checksum: row.get(7)?,
                is_online: row.get::<_, i64>(8)? == 1,
                latency_ms: row.get(9)?,
                role: MeshRole::from_str(&role_str),
            })
        })
        .map_err(|e| MeshError::Database(e.to_string()))?;

    let mut peers = Vec::new();
    for r in rows {
        peers.push(r.map_err(|e| MeshError::Database(e.to_string()))?);
    }
    Ok(peers)
}

/// Updates the local branch sync epoch and catalog checksum.
pub fn record_local_epoch(
    conn: &Connection,
    branch_id: &str,
    epoch: u64,
    checksum: &str,
) -> Result<(), MeshError> {
    let now = Utc::now().timestamp_millis();
    conn.execute(
        r#"
        INSERT INTO mesh_sync_epochs (branch_id, current_epoch, catalog_checksum, updated_at_utc)
        VALUES (?1, ?2, ?3, ?4)
        ON CONFLICT(branch_id) DO UPDATE SET
            current_epoch = excluded.current_epoch,
            catalog_checksum = excluded.catalog_checksum,
            updated_at_utc = excluded.updated_at_utc
        "#,
        params![branch_id, epoch, checksum, now],
    )
    .map_err(|e| MeshError::Database(e.to_string()))?;
    Ok(())
}

/// Retrieves the current sync epoch for a branch.
pub fn get_local_epoch(conn: &Connection, branch_id: &str) -> Result<(u64, String), MeshError> {
    conn.query_row(
        r#"
        SELECT current_epoch, catalog_checksum
        FROM mesh_sync_epochs
        WHERE branch_id = ?1
        "#,
        params![branch_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )
    .map_err(|e| MeshError::Database(e.to_string()))
}
