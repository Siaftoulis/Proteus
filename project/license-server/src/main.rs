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

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let conn = Connection::open("licenses.db").expect("Failed to open database");
    db::init_db(&conn).expect("Failed to init database");

    let state = AppState {
        conn: std::sync::Arc::new(std::sync::Mutex::new(conn)),
    };

    let app = Router::new()
        .route("/verify", get(verify_license))
        .route("/issue", post(issue_license))
        .route("/health", get(|| async { "ok" }))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = "0.0.0.0:3000";
    info!("License server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind");
    axum::serve(listener, app).await.expect("Server error");
}

async fn verify_license(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<VerifyRequest>,
) -> impl IntoResponse {
    let conn = state.conn.lock().unwrap();

    match db::get_license(&conn, &params.license_key) {
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
                let _ = db::increment_activations(&conn, &params.license_key);
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

async fn issue_license(
    State(state): State<AppState>,
    Json(req): Json<IssueRequest>,
) -> impl IntoResponse {
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
