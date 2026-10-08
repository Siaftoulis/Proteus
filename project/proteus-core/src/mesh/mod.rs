//! Multi-Branch LAN Mesh Synchronization Engine for Proteus BOS.
//! Peer-to-peer branch discovery, vector clock differential replication, and offline ledger sync.
//! 100% Original Bespoke Implementation. Zero third-party boilerplate.

pub mod beacon;
pub mod delta;
pub mod discovery;
pub mod reconcile;
pub mod types;
pub mod vector_clock;

pub use beacon::{
    bind_mesh_socket, broadcast_beacon, process_incoming_beacon, receive_beacon_packet,
    MeshBeaconConfig, DEFAULT_MESH_PORT, MAX_BEACON_PACKET_SIZE,
};
pub use delta::{
    append_delta_mutation, extract_branch_delta, init_delta_schema, BranchDeltaPackage,
    DeltaMutation, MutationOp,
};
pub use discovery::{
    evaluate_mesh_topology, prune_stale_peers, step_mesh_discovery, MeshTopologySummary,
};
pub use reconcile::{
    apply_remote_delta, resolve_concurrent_conflict, ConflictWinner, ReconciliationResult,
};
pub use types::{
    compute_catalog_checksum, get_local_epoch, init_mesh_schema, list_active_peers,
    record_local_epoch, upsert_peer_node, BranchBeacon, MeshError, MeshRole, PeerNode,
};
pub use vector_clock::{
    get_vector_clock, init_vector_clock_schema, record_local_mutation, save_vector_clock,
    ClockOrdering, VectorClock,
};

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_beacon_serialization_and_packet_magic() {
        let beacon = BranchBeacon::new(
            "BR-ATH-01",
            "NODE-001",
            "Athens Flagship",
            7445,
            1042,
            "abc123hash",
            MeshRole::Coordinator,
        );

        let bytes = beacon.to_packet_bytes().unwrap();
        assert!(!bytes.is_empty());

        let decoded = BranchBeacon::from_packet_bytes(&bytes).unwrap();
        assert_eq!(decoded.branch_id, "BR-ATH-01");
        assert_eq!(decoded.node_id, "NODE-001");
        assert_eq!(decoded.branch_name, "Athens Flagship");
        assert_eq!(decoded.listen_port, 7445);
        assert_eq!(decoded.sync_epoch, 1042);
        assert_eq!(decoded.role, MeshRole::Coordinator);

        // Invalid packet test
        let mut corrupted_bytes = bytes.clone();
        corrupted_bytes[0] = b'{';
        let invalid = BranchBeacon::from_packet_bytes(b"invalid payload");
        assert!(invalid.is_err());
    }

    #[test]
    fn test_catalog_checksum_deterministic() {
        let items = [
            ("SKU-OIL-1L", 9.50, "Ελαιόλαδο 1L"),
            ("SKU-FETA-500", 6.80, "Φέτα ΠΟΠ 500g"),
        ];

        let hash1 = compute_catalog_checksum(&items);
        let hash2 = compute_catalog_checksum(&items);
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA-256 hex string

        // Modifying price must change checksum
        let items_modified = [
            ("SKU-OIL-1L", 9.90, "Ελαιόλαδο 1L"),
            ("SKU-FETA-500", 6.80, "Φέτα ΠΟΠ 500g"),
        ];
        let hash3 = compute_catalog_checksum(&items_modified);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_mesh_sqlite_lifecycle_and_peer_discovery() {
        let conn = Connection::open_in_memory().unwrap();
        init_mesh_schema(&conn).unwrap();

        // 1. Record local epoch
        record_local_epoch(&conn, "BR-ATH-01", 100, "initial_checksum").unwrap();
        let (epoch, checksum) = get_local_epoch(&conn, "BR-ATH-01").unwrap();
        assert_eq!(epoch, 100);
        assert_eq!(checksum, "initial_checksum");

        // 2. Insert peer node
        let peer = PeerNode {
            node_id: "NODE-THES-01".to_string(),
            branch_id: "BR-SKG-02".to_string(),
            branch_name: "Thessaloniki Center".to_string(),
            ip_address: "192.168.1.150".to_string(),
            port: 7445,
            last_seen_utc: 1000_000,
            sync_epoch: 98,
            catalog_checksum: "peer_checksum".to_string(),
            is_online: true,
            latency_ms: 12,
            role: MeshRole::PeerNode,
        };
        upsert_peer_node(&conn, &peer).unwrap();

        // Active lookup with 30s TTL
        let active = list_active_peers(&conn, 1005_000, 30).unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].node_id, "NODE-THES-01");
        assert_eq!(active[0].branch_name, "Thessaloniki Center");
        assert_eq!(active[0].role, MeshRole::PeerNode);

        // Stale lookup (60s elapsed > 30s TTL)
        let stale = list_active_peers(&conn, 1035_000, 30).unwrap();
        assert_eq!(stale.len(), 0);
    }
}
