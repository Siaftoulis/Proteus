// Proteus Core — QR Pairing & Mobile Device Association Engine
// Designed from first principles. Real cryptographic token generation & SQLite authorization.

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

fn current_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub fn hash_pairing_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PairingPayload {
    pub pairing_id: String,
    pub store_id: String,
    pub terminal_id: String,
    pub terminal_name: String,
    pub lan_ip: String,
    pub port: u16,
    pub auth_token: String,
    pub cert_fingerprint: String,
    pub expires_at_epoch: u64,
}

impl PairingPayload {
    pub fn new(
        store_id: &str,
        terminal_id: &str,
        terminal_name: &str,
        lan_ip: &str,
        port: u16,
        cert_fingerprint: &str,
        ttl_secs: u64,
    ) -> Self {
        let pairing_id = uuid::Uuid::now_v7().to_string();
        let auth_token = format!("{:x}", Sha256::digest(format!("{}-{}-{}", pairing_id, lan_ip, current_epoch_secs()).as_bytes()));
        let expires_at_epoch = current_epoch_secs() + ttl_secs;

        Self {
            pairing_id,
            store_id: store_id.to_string(),
            terminal_id: terminal_id.to_string(),
            terminal_name: terminal_name.to_string(),
            lan_ip: lan_ip.to_string(),
            port,
            auth_token,
            cert_fingerprint: cert_fingerprint.to_string(),
            expires_at_epoch,
        }
    }

    pub fn is_expired(&self, current_epoch: u64) -> bool {
        current_epoch > self.expires_at_epoch
    }

    pub fn to_qr_string(&self) -> Result<String, serde_json::Error> {
        let json = serde_json::to_string(self)?;
        Ok(format!("proteus-pair://{}", json))
    }

    pub fn from_qr_string(raw: &str) -> Result<Self, String> {
        let trimmed = raw.trim();
        let json_str = if let Some(stripped) = trimmed.strip_prefix("proteus-pair://") {
            stripped
        } else {
            trimmed
        };

        let payload: Self = serde_json::from_str(json_str)
            .map_err(|e| format!("Invalid pairing payload JSON: {}", e))?;

        if payload.is_expired(current_epoch_secs()) {
            return Err("Pairing token has expired".into());
        }

        Ok(payload)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PairedDevice {
    pub device_id: String,
    pub device_name: String,
    pub terminal_id: String,
    pub auth_token_hash: String,
    pub allowed_roles: Vec<String>,
    pub paired_at_epoch: u64,
    pub last_seen_epoch: u64,
    pub is_active: bool,
}

pub fn init_pairing_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS paired_mobile_devices (
            device_id TEXT PRIMARY KEY,
            device_name TEXT NOT NULL,
            terminal_id TEXT NOT NULL,
            auth_token_hash TEXT NOT NULL,
            allowed_roles TEXT NOT NULL,
            paired_at_epoch INTEGER NOT NULL,
            last_seen_epoch INTEGER NOT NULL,
            is_active INTEGER NOT NULL DEFAULT 1
        );

        CREATE TABLE IF NOT EXISTS pairing_audit_logs (
            event_id TEXT PRIMARY KEY,
            device_id TEXT NOT NULL,
            action TEXT NOT NULL,
            timestamp_epoch INTEGER NOT NULL,
            details TEXT
        );

        CREATE INDEX IF NOT EXISTS idx_paired_mobile_active ON paired_mobile_devices(is_active);
        CREATE INDEX IF NOT EXISTS idx_paired_mobile_terminal ON paired_mobile_devices(terminal_id);
        "#,
    )
}

pub fn register_paired_device(
    conn: &Connection,
    device_id: &str,
    device_name: &str,
    terminal_id: &str,
    raw_auth_token: &str,
    roles: &[String],
) -> Result<PairedDevice, rusqlite::Error> {
    let now = current_epoch_secs();
    let token_hash = hash_pairing_token(raw_auth_token);
    let roles_json = serde_json::to_string(roles).unwrap_or_else(|_| "[]".into());

    conn.execute(
        r#"
        INSERT INTO paired_mobile_devices (
            device_id, device_name, terminal_id, auth_token_hash, allowed_roles, paired_at_epoch, last_seen_epoch, is_active
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1)
        ON CONFLICT(device_id) DO UPDATE SET
            device_name = excluded.device_name,
            terminal_id = excluded.terminal_id,
            auth_token_hash = excluded.auth_token_hash,
            allowed_roles = excluded.allowed_roles,
            last_seen_epoch = excluded.last_seen_epoch,
            is_active = 1
        "#,
        params![device_id, device_name, terminal_id, token_hash, roles_json, now, now],
    )?;

    let event_id = uuid::Uuid::now_v7().to_string();
    let _ = conn.execute(
        "INSERT INTO pairing_audit_logs (event_id, device_id, action, timestamp_epoch, details) VALUES (?1, ?2, 'registered', ?3, ?4)",
        params![event_id, device_id, now, format!("Paired with terminal {}", terminal_id)],
    );

    Ok(PairedDevice {
        device_id: device_id.to_string(),
        device_name: device_name.to_string(),
        terminal_id: terminal_id.to_string(),
        auth_token_hash: token_hash,
        allowed_roles: roles.to_vec(),
        paired_at_epoch: now,
        last_seen_epoch: now,
        is_active: true,
    })
}

pub fn verify_device_token(
    conn: &Connection,
    device_id: &str,
    raw_auth_token: &str,
) -> Result<bool, rusqlite::Error> {
    let expected_hash = hash_pairing_token(raw_auth_token);
    let mut stmt = conn.prepare(
        "SELECT auth_token_hash, is_active FROM paired_mobile_devices WHERE device_id = ?1",
    )?;

    let mut rows = stmt.query(params![device_id])?;
    if let Some(row) = rows.next()? {
        let stored_hash: String = row.get(0)?;
        let is_active: bool = row.get(1)?;
        if is_active && stored_hash == expected_hash {
            let now = current_epoch_secs();
            let _ = conn.execute(
                "UPDATE paired_mobile_devices SET last_seen_epoch = ?1 WHERE device_id = ?2",
                params![now, device_id],
            );
            return Ok(true);
        }
    }

    Ok(false)
}

pub fn revoke_paired_device(conn: &Connection, device_id: &str) -> Result<bool, rusqlite::Error> {
    let now = current_epoch_secs();
    let rows_affected = conn.execute(
        "UPDATE paired_mobile_devices SET is_active = 0 WHERE device_id = ?1",
        params![device_id],
    )?;

    if rows_affected > 0 {
        let event_id = uuid::Uuid::now_v7().to_string();
        let _ = conn.execute(
            "INSERT INTO pairing_audit_logs (event_id, device_id, action, timestamp_epoch, details) VALUES (?1, ?2, 'revoked', ?3, NULL)",
            params![event_id, device_id, now],
        );
    }

    Ok(rows_affected > 0)
}

pub fn list_active_paired_devices(conn: &Connection) -> Result<Vec<PairedDevice>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        r#"
        SELECT device_id, device_name, terminal_id, auth_token_hash, allowed_roles, paired_at_epoch, last_seen_epoch, is_active
        FROM paired_mobile_devices
        WHERE is_active = 1
        ORDER BY last_seen_epoch DESC
        "#,
    )?;

    let iter = stmt.query_map([], |row| {
        let roles_json: String = row.get(4)?;
        let allowed_roles = serde_json::from_str(&roles_json).unwrap_or_default();
        Ok(PairedDevice {
            device_id: row.get(0)?,
            device_name: row.get(1)?,
            terminal_id: row.get(2)?,
            auth_token_hash: row.get(3)?,
            allowed_roles,
            paired_at_epoch: row.get(5)?,
            last_seen_epoch: row.get(6)?,
            is_active: row.get(7)?,
        })
    })?;

    let mut list = Vec::new();
    for res in iter {
        list.push(res?);
    }
    Ok(list)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pairing_payload_serialization_roundtrip() {
        let payload = PairingPayload::new(
            "store_heraklion_01",
            "pos_lane_01",
            "Main Terminal",
            "192.168.1.150",
            7443,
            "sha256:4a3b2c1d0e...",
            600,
        );

        assert!(!payload.is_expired(payload.expires_at_epoch - 10));
        assert!(payload.is_expired(payload.expires_at_epoch + 1));

        let qr_str = payload.to_qr_string().expect("QR string generation failed");
        assert!(qr_str.starts_with("proteus-pair://"));

        let decoded = PairingPayload::from_qr_string(&qr_str).expect("Decoding failed");
        assert_eq!(payload.terminal_id, decoded.terminal_id);
        assert_eq!(payload.lan_ip, decoded.lan_ip);
        assert_eq!(payload.port, decoded.port);
        assert_eq!(payload.auth_token, decoded.auth_token);
    }

    #[test]
    fn test_pairing_expiration_check() {
        let mut payload = PairingPayload::new(
            "store_01",
            "terminal_01",
            "Terminal",
            "127.0.0.1",
            7443,
            "dummy_cert",
            10,
        );
        payload.expires_at_epoch = current_epoch_secs().saturating_sub(5); // expired 5 seconds ago

        let qr_str = payload.to_qr_string().unwrap();
        let res = PairingPayload::from_qr_string(&qr_str);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err(), "Pairing token has expired");
    }

    #[test]
    fn test_device_registration_and_token_verification() {
        let conn = Connection::open_in_memory().unwrap();
        init_pairing_schema(&conn).unwrap();

        let device_id = "pixel_8_pro_tech1";
        let raw_token = "secret_auth_token_999";
        let roles = vec!["technician".into(), "van_sales".into()];

        let device = register_paired_device(
            &conn,
            device_id,
            "Nikos Phone",
            "terminal_01",
            raw_token,
            &roles,
        ).unwrap();

        assert_eq!(device.device_id, device_id);
        assert_eq!(device.allowed_roles.len(), 2);

        // Verification with valid token
        let is_valid = verify_device_token(&conn, device_id, raw_token).unwrap();
        assert!(is_valid);

        // Verification with invalid token
        let is_invalid = verify_device_token(&conn, device_id, "wrong_token").unwrap();
        assert!(!is_invalid);

        // List active
        let active = list_active_paired_devices(&conn).unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].device_id, device_id);

        // Revoke
        let revoked = revoke_paired_device(&conn, device_id).unwrap();
        assert!(revoked);

        // Verification after revoking must fail
        let after_revoke = verify_device_token(&conn, device_id, raw_token).unwrap();
        assert!(!after_revoke);

        let active_after = list_active_paired_devices(&conn).unwrap();
        assert!(active_after.is_empty());
    }
}
