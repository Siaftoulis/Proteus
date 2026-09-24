#![cfg_attr(not(test), windows_subsystem = "windows")]

//! Proteus Web & Marketplace Server (proteus-web).
//! Web Portal, Landing Showcase, Gated Compilation API, and Digital Contracts Hub.

mod contracts;
mod marketplace;
mod portal;
mod slot_board;
mod ui;
mod ui_certifications;
mod ui_contracts;
mod ui_css;
mod ui_freelance;
mod ui_landing;
mod ui_marketplace;

use axum::{
    extract::{Extension, Json, Path},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use marketplace::{compile_package_gate, CertificationTier, PackageCompileRequest};
use portal::{calculate_subscription_quote, PricingQuoteRequest};
use slot_board::{AcceptSlotRequest, FloorCalculatorRequest, SlotBoardManager, SubmitBidRequest};
use tower_http::cors::CorsLayer;
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let slot_manager = SlotBoardManager::new();

    let app = Router::new()
        .route("/", get(ui_landing::landing_page_handler))
        .route("/hub", get(ui::index_page_handler))
        .route("/health", get(health_handler))
        .route("/api/v1/tickets", get(tickets_handler))
        .route("/api/v1/contact", post(contact_submit_handler))
        .route("/api/v1/marketplace/compile", post(compile_handler))
        .route("/api/v1/pricing/quote", post(quote_handler))
        .route("/api/v1/certifications/tiers", get(tiers_handler))
        .route("/api/v1/contracts/create", post(contract_create_handler))
        .route("/api/v1/contracts/sign", post(contract_sign_handler))
        .route("/api/v1/contracts/fund", post(contract_fund_handler))
        .route("/api/v1/contracts/payout", post(contract_payout_handler))
        .route("/api/v1/marketplace/packages", get(packages_handler))
        .route("/api/v1/marketplace/slot-boards", get(slot_boards_handler))
        .route("/api/v1/marketplace/slot-boards/:id", get(slot_board_get_handler))
        .route("/api/v1/marketplace/slot-boards/accept", post(slot_board_accept_handler))
        .route("/api/v1/marketplace/slot-boards/bid", post(slot_board_bid_handler))
        .route("/api/v1/pricing/floor-calculator", post(floor_calculator_handler))
        .layer(Extension(slot_manager))
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

async fn tickets_handler() -> impl IntoResponse {
    let db_path = crm_core::paths::get_database_path();
    if let Ok(conn) = rusqlite::Connection::open(&db_path) {
        if let Ok(tickets) = crm_core::tickets::list_tickets(&conn) {
            return (StatusCode::OK, Json(tickets)).into_response();
        }
    }
    (StatusCode::OK, Json(Vec::<crm_core::tickets::ServiceTicket>::new())).into_response()
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

async fn contract_create_handler(
    Json(payload): Json<contracts::CreateContractRequest>,
) -> impl IntoResponse {
    let contract = contracts::SlaContract::new(
        payload.contract_id,
        payload.shop_id,
        payload.technician_id,
        payload.service_scope,
        payload.response_time_hours,
        payload.monthly_retainer_eur,
    );
    (StatusCode::CREATED, Json(contract))
}

async fn contract_sign_handler(
    Json(mut payload): Json<contracts::SignContractRequest>,
) -> impl IntoResponse {
    match payload.signer_role.to_lowercase().as_str() {
        "shop" => payload.contract.sign_by_shop(),
        "technician" | "tech" => payload.contract.sign_by_technician(),
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "Invalid signer role; must be 'shop' or 'technician'" })),
            )
                .into_response()
        }
    }
    (StatusCode::OK, Json(payload.contract)).into_response()
}

async fn contract_fund_handler(
    Json(mut payload): Json<contracts::FundContractRequest>,
) -> impl IntoResponse {
    payload.contract.fund_escrow(payload.amount);
    (StatusCode::OK, Json(payload.contract))
}

async fn contract_payout_handler(
    Json(mut payload): Json<contracts::PayoutContractRequest>,
) -> impl IntoResponse {
    match payload.contract.release_escrow_payout() {
        Ok(payout) => (
            StatusCode::OK,
            Json(serde_json::to_value(contracts::PayoutContractResponse {
                contract: payload.contract,
                payout_eur: payout,
            }).unwrap()),
        )
            .into_response(),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": err })),
        )
            .into_response(),
    }
}

#[derive(serde::Deserialize)]
struct PackagesQuery {
    account: Option<String>,
}

async fn packages_handler(
    axum::extract::Query(query): axum::extract::Query<PackagesQuery>,
) -> impl IntoResponse {
    let email = query.account.unwrap_or_default();
    let packages = if email.trim().is_empty() {
        marketplace::get_all_marketplace_packages()
    } else {
        marketplace::get_licensed_packages_for_account(&email)
    };
    (StatusCode::OK, Json(packages))
}

#[derive(serde::Deserialize)]
struct ContactSubmission {
    name: String,
    email: String,
    phone: Option<String>,
    company: Option<String>,
    service_type: Option<String>,
    message: String,
}

async fn contact_submit_handler(Json(payload): Json<ContactSubmission>) -> impl IntoResponse {
    let service = payload.service_type.as_deref().unwrap_or("General Inquiry");
    let company = payload.company.as_deref().unwrap_or("Independent");
    info!(
        "Lead received: {} <{}> | Company: {} | Service: {} [chars: {}]",
        payload.name,
        payload.email,
        company,
        service,
        payload.message.len()
    );
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "success",
            "message": "Το αίτημά σας καταχωρήθηκε επιτυχώς. Η ομάδα μηχανικών θα επικοινωνήσει μαζί σας σύντομα.",
            "sender": payload.name,
            "email": payload.email,
            "service": service,
            "has_phone": payload.phone.is_some()
        })),
    )
}

async fn slot_boards_handler(Extension(mgr): Extension<SlotBoardManager>) -> impl IntoResponse {
    let boards = mgr.list_boards();
    (StatusCode::OK, Json(boards))
}

async fn slot_board_get_handler(
    Extension(mgr): Extension<SlotBoardManager>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match mgr.get_board(&id) {
        Some(board) => (StatusCode::OK, Json(board)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": format!("Slot board '{}' not found", id) })),
        )
            .into_response(),
    }
}

async fn slot_board_accept_handler(
    Extension(mgr): Extension<SlotBoardManager>,
    Json(payload): Json<AcceptSlotRequest>,
) -> impl IntoResponse {
    match mgr.accept_slot(&payload) {
        Ok(board) => (
            StatusCode::OK,
            Json(serde_json::json!({ "success": true, "board": board })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "success": false, "error": err })),
        )
            .into_response(),
    }
}

async fn slot_board_bid_handler(
    Extension(mgr): Extension<SlotBoardManager>,
    Json(payload): Json<SubmitBidRequest>,
) -> impl IntoResponse {
    match mgr.submit_bid(&payload) {
        Ok(board) => (
            StatusCode::OK,
            Json(serde_json::json!({ "success": true, "board": board })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "success": false, "error": err })),
        )
            .into_response(),
    }
}

async fn floor_calculator_handler(Json(payload): Json<FloorCalculatorRequest>) -> impl IntoResponse {
    let res = SlotBoardManager::evaluate_floor(&payload);
    (StatusCode::OK, Json(res))
}

