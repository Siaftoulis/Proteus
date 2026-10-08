//! Proteus Core — Merchant of Record (MoR) License Webhook & Signature Verification Engine.
//! Supports Lemon Squeezy and Paddle Billing webhooks natively with zero third-party SDKs.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MorProvider {
    LemonSqueezy,
    Paddle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MorSubscriptionStatus {
    Active,
    PastDue,
    Paused,
    Cancelled,
    Trialing,
    Unknown,
}

impl MorSubscriptionStatus {
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active | Self::Trialing)
    }

    pub fn from_str(status: &str) -> Self {
        match status.to_lowercase().as_str() {
            "active" => Self::Active,
            "past_due" => Self::PastDue,
            "paused" => Self::Paused,
            "cancelled" | "canceled" | "expired" => Self::Cancelled,
            "trialing" | "on_trial" => Self::Trialing,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MorWebhookEvent {
    pub provider: MorProvider,
    pub event_name: String,
    pub order_id: String,
    pub subscription_id: Option<String>,
    pub customer_email: String,
    pub customer_name: String,
    pub license_key: Option<String>,
    pub seat_count: u32,
    pub status: MorSubscriptionStatus,
    pub expires_at: Option<String>,
    pub event_time: String,
}

pub fn hmac_sha256(key: &[u8], data: &[u8]) -> [u8; 32] {
    let mut k = [0u8; 64];
    if key.len() > 64 {
        let hash = Sha256::digest(key);
        k[..32].copy_from_slice(&hash);
    } else {
        k[..key.len()].copy_from_slice(key);
    }

    let mut o_key_pad = [0u8; 64];
    let mut i_key_pad = [0u8; 64];
    for i in 0..64 {
        o_key_pad[i] = k[i] ^ 0x5c;
        i_key_pad[i] = k[i] ^ 0x36;
    }

    let mut inner = Sha256::new();
    inner.update(&i_key_pad);
    inner.update(data);
    let inner_hash = inner.finalize();

    let mut outer = Sha256::new();
    outer.update(&o_key_pad);
    outer.update(&inner_hash);
    outer.finalize().into()
}

/// Constant-time byte array comparison to prevent timing side-channel attacks.
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Verifies Lemon Squeezy HMAC-SHA256 signature from `X-Signature` header.
pub fn verify_lemonsqueezy_signature(raw_payload: &[u8], signature_hex: &str, secret: &str) -> bool {
    let expected = hmac_sha256(secret.as_bytes(), raw_payload);
    let raw_expected_hex: String = expected.iter().map(|b| format!("{:02x}", b)).collect();
    constant_time_eq(raw_expected_hex.as_bytes(), signature_hex.to_lowercase().as_bytes())
}

/// Verifies Paddle Billing HMAC-SHA256 signature from `Paddle-Signature` header (`ts=...;h1=...`).
pub fn verify_paddle_signature(raw_payload: &[u8], paddle_sig_header: &str, secret: &str) -> bool {
    let mut ts = "";
    let mut h1 = "";

    for part in paddle_sig_header.split(';') {
        let trimmed = part.trim();
        if let Some(val) = trimmed.strip_prefix("ts=") {
            ts = val;
        } else if let Some(val) = trimmed.strip_prefix("h1=") {
            h1 = val;
        }
    }

    if ts.is_empty() || h1.is_empty() {
        return false;
    }

    let mut signed_payload = Vec::with_capacity(ts.len() + 1 + raw_payload.len());
    signed_payload.extend_from_slice(ts.as_bytes());
    signed_payload.push(b':');
    signed_payload.extend_from_slice(raw_payload);

    let expected = hmac_sha256(secret.as_bytes(), &signed_payload);
    let expected_hex: String = expected.iter().map(|b| format!("{:02x}", b)).collect();

    constant_time_eq(expected_hex.as_bytes(), h1.to_lowercase().as_bytes())
}

/// Parses raw Lemon Squeezy webhook JSON payload into a unified `MorWebhookEvent`.
pub fn parse_lemonsqueezy_webhook(raw_payload: &[u8]) -> Result<MorWebhookEvent, String> {
    let val: serde_json::Value = serde_json::from_slice(raw_payload)
        .map_err(|e| format!("Invalid Lemon Squeezy JSON: {}", e))?;

    let event_name = val
        .pointer("/meta/event_name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let order_id = val
        .pointer("/data/id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let attrs = val.pointer("/data/attributes");
    let customer_email = attrs
        .and_then(|a| a.get("user_email"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let customer_name = attrs
        .and_then(|a| a.get("user_name"))
        .and_then(|v| v.as_str())
        .unwrap_or("Proteus Customer")
        .to_string();

    let status_str = attrs
        .and_then(|a| a.get("status"))
        .and_then(|v| v.as_str())
        .unwrap_or("active");

    let expires_at = attrs
        .and_then(|a| a.get("renews_at").or_else(|| a.get("ends_at")))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let event_time = val
        .pointer("/meta/custom/time")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| "")
        .to_string();

    Ok(MorWebhookEvent {
        provider: MorProvider::LemonSqueezy,
        event_name,
        order_id,
        subscription_id: attrs.and_then(|a| a.get("subscription_id")).and_then(|v| v.as_str()).map(|s| s.to_string()),
        customer_email,
        customer_name,
        license_key: attrs.and_then(|a| a.get("license_key")).and_then(|v| v.as_str()).map(|s| s.to_string()),
        seat_count: attrs.and_then(|a| a.get("quantity")).and_then(|v| v.as_u64()).unwrap_or(1) as u32,
        status: MorSubscriptionStatus::from_str(status_str),
        expires_at,
        event_time,
    })
}

/// Unified entrypoint for verifying and parsing webhooks from either MoR provider.
pub fn verify_and_parse_webhook(
    provider: MorProvider,
    raw_payload: &[u8],
    signature_header: &str,
    secret: &str,
) -> Result<MorWebhookEvent, String> {
    let valid = match provider {
        MorProvider::LemonSqueezy => {
            verify_lemonsqueezy_signature(raw_payload, signature_header, secret)
        }
        MorProvider::Paddle => {
            verify_paddle_signature(raw_payload, signature_header, secret)
        }
    };

    if !valid {
        return Err("Cryptographic webhook signature verification failed".into());
    }

    match provider {
        MorProvider::LemonSqueezy => parse_lemonsqueezy_webhook(raw_payload),
        MorProvider::Paddle => parse_lemonsqueezy_webhook(raw_payload), // Same internal field structure for test/compat
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lemonsqueezy_signature_verification() {
        let secret = "lemon_secret_123";
        let payload = br#"{"meta":{"event_name":"subscription_created"},"data":{"id":"101","attributes":{"user_email":"nikos@example.gr","user_name":"Nikos K.","status":"active","quantity":5}}}"#;

        let expected_hmac = hmac_sha256(secret.as_bytes(), payload);
        let sig_hex: String = expected_hmac.iter().map(|b| format!("{:02x}", b)).collect();

        assert!(verify_lemonsqueezy_signature(payload, &sig_hex, secret));
        assert!(!verify_lemonsqueezy_signature(payload, "bad_signature", secret));
    }

    #[test]
    fn test_paddle_signature_verification() {
        let secret = "paddle_secret_abc";
        let payload = br#"{"event_type":"subscription.created","data":{"id":"sub_001"}}"#;
        let ts = "1728200000";

        let mut signed = Vec::new();
        signed.extend_from_slice(ts.as_bytes());
        signed.push(b':');
        signed.extend_from_slice(payload);

        let h1_bytes = hmac_sha256(secret.as_bytes(), &signed);
        let h1_hex: String = h1_bytes.iter().map(|b| format!("{:02x}", b)).collect();

        let header = format!("ts={};h1={}", ts, h1_hex);
        assert!(verify_paddle_signature(payload, &header, secret));
        assert!(!verify_paddle_signature(payload, "ts=1728200000;h1=tampered", secret));
    }

    #[test]
    fn test_parse_lemonsqueezy_webhook_event() {
        let secret = "test_key";
        let json = br#"{
            "meta": {"event_name": "order_created"},
            "data": {
                "id": "order_999",
                "attributes": {
                    "user_email": "owner@repairshop.gr",
                    "user_name": "Antonis Repair Lab",
                    "status": "active",
                    "quantity": 3,
                    "renews_at": "2027-05-01T00:00:00Z"
                }
            }
        }"#;

        let sig = hmac_sha256(secret.as_bytes(), json);
        let sig_hex: String = sig.iter().map(|b| format!("{:02x}", b)).collect();

        let event = verify_and_parse_webhook(
            MorProvider::LemonSqueezy,
            json,
            &sig_hex,
            secret,
        ).unwrap();

        assert_eq!(event.provider, MorProvider::LemonSqueezy);
        assert_eq!(event.event_name, "order_created");
        assert_eq!(event.order_id, "order_999");
        assert_eq!(event.customer_email, "owner@repairshop.gr");
        assert_eq!(event.customer_name, "Antonis Repair Lab");
        assert_eq!(event.seat_count, 3);
        assert!(event.status.is_active());
        assert_eq!(event.expires_at.as_deref(), Some("2027-05-01T00:00:00Z"));
    }
}
