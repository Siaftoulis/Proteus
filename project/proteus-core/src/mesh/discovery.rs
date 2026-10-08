//! Peer Heartbeat & Topology Health Tracking for Multi-Branch LAN Mesh.
//! Manages continuous beacon announcement, active peer heartbeats, and stale peer pruning.
//! 100% Original Bespoke Implementation.

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::net::UdpSocket;

use super::beacon::{
    broadcast_beacon, process_incoming_beacon, receive_beacon_packet, MeshBeaconConfig,
    MAX_BEACON_PACKET_SIZE,
};
use super::types::{list_active_peers, BranchBeacon, MeshError, MeshRole, PeerNode};

/// Summary of current branch mesh topology health.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeshTopologySummary {
    pub local_branch_id: String,
    pub total_known_peers: usize,
    pub online_peers_count: usize,
    pub synchronized_catalog_count: usize,
    pub drifted_catalog_count: usize,
    pub coordinator_branch_id: Option<String>,
}

/// Marks peers as offline if they have not sent a heartbeat within the TTL period.
pub fn prune_stale_peers(
    conn: &Connection,
    current_ts: i64,
    ttl_secs: i64,
) -> Result<usize, MeshError> {
    let cutoff_ts = current_ts.saturating_sub(ttl_secs * 1000);
    let updated = conn
        .execute(
            r#"
            UPDATE mesh_peers
            SET is_online = 0
            WHERE last_seen_utc < ?1 AND is_online = 1
            "#,
            params![cutoff_ts],
        )
        .map_err(|e| MeshError::Database(e.to_string()))?;
    Ok(updated)
}

/// Evaluates network topology health and catalog synchronization across all known branch peers.
pub fn evaluate_mesh_topology(
    conn: &Connection,
    local_branch_id: &str,
    local_catalog_checksum: &str,
    current_ts: i64,
    ttl_secs: i64,
) -> Result<MeshTopologySummary, MeshError> {
    let active_peers = list_active_peers(conn, current_ts, ttl_secs)?;

    let mut total_known: usize = 0;
    let mut coordinator_id = None;
    let mut synchronized_catalog = 0;
    let mut drifted_catalog = 0;

    // Count all registered peers in database
    conn.query_row("SELECT count(*) FROM mesh_peers", [], |row| {
        total_known = row.get(0)?;
        Ok(())
    })
    .map_err(|e| MeshError::Database(e.to_string()))?;

    for peer in &active_peers {
        if peer.role == MeshRole::Coordinator {
            coordinator_id = Some(peer.branch_id.clone());
        }

        if peer.catalog_checksum == local_catalog_checksum {
            synchronized_catalog += 1;
        } else {
            drifted_catalog += 1;
        }
    }

    Ok(MeshTopologySummary {
        local_branch_id: local_branch_id.to_string(),
        total_known_peers: total_known,
        online_peers_count: active_peers.len(),
        synchronized_catalog_count: synchronized_catalog,
        drifted_catalog_count: drifted_catalog,
        coordinator_branch_id: coordinator_id,
    })
}

/// Executes a single discovery tick: broadcasts local beacon, drains incoming queue, and prunes stale peers.
pub fn step_mesh_discovery(
    conn: &Connection,
    socket: &UdpSocket,
    config: &MeshBeaconConfig,
    local_beacon: &BranchBeacon,
    current_ts: i64,
) -> Result<Vec<PeerNode>, MeshError> {
    // 1. Broadcast local beacon
    let _ = broadcast_beacon(socket, config.broadcast_port, local_beacon);

    // 2. Drain incoming socket packets
    let mut newly_discovered = Vec::new();
    let mut buffer = [0u8; MAX_BEACON_PACKET_SIZE];

    loop {
        match receive_beacon_packet(socket, &mut buffer) {
            Ok(Some((beacon, sender_addr))) => {
                if let Ok(Some(peer)) =
                    process_incoming_beacon(conn, &beacon, sender_addr, &local_beacon.node_id)
                {
                    newly_discovered.push(peer);
                }
            }
            Ok(None) => break, // Socket drained
            Err(_) => break,
        }
    }

    // 3. Mark inactive peers offline
    prune_stale_peers(conn, current_ts, config.peer_ttl_seconds)?;

    Ok(newly_discovered)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::types::{init_mesh_schema, upsert_peer_node};
    use chrono::Utc;
    use rusqlite::Connection;

    #[test]
    fn test_stale_peer_pruning_and_status_transition() {
        let conn = Connection::open_in_memory().unwrap();
        init_mesh_schema(&conn).unwrap();

        let p1 = PeerNode {
            node_id: "P1".to_string(),
            branch_id: "BR-1".to_string(),
            branch_name: "Branch 1".to_string(),
            ip_address: "192.168.1.10".to_string(),
            port: 7446,
            last_seen_utc: 100_000, // Very old
            sync_epoch: 10,
            catalog_checksum: "hash1".to_string(),
            is_online: true,
            latency_ms: 10,
            role: MeshRole::PeerNode,
        };

        let p2 = PeerNode {
            node_id: "P2".to_string(),
            branch_id: "BR-2".to_string(),
            branch_name: "Branch 2".to_string(),
            ip_address: "192.168.1.11".to_string(),
            port: 7446,
            last_seen_utc: 150_000, // Recent
            sync_epoch: 10,
            catalog_checksum: "hash1".to_string(),
            is_online: true,
            latency_ms: 15,
            role: MeshRole::Coordinator,
        };

        upsert_peer_node(&conn, &p1).unwrap();
        upsert_peer_node(&conn, &p2).unwrap();

        // Prune with current_ts = 152_000, ttl = 10s (cutoff = 142_000)
        let pruned = prune_stale_peers(&conn, 152_000, 10).unwrap();
        assert_eq!(pruned, 1); // P1 pruned

        let p1_online: i64 = conn
            .query_row("SELECT is_online FROM mesh_peers WHERE node_id = 'P1'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(p1_online, 0);

        let p2_online: i64 = conn
            .query_row("SELECT is_online FROM mesh_peers WHERE node_id = 'P2'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(p2_online, 1);
    }

    #[test]
    fn test_evaluate_mesh_topology_drift_and_coordinator() {
        let conn = Connection::open_in_memory().unwrap();
        init_mesh_schema(&conn).unwrap();

        let base_ts = Utc::now().timestamp_millis();

        let peer_sync = PeerNode {
            node_id: "SYNC-NODE".to_string(),
            branch_id: "BR-LARISSA".to_string(),
            branch_name: "Larissa".to_string(),
            ip_address: "192.168.1.30".to_string(),
            port: 7446,
            last_seen_utc: base_ts,
            sync_epoch: 5,
            catalog_checksum: "CAT-MATCH".to_string(),
            is_online: true,
            latency_ms: 8,
            role: MeshRole::Coordinator,
        };

        let peer_drift = PeerNode {
            node_id: "DRIFT-NODE".to_string(),
            branch_id: "BR-PATRA".to_string(),
            branch_name: "Patra".to_string(),
            ip_address: "192.168.1.31".to_string(),
            port: 7446,
            last_seen_utc: base_ts,
            sync_epoch: 4,
            catalog_checksum: "CAT-OLD-VERSION".to_string(),
            is_online: true,
            latency_ms: 22,
            role: MeshRole::PeerNode,
        };

        upsert_peer_node(&conn, &peer_sync).unwrap();
        upsert_peer_node(&conn, &peer_drift).unwrap();

        let summary = evaluate_mesh_topology(&conn, "BR-ATHENS", "CAT-MATCH", base_ts, 15).unwrap();
        assert_eq!(summary.total_known_peers, 2);
        assert_eq!(summary.online_peers_count, 2);
        assert_eq!(summary.synchronized_catalog_count, 1);
        assert_eq!(summary.drifted_catalog_count, 1);
        assert_eq!(summary.coordinator_branch_id.as_deref(), Some("BR-LARISSA"));
    }
}
