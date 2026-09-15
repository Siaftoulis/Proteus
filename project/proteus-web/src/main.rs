//! Proteus Web & Marketplace Server (proteus-web).
//! Web Portal, Landing Showcase, Gated Compilation API, and Digital Contracts Hub.

mod contracts;
mod marketplace;
mod portal;

use axum::{
    extract::Json,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use marketplace::{compile_package_gate, CertificationTier, PackageCompileRequest};
use portal::{calculate_subscription_quote, PricingQuoteRequest};
use tower_http::cors::CorsLayer;
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/api/v1/marketplace/compile", post(compile_handler))
        .route("/api/v1/pricing/quote", post(quote_handler))
        .route("/api/v1/certifications/tiers", get(tiers_handler))
        .layer(CorsLayer::permissive());

    let addr = "0.0.0.0:8080";
    info!("Proteus Web Hub & Marketplace listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind port 8080");
    axum::serve(listener, app).await.expect("Server error");
}

async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, "Proteus Web Hub OK")
}

async fn compile_handler(Json(payload): Json<PackageCompileRequest>) -> impl IntoResponse {
    let result = compile_package_gate(&payload);
    if result.success {
        (StatusCode::OK, Json(result))
    } else {
        (StatusCode::FORBIDDEN, Json(result))
    }
}

async fn quote_handler(Json(payload): Json<PricingQuoteRequest>) -> impl IntoResponse {
    let quote = calculate_subscription_quote(&payload);
    (StatusCode::OK, Json(quote))
}

async fn tiers_handler() -> impl IntoResponse {
    let tiers = vec![
        CertificationTier::Tier0Uncertified,
        CertificationTier::Tier1Associate,
        CertificationTier::Tier2Specialist,
        CertificationTier::Tier3Professional,
        CertificationTier::Tier4Practitioner,
        CertificationTier::Tier5Architect,
        CertificationTier::Tier6Principal,
        CertificationTier::Tier7MasterEngineer,
        CertificationTier::Tier8PremierPartner,
        CertificationTier::Tier9EliteAgency,
        CertificationTier::Tier10EnterpriseAgency,
    ];

    let tiers_info: Vec<serde_json::Value> = tiers
        .into_iter()
        .map(|t| {
            serde_json::json!({
                "tier": format!("{:?}", t),
                "platform_commission_pct": t.platform_commission_pct(),
                "partner_share_pct": t.partner_share_pct(),
                "monthly_badge_fee_eur": t.monthly_badge_fee(),
            })
        })
        .collect();

    (StatusCode::OK, Json(tiers_info))
}
