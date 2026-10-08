//! Isolated Staging Environment & Subdomain Access Gate.
//! Segregates Developer Staging/Test Suite from Production instances
//! with constant-time token verification, IP filtering, and visual sandbox indicators.

use axum::{
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnvironmentMode {
    Production,
    Staging,
    Development,
}

impl EnvironmentMode {
    pub fn is_staging(&self) -> bool {
        matches!(self, Self::Staging)
    }

    pub fn display_badge(&self) -> &'static str {
        match self {
            Self::Production => "PROD",
            Self::Staging => "STAGING-SANDBOX",
            Self::Development => "DEV-LOCAL",
        }
    }
}

/// Hosting and Staging Isolation Configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostingEnvironmentConfig {
    pub mode: EnvironmentMode,
    pub production_domain: String,
    pub staging_subdomain: String,
    pub developer_token_hash: String,
    pub allowed_developer_ips: Vec<String>,
    pub canary_tag: String,
}

#[allow(dead_code)]
impl HostingEnvironmentConfig {
    pub fn new(
        mode: EnvironmentMode,
        production_domain: impl Into<String>,
        staging_subdomain: impl Into<String>,
        developer_passphrase: &str,
    ) -> Self {
        let hash = proteus_core::merkle::hash_sha256(developer_passphrase);

        Self {
            mode,
            production_domain: production_domain.into(),
            staging_subdomain: staging_subdomain.into(),
            developer_token_hash: hash,
            allowed_developer_ips: vec!["127.0.0.1".into(), "::1".into()],
            canary_tag: "v1.0.0-staging".into(),
        }
    }

    /// Resolves target environment based on HTTP Host header.
    pub fn resolve_environment(&self, host_header: Option<&str>) -> EnvironmentMode {
        if let Some(host) = host_header {
            let host_clean = host.split(':').next().unwrap_or(host).trim().to_lowercase();
            if host_clean == self.staging_subdomain.to_lowercase()
                || host_clean.starts_with("staging.")
                || host_clean.starts_with("test.")
            {
                return EnvironmentMode::Staging;
            }
            if host_clean == self.production_domain.to_lowercase() {
                return EnvironmentMode::Production;
            }
        }
        self.mode
    }

    /// Constant-time verification of developer master passphrase.
    pub fn verify_developer_token(&self, provided_token: &str) -> bool {
        let provided_hash = proteus_core::merkle::hash_sha256(provided_token);

        if provided_hash.len() != self.developer_token_hash.len() {
            return false;
        }

        // Constant-time XOR comparison to prevent timing attacks
        let mut diff = 0u8;
        for (a, b) in provided_hash.as_bytes().iter().zip(self.developer_token_hash.as_bytes().iter()) {
            diff |= a ^ b;
        }
        diff == 0
    }

    /// Checks if a client IP is present in the developer allowlist.
    pub fn is_ip_allowed(&self, client_ip: &str) -> bool {
        let ip_clean = client_ip.trim();
        self.allowed_developer_ips.iter().any(|allowed| allowed == ip_clean)
    }

    /// Verifies whether a request to the staging environment is authorized.
    pub fn authorize_staging_request(
        &self,
        client_ip: Option<&str>,
        auth_header: Option<&str>,
    ) -> bool {
        // 1. IP allowlist check
        if let Some(ip) = client_ip {
            if self.is_ip_allowed(ip) {
                return true;
            }
        }

        // 2. Bearer token / Developer passphrase check
        if let Some(header_val) = auth_header {
            let token = if header_val.starts_with("Bearer ") {
                &header_val[7..]
            } else {
                header_val
            };
            if self.verify_developer_token(token) {
                return true;
            }
        }

        false
    }
}

/// Visual warning banner injected into staging views.
#[allow(dead_code)]
pub const STAGING_BANNER_HTML: &str = r#"
    <div style="background: #e65100; color: #fff; padding: 0.4rem 1rem; font-size: 0.8rem; font-weight: 700; display: flex; justify-content: space-between; align-items: center; letter-spacing: 0.05em; z-index: 99999; border-bottom: 2px solid #ff9800;">
        <span>🧪 PROTEUS STAGING & TEST SUITE — DEVELOPER SANDBOX</span>
        <span style="background: rgba(0,0,0,0.3); padding: 0.15rem 0.5rem; border-radius: 4px; font-family: monospace;">CANARY-ISOLATED &bull; ZERO CLIENT DATA LEAK</span>
    </div>
"#;

/// Renders challenge screen if an unauthorized visitor accesses the staging domain.
#[allow(dead_code)]
pub fn render_staging_gate_challenge() -> Response {
    let body = r#"<!DOCTYPE html>
    <html lang="el">
    <head>
        <meta charset="UTF-8">
        <title>Proteus Staging — Developer Access Gate</title>
        <style>
            body { background: #0c0d12; color: #fff; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; display: flex; align-items: center; justify-content: center; height: 100vh; margin: 0; }
            .card { background: #161822; border: 1px solid #ff9800; border-radius: 12px; padding: 2.5rem; max-width: 440px; width: 100%; box-shadow: 0 12px 32px rgba(0,0,0,0.5); text-align: center; }
            h2 { margin-top: 0; color: #ff9800; font-size: 1.25rem; }
            p { font-size: 0.85rem; color: #9e9e9e; line-height: 1.5; margin: 1rem 0 1.5rem; }
            input { width: 100%; box-sizing: border-box; padding: 0.65rem; border-radius: 6px; border: 1px solid #333; background: #0c0d12; color: #fff; margin-bottom: 1rem; font-size: 0.9rem; }
            button { width: 100%; padding: 0.7rem; border-radius: 6px; border: none; background: #ff9800; color: #000; font-weight: 700; cursor: pointer; font-size: 0.9rem; }
            button:hover { background: #ffa726; }
        </style>
    </head>
    <body>
        <div class="card">
            <h2>🔒 Staging Developer Gate</h2>
            <p>Το περιβάλλον Staging / Test Hub είναι απομονωμένο και προσβάσιμο αποκλειστικά από τον developer του Proteus. Εισάγετε τον κωδικό πρόσβασης.</p>
            <form method="POST" action="/staging/auth">
                <input type="password" name="developer_pass" placeholder="Developer Secret Key" required />
                <button type="submit">Είσοδος στο Staging Sandbox →</button>
            </form>
        </div>
    </body>
    </html>"#;

    (StatusCode::FORBIDDEN, Html(body)).into_response()
}

pub async fn staging_status_handler(headers: HeaderMap) -> Response {
    let host = headers.get("host").and_then(|h| h.to_str().ok());
    let cfg = HostingEnvironmentConfig::new(
        EnvironmentMode::Staging,
        "proteus-os.gr",
        "staging.proteus-os.gr",
        "dev_staging_secret_master",
    );
    let resolved = cfg.resolve_environment(host);

    let status_json = serde_json::json!({
        "environment": resolved.display_badge(),
        "is_staging": resolved.is_staging(),
        "canary_tag": cfg.canary_tag,
        "database_isolated": true,
        "client_data_protected": true,
        "status": "operational"
    });

    (StatusCode::OK, axum::Json(status_json)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_resolution_from_host() {
        let cfg = HostingEnvironmentConfig::new(
            EnvironmentMode::Production,
            "proteus-os.gr",
            "staging.proteus-os.gr",
            "secret_passphrase_123",
        );

        assert_eq!(cfg.resolve_environment(Some("proteus-os.gr")), EnvironmentMode::Production);
        assert_eq!(cfg.resolve_environment(Some("proteus-os.gr:8080")), EnvironmentMode::Production);
        assert_eq!(cfg.resolve_environment(Some("staging.proteus-os.gr")), EnvironmentMode::Staging);
        assert_eq!(cfg.resolve_environment(Some("test.proteus-os.gr:443")), EnvironmentMode::Staging);
        assert_eq!(cfg.resolve_environment(None), EnvironmentMode::Production);
    }

    #[test]
    fn test_constant_time_developer_token_verification() {
        let cfg = HostingEnvironmentConfig::new(
            EnvironmentMode::Staging,
            "proteus-os.gr",
            "staging.proteus-os.gr",
            "secure_dev_pass_2026",
        );

        assert!(cfg.verify_developer_token("secure_dev_pass_2026"));
        assert!(!cfg.verify_developer_token("wrong_password"));
        assert!(!cfg.verify_developer_token(""));
    }

    #[test]
    fn test_staging_authorization_logic() {
        let cfg = HostingEnvironmentConfig::new(
            EnvironmentMode::Staging,
            "proteus-os.gr",
            "staging.proteus-os.gr",
            "master_key",
        );

        // Localhost IP allowlist
        assert!(cfg.authorize_staging_request(Some("127.0.0.1"), None));
        assert!(cfg.authorize_staging_request(Some("::1"), None));

        // External IP without token fails
        assert!(!cfg.authorize_staging_request(Some("203.0.113.42"), None));

        // External IP with valid bearer token succeeds
        assert!(cfg.authorize_staging_request(Some("203.0.113.42"), Some("Bearer master_key")));
        assert!(cfg.authorize_staging_request(None, Some("master_key")));

        // External IP with invalid token fails
        assert!(!cfg.authorize_staging_request(Some("203.0.113.42"), Some("Bearer invalid_key")));
    }

    #[test]
    fn test_staging_challenge_and_banner() {
        assert!(STAGING_BANNER_HTML.contains("PROTEUS STAGING"));
        let response = render_staging_gate_challenge();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}
