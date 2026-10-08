//! Data types and error definitions for Ergani II Work Card.
//! 100% Original Implementation. Zero third-party boilerplate.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum WorkCardError {
    #[error("Employee not found for provided badge or PIN")]
    EmployeeNotFound,
    #[error("Invalid PIN code for employee {0}")]
    InvalidPin(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Invalid event sequence: employee is already clocked in")]
    AlreadyClockedIn,
    #[error("Invalid event sequence: employee is not clocked in")]
    NotClockedIn,
    #[error("Merkle chain verification failed at event {0}: hash mismatch")]
    MerkleChainCorrupted(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkCardEventType {
    ClockIn,
    ClockOut,
    BreakStart,
    BreakEnd,
}

impl WorkCardEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ClockIn => "CLOCK_IN",
            Self::ClockOut => "CLOCK_OUT",
            Self::BreakStart => "BREAK_START",
            Self::BreakEnd => "BREAK_END",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "CLOCK_IN" => Some(Self::ClockIn),
            "CLOCK_OUT" => Some(Self::ClockOut),
            "BREAK_START" => Some(Self::BreakStart),
            "BREAK_END" => Some(Self::BreakEnd),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmployeeProfile {
    pub employee_id: String,
    pub afm: String,
    pub amka: String,
    pub full_name: String,
    pub job_title: String,
    pub pin_code: String,
    pub qr_badge_token: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkCardEvent {
    pub event_id: String,
    pub employee_id: String,
    pub event_type: WorkCardEventType,
    pub timestamp_utc: i64,
    pub local_time_str: String,
    pub is_offline_fallback: bool,
    pub outage_reason: Option<String>,
    pub merkle_hash: String,
    pub ergani_submission_id: Option<String>,
    pub sync_status: String, // PENDING, PENDING_OUTAGE_SYNC, SYNCED, FAILED
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErganiSyncSummary {
    pub total_queued: usize,
    pub synced_count: usize,
    pub failed_count: usize,
    pub latest_merkle_root: String,
}
