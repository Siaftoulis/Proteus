//! Proteus Core — Sovereign License Verification & Activation Engine.
//! Supports Online Validation, Signed Offline Tokens, and MoR Webhooks (Lemon Squeezy & Paddle).

pub mod checkout;
pub mod mor;
pub mod offline;

pub use checkout::*;
pub use mor::*;
pub use offline::*;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;
use tracing::instrument;

const LICENSE_SERVER: &str = "https://license.crmbuilder.com";
pub const COMMUNITY_MAX_USERS: i32 = 3;

#[derive(Debug, Error)]
pub enum LicenseError {
    #[error("Failed to contact license server: {0}")]
    Network(String),
    #[error("Invalid server response: {0}")]
    Parse(String),
    #[error("Cache read failed: {0}")]
    Cache(String),
    #[error("Offline token format invalid")]
    InvalidTokenFormat,
    #[error("Offline token signature mismatch (tampered)")]
    SignatureMismatch,
    #[error("Offline token expired at {0}")]
    TokenExpired(String),
    #[error("Machine ID mismatch. Bound to {expected}, running on {actual}")]
    MachineMismatch { expected: String, actual: String },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LicenseInfo {
    pub valid: bool,
    pub max_users: i32,
    pub features: Vec<String>,
    pub expires_at: String,
    pub cached_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ServerResponse {
    pub valid: bool,
    pub max_users: i32,
    pub features: Vec<String>,
    pub expires_at: String,
    pub reason: Option<String>,
}

fn cache_dir() -> PathBuf {
    let mut path = std::env::current_exe().unwrap_or_default();
    path.pop();
    path
}

fn cache_path() -> PathBuf {
    let mut path = cache_dir();
    path.push(".license_cache.json");
    path
}

pub fn read_cache() -> Option<LicenseInfo> {
    let path = cache_path();
    let data = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn write_cache(info: &LicenseInfo) {
    if let Ok(data) = serde_json::to_string(info) {
        let _ = fs::write(cache_path(), data);
    }
}

pub fn clear_cache() {
    let _ = fs::remove_file(cache_path());
}

#[instrument(skip(license_key, machine_id), fields(key_len = license_key.len()))]
pub fn verify_online(license_key: &str, machine_id: &str) -> Result<LicenseInfo, LicenseError> {
    let base_server =
        std::env::var("PROTEUS_LICENSE_SERVER").unwrap_or_else(|_| LICENSE_SERVER.to_string());
    let url = format!(
        "{}/verify?license_key={}&machine_id={}",
        base_server, license_key, machine_id
    );

    let resp: ServerResponse = ureq::get(&url)
        .call()
        .map_err(|e| LicenseError::Network(e.to_string()))?
        .into_json()
        .map_err(|e| LicenseError::Parse(e.to_string()))?;

    let info = LicenseInfo {
        valid: resp.valid,
        max_users: resp.max_users,
        features: resp.features,
        expires_at: resp.expires_at,
        cached_at: chrono::Utc::now().to_rfc3339(),
    };

    if resp.valid {
        write_cache(&info);
        tracing::info!("License verified online");
    } else {
        clear_cache();
        tracing::warn!("License invalid");
    }

    Ok(info)
}

pub fn get_license_info(license_key: Option<&str>, machine_id: &str) -> LicenseInfo {
    if let Some(key) = license_key {
        if key.starts_with("PROT-LIC-") {
            if let Ok(claims) =
                verify_offline_token(key, Some(machine_id), &get_master_signing_key())
            {
                let info = LicenseInfo {
                    valid: true,
                    max_users: claims.max_users,
                    features: claims.features,
                    expires_at: claims.expires_at,
                    cached_at: chrono::Utc::now().to_rfc3339(),
                };
                write_cache(&info);
                return info;
            }
        } else if let Ok(info) = verify_online(key, machine_id) {
            return info;
        }
        tracing::debug!("Verification failed, falling back to cache");
    }

    read_cache().unwrap_or(LicenseInfo {
        valid: false,
        max_users: COMMUNITY_MAX_USERS,
        features: vec![],
        expires_at: String::new(),
        cached_at: String::new(),
    })
}

pub fn get_max_users(license_key: Option<&str>, machine_id: &str) -> i32 {
    let info = get_license_info(license_key, machine_id);
    if info.valid && info.max_users > COMMUNITY_MAX_USERS {
        info.max_users
    } else {
        COMMUNITY_MAX_USERS
    }
}

pub fn is_licensed(license_key: Option<&str>, machine_id: &str) -> bool {
    let info = get_license_info(license_key, machine_id);
    info.valid
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static CACHE_LOCK: Mutex<()> = Mutex::new(());

    fn with_cache<T>(f: impl FnOnce() -> T) -> T {
        let _lock = CACHE_LOCK.lock().unwrap();
        clear_cache();
        let result = f();
        clear_cache();
        result
    }

    #[test]
    fn test_default_license_info() {
        with_cache(|| {
            let info = get_license_info(None, "test-machine");
            assert!(!info.valid);
            assert_eq!(info.max_users, COMMUNITY_MAX_USERS);
            assert!(info.features.is_empty());
        });
    }

    #[test]
    fn test_cache_roundtrip() {
        with_cache(|| {
            let info = LicenseInfo {
                valid: true,
                max_users: 10,
                features: vec!["export".to_string()],
                expires_at: "2027-01-01".to_string(),
                cached_at: "2026-07-04".to_string(),
            };
            write_cache(&info);
            let cached = read_cache().unwrap();
            assert!(cached.valid);
            assert_eq!(cached.max_users, 10);
            assert_eq!(cached.features, vec!["export"]);
        });
    }

    #[test]
    fn test_get_max_users_without_license() {
        with_cache(|| {
            let max = get_max_users(None, "machine-1");
            assert_eq!(max, COMMUNITY_MAX_USERS);
        });
    }

    #[test]
    fn test_is_licensed_without_license() {
        with_cache(|| {
            assert!(!is_licensed(None, "machine-1"));
        });
    }

    #[test]
    fn test_license_cache_fallback() {
        with_cache(|| {
            let info = LicenseInfo {
                valid: true,
                max_users: 5,
                features: vec![],
                expires_at: "2027-01-01".to_string(),
                cached_at: "2026-07-04".to_string(),
            };
            write_cache(&info);
            let retrieved = get_license_info(Some("invalid-key-for-offline-test"), "machine-1");
            assert!(retrieved.valid);
            assert_eq!(retrieved.max_users, 5);
        });
    }

    #[test]
    fn test_community_max_users_constant() {
        assert_eq!(COMMUNITY_MAX_USERS, 3);
    }

    #[test]
    fn test_offline_pilot_token_issuance_and_verification() {
        with_cache(|| {
            let token =
                issue_pilot_token("Tech Repair Lab", "123456789", Some("TERM-101"), 30).unwrap();
            assert!(token.starts_with("PROT-LIC-"));

            let claims =
                verify_offline_token(&token, Some("TERM-101"), &get_master_signing_key()).unwrap();
            assert_eq!(claims.customer_name, "Tech Repair Lab");
            assert_eq!(claims.shop_afm, "123456789");
            assert_eq!(claims.max_users, 5);
            assert!(claims.is_pilot);

            let info = install_offline_token(&token, "TERM-101").unwrap();
            assert!(info.valid);
            assert_eq!(info.max_users, 5);

            let retrieved = get_license_info(Some(&token), "TERM-101");
            assert!(retrieved.valid);
            assert_eq!(retrieved.max_users, 5);
        });
    }

    #[test]
    fn test_offline_tampered_token_rejected() {
        let token = issue_pilot_token("Shop A", "999999999", None, 30).unwrap();
        let mut tampered = token.clone();
        tampered.push('a');
        let err = verify_offline_token(&tampered, None, &get_master_signing_key()).unwrap_err();
        assert!(matches!(err, LicenseError::SignatureMismatch));
    }

    #[test]
    fn test_offline_machine_mismatch_rejected() {
        let token = issue_pilot_token("Shop B", "888888888", Some("POS-ALPHA"), 30).unwrap();
        let err =
            verify_offline_token(&token, Some("POS-BETA"), &get_master_signing_key()).unwrap_err();
        assert!(matches!(err, LicenseError::MachineMismatch { .. }));
    }
}
