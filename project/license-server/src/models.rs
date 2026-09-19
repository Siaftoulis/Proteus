use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct License {
    pub id: String,
    pub license_key: String,
    pub customer_email: String,
    pub max_users: i32,
    pub features: String,
    pub issued_at: String,
    pub expires_at: String,
    pub activations: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyRequest {
    pub license_key: String,
    pub machine_id: Option<String>,
    pub activate: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyResponse {
    pub valid: bool,
    pub max_users: i32,
    pub features: Vec<String>,
    pub expires_at: String,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IssueRequest {
    pub customer_email: String,
    pub max_users: i32,
    pub features: Vec<String>,
    pub expires_days: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IssueResponse {
    pub license_key: String,
    pub message: String,
}

pub fn generate_license_key() -> String {
    let parts: Vec<String> = (0..3)
        .map(|_| {
            Uuid::new_v4()
                .to_string()
                .replace('-', "")
                .get(0..8)
                .unwrap()
                .to_uppercase()
        })
        .collect();
    format!("CRM-{}-{}-{}", parts[0], parts[1], parts[2])
}
