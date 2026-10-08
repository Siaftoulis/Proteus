mod db;
mod models;

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{Days, Utc};
use rusqlite::Connection;
use tower_http::cors::CorsLayer;
use tracing::info;

use models::*;

#[derive(Clone)]
struct AppState {
    conn: std::sync::Arc<std::sync::Mutex<Connection>>,
}

fn admin_secret() -> String {
    match std::env::var("ADMIN_SECRET") {
        Ok(s) if !s.trim().is_empty() => s,
        _ => {
            #[cfg(debug_assertions)]
            {
                "license-admin-secret".to_string()
            }
            #[cfg(not(debug_assertions))]
            {
                static RUNTIME_ADMIN_KEY: std::sync::OnceLock<String> = std::sync::OnceLock::new();
                RUNTIME_ADMIN_KEY
                    .get_or_init(|| format!("adm-{}", uuid::Uuid::new_v4()))
                    .clone()
            }
        }
    }
}

fn check_admin_auth(headers: &axum::http::HeaderMap) -> bool {
    let expected = admin_secret();
    if let Some(auth_val) = headers.get(axum::http::header::AUTHORIZATION).and_then(|h| h.to_str().ok()) {
        if let Some(token) = auth_val.strip_prefix("Bearer ").or_else(|| auth_val.strip_prefix("bearer ")) {
            if token.trim() == expected {
                return true;
            }
        }
    }
    if let Some(key_val) = headers.get("X-Admin-Key").and_then(|h| h.to_str().ok()) {
        if key_val.trim() == expected {
            return true;
        }
    }
    false
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let conn = Connection::open("licenses.db").expect("Failed to open database");
    db::init_db(&conn).expect("Failed to init database");

    let state = AppState {
        conn: std::sync::Arc::new(std::sync::Mutex::new(conn)),
    };

    let cors = {
        let allowed = std::env::var("ALLOWED_ORIGINS").unwrap_or_default();
        if allowed.is_empty() {
            CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
                .allow_headers([axum::http::header::AUTHORIZATION, axum::http::header::CONTENT_TYPE])
        } else {
            let origins: Vec<_> = allowed
                .split(',')
                .filter_map(|s| s.trim().parse().ok())
                .collect();
            CorsLayer::new()
                .allow_origin(tower_http::cors::AllowOrigin::list(origins))
                .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
                .allow_headers([axum::http::header::AUTHORIZATION, axum::http::header::CONTENT_TYPE])
        }
    };

    let app = Router::new()
        .route("/verify", get(verify_license).post(verify_license_post))
        .route("/issue", post(issue_license))
        .route("/health", get(|| async { "ok" }))
        .layer(cors)
        .with_state(state);

    let addr = "0.0.0.0:3000";
    info!("License server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind");
    axum::serve(listener, app).await.expect("Server error");
}

fn verify_license_core(conn: &Connection, params: &VerifyRequest) -> (StatusCode, Json<VerifyResponse>) {
    match db::get_license(conn, &params.license_key) {
        Ok(Some(license)) => {
            let now = Utc::now().naive_utc();
            let expires = match chrono::NaiveDateTime::parse_from_str(
                &license.expires_at,
                "%Y-%m-%dT%H:%M:%S",
            ) {
                Ok(dt) => dt,
                Err(_) => {
                    return (
                        StatusCode::OK,
                        Json(VerifyResponse {
                            valid: false,
                            max_users: 3,
                            features: vec![],
                            expires_at: license.expires_at,
                            reason: Some("Invalid expiry date format".to_string()),
                        }),
                    )
                }
            };

            if now > expires {
                return (
                    StatusCode::OK,
                    Json(VerifyResponse {
                        valid: false,
                        max_users: 3,
                        features: vec![],
                        expires_at: license.expires_at,
                        reason: Some("License expired".to_string()),
                    }),
                );
            }

            if params.activate.unwrap_or(false) {
                if license.activations >= license.max_users {
                    return (
                        StatusCode::OK,
                        Json(VerifyResponse {
                            valid: false,
                            max_users: license.max_users,
                            features: vec![],
                            expires_at: license.expires_at,
                            reason: Some("Maximum activations reached".to_string()),
                        }),
                    );
                }
                let _ = db::increment_activations(conn, &params.license_key);
            }

            let features: Vec<String> =
                serde_json::from_str(&license.features).unwrap_or_default();

            (
                StatusCode::OK,
                Json(VerifyResponse {
                    valid: true,
                    max_users: license.max_users,
                    features,
                    expires_at: license.expires_at,
                    reason: None,
                }),
            )
        }
        Ok(None) => (
            StatusCode::OK,
            Json(VerifyResponse {
                valid: false,
                max_users: 3,
                features: vec![],
                expires_at: String::new(),
                reason: Some("License key not found".to_string()),
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(VerifyResponse {
                valid: false,
                max_users: 3,
                features: vec![],
                expires_at: String::new(),
                reason: Some(format!("Server error: {}", e)),
            }),
        ),
    }
}

async fn verify_license(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<VerifyRequest>,
) -> impl IntoResponse {
    let conn = state.conn.lock().unwrap();
    verify_license_core(&conn, &params)
}

async fn verify_license_post(
    State(state): State<AppState>,
    Json(params): Json<VerifyRequest>,
) -> impl IntoResponse {
    let conn = state.conn.lock().unwrap();
    verify_license_core(&conn, &params)
}

async fn issue_license(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<IssueRequest>,
) -> impl IntoResponse {
    if !check_admin_auth(&headers) {
        return (
            StatusCode::UNAUTHORIZED,
            Json(IssueResponse {
                license_key: String::new(),
                message: "Unauthorized: valid admin API key (Authorization: Bearer <key> or X-Admin-Key) required".to_string(),
            }),
        );
    }

    let conn = state.conn.lock().unwrap();

    let license_key = generate_license_key();
    let now = Utc::now().naive_utc();
    let expires = now
        .checked_add_days(Days::new(req.expires_days as u64))
        .unwrap();

    let license = License {
        id: uuid::Uuid::new_v4().to_string(),
        license_key: license_key.clone(),
        customer_email: req.customer_email,
        max_users: req.max_users,
        features: serde_json::to_string(&req.features).unwrap_or_else(|_| "[]".to_string()),
        issued_at: now.format("%Y-%m-%dT%H:%M:%S").to_string(),
        expires_at: expires.format("%Y-%m-%dT%H:%M:%S").to_string(),
        activations: 0,
    };

    match db::insert_license(&conn, &license) {
        Ok(()) => (
            StatusCode::CREATED,
            Json(IssueResponse {
                license_key,
                message: "License issued successfully".to_string(),
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(IssueResponse {
                license_key: String::new(),
                message: format!("Failed to issue license: {}", e),
            }),
        ),
    }
}
