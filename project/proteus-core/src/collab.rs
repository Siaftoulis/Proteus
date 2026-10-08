//! Multi-User Real-Time Collaboration & Distributed Record Locking Engine.
//!
//! Provides concurrency control, deterministic Last-Write-Wins (LWW) conflict resolution,
//! distributed TTL lease locks to prevent concurrent overrides on shared counters/benches,
//! and persistent audit tracking of active lock leases.

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn current_epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Distributed record lease lock representing an active operator editing session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordLock {
    pub lock_key: String,
    pub entity_type: String,
    pub entity_id: String,
    pub holder_client_id: String,
    pub holder_user_name: String,
    pub holder_station: String,
    pub acquired_at_ms: u64,
    pub ttl_ms: u64,
}

impl RecordLock {
    pub fn new(
        entity_type: impl Into<String>,
        entity_id: impl Into<String>,
        client_id: impl Into<String>,
        user_name: impl Into<String>,
        station: impl Into<String>,
        ttl_ms: u64,
    ) -> Self {
        let e_type = entity_type.into();
        let e_id = entity_id.into();
        let key = format!("{}:{}", e_type, e_id);
        Self {
            lock_key: key,
            entity_type: e_type,
            entity_id: e_id,
            holder_client_id: client_id.into(),
            holder_user_name: user_name.into(),
            holder_station: station.into(),
            acquired_at_ms: current_epoch_ms(),
            ttl_ms,
        }
    }

    pub fn is_expired(&self, now_ms: u64) -> bool {
        now_ms.saturating_sub(self.acquired_at_ms) > self.ttl_ms
    }

    pub fn expires_at_ms(&self) -> u64 {
        self.acquired_at_ms + self.ttl_ms
    }
}

/// Error returned when another operator holds an active non-expired lock lease.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockConflict {
    pub entity_type: String,
    pub entity_id: String,
    pub current_holder_name: String,
    pub current_holder_station: String,
    pub remaining_seconds: u64,
}

/// Thread-safe in-memory and database-backed distributed record lock manager.
#[derive(Clone, Default)]
pub struct RecordLockManager {
    locks: Arc<Mutex<HashMap<String, RecordLock>>>,
}

impl RecordLockManager {
    pub fn new() -> Self {
        Self {
            locks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Attempts to acquire an exclusive editing lock for a record.
    pub fn acquire_lock(
        &self,
        entity_type: &str,
        entity_id: &str,
        client_id: &str,
        user_name: &str,
        station: &str,
        ttl_ms: u64,
    ) -> Result<RecordLock, LockConflict> {
        let key = format!("{}:{}", entity_type, entity_id);
        let now = current_epoch_ms();
        let mut map = self.locks.lock().unwrap();

        if let Some(existing) = map.get(&key) {
            if !existing.is_expired(now) && existing.holder_client_id != client_id {
                let remaining = (existing.expires_at_ms().saturating_sub(now)) / 1000;
                return Err(LockConflict {
                    entity_type: entity_type.to_string(),
                    entity_id: entity_id.to_string(),
                    current_holder_name: existing.holder_user_name.clone(),
                    current_holder_station: existing.holder_station.clone(),
                    remaining_seconds: remaining.max(1),
                });
            }
        }

        let lock = RecordLock::new(entity_type, entity_id, client_id, user_name, station, ttl_ms);
        map.insert(key, lock.clone());
        Ok(lock)
    }

    /// Releases an existing lock if the caller is the holder.
    pub fn release_lock(&self, entity_type: &str, entity_id: &str, client_id: &str) -> bool {
        let key = format!("{}:{}", entity_type, entity_id);
        let mut map = self.locks.lock().unwrap();
        if let Some(existing) = map.get(&key) {
            if existing.holder_client_id == client_id {
                map.remove(&key);
                return true;
            }
        }
        false
    }

    /// Extends the lease TTL for an active lock held by the caller.
    pub fn heartbeat_lock(&self, entity_type: &str, entity_id: &str, client_id: &str, ttl_ms: u64) -> bool {
        let key = format!("{}:{}", entity_type, entity_id);
        let now = current_epoch_ms();
        let mut map = self.locks.lock().unwrap();
        if let Some(existing) = map.get_mut(&key) {
            if existing.holder_client_id == client_id {
                existing.acquired_at_ms = now;
                existing.ttl_ms = ttl_ms;
                return true;
            }
        }
        false
    }

    /// Retrieves active lock details if one exists and has not expired.
    pub fn get_active_lock(&self, entity_type: &str, entity_id: &str) -> Option<RecordLock> {
        let key = format!("{}:{}", entity_type, entity_id);
        let now = current_epoch_ms();
        let mut map = self.locks.lock().unwrap();
        if let Some(existing) = map.get(&key) {
            if !existing.is_expired(now) {
                return Some(existing.clone());
            } else {
                map.remove(&key);
            }
        }
        None
    }

    /// Cleans up all stale locks whose lease has expired.
    pub fn purge_expired(&self) -> usize {
        let now = current_epoch_ms();
        let mut map = self.locks.lock().unwrap();
        let initial_len = map.len();
        map.retain(|_, lock| !lock.is_expired(now));
        initial_len - map.len()
    }
}

/// Deterministic Last-Write-Wins (LWW) Mutation Register.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LwwMutation<T> {
    pub entity_id: String,
    pub timestamp_ms: u64,
    pub client_id: String,
    pub payload: T,
}

impl<T: Clone> LwwMutation<T> {
    pub fn new(entity_id: impl Into<String>, client_id: impl Into<String>, payload: T) -> Self {
        Self {
            entity_id: entity_id.into(),
            timestamp_ms: current_epoch_ms(),
            client_id: client_id.into(),
            payload,
        }
    }

    /// Reconciles two concurrent mutations deterministically. Higher timestamp wins; client_id breaks ties.
    pub fn reconcile<'a>(&'a self, other: &'a Self) -> &'a Self {
        if self.timestamp_ms > other.timestamp_ms {
            self
        } else if other.timestamp_ms > self.timestamp_ms {
            other
        } else if self.client_id >= other.client_id {
            self
        } else {
            other
        }
    }
}

/// Initializes SQLite schema for persisting active collaboration locks.
pub fn ensure_collab_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS collab_record_locks (
            lock_key TEXT PRIMARY KEY,
            entity_type TEXT NOT NULL,
            entity_id TEXT NOT NULL,
            holder_client_id TEXT NOT NULL,
            holder_user_name TEXT NOT NULL,
            holder_station TEXT NOT NULL,
            acquired_at_ms INTEGER NOT NULL,
            ttl_ms INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_collab_locks_expiry ON collab_record_locks(acquired_at_ms, ttl_ms);",
    )
}

/// Persists an active lock lease to SQLite.
pub fn sync_lock_to_sqlite(conn: &Connection, lock: &RecordLock) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO collab_record_locks (
            lock_key, entity_type, entity_id, holder_client_id, holder_user_name, holder_station, acquired_at_ms, ttl_ms
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            lock.lock_key,
            lock.entity_type,
            lock.entity_id,
            lock.holder_client_id,
            lock.holder_user_name,
            lock.holder_station,
            lock.acquired_at_ms as i64,
            lock.ttl_ms as i64,
        ],
    )?;
    Ok(())
}

/// Removes a lock from SQLite upon release.
pub fn remove_lock_from_sqlite(conn: &Connection, entity_type: &str, entity_id: &str) -> rusqlite::Result<bool> {
    let key = format!("{}:{}", entity_type, entity_id);
    let rows = conn.execute("DELETE FROM collab_record_locks WHERE lock_key = ?1", params![key])?;
    Ok(rows > 0)
}

/// Loads all unexpired locks from SQLite.
pub fn list_active_db_locks(conn: &Connection, now_ms: u64) -> rusqlite::Result<Vec<RecordLock>> {
    let mut stmt = conn.prepare(
        "SELECT lock_key, entity_type, entity_id, holder_client_id, holder_user_name, holder_station, acquired_at_ms, ttl_ms
         FROM collab_record_locks WHERE (acquired_at_ms + ttl_ms) > ?1",
    )?;
    let rows = stmt.query_map(params![now_ms as i64], |row| {
        let acquired: i64 = row.get(6)?;
        let ttl: i64 = row.get(7)?;
        Ok(RecordLock {
            lock_key: row.get(0)?,
            entity_type: row.get(1)?,
            entity_id: row.get(2)?,
            holder_client_id: row.get(3)?,
            holder_user_name: row.get(4)?,
            holder_station: row.get(5)?,
            acquired_at_ms: acquired as u64,
            ttl_ms: ttl as u64,
        })
    })?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_lock_acquire_and_conflict() {
        let mgr = RecordLockManager::new();
        let lock = mgr.acquire_lock("ticket", "TCK-100", "client-a", "Dimitris", "Counter 1", 5000);
        assert!(lock.is_ok());

        // Concurrent attempt from another client
        let conflict = mgr.acquire_lock("ticket", "TCK-100", "client-b", "Eleni", "Counter 2", 5000);
        assert!(conflict.is_err());
        let err = conflict.unwrap_err();
        assert_eq!(err.current_holder_name, "Dimitris");
        assert_eq!(err.current_holder_station, "Counter 1");

        // Same client re-acquiring succeeds
        let same = mgr.acquire_lock("ticket", "TCK-100", "client-a", "Dimitris", "Counter 1", 5000);
        assert!(same.is_ok());
    }

    #[test]
    fn test_record_lock_release_and_heartbeat() {
        let mgr = RecordLockManager::new();
        let _ = mgr.acquire_lock("invoice", "INV-501", "client-a", "Nikos", "Back Office", 3000);

        assert!(mgr.get_active_lock("invoice", "INV-501").is_some());
        assert!(mgr.heartbeat_lock("invoice", "INV-501", "client-a", 10000));

        // Unauthorized release fails
        assert!(!mgr.release_lock("invoice", "INV-501", "client-b"));
        // Authorized release succeeds
        assert!(mgr.release_lock("invoice", "INV-501", "client-a"));
        assert!(mgr.get_active_lock("invoice", "INV-501").is_none());
    }

    #[test]
    fn test_lww_mutation_reconcile() {
        let mut m1 = LwwMutation::new("record-1", "node-alpha", "State A".to_string());
        m1.timestamp_ms = 1000;

        let mut m2 = LwwMutation::new("record-1", "node-beta", "State B".to_string());
        m2.timestamp_ms = 2000;

        assert_eq!(m1.reconcile(&m2).payload, "State B");
        assert_eq!(m2.reconcile(&m1).payload, "State B");

        // Equal timestamp tie-break by client_id ("node-beta" > "node-alpha")
        let mut m3 = LwwMutation::new("record-1", "node-alpha", "State A".to_string());
        m3.timestamp_ms = 3000;
        let mut m4 = LwwMutation::new("record-1", "node-beta", "State B".to_string());
        m4.timestamp_ms = 3000;
        assert_eq!(m3.reconcile(&m4).payload, "State B");
    }

    #[test]
    fn test_collab_sqlite_schema_and_locks() {
        let conn = Connection::open_in_memory().unwrap();
        ensure_collab_schema(&conn).unwrap();

        let lock = RecordLock::new("wms_zone", "ZONE-A", "client-1", "Alex", "Warehouse", 15000);
        sync_lock_to_sqlite(&conn, &lock).unwrap();

        let active = list_active_db_locks(&conn, lock.acquired_at_ms).unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].holder_user_name, "Alex");

        let removed = remove_lock_from_sqlite(&conn, "wms_zone", "ZONE-A").unwrap();
        assert!(removed);

        let empty = list_active_db_locks(&conn, lock.acquired_at_ms).unwrap();
        assert!(empty.is_empty());
    }
}
