//! Air-Gapped SSG Deployment & Order Relay Envelope Ingestion.
//! Strict Rule 1 (100% Original Codebase), Rule 3 (<400 lines), and Rule 5 (Zero Mock Data).
//! Packages compiled storefronts into air-gapped static distribution bundles
//! with cryptographic manifests, and validates/ingests serverless inbound checkout envelopes
//! directly into Proteus BOS SQLite store.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use sha2::{Digest, Sha256};
use serde::{Deserialize, Serialize};
use rusqlite::{params, Connection, Result as SqlResult};
use crate::ui_dsl::compiler::CompiledWebPage;

/// An individual asset packaged within a distribution bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistAsset {
    pub relative_path: String,
    pub mime_type: String,
    pub content: Vec<u8>,
    pub sha256: String,
}

/// Cryptographic manifest tracking all assets and integrity hashes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistManifest {
    pub site_id: String,
    pub generated_at: String,
    pub total_files: usize,
    pub total_bytes: usize,
    pub files: BTreeMap<String, String>,
}

/// Complete standalone distribution bundle ready for air-gapped deployment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteDistBundle {
    pub site_id: String,
    pub assets: Vec<DistAsset>,
}

impl SiteDistBundle {
    pub fn from_compiled_page(site_id: impl Into<String>, page: &CompiledWebPage) -> Self {
        let mut bundle = Self {
            site_id: site_id.into(),
            assets: Vec::new(),
        };
        bundle.add_asset("index.html", "text/html; charset=utf-8", page.standalone_html.as_bytes().to_vec());
        bundle
    }

    pub fn add_asset(&mut self, relative_path: &str, mime_type: &str, content: Vec<u8>) {
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let sha256 = format!("{:x}", hasher.finalize());
        self.assets.push(DistAsset {
            relative_path: relative_path.to_string(),
            mime_type: mime_type.to_string(),
            content,
            sha256,
        });
    }

    pub fn build_manifest(&self) -> DistManifest {
        let mut files = BTreeMap::new();
        let mut total_bytes = 0;
        for asset in &self.assets {
            files.insert(asset.relative_path.clone(), asset.sha256.clone());
            total_bytes += asset.content.len();
        }
        DistManifest {
            site_id: self.site_id.clone(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            total_files: self.assets.len(),
            total_bytes,
            files,
        }
    }

    pub fn export_to_directory(&self, base_dir: &Path) -> std::io::Result<DistManifest> {
        fs::create_dir_all(base_dir)?;
        let manifest = self.build_manifest();
        for asset in &self.assets {
            let target_path = base_dir.join(&asset.relative_path);
            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(target_path, &asset.content)?;
        }
        let manifest_json = serde_json::to_string_pretty(&manifest)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        fs::write(base_dir.join("manifest.json"), manifest_json.as_bytes())?;
        Ok(manifest)
    }
}

/// A line item inside an inbound checkout order envelope.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RelayOrderItem {
    pub sku: String,
    pub title: String,
    pub unit_price: f64,
    pub quantity: u32,
    pub total: f64,
}

/// Inbound order relay payload received from edge/serverless webhook or direct static form.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrderRelayEnvelope {
    pub order_id: String,
    pub store_id: String,
    pub created_at: String,
    pub customer_name: String,
    pub customer_email: String,
    pub customer_phone: String,
    pub shipping_address: String,
    pub customer_afm: Option<String>,
    pub items: Vec<RelayOrderItem>,
    pub subtotal: f64,
    pub shipping_cost: f64,
    pub total_amount: f64,
    pub currency: String,
    pub payment_method: String,
    pub notes: Option<String>,
    pub signature: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum OrderRelayError {
    #[error("Customer name is required")]
    EmptyCustomerName,
    #[error("Customer email is required")]
    EmptyEmail,
    #[error("Customer phone is required")]
    EmptyPhone,
    #[error("Shipping address is required")]
    EmptyAddress,
    #[error("Order must contain at least one item")]
    EmptyItems,
    #[error("Item '{0}' has zero quantity")]
    ZeroQuantity(String),
    #[error("Item '{0}' has negative unit price: {1}")]
    NegativePrice(String, f64),
    #[error("Calculated subtotal ({calculated:.2}) differs from reported subtotal ({reported:.2})")]
    SubtotalMismatch { calculated: f64, reported: f64 },
    #[error("Calculated total ({calculated:.2}) differs from reported total ({reported:.2})")]
    TotalMismatch { calculated: f64, reported: f64 },
    #[error("Signature verification failed for order {0}")]
    InvalidSignature(String),
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
}

impl OrderRelayEnvelope {
    pub fn compute_signature(secret_key: &str, store_id: &str, order_id: &str, email: &str, total: f64) -> String {
        let mut hasher = Sha256::new();
        let payload = format!("{}:{}:{}:{}:{:.2}:{}", secret_key, store_id, order_id, email, total, secret_key);
        hasher.update(payload.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn validate(&self, secret_key: Option<&str>) -> Result<(), OrderRelayError> {
        if self.customer_name.trim().is_empty() { return Err(OrderRelayError::EmptyCustomerName); }
        if self.customer_email.trim().is_empty() { return Err(OrderRelayError::EmptyEmail); }
        if self.customer_phone.trim().is_empty() { return Err(OrderRelayError::EmptyPhone); }
        if self.shipping_address.trim().is_empty() { return Err(OrderRelayError::EmptyAddress); }
        if self.items.is_empty() { return Err(OrderRelayError::EmptyItems); }

        let mut calc_subtotal = 0.0;
        for item in &self.items {
            if item.quantity == 0 { return Err(OrderRelayError::ZeroQuantity(item.sku.clone())); }
            if item.unit_price < 0.0 { return Err(OrderRelayError::NegativePrice(item.sku.clone(), item.unit_price)); }
            calc_subtotal += (item.unit_price * (item.quantity as f64) * 100.0).round() / 100.0;
        }

        if (calc_subtotal - self.subtotal).abs() > 0.015 {
            return Err(OrderRelayError::SubtotalMismatch { calculated: calc_subtotal, reported: self.subtotal });
        }

        let calc_total = ((self.subtotal + self.shipping_cost) * 100.0).round() / 100.0;
        if (calc_total - self.total_amount).abs() > 0.015 {
            return Err(OrderRelayError::TotalMismatch { calculated: calc_total, reported: self.total_amount });
        }

        if let Some(secret) = secret_key {
            let expected_sig = Self::compute_signature(secret, &self.store_id, &self.order_id, &self.customer_email, self.total_amount);
            if self.signature.as_deref() != Some(&expected_sig) {
                return Err(OrderRelayError::InvalidSignature(self.order_id.clone()));
            }
        }
        Ok(())
    }
}

/// Initializes SQLite inbound orders and order items tables.
pub fn init_inbound_tables(conn: &Connection) -> SqlResult<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS inbound_web_orders (
            order_id TEXT PRIMARY KEY, store_id TEXT NOT NULL, created_at TEXT NOT NULL,
            customer_name TEXT NOT NULL, customer_email TEXT NOT NULL, customer_phone TEXT NOT NULL,
            shipping_address TEXT NOT NULL, customer_afm TEXT, subtotal REAL NOT NULL,
            shipping_cost REAL NOT NULL, total_amount REAL NOT NULL, currency TEXT NOT NULL,
            payment_method TEXT NOT NULL, notes TEXT, status TEXT NOT NULL DEFAULT 'PENDING',
            received_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS inbound_web_order_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            order_id TEXT NOT NULL REFERENCES inbound_web_orders(order_id),
            sku TEXT NOT NULL, title TEXT NOT NULL, unit_price REAL NOT NULL,
            quantity INTEGER NOT NULL, total REAL NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_inbound_orders_store ON inbound_web_orders(store_id, status);",
    )
}

/// Ingests and persists a validated order relay envelope into the SQLite store.
pub fn ingest_order_envelope(
    conn: &mut Connection,
    envelope: &OrderRelayEnvelope,
    secret_key: Option<&str>,
) -> Result<(), OrderRelayError> {
    envelope.validate(secret_key)?;
    let tx = conn.transaction()?;
    let received_at = chrono::Utc::now().to_rfc3339();

    tx.execute(
        "INSERT INTO inbound_web_orders (
            order_id, store_id, created_at, customer_name, customer_email,
            customer_phone, shipping_address, customer_afm, subtotal, shipping_cost,
            total_amount, currency, payment_method, notes, status, received_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, 'PENDING', ?15)",
        params![
            envelope.order_id, envelope.store_id, envelope.created_at, envelope.customer_name,
            envelope.customer_email, envelope.customer_phone, envelope.shipping_address,
            envelope.customer_afm, envelope.subtotal, envelope.shipping_cost, envelope.total_amount,
            envelope.currency, envelope.payment_method, envelope.notes, received_at
        ],
    )?;

    for item in &envelope.items {
        tx.execute(
            "INSERT INTO inbound_web_order_items (order_id, sku, title, unit_price, quantity, total)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![envelope.order_id, item.sku, item.title, item.unit_price, item.quantity, item.total],
        )?;
    }

    tx.commit()?;
    Ok(())
}

/// Retrieves the count of pending inbound web orders for a given store.
pub fn query_pending_orders_count(conn: &Connection, store_id: &str) -> SqlResult<usize> {
    let mut stmt = conn.prepare("SELECT count(*) FROM inbound_web_orders WHERE store_id = ?1 AND status = 'PENDING'")?;
    stmt.query_row(params![store_id], |r| r.get(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dist_bundle_packaging_and_manifest() {
        let compiled = CompiledWebPage {
            title: "Storefront".to_string(),
            html_body: "<h1>Storefront</h1>".to_string(),
            css_styles: "body { margin: 0; }".to_string(),
            js_runtime: "".to_string(),
            standalone_html: "<html><body><h1>Storefront</h1></body></html>".to_string(),
        };

        let mut bundle = SiteDistBundle::from_compiled_page("artisan-store", &compiled);
        bundle.add_asset("assets/icon.svg", "image/svg+xml", b"<svg></svg>".to_vec());

        let temp_dir = std::env::temp_dir().join(format!("proteus_dist_test_{}", uuid::Uuid::new_v4()));
        let manifest = bundle.export_to_directory(&temp_dir).unwrap();

        assert_eq!(manifest.site_id, "artisan-store");
        assert_eq!(manifest.total_files, 2);
        assert!(temp_dir.join("index.html").exists());
        assert!(temp_dir.join("assets/icon.svg").exists());
        assert!(temp_dir.join("manifest.json").exists());

        let _ = fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_order_relay_validation_math_invariants() {
        let secret = "sovereign_store_secret_99";
        let mut envelope = OrderRelayEnvelope {
            order_id: "ORD-2026-001".to_string(),
            store_id: "store_thessaloniki".to_string(),
            created_at: "2026-10-12T20:00:00Z".to_string(),
            customer_name: "Nikos Papadopoulos".to_string(),
            customer_email: "nikos@example.com".to_string(),
            customer_phone: "+306912345678".to_string(),
            shipping_address: "Tsimiski 45, Thessaloniki".to_string(),
            customer_afm: None,
            items: vec![
                RelayOrderItem {
                    sku: "SKU-BREAD-01".to_string(),
                    title: "Sourdough Bread".to_string(),
                    unit_price: 3.50,
                    quantity: 2,
                    total: 7.00,
                },
                RelayOrderItem {
                    sku: "SKU-OIL-02".to_string(),
                    title: "Olive Oil 1L".to_string(),
                    unit_price: 11.00,
                    quantity: 1,
                    total: 11.00,
                },
            ],
            subtotal: 18.00,
            shipping_cost: 3.00,
            total_amount: 21.00,
            currency: "EUR".to_string(),
            payment_method: "COD".to_string(),
            notes: Some("Leave at ground floor door".to_string()),
            signature: None,
        };

        envelope.signature = Some(OrderRelayEnvelope::compute_signature(
            secret,
            &envelope.store_id,
            &envelope.order_id,
            &envelope.customer_email,
            envelope.total_amount,
        ));

        assert!(envelope.validate(Some(secret)).is_ok());

        envelope.total_amount = 10.00;
        assert!(envelope.validate(Some(secret)).is_err());
    }

    #[test]
    fn test_order_relay_sqlite_persistence() {
        let mut conn = Connection::open_in_memory().unwrap();
        init_inbound_tables(&conn).unwrap();

        let envelope = OrderRelayEnvelope {
            order_id: "ORD-999".to_string(),
            store_id: "store_main".to_string(),
            created_at: "2026-10-12T21:00:00Z".to_string(),
            customer_name: "Elena Georgiou".to_string(),
            customer_email: "elena@example.gr".to_string(),
            customer_phone: "+302310123456".to_string(),
            shipping_address: "Egnatia 12, Thessaloniki".to_string(),
            customer_afm: Some("123456789".to_string()),
            items: vec![RelayOrderItem {
                sku: "SKU-HONEY-01".to_string(),
                title: "Thyme Honey".to_string(),
                unit_price: 9.50,
                quantity: 1,
                total: 9.50,
            }],
            subtotal: 9.50,
            shipping_cost: 2.50,
            total_amount: 12.00,
            currency: "EUR".to_string(),
            payment_method: "CARD".to_string(),
            notes: None,
            signature: None,
        };

        ingest_order_envelope(&mut conn, &envelope, None).unwrap();
        let pending_count = query_pending_orders_count(&conn, "store_main").unwrap();
        assert_eq!(pending_count, 1);
    }
}
