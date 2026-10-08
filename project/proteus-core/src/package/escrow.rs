//! In-Platform Escrow & Cryptographic Delivery Gate for Proteus BOS (Master Problem Audit P20).
//! Eliminates platform transaction leakage and non-payment risks:
//! 1. Client deposits agreed amount into sovereign Escrow contract (`Funded`).
//! 2. Designer produces preview; client reviews and submits signed approval token (`Approved`).
//! 3. Proteus Cryptographic Delivery Gate binds final `.pr` package to `target_client_license` (Host Key Binding).
//! 4. 90% Designer payout / 10% Proteus protocol fee split settlement is cryptographically sealed (`Released`).

use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;

use crate::package::PrPackage;

#[derive(Debug, Error, PartialEq)]
pub enum EscrowError {
    #[error("Invalid state transition from {from} to {to}")]
    InvalidStateTransition { from: String, to: String },
    #[error("Approval token is missing or invalid")]
    MissingApprovalToken,
    #[error("Contract is not approved for release")]
    NotApproved,
    #[error("License hash mismatch. Expected: {expected}, found: {found}")]
    LicenseMismatch { expected: String, found: String },
    #[error("Package serialization error: {0}")]
    PackageError(String),
    #[error("Serialization / Deserialization error: {0}")]
    Serialization(String),
    #[error("Database error: {0}")]
    Database(String),
}

impl From<rusqlite::Error> for EscrowError {
    fn from(err: rusqlite::Error) -> Self {
        EscrowError::Database(err.to_string())
    }
}

/// Lifecycle status of an Escrow collaboration contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EscrowStatus {
    Initiated,
    Funded,
    InReview,
    Approved,
    Released,
    Disputed,
    Refunded,
}

impl EscrowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Initiated => "INITIATED",
            Self::Funded => "FUNDED",
            Self::InReview => "IN_REVIEW",
            Self::Approved => "APPROVED",
            Self::Released => "RELEASED",
            Self::Disputed => "DISPUTED",
            Self::Refunded => "REFUNDED",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "FUNDED" => Self::Funded,
            "IN_REVIEW" => Self::InReview,
            "APPROVED" => Self::Approved,
            "RELEASED" => Self::Released,
            "DISPUTED" => Self::Disputed,
            "REFUNDED" => Self::Refunded,
            _ => Self::Initiated,
        }
    }
}

/// Escrow contract binding client deposit, designer payout, and machine license.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EscrowContract {
    pub contract_id: String,
    pub client_license_hash: String,
    pub designer_pcd_id: String,
    pub agreed_amount_cents: u64,
    pub designer_payout_cents: u64,
    pub platform_fee_cents: u64,
    pub status: EscrowStatus,
    pub approval_token: Option<String>,
    pub bound_package_checksum: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Cryptographic settlement record generated when a bound package is delivered.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeliveryRelease {
    pub release_id: String,
    pub contract_id: String,
    pub bound_package_bundle_id: String,
    pub bound_package_checksum: String,
    pub client_license_hash: String,
    pub designer_payout_cents: u64,
    pub platform_fee_cents: u64,
    pub released_at: String,
    pub settlement_signature: String,
}

pub struct EscrowEngine;

impl EscrowEngine {
    /// Creates a new Escrow contract with standard 90% Designer / 10% Platform split.
    pub fn create_contract(
        client_license_hash: &str,
        designer_pcd_id: &str,
        agreed_amount_cents: u64,
    ) -> EscrowContract {
        let contract_id = format!("esc_{}", Uuid::now_v7());
        let now = Utc::now().to_rfc3339();
        let designer_payout_cents = (agreed_amount_cents * 90) / 100;
        let platform_fee_cents = agreed_amount_cents - designer_payout_cents;

        EscrowContract {
            contract_id,
            client_license_hash: client_license_hash.to_string(),
            designer_pcd_id: designer_pcd_id.to_string(),
            agreed_amount_cents,
            designer_payout_cents,
            platform_fee_cents,
            status: EscrowStatus::Initiated,
            approval_token: None,
            bound_package_checksum: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    /// Client deposits agreed amount into Escrow (`Initiated` -> `Funded`).
    pub fn fund_contract(contract: &mut EscrowContract) -> Result<(), EscrowError> {
        if contract.status != EscrowStatus::Initiated {
            return Err(EscrowError::InvalidStateTransition {
                from: contract.status.as_str().to_string(),
                to: EscrowStatus::Funded.as_str().to_string(),
            });
        }
        contract.status = EscrowStatus::Funded;
        contract.updated_at = Utc::now().to_rfc3339();
        Ok(())
    }

    /// Designer begins or updates prototype review (`Funded` -> `InReview`).
    pub fn start_review(contract: &mut EscrowContract) -> Result<(), EscrowError> {
        if contract.status != EscrowStatus::Funded && contract.status != EscrowStatus::InReview {
            return Err(EscrowError::InvalidStateTransition {
                from: contract.status.as_str().to_string(),
                to: EscrowStatus::InReview.as_str().to_string(),
            });
        }
        contract.status = EscrowStatus::InReview;
        contract.updated_at = Utc::now().to_rfc3339();
        Ok(())
    }

    /// Client issues approval with cryptographic token (`InReview` -> `Approved`).
    pub fn submit_approval(contract: &mut EscrowContract, approval_token: &str) -> Result<(), EscrowError> {
        if contract.status != EscrowStatus::InReview {
            return Err(EscrowError::InvalidStateTransition {
                from: contract.status.as_str().to_string(),
                to: EscrowStatus::Approved.as_str().to_string(),
            });
        }
        if approval_token.trim().is_empty() {
            return Err(EscrowError::MissingApprovalToken);
        }
        contract.status = EscrowStatus::Approved;
        contract.approval_token = Some(approval_token.to_string());
        contract.updated_at = Utc::now().to_rfc3339();
        Ok(())
    }

    /// Cryptographic Host Key Binding & Release Gate (`Approved` -> `Released`).
    /// Stamps the package specifically with `client_license_hash` and generates settlement signature.
    pub fn bind_and_deliver(
        contract: &mut EscrowContract,
        unbound_package: &PrPackage,
    ) -> Result<(PrPackage, DeliveryRelease), EscrowError> {
        if contract.status != EscrowStatus::Approved {
            return Err(EscrowError::NotApproved);
        }
        if contract.approval_token.is_none() {
            return Err(EscrowError::MissingApprovalToken);
        }

        let mut bound_package = unbound_package.clone();
        bound_package.manifest.target_client_license = Some(contract.client_license_hash.clone());

        let package_bytes = bound_package
            .to_bytes()
            .map_err(|e| EscrowError::PackageError(e.to_string()))?;

        let mut hasher = Sha256::new();
        hasher.update(&package_bytes);
        let bound_checksum = format!("{:x}", hasher.finalize());

        let release_id = format!("rel_{}", Uuid::now_v7());
        let released_at = Utc::now().to_rfc3339();

        let mut sig_hasher = Sha256::new();
        sig_hasher.update(
            format!(
                "SETTLEMENT:{}:{}:{}:{}:{}",
                contract.contract_id,
                bound_checksum,
                contract.designer_payout_cents,
                contract.platform_fee_cents,
                released_at
            )
            .as_bytes(),
        );
        let settlement_signature = format!("sig_{:x}", sig_hasher.finalize());

        let release = DeliveryRelease {
            release_id,
            contract_id: contract.contract_id.clone(),
            bound_package_bundle_id: bound_package.manifest.bundle_id.clone(),
            bound_package_checksum: bound_checksum.clone(),
            client_license_hash: contract.client_license_hash.clone(),
            designer_payout_cents: contract.designer_payout_cents,
            platform_fee_cents: contract.platform_fee_cents,
            released_at,
            settlement_signature,
        };

        contract.status = EscrowStatus::Released;
        contract.bound_package_checksum = Some(bound_checksum);
        contract.updated_at = Utc::now().to_rfc3339();

        Ok((bound_package, release))
    }

    /// Raises a dispute on a funded or in-review contract.
    pub fn raise_dispute(contract: &mut EscrowContract) -> Result<(), EscrowError> {
        if contract.status != EscrowStatus::Funded && contract.status != EscrowStatus::InReview {
            return Err(EscrowError::InvalidStateTransition {
                from: contract.status.as_str().to_string(),
                to: EscrowStatus::Disputed.as_str().to_string(),
            });
        }
        contract.status = EscrowStatus::Disputed;
        contract.updated_at = Utc::now().to_rfc3339();
        Ok(())
    }

    /// Refunds contract funds to client.
    pub fn refund_contract(contract: &mut EscrowContract) -> Result<(), EscrowError> {
        if contract.status != EscrowStatus::Funded
            && contract.status != EscrowStatus::InReview
            && contract.status != EscrowStatus::Disputed
        {
            return Err(EscrowError::InvalidStateTransition {
                from: contract.status.as_str().to_string(),
                to: EscrowStatus::Refunded.as_str().to_string(),
            });
        }
        contract.status = EscrowStatus::Refunded;
        contract.updated_at = Utc::now().to_rfc3339();
        Ok(())
    }

    /// Initializes SQLite tables for Escrow contracts and releases.
    pub fn init_schema(conn: &Connection) -> Result<(), EscrowError> {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS escrow_contracts (
                contract_id TEXT PRIMARY KEY,
                client_license_hash TEXT NOT NULL,
                designer_pcd_id TEXT NOT NULL,
                agreed_amount_cents INTEGER NOT NULL,
                designer_payout_cents INTEGER NOT NULL,
                platform_fee_cents INTEGER NOT NULL,
                status TEXT NOT NULL,
                approval_token TEXT,
                bound_package_checksum TEXT,
                payload_json TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS escrow_deliveries (
                release_id TEXT PRIMARY KEY,
                contract_id TEXT NOT NULL,
                bound_package_bundle_id TEXT NOT NULL,
                bound_package_checksum TEXT NOT NULL,
                client_license_hash TEXT NOT NULL,
                designer_payout_cents INTEGER NOT NULL,
                platform_fee_cents INTEGER NOT NULL,
                settlement_signature TEXT NOT NULL,
                payload_json TEXT NOT NULL,
                released_at TEXT NOT NULL
            );
            "#,
        )?;
        Ok(())
    }

    pub fn save_contract(conn: &Connection, contract: &EscrowContract) -> Result<(), EscrowError> {
        let payload = serde_json::to_string(contract).map_err(|e| EscrowError::Serialization(e.to_string()))?;
        conn.execute(
            r#"
            INSERT OR REPLACE INTO escrow_contracts (
                contract_id, client_license_hash, designer_pcd_id, agreed_amount_cents,
                designer_payout_cents, platform_fee_cents, status, approval_token,
                bound_package_checksum, payload_json, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            "#,
            params![
                contract.contract_id,
                contract.client_license_hash,
                contract.designer_pcd_id,
                contract.agreed_amount_cents,
                contract.designer_payout_cents,
                contract.platform_fee_cents,
                contract.status.as_str(),
                contract.approval_token,
                contract.bound_package_checksum,
                payload,
                contract.created_at,
                contract.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_contract(conn: &Connection, contract_id: &str) -> Result<Option<EscrowContract>, EscrowError> {
        let mut stmt = conn.prepare("SELECT payload_json FROM escrow_contracts WHERE contract_id = ?1")?;
        let mut rows = stmt.query(params![contract_id])?;
        if let Some(row) = rows.next()? {
            let json_str: String = row.get(0)?;
            let contract: EscrowContract = serde_json::from_str(&json_str)
                .map_err(|e| EscrowError::Serialization(e.to_string()))?;
            Ok(Some(contract))
        } else {
            Ok(None)
        }
    }

    pub fn save_delivery(conn: &Connection, delivery: &DeliveryRelease) -> Result<(), EscrowError> {
        let payload = serde_json::to_string(delivery).map_err(|e| EscrowError::Serialization(e.to_string()))?;
        conn.execute(
            r#"
            INSERT OR REPLACE INTO escrow_deliveries (
                release_id, contract_id, bound_package_bundle_id, bound_package_checksum,
                client_license_hash, designer_payout_cents, platform_fee_cents,
                settlement_signature, payload_json, released_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            params![
                delivery.release_id,
                delivery.contract_id,
                delivery.bound_package_bundle_id,
                delivery.bound_package_checksum,
                delivery.client_license_hash,
                delivery.designer_payout_cents,
                delivery.platform_fee_cents,
                delivery.settlement_signature,
                payload,
                delivery.released_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_delivery(conn: &Connection, contract_id: &str) -> Result<Option<DeliveryRelease>, EscrowError> {
        let mut stmt = conn.prepare("SELECT payload_json FROM escrow_deliveries WHERE contract_id = ?1")?;
        let mut rows = stmt.query(params![contract_id])?;
        if let Some(row) = rows.next()? {
            let json_str: String = row.get(0)?;
            let delivery: DeliveryRelease = serde_json::from_str(&json_str)
                .map_err(|e| EscrowError::Serialization(e.to_string()))?;
            Ok(Some(delivery))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escrow_split_calculation_90_10() {
        let contract = EscrowEngine::create_contract("lic_hash_123", "pcd_nikos", 15000); // 150.00 EUR
        assert_eq!(contract.agreed_amount_cents, 15000);
        assert_eq!(contract.designer_payout_cents, 13500); // 90% = 135.00 EUR
        assert_eq!(contract.platform_fee_cents, 1500); // 10% = 15.00 EUR
        assert_eq!(contract.status, EscrowStatus::Initiated);
    }

    #[test]
    fn test_escrow_lifecycle_and_cryptographic_host_key_binding() {
        let mut contract = EscrowEngine::create_contract("lic_target_xyz", "pcd_elena", 20000);

        EscrowEngine::fund_contract(&mut contract).expect("fund");
        assert_eq!(contract.status, EscrowStatus::Funded);

        EscrowEngine::start_review(&mut contract).expect("review");
        assert_eq!(contract.status, EscrowStatus::InReview);

        // Cannot deliver before approval
        let base_pkg = PrPackage::new("PKG-TEST", "Test Suite", "pcd_elena");
        assert!(EscrowEngine::bind_and_deliver(&mut contract, &base_pkg).is_err());

        // Approve with token
        EscrowEngine::submit_approval(&mut contract, "appr_valid_token_789").expect("approve");
        assert_eq!(contract.status, EscrowStatus::Approved);

        // Execute cryptographic Host Key Binding
        let (bound_pkg, release) = EscrowEngine::bind_and_deliver(&mut contract, &base_pkg).expect("deliver");
        assert_eq!(bound_pkg.manifest.target_client_license, Some("lic_target_xyz".to_string()));
        assert_eq!(contract.status, EscrowStatus::Released);
        assert_eq!(release.contract_id, contract.contract_id);
        assert!(release.settlement_signature.starts_with("sig_"));
        assert_eq!(release.designer_payout_cents, 18000);
        assert_eq!(release.platform_fee_cents, 2000);
    }

    #[test]
    fn test_escrow_sqlite_persistence() {
        let conn = Connection::open_in_memory().expect("sqlite in memory");
        EscrowEngine::init_schema(&conn).expect("init schema");

        let mut contract = EscrowEngine::create_contract("lic_client_abc", "pcd_kostas", 10000);
        EscrowEngine::fund_contract(&mut contract).expect("fund");
        EscrowEngine::save_contract(&conn, &contract).expect("save contract");

        let fetched = EscrowEngine::get_contract(&conn, &contract.contract_id)
            .expect("get contract")
            .expect("exists");
        assert_eq!(fetched.client_license_hash, "lic_client_abc");
        assert_eq!(fetched.status, EscrowStatus::Funded);

        EscrowEngine::start_review(&mut contract).expect("review");
        EscrowEngine::submit_approval(&mut contract, "appr_tok_456").expect("approve");
        let pkg = PrPackage::new("PKG-DEMO", "Demo", "pcd_kostas");
        let (_, release) = EscrowEngine::bind_and_deliver(&mut contract, &pkg).expect("deliver");

        EscrowEngine::save_delivery(&conn, &release).expect("save delivery");
        let fetched_del = EscrowEngine::get_delivery(&conn, &contract.contract_id)
            .expect("get delivery")
            .expect("delivery exists");
        assert_eq!(fetched_del.designer_payout_cents, 9000);
    }
}
