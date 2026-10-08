//! Notification types, channels, statuses, and validation for Proteus Notifications Gateway.
//! Strict Rule 1 (100% Original Codebase) and Rule 3 (<400 lines).

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NotificationError {
    #[error("Database error: {0}")]
    Sql(#[from] rusqlite::Error),
    #[error("GDPR Art. 6 / Law 4624/2019 violation: Explicit consent required for marketing dispatch")]
    ConsentRequired,
    #[error("Invalid recipient format: {0}")]
    InvalidRecipient(String),
    #[error("Invalid tracking token or signature mismatch")]
    InvalidToken,
    #[error("Gateway error: {0}")]
    GatewayError(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationChannel {
    Sms,
    Viber,
    Email,
}

impl NotificationChannel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sms => "SMS",
            Self::Viber => "VIBER",
            Self::Email => "EMAIL",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "VIBER" => Self::Viber,
            "EMAIL" => Self::Email,
            _ => Self::Sms,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationCategory {
    Transactional,
    Marketing,
}

impl NotificationCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Transactional => "TRANSACTIONAL",
            Self::Marketing => "MARKETING",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "MARKETING" => Self::Marketing,
            _ => Self::Transactional,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationStatus {
    Pending,
    Dispatched,
    Delivered,
    Failed,
}

impl NotificationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "PENDING",
            Self::Dispatched => "DISPATCHED",
            Self::Delivered => "DELIVERED",
            Self::Failed => "FAILED",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "DISPATCHED" => Self::Dispatched,
            "DELIVERED" => Self::Delivered,
            "FAILED" => Self::Failed,
            _ => Self::Pending,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxNotification {
    pub id: String,
    pub channel: NotificationChannel,
    pub category: NotificationCategory,
    pub recipient: String,
    pub subject: Option<String>,
    pub content: String,
    pub status: NotificationStatus,
    pub attempts: u32,
    pub max_retries: u32,
    pub last_error: Option<String>,
    pub scheduled_at: i64,
    pub created_at: i64,
    pub dispatched_at: Option<i64>,
    pub gdpr_consent_verified: bool,
    pub metadata_json: String,
}

/// Validates recipient contact format (E.164 phone or valid email address).
pub fn validate_recipient(channel: NotificationChannel, recipient: &str) -> Result<(), NotificationError> {
    let clean = recipient.trim();
    match channel {
        NotificationChannel::Sms | NotificationChannel::Viber => {
            if clean.len() < 10 || !clean.chars().all(|c| c.is_ascii_digit() || c == '+') {
                return Err(NotificationError::InvalidRecipient(recipient.to_string()));
            }
        }
        NotificationChannel::Email => {
            if !clean.contains('@') || !clean.contains('.') || clean.len() < 5 {
                return Err(NotificationError::InvalidRecipient(recipient.to_string()));
            }
        }
    }
    Ok(())
}
