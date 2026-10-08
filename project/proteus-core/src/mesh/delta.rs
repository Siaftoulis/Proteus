//! Differential Transaction Log & Branch Delta Extraction.
//! Captures localized database mutations and packages diffs for peer synchronization.
//! 100% Original Bespoke Implementation. Zero third-party boilerplate.

use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::types::MeshError;
use super::vector_clock::VectorClock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MutationOp {
    Upsert,
    Delete,
}

impl MutationOp {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Upsert => "UPSERT",
            Self::Delete => "DELETE",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "DELETE" => Self::Delete,
            _ => Self::Upsert,
        }
    }
}

/// An atomic data mutation tagged with branch origin and causal vector clock.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeltaMutation {
    pub mutation_id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub operation: MutationOp,
    pub origin_branch_id: String,
    pub sequence_number: u64,
    pub payload_json: String,
    pub vector_clock: VectorClock,
    pub created_at_utc: i64,
}

impl DeltaMutation {
    pub fn new(
        entity_type: impl Into<String>,
        entity_id: impl Into<String>,
        operation: MutationOp,
        origin_branch_id: impl Into<String>,
        sequence_number: u64,
        payload_json: impl Into<String>,
        vector_clock: VectorClock,
    ) -> Self {
        Self {
            mutation_id: Uuid::new_v4().to_string(),
            entity_type: entity_type.into(),
            entity_id: entity_id.into(),
            operation,
            origin_branch_id: origin_branch_id.into(),
            sequence_number,
            payload_json: payload_json.into(),
            vector_clock,
            created_at_utc: Utc::now().timestamp_millis(),
        }
    }
}

/// A serialized payload containing unmerged mutations sent from a source branch to a peer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BranchDeltaPackage {
    pub package_id: String,
    pub source_branch_id: String,
    pub target_branch_id: Option<String>,
    pub mutations: Vec<DeltaMutation>,
    pub catalog_checksum: String,
    pub created_at_utc: i64,
}

/// Initializes SQLite schema for differential transaction logging.
pub fn init_delta_schema(conn: &Connection) -> Result<(), MeshError> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS mesh_delta_log (
            mutation_id TEXT PRIMARY KEY,
            entity_type TEXT NOT NULL,
            entity_id TEXT NOT NULL,
            operation TEXT NOT NULL,
            origin_branch_id TEXT NOT NULL,
            sequence_number INTEGER NOT NULL,
            payload_json TEXT NOT NULL,
            vector_clock_json TEXT NOT NULL,
            created_at_utc INTEGER NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_mesh_delta_branch_seq 
            ON mesh_delta_log(origin_branch_id, sequence_number);
        CREATE INDEX IF NOT EXISTS idx_mesh_delta_entity 
            ON mesh_delta_log(entity_type, entity_id);
        "#,
    )
    .map_err(|e| MeshError::Database(e.to_string()))?;
    Ok(())
}

/// Appends a mutation entry into the branch's local delta log.
pub fn append_delta_mutation(conn: &Connection, mutation: &DeltaMutation) -> Result<(), MeshError> {
    let clock_json = mutation.vector_clock.to_json()?;

    conn.execute(
        r#"
        INSERT INTO mesh_delta_log (
            mutation_id, entity_type, entity_id, operation,
            origin_branch_id, sequence_number, payload_json,
            vector_clock_json, created_at_utc
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        "#,
        params![
            mutation.mutation_id,
            mutation.entity_type,
            mutation.entity_id,
            mutation.operation.as_str(),
            mutation.origin_branch_id,
            mutation.sequence_number,
            mutation.payload_json,
            clock_json,
            mutation.created_at_utc,
        ],
    )
    .map_err(|e| MeshError::Database(e.to_string()))?;

    Ok(())
}

/// Extracts all mutations newer than the peer's known vector clock.
pub fn extract_branch_delta(
    conn: &Connection,
    peer_known_clock: &VectorClock,
    source_branch_id: &str,
    target_branch_id: Option<&str>,
    current_catalog_checksum: &str,
) -> Result<BranchDeltaPackage, MeshError> {
    let mut stmt = conn
        .prepare(
            r#"
            SELECT mutation_id, entity_type, entity_id, operation,
                   origin_branch_id, sequence_number, payload_json,
                   vector_clock_json, created_at_utc
            FROM mesh_delta_log
            ORDER BY created_at_utc ASC
            "#,
        )
        .map_err(|e| MeshError::Database(e.to_string()))?;

    let rows = stmt
        .query_map([], |row| {
            let op_str: String = row.get(3)?;
            let clock_json: String = row.get(7)?;
            let clock = VectorClock::from_json(&clock_json).unwrap_or_default();

            Ok(DeltaMutation {
                mutation_id: row.get(0)?,
                entity_type: row.get(1)?,
                entity_id: row.get(2)?,
                operation: MutationOp::from_str(&op_str),
                origin_branch_id: row.get(4)?,
                sequence_number: row.get(5)?,
                payload_json: row.get(6)?,
                vector_clock: clock,
                created_at_utc: row.get(8)?,
            })
        })
        .map_err(|e| MeshError::Database(e.to_string()))?;

    let mut unmerged = Vec::new();
    for r in rows {
        let mutation = r.map_err(|e| MeshError::Database(e.to_string()))?;
        let peer_known_seq = peer_known_clock.get(&mutation.origin_branch_id);

        // Include mutation if the peer has not observed this sequence number yet
        if mutation.sequence_number > peer_known_seq {
            unmerged.push(mutation);
        }
    }

    Ok(BranchDeltaPackage {
        package_id: Uuid::new_v4().to_string(),
        source_branch_id: source_branch_id.to_string(),
        target_branch_id: target_branch_id.map(|s| s.to_string()),
        mutations: unmerged,
        catalog_checksum: current_catalog_checksum.to_string(),
        created_at_utc: Utc::now().timestamp_millis(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_delta_logging_and_selective_extraction() {
        let conn = Connection::open_in_memory().unwrap();
        init_delta_schema(&conn).unwrap();

        // Branch ATH mutations
        let mut vc1 = VectorClock::new();
        vc1.increment("BR-ATH");

        let m1 = DeltaMutation::new(
            "product",
            "SKU-FETA",
            MutationOp::Upsert,
            "BR-ATH",
            1,
            r#"{"name":"Feta 500g","price":6.50}"#,
            vc1.clone(),
        );
        append_delta_mutation(&conn, &m1).unwrap();

        vc1.increment("BR-ATH");
        let m2 = DeltaMutation::new(
            "product",
            "SKU-OIL",
            MutationOp::Upsert,
            "BR-ATH",
            2,
            r#"{"name":"Olive Oil 1L","price":9.90}"#,
            vc1.clone(),
        );
        append_delta_mutation(&conn, &m2).unwrap();

        // Peer knows sequence 0 -> must receive both mutations
        let mut peer_clock_empty = VectorClock::new();
        let delta_all = extract_branch_delta(
            &conn,
            &peer_clock_empty,
            "BR-ATH",
            Some("BR-SKG"),
            "hash_123",
        )
        .unwrap();
        assert_eq!(delta_all.mutations.len(), 2);
        assert_eq!(delta_all.mutations[0].entity_id, "SKU-FETA");
        assert_eq!(delta_all.mutations[1].entity_id, "SKU-OIL");

        // Peer has observed sequence 1 -> must receive only mutation 2
        peer_clock_empty.increment("BR-ATH"); // now seq = 1
        let delta_partial = extract_branch_delta(
            &conn,
            &peer_clock_empty,
            "BR-ATH",
            Some("BR-SKG"),
            "hash_123",
        )
        .unwrap();
        assert_eq!(delta_partial.mutations.len(), 1);
        assert_eq!(delta_partial.mutations[0].entity_id, "SKU-OIL");
        assert_eq!(delta_partial.mutations[0].sequence_number, 2);

        // Peer is fully caught up (seq = 2) -> empty delta
        peer_clock_empty.increment("BR-ATH"); // now seq = 2
        let delta_empty = extract_branch_delta(
            &conn,
            &peer_clock_empty,
            "BR-ATH",
            Some("BR-SKG"),
            "hash_123",
        )
        .unwrap();
        assert!(delta_empty.mutations.is_empty());
    }
}
