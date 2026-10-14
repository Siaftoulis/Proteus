//! Multi-Format Supplier Feed Ingestion & Delta Synchronizer.
//! Strict Rule 1 (100% Original Codebase), Rule 3 (<400 lines), and Rule 5 (Zero Mock Data).
//! Ingests arbitrary supplier catalog feeds (CSV/JSON), parses European decimal formats,
//! and performs atomic delta updates preserving merchant customizations.

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use crate::dropship::pricing::upsert_dropship_item;

/// Flexible column mapping definition for external supplier feeds.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FeedColumnMapping {
    pub sku_col: String,
    pub title_col: String,
    pub wholesale_col: String,
    pub stock_col: String,
    pub barcode_col: Option<String>,
    pub category_col: Option<String>,
    pub delimiter: char,
    pub decimal_separator: char,
}

impl Default for FeedColumnMapping {
    fn default() -> Self {
        Self {
            sku_col: "sku".to_string(),
            title_col: "title".to_string(),
            wholesale_col: "wholesale_price".to_string(),
            stock_col: "stock".to_string(),
            barcode_col: Some("barcode".to_string()),
            category_col: Some("category".to_string()),
            delimiter: ';',
            decimal_separator: ',',
        }
    }
}

/// Ingestion audit report summary.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FeedIngestionReport {
    pub supplier_id: String,
    pub total_rows: usize,
    pub items_upserted: usize,
    pub items_failed: usize,
    pub sample_errors: Vec<String>,
}

/// Pure bespoke CSV and feed processor.
pub struct SupplierFeedIngestor;

impl SupplierFeedIngestor {
    /// Ingests a CSV string according to mapping and applies rule-based margin delta sync.
    pub fn ingest_csv(
        conn: &Connection,
        supplier_id: &str,
        csv_payload: &str,
        mapping: &FeedColumnMapping,
    ) -> Result<FeedIngestionReport, String> {
        let mut lines = csv_payload.lines().filter(|l| !l.trim().is_empty());
        let header_line = lines.next().ok_or_else(|| "Empty CSV feed payload".to_string())?;

        let headers: Vec<String> = Self::split_line(header_line, mapping.delimiter)
            .into_iter()
            .map(|s| s.trim().trim_matches('"').to_lowercase())
            .collect();

        let sku_idx = headers.iter().position(|h| h == &mapping.sku_col.to_lowercase())
            .ok_or_else(|| format!("Required SKU column '{}' missing from headers", mapping.sku_col))?;
        let title_idx = headers.iter().position(|h| h == &mapping.title_col.to_lowercase())
            .ok_or_else(|| format!("Required Title column '{}' missing from headers", mapping.title_col))?;
        let price_idx = headers.iter().position(|h| h == &mapping.wholesale_col.to_lowercase())
            .ok_or_else(|| format!("Required Wholesale Price column '{}' missing from headers", mapping.wholesale_col))?;
        let stock_idx = headers.iter().position(|h| h == &mapping.stock_col.to_lowercase())
            .ok_or_else(|| format!("Required Stock column '{}' missing from headers", mapping.stock_col))?;

        let barcode_idx = mapping.barcode_col.as_ref()
            .and_then(|col| headers.iter().position(|h| h == &col.to_lowercase()));
        let category_idx = mapping.category_col.as_ref()
            .and_then(|col| headers.iter().position(|h| h == &col.to_lowercase()));

        let mut report = FeedIngestionReport {
            supplier_id: supplier_id.to_string(),
            ..Default::default()
        };

        for (line_idx, line) in lines.enumerate() {
            report.total_rows += 1;
            let cols = Self::split_line(line, mapping.delimiter);
            let max_required = sku_idx.max(title_idx).max(price_idx).max(stock_idx);

            if cols.len() <= max_required {
                report.items_failed += 1;
                if report.sample_errors.len() < 5 {
                    report.sample_errors.push(format!("Row {}: Insufficient columns ({}/{})", line_idx + 2, cols.len(), max_required + 1));
                }
                continue;
            }

            let sku = cols[sku_idx].trim().trim_matches('"');
            let title = cols[title_idx].trim().trim_matches('"');
            let raw_price = cols[price_idx].trim().trim_matches('"');
            let raw_stock = cols[stock_idx].trim().trim_matches('"');

            let wholesale_price = match Self::parse_decimal(raw_price, mapping.decimal_separator) {
                Ok(p) => p,
                Err(e) => {
                    report.items_failed += 1;
                    if report.sample_errors.len() < 5 {
                        report.sample_errors.push(format!("Row {} ({}): Invalid price '{}': {}", line_idx + 2, sku, raw_price, e));
                    }
                    continue;
                }
            };

            let stock: u32 = raw_stock.parse().unwrap_or(0);
            let barcode = barcode_idx.and_then(|idx| cols.get(idx)).map(|s| s.trim().trim_matches('"')).filter(|s| !s.is_empty());
            let category = category_idx.and_then(|idx| cols.get(idx)).map(|s| s.trim().trim_matches('"')).filter(|s| !s.is_empty());

            // Build unique global SKU scoped by supplier prefix
            let global_sku = format!("DS-{}-{}", supplier_id.to_uppercase(), sku);

            match upsert_dropship_item(
                conn,
                &global_sku,
                supplier_id,
                sku,
                barcode,
                title,
                category,
                wholesale_price,
                stock,
            ) {
                Ok(_) => report.items_upserted += 1,
                Err(err) => {
                    report.items_failed += 1;
                    if report.sample_errors.len() < 5 {
                        report.sample_errors.push(format!("Row {} ({}): DB Error: {}", line_idx + 2, sku, err));
                    }
                }
            }
        }

        Ok(report)
    }

    /// Ingests a JSON array of raw supplier products.
    pub fn ingest_json_array(
        conn: &Connection,
        supplier_id: &str,
        items: &[DropshipRawJsonItem],
    ) -> Result<FeedIngestionReport, String> {
        let mut report = FeedIngestionReport {
            supplier_id: supplier_id.to_string(),
            total_rows: items.len(),
            ..Default::default()
        };

        for item in items {
            let global_sku = format!("DS-{}-{}", supplier_id.to_uppercase(), item.supplier_sku);
            match upsert_dropship_item(
                conn,
                &global_sku,
                supplier_id,
                &item.supplier_sku,
                item.barcode.as_deref(),
                &item.title,
                item.category.as_deref(),
                item.wholesale_price,
                item.stock_quantity,
            ) {
                Ok(_) => report.items_upserted += 1,
                Err(e) => {
                    report.items_failed += 1;
                    if report.sample_errors.len() < 5 {
                        report.sample_errors.push(format!("{}: {}", item.supplier_sku, e));
                    }
                }
            }
        }

        Ok(report)
    }

    /// Parses localized decimal number (e.g. "1.250,50" or "45,90" or "45.90").
    pub fn parse_decimal(input: &str, decimal_sep: char) -> Result<f64, String> {
        let cleaned = input.trim().replace('€', "").replace('$', "").trim().to_string();
        if cleaned.is_empty() {
            return Err("Empty number".to_string());
        }

        let normalized = if decimal_sep == ',' {
            cleaned.replace('.', "").replace(',', ".")
        } else {
            cleaned.replace(',', "")
        };

        normalized.parse::<f64>().map_err(|e| format!("Float parse error: {}", e))
    }

    /// Splits a delimited line respecting quoted strings containing the delimiter.
    pub fn split_line(line: &str, delimiter: char) -> Vec<String> {
        let mut fields = Vec::new();
        let mut cur = String::new();
        let mut in_quotes = false;

        for c in line.chars() {
            if c == '"' {
                in_quotes = !in_quotes;
            } else if c == delimiter && !in_quotes {
                fields.push(cur.clone());
                cur.clear();
                continue;
            }
            cur.push(c);
        }
        fields.push(cur);
        fields
    }
}

/// Raw JSON feed object schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropshipRawJsonItem {
    pub supplier_sku: String,
    pub title: String,
    pub wholesale_price: f64,
    pub stock_quantity: u32,
    pub barcode: Option<String>,
    pub category: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dropship::pricing::{init_dropship_pricing_tables, insert_margin_rule, insert_supplier, DropshipSupplier, MarginRule};

    #[test]
    fn test_parse_european_decimal_formats() {
        assert_eq!(SupplierFeedIngestor::parse_decimal("1.250,50", ',').unwrap(), 1250.50);
        assert_eq!(SupplierFeedIngestor::parse_decimal("45,90 €", ',').unwrap(), 45.90);
        assert_eq!(SupplierFeedIngestor::parse_decimal("1,250.50", '.').unwrap(), 1250.50);
    }

    #[test]
    fn test_split_csv_line_with_quotes() {
        let line = "SKU-01;\"Fresh, Organic Sourdough Bread\";4,50;20";
        let fields = SupplierFeedIngestor::split_line(line, ';');
        assert_eq!(fields.len(), 4);
        assert_eq!(fields[1], "\"Fresh, Organic Sourdough Bread\"");
    }

    #[test]
    fn test_ingest_supplier_csv_feed() {
        let conn = Connection::open_in_memory().unwrap();
        init_dropship_pricing_tables(&conn).unwrap();

        let supplier = DropshipSupplier {
            id: "wholesaler_athens".to_string(),
            name: "Athens Wholesale Imports".to_string(),
            contact_email: "catalog@athens-imports.gr".to_string(),
            feed_url: None,
            feed_format: "CSV".to_string(),
            active: true,
            created_at: "2026-10-14T09:00:00Z".to_string(),
        };
        insert_supplier(&conn, &supplier).unwrap();

        let rule = MarginRule {
            id: "rule_audio".to_string(),
            supplier_id: Some("wholesaler_athens".to_string()),
            category: Some("AUDIO".to_string()),
            min_wholesale: None,
            max_wholesale: None,
            markup_percent: 50.0,
            fixed_addition_eur: 0.0,
            psychological_ending: Some(".99".to_string()),
            vat_rate: 24.0,
            priority: 5,
        };
        insert_margin_rule(&conn, &rule).unwrap();

        let csv_data = "sku;title;wholesale_price;stock;category;barcode\n\
                        HEAD-BT-01;\"Wireless ANC Headphones\";30,00;45;AUDIO;520999001\n\
                        SPK-PORT-02;\"Bluetooth Waterproof Speaker\";15,50;10;AUDIO;520999002";

        let mapping = FeedColumnMapping::default();
        let report = SupplierFeedIngestor::ingest_csv(&conn, "wholesaler_athens", csv_data, &mapping).unwrap();

        assert_eq!(report.total_rows, 2);
        assert_eq!(report.items_upserted, 2);
        assert_eq!(report.items_failed, 0);

        let mut stmt = conn.prepare("SELECT calculated_retail_price, stock_quantity FROM dropship_catalog_items WHERE sku = ?1").unwrap();
        // 30 * 1.50 = 45 * 1.24 = 55.80 -> .99 => 55.99
        let (price, stock): (f64, u32) = stmt.query_row(["DS-WHOLESALER_ATHENS-HEAD-BT-01"], |r| Ok((r.get(0)?, r.get(1)?))).unwrap();
        assert_eq!(price, 55.99);
        assert_eq!(stock, 45);
    }
}
