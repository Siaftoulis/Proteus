//! Types, enums, and models for the Multi-Store Cross-Database Federation Engine.
//! Designed from first principles for zero-rewrite database federation.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Role of a node within the multi-store federation topology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeRole {
    Hub,          // Central Headquarters / Primary Datacenter
    Branch,       // Retail branch / Service shop location
    Warehouse,    // Central distribution or cold-chain warehouse
    MobileRelay,  // Van sales / Mobile field service terminal
}

impl NodeRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Hub => "HUB",
            Self::Branch => "BRANCH",
            Self::Warehouse => "WAREHOUSE",
            Self::MobileRelay => "MOBILE_RELAY",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "HUB" => Self::Hub,
            "BRANCH" => Self::Branch,
            "WAREHOUSE" => Self::Warehouse,
            "MOBILE_RELAY" => Self::MobileRelay,
            _ => Self::Branch,
        }
    }
}

/// Operational status of a federated store node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeStatus {
    Active,
    Syncing,
    Suspended,
    Offline,
}

impl NodeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "ACTIVE",
            Self::Syncing => "SYNCING",
            Self::Suspended => "SUSPENDED",
            Self::Offline => "OFFLINE",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "ACTIVE" => Self::Active,
            "SYNCING" => Self::Syncing,
            "SUSPENDED" => Self::Suspended,
            "OFFLINE" => Self::Offline,
            _ => Self::Offline,
        }
    }
}

/// Permitted synchronization data-flow direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncDirection {
    Bidirectional,
    PushOnly,
    PullOnly,
}

impl SyncDirection {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Bidirectional => "BIDIRECTIONAL",
            Self::PushOnly => "PUSH_ONLY",
            Self::PullOnly => "PULL_ONLY",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "BIDIRECTIONAL" => Self::Bidirectional,
            "PUSH_ONLY" => Self::PushOnly,
            "PULL_ONLY" => Self::PullOnly,
            _ => Self::Bidirectional,
        }
    }
}

/// Mutating operation applied to a replicated database record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeOp {
    Insert,
    Update,
    Delete,
}

impl ChangeOp {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Insert => "INSERT",
            Self::Update => "UPDATE",
            Self::Delete => "DELETE",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "INSERT" => Self::Insert,
            "UPDATE" => Self::Update,
            "DELETE" => Self::Delete,
            _ => Self::Update,
        }
    }
}

/// Metadata and connection endpoint for a federated peer store.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FederatedStoreNode {
    pub node_id: String,
    pub store_id: String,
    pub store_name: String,
    pub endpoint_url: String,
    pub role: NodeRole,
    pub status: NodeStatus,
    pub sync_direction: SyncDirection,
    pub last_sync_seq: u64,
    pub last_sync_time: Option<String>,
    pub created_at: String,
}

/// Cryptographically sealed transaction delta in the cross-store replication stream.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FederatedTransaction {
    pub sequence: u64,
    pub tx_id: String,
    pub origin_store_id: String,
    pub entity: String,
    pub record_id: String,
    pub op: ChangeOp,
    pub payload_json: String,
    pub payload_hash: String,
    pub natural_key: Option<String>,
    pub created_at: String,
}

/// Sync progress tracking checkpoint for a remote store node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncCheckpoint {
    pub node_id: String,
    pub remote_store_id: String,
    pub last_received_seq: u64,
    pub last_sent_seq: u64,
    pub updated_at: String,
}

/// Comprehensive outcome report for a batch federation synchronization.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplicationReport {
    pub processed_count: usize,
    pub applied_count: usize,
    pub conflicts_resolved: usize,
    pub skipped_count: usize,
    pub highest_seq_applied: u64,
}

/// Federation engine error taxonomy.
#[derive(Debug, Error)]
pub enum FederationError {
    #[error("Database error: {0}")]
    Sql(#[from] rusqlite::Error),
    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Integrity error: {0}")]
    IntegrityViolation(String),
    #[error("Node not found: {0}")]
    NodeNotFound(String),
    #[error("Replication conflict: {0}")]
    Conflict(String),
}
