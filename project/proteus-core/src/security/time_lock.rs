//! Monotonic Clock Validation & Anti-Rollback High-Water Mark Engine (Master Problem Audit P3).
//! Prevents offline lease bypass and timestamp tampering by tracking a monotonic high-water mark
//! in a secure SQLite table. When clock rollback is detected, enters Grace Read-Only Mode.

use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

pub const DEFAULT_TOLERANCE_SECS: i64 = 60; // 60s tolerance for normal NTP drift corrections

#[derive(Debug, Error, PartialEq)]
pub enum TimeLockError {
    #[error("Clock rollback detected! Current: {current}, High-Water: {high_water} (Delta: {delta_secs}s). Write operations locked.")]
    ClockRollbackRejected {
        current: i64,
        high_water: i64,
        delta_secs: i64,
    },
    #[error("Invalid or corrupted time-unlock token")]
    InvalidUnlockToken,
    #[error("Time-unlock token has expired")]
    TokenExpired,
    #[error("Machine ID mismatch for unlock token")]
    MachineMismatch,
    #[error("Database error: {0}")]
    Database(String),
}

impl From<rusqlite::Error> for TimeLockError {
    fn from(err: rusqlite::Error) -> Self {
        TimeLockError::Database(err.to_string())
    }
}

/// Operational state of the temporal integrity guard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeLockStatus {
    Synchronized,
    TamperLocked,
    Unlocked,
}

impl TimeLockStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Synchronized => "SYNCHRONIZED",
            Self::TamperLocked => "TAMPER_LOCKED",
            Self::Unlocked => "UNLOCKED",
        }
    }
}

/// Persistent temporal integrity state record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeIntegrityRecord {
    pub high_water_secs: i64,
    pub high_water_iso: String,
    pub total_events: u64,
    pub tamper_locked: bool,
    pub lock_reason: Option<String>,
    pub updated_at: String,
}

/// Cryptographically signed administrative time-unlock token.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeUnlockToken {
    pub machine_id: String,
    pub valid_until_epoch_secs: i64,
    pub signature: String,
}

impl TimeUnlockToken {
    pub fn generate(machine_id: &str, valid_until_epoch_secs: i64, secret_key: &str) -> String {
        let payload = format!("UNLOCK:{machine_id}:{valid_until_epoch_secs}:{secret_key}");
        let mut hasher = Sha256::new();
        hasher.update(payload.as_bytes());
        let signature = format!("{:x}", hasher.finalize());
        format!("tok.{machine_id}.{valid_until_epoch_secs}.{signature}")
    }

    pub fn parse_and_verify(
        token_str: &str,
        expected_machine_id: &str,
        current_time_secs: i64,
        secret_key: &str,
    ) -> Result<Self, TimeLockError> {
        let parts: Vec<&str> = token_str.split('.').collect();
        if parts.len() != 4 || parts[0] != "tok" {
            return Err(TimeLockError::InvalidUnlockToken);
        }

        let machine_id = parts[1];
        if machine_id != expected_machine_id {
            return Err(TimeLockError::MachineMismatch);
        }

        let valid_until: i64 = parts[2].parse().map_err(|_| TimeLockError::InvalidUnlockToken)?;
        if current_time_secs > valid_until {
            return Err(TimeLockError::TokenExpired);
        }

        let expected_signature = {
            let payload = format!("UNLOCK:{machine_id}:{valid_until}:{secret_key}");
            let mut hasher = Sha256::new();
            hasher.update(payload.as_bytes());
            format!("{:x}", hasher.finalize())
        };

        if parts[3] != expected_signature {
            return Err(TimeLockError::InvalidUnlockToken);
        }

        Ok(Self {
            machine_id: machine_id.to_string(),
            valid_until_epoch_secs: valid_until,
            signature: parts[3].to_string(),
        })
    }
}

pub struct TimeIntegrityGuard;

impl TimeIntegrityGuard {
    pub fn init_schema(conn: &Connection) -> Result<(), TimeLockError> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS system_time_highwater (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                high_water_secs INTEGER NOT NULL,
                high_water_iso TEXT NOT NULL,
                total_events INTEGER NOT NULL DEFAULT 0,
                tamper_locked INTEGER NOT NULL DEFAULT 0,
                lock_reason TEXT,
                updated_at TEXT NOT NULL
            );",
        )?;
        Ok(())
    }

    pub fn get_record(conn: &Connection) -> Result<Option<TimeIntegrityRecord>, TimeLockError> {
        let mut stmt = conn.prepare("SELECT high_water_secs, high_water_iso, total_events, tamper_locked, lock_reason, updated_at FROM system_time_highwater WHERE id = 1")?;
        let mut rows = stmt.query(params![])?;
        if let Some(row) = rows.next()? {
            Ok(Some(TimeIntegrityRecord {
                high_water_secs: row.get(0)?,
                high_water_iso: row.get(1)?,
                total_events: row.get(2)?,
                tamper_locked: row.get::<_, i64>(3)? == 1,
                lock_reason: row.get(4)?,
                updated_at: row.get(5)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Checks current clock against persistent high-water mark.
    /// Transitions to TamperLocked if clock has rolled backward beyond tolerance.
    pub fn check_clock(conn: &Connection, current_secs: i64) -> Result<TimeLockStatus, TimeLockError> {
        Self::check_clock_with_tolerance(conn, current_secs, DEFAULT_TOLERANCE_SECS)
    }

    pub fn check_clock_with_tolerance(
        conn: &Connection,
        current_secs: i64,
        tolerance_secs: i64,
    ) -> Result<TimeLockStatus, TimeLockError> {
        let Some(record) = Self::get_record(conn)? else {
            return Ok(TimeLockStatus::Synchronized);
        };

        if record.tamper_locked {
            return Ok(TimeLockStatus::TamperLocked);
        }

        if current_secs < record.high_water_secs - tolerance_secs {
            let delta = record.high_water_secs - current_secs;
            let reason = format!("Detected clock rollback of {delta}s (Current: {current_secs}, High-Water: {})", record.high_water_secs);
            let now_iso = Utc::now().to_rfc3339();

            conn.execute(
                r#"
                UPDATE system_time_highwater
                SET tamper_locked = 1, lock_reason = ?1, updated_at = ?2
                WHERE id = 1
                "#,
                params![reason, now_iso],
            )?;

            return Ok(TimeLockStatus::TamperLocked);
        }

        Ok(TimeLockStatus::Synchronized)
    }

    /// Records a business mutation timestamp, advancing the monotonic high-water mark.
    /// Fails immediately if clock is tamper-locked.
    pub fn record_event(conn: &Connection, current_secs: i64) -> Result<(), TimeLockError> {
        let status = Self::check_clock(conn, current_secs)?;
        if status == TimeLockStatus::TamperLocked {
            let rec = Self::get_record(conn)?.unwrap_or(TimeIntegrityRecord {
                high_water_secs: current_secs,
                high_water_iso: "".into(),
                total_events: 0,
                tamper_locked: true,
                lock_reason: None,
                updated_at: "".into(),
            });
            return Err(TimeLockError::ClockRollbackRejected {
                current: current_secs,
                high_water: rec.high_water_secs,
                delta_secs: rec.high_water_secs.saturating_sub(current_secs),
            });
        }

        let now_iso = Utc::now().to_rfc3339();
        conn.execute(
            r#"
            INSERT INTO system_time_highwater (id, high_water_secs, high_water_iso, total_events, tamper_locked, lock_reason, updated_at)
            VALUES (1, ?1, ?2, 1, 0, NULL, ?2)
            ON CONFLICT(id) DO UPDATE SET
                high_water_secs = MAX(high_water_secs, ?1),
                high_water_iso = CASE WHEN ?1 >= high_water_secs THEN ?2 ELSE high_water_iso END,
                total_events = total_events + 1,
                updated_at = ?2
            WHERE tamper_locked = 0;
            "#,
            params![current_secs, now_iso],
        )?;
        Ok(())
    }

    /// Unlocks a tamper-locked system using an authentic administrative token.
    pub fn apply_unlock_token(
        conn: &Connection,
        token_str: &str,
        machine_id: &str,
        current_secs: i64,
        secret_key: &str,
    ) -> Result<(), TimeLockError> {
        TimeUnlockToken::parse_and_verify(token_str, machine_id, current_secs, secret_key)?;

        let now_iso = Utc::now().to_rfc3339();
        conn.execute(
            r#"
            UPDATE system_time_highwater
            SET tamper_locked = 0,
                lock_reason = NULL,
                high_water_secs = ?1,
                high_water_iso = ?2,
                updated_at = ?2
            WHERE id = 1
            "#,
            params![current_secs, now_iso],
        )?;
        Ok(())
    }

    /// Authoritatively resets time high-water mark via authenticated remote hub sync.
    pub fn apply_remote_ntp_sync(
        conn: &Connection,
        authoritative_secs: i64,
    ) -> Result<(), TimeLockError> {
        let now_iso = Utc::now().to_rfc3339();
        conn.execute(
            r#"
            INSERT INTO system_time_highwater (id, high_water_secs, high_water_iso, total_events, tamper_locked, lock_reason, updated_at)
            VALUES (1, ?1, ?2, 1, 0, NULL, ?2)
            ON CONFLICT(id) DO UPDATE SET
                high_water_secs = ?1,
                high_water_iso = ?2,
                tamper_locked = 0,
                lock_reason = NULL,
                updated_at = ?2
            "#,
            params![authoritative_secs, now_iso],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monotonic_high_water_forward_progress() {
        let conn = Connection::open_in_memory().expect("sqlite in memory");
        TimeIntegrityGuard::init_schema(&conn).expect("init schema");

        let t1 = 1790000000;
        TimeIntegrityGuard::record_event(&conn, t1).expect("t1 record");

        let rec = TimeIntegrityGuard::get_record(&conn).expect("get").expect("exists");
        assert_eq!(rec.high_water_secs, t1);
        assert_eq!(rec.total_events, 1);
        assert!(!rec.tamper_locked);

        // Advance forward
        let t2 = 1790000500;
        TimeIntegrityGuard::record_event(&conn, t2).expect("t2 record");
        let rec2 = TimeIntegrityGuard::get_record(&conn).expect("get").expect("exists");
        assert_eq!(rec2.high_water_secs, t2);
        assert_eq!(rec2.total_events, 2);
    }

    #[test]
    fn test_clock_rollback_detection_and_lock() {
        let conn = Connection::open_in_memory().expect("sqlite in memory");
        TimeIntegrityGuard::init_schema(&conn).expect("init schema");

        let t_valid = 1790005000;
        TimeIntegrityGuard::record_event(&conn, t_valid).expect("valid event");

        // Roll back clock by 3 days (259,200 seconds)
        let t_rollback = t_valid - 259200;
        let status = TimeIntegrityGuard::check_clock(&conn, t_rollback).expect("check");
        assert_eq!(status, TimeLockStatus::TamperLocked);

        // Attempting to record should now fail
        let res = TimeIntegrityGuard::record_event(&conn, t_rollback);
        assert!(matches!(res, Err(TimeLockError::ClockRollbackRejected { .. })));

        let rec = TimeIntegrityGuard::get_record(&conn).expect("get").expect("exists");
        assert!(rec.tamper_locked);
        assert!(rec.lock_reason.unwrap().contains("Detected clock rollback"));
    }

    #[test]
    fn test_minor_ntp_jitter_within_tolerance_allowed() {
        let conn = Connection::open_in_memory().expect("sqlite in memory");
        TimeIntegrityGuard::init_schema(&conn).expect("init schema");

        let t_now = 1790000000;
        TimeIntegrityGuard::record_event(&conn, t_now).expect("record");

        // Small 15s jitter backward (within 60s tolerance)
        let t_jitter = t_now - 15;
        let status = TimeIntegrityGuard::check_clock(&conn, t_jitter).expect("check");
        assert_eq!(status, TimeLockStatus::Synchronized);
    }

    #[test]
    fn test_unlock_token_generation_and_recovery() {
        let conn = Connection::open_in_memory().expect("sqlite in memory");
        TimeIntegrityGuard::init_schema(&conn).expect("init schema");

        let secret = "proteus_sovereign_secret_key";
        let machine_id = "mach_win_dell_789";

        let t_valid = 1790005000;
        TimeIntegrityGuard::record_event(&conn, t_valid).expect("record");

        // Rollback triggers lock
        let t_rollback = t_valid - 10000;
        let _ = TimeIntegrityGuard::check_clock(&conn, t_rollback);
        assert!(TimeIntegrityGuard::get_record(&conn).unwrap().unwrap().tamper_locked);

        // Generate token valid until t_valid + 3600
        let token = TimeUnlockToken::generate(machine_id, t_valid + 3600, secret);

        // Apply unlock token with correct machine ID
        TimeIntegrityGuard::apply_unlock_token(&conn, &token, machine_id, t_rollback, secret)
            .expect("unlock must succeed");

        let rec = TimeIntegrityGuard::get_record(&conn).expect("get").expect("exists");
        assert!(!rec.tamper_locked);

        // Now event recording works again
        TimeIntegrityGuard::record_event(&conn, t_rollback + 10).expect("record succeeds");
    }

    #[test]
    fn test_remote_ntp_sync_recovery() {
        let conn = Connection::open_in_memory().expect("sqlite in memory");
        TimeIntegrityGuard::init_schema(&conn).expect("init schema");

        let t1 = 1790000000;
        TimeIntegrityGuard::record_event(&conn, t1).expect("record");

        // Rollback
        let _ = TimeIntegrityGuard::check_clock(&conn, t1 - 5000);
        assert!(TimeIntegrityGuard::get_record(&conn).unwrap().unwrap().tamper_locked);

        // Authoritative NTP sync
        let authoritative = 1790001000;
        TimeIntegrityGuard::apply_remote_ntp_sync(&conn, authoritative).expect("ntp sync");

        let rec = TimeIntegrityGuard::get_record(&conn).unwrap().unwrap();
        assert!(!rec.tamper_locked);
        assert_eq!(rec.high_water_secs, authoritative);
    }
}
