use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;
use tracing::instrument;

const LICENSE_SERVER: &str = "https://license.crmbuilder.com";
const COMMUNITY_MAX_USERS: i32 = 3;

#[derive(Debug, Error)]
pub enum LicenseError {
    #[error("Failed to contact license server: {0}")]
    Network(String),
    #[error("Invalid server response: {0}")]
    Parse(String),
    #[error("Cache read failed: {0}")]
    Cache(String),
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

fn read_cache() -> Option<LicenseInfo> {
    let path = cache_path();
    let data = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&data).ok()
}

fn write_cache(info: &LicenseInfo) {
    if let Ok(data) = serde_json::to_string(info) {
        let _ = fs::write(cache_path(), data);
    }
}

fn clear_cache() {
    let _ = fs::remove_file(cache_path());
}

#[instrument(skip(license_key, machine_id), fields(key_len = license_key.len()))]
pub fn verify_online(license_key: &str, machine_id: &str) -> Result<LicenseInfo, LicenseError> {
    let base_server = std::env::var("PROTEUS_LICENSE_SERVER")
        .unwrap_or_else(|_| LICENSE_SERVER.to_string());
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
        if let Ok(info) = verify_online(key, machine_id) {
            return info;
        }
        tracing::debug!("Online verification failed, falling back to cache");
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
}
