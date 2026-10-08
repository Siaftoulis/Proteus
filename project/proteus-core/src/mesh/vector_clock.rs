//! Distributed Vector Clock Data Structures & Monotonic Mutation Tracking.
//! Provides partial ordering, causality tracking, and concurrency detection across offline branches.
//! 100% Original Bespoke Implementation. Zero third-party boilerplate.

use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::types::MeshError;

/// Causal relationship between two vector clocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClockOrdering {
    Equal,
    Dominates, // Self is strictly newer than other
    Dominated, // Self is strictly older than other
    Concurrent, // Branch split / conflicting mutations
}

/// A distributed logical clock mapping branch IDs to monotonic sequence numbers.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct VectorClock {
    pub entries: BTreeMap<String, u64>,
}

impl VectorClock {
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    /// Increments the monotonic sequence counter for a specific branch node.
    pub fn increment(&mut self, branch_id: &str) -> u64 {
        let counter = self.entries.entry(branch_id.to_string()).or_insert(0);
        *counter += 1;
        *counter
    }

    /// Retrieves the sequence counter for a specific branch node.
    pub fn get(&self, branch_id: &str) -> u64 {
        self.entries.get(branch_id).copied().unwrap_or(0)
    }

    /// Merges another vector clock by taking the point-wise maximum of all branch counters.
    pub fn merge(&mut self, other: &VectorClock) {
        for (branch, &other_seq) in &other.entries {
            let self_seq = self.entries.entry(branch.clone()).or_insert(0);
            if other_seq > *self_seq {
                *self_seq = other_seq;
            }
        }
    }

    /// Determines the causal relationship between self and another vector clock.
    pub fn compare(&self, other: &VectorClock) -> ClockOrdering {
        let mut self_greater = false;
        let mut other_greater = false;

        // Check all keys present in self
        for (branch, &self_val) in &self.entries {
            let other_val = other.get(branch);
            if self_val > other_val {
                self_greater = true;
            } else if self_val < other_val {
                other_greater = true;
            }
        }

        // Check keys present only in other
        for (branch, &other_val) in &other.entries {
            if !self.entries.contains_key(branch) && other_val > 0 {
                other_greater = true;
            }
        }

        match (self_greater, other_greater) {
            (false, false) => ClockOrdering::Equal,
            (true, false) => ClockOrdering::Dominates,
            (false, true) => ClockOrdering::Dominated,
            (true, true) => ClockOrdering::Concurrent,
        }
    }

    pub fn to_json(&self) -> Result<String, MeshError> {
        serde_json::to_string(&self.entries).map_err(|e| MeshError::Serialization(e.to_string()))
    }

    pub fn from_json(json: &str) -> Result<Self, MeshError> {
        let entries: BTreeMap<String, u64> =
            serde_json::from_str(json).map_err(|e| MeshError::Serialization(e.to_string()))?;
        Ok(Self { entries })
    }
}

/// Initializes SQLite table for storing entity vector clocks.
pub fn init_vector_clock_schema(conn: &Connection) -> Result<(), MeshError> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS mesh_vector_clocks (
            entity_type TEXT NOT NULL,
            entity_id TEXT NOT NULL,
            branch_id TEXT NOT NULL,
            clock_json TEXT NOT NULL,
            last_mutated_utc INTEGER NOT NULL,
            PRIMARY KEY (entity_type, entity_id)
        );

        CREATE INDEX IF NOT EXISTS idx_mesh_vc_branch ON mesh_vector_clocks(branch_id);
        "#,
    )
    .map_err(|e| MeshError::Database(e.to_string()))?;
    Ok(())
}

/// Records a local mutation on an entity, advancing the branch's sequence number in its vector clock.
pub fn record_local_mutation(
    conn: &Connection,
    entity_type: &str,
    entity_id: &str,
    local_branch_id: &str,
) -> Result<VectorClock, MeshError> {
    let mut clock = get_vector_clock(conn, entity_type, entity_id)?.unwrap_or_default();
    clock.increment(local_branch_id);

    let clock_json = clock.to_json()?;
    let now = Utc::now().timestamp_millis();

    conn.execute(
        r#"
        INSERT INTO mesh_vector_clocks (entity_type, entity_id, branch_id, clock_json, last_mutated_utc)
        VALUES (?1, ?2, ?3, ?4, ?5)
        ON CONFLICT(entity_type, entity_id) DO UPDATE SET
            branch_id = excluded.branch_id,
            clock_json = excluded.clock_json,
            last_mutated_utc = excluded.last_mutated_utc
        "#,
        params![entity_type, entity_id, local_branch_id, clock_json, now],
    )
    .map_err(|e| MeshError::Database(e.to_string()))?;

    Ok(clock)
}

/// Retrieves the current vector clock for a specific entity.
pub fn get_vector_clock(
    conn: &Connection,
    entity_type: &str,
    entity_id: &str,
) -> Result<Option<VectorClock>, MeshError> {
    let mut stmt = conn
        .prepare(
            r#"
            SELECT clock_json
            FROM mesh_vector_clocks
            WHERE entity_type = ?1 AND entity_id = ?2
            "#,
        )
        .map_err(|e| MeshError::Database(e.to_string()))?;

    let mut rows = stmt
        .query(params![entity_type, entity_id])
        .map_err(|e| MeshError::Database(e.to_string()))?;

    if let Some(row) = rows.next().map_err(|e| MeshError::Database(e.to_string()))? {
        let json: String = row.get(0).map_err(|e| MeshError::Database(e.to_string()))?;
        let clock = VectorClock::from_json(&json)?;
        Ok(Some(clock))
    } else {
        Ok(None)
    }
}

/// Persists an updated vector clock into the SQLite database.
pub fn save_vector_clock(
    conn: &Connection,
    entity_type: &str,
    entity_id: &str,
    branch_id: &str,
    clock: &VectorClock,
) -> Result<(), MeshError> {
    let clock_json = clock.to_json()?;
    let now = Utc::now().timestamp_millis();

    conn.execute(
        r#"
        INSERT INTO mesh_vector_clocks (entity_type, entity_id, branch_id, clock_json, last_mutated_utc)
        VALUES (?1, ?2, ?3, ?4, ?5)
        ON CONFLICT(entity_type, entity_id) DO UPDATE SET
            branch_id = excluded.branch_id,
            clock_json = excluded.clock_json,
            last_mutated_utc = excluded.last_mutated_utc
        "#,
        params![entity_type, entity_id, branch_id, clock_json, now],
    )
    .map_err(|e| MeshError::Database(e.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_vector_clock_comparisons() {
        let mut vc1 = VectorClock::new();
        let mut vc2 = VectorClock::new();

        // Initially equal (empty)
        assert_eq!(vc1.compare(&vc2), ClockOrdering::Equal);

        // vc1 advances Branch A
        vc1.increment("BR-ATH");
        assert_eq!(vc1.compare(&vc2), ClockOrdering::Dominates);
        assert_eq!(vc2.compare(&vc1), ClockOrdering::Dominated);

        // vc2 merges vc1 and advances Branch B
        vc2.merge(&vc1);
        assert_eq!(vc2.compare(&vc1), ClockOrdering::Equal);
        vc2.increment("BR-SKG");
        assert_eq!(vc2.compare(&vc1), ClockOrdering::Dominates);

        // vc1 independently advances Branch A -> Concurrent divergence
        vc1.increment("BR-ATH");
        assert_eq!(vc1.compare(&vc2), ClockOrdering::Concurrent);
        assert_eq!(vc2.compare(&vc1), ClockOrdering::Concurrent);

        // Merging resolves concurrency
        let mut merged = vc1.clone();
        merged.merge(&vc2);
        assert_eq!(merged.get("BR-ATH"), 2);
        assert_eq!(merged.get("BR-SKG"), 1);
        assert_eq!(merged.compare(&vc1), ClockOrdering::Dominates);
        assert_eq!(merged.compare(&vc2), ClockOrdering::Dominates);
    }

    #[test]
    fn test_vector_clock_sqlite_persistence() {
        let conn = Connection::open_in_memory().unwrap();
        init_vector_clock_schema(&conn).unwrap();

        // 1. Initial query returns None
        let initial = get_vector_clock(&conn, "product", "SKU-9901").unwrap();
        assert!(initial.is_none());

        // 2. Record local mutation
        let vc_after1 = record_local_mutation(&conn, "product", "SKU-9901", "BR-ATH").unwrap();
        assert_eq!(vc_after1.get("BR-ATH"), 1);

        // 3. Second local mutation on same entity
        let vc_after2 = record_local_mutation(&conn, "product", "SKU-9901", "BR-ATH").unwrap();
        assert_eq!(vc_after2.get("BR-ATH"), 2);

        // 4. Remote merge simulation
        let mut remote_vc = VectorClock::new();
        remote_vc.increment("BR-SKG");
        remote_vc.increment("BR-SKG");

        let mut current = get_vector_clock(&conn, "product", "SKU-9901").unwrap().unwrap();
        current.merge(&remote_vc);
        save_vector_clock(&conn, "product", "SKU-9901", "BR-SKG", &current).unwrap();

        let persisted = get_vector_clock(&conn, "product", "SKU-9901").unwrap().unwrap();
        assert_eq!(persisted.get("BR-ATH"), 2);
        assert_eq!(persisted.get("BR-SKG"), 2);
    }
}
