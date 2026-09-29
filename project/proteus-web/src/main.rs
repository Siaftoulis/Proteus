#![cfg_attr(not(test), windows_subsystem = "windows")]

//! Proteus Web & Marketplace Server (proteus-web).
//! Web Portal, Landing Showcase, Gated Compilation API, and Digital Contracts Hub.

mod contracts;
mod domains_hosting;
mod marketplace;
mod portal;
mod slot_board;
mod ui;
mod ui_certifications;
mod ui_contracts;
mod ui_css;
mod ui_domains_hosting;
mod ui_freelance;
mod ui_landing;
mod ui_marketplace;
mod ui_telemetry;

use axum::{
    extract::{Extension, Json, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use domains_hosting::{
    attach_extra_database, init_domains_hosting_tables, list_client_domains, list_client_hosting,
    register_domain, search_domain_availability, subscribe_hosting, DomainRegisterRequest,
    HostingSubscribeRequest, HostingTier,
};
use marketplace::{compile_package_gate, CertificationTier, ClientProjectBrief, PackageCompileRequest};
use portal::{calculate_subscription_quote, PricingQuoteRequest};
use slot_board::{AcceptSlotRequest, FloorCalculatorRequest, SlotBoardManager, SubmitBidRequest};
use tower_http::cors::CorsLayer;
use tracing::info;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Ensure database tables for domains and hosting are initialized
    let db_path = proteus_core::paths::get_database_path();
    if let Ok(conn) = rusqlite::Connection::open(&db_path) {
        let _ = init_domains_hosting_tables(&conn);
    }

    let slot_manager = SlotBoardManager::new();

    let app = Router::new()
        .route("/", get(ui_landing::landing_page_handler))
        .route("/hub", get(ui::index_page_handler))
        .route("/assets/logo.png", get(logo_png_handler))
        .route("/assets/logo.svg", get(logo_svg_handler))
        .route("/favicon.ico", get(favicon_handler))
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
        .route("/api/v1/marketplace/submit-brief", post(submit_brief_handler))
        .route("/api/v1/marketplace/slot-boards", get(slot_boards_handler))
        .route("/api/v1/marketplace/slot-boards/:id", get(slot_board_get_handler))
        .route("/api/v1/marketplace/slot-boards/accept", post(slot_board_accept_handler))
        .route("/api/v1/marketplace/slot-boards/bid", post(slot_board_bid_handler))
        .route("/api/v1/pricing/floor-calculator", post(floor_calculator_handler))
        .route("/api/v1/domains/search", post(domains_search_handler))
        .route("/api/v1/domains/register", post(domains_register_handler))
        .route("/api/v1/domains/list", get(domains_list_handler))
        .route("/api/v1/hosting/plans", get(hosting_plans_handler))
        .route("/api/v1/hosting/subscribe", post(hosting_subscribe_handler))
        .route("/api/v1/hosting/list", get(hosting_list_handler))
        .route("/api/v1/hosting/provision-db", post(hosting_provision_db_handler))
        .route("/api/v1/backups/list", get(backups_list_handler))
        .route("/api/v1/backups/create", post(backups_create_handler))
        .route("/api/v1/backups/download/:filename", get(backups_download_handler))
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

async fn logo_png_handler() -> impl IntoResponse {
    (
        [(axum::http::header::CONTENT_TYPE, "image/png")],
        &include_bytes!("../../assets/proteus_emblem_256.png")[..],
    )
}

async fn logo_svg_handler() -> impl IntoResponse {
    (
        [(axum::http::header::CONTENT_TYPE, "image/svg+xml")],
        &include_bytes!("../../assets/proteus_king_emblem.svg")[..],
    )
}

async fn favicon_handler() -> impl IntoResponse {
    (
        [(axum::http::header::CONTENT_TYPE, "image/x-icon")],
        &include_bytes!("../../assets/favicon.ico")[..],
    )
}

async fn tickets_handler() -> impl IntoResponse {
    let db_path = proteus_core::paths::get_database_path();
    if let Ok(conn) = rusqlite::Connection::open(&db_path) {
        if let Ok(tickets) = proteus_core::tickets::list_tickets(&conn) {
            return (StatusCode::OK, Json(tickets)).into_response();
        }
    }
    (StatusCode::OK, Json(Vec::<proteus_core::tickets::ServiceTicket>::new())).into_response()
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

#[derive(serde::Deserialize)]
struct DomainSearchQuery {
    query: String,
}

#[derive(serde::Deserialize)]
struct ClientQuery {
    client_id: Option<String>,
}

#[derive(serde::Deserialize)]
struct ProvisionDbRequest {
    subscription_id: String,
    database_name: String,
}

async fn domains_search_handler(Json(payload): Json<DomainSearchQuery>) -> impl IntoResponse {
    let results = search_domain_availability(&payload.query);
    (StatusCode::OK, Json(results))
}

async fn domains_register_handler(Json(payload): Json<DomainRegisterRequest>) -> impl IntoResponse {
    let db_path = proteus_core::paths::get_database_path();
    let conn = match rusqlite::Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    };

    match register_domain(&conn, &payload) {
        Ok(rec) => (StatusCode::OK, Json(serde_json::json!({ "success": true, "record": rec }))).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "success": false, "error": err }))).into_response(),
    }
}

async fn domains_list_handler(Query(params): Query<ClientQuery>) -> impl IntoResponse {
    let client_id = params.client_id.unwrap_or_else(|| "default_client".to_string());
    let db_path = proteus_core::paths::get_database_path();
    let conn = match rusqlite::Connection::open(&db_path) {
        Ok(c) => c,
        Err(_) => return (StatusCode::OK, Json(Vec::<serde_json::Value>::new())).into_response(),
    };

    let domains = list_client_domains(&conn, &client_id);
    (StatusCode::OK, Json(domains)).into_response()
}

async fn hosting_plans_handler() -> impl IntoResponse {
    let plans = vec![
        serde_json::json!({
            "tier": "SelfHostedOnPremise",
            "title": HostingTier::SelfHostedOnPremise.display_name(),
            "monthly_fee_eur": HostingTier::SelfHostedOnPremise.monthly_fee_eur(),
            "storage": "Ιδιόκτητος Δίσκος Πελάτη",
            "ssl": "Cloudflare Tunnel / Tailscale",
            "databases": "Απεριόριστες",
        }),
        serde_json::json!({
            "tier": "ManagedCloudStarter",
            "title": HostingTier::ManagedCloudStarter.display_name(),
            "monthly_fee_eur": HostingTier::ManagedCloudStarter.monthly_fee_eur(),
            "storage": "10 GB NVMe Cloud SSD",
            "ssl": "Αυτόματο Wildcard SSL/TLS",
            "databases": "1 Cloud SQLite DB",
        }),
        serde_json::json!({
            "tier": "ManagedCloudBusiness",
            "title": HostingTier::ManagedCloudBusiness.display_name(),
            "monthly_fee_eur": HostingTier::ManagedCloudBusiness.monthly_fee_eur(),
            "storage": "50 GB NVMe Cloud SSD",
            "ssl": "Αυτόματο Wildcard SSL/TLS & Edge CDN",
            "databases": "Έως 5 Cloud DBs (Orders, Customers, Bookings)",
        }),
        serde_json::json!({
            "tier": "ManagedEnterpriseCluster",
            "title": HostingTier::ManagedEnterpriseCluster.display_name(),
            "monthly_fee_eur": HostingTier::ManagedEnterpriseCluster.monthly_fee_eur(),
            "storage": "Dedicated VPS Instance",
            "ssl": "Dedicated Multi-Region SSL + SLA 99.99%",
            "databases": "Απεριόριστες DBs με Full Sync",
        }),
    ];
    (StatusCode::OK, Json(plans))
}

async fn hosting_subscribe_handler(Json(payload): Json<HostingSubscribeRequest>) -> impl IntoResponse {
    let db_path = proteus_core::paths::get_database_path();
    let conn = match rusqlite::Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    };

    match subscribe_hosting(&conn, &payload) {
        Ok(sub) => (StatusCode::OK, Json(serde_json::json!({ "success": true, "subscription": sub }))).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "success": false, "error": err }))).into_response(),
    }
}

async fn hosting_list_handler(Query(params): Query<ClientQuery>) -> impl IntoResponse {
    let client_id = params.client_id.unwrap_or_else(|| "default_client".to_string());
    let db_path = proteus_core::paths::get_database_path();
    let conn = match rusqlite::Connection::open(&db_path) {
        Ok(c) => c,
        Err(_) => return (StatusCode::OK, Json(Vec::<serde_json::Value>::new())).into_response(),
    };

    let subscriptions = list_client_hosting(&conn, &client_id);
    (StatusCode::OK, Json(subscriptions)).into_response()
}

async fn hosting_provision_db_handler(Json(payload): Json<ProvisionDbRequest>) -> impl IntoResponse {
    let db_path = proteus_core::paths::get_database_path();
    let conn = match rusqlite::Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    };

    match attach_extra_database(&conn, &payload.subscription_id, &payload.database_name) {
        Ok(dbs) => (StatusCode::OK, Json(serde_json::json!({ "success": true, "active_databases": dbs }))).into_response(),
        Err(err) => (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "success": false, "error": err }))).into_response(),
    }
}

async fn submit_brief_handler(
    Extension(mgr): Extension<SlotBoardManager>,
    Json(payload): Json<ClientProjectBrief>,
) -> impl IntoResponse {
    info!(
        "Bespoke Project Brief received for '{}' ({}) - Budget: €{:.2}",
        payload.business_name, payload.business_nature, payload.proposed_budget_eur
    );

    let screens = payload.required_screens.len().max(1);
    let peripherals = payload.hardware_peripherals.len();
    let metrics = proteus_core::pricing::ProjectComplexityMetrics::new(screens, 2, 2, peripherals);

    let board_res = proteus_core::pricing::ProjectSlotBoard::new(
        payload.brief_id.clone(),
        format!("{} — {}", payload.business_name, payload.business_nature),
        payload.business_name.clone(),
        payload.proposed_budget_eur,
        metrics,
        payload.submitted_at.clone(),
    );

    match board_res {
        Ok(board) => {
            mgr.add_board(board.clone());

            // Persist to store.db for live Studio ingestion
            let db_path = proteus_core::paths::get_database_path();
            if let Ok(conn) = rusqlite::Connection::open(&db_path) {
                let _ = proteus_core::brief::insert_or_update_brief(&conn, &payload);
            }

            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "success": true,
                    "message": "Το εξατομικευμένο επιχειρησιακό σας αίτημα αναρτήθηκε επιτυχώς στο 7-Slot Board.",
                    "slot_board": board,
                })),
            )
                .into_response()
        }
        Err(err) => {
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "success": false,
                    "error": format!("Αποτυχία καταχώρησης: {}", err),
                })),
            )
                .into_response()
        }
    }
}

async fn backups_list_handler() -> impl IntoResponse {
    match portal::get_backups_list() {
        Ok(backups) => (
            StatusCode::OK,
            Json(serde_json::json!({ "success": true, "backups": backups })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "success": false, "error": e })),
        ),
    }
}

async fn backups_create_handler() -> impl IntoResponse {
    match portal::trigger_backup_snapshot() {
        Ok(entry) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "message": "Live snapshot created and encrypted with Argon2id + XChaCha20Poly1305",
                "backup": entry,
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "success": false, "error": e })),
        ),
    }
}

async fn backups_download_handler(Path(filename): Path<String>) -> impl IntoResponse {
    if filename.contains('/') || filename.contains('\\') || filename.contains("..") {
        return (
            StatusCode::BAD_REQUEST,
            [(axum::http::header::CONTENT_TYPE, "text/plain".to_string()), (axum::http::header::CONTENT_DISPOSITION, "inline".to_string())],
            Vec::new(),
        );
    }

    let backup_dir = proteus_core::paths::get_backup_dir();
    let file_path = backup_dir.join(&filename);

    if !file_path.exists() {
        return (
            StatusCode::NOT_FOUND,
            [(axum::http::header::CONTENT_TYPE, "text/plain".to_string()), (axum::http::header::CONTENT_DISPOSITION, "inline".to_string())],
            Vec::new(),
        );
    }

    match std::fs::read(&file_path) {
        Ok(bytes) => (
            StatusCode::OK,
            [
                (axum::http::header::CONTENT_TYPE, "application/octet-stream".to_string()),
                (axum::http::header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", filename)),
            ],
            bytes,
        ),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            [(axum::http::header::CONTENT_TYPE, "text/plain".to_string()), (axum::http::header::CONTENT_DISPOSITION, "inline".to_string())],
            Vec::new(),
        ),
    }
}


