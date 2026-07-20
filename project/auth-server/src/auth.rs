use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use tracing::instrument;

use crate::models::TokenClaims;

fn jwt_secret() -> String {
    std::env::var("JWT_SECRET").unwrap_or_else(|_| "crm-builder-dev-secret".to_string())
}
const ACCESS_TOKEN_EXPIRY: usize = 3600;

#[instrument(skip(password))]
pub fn hash_password(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| format!("Failed to hash password: {}", e))?;
    Ok(hash.to_string())
}

#[instrument(skip(password, hash))]
pub fn verify_password(password: &str, hash: &str) -> Result<bool, String> {
    let parsed_hash =
        PasswordHash::new(hash).map_err(|e| format!("Invalid password hash: {}", e))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

#[instrument(skip(user_id, email))]
pub fn create_access_token(user_id: &str, email: &str) -> Result<String, String> {
    let now = chrono::Utc::now().timestamp() as usize;
    let claims = TokenClaims {
        sub: user_id.to_string(),
        email: email.to_string(),
        iat: now,
        exp: now + ACCESS_TOKEN_EXPIRY,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret().as_bytes()),
    )
    .map_err(|e| format!("Failed to create token: {}", e))
}

pub fn verify_access_token(token: &str) -> Result<TokenClaims, String> {
    let token_data = decode::<TokenClaims>(
        token,
        &DecodingKey::from_secret(jwt_secret().as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| format!("Invalid token: {}", e))?;
    Ok(token_data.claims)
}

pub fn generate_refresh_token() -> String {
    use rand::Rng;
    let bytes: [u8; 32] = rand::rngs::OsRng.gen();
    hex::encode(bytes)
}

pub struct GoogleUserInfo {
    pub sub: String,
    pub email: String,
    pub name: String,
    pub picture: String,
}

async fn google_client_id() -> Result<String, String> {
    std::env::var("GOOGLE_CLIENT_ID").map_err(|_| "GOOGLE_CLIENT_ID not set".to_string())
}

async fn google_client_secret() -> Result<String, String> {
    std::env::var("GOOGLE_CLIENT_SECRET").map_err(|_| "GOOGLE_CLIENT_SECRET not set".to_string())
}

#[instrument(skip(code, redirect_uri))]
pub async fn exchange_google_code(code: &str, redirect_uri: &str) -> Result<GoogleUserInfo, String> {
    let client_id = google_client_id().await?;
    let client_secret = google_client_secret().await?;

    let client = reqwest::Client::new();
    let token_resp = client
        .post("https://oauth2.googleapis.com/token")
        .form(&[
            ("code", code),
            ("client_id", &client_id),
            ("client_secret", &client_secret),
            ("redirect_uri", redirect_uri),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await
        .map_err(|e| format!("Google token exchange failed: {}", e))?;

    let token_data: serde_json::Value = token_resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse token response: {}", e))?;

    let access_token = token_data["access_token"]
        .as_str()
        .ok_or_else(|| "No access_token in Google response".to_string())?;

    let user_resp = client
        .get("https://www.googleapis.com/oauth2/v2/userinfo")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("Google userinfo failed: {}", e))?;

    let user_data: serde_json::Value = user_resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse userinfo: {}", e))?;

    Ok(GoogleUserInfo {
        sub: user_data["id"].as_str().unwrap_or("").to_string(),
        email: user_data["email"].as_str().unwrap_or("").to_string(),
        name: user_data["name"].as_str().unwrap_or("").to_string(),
        picture: user_data["picture"].as_str().unwrap_or("").to_string(),
    })
}
