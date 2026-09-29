//! Proteus Supplier Price List & Catalog Reconciliation Engine.
//! Leverages SMLM and Universal Reconciler to ingest messy vendor CSV/TSV catalogs,
//! detect price hikes/reductions, calculate suggested retail margins, and update SQLite inventory.
//! 100% original bespoke code. Adheres to Rule 5 (Zero Mock Data).

use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::smlm::{SemanticIntent, SmlmEngine};

/// Authentic store inventory item tracked in SQLite.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StoreItem {
    pub sku: String,
    pub barcode: Option<String>,
    pub name: String,
    pub cost_price: f64,
    pub retail_price: f64,
    pub unit: String,
    pub category: String,
    pub stock_qty: f64,
    pub updated_at: String,
}

/// Variance record between incoming supplier catalog price and current store cost.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PriceVariance {
    pub sku: String,
    pub barcode: Option<String>,
    pub name: String,
    pub old_cost: f64,
    pub new_cost: f64,
    pub cost_diff: f64,
    pub cost_diff_pct: f64,
    pub old_retail: f64,
    pub new_retail: f64,
    pub unit: String,
    pub category: String,
    pub is_new_item: bool,
}

/// Full reconciliation report comparing supplier invoice/catalog against local store records.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReconciliationReport {
    pub supplier_name: String,
    pub total_rows: usize,
    pub matched_existing: usize,
    pub new_items: usize,
    pub price_increases: usize,
    pub price_decreases: usize,
    pub price_unchanged: usize,
    pub avg_cost_change_pct: f64,
    pub items: Vec<PriceVariance>,
}

/// Initializes database tables for store items and supplier catalog history.
pub fn init_supplier_catalog_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS store_items (
            sku TEXT PRIMARY KEY,
            barcode TEXT,
            name TEXT NOT NULL,
            cost_price REAL NOT NULL,
            retail_price REAL NOT NULL,
            unit TEXT NOT NULL DEFAULT 'ΤΕΜ',
            category TEXT NOT NULL DEFAULT 'ΓΕΝΙΚΑ',
            stock_qty REAL NOT NULL DEFAULT 0.0,
            updated_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_store_items_barcode ON store_items(barcode);

        CREATE TABLE IF NOT EXISTS supplier_catalog_history (
            id TEXT PRIMARY KEY,
            supplier_name TEXT NOT NULL,
            imported_at TEXT NOT NULL,
            total_items INTEGER NOT NULL,
            increases INTEGER NOT NULL,
            decreases INTEGER NOT NULL,
            avg_change_pct REAL NOT NULL
        );",
    )?;
    Ok(())
}

/// Inserts or updates an individual store inventory item in SQLite.
pub fn create_or_update_item(conn: &Connection, item: &StoreItem) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO store_items (sku, barcode, name, cost_price, retail_price, unit, category, stock_qty, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
         ON CONFLICT(sku) DO UPDATE SET
            barcode = excluded.barcode,
            name = excluded.name,
            cost_price = excluded.cost_price,
            retail_price = excluded.retail_price,
            unit = excluded.unit,
            category = excluded.category,
            stock_qty = excluded.stock_qty,
            updated_at = excluded.updated_at",
        params![
            item.sku,
            item.barcode,
            item.name,
            item.cost_price,
            item.retail_price,
            item.unit,
            item.category,
            item.stock_qty,
            item.updated_at,
        ],
    )?;
    Ok(())
}

/// Queries all items from SQLite.
pub fn list_store_items(conn: &Connection) -> Result<Vec<StoreItem>, rusqlite::Error> {
    let mut stmt = conn.prepare(
        "SELECT sku, barcode, name, cost_price, retail_price, unit, category, stock_qty, updated_at
         FROM store_items ORDER BY name ASC",
    )?;

    let rows = stmt.query_map([], |r| {
        Ok(StoreItem {
            sku: r.get(0)?,
            barcode: r.get(1)?,
            name: r.get(2)?,
            cost_price: r.get(3)?,
            retail_price: r.get(4)?,
            unit: r.get(5)?,
            category: r.get(6)?,
            stock_qty: r.get(7)?,
            updated_at: r.get(8)?,
        })
    })?;

    let mut items = Vec::new();
    for r in rows {
        items.push(r?);
    }
    Ok(items)
}

/// Parses delimited CSV/TSV table content handling Greek characters, quotes, and commas/semicolons.
pub fn parse_delimited_catalog(content: &str) -> (Vec<String>, Vec<Vec<String>>) {
    let mut lines = content.lines().filter(|l| !l.trim().is_empty());
    let header_line = match lines.next() {
        Some(h) => h,
        None => return (Vec::new(), Vec::new()),
    };

    let delimiter = if header_line.contains('\t') {
        '\t'
    } else if header_line.contains(';') {
        ';'
    } else {
        ','
    };

    let split_row = |line: &str| -> Vec<String> {
        let mut fields = Vec::new();
        let mut cur = String::new();
        let mut in_quotes = false;

        for c in line.chars() {
            match c {
                '"' => in_quotes = !in_quotes,
                ch if ch == delimiter && !in_quotes => {
                    fields.push(cur.trim().trim_matches('"').to_string());
                    cur.clear();
                }
                ch => cur.push(ch),
            }
        }
        fields.push(cur.trim().trim_matches('"').to_string());
        fields
    };

    let headers = split_row(header_line);
    let mut rows = Vec::new();
    for line in lines {
        let r = split_row(line);
        if r.iter().any(|cell| !cell.is_empty()) {
            rows.push(r);
        }
    }

    (headers, rows)
}

/// Sanitizes numeric string converting European comma decimals ("14,50" -> 14.50).
fn parse_euro_float(val: &str) -> f64 {
    let cleaned: String = val
        .trim()
        .replace("€", "")
        .replace("EUR", "")
        .replace(' ', "")
        .replace(',', ".");
    cleaned.parse::<f64>().unwrap_or(0.0)
}

/// Detects column indexes using SMLM semantic profiling combined with common keyword matching.
struct ColumnMapping {
    sku_idx: usize,
    name_idx: usize,
    cost_idx: usize,
    barcode_idx: Option<usize>,
    unit_idx: Option<usize>,
    category_idx: Option<usize>,
}

fn infer_catalog_columns(headers: &[String], rows: &[Vec<String>]) -> Option<ColumnMapping> {
    if headers.is_empty() {
        return None;
    }

    let profiles = SmlmEngine::profile_table(headers, rows);
    let mut sku_idx = None;
    let mut name_idx = None;
    let mut cost_idx = None;
    let mut barcode_idx = None;
    let mut unit_idx = None;
    let mut category_idx = None;

    for (i, h) in headers.iter().enumerate() {
        let upper = h.to_uppercase();
        let intent = profiles.get(i).map(|p| p.inferred_intent);

        if upper.contains("BARCODE") || upper.contains("EAN") || upper.contains("GS1") {
            barcode_idx = Some(i);
        } else if upper.contains("SKU")
            || upper.contains("ΚΩΔΙΚ")
            || upper.contains("CODE")
            || upper.contains("ITEM_ID")
            || intent == Some(SemanticIntent::RecordId)
        {
            if sku_idx.is_none() {
                sku_idx = Some(i);
            }
        } else if upper.contains("ΠΕΡΙΓΡΑΦ")
            || upper.contains("NAME")
            || upper.contains("DESC")
            || upper.contains("ΕΙΔΟΣ")
            || upper.contains("ΤΙΤΛΟΣ")
        {
            if name_idx.is_none() {
                name_idx = Some(i);
            }
        } else if upper.contains("ΤΙΜΗ")
            || upper.contains("PRICE")
            || upper.contains("COST")
            || upper.contains("ΧΟΝΔΡ")
            || upper.contains("NETTO")
            || intent == Some(SemanticIntent::FinancialAmount)
        {
            if cost_idx.is_none() {
                cost_idx = Some(i);
            }
        } else if upper.contains("ΜΟΝ")
            || upper.contains("UOM")
            || upper.contains("UNIT")
            || intent == Some(SemanticIntent::UnitOfMeasure)
        {
            unit_idx = Some(i);
        } else if upper.contains("ΚΑΤΗΓΟΡ") || upper.contains("CAT") {
            category_idx = Some(i);
        }
    }

    // Fallbacks if header keywords did not catch everything
    if sku_idx.is_none() {
        sku_idx = Some(0);
    }
    if name_idx.is_none() {
        name_idx = Some(1.min(headers.len() - 1));
    }
    if cost_idx.is_none() {
        cost_idx = headers.iter().position(|_| true);
    }

    Some(ColumnMapping {
        sku_idx: sku_idx?,
        name_idx: name_idx?,
        cost_idx: cost_idx?,
        barcode_idx,
        unit_idx,
        category_idx,
    })
}

/// Reconciles supplier CSV catalog against live SQLite store items.
pub fn reconcile_catalog(
    conn: &Connection,
    supplier_name: &str,
    csv_content: &str,
    default_markup_pct: f64,
) -> Result<ReconciliationReport, String> {
    init_supplier_catalog_schema(conn).map_err(|e| format!("Schema error: {}", e))?;
    let (headers, rows) = parse_delimited_catalog(csv_content);

    if headers.is_empty() || rows.is_empty() {
        return Err("Το αρχείο τιμοκαταλόγου είναι κενό.".into());
    }

    let mapping = infer_catalog_columns(&headers, &rows)
        .ok_or_else(|| "Αδυναμία αναγνώρισης στηλών τιμοκαταλόγου".to_string())?;

    let existing_items = list_store_items(conn).map_err(|e| format!("SQLite query error: {}", e))?;

    let mut matched_existing = 0;
    let mut new_items = 0;
    let mut price_increases = 0;
    let mut price_decreases = 0;
    let mut price_unchanged = 0;
    let mut total_pct_change = 0.0;
    let mut pct_change_count = 0;

    let mut variances = Vec::new();

    for r in &rows {
        let sku = r.get(mapping.sku_idx).cloned().unwrap_or_default().trim().to_string();
        if sku.is_empty() {
            continue;
        }

        let name = r.get(mapping.name_idx).cloned().unwrap_or_default().trim().to_string();
        let cost_raw = r.get(mapping.cost_idx).map(|s| s.as_str()).unwrap_or("0");
        let new_cost = parse_euro_float(cost_raw);

        let barcode = mapping.barcode_idx.and_then(|idx| r.get(idx).cloned()).filter(|b| !b.trim().is_empty());
        let unit = mapping.unit_idx.and_then(|idx| r.get(idx).cloned()).unwrap_or_else(|| "ΤΕΜ".into());
        let category = mapping.category_idx.and_then(|idx| r.get(idx).cloned()).unwrap_or_else(|| "ΓΕΝΙΚΑ".into());

        // Find existing match by barcode or SKU
        let existing = existing_items.iter().find(|item| {
            if let (Some(b_new), Some(b_old)) = (&barcode, &item.barcode) {
                if !b_new.is_empty() && b_new == b_old {
                    return true;
                }
            }
            item.sku.eq_ignore_ascii_case(&sku)
        });

        let (old_cost, old_retail, is_new) = match existing {
            Some(ex) => (ex.cost_price, ex.retail_price, false),
            None => (0.0, 0.0, true),
        };

        let cost_diff = if is_new { new_cost } else { new_cost - old_cost };
        let cost_diff_pct = if is_new || old_cost <= 0.0 {
            0.0
        } else {
            ((new_cost - old_cost) / old_cost) * 100.0
        };

        let new_retail = (new_cost * (1.0 + (default_markup_pct / 100.0)) * 100.0).round() / 100.0;

        if is_new {
            new_items += 1;
        } else {
            matched_existing += 1;
            if cost_diff > 0.001 {
                price_increases += 1;
                total_pct_change += cost_diff_pct;
                pct_change_count += 1;
            } else if cost_diff < -0.001 {
                price_decreases += 1;
                total_pct_change += cost_diff_pct;
                pct_change_count += 1;
            } else {
                price_unchanged += 1;
            }
        }

        variances.push(PriceVariance {
            sku,
            barcode,
            name,
            old_cost,
            new_cost,
            cost_diff,
            cost_diff_pct,
            old_retail,
            new_retail,
            unit,
            category,
            is_new_item: is_new,
        });
    }

    let avg_cost_change_pct = if pct_change_count > 0 {
        (total_pct_change / pct_change_count as f64 * 10.0).round() / 10.0
    } else {
        0.0
    };

    Ok(ReconciliationReport {
        supplier_name: supplier_name.to_string(),
        total_rows: rows.len(),
        matched_existing,
        new_items,
        price_increases,
        price_decreases,
        price_unchanged,
        avg_cost_change_pct,
        items: variances,
    })
}

/// Applies reconciled price variances to live SQLite store inventory.
pub fn apply_price_reconciliation(
    conn: &Connection,
    report: &ReconciliationReport,
) -> Result<usize, String> {
    init_supplier_catalog_schema(conn).map_err(|e| format!("Schema error: {}", e))?;
    let now = Utc::now().to_rfc3339();

    let tx = conn.unchecked_transaction().map_err(|e| format!("Transaction error: {}", e))?;

    let mut applied_count = 0;
    for item in &report.items {
        tx.execute(
            "INSERT INTO store_items (sku, barcode, name, cost_price, retail_price, unit, category, stock_qty, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0.0, ?8)
             ON CONFLICT(sku) DO UPDATE SET
                barcode = COALESCE(excluded.barcode, store_items.barcode),
                name = excluded.name,
                cost_price = excluded.cost_price,
                retail_price = excluded.retail_price,
                unit = excluded.unit,
                category = excluded.category,
                updated_at = excluded.updated_at",
            params![
                item.sku,
                item.barcode,
                item.name,
                item.new_cost,
                item.new_retail,
                item.unit,
                item.category,
                now,
            ],
        ).map_err(|e| format!("Failed to update item {}: {}", item.sku, e))?;
        applied_count += 1;
    }

    let history_id = Uuid::new_v4().to_string();
    tx.execute(
        "INSERT INTO supplier_catalog_history (id, supplier_name, imported_at, total_items, increases, decreases, avg_change_pct)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            history_id,
            report.supplier_name,
            now,
            report.total_rows as i64,
            report.price_increases as i64,
            report.price_decreases as i64,
            report.avg_cost_change_pct,
        ],
    ).map_err(|e| format!("Failed to record catalog history: {}", e))?;

    tx.commit().map_err(|e| format!("Commit error: {}", e))?;
    Ok(applied_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_parsing_and_decimal_cleaning() {
        let raw = "ΚΩΔΙΚΟΣ;ΠΕΡΙΓΡΑΦΗ;ΤΙΜΗ;BARCODE;ΜΟΝΑΔΑ\n\
                   BOS-100;Δράπανο Κρουστικό;85,50 €;5201234567890;ΤΕΜ\n\
                   WUR-200;Σπρέι Σιλικόνης;4,20;4012345678901;ΤΕΜ\n";

        let (headers, rows) = parse_delimited_catalog(raw);
        assert_eq!(headers.len(), 5);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0][0], "BOS-100");
        assert_eq!(parse_euro_float(&rows[0][2]), 85.50);
        assert_eq!(parse_euro_float(&rows[1][2]), 4.20);
    }

    #[test]
    fn test_reconciliation_workflow_live_sqlite() {
        let conn = Connection::open_in_memory().unwrap();
        init_supplier_catalog_schema(&conn).unwrap();

        // Seed 1 existing item in store inventory
        let initial_item = StoreItem {
            sku: "BOS-100".into(),
            barcode: Some("5201234567890".into()),
            name: "Δράπανο Κρουστικό 18V".into(),
            cost_price: 80.00,
            retail_price: 110.00,
            unit: "ΤΕΜ".into(),
            category: "ΕΡΓΑΛΕΙΑ".into(),
            stock_qty: 5.0,
            updated_at: Utc::now().to_rfc3339(),
        };
        create_or_update_item(&conn, &initial_item).unwrap();

        // Incoming vendor CSV with price increase (+6.875%) and 1 new product
        let incoming_csv = "ΚΩΔΙΚΟΣ;ΠΕΡΙΓΡΑΦΗ;ΤΙΜΗ;BARCODE;ΜΟΝΑΔΑ\n\
                            BOS-100;Δράπανο Κρουστικό 18V;85,50;5201234567890;ΤΕΜ\n\
                            NEW-999;Νέο Κατσαβίδι Torx;3,50;5209999999999;ΤΕΜ\n";

        let report = reconcile_catalog(&conn, "BOSCH Hellas", incoming_csv, 35.0).unwrap();

        assert_eq!(report.total_rows, 2);
        assert_eq!(report.matched_existing, 1);
        assert_eq!(report.new_items, 1);
        assert_eq!(report.price_increases, 1);

        let bos = report.items.iter().find(|i| i.sku == "BOS-100").unwrap();
        assert_eq!(bos.old_cost, 80.00);
        assert_eq!(bos.new_cost, 85.50);
        assert_eq!(bos.cost_diff, 5.50);
        assert!(!bos.is_new_item);
        // With 35% markup on 85.50: 85.50 * 1.35 = 115.425 -> 115.43
        assert_eq!(bos.new_retail, 115.43);

        let new_prod = report.items.iter().find(|i| i.sku == "NEW-999").unwrap();
        assert!(new_prod.is_new_item);
        assert_eq!(new_prod.new_cost, 3.50);
        // With 35% markup on 3.50: 3.50 * 1.35 = 4.725 -> 4.73
        assert_eq!(new_prod.new_retail, 4.73);

        // Apply changes to SQLite
        let applied = apply_price_reconciliation(&conn, &report).unwrap();
        assert_eq!(applied, 2);

        // Verify SQLite table now contains both updated items
        let items = list_store_items(&conn).unwrap();
        assert_eq!(items.len(), 2);

        let updated_bos = items.iter().find(|i| i.sku == "BOS-100").unwrap();
        assert_eq!(updated_bos.cost_price, 85.50);
        assert_eq!(updated_bos.retail_price, 115.43);
    }
}
