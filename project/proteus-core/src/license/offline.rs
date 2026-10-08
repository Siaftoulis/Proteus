//! Proteus Core — Cryptographically Signed Offline License Token Engine.
//! Issues and verifies offline HMAC-SHA256 license tokens for standalone terminals and pilot deployments.

use super::{write_cache, LicenseError, LicenseInfo};
use serde::{Deserialize, Serialize};

/// Cryptographically signed offline license claims for standalone terminals and pilot shops.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OfflineLicenseClaims {
    pub license_id: String,
    pub customer_name: String,
    pub shop_afm: String,
    pub machine_id: Option<String>,
    pub max_users: i32,
    pub features: Vec<String>,
    pub issued_at: String,
    pub expires_at: String,
    pub is_pilot: bool,
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn from_hex(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 {
        return None;
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok())
        .collect()
}

pub fn get_master_signing_key() -> String {
    std::env::var("PROTEUS_ROOT_SIGNING_KEY")
        .unwrap_or_else(|_| "PROTEUS_OFFLINE_ROOT_MASTER_KEY_2026_PILOT".to_string())
}

pub fn issue_offline_token(
    claims: &OfflineLicenseClaims,
    master_key: &str,
) -> Result<String, LicenseError> {
    let json = serde_json::to_string(claims).map_err(|e| LicenseError::Parse(e.to_string()))?;
    let payload_hex = to_hex(json.as_bytes());
    let sig = crate::license::mor::hmac_sha256(master_key.as_bytes(), payload_hex.as_bytes());
    let sig_hex = to_hex(&sig);
    Ok(format!("PROT-LIC-{}.{}", payload_hex, sig_hex))
}

pub fn verify_offline_token(
    token: &str,
    current_machine_id: Option<&str>,
    master_key: &str,
) -> Result<OfflineLicenseClaims, LicenseError> {
    let raw = token
        .strip_prefix("PROT-LIC-")
        .ok_or(LicenseError::InvalidTokenFormat)?;
    let parts: Vec<&str> = raw.split('.').collect();
    if parts.len() != 2 {
        return Err(LicenseError::InvalidTokenFormat);
    }
    let payload_hex = parts[0];
    let signature_hex = parts[1];

    let expected_sig =
        crate::license::mor::hmac_sha256(master_key.as_bytes(), payload_hex.as_bytes());
    let expected_sig_hex = to_hex(&expected_sig);

    if signature_hex.to_lowercase() != expected_sig_hex.to_lowercase() {
        return Err(LicenseError::SignatureMismatch);
    }

    let payload_bytes = from_hex(payload_hex).ok_or(LicenseError::InvalidTokenFormat)?;
    let claims: OfflineLicenseClaims =
        serde_json::from_slice(&payload_bytes).map_err(|e| LicenseError::Parse(e.to_string()))?;

    if let (Some(bound_m), Some(curr_m)) = (&claims.machine_id, current_machine_id) {
        if !bound_m.is_empty() && bound_m != curr_m {
            return Err(LicenseError::MachineMismatch {
                expected: bound_m.clone(),
                actual: curr_m.to_string(),
            });
        }
    }

    if let Ok(exp) = chrono::DateTime::parse_from_rfc3339(&claims.expires_at) {
        if chrono::Utc::now() > exp {
            return Err(LicenseError::TokenExpired(claims.expires_at.clone()));
        }
    }

    Ok(claims)
}

pub fn install_offline_token(token: &str, machine_id: &str) -> Result<LicenseInfo, LicenseError> {
    let claims = verify_offline_token(token, Some(machine_id), &get_master_signing_key())?;
    let info = LicenseInfo {
        valid: true,
        max_users: claims.max_users,
        features: claims.features,
        expires_at: claims.expires_at,
        cached_at: chrono::Utc::now().to_rfc3339(),
    };
    write_cache(&info);
    Ok(info)
}

pub fn issue_pilot_token(
    customer_name: &str,
    shop_afm: &str,
    machine_id: Option<&str>,
    days: i64,
) -> Result<String, LicenseError> {
    let now = chrono::Utc::now();
    let expires = now + chrono::Duration::days(days);
    let claims = OfflineLicenseClaims {
        license_id: format!("PILOT-{}", now.timestamp_millis()),
        customer_name: customer_name.to_string(),
        shop_afm: shop_afm.to_string(),
        machine_id: machine_id.map(|s| s.to_string()),
        max_users: 5,
        features: vec![
            "intake".to_string(),
            "kanban".to_string(),
            "escpos_printing".to_string(),
            "smlm_inventory".to_string(),
            "contractor_ledger".to_string(),
            "offline_audit".to_string(),
        ],
        issued_at: now.to_rfc3339(),
        expires_at: expires.to_rfc3339(),
        is_pilot: true,
    };
    issue_offline_token(&claims, &get_master_signing_key())
}
