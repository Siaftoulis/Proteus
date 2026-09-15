//! Proteus Web Portal & Public Showcase Engine.
//! Implements pricing bracket calculations, templates catalog, and public web endpoints.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingQuoteRequest {
    pub seats: u32,
    pub include_cloud_sync: bool,
    pub include_cloud_backups: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingQuoteResponse {
    pub seats: u32,
    pub core_license_monthly: f64,
    pub cloud_sync_monthly: f64,
    pub cloud_backup_monthly: f64,
    pub total_monthly: f64,
    pub annual_total_with_discount: f64, // 2 months free on annual
}

/// Calculate subscription pricing according to Document 18 Section 6 brackets.
pub fn calculate_subscription_quote(req: &PricingQuoteRequest) -> PricingQuoteResponse {
    let core_license_monthly = if req.seats >= 150 {
        199.00 // Enterprise cap
    } else if req.seats <= 4 {
        7.99 // Base tier
    } else {
        let mut total = 7.99;
        let mut remaining = req.seats - 4;

        // Zone 1: 5-20 users (up to 16 additional users @ 1.50€)
        let z1 = remaining.min(16);
        total += z1 as f64 * 1.50;
        remaining -= z1;

        // Zone 2: 21-60 users (up to 40 additional users @ 1.00€)
        if remaining > 0 {
            let z2 = remaining.min(40);
            total += z2 as f64 * 1.00;
            remaining -= z2;
        }

        // Zone 3: 61-150 users (@ 0.60€)
        if remaining > 0 {
            total += remaining as f64 * 0.60;
        }

        total
    };

    let cloud_sync_monthly = if !req.include_cloud_sync {
        0.0
    } else if req.seats <= 20 {
        15.0
    } else if req.seats <= 100 {
        45.0
    } else {
        120.0
    };

    let cloud_backup_monthly = if req.include_cloud_backups { 9.99 } else { 0.0 };

    let sum = core_license_monthly + cloud_sync_monthly + cloud_backup_monthly;
    let total_monthly = (sum * 100.0).round() / 100.0;
    let annual_total_with_discount = (total_monthly * 10.0 * 100.0).round() / 100.0; // 10 months pay for 12

    PricingQuoteResponse {
        seats: req.seats,
        core_license_monthly: (core_license_monthly * 100.0).round() / 100.0,
        cloud_sync_monthly,
        cloud_backup_monthly,
        total_monthly,
        annual_total_with_discount,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_pricing_brackets() {
        // 1-4 seats
        let q1 = calculate_subscription_quote(&PricingQuoteRequest {
            seats: 3,
            include_cloud_sync: false,
            include_cloud_backups: false,
        });
        assert_eq!(q1.core_license_monthly, 7.99);

        // 10 seats: 7.99 + (6 * 1.50) = 16.99
        let q2 = calculate_subscription_quote(&PricingQuoteRequest {
            seats: 10,
            include_cloud_sync: false,
            include_cloud_backups: false,
        });
        assert_eq!(q2.core_license_monthly, 16.99);

        // 150+ seats: 199.00 flat
        let q3 = calculate_subscription_quote(&PricingQuoteRequest {
            seats: 200,
            include_cloud_sync: false,
            include_cloud_backups: false,
        });
        assert_eq!(q3.core_license_monthly, 199.00);
    }

    #[test]
    fn test_combined_cloud_quote() {
        let q = calculate_subscription_quote(&PricingQuoteRequest {
            seats: 4,
            include_cloud_sync: true,
            include_cloud_backups: true,
        });
        // Core: 7.99, Sync: 15.00, Backup: 9.99 -> Total: 32.98
        assert_eq!(q.core_license_monthly, 7.99);
        assert_eq!(q.cloud_sync_monthly, 15.0);
        assert_eq!(q.cloud_backup_monthly, 9.99);
        assert_eq!(q.total_monthly, 32.98);
    }
}
