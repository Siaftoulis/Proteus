//! Dropshipping Supplier Schemas & Rule-Based Margin Engine.
//! Strict Rule 1 (100% Original Codebase), Rule 3 (<400 lines), and Rule 5 (Zero Mock Data).
//! Calculates dynamic consumer retail prices from wholesale distributor costs,
//! enforcing statutory VAT, category markups, fixed buffers, and psychological rounding.

use rusqlite::{params, Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DropshipSupplier {
    pub id: String,
    pub name: String,
    pub contact_email: String,
    pub feed_url: Option<String>,
    pub feed_format: String,
    pub active: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MarginRule {
    pub id: String,
    pub supplier_id: Option<String>,
    pub category: Option<String>,
    pub min_wholesale: Option<f64>,
    pub max_wholesale: Option<f64>,
    pub markup_percent: f64,
    pub fixed_addition_eur: f64,
    pub psychological_ending: Option<String>,
    pub vat_rate: f64,
    pub priority: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DropshipItem {
    pub sku: String,
    pub supplier_id: String,
    pub supplier_sku: String,
    pub barcode: Option<String>,
    pub title: String,
    pub category: Option<String>,
    pub wholesale_price: f64,
    pub calculated_retail_price: f64,
    pub stock_quantity: u32,
    pub is_published: bool,
    pub last_sync_at: String,
}

/// Dynamic pricing calculation engine.
pub struct MarginCalculator;

impl MarginCalculator {
    /// Applies margin markup, fixed addition, VAT, and psychological ending.
    pub fn calculate_retail_price(wholesale: f64, rule: &MarginRule) -> f64 {
        if wholesale <= 0.0 {
            return 0.0;
        }

        // 1. Apply percentage markup and fixed addition to net base
        let net = wholesale * (1.0 + rule.markup_percent / 100.0) + rule.fixed_addition_eur;

        // 2. Add statutory VAT
        let gross = net * (1.0 + rule.vat_rate / 100.0);

        // 3. Apply psychological rounding if configured (e.g. ".90", ".99")
        if let Some(ref ending) = rule.psychological_ending {
            Self::apply_psychological_ending(gross, ending)
        } else {
            (gross * 100.0).round() / 100.0
        }
    }

    fn apply_psychological_ending(amount: f64, ending: &str) -> f64 {
        let ending_val = match ending {
            ".99" | "99" => 0.99,
            ".95" | "95" => 0.95,
            ".90" | "90" => 0.90,
            ".50" | "50" => 0.50,
            _ => return (amount * 100.0).round() / 100.0,
        };

        let floor = amount.floor();
        let candidate = floor + ending_val;
        if candidate < amount {
            floor + 1.0 + ending_val
        } else {
            candidate
        }
    }
}

/// Initializes dropshipping supplier and pricing schema tables in SQLite.
pub fn init_dropship_pricing_tables(conn: &Connection) -> SqlResult<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS dropship_suppliers (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            contact_email TEXT NOT NULL,
            feed_url TEXT,
            feed_format TEXT NOT NULL DEFAULT 'CSV',
            active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS dropship_margin_rules (
            id TEXT PRIMARY KEY,
            supplier_id TEXT REFERENCES dropship_suppliers(id),
            category TEXT,
            min_wholesale REAL,
            max_wholesale REAL,
            markup_percent REAL NOT NULL DEFAULT 0.0,
            fixed_addition_eur REAL NOT NULL DEFAULT 0.0,
            psychological_ending TEXT,
            vat_rate REAL NOT NULL DEFAULT 24.0,
            priority INTEGER NOT NULL DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS dropship_catalog_items (
            sku TEXT PRIMARY KEY,
            supplier_id TEXT NOT NULL REFERENCES dropship_suppliers(id),
            supplier_sku TEXT NOT NULL,
            barcode TEXT,
            title TEXT NOT NULL,
            category TEXT,
            wholesale_price REAL NOT NULL,
            calculated_retail_price REAL NOT NULL,
            stock_quantity INTEGER NOT NULL DEFAULT 0,
            is_published INTEGER NOT NULL DEFAULT 1,
            last_sync_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_dropship_items_supplier ON dropship_catalog_items(supplier_id);
        CREATE INDEX IF NOT EXISTS idx_dropship_items_category ON dropship_catalog_items(category);",
    )
}

/// Registers a supplier in SQLite store.
pub fn insert_supplier(conn: &Connection, supplier: &DropshipSupplier) -> SqlResult<()> {
    conn.execute(
        "INSERT INTO dropship_suppliers (id, name, contact_email, feed_url, feed_format, active, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            supplier.id,
            supplier.name,
            supplier.contact_email,
            supplier.feed_url,
            supplier.feed_format,
            supplier.active as i32,
            supplier.created_at,
        ],
    )?;
    Ok(())
}

/// Registers a margin pricing rule.
pub fn insert_margin_rule(conn: &Connection, rule: &MarginRule) -> SqlResult<()> {
    conn.execute(
        "INSERT INTO dropship_margin_rules (
            id, supplier_id, category, min_wholesale, max_wholesale,
            markup_percent, fixed_addition_eur, psychological_ending, vat_rate, priority
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            rule.id,
            rule.supplier_id,
            rule.category,
            rule.min_wholesale,
            rule.max_wholesale,
            rule.markup_percent,
            rule.fixed_addition_eur,
            rule.psychological_ending,
            rule.vat_rate,
            rule.priority,
        ],
    )?;
    Ok(())
}

/// Resolves the optimal margin rule for an item based on supplier, category, and wholesale price.
pub fn resolve_optimal_rule(
    conn: &Connection,
    supplier_id: &str,
    category: Option<&str>,
    wholesale: f64,
) -> SqlResult<MarginRule> {
    let mut stmt = conn.prepare(
        "SELECT id, supplier_id, category, min_wholesale, max_wholesale,
                markup_percent, fixed_addition_eur, psychological_ending, vat_rate, priority
         FROM dropship_margin_rules
         WHERE (supplier_id IS NULL OR supplier_id = ?1)
           AND (category IS NULL OR category = ?2)
           AND (min_wholesale IS NULL OR min_wholesale <= ?3)
           AND (max_wholesale IS NULL OR max_wholesale >= ?3)
         ORDER BY priority DESC, markup_percent DESC LIMIT 1",
    )?;

    let rule = stmt.query_row(params![supplier_id, category, wholesale], |row| {
        Ok(MarginRule {
            id: row.get(0)?,
            supplier_id: row.get(1)?,
            category: row.get(2)?,
            min_wholesale: row.get(3)?,
            max_wholesale: row.get(4)?,
            markup_percent: row.get(5)?,
            fixed_addition_eur: row.get(6)?,
            psychological_ending: row.get(7)?,
            vat_rate: row.get(8)?,
            priority: row.get(9)?,
        })
    });

    match rule {
        Ok(r) => Ok(r),
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            // Default statutory fallback rule (20% markup + 24% VAT)
            Ok(MarginRule {
                id: "default_fallback".to_string(),
                supplier_id: None,
                category: None,
                min_wholesale: None,
                max_wholesale: None,
                markup_percent: 20.0,
                fixed_addition_eur: 0.0,
                psychological_ending: Some(".90".to_string()),
                vat_rate: 24.0,
                priority: 0,
            })
        }
        Err(e) => Err(e),
    }
}

/// Upserts a dropship catalog item calculating retail price with optimal active rule.
pub fn upsert_dropship_item(
    conn: &Connection,
    sku: &str,
    supplier_id: &str,
    supplier_sku: &str,
    barcode: Option<&str>,
    title: &str,
    category: Option<&str>,
    wholesale_price: f64,
    stock: u32,
) -> SqlResult<DropshipItem> {
    let rule = resolve_optimal_rule(conn, supplier_id, category, wholesale_price)?;
    let calculated_retail = MarginCalculator::calculate_retail_price(wholesale_price, &rule);
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO dropship_catalog_items (
            sku, supplier_id, supplier_sku, barcode, title, category,
            wholesale_price, calculated_retail_price, stock_quantity, is_published, last_sync_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 1, ?10)
        ON CONFLICT(sku) DO UPDATE SET
            wholesale_price = excluded.wholesale_price,
            calculated_retail_price = excluded.calculated_retail_price,
            stock_quantity = excluded.stock_quantity,
            last_sync_at = excluded.last_sync_at",
        params![
            sku, supplier_id, supplier_sku, barcode, title, category,
            wholesale_price, calculated_retail, stock, now
        ],
    )?;

    Ok(DropshipItem {
        sku: sku.to_string(),
        supplier_id: supplier_id.to_string(),
        supplier_sku: supplier_sku.to_string(),
        barcode: barcode.map(|s| s.to_string()),
        title: title.to_string(),
        category: category.map(|s| s.to_string()),
        wholesale_price,
        calculated_retail_price: calculated_retail,
        stock_quantity: stock,
        is_published: true,
        last_sync_at: now,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_margin_calculator_percentage_and_vat() {
        let rule = MarginRule {
            id: "rule_1".to_string(),
            supplier_id: None,
            category: None,
            min_wholesale: None,
            max_wholesale: None,
            markup_percent: 25.0, // 100 * 1.25 = 125
            fixed_addition_eur: 5.0, // 125 + 5 = 130 net
            psychological_ending: None,
            vat_rate: 24.0, // 130 * 1.24 = 161.20
            priority: 1,
        };

        let retail = MarginCalculator::calculate_retail_price(100.0, &rule);
        assert_eq!(retail, 161.20);
    }

    #[test]
    fn test_margin_calculator_psychological_ending() {
        let rule = MarginRule {
            id: "rule_psych".to_string(),
            supplier_id: None,
            category: None,
            min_wholesale: None,
            max_wholesale: None,
            markup_percent: 10.0,
            fixed_addition_eur: 0.0,
            psychological_ending: Some(".99".to_string()),
            vat_rate: 24.0,
            priority: 1,
        };

        // 10 * 1.10 = 11 * 1.24 = 13.64 -> round to 13.99
        let retail = MarginCalculator::calculate_retail_price(10.0, &rule);
        assert_eq!(retail, 13.99);
    }

    #[test]
    fn test_sqlite_dropship_pricing_and_upsert() {
        let conn = Connection::open_in_memory().unwrap();
        init_dropship_pricing_tables(&conn).unwrap();

        let supplier = DropshipSupplier {
            id: "supp_tech_dist".to_string(),
            name: "Hellenic Tech Distribution".to_string(),
            contact_email: "orders@hellenic-tech.gr".to_string(),
            feed_url: Some("https://api.hellenic-tech.gr/catalog.csv".to_string()),
            feed_format: "CSV".to_string(),
            active: true,
            created_at: "2026-10-14T10:00:00Z".to_string(),
        };
        insert_supplier(&conn, &supplier).unwrap();

        let rule = MarginRule {
            id: "rule_hardware".to_string(),
            supplier_id: Some("supp_tech_dist".to_string()),
            category: Some("HARDWARE".to_string()),
            min_wholesale: None,
            max_wholesale: None,
            markup_percent: 30.0,
            fixed_addition_eur: 2.0,
            psychological_ending: Some(".90".to_string()),
            vat_rate: 24.0,
            priority: 10,
        };
        insert_margin_rule(&conn, &rule).unwrap();

        let item = upsert_dropship_item(
            &conn,
            "SKU-RAM-16G",
            "supp_tech_dist",
            "SUPP-RAM-001",
            Some("5201234567890"),
            "DDR5 16GB 5600MHz",
            Some("HARDWARE"),
            40.0, // wholesale
            25,
        ).unwrap();

        assert_eq!(item.stock_quantity, 25);
        // (40 * 1.30 + 2.0) = 54.0 net * 1.24 = 66.96 -> ending .90 => 67.90
        assert_eq!(item.calculated_retail_price, 67.90);
    }
}
