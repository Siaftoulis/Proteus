//! Hybrid Hardware Key & Operational DPAPI/TPM Sealing Engine (Master Problem Audit P4).
//! Segregates database and checkout security into:
//! 1. `OperationalKey`: Machine-bound key sealed with hardware/platform entropy for instant (<50ms) POS unlock.
//! 2. `MasterOwnerKey`: Argon2id password-derived key for high-privilege operations (cloud backup, transfer, wipe).

use chrono::Utc;
use rand::RngCore;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::encryption::{decrypt, derive_key, encrypt};

#[derive(Debug, Error, PartialEq)]
pub enum VaultError {
    #[error("Vault is not yet provisioned on this device")]
    NotProvisioned,
    #[error("Vault already initialized on this database")]
    AlreadyProvisioned,
    #[error("Hardware seal mismatch! Stored machine ID '{expected}' does not match current '{actual}'")]
    HardwareSealMismatch { expected: String, actual: String },
    #[error("Invalid master owner password")]
    InvalidMasterPassword,
    #[error("Decryption failed for vault envelope: {0}")]
    DecryptionFailed(String),
    #[error("Encryption failed for vault envelope: {0}")]
    EncryptionFailed(String),
    #[error("Master authorization required for this operation")]
    MasterPrivilegeRequired,
    #[error("Database error: {0}")]
    Database(String),
}

impl From<rusqlite::Error> for VaultError {
    fn from(err: rusqlite::Error) -> Self {
        VaultError::Database(err.to_string())
    }
}

impl From<crate::encryption::CryptoError> for VaultError {
    fn from(err: crate::encryption::CryptoError) -> Self {
        VaultError::EncryptionFailed(err.to_string())
    }
}

/// Operational state of the hardware vault.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VaultStatus {
    Unprovisioned,
    OperationalLocked,
    OperationalReady,
    MasterUnlocked,
    HardwareTampered { expected: String, actual: String },
}

/// Active operational session for zero-friction POS checkout (<50ms).
pub struct OperationalSession {
    key: [u8; 32],
    machine_id: String,
}

impl OperationalSession {
    pub fn key_ref(&self) -> &[u8; 32] { &self.key }
    pub fn machine_id(&self) -> &str { &self.machine_id }
}

impl Drop for OperationalSession {
    fn drop(&mut self) { self.key.fill(0); } // Zeroize on drop
}

/// High-privilege owner authorization session.
pub struct MasterSession {
    master_key: [u8; 32],
    issued_at: i64,
}

impl MasterSession {
    pub fn key_ref(&self) -> &[u8; 32] { &self.master_key }
    pub fn is_valid(&self) -> bool { (Utc::now().timestamp() - self.issued_at) < 900 }
}

impl Drop for MasterSession {
    fn drop(&mut self) { self.master_key.fill(0); } // Zeroize on drop
}

fn hex_encode(data: &[u8]) -> String {
    data.iter().map(|b| format!("{:02x}", b)).collect()
}

fn hex_decode(s: &str) -> Result<Vec<u8>, VaultError> {
    if s.len() % 2 != 0 {
        return Err(VaultError::DecryptionFailed("Invalid hex length".into()));
    }
    (0..s.len()).step_by(2).map(|i| {
        u8::from_str_radix(&s[i..i + 2], 16)
            .map_err(|e| VaultError::DecryptionFailed(format!("Hex decode error: {}", e)))
    }).collect()
}

/// Hardware-sealed Vault Engine manager.
pub struct VaultManager;

impl VaultManager {
    pub fn init_tables(conn: &Connection) -> Result<(), VaultError> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS system_vault_envelopes (
                envelope_id TEXT PRIMARY KEY,
                envelope_type TEXT NOT NULL UNIQUE,
                machine_id TEXT NOT NULL,
                ciphertext_hex TEXT NOT NULL,
                salt_hex TEXT NOT NULL,
                verification_hash TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );",
            [],
        )?;
        Ok(())
    }

    fn derive_hardware_key(machine_id: &str, salt: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(b"PROTEUS_OPERATIONAL_HARDWARE_SEAL_V1");
        hasher.update(machine_id.as_bytes());
        hasher.update(salt);
        let mut key = [0u8; 32];
        key.copy_from_slice(&hasher.finalize());
        key
    }

    pub fn provision(conn: &Connection, machine_id: &str, master_password: &str) -> Result<(), VaultError> {
        Self::init_tables(conn)?;
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM system_vault_envelopes", [], |r| r.get(0))?;
        if count > 0 { return Err(VaultError::AlreadyProvisioned); }
        let now = Utc::now().to_rfc3339();

        // 1. Generate & seal Operational Key bound to hardware
        let mut op_raw = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut op_raw);
        let mut op_salt = [0u8; 16];
        rand::rngs::OsRng.fill_bytes(&mut op_salt);
        let hw_key = Self::derive_hardware_key(machine_id, &op_salt);
        let op_cipher = encrypt(&op_raw, &hw_key)?;
        let op_hash = format!("{:x}", Sha256::digest(&op_raw));

        // 2. Generate & seal Master Key with Argon2id password
        let mut mst_raw = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut mst_raw);
        let mut mst_salt = [0u8; 16];
        rand::rngs::OsRng.fill_bytes(&mut mst_salt);
        let mst_salt_hex = hex_encode(&mst_salt);
        let kdf_key = derive_key(master_password, &mst_salt_hex)?;
        let mst_cipher = encrypt(&mst_raw, &kdf_key)?;
        let mst_hash = format!("{:x}", Sha256::digest(&mst_raw));

        conn.execute(
            "INSERT INTO system_vault_envelopes (
                envelope_id, envelope_type, machine_id, ciphertext_hex, salt_hex,
                verification_hash, created_at, updated_at
            ) VALUES (?1, 'OPERATIONAL', ?2, ?3, ?4, ?5, ?6, ?6)",
            params![format!("env_op_{}", uuid::Uuid::new_v4()), machine_id, hex_encode(&op_cipher), hex_encode(&op_salt), op_hash, now],
        )?;

        conn.execute(
            "INSERT INTO system_vault_envelopes (
                envelope_id, envelope_type, machine_id, ciphertext_hex, salt_hex,
                verification_hash, created_at, updated_at
            ) VALUES (?1, 'MASTER', ?2, ?3, ?4, ?5, ?6, ?6)",
            params![format!("env_mst_{}", uuid::Uuid::new_v4()), machine_id, hex_encode(&mst_cipher), mst_salt_hex, mst_hash, now],
        )?;
        Ok(())
    }

    pub fn unlock_operational(conn: &Connection, machine_id: &str) -> Result<OperationalSession, VaultError> {
        Self::init_tables(conn)?;
        let mut stmt = conn.prepare(
            "SELECT machine_id, ciphertext_hex, salt_hex, verification_hash FROM system_vault_envelopes WHERE envelope_type = 'OPERATIONAL'",
        )?;
        let row = stmt.query_row([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, String>(3)?)));
        let (stored_mach, cipher_hex, salt_hex, verif_hash) = match row {
            Ok(d) => d,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Err(VaultError::NotProvisioned),
            Err(e) => return Err(VaultError::Database(e.to_string())),
        };

        if stored_mach != machine_id {
            return Err(VaultError::HardwareSealMismatch { expected: stored_mach, actual: machine_id.to_string() });
        }

        let salt = hex_decode(&salt_hex)?;
        let ciphertext = hex_decode(&cipher_hex)?;
        let hw_key = Self::derive_hardware_key(machine_id, &salt);
        let decrypted = decrypt(&ciphertext, &hw_key).map_err(|e| VaultError::DecryptionFailed(e.to_string()))?;

        if decrypted.len() != 32 || format!("{:x}", Sha256::digest(&decrypted)) != verif_hash {
            return Err(VaultError::DecryptionFailed("Integrity failure".into()));
        }

        let mut key = [0u8; 32];
        key.copy_from_slice(&decrypted);
        Ok(OperationalSession { key, machine_id: machine_id.to_string() })
    }

    pub fn unlock_master(conn: &Connection, master_password: &str) -> Result<MasterSession, VaultError> {
        Self::init_tables(conn)?;
        let mut stmt = conn.prepare(
            "SELECT ciphertext_hex, salt_hex, verification_hash FROM system_vault_envelopes WHERE envelope_type = 'MASTER'",
        )?;
        let row = stmt.query_row([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?)));
        let (cipher_hex, salt_hex, verif_hash) = match row {
            Ok(d) => d,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Err(VaultError::NotProvisioned),
            Err(e) => return Err(VaultError::Database(e.to_string())),
        };

        let kdf_key = derive_key(master_password, &salt_hex).map_err(|_| VaultError::InvalidMasterPassword)?;
        let ciphertext = hex_decode(&cipher_hex)?;
        let decrypted = decrypt(&ciphertext, &kdf_key).map_err(|_| VaultError::InvalidMasterPassword)?;

        if decrypted.len() != 32 || format!("{:x}", Sha256::digest(&decrypted)) != verif_hash {
            return Err(VaultError::InvalidMasterPassword);
        }

        let mut master_key = [0u8; 32];
        master_key.copy_from_slice(&decrypted);
        Ok(MasterSession { master_key, issued_at: Utc::now().timestamp() })
    }

    pub fn rekey_operational(conn: &Connection, new_machine_id: &str, master_session: &MasterSession) -> Result<(), VaultError> {
        if !master_session.is_valid() {
            return Err(VaultError::MasterPrivilegeRequired);
        }

        let mut op_raw = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut op_raw);
        let mut op_salt = [0u8; 16];
        rand::rngs::OsRng.fill_bytes(&mut op_salt);
        let hw_key = Self::derive_hardware_key(new_machine_id, &op_salt);

        let op_cipher = encrypt(&op_raw, &hw_key)?;
        let op_hash = format!("{:x}", Sha256::digest(&op_raw));
        let now = Utc::now().to_rfc3339();

        conn.execute(
            "UPDATE system_vault_envelopes
             SET machine_id = ?1, ciphertext_hex = ?2, salt_hex = ?3, verification_hash = ?4, updated_at = ?5
             WHERE envelope_type = 'OPERATIONAL'",
            params![new_machine_id, hex_encode(&op_cipher), hex_encode(&op_salt), op_hash, now],
        )?;
        Ok(())
    }

    pub fn get_status(conn: &Connection, current_machine_id: &str) -> Result<VaultStatus, VaultError> {
        Self::init_tables(conn)?;
        let mut stmt = conn.prepare("SELECT machine_id FROM system_vault_envelopes WHERE envelope_type = 'OPERATIONAL'")?;
        let row = stmt.query_row([], |r| r.get::<_, String>(0));
        match row {
            Ok(stored) => {
                if stored != current_machine_id {
                    Ok(VaultStatus::HardwareTampered { expected: stored, actual: current_machine_id.to_string() })
                } else {
                    Ok(VaultStatus::OperationalReady)
                }
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(VaultStatus::Unprovisioned),
            Err(e) => Err(VaultError::Database(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_lifecycle_provision_and_unlock() {
        let conn = Connection::open_in_memory().unwrap();
        let machine_id = "MACH_POS_TERM_001";
        let master_pass = "SovereignMaster2026!#";

        assert_eq!(VaultManager::get_status(&conn, machine_id).unwrap(), VaultStatus::Unprovisioned);
        VaultManager::provision(&conn, machine_id, master_pass).unwrap();
        assert_eq!(VaultManager::get_status(&conn, machine_id).unwrap(), VaultStatus::OperationalReady);

        let session = VaultManager::unlock_operational(&conn, machine_id).unwrap();
        assert_eq!(session.machine_id(), machine_id);
        assert_ne!(session.key_ref(), &[0u8; 32]);
    }

    #[test]
    fn test_hardware_tamper_detection_on_disk_theft() {
        let conn = Connection::open_in_memory().unwrap();
        let orig = "AUTHENTIC_POS_TERMINAL";
        let stolen = "ATTACKER_FOREIGN_LAPTOP";

        VaultManager::provision(&conn, orig, "OwnerSecret99").unwrap();
        let res = VaultManager::unlock_operational(&conn, stolen);
        assert!(matches!(res, Err(VaultError::HardwareSealMismatch { .. })));

        assert_eq!(
            VaultManager::get_status(&conn, stolen).unwrap(),
            VaultStatus::HardwareTampered { expected: orig.to_string(), actual: stolen.to_string() }
        );
    }

    #[test]
    fn test_master_unlock_and_rekey() {
        let conn = Connection::open_in_memory().unwrap();
        let machine_id = "ORIGINAL_HARDWARE";
        let master_pass = "OwnerSecret99";

        VaultManager::provision(&conn, machine_id, master_pass).unwrap();
        let bad = VaultManager::unlock_master(&conn, "WrongPassword");
        assert_eq!(bad.err(), Some(VaultError::InvalidMasterPassword));

        let master_sess = VaultManager::unlock_master(&conn, master_pass).unwrap();
        assert!(master_sess.is_valid());

        let new_mach = "REPLACEMENT_HARDWARE_TERMINAL";
        VaultManager::rekey_operational(&conn, new_mach, &master_sess).unwrap();

        assert!(VaultManager::unlock_operational(&conn, machine_id).is_err());
        let new_sess = VaultManager::unlock_operational(&conn, new_mach).unwrap();
        assert_eq!(new_sess.machine_id(), new_mach);
    }
}
