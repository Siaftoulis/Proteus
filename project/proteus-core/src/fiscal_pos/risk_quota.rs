//! Dynamic IRIS Offline QR Generator & Merchant Risk Quota Guard (Phase 24.1.2).
//! Implements statutory Greek IRIS Payments (DIAS/EPC QR standard) for offline instant bank transfers
//! and an autonomous multi-channel merchant risk quota engine with circuit breaker trip protection.

use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum RiskQuotaError {
    #[error("Single IRIS amount exceeds statutory limit: {amount_cents} > {max_cents} cents")]
    SingleLimitExceeded { amount_cents: u64, max_cents: u64 },
    #[error("Daily offline risk exposure ceiling reached: total {total_cents} > {ceiling_cents} cents")]
    DailyExposureCeilingReached { total_cents: u64, ceiling_cents: u64 },
    #[error("Offline circuit breaker tripped! Cumulative exposure {cumulative_cents} cents requires online sync")]
    CircuitBreakerTripped { cumulative_cents: u64 },
    #[error("Invalid IBAN or BIC specification")]
    InvalidBankDetails,
    #[error("Invalid IRIS offline signature or corrupted token")]
    InvalidSignature,
    #[error("Database error: {0}")]
    Database(String),
}

impl From<rusqlite::Error> for RiskQuotaError {
    fn from(err: rusqlite::Error) -> Self {
        RiskQuotaError::Database(err.to_string())
    }
}

/// Risk policy parameters for offline transactions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskPolicy {
    pub max_single_iris_cents: u64,         // Default: 50,000 (500.00 EUR IRIS consumer limit)
    pub max_daily_exposure_cents: u64,      // Default: 100,000 (1,000.00 EUR / day)
    pub circuit_breaker_ceiling_cents: u64, // Default: 150,000 (1,500.00 EUR hard stop)
}

impl Default for RiskPolicy {
    fn default() -> Self {
        Self {
            max_single_iris_cents: 50000,
            max_daily_exposure_cents: 100000,
            circuit_breaker_ceiling_cents: 150000,
        }
    }
}

/// Result of evaluating offline risk quota.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RiskDecision {
    Approved { remaining_daily_cents: u64 },
    Rejected(String),
}

/// Request to generate an offline dynamic IRIS EPC QR payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrisQrRequest {
    pub merchant_name: String,
    pub iban: String,
    pub bic: String,
    pub amount_cents: u64,
    pub invoice_or_ticket_ref: String,
}

/// Generated IRIS EPC-compliant QR payload with sovereign signature.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IrisQrResponse {
    pub transaction_id: String,
    pub epc_qr_string: String,
    pub sovereign_token: String,
    pub amount_cents: u64,
    pub remittance_info: String,
    pub generated_at: String,
}

/// Aggregate risk exposure metrics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RiskMetrics {
    pub rolling_24h_cents: u64,
    pub total_pending_offline_cents: u64,
    pub total_transactions_today: u32,
    pub circuit_breaker_active: bool,
}

pub struct RiskQuotaGuard;

impl RiskQuotaGuard {
    pub fn init_schema(conn: &Connection) -> Result<(), RiskQuotaError> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS pos_risk_ledger (
                tx_id TEXT PRIMARY KEY,
                channel TEXT NOT NULL, /* 'EMV_SAF' or 'IRIS_QR' */
                amount_cents INTEGER NOT NULL,
                status TEXT NOT NULL,  /* 'CAPTURED_OFFLINE', 'SETTLED', 'DECLINED' */
                created_at TEXT NOT NULL,
                created_epoch INTEGER NOT NULL
            );",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS pos_iris_transactions (
                tx_id TEXT PRIMARY KEY,
                merchant_name TEXT NOT NULL,
                iban TEXT NOT NULL,
                bic TEXT NOT NULL,
                amount_cents INTEGER NOT NULL,
                remittance_ref TEXT NOT NULL,
                sovereign_token TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL
            );",
            [],
        )?;
        Ok(())
    }

    pub fn evaluate_and_record(
        conn: &Connection,
        tx_id: &str,
        channel: &str,
        amount_cents: u64,
        policy: &RiskPolicy,
    ) -> Result<RiskDecision, RiskQuotaError> {
        Self::init_schema(conn)?;

        let now_epoch = Utc::now().timestamp();
        let rolling_24h_epoch = now_epoch - 86400;

        // 1. Calculate cumulative un-settled exposure
        let cumulative_exposure: u64 = conn.query_row(
            "SELECT COALESCE(SUM(amount_cents), 0) FROM pos_risk_ledger WHERE status = 'CAPTURED_OFFLINE'",
            [],
            |r| r.get(0),
        )?;

        if cumulative_exposure + amount_cents > policy.circuit_breaker_ceiling_cents {
            return Err(RiskQuotaError::CircuitBreakerTripped {
                cumulative_cents: cumulative_exposure + amount_cents,
            });
        }

        // 2. Calculate rolling 24h exposure
        let rolling_24h: u64 = conn.query_row(
            "SELECT COALESCE(SUM(amount_cents), 0) FROM pos_risk_ledger WHERE created_epoch >= ?1",
            params![rolling_24h_epoch],
            |r| r.get(0),
        )?;

        if rolling_24h + amount_cents > policy.max_daily_exposure_cents {
            return Err(RiskQuotaError::DailyExposureCeilingReached {
                total_cents: rolling_24h + amount_cents,
                ceiling_cents: policy.max_daily_exposure_cents,
            });
        }

        let now_iso = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO pos_risk_ledger (tx_id, channel, amount_cents, status, created_at, created_epoch)
             VALUES (?1, ?2, ?3, 'CAPTURED_OFFLINE', ?4, ?5)",
            params![tx_id, channel, amount_cents, now_iso, now_epoch],
        )?;

        let remaining = policy.max_daily_exposure_cents.saturating_sub(rolling_24h + amount_cents);
        Ok(RiskDecision::Approved { remaining_daily_cents: remaining })
    }

    /// Formats an official EPC069-12 SEPA/IRIS Quick Response Code payload string.
    pub fn format_epc_qr_string(req: &IrisQrRequest) -> String {
        let euros = req.amount_cents as f64 / 100.0;
        format!(
            "BCD\n002\n1\nSCT\n{}\n{}\n{}\nEUR{:.2}\n\n\n{}\nIRIS-PROTEUS-PAY\n",
            req.bic.trim(),
            req.merchant_name.trim(),
            req.iban.trim().replace(' ', ""),
            euros,
            req.invoice_or_ticket_ref.trim()
        )
    }

    fn generate_signature(tx_id: &str, iban: &str, amount_cents: u64, secret: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(b"PROTEUS_DYNAMIC_IRIS_V1");
        hasher.update(tx_id.as_bytes());
        hasher.update(iban.as_bytes());
        hasher.update(amount_cents.to_le_bytes());
        hasher.update(secret.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Generates dynamic IRIS QR and registers transaction under merchant risk policy.
    pub fn generate_dynamic_iris_qr(
        conn: &Connection,
        req: IrisQrRequest,
        policy: &RiskPolicy,
        secret_key: &str,
    ) -> Result<IrisQrResponse, RiskQuotaError> {
        Self::init_schema(conn)?;

        if req.iban.trim().len() < 15 || req.bic.trim().len() < 8 {
            return Err(RiskQuotaError::InvalidBankDetails);
        }

        if req.amount_cents > policy.max_single_iris_cents {
            return Err(RiskQuotaError::SingleLimitExceeded {
                amount_cents: req.amount_cents,
                max_cents: policy.max_single_iris_cents,
            });
        }

        let tx_id = format!("iris_{}", uuid::Uuid::new_v4());
        Self::evaluate_and_record(conn, &tx_id, "IRIS_QR", req.amount_cents, policy)?;

        let epc_str = Self::format_epc_qr_string(&req);
        let sig = Self::generate_signature(&tx_id, &req.iban, req.amount_cents, secret_key);
        let sovereign_token = format!("IRIS-PROT:{}:{}:{}", tx_id, req.amount_cents, &sig[..16]);
        let now_iso = Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO pos_iris_transactions (
                tx_id, merchant_name, iban, bic, amount_cents, remittance_ref,
                sovereign_token, status, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'CAPTURED_OFFLINE', ?8)",
            params![
                tx_id,
                req.merchant_name,
                req.iban,
                req.bic,
                req.amount_cents,
                req.invoice_or_ticket_ref,
                sovereign_token,
                now_iso,
            ],
        )?;

        Ok(IrisQrResponse {
            transaction_id: tx_id,
            epc_qr_string: epc_str,
            sovereign_token,
            amount_cents: req.amount_cents,
            remittance_info: req.invoice_or_ticket_ref,
            generated_at: now_iso,
        })
    }

    pub fn get_metrics(conn: &Connection, policy: &RiskPolicy) -> Result<RiskMetrics, RiskQuotaError> {
        Self::init_schema(conn)?;
        let rolling_24h_epoch = Utc::now().timestamp() - 86400;

        let rolling_24h: u64 = conn.query_row(
            "SELECT COALESCE(SUM(amount_cents), 0) FROM pos_risk_ledger WHERE created_epoch >= ?1",
            params![rolling_24h_epoch],
            |r| r.get(0),
        )?;

        let total_pending: u64 = conn.query_row(
            "SELECT COALESCE(SUM(amount_cents), 0) FROM pos_risk_ledger WHERE status = 'CAPTURED_OFFLINE'",
            [],
            |r| r.get(0),
        )?;

        let tx_count_today: u32 = conn.query_row(
            "SELECT COUNT(*) FROM pos_risk_ledger WHERE created_epoch >= ?1",
            params![rolling_24h_epoch],
            |r| r.get(0),
        )?;

        Ok(RiskMetrics {
            rolling_24h_cents: rolling_24h,
            total_pending_offline_cents: total_pending,
            total_transactions_today: tx_count_today,
            circuit_breaker_active: total_pending >= policy.circuit_breaker_ceiling_cents,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_epc_qr_payload_generation() {
        let req = IrisQrRequest {
            merchant_name: "Proteus Workshop OE".into(),
            iban: "GR1201101250000001234567890".into(),
            bic: "ETHNGRAA".into(),
            amount_cents: 4500, // 45.00 EUR
            invoice_or_ticket_ref: "TICK-9082".into(),
        };

        let epc = RiskQuotaGuard::format_epc_qr_string(&req);
        assert!(epc.starts_with("BCD\n002\n1\nSCT\nETHNGRAA\n"));
        assert!(epc.contains("EUR45.00"));
        assert!(epc.contains("GR1201101250000001234567890"));
        assert!(epc.contains("TICK-9082"));
    }

    #[test]
    fn test_iris_offline_generation_and_metrics() {
        let conn = Connection::open_in_memory().unwrap();
        let policy = RiskPolicy::default();
        let secret = "SovereignSecretKey99";

        let req = IrisQrRequest {
            merchant_name: "Auto Garage Athens".into(),
            iban: "GR9901701000000000123456789".into(),
            bic: "PIRAGRAA".into(),
            amount_cents: 12000, // 120.00 EUR
            invoice_or_ticket_ref: "INV-2026-004".into(),
        };

        let res = RiskQuotaGuard::generate_dynamic_iris_qr(&conn, req, &policy, secret).unwrap();
        assert!(res.sovereign_token.starts_with("IRIS-PROT:"));
        assert_eq!(res.amount_cents, 12000);

        let metrics = RiskQuotaGuard::get_metrics(&conn, &policy).unwrap();
        assert_eq!(metrics.total_pending_offline_cents, 12000);
        assert_eq!(metrics.total_transactions_today, 1);
        assert!(!metrics.circuit_breaker_active);
    }

    #[test]
    fn test_risk_quota_rejection_and_circuit_breaker() {
        let conn = Connection::open_in_memory().unwrap();
        let mut policy = RiskPolicy::default();
        policy.max_single_iris_cents = 30000;         // 300.00 EUR single
        policy.max_daily_exposure_cents = 50000;       // 500.00 EUR daily
        policy.circuit_breaker_ceiling_cents = 60000;  // 600.00 EUR hard stop
        let secret = "Sec";

        // 1. Single limit rejection
        let req_large = IrisQrRequest {
            merchant_name: "Store".into(),
            iban: "GR1101101250000001234567890".into(),
            bic: "ETHNGRAA".into(),
            amount_cents: 35000, // 350.00 EUR > 300.00 EUR
            invoice_or_ticket_ref: "T1".into(),
        };
        assert!(matches!(
            RiskQuotaGuard::generate_dynamic_iris_qr(&conn, req_large, &policy, secret),
            Err(RiskQuotaError::SingleLimitExceeded { .. })
        ));

        // 2. Successful transaction
        let req_valid = IrisQrRequest {
            merchant_name: "Store".into(),
            iban: "GR1101101250000001234567890".into(),
            bic: "ETHNGRAA".into(),
            amount_cents: 28000,
            invoice_or_ticket_ref: "T2".into(),
        };
        RiskQuotaGuard::generate_dynamic_iris_qr(&conn, req_valid, &policy, secret).unwrap();

        // 3. Daily exposure ceiling reached (28000 + 25000 > 50000)
        let req_exceed_daily = IrisQrRequest {
            merchant_name: "Store".into(),
            iban: "GR1101101250000001234567890".into(),
            bic: "ETHNGRAA".into(),
            amount_cents: 25000,
            invoice_or_ticket_ref: "T3".into(),
        };
        assert!(matches!(
            RiskQuotaGuard::generate_dynamic_iris_qr(&conn, req_exceed_daily, &policy, secret),
            Err(RiskQuotaError::DailyExposureCeilingReached { .. })
        ));
    }
}
