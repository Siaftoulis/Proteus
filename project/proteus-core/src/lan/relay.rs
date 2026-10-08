//! Proteus Core — Zero-Configuration Cloud Relay Tunnel & Authenticated Session Handshake.
//! Enables mobile companions and field technicians outside the LAN to synchronize securely
//! with shop terminals without requiring router port-forwarding or public IP exposure.

use crate::license::mor::{constant_time_eq, hmac_sha256};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

fn current_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn to_hex_string(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelayRole {
    HostTerminal,
    CompanionClient,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelayClientState {
    Disconnected,
    Connecting,
    Connected,
    Relaying,
    Reconnecting,
    Failed(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelayHandshakeStatus {
    Accepted,
    Denied(String),
    WaitingForPeer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelayMessageType {
    SyncOutbox,
    SyncAck,
    LiveQuery,
    LiveResponse,
    Ping,
    Pong,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RelayHandshakeRequest {
    pub session_id: String,
    pub client_id: String,
    pub store_id: String,
    pub role: RelayRole,
    pub timestamp_epoch: u64,
    pub auth_signature: String,
}

impl RelayHandshakeRequest {
    pub fn new(
        session_id: &str,
        client_id: &str,
        store_id: &str,
        role: RelayRole,
        secret: &str,
    ) -> Self {
        let timestamp_epoch = current_epoch_secs();
        let payload = format!("{}:{}:{}:{:?}:{}", session_id, client_id, store_id, role, timestamp_epoch);
        let tag = hmac_sha256(secret.as_bytes(), payload.as_bytes());
        let auth_signature = to_hex_string(&tag);

        Self {
            session_id: session_id.to_string(),
            client_id: client_id.to_string(),
            store_id: store_id.to_string(),
            role,
            timestamp_epoch,
            auth_signature,
        }
    }

    pub fn verify(&self, secret: &str, max_drift_secs: u64) -> Result<(), String> {
        let now = current_epoch_secs();
        if now.saturating_sub(self.timestamp_epoch) > max_drift_secs
            || self.timestamp_epoch.saturating_sub(now) > max_drift_secs
        {
            return Err("Relay handshake timestamp expired or invalid clock drift".into());
        }

        let payload = format!(
            "{}:{}:{}:{:?}:{}",
            self.session_id, self.client_id, self.store_id, self.role, self.timestamp_epoch
        );
        let expected_tag = hmac_sha256(secret.as_bytes(), payload.as_bytes());
        let expected_hex = to_hex_string(&expected_tag);

        if !constant_time_eq(self.auth_signature.as_bytes(), expected_hex.as_bytes()) {
            return Err("Invalid relay handshake authentication signature".into());
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RelayHandshakeResponse {
    pub status: RelayHandshakeStatus,
    pub session_id: String,
    pub server_epoch: u64,
    pub session_ticket: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RelayEnvelope {
    pub envelope_id: String,
    pub session_id: String,
    pub sender_id: String,
    pub recipient_id: String,
    pub msg_type: RelayMessageType,
    pub payload_bytes: Vec<u8>,
    pub timestamp_epoch: u64,
    pub hmac_tag: String,
}

impl RelayEnvelope {
    pub fn new(
        session_id: &str,
        sender_id: &str,
        recipient_id: &str,
        msg_type: RelayMessageType,
        payload_bytes: Vec<u8>,
        secret: &str,
    ) -> Self {
        let envelope_id = uuid::Uuid::now_v7().to_string();
        let timestamp_epoch = current_epoch_secs();
        let auth_data = Self::build_signature_data(
            &envelope_id,
            session_id,
            sender_id,
            recipient_id,
            msg_type,
            timestamp_epoch,
            &payload_bytes,
        );
        let tag = hmac_sha256(secret.as_bytes(), &auth_data);
        let hmac_tag = to_hex_string(&tag);

        Self {
            envelope_id,
            session_id: session_id.to_string(),
            sender_id: sender_id.to_string(),
            recipient_id: recipient_id.to_string(),
            msg_type,
            payload_bytes,
            timestamp_epoch,
            hmac_tag,
        }
    }

    fn build_signature_data(
        envelope_id: &str,
        session_id: &str,
        sender_id: &str,
        recipient_id: &str,
        msg_type: RelayMessageType,
        timestamp_epoch: u64,
        payload_bytes: &[u8],
    ) -> Vec<u8> {
        let header = format!(
            "{}:{}:{}:{}:{:?}:{}:",
            envelope_id, session_id, sender_id, recipient_id, msg_type, timestamp_epoch
        );
        let mut data = header.into_bytes();
        data.extend_from_slice(payload_bytes);
        data
    }

    pub fn verify(&self, secret: &str, max_drift_secs: u64) -> Result<(), String> {
        let now = current_epoch_secs();
        if now.saturating_sub(self.timestamp_epoch) > max_drift_secs
            || self.timestamp_epoch.saturating_sub(now) > max_drift_secs
        {
            return Err("Relay envelope timestamp expired or invalid clock drift".into());
        }

        let auth_data = Self::build_signature_data(
            &self.envelope_id,
            &self.session_id,
            &self.sender_id,
            &self.recipient_id,
            self.msg_type,
            self.timestamp_epoch,
            &self.payload_bytes,
        );
        let expected_tag = hmac_sha256(secret.as_bytes(), &auth_data);
        let expected_hex = to_hex_string(&expected_tag);

        if !constant_time_eq(self.hmac_tag.as_bytes(), expected_hex.as_bytes()) {
            return Err("Tamper detected: Invalid relay envelope HMAC tag".into());
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RelayConfig {
    pub relay_url: String,
    pub enabled: bool,
    pub connect_timeout_ms: u64,
    pub keepalive_interval_secs: u64,
    pub last_connected_epoch: Option<u64>,
}

impl Default for RelayConfig {
    fn default() -> Self {
        Self {
            relay_url: "https://relay.proteus-bos.gr:8443".to_string(),
            enabled: true,
            connect_timeout_ms: 3000,
            keepalive_interval_secs: 15,
            last_connected_epoch: None,
        }
    }
}

pub fn init_relay_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS relay_tunnel_config (
            store_id TEXT PRIMARY KEY,
            relay_url TEXT NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1,
            connect_timeout_ms INTEGER NOT NULL DEFAULT 3000,
            keepalive_interval_secs INTEGER NOT NULL DEFAULT 15,
            last_connected_epoch INTEGER,
            updated_at_epoch INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS relay_audit_logs (
            event_id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            client_id TEXT NOT NULL,
            event_type TEXT NOT NULL,
            timestamp_epoch INTEGER NOT NULL,
            details TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_relay_audit_session ON relay_audit_logs(session_id);
        CREATE INDEX IF NOT EXISTS idx_relay_audit_timestamp ON relay_audit_logs(timestamp_epoch);
        "#,
    )
}

pub fn save_relay_config(
    conn: &Connection,
    store_id: &str,
    config: &RelayConfig,
) -> Result<(), rusqlite::Error> {
    let now = current_epoch_secs();
    conn.execute(
        r#"
        INSERT INTO relay_tunnel_config (
            store_id, relay_url, enabled, connect_timeout_ms, keepalive_interval_secs, last_connected_epoch, updated_at_epoch
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        ON CONFLICT(store_id) DO UPDATE SET
            relay_url = excluded.relay_url,
            enabled = excluded.enabled,
            connect_timeout_ms = excluded.connect_timeout_ms,
            keepalive_interval_secs = excluded.keepalive_interval_secs,
            last_connected_epoch = excluded.last_connected_epoch,
            updated_at_epoch = excluded.updated_at_epoch
        "#,
        params![
            store_id,
            config.relay_url,
            config.enabled as i32,
            config.connect_timeout_ms as i64,
            config.keepalive_interval_secs as i64,
            config.last_connected_epoch.map(|e| e as i64),
            now as i64,
        ],
    )?;
    Ok(())
}

pub fn load_relay_config(
    conn: &Connection,
    store_id: &str,
) -> Result<Option<RelayConfig>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        r#"
        SELECT relay_url, enabled, connect_timeout_ms, keepalive_interval_secs, last_connected_epoch
        FROM relay_tunnel_config
        WHERE store_id = ?1
        "#,
    )?;

    let mut rows = stmt.query(params![store_id])?;
    if let Some(row) = rows.next()? {
        let relay_url: String = row.get(0)?;
        let enabled_int: i32 = row.get(1)?;
        let connect_timeout_ms: i64 = row.get(2)?;
        let keepalive_interval_secs: i64 = row.get(3)?;
        let last_connected_epoch: Option<i64> = row.get(4)?;

        Ok(Some(RelayConfig {
            relay_url,
            enabled: enabled_int != 0,
            connect_timeout_ms: connect_timeout_ms as u64,
            keepalive_interval_secs: keepalive_interval_secs as u64,
            last_connected_epoch: last_connected_epoch.map(|e| e as u64),
        }))
    } else {
        Ok(None)
    }
}

pub fn log_relay_audit_event(
    conn: &Connection,
    session_id: &str,
    client_id: &str,
    event_type: &str,
    details: Option<&str>,
) -> Result<(), rusqlite::Error> {
    let event_id = uuid::Uuid::now_v7().to_string();
    let now = current_epoch_secs();
    conn.execute(
        r#"
        INSERT INTO relay_audit_logs (event_id, session_id, client_id, event_type, timestamp_epoch, details)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        "#,
        params![event_id, session_id, client_id, event_type, now as i64, details],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relay_handshake_creation_and_verification() {
        let secret = "super-secret-pairing-token-12345";
        let req = RelayHandshakeRequest::new(
            "sess-abc-001",
            "mobile-dev-42",
            "store-ath-01",
            RelayRole::CompanionClient,
            secret,
        );

        assert_eq!(req.client_id, "mobile-dev-42");
        assert_eq!(req.role, RelayRole::CompanionClient);
        assert!(req.verify(secret, 300).is_ok());

        // Verify with invalid secret fails
        assert!(req.verify("wrong-secret-token", 300).is_err());
    }

    #[test]
    fn test_relay_handshake_timestamp_drift_rejection() {
        let secret = "secret-token-xyz";
        let mut req = RelayHandshakeRequest::new(
            "sess-abc-002",
            "host-terminal-01",
            "store-ath-01",
            RelayRole::HostTerminal,
            secret,
        );

        // Simulate 400 seconds ago (outside 300s threshold)
        req.timestamp_epoch = current_epoch_secs().saturating_sub(400);
        let err = req.verify(secret, 300);
        assert!(err.is_err());
        assert!(err.unwrap_err().contains("expired"));
    }

    #[test]
    fn test_relay_envelope_authentication_and_tampering() {
        let secret = "relay-session-key-999";
        let payload = b"{\"action\":\"sync_outbox\",\"items\":[1,2,3]}".to_vec();

        let mut env = RelayEnvelope::new(
            "sess-99",
            "mobile-dev-42",
            "terminal-01",
            RelayMessageType::SyncOutbox,
            payload,
            secret,
        );

        assert!(env.verify(secret, 300).is_ok());

        // Tamper with payload
        env.payload_bytes.push(0xFF);
        let err = env.verify(secret, 300);
        assert!(err.is_err());
        assert!(err.unwrap_err().contains("Tamper detected"));
    }

    #[test]
    fn test_relay_schema_save_load_config_and_audit() {
        let conn = Connection::open_in_memory().unwrap();
        init_relay_schema(&conn).unwrap();

        let default_loaded = load_relay_config(&conn, "STORE-TEST").unwrap();
        assert!(default_loaded.is_none());

        let cfg = RelayConfig {
            relay_url: "https://custom-relay.example.com:9443".to_string(),
            enabled: true,
            connect_timeout_ms: 5000,
            keepalive_interval_secs: 20,
            last_connected_epoch: Some(1700000000),
        };

        save_relay_config(&conn, "STORE-TEST", &cfg).unwrap();
        let loaded = load_relay_config(&conn, "STORE-TEST").unwrap().expect("Config must be loaded");

        assert_eq!(loaded.relay_url, "https://custom-relay.example.com:9443");
        assert_eq!(loaded.connect_timeout_ms, 5000);
        assert_eq!(loaded.keepalive_interval_secs, 20);
        assert_eq!(loaded.last_connected_epoch, Some(1700000000));

        // Test audit logging
        log_relay_audit_event(
            &conn,
            "sess-001",
            "client-mobile-01",
            "SESSION_ESTABLISHED",
            Some("Handshake verified successfully via HMAC"),
        )
        .unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM relay_audit_logs WHERE session_id = ?1",
                params!["sess-001"],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }
}
