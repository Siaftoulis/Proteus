use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer, AllowOrigin};

use auth_server::auth;
use auth_server::db::AppDb;
use auth_server::models::*;

struct AppState {
    db: AppDb,
    rate_limiter: RateLimitState,
}

fn err(status: StatusCode, msg: &str) -> (StatusCode, Json<serde_json::Value>) {
    (status, Json(json!({"error": msg})))
}

fn internal(e: String) -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e})))
}

// ponytail: global in-memory rate limiter, per-IP sliding window
#[derive(Clone)]
struct RateLimitState {
    requests: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    max_per_minute: usize,
}

impl RateLimitState {
    fn new(max_per_minute: usize) -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            max_per_minute,
        }
    }

    async fn check(&self, ip: &str) -> bool {
        let mut map = self.requests.lock().await;
        let now = Instant::now();
        let window = Duration::from_secs(60);
        let entry = map.entry(ip.to_string()).or_default();
        entry.retain(|t| now.duration_since(*t) < window);
        if entry.len() >= self.max_per_minute {
            return false;
        }
        entry.push(now);
        true
    }
}

#[allow(dead_code)]
fn require_role(user: &User, allowed_roles: &[&str]) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    if !allowed_roles.contains(&user.role.as_str()) {
        return Err(err(StatusCode::FORBIDDEN, &format!("Role '{}' not authorized. Required: {:?}", user.role, allowed_roles)));
    }
    Ok(())
}

fn extract_ip(headers: &axum::http::HeaderMap) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string()
}

fn extract_bearer_token(headers: &axum::http::HeaderMap) -> Option<String> {
    let auth_header = headers.get(axum::http::header::AUTHORIZATION)?.to_str().ok()?;
    auth_header.strip_prefix("Bearer ").or_else(|| auth_header.strip_prefix("bearer ")).map(|s| s.trim().to_string())
}

async fn check_license(db: &AppDb, user_id: &str) -> LicenseStatus {
    let license_key = match db.get_license_key(user_id) {
        Ok(Some(k)) => k,
        _ => return LicenseStatus {
            tier: "free".to_string(),
            valid: true,
            max_users: 1,
            expires_at: None,
        },
    };

    let resp = reqwest::get(format!("http://127.0.0.1:3000/verify?license_key={}", license_key))
        .await;

    match resp {
        Ok(r) => {
            if let Ok(data) = r.json::<serde_json::Value>().await {
                let valid = data["valid"].as_bool().unwrap_or(false);
                let max_users = data["max_users"].as_i64().unwrap_or(1) as i32;
                let expires = data["expires_at"].as_str().map(|s| s.to_string());
                let tier = if valid { "paid" } else { "free" };
                return LicenseStatus { tier: tier.to_string(), valid, max_users, expires_at: expires };
            }
            LicenseStatus { tier: "free".to_string(), valid: true, max_users: 1, expires_at: None }
        }
        Err(_) => LicenseStatus { tier: "free".to_string(), valid: true, max_users: 1, expires_at: None },
    }
}

async fn make_auth_response(db: &AppDb, user: &User) -> Result<Json<AuthResponse>, (StatusCode, Json<serde_json::Value>)> {
    let access_token = auth::create_access_token(&user.id, &user.email).map_err(internal)?;
    let refresh_token = auth::generate_refresh_token();
    db.store_refresh_token(&user.id, &refresh_token).map_err(internal)?;
    let license = check_license(db, &user.id).await;
    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        user: user.clone(),
        license,
    }))
}

async fn register(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<serde_json::Value>)> {
    let ip = extract_ip(&headers);
    if !state.rate_limiter.check(&ip).await {
        let _ = state.db.log_auth_event("RATE_LIMIT_EXCEEDED", None, &ip, "endpoint=register");
        return Err(err(StatusCode::TOO_MANY_REQUESTS, "Too many requests. Try again later."));
    }
    if req.email.is_empty() || req.password.is_empty() || req.name.is_empty() {
        return Err(err(StatusCode::BAD_REQUEST, "Email, password, and name are required"));
    }

    let existing = state.db.get_user_by_email(&req.email).map_err(internal)?;
    if existing.is_some() {
        let _ = state.db.log_auth_event("REGISTER_FAILED", None, &ip, &format!("email={} reason=conflict", req.email));
        return Err(err(StatusCode::CONFLICT, "Email already registered"));
    }

    let password_hash = auth::hash_password(&req.password).map_err(internal)?;
    let user = state.db.create_user(&CreateUserRequest {
        email: req.email.clone(),
        password: Some(password_hash),
        name: req.name.clone(),
        provider: "email".to_string(),
        provider_id: req.email.clone(),
        avatar_url: None,
        role: None,
    }).map_err(internal)?;

    let _ = state.db.log_auth_event("REGISTER_SUCCESS", Some(&user.id), &ip, &format!("email={}", user.email));
    make_auth_response(&state.db, &user).await
}

async fn login(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<serde_json::Value>)> {
    let ip = extract_ip(&headers);
    if !state.rate_limiter.check(&ip).await {
        let _ = state.db.log_auth_event("RATE_LIMIT_EXCEEDED", None, &ip, "endpoint=login");
        return Err(err(StatusCode::TOO_MANY_REQUESTS, "Too many requests. Try again later."));
    }
    let user_match = state.db.get_user_by_email(&req.email).map_err(internal)?;
    if user_match.is_none() {
        let _ = state.db.log_auth_event("LOGIN_FAILED", None, &ip, &format!("email={} reason=not_found", req.email));
        return Err(err(StatusCode::UNAUTHORIZED, "Invalid email or password"));
    }
    let (user, password_hash) = user_match.unwrap();

    let hash = match password_hash {
        Some(h) => h,
        None => {
            let _ = state.db.log_auth_event("LOGIN_FAILED", Some(&user.id), &ip, &format!("email={} reason=google_account", req.email));
            return Err(err(StatusCode::UNAUTHORIZED, "This account uses Google login. Please sign in with Google."));
        }
    };

    let valid = auth::verify_password(&req.password, &hash).map_err(internal)?;
    if !valid {
        let _ = state.db.log_auth_event("LOGIN_FAILED", Some(&user.id), &ip, &format!("email={} reason=bad_password", req.email));
        return Err(err(StatusCode::UNAUTHORIZED, "Invalid email or password"));
    }

    let _ = state.db.log_auth_event("LOGIN_SUCCESS", Some(&user.id), &ip, &format!("email={}", user.email));
    make_auth_response(&state.db, &user).await
}

async fn refresh(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<serde_json::Value>)> {
    let user_id = state.db.validate_refresh_token(&req.refresh_token).map_err(internal)?
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, "Invalid or expired refresh token"))?;

    state.db.delete_refresh_token(&req.refresh_token).map_err(internal)?;

    let user = state.db.get_user_by_id(&user_id).map_err(internal)?
        .ok_or_else(|| err(StatusCode::NOT_FOUND, "User not found"))?;

    make_auth_response(&state.db, &user).await
}

async fn google_login(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<GoogleAuthRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<serde_json::Value>)> {
    if !state.rate_limiter.check(&extract_ip(&headers)).await {
        return Err(err(StatusCode::TOO_MANY_REQUESTS, "Too many requests. Try again later."));
    }
    let google_user = auth::exchange_google_code(&req.code, &req.redirect_uri)
        .await
        .map_err(|e| err(StatusCode::UNAUTHORIZED, &e))?;

    let user = match state.db.get_user_by_provider("google", &google_user.sub).map_err(internal)? {
        Some(u) => u,
        None => {
            state.db.create_user(&CreateUserRequest {
                email: google_user.email.clone(),
                password: None,
                name: google_user.name.clone(),
                provider: "google".to_string(),
                provider_id: google_user.sub.clone(),
                avatar_url: Some(google_user.picture.clone()),
                role: None,
            }).map_err(internal)?
        }
    };

    make_auth_response(&state.db, &user).await
}

async fn verify(Json(req): Json<serde_json::Value>) -> Json<serde_json::Value> {
    let token = req.get("access_token").and_then(|v| v.as_str()).unwrap_or("");
    match auth::verify_access_token(token) {
        Ok(claims) => Json(json!({
            "valid": true,
            "user_id": claims.sub,
            "email": claims.email,
        })),
        Err(e) => Json(json!({
            "valid": false,
            "reason": e,
        })),
    }
}

async fn latest_version(
    State(state): State<Arc<AppState>>,
) -> Result<Json<VersionInfo>, (StatusCode, Json<serde_json::Value>)> {
    match state.db.get_latest_version() {
        Ok(Some(v)) => Ok(Json(v)),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({"error": "No version available"})))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e})))),
    }
}

async fn download_version(
    _state: State<Arc<AppState>>,
    Path(_version): Path<String>,
) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    Err((StatusCode::SERVICE_UNAVAILABLE, Json(json!({"error": "Binary distribution not yet configured"}))))
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({"status": "ok", "service": "auth-server", "version": "0.1.0"}))
}

async fn delete_user_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(user_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let ip = extract_ip(&headers);
    let token = extract_bearer_token(&headers)
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, "Missing authorization token"))?;
    let claims = auth::verify_access_token(&token)
        .map_err(|e| err(StatusCode::UNAUTHORIZED, &e))?;

    let caller = state.db.get_user_by_id(&claims.sub).map_err(internal)?
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, "User not found"))?;

    let is_authorized = claims.sub == user_id || caller.role == "ceo" || caller.role == "admin";
    if !is_authorized {
        let _ = state.db.log_auth_event("UNAUTHORIZED_DELETION_ATTEMPT", Some(&claims.sub), &ip, &format!("target_user={}", user_id));
        return Err(err(StatusCode::FORBIDDEN, "Not authorized to delete this user profile"));
    }

    let deleted = state.db.delete_user(&user_id).map_err(internal)?;
    if !deleted {
        return Err(err(StatusCode::NOT_FOUND, "User not found"));
    }

    let _ = state.db.log_auth_event("USER_DELETED_GDPR", Some(&claims.sub), &ip, &format!("deleted_user={}", user_id));
    Ok(Json(json!({
        "success": true,
        "message": "User personal data permanently erased per GDPR Art. 17 / Law 4624/2019"
    })))
}

async fn export_user_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Path(user_id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let ip = extract_ip(&headers);
    let token = extract_bearer_token(&headers)
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, "Missing authorization token"))?;
    let claims = auth::verify_access_token(&token)
        .map_err(|e| err(StatusCode::UNAUTHORIZED, &e))?;

    let caller = state.db.get_user_by_id(&claims.sub).map_err(internal)?
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, "User not found"))?;

    let is_authorized = claims.sub == user_id || caller.role == "ceo" || caller.role == "admin";
    if !is_authorized {
        let _ = state.db.log_auth_event("UNAUTHORIZED_EXPORT_ATTEMPT", Some(&claims.sub), &ip, &format!("target_user={}", user_id));
        return Err(err(StatusCode::FORBIDDEN, "Not authorized to export this user data"));
    }

    let export = state.db.export_user_data(&user_id).map_err(internal)?
        .ok_or_else(|| err(StatusCode::NOT_FOUND, "User not found"))?;

    let _ = state.db.log_auth_event("USER_EXPORT_GDPR", Some(&claims.sub), &ip, &format!("exported_user={}", user_id));
    Ok(Json(export))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    // ponytail: dev fallback if JWT_SECRET not set, but warn loudly
    if std::env::var("JWT_SECRET").is_err() {
        tracing::warn!("JWT_SECRET not set — using dev fallback. Set JWT_SECRET in production.");
    }

    let db = AppDb::new("data/auth.db").expect("Failed to open database");
    let state = Arc::new(AppState { db, rate_limiter: RateLimitState::new(30) });

    let cors = {
        let allowed = std::env::var("ALLOWED_ORIGINS").unwrap_or_default();
        if allowed.is_empty() {
            CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any)
        } else {
            let origins: Vec<_> = allowed
                .split(',')
                .filter_map(|s| s.trim().parse().ok())
                .collect();
            CorsLayer::new().allow_origin(AllowOrigin::list(origins)).allow_methods(Any).allow_headers(Any)
        }
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login))
        .route("/api/auth/refresh", post(refresh))
        .route("/api/auth/google", post(google_login))
        .route("/api/auth/verify", post(verify))
        .route("/api/auth/users/{id}", axum::routing::delete(delete_user_handler))
        .route("/api/auth/users/{id}/export", get(export_user_handler))
        .route("/api/distro/latest-version", get(latest_version))
        .route("/api/distro/download/{version}", get(download_version))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001")
        .await
        .expect("Failed to bind port 3001");
    tracing::info!("Auth server listening on 0.0.0.0:3001");
    axum::serve(listener, app).await.unwrap();
}
