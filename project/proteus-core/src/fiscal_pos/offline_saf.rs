//! EMV Store-and-Forward (SaF) Offline Capture & Cryptographic Voucher Engine (Phase 24.1.1).
//! Enables zero-downtime card payments during internet/LAN outages under strict merchant risk limits.
//! Cryptographically seals card cryptograms without PCI-DSS cleartext exposure.

use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::encryption::encrypt;

#[derive(Debug, Error, PartialEq)]
pub enum SafError {
    #[error("Offline single transaction cap exceeded: {amount_cents} > {max_cents} cents")]
    SingleLimitExceeded { amount_cents: u64, max_cents: u64 },
    #[error("Offline cumulative risk cap exceeded: total {total_cents} > {max_cents} cents")]
    CumulativeLimitExceeded { total_cents: u64, max_cents: u64 },
    #[error("Offline queue count limit reached: {count} >= {max_count}")]
    QueueCountExceeded { count: u32, max_count: u32 },
    #[error("Voucher not found: {0}")]
    NotFound(String),
    #[error("Cryptographic error: {0}")]
    Crypto(String),
    #[error("Database error: {0}")]
    Database(String),
}

impl From<rusqlite::Error> for SafError {
    fn from(err: rusqlite::Error) -> Self {
        SafError::Database(err.to_string())
    }
}

/// Operational status of an offline store-and-forward voucher.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SafStatus {
    CapturedOffline,
    Settling,
    Settled,
    DeclinedOnline,
}

impl SafStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CapturedOffline => "CAPTURED_OFFLINE",
            Self::Settling => "SETTLING",
            Self::Settled => "SETTLED",
            Self::DeclinedOnline => "DECLINED_ONLINE",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "SETTLING" => Self::Settling,
            "SETTLED" => Self::Settled,
            "DECLINED_ONLINE" => Self::DeclinedOnline,
            _ => Self::CapturedOffline,
        }
    }
}

/// Merchant risk limits for offline card authorizations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafRiskConfig {
    pub max_single_amount_cents: u64,
    pub max_cumulative_cents: u64,
    pub max_queue_count: u32,
    pub max_offline_hours: u32,
}

impl Default for SafRiskConfig {
    fn default() -> Self {
        Self {
            max_single_amount_cents: 5000, // 50.00 EUR single transaction cap
            max_cumulative_cents: 50000,  // 500.00 EUR total cumulative cap
            max_queue_count: 20,          // Max 20 vouchers queued offline
            max_offline_hours: 72,        // Max 72 hours before mandatory settlement
        }
    }
}

/// Request to capture an EMV card payment offline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafCaptureRequest {
    pub terminal_id: String,
    pub merchant_id: String,
    pub masked_pan: String,
    pub card_brand: String,
    pub amount_cents: u64,
    pub currency: String,
    pub emv_cryptogram: String,
}

/// Cryptographically sealed offline transaction voucher.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SafVoucher {
    pub voucher_id: String,
    pub terminal_id: String,
    pub merchant_id: String,
    pub masked_pan: String,
    pub card_brand: String,
    pub amount_cents: u64,
    pub currency: String,
    pub emv_tc_hash: String,
    pub encrypted_payload: String,
    pub offline_auth_code: String,
    pub signature: String,
    pub status: SafStatus,
    pub captured_at: String,
    pub settled_at: Option<String>,
}

/// Summary metrics of offline vouchers in queue.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SafSummary {
    pub pending_count: u32,
    pub pending_amount_cents: u64,
    pub settled_count: u32,
    pub settled_amount_cents: u64,
    pub declined_count: u32,
}

pub struct SafEngine;

impl SafEngine {
    pub fn init_schema(conn: &Connection) -> Result<(), SafError> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS pos_saf_vouchers (
                voucher_id TEXT PRIMARY KEY,
                terminal_id TEXT NOT NULL,
                merchant_id TEXT NOT NULL,
                masked_pan TEXT NOT NULL,
                card_brand TEXT NOT NULL,
                amount_cents INTEGER NOT NULL,
                currency TEXT NOT NULL,
                emv_tc_hash TEXT NOT NULL,
                encrypted_payload TEXT NOT NULL,
                offline_auth_code TEXT NOT NULL,
                signature TEXT NOT NULL,
                status TEXT NOT NULL,
                captured_at TEXT NOT NULL,
                settled_at TEXT
            );",
            [],
        )?;
        Ok(())
    }

    fn calculate_signature(voucher_id: &str, terminal_id: &str, amount_cents: u64, tc_hash: &str, key: &[u8; 32]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(b"PROTEUS_EMV_SAF_VOUCHER_V1");
        hasher.update(voucher_id.as_bytes());
        hasher.update(terminal_id.as_bytes());
        hasher.update(amount_cents.to_le_bytes());
        hasher.update(tc_hash.as_bytes());
        hasher.update(key);
        format!("{:x}", hasher.finalize())
    }

    pub fn capture_offline(
        conn: &Connection,
        req: SafCaptureRequest,
        risk: &SafRiskConfig,
        terminal_key: &[u8; 32],
    ) -> Result<SafVoucher, SafError> {
        Self::init_schema(conn)?;

        if req.amount_cents > risk.max_single_amount_cents {
            return Err(SafError::SingleLimitExceeded { amount_cents: req.amount_cents, max_cents: risk.max_single_amount_cents });
        }

        let (cur_count, cur_total): (u32, u64) = conn.query_row(
            "SELECT COUNT(*), COALESCE(SUM(amount_cents), 0) FROM pos_saf_vouchers WHERE status = 'CAPTURED_OFFLINE'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;

        if cur_count >= risk.max_queue_count {
            return Err(SafError::QueueCountExceeded { count: cur_count, max_count: risk.max_queue_count });
        }
        if cur_total + req.amount_cents > risk.max_cumulative_cents {
            return Err(SafError::CumulativeLimitExceeded { total_cents: cur_total + req.amount_cents, max_cents: risk.max_cumulative_cents });
        }

        let enc_bytes = encrypt(req.emv_cryptogram.as_bytes(), terminal_key)
            .map_err(|e| SafError::Crypto(e.to_string()))?;
        let enc_hex: String = enc_bytes.iter().map(|b| format!("{:02x}", b)).collect();
        let tc_hash = format!("{:x}", Sha256::digest(req.emv_cryptogram.as_bytes()));
        let voucher_id = format!("saf_{}", uuid::Uuid::new_v4());
        let offline_auth_code = format!("OFF-{:06}", rand::random::<u32>() % 1000000);
        let signature = Self::calculate_signature(&voucher_id, &req.terminal_id, req.amount_cents, &tc_hash, terminal_key);
        let now = Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO pos_saf_vouchers (
                voucher_id, terminal_id, merchant_id, masked_pan, card_brand,
                amount_cents, currency, emv_tc_hash, encrypted_payload,
                offline_auth_code, signature, status, captured_at, settled_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, 'CAPTURED_OFFLINE', ?12, NULL)",
            params![voucher_id, req.terminal_id, req.merchant_id, req.masked_pan, req.card_brand, req.amount_cents, req.currency, tc_hash, enc_hex, offline_auth_code, signature, now],
        )?;

        Ok(SafVoucher {
            voucher_id,
            terminal_id: req.terminal_id,
            merchant_id: req.merchant_id,
            masked_pan: req.masked_pan,
            card_brand: req.card_brand,
            amount_cents: req.amount_cents,
            currency: req.currency,
            emv_tc_hash: tc_hash,
            encrypted_payload: enc_hex,
            offline_auth_code,
            signature,
            status: SafStatus::CapturedOffline,
            captured_at: now,
            settled_at: None,
        })
    }

    pub fn get_pending(conn: &Connection) -> Result<Vec<SafVoucher>, SafError> {
        Self::init_schema(conn)?;
        let mut stmt = conn.prepare(
            "SELECT voucher_id, terminal_id, merchant_id, masked_pan, card_brand,
                    amount_cents, currency, emv_tc_hash, encrypted_payload,
                    offline_auth_code, signature, status, captured_at, settled_at
             FROM pos_saf_vouchers WHERE status = 'CAPTURED_OFFLINE' ORDER BY captured_at ASC",
        )?;
        let rows = stmt.query_map([], |r| {
            let st_str: String = r.get(11)?;
            Ok(SafVoucher {
                voucher_id: r.get(0)?,
                terminal_id: r.get(1)?,
                merchant_id: r.get(2)?,
                masked_pan: r.get(3)?,
                card_brand: r.get(4)?,
                amount_cents: r.get(5)?,
                currency: r.get(6)?,
                emv_tc_hash: r.get(7)?,
                encrypted_payload: r.get(8)?,
                offline_auth_code: r.get(9)?,
                signature: r.get(10)?,
                status: SafStatus::parse(&st_str),
                captured_at: r.get(12)?,
                settled_at: r.get(13)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(SafError::from)
    }

    pub fn mark_settled(conn: &Connection, voucher_id: &str) -> Result<(), SafError> {
        Self::init_schema(conn)?;
        let now = Utc::now().to_rfc3339();
        let affected = conn.execute(
            "UPDATE pos_saf_vouchers SET status = 'SETTLED', settled_at = ?1 WHERE voucher_id = ?2",
            params![now, voucher_id],
        )?;
        if affected == 0 { return Err(SafError::NotFound(voucher_id.to_string())); }
        Ok(())
    }

    pub fn mark_declined(conn: &Connection, voucher_id: &str) -> Result<(), SafError> {
        Self::init_schema(conn)?;
        let now = Utc::now().to_rfc3339();
        let affected = conn.execute(
            "UPDATE pos_saf_vouchers SET status = 'DECLINED_ONLINE', settled_at = ?1 WHERE voucher_id = ?2",
            params![now, voucher_id],
        )?;
        if affected == 0 { return Err(SafError::NotFound(voucher_id.to_string())); }
        Ok(())
    }

    pub fn get_summary(conn: &Connection) -> Result<SafSummary, SafError> {
        Self::init_schema(conn)?;
        let pending: (u32, u64) = conn.query_row(
            "SELECT COUNT(*), COALESCE(SUM(amount_cents), 0) FROM pos_saf_vouchers WHERE status = 'CAPTURED_OFFLINE'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        let settled: (u32, u64) = conn.query_row(
            "SELECT COUNT(*), COALESCE(SUM(amount_cents), 0) FROM pos_saf_vouchers WHERE status = 'SETTLED'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        let declined: u32 = conn.query_row(
            "SELECT COUNT(*) FROM pos_saf_vouchers WHERE status = 'DECLINED_ONLINE'",
            [],
            |r| r.get(0),
        )?;
        Ok(SafSummary {
            pending_count: pending.0,
            pending_amount_cents: pending.1,
            settled_count: settled.0,
            settled_amount_cents: settled.1,
            declined_count: declined,
        })
    }

    pub fn format_receipt_slip(v: &SafVoucher) -> String {
        let euros = v.amount_cents as f64 / 100.0;
        format!(
            "================================\n   PROTEUS OFFLINE STORE & FORWARD\n           CARD RECEIPT\n================================\nTERMINAL: {}\nMERCHANT: {}\nDATE/TIME: {}\n--------------------------------\nCARD: {} ({})\nAMOUNT: {:.2} {}\nAUTH CODE: {}\nTC HASH: {:.16}...\nSTATUS: OFFLINE CAPTURED (SaF)\n================================\n CUSTOMER SIGNATURE STORED\n================================\n",
            v.terminal_id, v.merchant_id, v.captured_at, v.masked_pan, v.card_brand, euros, v.currency, v.offline_auth_code, v.emv_tc_hash
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_saf_capture_and_settlement_lifecycle() {
        let conn = Connection::open_in_memory().unwrap();
        let risk = SafRiskConfig::default();
        let term_key = [7u8; 32];

        let req = SafCaptureRequest {
            terminal_id: "POS_ISLAND_DELIVERY_01".into(),
            merchant_id: "MERCH_GR_999888".into(),
            masked_pan: "**** **** **** 4242".into(),
            card_brand: "VISA".into(),
            amount_cents: 2500, // 25.00 EUR
            currency: "EUR".into(),
            emv_cryptogram: "ARQC_9F26_8B47AC12E0987B".into(),
        };

        let voucher = SafEngine::capture_offline(&conn, req, &risk, &term_key).unwrap();
        assert_eq!(voucher.status, SafStatus::CapturedOffline);
        assert_eq!(voucher.amount_cents, 2500);

        let pending = SafEngine::get_pending(&conn).unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].voucher_id, voucher.voucher_id);

        let slip = SafEngine::format_receipt_slip(&voucher);
        assert!(slip.contains("OFFLINE CAPTURED (SaF)"));
        assert!(slip.contains("25.00 EUR"));

        // Settle voucher
        SafEngine::mark_settled(&conn, &voucher.voucher_id).unwrap();
        let summary = SafEngine::get_summary(&conn).unwrap();
        assert_eq!(summary.pending_count, 0);
        assert_eq!(summary.settled_count, 1);
        assert_eq!(summary.settled_amount_cents, 2500);
    }

    #[test]
    fn test_saf_risk_limits_rejection() {
        let conn = Connection::open_in_memory().unwrap();
        let mut risk = SafRiskConfig::default();
        risk.max_single_amount_cents = 3000;
        risk.max_cumulative_cents = 5000;
        let term_key = [9u8; 32];

        let req_large = SafCaptureRequest {
            terminal_id: "TERM_1".into(),
            merchant_id: "M1".into(),
            masked_pan: "**** 1111".into(),
            card_brand: "MASTERCARD".into(),
            amount_cents: 3500,
            currency: "EUR".into(),
            emv_cryptogram: "CRYPTO_1".into(),
        };
        let err = SafEngine::capture_offline(&conn, req_large, &risk, &term_key).unwrap_err();
        assert!(matches!(err, SafError::SingleLimitExceeded { .. }));

        let req_valid = SafCaptureRequest {
            terminal_id: "TERM_1".into(),
            merchant_id: "M1".into(),
            masked_pan: "**** 1111".into(),
            card_brand: "MASTERCARD".into(),
            amount_cents: 2800,
            currency: "EUR".into(),
            emv_cryptogram: "CRYPTO_1".into(),
        };
        SafEngine::capture_offline(&conn, req_valid, &risk, &term_key).unwrap();

        let req_cum = SafCaptureRequest {
            terminal_id: "TERM_1".into(),
            merchant_id: "M1".into(),
            masked_pan: "**** 2222".into(),
            card_brand: "VISA".into(),
            amount_cents: 2500,
            currency: "EUR".into(),
            emv_cryptogram: "CRYPTO_2".into(),
        };
        let err2 = SafEngine::capture_offline(&conn, req_cum, &risk, &term_key).unwrap_err();
        assert!(matches!(err2, SafError::CumulativeLimitExceeded { .. }));
    }
}
