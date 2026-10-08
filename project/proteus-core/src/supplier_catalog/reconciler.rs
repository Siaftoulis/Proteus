//! CSV/TSV catalog parsing, SMLM column inference, and price reconciliation logic.
//! Ingests messy vendor catalogs, detects price changes, and applies updates to SQLite.

use chrono::Utc;
use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::smlm::{SemanticIntent, SmlmEngine};
use super::db::*;
use super::types::*;

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
pub fn parse_euro_float(val: &str) -> f64 {
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
