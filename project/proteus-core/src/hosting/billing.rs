//! Automated VPS Billing Markup Calculator & Multi-Tenant Infrastructure Invoicing.
//! Computes raw provider infrastructure cost (vCPU, RAM, NVMe storage, egress, dedicated IP)
//! and applies an automated +50% margin markup with VAT breakdown and SQLite persistence.

use rusqlite::{params, Connection, Result as SqliteResult};
use serde::{Deserialize, Serialize};

/// Standard profit margin applied to raw VPS infrastructure cost: +50%.
pub const DEFAULT_VPS_MARKUP_PERCENT: f64 = 50.0;

/// Standard Greek VAT rate for digital services: 24%.
pub const DEFAULT_VAT_PERCENT: f64 = 24.0;

/// Resource consumption specification for a client's hosted instance.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VpsResourceSpec {
    pub vcpu_cores: u32,
    pub ram_gb: u32,
    pub storage_gb: u64,
    pub egress_tb: f64,
    pub dedicated_ips: u32,
}

impl Default for VpsResourceSpec {
    fn default() -> Self {
        Self {
            vcpu_cores: 2,
            ram_gb: 4,
            storage_gb: 50, // Standard 50GB allocated container
            egress_tb: 1.0,
            dedicated_ips: 1,
        }
    }
}

/// Baseline cost rates for raw VPS infrastructure (in Euro cents / 100ths of EUR).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VpsProviderRates {
    pub base_server_cents: u64,       // e.g. 450 cents (4.50€) for 2 vCPU / 4GB RAM base
    pub storage_rate_per_gb_cents: u64, // e.g. 5 cents (0.05€) per GB above 20GB included
    pub storage_included_gb: u64,       // e.g. 20GB included in base
    pub egress_rate_per_tb_cents: u64,  // e.g. 100 cents (1.00€) per TB
    pub egress_included_tb: f64,        // e.g. 1.0 TB included in base
    pub ip_rate_cents: u64,             // e.g. 150 cents (1.50€) per dedicated IPv4
}

impl Default for VpsProviderRates {
    fn default() -> Self {
        Self {
            base_server_cents: 450,       // 4.50 EUR base
            storage_rate_per_gb_cents: 5,  // 0.05 EUR / GB
            storage_included_gb: 20,
            egress_rate_per_tb_cents: 100, // 1.00 EUR / TB
            egress_included_tb: 1.0,
            ip_rate_cents: 150,           // 1.50 EUR / IP
        }
    }
}

/// Detailed line item within the VPS billing calculation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BillingLineItem {
    pub description: String,
    pub raw_cost_cents: u64,
    pub retail_price_cents: u64,
    pub markup_cents: u64,
}

/// Complete billing calculation report with 50% margin markup and VAT.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VpsBillingQuote {
    pub tenant_id: String,
    pub raw_cost_cents: u64,
    pub margin_markup_cents: u64,
    pub retail_subtotal_cents: u64,
    pub vat_rate_percent: f64,
    pub vat_cents: u64,
    pub total_cents: u64,
    pub markup_percent: f64,
    pub currency: String,
    pub line_items: Vec<BillingLineItem>,
}

/// Calculates raw cost and retail markup (+50%) for given VPS resources.
pub fn calculate_vps_quote(
    tenant_id: &str,
    spec: &VpsResourceSpec,
    rates: &VpsProviderRates,
    markup_percent: f64,
    vat_percent: f64,
) -> VpsBillingQuote {
    let mut line_items = Vec::new();
    let markup_multiplier = 1.0 + (markup_percent / 100.0);

    // 1. Base compute & RAM
    let base_raw = rates.base_server_cents;
    let base_retail = ((base_raw as f64) * markup_multiplier).round() as u64;
    line_items.push(BillingLineItem {
        description: format!("Compute & RAM ({} vCPU, {}GB RAM)", spec.vcpu_cores, spec.ram_gb),
        raw_cost_cents: base_raw,
        retail_price_cents: base_retail,
        markup_cents: base_retail.saturating_sub(base_raw),
    });

    // 2. Extra Storage over included threshold
    let extra_storage = spec.storage_gb.saturating_sub(rates.storage_included_gb);
    let storage_raw = extra_storage * rates.storage_rate_per_gb_cents;
    let storage_retail = ((storage_raw as f64) * markup_multiplier).round() as u64;
    line_items.push(BillingLineItem {
        description: format!("NVMe Storage ({}GB total, {}GB chargeable)", spec.storage_gb, extra_storage),
        raw_cost_cents: storage_raw,
        retail_price_cents: storage_retail,
        markup_cents: storage_retail.saturating_sub(storage_raw),
    });

    // 3. Egress Bandwidth
    let extra_egress = (spec.egress_tb - rates.egress_included_tb).max(0.0);
    let egress_raw = (extra_egress * (rates.egress_rate_per_tb_cents as f64)).round() as u64;
    let egress_retail = ((egress_raw as f64) * markup_multiplier).round() as u64;
    if extra_egress > 0.0 {
        line_items.push(BillingLineItem {
            description: format!("Network Egress ({:.2} TB extra)", extra_egress),
            raw_cost_cents: egress_raw,
            retail_price_cents: egress_retail,
            markup_cents: egress_retail.saturating_sub(egress_raw),
        });
    }

    // 4. Dedicated IP Addresses
    let ip_raw = (spec.dedicated_ips as u64) * rates.ip_rate_cents;
    let ip_retail = ((ip_raw as f64) * markup_multiplier).round() as u64;
    line_items.push(BillingLineItem {
        description: format!("Dedicated IPv4 ({} address)", spec.dedicated_ips),
        raw_cost_cents: ip_raw,
        retail_price_cents: ip_retail,
        markup_cents: ip_retail.saturating_sub(ip_raw),
    });

    let raw_total: u64 = line_items.iter().map(|item| item.raw_cost_cents).sum();
    let retail_subtotal: u64 = line_items.iter().map(|item| item.retail_price_cents).sum();
    let markup_total = retail_subtotal.saturating_sub(raw_total);

    let vat_cents = ((retail_subtotal as f64) * (vat_percent / 100.0)).round() as u64;
    let grand_total = retail_subtotal + vat_cents;

    VpsBillingQuote {
        tenant_id: tenant_id.to_string(),
        raw_cost_cents: raw_total,
        margin_markup_cents: markup_total,
        retail_subtotal_cents: retail_subtotal,
        vat_rate_percent: vat_percent,
        vat_cents,
        total_cents: grand_total,
        markup_percent,
        currency: "EUR".to_string(),
        line_items,
    }
}

/// Initializes SQLite schema for persisting tenant VPS invoices and quotes.
pub fn init_vps_billing_schema(conn: &Connection) -> SqliteResult<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS vps_billing_invoices (
            invoice_id TEXT PRIMARY KEY,
            tenant_id TEXT NOT NULL,
            billing_period_start TEXT NOT NULL,
            billing_period_end TEXT NOT NULL,
            raw_cost_cents INTEGER NOT NULL,
            retail_subtotal_cents INTEGER NOT NULL,
            margin_markup_cents INTEGER NOT NULL,
            vat_cents INTEGER NOT NULL,
            total_cents INTEGER NOT NULL,
            status TEXT NOT NULL DEFAULT 'UNPAID',
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        CREATE INDEX IF NOT EXISTS idx_vps_billing_tenant ON vps_billing_invoices(tenant_id);
        CREATE INDEX IF NOT EXISTS idx_vps_billing_status ON vps_billing_invoices(status);",
    )
}

/// Persists a generated billing quote into the SQLite database as an invoice record.
pub fn record_vps_invoice(
    conn: &Connection,
    invoice_id: &str,
    quote: &VpsBillingQuote,
    period_start: &str,
    period_end: &str,
) -> SqliteResult<()> {
    conn.execute(
        "INSERT INTO vps_billing_invoices (
            invoice_id, tenant_id, billing_period_start, billing_period_end,
            raw_cost_cents, retail_subtotal_cents, margin_markup_cents, vat_cents, total_cents, status
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'UNPAID')
        ON CONFLICT(invoice_id) DO UPDATE SET
            total_cents = excluded.total_cents,
            status = excluded.status",
        params![
            invoice_id,
            quote.tenant_id,
            period_start,
            period_end,
            quote.raw_cost_cents as i64,
            quote.retail_subtotal_cents as i64,
            quote.margin_markup_cents as i64,
            quote.vat_cents as i64,
            quote.total_cents as i64,
        ],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vps_calculation_with_50_percent_margin() {
        let spec = VpsResourceSpec {
            vcpu_cores: 2,
            ram_gb: 4,
            storage_gb: 50, // 30GB extra over 20GB included
            egress_tb: 1.0,  // included
            dedicated_ips: 1, // 150 cents
        };

        let rates = VpsProviderRates::default();
        // Base = 450 cents
        // Storage extra = 30 * 5 = 150 cents
        // IP = 150 cents
        // Raw total = 450 + 150 + 150 = 750 cents (7.50 EUR)
        let quote = calculate_vps_quote(
            "client_omega",
            &spec,
            &rates,
            DEFAULT_VPS_MARKUP_PERCENT, // 50%
            DEFAULT_VAT_PERCENT,        // 24%
        );

        assert_eq!(quote.raw_cost_cents, 750);
        // Retail: 750 * 1.5 = 1125 cents (11.25 EUR)
        assert_eq!(quote.retail_subtotal_cents, 1125);
        assert_eq!(quote.margin_markup_cents, 375); // 3.75 EUR net margin
        assert_eq!(quote.markup_percent, 50.0);

        // VAT: 1125 * 0.24 = 270 cents (2.70 EUR)
        assert_eq!(quote.vat_cents, 270);
        // Grand total: 1125 + 270 = 1395 cents (13.95 EUR)
        assert_eq!(quote.total_cents, 1395);
        assert_eq!(quote.currency, "EUR");
        assert_eq!(quote.line_items.len(), 3);
    }

    #[test]
    fn test_vps_quote_with_extra_egress() {
        let spec = VpsResourceSpec {
            vcpu_cores: 4,
            ram_gb: 8,
            storage_gb: 70, // 50GB extra (250 cents)
            egress_tb: 3.0,  // 2.0 TB extra (200 cents)
            dedicated_ips: 2, // 300 cents
        };

        let rates = VpsProviderRates {
            base_server_cents: 800,
            ..Default::default()
        };

        // Raw total = 800 + 250 + 200 + 300 = 1550 cents
        let quote = calculate_vps_quote("client_scale", &spec, &rates, 50.0, 24.0);
        assert_eq!(quote.raw_cost_cents, 1550);
        // Retail subtotal = 1550 * 1.5 = 2325 cents
        assert_eq!(quote.retail_subtotal_cents, 2325);
        assert_eq!(quote.margin_markup_cents, 775);
        assert_eq!(quote.line_items.len(), 4);
    }

    #[test]
    fn test_vps_billing_sqlite_persistence() {
        let conn = Connection::open_in_memory().unwrap();
        init_vps_billing_schema(&conn).unwrap();

        let spec = VpsResourceSpec::default();
        let rates = VpsProviderRates::default();
        let quote = calculate_vps_quote("tenant_alpha", &spec, &rates, 50.0, 24.0);

        record_vps_invoice(&conn, "INV-2026-001", &quote, "2026-11-01", "2026-11-30").unwrap();

        let (tenant, total, status): (String, i64, String) = conn
            .query_row(
                "SELECT tenant_id, total_cents, status FROM vps_billing_invoices WHERE invoice_id = 'INV-2026-001'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();

        assert_eq!(tenant, "tenant_alpha");
        assert_eq!(total, quote.total_cents as i64);
        assert_eq!(status, "UNPAID");
    }
}
