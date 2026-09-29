//! Domain Reseller & Managed Hosting Engine for Proteus Web Portal.
//! Supports:
//! - Domain name availability search, registration, and recurring leasing (.gr, .com, .eu, .shop).
//! - Automated DNS record configuration (A, CNAME, TXT for SSL/TLS certificates).
//! - Dual hosting options: Managed Proteus Cloud Hosting vs Self-Hosting on customer server.
//! - Web Portal database provisioning: Extra databases (customers, e-commerce orders, bookings)
//!   federated with the local `proteus-core` SQLite engine.

use rusqlite::{params, Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DomainStatus {
    Available,
    RegisteredActive,
    PendingDnsVerification,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecord {
    pub record_type: String, // "A", "CNAME", "TXT"
    pub name: String,
    pub value: String,
    pub ttl: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainSearchResult {
    pub domain: String,
    pub tld: String,
    pub is_available: bool,
    pub wholesale_cost_eur: f64,
    pub retail_price_eur: f64,
    pub renewal_price_eur: f64,
    pub platform_margin_eur: f64,
    pub registrar: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainRegisterRequest {
    pub client_id: String,
    pub domain: String,
    pub auto_renew: bool,
    pub contact_email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainRegistrationRecord {
    pub id: String,
    pub client_id: String,
    pub domain: String,
    pub tld: String,
    pub price_eur: f64,
    pub status: DomainStatus,
    pub auto_renew: bool,
    pub registered_at: String,
    pub expires_at: String,
    pub dns_records: Vec<DnsRecord>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HostingTier {
    SelfHostedOnPremise, // 0€/mo hosting (included with Proteus license)
    ManagedCloudStarter, // 14.99€/mo (10GB SSD, 1 DB, auto SSL)
    ManagedCloudBusiness, // 29.99€/mo (50GB SSD, 3 DBs, real-time POS sync)
    ManagedEnterpriseCluster, // 89.00€/mo (Dedicated VPS, unlimited DBs, 99.99% SLA)
}

impl HostingTier {
    pub fn monthly_fee_eur(&self) -> f64 {
        match self {
            Self::SelfHostedOnPremise => 0.0,
            Self::ManagedCloudStarter => 14.99,
            Self::ManagedCloudBusiness => 29.99,
            Self::ManagedEnterpriseCluster => 89.00,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::SelfHostedOnPremise => "Self-Hosting (Ιδιόκτητος Server Πελάτη)",
            Self::ManagedCloudStarter => "Proteus Managed Cloud Starter",
            Self::ManagedCloudBusiness => "Proteus Managed Cloud Business & E-Shop",
            Self::ManagedEnterpriseCluster => "Proteus Dedicated Enterprise Cluster",
        }
    }

    #[allow(dead_code)]
    pub fn max_databases(&self) -> usize {
        match self {
            Self::SelfHostedOnPremise => usize::MAX,
            Self::ManagedCloudStarter => 1,
            Self::ManagedCloudBusiness => 5,
            Self::ManagedEnterpriseCluster => usize::MAX,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostingSubscription {
    pub id: String,
    pub client_id: String,
    pub domain: String,
    pub plan_tier: HostingTier,
    pub is_managed: bool,
    pub monthly_fee_eur: f64,
    pub status: String,
    pub server_endpoint: String,
    pub active_databases: Vec<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostingSubscribeRequest {
    pub client_id: String,
    pub domain: String,
    pub plan_tier: HostingTier,
    pub initial_databases: Vec<String>,
}

/// Initialize SQLite tables for domain registration and hosting subscriptions.
pub fn init_domains_hosting_tables(conn: &Connection) -> SqlResult<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS client_domains (
            id TEXT PRIMARY KEY,
            client_id TEXT NOT NULL,
            domain TEXT NOT NULL UNIQUE,
            tld TEXT NOT NULL,
            price_eur REAL NOT NULL,
            status TEXT NOT NULL,
            auto_renew INTEGER NOT NULL,
            registered_at TEXT NOT NULL,
            expires_at TEXT NOT NULL,
            dns_records_json TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS client_hosting_subscriptions (
            id TEXT PRIMARY KEY,
            client_id TEXT NOT NULL,
            domain TEXT NOT NULL,
            plan_tier TEXT NOT NULL,
            is_managed INTEGER NOT NULL,
            monthly_fee_eur REAL NOT NULL,
            status TEXT NOT NULL,
            server_endpoint TEXT NOT NULL,
            databases_json TEXT NOT NULL,
            created_at TEXT NOT NULL
        );
        "#,
    )
}

/// Search domain availability across supported TLDs with calculated reseller margins.
pub fn search_domain_availability(query: &str) -> Vec<DomainSearchResult> {
    let clean_query = query.trim().to_lowercase();
    let base_name = clean_query
        .split('.')
        .next()
        .unwrap_or("")
        .replace(|c: char| !c.is_alphanumeric() && c != '-', "");

    if base_name.is_empty() {
        return Vec::new();
    }

    let tld_configs = [
        (".gr", 11.50, 19.90, 19.90, "EETT / Registrar Partner"),
        (".com", 10.00, 16.90, 16.90, "Verisign / Registrar API"),
        (".eu", 8.00, 14.50, 14.50, "EURid / Registrar Partner"),
        (".shop", 14.00, 24.90, 24.90, "Global Registries API"),
    ];

    tld_configs
        .iter()
        .map(|&(tld, wholesale, retail, renewal, registrar)| {
            let full_domain = format!("{}{}", base_name, tld);
            // Simulated check: domains with 'google' or 'apple' are marked unavailable
            let is_available = !base_name.contains("google")
                && !base_name.contains("apple")
                && !base_name.contains("microsoft");

            DomainSearchResult {
                domain: full_domain,
                tld: tld.to_string(),
                is_available,
                wholesale_cost_eur: wholesale,
                retail_price_eur: retail,
                renewal_price_eur: renewal,
                platform_margin_eur: ((retail - wholesale) * 100.0).round() / 100.0,
                registrar: registrar.to_string(),
            }
        })
        .collect()
}

/// Register a new domain for a client, generate automated DNS zone records, and persist to SQLite.
pub fn register_domain(
    conn: &Connection,
    req: &DomainRegisterRequest,
) -> Result<DomainRegistrationRecord, String> {
    let domain = req.domain.trim().to_lowercase();
    if domain.is_empty() || !domain.contains('.') {
        return Err("Μη έγκυρο όνομα domain.".to_string());
    }

    let tld = format!(".{}", domain.split('.').last().unwrap_or(""));
    let retail_price = match tld.as_str() {
        ".gr" => 19.90,
        ".com" => 16.90,
        ".eu" => 14.50,
        ".shop" => 24.90,
        _ => 22.00,
    };

    let id = format!("DOM-{}", &uuid::Uuid::new_v4().to_string()[..8].to_uppercase());
    let now = chrono::Utc::now();
    let expires = now + chrono::Duration::days(365);
    let registered_at = now.to_rfc3339();
    let expires_at = expires.to_rfc3339();

    // Default automated DNS records for Proteus Edge & SSL
    let default_dns = vec![
        DnsRecord {
            record_type: "A".to_string(),
            name: "@".to_string(),
            value: "185.199.108.153".to_string(), // Proteus Edge IP
            ttl: 3600,
        },
        DnsRecord {
            record_type: "CNAME".to_string(),
            name: "www".to_string(),
            value: domain.clone(),
            ttl: 3600,
        },
        DnsRecord {
            record_type: "TXT".to_string(),
            name: "_proteus-challenge".to_string(),
            value: format!("proteus-verify-token={}", id),
            ttl: 300,
        },
    ];

    let dns_json = serde_json::to_string(&default_dns).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO client_domains (id, client_id, domain, tld, price_eur, status, auto_renew, registered_at, expires_at, dns_records_json)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            id,
            req.client_id,
            domain,
            tld,
            retail_price,
            "RegisteredActive",
            if req.auto_renew { 1 } else { 0 },
            registered_at,
            expires_at,
            dns_json,
        ],
    ).map_err(|e| format!("Αποτυχία καταχώρησης domain στη βάση: {}", e))?;

    Ok(DomainRegistrationRecord {
        id,
        client_id: req.client_id.clone(),
        domain,
        tld,
        price_eur: retail_price,
        status: DomainStatus::RegisteredActive,
        auto_renew: req.auto_renew,
        registered_at,
        expires_at,
        dns_records: default_dns,
    })
}

/// Retrieve all registered domains for a client from SQLite.
pub fn list_client_domains(conn: &Connection, client_id: &str) -> Vec<DomainRegistrationRecord> {
    let mut stmt = match conn.prepare(
        "SELECT id, client_id, domain, tld, price_eur, status, auto_renew, registered_at, expires_at, dns_records_json
         FROM client_domains WHERE client_id = ?1 ORDER BY registered_at DESC"
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let rows = stmt.query_map(params![client_id], |row| {
        let dns_json: String = row.get(9)?;
        let dns_records: Vec<DnsRecord> = serde_json::from_str(&dns_json).unwrap_or_default();
        let auto_renew_int: i32 = row.get(6)?;

        Ok(DomainRegistrationRecord {
            id: row.get(0)?,
            client_id: row.get(1)?,
            domain: row.get(2)?,
            tld: row.get(3)?,
            price_eur: row.get(4)?,
            status: DomainStatus::RegisteredActive,
            auto_renew: auto_renew_int == 1,
            registered_at: row.get(7)?,
            expires_at: row.get(8)?,
            dns_records,
        })
    });

    match rows {
        Ok(iter) => iter.filter_map(|r| r.ok()).collect(),
        Err(_) => Vec::new(),
    }
}

/// Provision a hosting subscription for a client (Managed Cloud or Self-Hosted) and store in SQLite.
pub fn subscribe_hosting(
    conn: &Connection,
    req: &HostingSubscribeRequest,
) -> Result<HostingSubscription, String> {
    let id = format!("HST-{}", &uuid::Uuid::new_v4().to_string()[..8].to_uppercase());
    let is_managed = req.plan_tier != HostingTier::SelfHostedOnPremise;
    let fee = req.plan_tier.monthly_fee_eur();
    let created_at = chrono::Utc::now().to_rfc3339();

    let server_endpoint = if is_managed {
        format!("https://{}.cloud.proteusbos.gr", req.domain.replace('.', "-"))
    } else {
        "local-tunnel://lan-gateway:8080".to_string()
    };

    let mut dbs = req.initial_databases.clone();
    if dbs.is_empty() {
        dbs.push("proteus_core_store".to_string());
    }

    let dbs_json = serde_json::to_string(&dbs).map_err(|e| e.to_string())?;
    let tier_str = format!("{:?}", req.plan_tier);

    conn.execute(
        "INSERT INTO client_hosting_subscriptions (id, client_id, domain, plan_tier, is_managed, monthly_fee_eur, status, server_endpoint, databases_json, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            id,
            req.client_id,
            req.domain,
            tier_str,
            if is_managed { 1 } else { 0 },
            fee,
            "Active",
            server_endpoint,
            dbs_json,
            created_at,
        ],
    ).map_err(|e| format!("Αποτυχία ενεργοποίησης φιλοξενίας: {}", e))?;

    Ok(HostingSubscription {
        id,
        client_id: req.client_id.clone(),
        domain: req.domain.clone(),
        plan_tier: req.plan_tier,
        is_managed,
        monthly_fee_eur: fee,
        status: "Active".to_string(),
        server_endpoint,
        active_databases: dbs,
        created_at,
    })
}

/// Retrieve hosting subscriptions for a client from SQLite.
pub fn list_client_hosting(conn: &Connection, client_id: &str) -> Vec<HostingSubscription> {
    let mut stmt = match conn.prepare(
        "SELECT id, client_id, domain, plan_tier, is_managed, monthly_fee_eur, status, server_endpoint, databases_json, created_at
         FROM client_hosting_subscriptions WHERE client_id = ?1 ORDER BY created_at DESC"
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let rows = stmt.query_map(params![client_id], |row| {
        let tier_str: String = row.get(3)?;
        let plan_tier = match tier_str.as_str() {
            "ManagedCloudStarter" => HostingTier::ManagedCloudStarter,
            "ManagedCloudBusiness" => HostingTier::ManagedCloudBusiness,
            "ManagedEnterpriseCluster" => HostingTier::ManagedEnterpriseCluster,
            _ => HostingTier::SelfHostedOnPremise,
        };
        let is_managed_int: i32 = row.get(4)?;
        let dbs_json: String = row.get(8)?;
        let active_databases: Vec<String> = serde_json::from_str(&dbs_json).unwrap_or_default();

        Ok(HostingSubscription {
            id: row.get(0)?,
            client_id: row.get(1)?,
            domain: row.get(2)?,
            plan_tier,
            is_managed: is_managed_int == 1,
            monthly_fee_eur: row.get(5)?,
            status: row.get(6)?,
            server_endpoint: row.get(7)?,
            active_databases,
            created_at: row.get(9)?,
        })
    });

    match rows {
        Ok(iter) => iter.filter_map(|r| r.ok()).collect(),
        Err(_) => Vec::new(),
    }
}

/// Attach an extra database (e.g. web customer accounts, online orders, appointment booking)
/// to an existing hosting subscription.
pub fn attach_extra_database(
    conn: &Connection,
    subscription_id: &str,
    new_db_name: &str,
) -> Result<Vec<String>, String> {
    let mut stmt = conn
        .prepare("SELECT databases_json FROM client_hosting_subscriptions WHERE id = ?1")
        .map_err(|e| e.to_string())?;

    let existing_json: String = stmt
        .query_row(params![subscription_id], |row| row.get(0))
        .map_err(|e| format!("Δεν βρέθηκε η συνδρομή hosting: {}", e))?;

    let mut dbs: Vec<String> = serde_json::from_str(&existing_json).unwrap_or_default();
    let clean_name = new_db_name.trim().to_lowercase().replace(' ', "_");
    if clean_name.is_empty() {
        return Err("Το όνομα της βάσης δεν μπορεί να είναι κενό.".to_string());
    }

    if !dbs.contains(&clean_name) {
        dbs.push(clean_name);
        let updated_json = serde_json::to_string(&dbs).map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE client_hosting_subscriptions SET databases_json = ?1 WHERE id = ?2",
            params![updated_json, subscription_id],
        ).map_err(|e| format!("Αποτυχία ενημέρωσης βάσεων: {}", e))?;
    }

    Ok(dbs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_search_availability_and_margins() {
        let results = search_domain_availability("speedy-garage");
        assert_eq!(results.len(), 4);

        let gr = results.iter().find(|r| r.tld == ".gr").unwrap();
        assert_eq!(gr.domain, "speedy-garage.gr");
        assert!(gr.is_available);
        assert_eq!(gr.retail_price_eur, 19.90);
        assert_eq!(gr.wholesale_cost_eur, 11.50);
        assert_eq!(gr.platform_margin_eur, 8.40);

        let com = results.iter().find(|r| r.tld == ".com").unwrap();
        assert_eq!(com.domain, "speedy-garage.com");
        assert_eq!(com.platform_margin_eur, 6.90);
    }

    #[test]
    fn test_domain_registration_and_dns_generation() {
        let conn = Connection::open_in_memory().unwrap();
        init_domains_hosting_tables(&conn).unwrap();

        let req = DomainRegisterRequest {
            client_id: "SHOP_SPEEDY_01".to_string(),
            domain: "speedy-garage.gr".to_string(),
            auto_renew: true,
            contact_email: "owner@speedy.gr".to_string(),
        };

        let rec = register_domain(&conn, &req).expect("Domain registration should succeed");
        assert_eq!(rec.domain, "speedy-garage.gr");
        assert_eq!(rec.tld, ".gr");
        assert_eq!(rec.price_eur, 19.90);
        assert!(rec.auto_renew);
        assert_eq!(rec.dns_records.len(), 3);
        assert_eq!(rec.dns_records[0].record_type, "A");

        let listed = list_client_domains(&conn, "SHOP_SPEEDY_01");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].domain, "speedy-garage.gr");
    }

    #[test]
    fn test_hosting_subscriptions_and_db_provisioning() {
        let conn = Connection::open_in_memory().unwrap();
        init_domains_hosting_tables(&conn).unwrap();

        // 1. Subscribe to Managed Cloud Business
        let req = HostingSubscribeRequest {
            client_id: "SHOP_SPEEDY_01".to_string(),
            domain: "speedy-garage.gr".to_string(),
            plan_tier: HostingTier::ManagedCloudBusiness,
            initial_databases: vec!["web_orders".to_string()],
        };

        let sub = subscribe_hosting(&conn, &req).expect("Subscription should succeed");
        assert_eq!(sub.monthly_fee_eur, 29.99);
        assert!(sub.is_managed);
        assert_eq!(sub.active_databases, vec!["web_orders".to_string()]);

        // 2. Attach extra database: web_customers and appointments
        let updated_dbs = attach_extra_database(&conn, &sub.id, "web_customers").unwrap();
        assert_eq!(updated_dbs.len(), 2);
        assert!(updated_dbs.contains(&"web_customers".to_string()));

        let listed = list_client_hosting(&conn, "SHOP_SPEEDY_01");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].active_databases.len(), 2);
    }
}
