//! In-App Share Point & Sandbox Preview Protocol for Proteus BOS (Master Problem Audit P18).
//! Facilitates friction-free collaboration between business clients and certified designers:
//! 1. Client anonymous requirements specification (`.prreq` container with SHA-256 seal).
//! 2. Designer interactive sandbox preview (`.prpreview` container with mandatory watermark).
//! 3. Client review feedback recording & cryptographic approval token generation for Escrow handoff.

use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;

pub const PRREQ_MAGIC: &[u8; 6] = b"PRREQ\x01";
pub const PRPREV_MAGIC: &[u8; 7] = b"PRPREV\x01";
pub const DEFAULT_PREVIEW_WATERMARK: &str = "PROTEUS SANDBOX PREVIEW — NOT LICENSED FOR PRODUCTION";

#[derive(Debug, Error, PartialEq)]
pub enum SharePointError {
    #[error("Magic header mismatch. File is not a valid Proteus Share Point container.")]
    InvalidHeader,
    #[error("Integrity check failed: computed {computed} does not match seal {expected}")]
    TamperedPayload { expected: String, computed: String },
    #[error("Serialization / Deserialization error: {0}")]
    Serialization(String),
    #[error("Database error: {0}")]
    Database(String),
}

impl From<rusqlite::Error> for SharePointError {
    fn from(err: rusqlite::Error) -> Self {
        SharePointError::Database(err.to_string())
    }
}

/// An individual field required by the business client.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldRequirement {
    pub name: String,
    pub data_type: String,
    pub is_required: bool,
    pub description: String,
}

/// A desired business automation or flow intent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkflowIntent {
    pub title: String,
    pub trigger_event: String,
    pub expected_action: String,
}

/// Anonymized business requirements package (`.prreq`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RequirementsPackage {
    pub req_id: String,
    pub client_anonymous_id: String,
    pub title: String,
    pub business_type: String,
    pub fields: Vec<FieldRequirement>,
    pub workflows: Vec<WorkflowIntent>,
    pub notes: String,
    pub created_at: String,
    pub sha256_seal: String,
}

impl RequirementsPackage {
    pub fn new(
        title: &str,
        business_type: &str,
        client_anon_id: &str,
        fields: Vec<FieldRequirement>,
        workflows: Vec<WorkflowIntent>,
        notes: &str,
    ) -> Self {
        let req_id = format!("req_{}", Uuid::now_v7());
        let created_at = Utc::now().to_rfc3339();
        let payload_for_hash = format!("{title}|{business_type}|{client_anon_id}|{created_at}|{}", fields.len());
        let mut hasher = Sha256::new();
        hasher.update(payload_for_hash.as_bytes());
        let sha256_seal = format!("{:x}", hasher.finalize());

        Self {
            req_id,
            client_anonymous_id: client_anon_id.to_string(),
            title: title.to_string(),
            business_type: business_type.to_string(),
            fields,
            workflows,
            notes: notes.to_string(),
            created_at,
            sha256_seal,
        }
    }

    pub fn compute_seal(&self) -> String {
        let payload_for_hash = format!(
            "{}|{}|{}|{}|{}",
            self.title, self.business_type, self.client_anonymous_id, self.created_at, self.fields.len()
        );
        let mut hasher = Sha256::new();
        hasher.update(payload_for_hash.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn verify_integrity(&self) -> bool {
        self.sha256_seal == self.compute_seal()
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, SharePointError> {
        let json_data = serde_json::to_vec(self).map_err(|e| SharePointError::Serialization(e.to_string()))?;
        let mut bytes = Vec::with_capacity(PRREQ_MAGIC.len() + json_data.len());
        bytes.extend_from_slice(PRREQ_MAGIC);
        bytes.extend_from_slice(&json_data);
        Ok(bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SharePointError> {
        if bytes.len() < PRREQ_MAGIC.len() || &bytes[0..PRREQ_MAGIC.len()] != PRREQ_MAGIC {
            return Err(SharePointError::InvalidHeader);
        }
        let package: Self = serde_json::from_slice(&bytes[PRREQ_MAGIC.len()..])
            .map_err(|e| SharePointError::Serialization(e.to_string()))?;
        let computed = package.compute_seal();
        if package.sha256_seal != computed {
            return Err(SharePointError::TamperedPayload {
                expected: package.sha256_seal,
                computed,
            });
        }
        Ok(package)
    }
}

/// Interactive Sandbox Preview Bundle (`.prpreview`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PreviewBundle {
    pub preview_id: String,
    pub req_id: Option<String>,
    pub bundle_id: String,
    pub title: String,
    pub designer_pcd_id: String,
    pub watermark_text: String,
    pub sample_views_json: String,
    pub sample_schema_json: String,
    pub created_at: String,
    pub sha256_seal: String,
}

impl PreviewBundle {
    pub fn new(
        req_id: Option<String>,
        bundle_id: &str,
        title: &str,
        designer_pcd_id: &str,
        sample_views_json: &str,
        sample_schema_json: &str,
    ) -> Self {
        let preview_id = format!("prev_{}", Uuid::now_v7());
        let created_at = Utc::now().to_rfc3339();
        let payload = format!("{bundle_id}|{title}|{designer_pcd_id}|{created_at}");
        let mut hasher = Sha256::new();
        hasher.update(payload.as_bytes());
        let sha256_seal = format!("{:x}", hasher.finalize());

        Self {
            preview_id,
            req_id,
            bundle_id: bundle_id.to_string(),
            title: title.to_string(),
            designer_pcd_id: designer_pcd_id.to_string(),
            watermark_text: DEFAULT_PREVIEW_WATERMARK.to_string(),
            sample_views_json: sample_views_json.to_string(),
            sample_schema_json: sample_schema_json.to_string(),
            created_at,
            sha256_seal,
        }
    }

    pub fn compute_seal(&self) -> String {
        let payload = format!(
            "{}|{}|{}|{}",
            self.bundle_id, self.title, self.designer_pcd_id, self.created_at
        );
        let mut hasher = Sha256::new();
        hasher.update(payload.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn verify_integrity(&self) -> bool {
        self.sha256_seal == self.compute_seal()
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, SharePointError> {
        let json_data = serde_json::to_vec(self).map_err(|e| SharePointError::Serialization(e.to_string()))?;
        let mut bytes = Vec::with_capacity(PRPREV_MAGIC.len() + json_data.len());
        bytes.extend_from_slice(PRPREV_MAGIC);
        bytes.extend_from_slice(&json_data);
        Ok(bytes)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, SharePointError> {
        if bytes.len() < PRPREV_MAGIC.len() || &bytes[0..PRPREV_MAGIC.len()] != PRPREV_MAGIC {
            return Err(SharePointError::InvalidHeader);
        }
        let bundle: Self = serde_json::from_slice(&bytes[PRPREV_MAGIC.len()..])
            .map_err(|e| SharePointError::Serialization(e.to_string()))?;
        let computed = bundle.compute_seal();
        if bundle.sha256_seal != computed {
            return Err(SharePointError::TamperedPayload {
                expected: bundle.sha256_seal,
                computed,
            });
        }
        Ok(bundle)
    }
}

/// Verdict on client's inspection of a sandbox preview.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewVerdict {
    Approved,
    NeedsRevision,
    Rejected,
}

impl ReviewVerdict {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Approved => "APPROVED",
            Self::NeedsRevision => "NEEDS_REVISION",
            Self::Rejected => "REJECTED",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "APPROVED" => Self::Approved,
            "REJECTED" => Self::Rejected,
            _ => Self::NeedsRevision,
        }
    }
}

/// Feedback comment tied to a preview widget or screen.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReviewFeedbackItem {
    pub item_id: String,
    pub target_element: String,
    pub comment: String,
    pub created_at: String,
}

impl ReviewFeedbackItem {
    pub fn new(target_element: &str, comment: &str) -> Self {
        Self {
            item_id: format!("fb_{}", Uuid::now_v7()),
            target_element: target_element.to_string(),
            comment: comment.to_string(),
            created_at: Utc::now().to_rfc3339(),
        }
    }
}

/// Completed review session submitted by the client.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClientReviewSession {
    pub session_id: String,
    pub preview_id: String,
    pub client_anonymous_id: String,
    pub verdict: ReviewVerdict,
    pub feedback_items: Vec<ReviewFeedbackItem>,
    pub approval_token: Option<String>,
    pub reviewed_at: String,
}

impl ClientReviewSession {
    pub fn new(
        preview_id: &str,
        client_anon_id: &str,
        verdict: ReviewVerdict,
        feedback_items: Vec<ReviewFeedbackItem>,
    ) -> Self {
        let session_id = format!("rev_{}", Uuid::now_v7());
        let reviewed_at = Utc::now().to_rfc3339();
        let approval_token = if verdict == ReviewVerdict::Approved {
            let mut hasher = Sha256::new();
            hasher.update(format!("APPROVAL:{preview_id}:{client_anon_id}:{reviewed_at}").as_bytes());
            Some(format!("appr_{:x}", hasher.finalize()))
        } else {
            None
        };

        Self {
            session_id,
            preview_id: preview_id.to_string(),
            client_anonymous_id: client_anon_id.to_string(),
            verdict,
            feedback_items,
            approval_token,
            reviewed_at,
        }
    }
}

/// SQLite persistence manager for local Share Point transfers & review sessions.
pub struct SharePointLedger;

impl SharePointLedger {
    pub fn init_schema(conn: &Connection) -> Result<(), SharePointError> {
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS sharepoint_requirements (
                req_id TEXT PRIMARY KEY,
                client_anonymous_id TEXT NOT NULL,
                title TEXT NOT NULL,
                business_type TEXT NOT NULL,
                payload_json TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS sharepoint_previews (
                preview_id TEXT PRIMARY KEY,
                req_id TEXT,
                bundle_id TEXT NOT NULL,
                title TEXT NOT NULL,
                designer_pcd_id TEXT NOT NULL,
                payload_json TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS sharepoint_reviews (
                session_id TEXT PRIMARY KEY,
                preview_id TEXT NOT NULL,
                client_anonymous_id TEXT NOT NULL,
                verdict TEXT NOT NULL,
                approval_token TEXT,
                payload_json TEXT NOT NULL,
                reviewed_at TEXT NOT NULL
            );
            "#,
        )?;
        Ok(())
    }

    pub fn save_requirement(conn: &Connection, req: &RequirementsPackage) -> Result<(), SharePointError> {
        let payload = serde_json::to_string(req).map_err(|e| SharePointError::Serialization(e.to_string()))?;
        conn.execute(
            r#"
            INSERT OR REPLACE INTO sharepoint_requirements (req_id, client_anonymous_id, title, business_type, payload_json, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            params![req.req_id, req.client_anonymous_id, req.title, req.business_type, payload, req.created_at],
        )?;
        Ok(())
    }

    pub fn get_requirement(conn: &Connection, req_id: &str) -> Result<Option<RequirementsPackage>, SharePointError> {
        let mut stmt = conn.prepare("SELECT payload_json FROM sharepoint_requirements WHERE req_id = ?1")?;
        let mut rows = stmt.query(params![req_id])?;
        if let Some(row) = rows.next()? {
            let json_str: String = row.get(0)?;
            let req: RequirementsPackage = serde_json::from_str(&json_str)
                .map_err(|e| SharePointError::Serialization(e.to_string()))?;
            Ok(Some(req))
        } else {
            Ok(None)
        }
    }

    pub fn save_preview(conn: &Connection, preview: &PreviewBundle) -> Result<(), SharePointError> {
        let payload = serde_json::to_string(preview).map_err(|e| SharePointError::Serialization(e.to_string()))?;
        conn.execute(
            r#"
            INSERT OR REPLACE INTO sharepoint_previews (preview_id, req_id, bundle_id, title, designer_pcd_id, payload_json, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![preview.preview_id, preview.req_id, preview.bundle_id, preview.title, preview.designer_pcd_id, payload, preview.created_at],
        )?;
        Ok(())
    }

    pub fn get_preview(conn: &Connection, preview_id: &str) -> Result<Option<PreviewBundle>, SharePointError> {
        let mut stmt = conn.prepare("SELECT payload_json FROM sharepoint_previews WHERE preview_id = ?1")?;
        let mut rows = stmt.query(params![preview_id])?;
        if let Some(row) = rows.next()? {
            let json_str: String = row.get(0)?;
            let preview: PreviewBundle = serde_json::from_str(&json_str)
                .map_err(|e| SharePointError::Serialization(e.to_string()))?;
            Ok(Some(preview))
        } else {
            Ok(None)
        }
    }

    pub fn save_review(conn: &Connection, review: &ClientReviewSession) -> Result<(), SharePointError> {
        let payload = serde_json::to_string(review).map_err(|e| SharePointError::Serialization(e.to_string()))?;
        conn.execute(
            r#"
            INSERT OR REPLACE INTO sharepoint_reviews (session_id, preview_id, client_anonymous_id, verdict, approval_token, payload_json, reviewed_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![
                review.session_id,
                review.preview_id,
                review.client_anonymous_id,
                review.verdict.as_str(),
                review.approval_token,
                payload,
                review.reviewed_at,
            ],
        )?;
        Ok(())
    }

    pub fn get_review(conn: &Connection, session_id: &str) -> Result<Option<ClientReviewSession>, SharePointError> {
        let mut stmt = conn.prepare("SELECT payload_json FROM sharepoint_reviews WHERE session_id = ?1")?;
        let mut rows = stmt.query(params![session_id])?;
        if let Some(row) = rows.next()? {
            let json_str: String = row.get(0)?;
            let review: ClientReviewSession = serde_json::from_str(&json_str)
                .map_err(|e| SharePointError::Serialization(e.to_string()))?;
            Ok(Some(review))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_requirements_package_serialization_and_tamper_detection() {
        let fields = vec![
            FieldRequirement {
                name: "customer_vat".to_string(),
                data_type: "Text".to_string(),
                is_required: true,
                description: "Greek AFM tax number".to_string(),
            },
            FieldRequirement {
                name: "license_plate".to_string(),
                data_type: "Text".to_string(),
                is_required: false,
                description: "Vehicle registration".to_string(),
            },
        ];
        let workflows = vec![WorkflowIntent {
            title: "Print Intake Slip".to_string(),
            trigger_event: "on_ticket_create".to_string(),
            expected_action: "escpos_thermal_print_80mm".to_string(),
        }];

        let req = RequirementsPackage::new(
            "Auto Workshop Intake",
            "Automotive",
            "anon_client_c8a49f",
            fields,
            workflows,
            "Needs quick search by plate",
        );
        assert!(req.verify_integrity());

        let bytes = req.to_bytes().expect("Should serialize to bytes");
        let decoded = RequirementsPackage::from_bytes(&bytes).expect("Should decode successfully");
        assert_eq!(decoded.title, "Auto Workshop Intake");
        assert_eq!(decoded.fields.len(), 2);

        // Tamper test
        let mut tampered_bytes = bytes.clone();
        tampered_bytes[PRREQ_MAGIC.len() + 10] ^= 0xFF;
        assert!(RequirementsPackage::from_bytes(&tampered_bytes).is_err());
    }

    #[test]
    fn test_preview_bundle_watermark_and_serialization() {
        let preview = PreviewBundle::new(
            Some("req_123".to_string()),
            "pkg_autofix",
            "Autofix Pro V2 Preview",
            "pcd_george",
            r#"{"views":[]}"#,
            r#"{"tables":[]}"#,
        );
        assert!(preview.verify_integrity());
        assert_eq!(preview.watermark_text, DEFAULT_PREVIEW_WATERMARK);

        let bytes = preview.to_bytes().expect("Should serialize preview");
        let decoded = PreviewBundle::from_bytes(&bytes).expect("Should decode preview");
        assert_eq!(decoded.designer_pcd_id, "pcd_george");
        assert_eq!(decoded.bundle_id, "pkg_autofix");
    }

    #[test]
    fn test_client_review_session_verdict_and_approval_token() {
        let items = vec![ReviewFeedbackItem::new("intake_button", "Move 10px right")];
        let session_approved = ClientReviewSession::new(
            "prev_999",
            "anon_client_c8a49f",
            ReviewVerdict::Approved,
            items.clone(),
        );
        assert!(session_approved.approval_token.is_some());
        assert!(session_approved.approval_token.unwrap().starts_with("appr_"));

        let session_revision = ClientReviewSession::new(
            "prev_999",
            "anon_client_c8a49f",
            ReviewVerdict::NeedsRevision,
            items,
        );
        assert!(session_revision.approval_token.is_none());
    }

    #[test]
    fn test_sharepoint_ledger_sqlite_crud() {
        let conn = Connection::open_in_memory().expect("open memory db");
        SharePointLedger::init_schema(&conn).expect("schema init");

        let req = RequirementsPackage::new("Bakery POS", "Bakery", "anon_client_bakery", vec![], vec![], "None");
        SharePointLedger::save_requirement(&conn, &req).expect("save req");
        let fetched_req = SharePointLedger::get_requirement(&conn, &req.req_id)
            .expect("fetch req")
            .expect("should exist");
        assert_eq!(fetched_req.title, "Bakery POS");

        let preview = PreviewBundle::new(Some(req.req_id.clone()), "pkg_bakery", "Bakery Preview", "pcd_alex", "{}", "{}");
        SharePointLedger::save_preview(&conn, &preview).expect("save prev");
        let fetched_prev = SharePointLedger::get_preview(&conn, &preview.preview_id)
            .expect("fetch prev")
            .expect("should exist");
        assert_eq!(fetched_prev.bundle_id, "pkg_bakery");

        let review = ClientReviewSession::new(
            &preview.preview_id,
            "anon_client_bakery",
            ReviewVerdict::Approved,
            vec![],
        );
        SharePointLedger::save_review(&conn, &review).expect("save review");
        let fetched_rev = SharePointLedger::get_review(&conn, &review.session_id)
            .expect("fetch review")
            .expect("should exist");
        assert_eq!(fetched_rev.verdict, ReviewVerdict::Approved);
    }
}
