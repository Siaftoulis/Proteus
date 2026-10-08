//! Proteus Multi-Store Cross-Database Federation Engine.
//! Designed from first principles for zero-rewrite database federation.

pub mod engine;
pub mod types;

pub use engine::MultiStoreFederationEngine;
pub use types::{
    ChangeOp, FederatedStoreNode, FederatedTransaction, FederationError, NodeRole, NodeStatus,
    ReplicationReport, SyncCheckpoint, SyncDirection,
};

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_federation_schema_and_node_registration() {
        let conn = Connection::open_in_memory().unwrap();
        let engine = MultiStoreFederationEngine::new("STORE_ATH_01", "Athens Flagship Store");
        engine.init_schema(&conn).unwrap();

        let branch_node = FederatedStoreNode {
            node_id: "NODE_THES_01".to_string(),
            store_id: "STORE_THES_01".to_string(),
            store_name: "Thessaloniki Branch".to_string(),
            endpoint_url: "sqlite://thessaloniki.db".to_string(),
            role: NodeRole::Branch,
            status: NodeStatus::Active,
            sync_direction: SyncDirection::Bidirectional,
            last_sync_seq: 0,
            last_sync_time: None,
            created_at: "2026-10-08T12:00:00Z".to_string(),
        };

        engine.register_node(&conn, &branch_node).unwrap();
        let nodes = engine.list_nodes(&conn).unwrap();

        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].node_id, "NODE_THES_01");
        assert_eq!(nodes[0].role, NodeRole::Branch);
        assert_eq!(nodes[0].status, NodeStatus::Active);
    }

    #[test]
    fn test_record_change_and_outgoing_deltas() {
        let conn = Connection::open_in_memory().unwrap();
        let engine = MultiStoreFederationEngine::new("STORE_ATH_01", "Athens Flagship Store");
        engine.init_schema(&conn).unwrap();

        let branch_node = FederatedStoreNode {
            node_id: "NODE_THES_01".to_string(),
            store_id: "STORE_THES_01".to_string(),
            store_name: "Thessaloniki Branch".to_string(),
            endpoint_url: "sqlite://thessaloniki.db".to_string(),
            role: NodeRole::Branch,
            status: NodeStatus::Active,
            sync_direction: SyncDirection::Bidirectional,
            last_sync_seq: 0,
            last_sync_time: None,
            created_at: "2026-10-08T12:00:00Z".to_string(),
        };
        engine.register_node(&conn, &branch_node).unwrap();

        let tx1 = engine
            .record_local_change(
                &conn,
                "inventory",
                "SKU-9901",
                ChangeOp::Insert,
                r#"{"sku":"SKU-9901","qty":50,"price":120.0}"#,
                Some("SKU-9901"),
            )
            .unwrap();

        assert_eq!(tx1.sequence, 1);
        assert!(!tx1.payload_hash.is_empty());

        let deltas = engine.pull_outgoing_deltas(&conn, "NODE_THES_01", 10).unwrap();
        assert_eq!(deltas.len(), 1);
        assert_eq!(deltas[0].record_id, "SKU-9901");

        // Second pull without new changes should yield empty batch
        let deltas_empty = engine.pull_outgoing_deltas(&conn, "NODE_THES_01", 10).unwrap();
        assert_eq!(deltas_empty.len(), 0);
    }

    #[test]
    fn test_tamper_detection_and_integrity_check() {
        let conn = Connection::open_in_memory().unwrap();
        let engine = MultiStoreFederationEngine::new("STORE_ATH_01", "Athens Flagship Store");
        engine.init_schema(&conn).unwrap();

        let mut tampered_tx = FederatedTransaction {
            sequence: 1,
            tx_id: "tx-corrupt-001".to_string(),
            origin_store_id: "STORE_THES_01".to_string(),
            entity: "pricing".to_string(),
            record_id: "PR-01".to_string(),
            op: ChangeOp::Update,
            payload_json: r#"{"price":10.0}"#.to_string(),
            payload_hash: "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            natural_key: None,
            created_at: "2026-10-08T12:00:00Z".to_string(),
        };

        let result = engine.apply_incoming_deltas(&conn, "NODE_THES_01", &[tampered_tx.clone()]);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FederationError::IntegrityViolation(_)));

        // Fix hash to valid SHA-256 and ensure successful apply
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(tampered_tx.payload_json.as_bytes());
        tampered_tx.payload_hash = format!("{:x}", hasher.finalize());

        let valid_result = engine.apply_incoming_deltas(&conn, "NODE_THES_01", &[tampered_tx]);
        assert!(valid_result.is_ok());
        let report = valid_result.unwrap();
        assert_eq!(report.applied_count, 1);
    }

    #[test]
    fn test_bidirectional_multi_store_sync_between_live_dbs() {
        let conn_hq = Connection::open_in_memory().unwrap();
        let conn_branch = Connection::open_in_memory().unwrap();

        let engine_hq = MultiStoreFederationEngine::new("STORE_HQ", "Headquarters Hub");
        let engine_branch = MultiStoreFederationEngine::new("STORE_BRANCH_A", "Patras Branch");

        engine_hq.init_schema(&conn_hq).unwrap();
        engine_branch.init_schema(&conn_branch).unwrap();

        // Register nodes
        let hq_in_branch = FederatedStoreNode {
            node_id: "NODE_HQ".to_string(),
            store_id: "STORE_HQ".to_string(),
            store_name: "Headquarters Hub".to_string(),
            endpoint_url: "sqlite://hq.db".to_string(),
            role: NodeRole::Hub,
            status: NodeStatus::Active,
            sync_direction: SyncDirection::Bidirectional,
            last_sync_seq: 0,
            last_sync_time: None,
            created_at: "2026-10-08T12:00:00Z".to_string(),
        };
        let branch_in_hq = FederatedStoreNode {
            node_id: "NODE_BRANCH_A".to_string(),
            store_id: "STORE_BRANCH_A".to_string(),
            store_name: "Patras Branch".to_string(),
            endpoint_url: "sqlite://patras.db".to_string(),
            role: NodeRole::Branch,
            status: NodeStatus::Active,
            sync_direction: SyncDirection::Bidirectional,
            last_sync_seq: 0,
            last_sync_time: None,
            created_at: "2026-10-08T12:00:00Z".to_string(),
        };

        engine_hq.register_node(&conn_hq, &branch_in_hq).unwrap();
        engine_branch.register_node(&conn_branch, &hq_in_branch).unwrap();

        // HQ creates a master catalog price update
        engine_hq
            .record_local_change(
                &conn_hq,
                "catalog",
                "CAT-808",
                ChangeOp::Update,
                r#"{"item":"Display Panel","msrp":89.90}"#,
                Some("CAT-808"),
            )
            .unwrap();

        // Branch creates a local repair service ticket
        engine_branch
            .record_local_change(
                &conn_branch,
                "service_tickets",
                "TCK-2026-001",
                ChangeOp::Insert,
                r#"{"ticket_no":"TCK-2026-001","status":"received","customer":"Nikos"}"#,
                Some("TCK-2026-001"),
            )
            .unwrap();

        // Perform live two-way sync
        let (hq_to_branch, branch_to_hq) = engine_hq
            .sync_peer_databases(
                &conn_hq,
                &engine_branch,
                &conn_branch,
                "NODE_BRANCH_A",
                "NODE_HQ",
                50,
            )
            .unwrap();

        assert_eq!(hq_to_branch.applied_count, 1);
        assert_eq!(branch_to_hq.applied_count, 1);

        // Verify Branch received HQ's catalog update
        let branch_deltas = engine_branch.pull_outgoing_deltas(&conn_branch, "NODE_HQ", 10).unwrap();
        // The outgoing deltas for HQ from Branch are now drained
        assert_eq!(branch_deltas.len(), 0);

        // Verify loopback prevention: Re-syncing does not duplicate transactions
        let (hq_to_branch_2, branch_to_hq_2) = engine_hq
            .sync_peer_databases(
                &conn_hq,
                &engine_branch,
                &conn_branch,
                "NODE_BRANCH_A",
                "NODE_HQ",
                50,
            )
            .unwrap();

        assert_eq!(hq_to_branch_2.applied_count, 0);
        assert_eq!(branch_to_hq_2.applied_count, 0);
    }
}
