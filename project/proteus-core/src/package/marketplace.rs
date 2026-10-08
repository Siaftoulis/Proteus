//! Template Marketplace Catalog & Discovery Engine for Proteus BOS.
//!
//! Enables discovering, searching, filtering, and 1-click converting industry business templates
//! (.pr packages) across Retail, Automotive, Healthcare, Hospitality, Services, and Logistics.

use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use super::PrPackage;

/// Industry category classification for business templates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemplateCategory {
    Retail,
    Automotive,
    Healthcare,
    Hospitality,
    Services,
    Logistics,
}

impl TemplateCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Retail => "Retail",
            Self::Automotive => "Automotive",
            Self::Healthcare => "Healthcare",
            Self::Hospitality => "Hospitality",
            Self::Services => "Services",
            Self::Logistics => "Logistics",
        }
    }

    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "retail" | "λιανική" => Some(Self::Retail),
            "automotive" | "αυτοκίνητα" | "συνεργείο" => Some(Self::Automotive),
            "healthcare" | "υγεία" | "ιατρείο" => Some(Self::Healthcare),
            "hospitality" | "εστίαση" => Some(Self::Hospitality),
            "services" | "υπηρεσίες" => Some(Self::Services),
            "logistics" | "μεταφορές" | "αποθήκη" => Some(Self::Logistics),
            _ => None,
        }
    }
}

/// Metadata, rating, and schema details of a verified marketplace template.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemplateListing {
    pub id: String,
    pub title: String,
    pub category: TemplateCategory,
    pub description: String,
    pub author: String,
    pub version: String,
    pub price_eur: f64,
    pub rating: f32,
    pub downloads_count: u32,
    pub tags: Vec<String>,
    pub verified: bool,
    pub preview_ddl: String,
}

/// In-memory catalog of industry templates and search engine.
pub struct TemplateCatalog {
    pub templates: Vec<TemplateListing>,
}

impl Default for TemplateCatalog {
    fn default() -> Self {
        Self {
            templates: Self::default_templates(),
        }
    }
}

impl TemplateCatalog {
    /// Authentic business templates created for sovereign Greek and European SMBs.
    pub fn default_templates() -> Vec<TemplateListing> {
        vec![
            TemplateListing {
                id: "TPL-MOTO-SERVICE".to_string(),
                title: "Μοτοσυκλέτες & Συνεργείο Pro".to_string(),
                category: TemplateCategory::Automotive,
                description: "Διαχείριση εντολών επισκευής μοτοσυκλετών, έλεγχος ελαστικών και ιστορικό συντήρησης.".to_string(),
                author: "PCD Automotive Specialist".to_string(),
                version: "1.2.0".to_string(),
                price_eur: 0.0,
                rating: 4.9,
                downloads_count: 245,
                tags: vec!["moto".into(), "service".into(), "vin".into()],
                verified: true,
                preview_ddl: "CREATE TABLE IF NOT EXISTS moto_service_inspections (id TEXT PRIMARY KEY, vin_number TEXT NOT NULL, engine_cc INTEGER, tire_condition TEXT, created_at TEXT NOT NULL);".to_string(),
            },
            TemplateListing {
                id: "TPL-RETAIL-TOUCH-POS".to_string(),
                title: "Λιανική & Ταμείο Touch POS".to_string(),
                category: TemplateCategory::Retail,
                description: "Πλήρες σύστημα λιανικής: barcode scanning, έλεγχος αποθέματος και ESC/POS θερμική εκτύπωση.".to_string(),
                author: "PCDA Retail Group".to_string(),
                version: "2.0.1".to_string(),
                price_eur: 0.0,
                rating: 5.0,
                downloads_count: 580,
                tags: vec!["pos".into(), "retail".into(), "receipt".into()],
                verified: true,
                preview_ddl: "CREATE TABLE IF NOT EXISTS retail_inventory_sync (id TEXT PRIMARY KEY, sku TEXT NOT NULL, barcode TEXT NOT NULL, stock_qty INTEGER, created_at TEXT NOT NULL);".to_string(),
            },
            TemplateListing {
                id: "TPL-DENTAL-CLINIC".to_string(),
                title: "Οδοντιατρείο & Καρτέλα Ασθενούς".to_string(),
                category: TemplateCategory::Healthcare,
                description: "Ιατρικό ιστορικό, ραντεβού ασθενών, διαχείριση συγκατάθεσης GDPR και ασφαλιστικά ταμεία.".to_string(),
                author: "PCSS Healthcare Lead".to_string(),
                version: "1.0.4".to_string(),
                price_eur: 49.0,
                rating: 4.8,
                downloads_count: 112,
                tags: vec!["clinic".into(), "amka".into(), "gdpr".into()],
                verified: true,
                preview_ddl: "CREATE TABLE IF NOT EXISTS clinic_patient_records (id TEXT PRIMARY KEY, amka TEXT NOT NULL, full_name TEXT NOT NULL, diagnosis TEXT, created_at TEXT NOT NULL);".to_string(),
            },
            TemplateListing {
                id: "TPL-LOGISTICS-WMS".to_string(),
                title: "Αποθήκη & Spatial WMS Light".to_string(),
                category: TemplateCategory::Logistics,
                description: "Διαχείριση θέσεων ραφιών, παλετών, διαδρομές directed putaway και δελτία αποστολής.".to_string(),
                author: "Proteus WMS Guild".to_string(),
                version: "1.1.0".to_string(),
                price_eur: 0.0,
                rating: 4.7,
                downloads_count: 180,
                tags: vec!["wms".into(), "warehouse".into(), "shipping".into()],
                verified: true,
                preview_ddl: "CREATE TABLE IF NOT EXISTS wms_storage_racks (id TEXT PRIMARY KEY, rack_code TEXT NOT NULL, max_weight_kg REAL, created_at TEXT NOT NULL);".to_string(),
            },
        ]
    }

    /// Searches and filters templates by keyword, category, and maximum price limit.
    pub fn search(
        &self,
        query: &str,
        category: Option<TemplateCategory>,
        max_price: Option<f64>,
    ) -> Vec<TemplateListing> {
        let q = query.trim().to_lowercase();
        self.templates
            .iter()
            .filter(|t| {
                if let Some(cat) = category {
                    if t.category != cat {
                        return false;
                    }
                }
                if let Some(price) = max_price {
                    if t.price_eur > price {
                        return false;
                    }
                }
                if !q.is_empty() {
                    let in_title = t.title.to_lowercase().contains(&q);
                    let in_desc = t.description.to_lowercase().contains(&q);
                    let in_tags = t.tags.iter().any(|tag| tag.to_lowercase().contains(&q));
                    if !in_title && !in_desc && !in_tags {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect()
    }

    /// Converts a template listing into a mountable `PrPackage`.
    pub fn to_pr_package(&self, template_id: &str) -> Option<PrPackage> {
        let tpl = self.templates.iter().find(|t| t.id == template_id)?;
        let mut pkg = PrPackage::new(&tpl.id, &tpl.title, &tpl.author);
        pkg.manifest.version = tpl.version.clone();
        pkg.schema.ddl_statements.push(tpl.preview_ddl.clone());
        Some(pkg)
    }
}

/// Initializes SQLite schema for template marketplace cache.
pub fn ensure_template_marketplace_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS template_marketplace_cache (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            category TEXT NOT NULL,
            description TEXT NOT NULL,
            author TEXT NOT NULL,
            version TEXT NOT NULL,
            price_eur REAL NOT NULL,
            rating REAL NOT NULL,
            downloads_count INTEGER NOT NULL,
            tags TEXT NOT NULL,
            verified INTEGER NOT NULL,
            preview_ddl TEXT NOT NULL,
            cached_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_template_category ON template_marketplace_cache(category);",
    )
}

/// Persists template listings to the local SQLite cache.
pub fn cache_template_listings(
    conn: &Connection,
    templates: &[TemplateListing],
) -> rusqlite::Result<()> {
    let now = Utc::now().to_rfc3339();
    let mut stmt = conn.prepare(
        "INSERT OR REPLACE INTO template_marketplace_cache (
            id, title, category, description, author, version, price_eur, rating, downloads_count, tags, verified, preview_ddl, cached_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
    )?;

    for t in templates {
        let tags_json = serde_json::to_string(&t.tags).unwrap_or_else(|_| "[]".into());
        stmt.execute(params![
            t.id,
            t.title,
            t.category.as_str(),
            t.description,
            t.author,
            t.version,
            t.price_eur,
            t.rating,
            t.downloads_count,
            tags_json,
            t.verified as i32,
            t.preview_ddl,
            now
        ])?;
    }
    Ok(())
}

/// Queries cached templates from SQLite.
pub fn query_cached_templates(
    conn: &Connection,
    category_filter: Option<&str>,
) -> rusqlite::Result<Vec<TemplateListing>> {
    let mut sql = "SELECT id, title, category, description, author, version, price_eur, rating, downloads_count, tags, verified, preview_ddl FROM template_marketplace_cache".to_string();
    if let Some(cat) = category_filter {
        sql.push_str(&format!(" WHERE category = '{}'", cat));
    }
    sql.push_str(" ORDER BY downloads_count DESC");

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |row| {
        let cat_str: String = row.get(2)?;
        let tags_str: String = row.get(9)?;
        let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();
        let verified_int: i32 = row.get(10)?;

        Ok(TemplateListing {
            id: row.get(0)?,
            title: row.get(1)?,
            category: TemplateCategory::from_str_loose(&cat_str).unwrap_or(TemplateCategory::Services),
            description: row.get(3)?,
            author: row.get(4)?,
            version: row.get(5)?,
            price_eur: row.get(6)?,
            rating: row.get(7)?,
            downloads_count: row.get(8)?,
            tags,
            verified: verified_int == 1,
            preview_ddl: row.get(11)?,
        })
    })?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_catalog_search_by_query_and_category() {
        let catalog = TemplateCatalog::default();

        let moto = catalog.search("μοτο", None, None);
        assert_eq!(moto.len(), 1);
        assert_eq!(moto[0].category, TemplateCategory::Automotive);

        let retail = catalog.search("", Some(TemplateCategory::Retail), None);
        assert_eq!(retail.len(), 1);
        assert_eq!(retail[0].id, "TPL-RETAIL-TOUCH-POS");

        let free_only = catalog.search("", None, Some(0.0));
        assert!(free_only.iter().all(|t| t.price_eur == 0.0));
    }

    #[test]
    fn test_to_pr_package_conversion_and_seal() {
        let catalog = TemplateCatalog::default();
        let pkg = catalog.to_pr_package("TPL-RETAIL-TOUCH-POS").unwrap();

        assert_eq!(pkg.manifest.bundle_id, "TPL-RETAIL-TOUCH-POS");
        assert_eq!(pkg.schema.ddl_statements.len(), 1);
        assert!(pkg.schema.ddl_statements[0].contains("retail_inventory_sync"));
        let bytes = pkg.to_bytes().unwrap();
        assert!(bytes.starts_with(b"PRPK"));
    }

    #[test]
    fn test_template_marketplace_sqlite_cache_roundtrip() {
        let conn = Connection::open_in_memory().unwrap();
        ensure_template_marketplace_schema(&conn).unwrap();

        let catalog = TemplateCatalog::default();
        cache_template_listings(&conn, &catalog.templates).unwrap();

        let loaded = query_cached_templates(&conn, None).unwrap();
        assert_eq!(loaded.len(), catalog.templates.len());

        let automotive_only = query_cached_templates(&conn, Some("Automotive")).unwrap();
        assert_eq!(automotive_only.len(), 1);
        assert_eq!(automotive_only[0].id, "TPL-MOTO-SERVICE");
    }
}
