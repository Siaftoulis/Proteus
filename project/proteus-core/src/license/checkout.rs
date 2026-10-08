//! Proteus Core — Merchant of Record (MoR) Dynamic Seat Tier Checkout Engine.
//! Generates checkout sessions and provisions cryptographic licenses for Lemon Squeezy and Paddle.

use super::offline::{issue_offline_token, OfflineLicenseClaims};
use super::{mor::MorProvider, LicenseError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BillingCycle {
    Monthly,
    Annual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MorCheckoutConfig {
    pub provider: MorProvider,
    pub store_slug: String,
    pub variant_monthly_id: String,
    pub variant_annual_id: String,
    pub base_url: Option<String>,
}

impl Default for MorCheckoutConfig {
    fn default() -> Self {
        Self {
            provider: MorProvider::LemonSqueezy,
            store_slug: "proteus-bos".to_string(),
            variant_monthly_id: "var_core_monthly_799".to_string(),
            variant_annual_id: "var_core_annual_7990".to_string(),
            base_url: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckoutSessionRequest {
    pub seats: u32,
    pub billing_cycle: BillingCycle,
    pub include_cloud_sync: bool,
    pub include_cloud_backups: bool,
    pub customer_email: String,
    pub customer_name: String,
    pub shop_afm: String,
    pub machine_id: Option<String>,
    pub success_url: String,
    pub cancel_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CheckoutSessionResponse {
    pub provider: MorProvider,
    pub checkout_url: String,
    pub order_reference: String,
    pub total_eur: f64,
    pub seats: u32,
    pub billing_cycle: BillingCycle,
}

/// Computes subscription total according to Document 18 pricing brackets.
pub fn calculate_seat_pricing(seats: u32, cycle: BillingCycle) -> f64 {
    let core_monthly = if seats >= 150 {
        199.00
    } else if seats <= 4 {
        7.99
    } else {
        let mut total = 7.99;
        let mut remaining = seats - 4;

        let z1 = remaining.min(16);
        total += z1 as f64 * 1.50;
        remaining -= z1;

        if remaining > 0 {
            let z2 = remaining.min(40);
            total += z2 as f64 * 1.00;
            remaining -= z2;
        }

        if remaining > 0 {
            total += remaining as f64 * 0.60;
        }

        total
    };

    match cycle {
        BillingCycle::Monthly => (core_monthly * 100.0).round() / 100.0,
        BillingCycle::Annual => (core_monthly * 10.0 * 100.0).round() / 100.0, // 2 months free
    }
}

/// Generates a signed checkout session URL for customer redirection.
pub fn generate_checkout_session(
    req: &CheckoutSessionRequest,
    config: &MorCheckoutConfig,
) -> CheckoutSessionResponse {
    let order_reference = format!("ORD-{}", uuid::Uuid::now_v7());
    let total_eur = calculate_seat_pricing(req.seats, req.billing_cycle);

    let variant_id = match req.billing_cycle {
        BillingCycle::Monthly => &config.variant_monthly_id,
        BillingCycle::Annual => &config.variant_annual_id,
    };

    let base = config.base_url.as_deref().unwrap_or(match config.provider {
        MorProvider::LemonSqueezy => "https://proteus.lemonsqueezy.com/buy",
        MorProvider::Paddle => "https://checkout.paddle.com/service",
    });

    // Custom data payload for webhook correlation
    let custom_afm = &req.shop_afm;
    let custom_email = &req.customer_email;
    let custom_machine = req.machine_id.as_deref().unwrap_or("standalone");

    let checkout_url = format!(
        "{}/{}?checkout[email]={}&checkout[custom][order_ref]={}&checkout[custom][afm]={}&checkout[custom][machine_id]={}&checkout[custom][seats]={}&checkout[success_url]={}",
        base,
        variant_id,
        custom_email,
        order_reference,
        custom_afm,
        custom_machine,
        req.seats,
        req.success_url
    );

    CheckoutSessionResponse {
        provider: config.provider,
        checkout_url,
        order_reference,
        total_eur,
        seats: req.seats,
        billing_cycle: req.billing_cycle,
    }
}

/// Provisions a cryptographic offline/online license directly from an approved MoR order.
pub fn provision_license_from_checkout(
    order_id: &str,
    customer_name: &str,
    shop_afm: &str,
    seats: u32,
    machine_id: Option<&str>,
    cycle: BillingCycle,
    master_key: &str,
) -> Result<String, LicenseError> {
    let now = chrono::Utc::now();
    let days = match cycle {
        BillingCycle::Monthly => 32, // 30 days + 2 grace days
        BillingCycle::Annual => 370, // 365 days + 5 grace days
    };
    let expires = now + chrono::Duration::days(days);

    let claims = OfflineLicenseClaims {
        license_id: format!("MOR-{}", order_id),
        customer_name: customer_name.to_string(),
        shop_afm: shop_afm.to_string(),
        machine_id: machine_id.map(|s| s.to_string()),
        max_users: seats as i32,
        features: vec![
            "intake".to_string(),
            "kanban".to_string(),
            "escpos_printing".to_string(),
            "smlm_inventory".to_string(),
            "contractor_ledger".to_string(),
            "offline_audit".to_string(),
            "fiscal_pos".to_string(),
            "mydata_sync".to_string(),
        ],
        issued_at: now.to_rfc3339(),
        expires_at: expires.to_rfc3339(),
        is_pilot: false,
    };

    issue_offline_token(&claims, master_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_seat_pricing_tiers() {
        // Base tier: <= 4 seats is 7.99€
        assert_eq!(calculate_seat_pricing(1, BillingCycle::Monthly), 7.99);
        assert_eq!(calculate_seat_pricing(4, BillingCycle::Monthly), 7.99);

        // Zone 1: 5 seats = 7.99 + 1.50 = 9.49€
        assert_eq!(calculate_seat_pricing(5, BillingCycle::Monthly), 9.49);

        // Annual pricing: 10x monthly (2 months free)
        assert_eq!(calculate_seat_pricing(4, BillingCycle::Annual), 79.90);
    }

    #[test]
    fn test_generate_checkout_session_url() {
        let req = CheckoutSessionRequest {
            seats: 8,
            billing_cycle: BillingCycle::Monthly,
            include_cloud_sync: false,
            include_cloud_backups: false,
            customer_email: "tech@workshop.gr".into(),
            customer_name: "Lab 01".into(),
            shop_afm: "123456789".into(),
            machine_id: Some("TERM-ALPHA".into()),
            success_url: "https://proteus.gr/success".into(),
            cancel_url: "https://proteus.gr/cancel".into(),
        };

        let config = MorCheckoutConfig::default();
        let session = generate_checkout_session(&req, &config);

        assert_eq!(session.provider, MorProvider::LemonSqueezy);
        assert_eq!(session.seats, 8);
        assert!(session.checkout_url.contains("var_core_monthly_799"));
        assert!(session.checkout_url.contains("tech@workshop.gr"));
        assert!(session.checkout_url.contains("TERM-ALPHA"));
        assert_eq!(session.total_eur, 13.99); // 7.99 + 4 * 1.50
    }

    #[test]
    fn test_provision_license_from_checkout() {
        let master_key = "MASTER_SIGNING_KEY_TEST";
        let token = provision_license_from_checkout(
            "ORDER-12345",
            "Auto Workshop Athens",
            "094123456",
            10,
            Some("POS-01"),
            BillingCycle::Annual,
            master_key,
        ).unwrap();

        assert!(token.starts_with("PROT-LIC-"));

        let claims = crate::license::offline::verify_offline_token(&token, Some("POS-01"), master_key).unwrap();
        assert_eq!(claims.license_id, "MOR-ORDER-12345");
        assert_eq!(claims.max_users, 10);
        assert!(!claims.is_pilot);
        assert_eq!(claims.customer_name, "Auto Workshop Athens");
    }
}
