//! 1-Click CPA Accounting Synchronization & Reconciliation Engine for Proteus BOS.
//! Implements 100% original, native Greek statutory accounting exports (Law 4308/2014 ΕΛΠ):
//! - Multi-rate VAT allocation buckets (24%, 13%, 6%, 0% Art. 22/39 exemptions).
//! - Automatic discrepancy detection (unfiled invoices, missing MARKs, outbox failures).
//! - CPA 1-Click JSON export for Greek chartered accountants.
//! - Greek European-format CSV export (semicolon delimiter, comma decimals).
//! Strict Rule 1 (100% Original Codebase) and Rule 5 (Zero Mock Data).

use crate::invoicing::VatCategory;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

/// Summarized VAT bucket for accounting ledger reconciliation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VatBucketSummary {
    pub vat_category: VatCategory,
    pub rate_percent: f64,
    pub net_eur: f64,
    pub vat_eur: f64,
    pub gross_eur: f64,
    pub line_count: usize,
}

/// Comprehensive fiscal and accounting period summary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PeriodFiscalReport {
    pub period_start: String,
    pub period_end: String,
    pub total_invoices_count: usize,
    pub total_net_eur: f64,
    pub total_vat_eur: f64,
    pub total_gross_eur: f64,
    pub vat_buckets: Vec<VatBucketSummary>,
    pub pending_outbox_count: usize,
    pub missing_mark_count: usize,
}

/// Specific audit discrepancy detected in fiscal records.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CpaDiscrepancy {
    pub invoice_id: String,
    pub issue_date: String,
    pub series: String,
    pub invoice_number: i64,
    pub recipient_afm: String,
    pub gross_eur: f64,
    pub discrepancy_reason: String,
}

/// Generates a complete fiscal and VAT bucket report for a given date range (YYYY-MM-DD).
pub fn generate_period_fiscal_report(
    conn: &Connection,
    start_date: &str,
    end_date: &str,
) -> Result<PeriodFiscalReport> {
    // 1. Calculate overall invoice counts and totals
    let mut inv_stmt = conn.prepare(
        r#"
        SELECT COUNT(*),
               COALESCE(SUM(total_net_eur), 0.0),
               COALESCE(SUM(total_vat_eur), 0.0),
               COALESCE(SUM(total_gross_eur), 0.0),
               COALESCE(SUM(CASE WHEN mydata_mark IS NULL OR mydata_mark = '' THEN 1 ELSE 0 END), 0)
        FROM invoices
        WHERE issue_date >= ?1 AND issue_date <= ?2 AND is_cancelled = 0
        "#,
    )?;

    let (total_invoices_count, raw_net, raw_vat, raw_gross, missing_mark_count): (usize, f64, f64, f64, usize) =
        inv_stmt.query_row(params![start_date, end_date], |r| {
            let cnt: i64 = r.get(0)?;
            let net: f64 = r.get(1)?;
            let vat: f64 = r.get(2)?;
            let gross: f64 = r.get(3)?;
            let missing: i64 = r.get(4)?;
            Ok((cnt as usize, net, vat, gross, missing as usize))
        })?;

    // 2. Aggregate line-item VAT buckets
    let mut line_stmt = conn.prepare(
        r#"
        SELECT l.vat_category,
               COUNT(*),
               COALESCE(SUM(l.net_total), 0.0),
               COALESCE(SUM(l.vat_amount), 0.0),
               COALESCE(SUM(l.gross_total), 0.0)
        FROM invoice_lines l
        JOIN invoices i ON l.invoice_id = i.invoice_id
        WHERE i.issue_date >= ?1 AND i.issue_date <= ?2 AND i.is_cancelled = 0
        GROUP BY l.vat_category
        ORDER BY l.vat_category ASC
        "#,
    )?;

    let bucket_rows = line_stmt.query_map(params![start_date, end_date], |r| {
        let vat_str: String = r.get(0)?;
        let cat = VatCategory::from_code(&vat_str).unwrap_or(VatCategory::Vat24);
        let count: i64 = r.get(1)?;
        let net: f64 = r.get(2)?;
        let vat: f64 = r.get(3)?;
        let gross: f64 = r.get(4)?;
        Ok((cat, count as usize, net, vat, gross))
    })?;

    let mut vat_buckets = Vec::new();
    for row in bucket_rows {
        let (cat, count, net, vat, gross) = row?;
        vat_buckets.push(VatBucketSummary {
            vat_category: cat,
            rate_percent: cat.rate_percent(),
            net_eur: (net * 100.0).round() / 100.0,
            vat_eur: (vat * 100.0).round() / 100.0,
            gross_eur: (gross * 100.0).round() / 100.0,
            line_count: count,
        });
    }

    // 3. Count pending outbox entries in this period
    let pending_outbox_count: i64 = conn.query_row(
        r#"
        SELECT COUNT(*)
        FROM mydata_outbox o
        JOIN invoices i ON o.invoice_id = i.invoice_id
        WHERE i.issue_date >= ?1 AND i.issue_date <= ?2 AND o.status = 'PENDING'
        "#,
        params![start_date, end_date],
        |r| r.get(0),
    )?;

    Ok(PeriodFiscalReport {
        period_start: start_date.to_string(),
        period_end: end_date.to_string(),
        total_invoices_count,
        total_net_eur: (raw_net * 100.0).round() / 100.0,
        total_vat_eur: (raw_vat * 100.0).round() / 100.0,
        total_gross_eur: (raw_gross * 100.0).round() / 100.0,
        vat_buckets,
        pending_outbox_count: pending_outbox_count as usize,
        missing_mark_count,
    })
}

/// Audits fiscal records and returns all detected compliance discrepancies.
pub fn audit_fiscal_discrepancies(
    conn: &Connection,
    start_date: &str,
    end_date: &str,
) -> Result<Vec<CpaDiscrepancy>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT i.invoice_id, i.issue_date, i.series, i.invoice_number, i.recipient_afm,
               i.total_net_eur, i.total_vat_eur, i.total_gross_eur, i.mydata_mark,
               o.status, o.last_error
        FROM invoices i
        LEFT JOIN mydata_outbox o ON i.invoice_id = o.invoice_id
        WHERE i.issue_date >= ?1 AND i.issue_date <= ?2 AND i.is_cancelled = 0
        ORDER BY i.issue_date ASC, i.invoice_number ASC
        "#,
    )?;

    let mut discrepancies = Vec::new();

    let rows = stmt.query_map(params![start_date, end_date], |r| {
        let inv_id: String = r.get(0)?;
        let issue_date: String = r.get(1)?;
        let series: String = r.get(2)?;
        let inv_num: i64 = r.get(3)?;
        let afm: String = r.get(4)?;
        let net: f64 = r.get(5)?;
        let vat: f64 = r.get(6)?;
        let gross: f64 = r.get(7)?;
        let mark: Option<String> = r.get(8)?;
        let outbox_status: Option<String> = r.get(9)?;
        let last_error: Option<String> = r.get(10)?;
        Ok((inv_id, issue_date, series, inv_num, afm, net, vat, gross, mark, outbox_status, last_error))
    })?;

    for row in rows {
        let (inv_id, issue_date, series, inv_num, afm, net, vat, gross, mark, outbox_status, last_error) = row?;

        // Check for missing myDATA MARK
        if mark.as_deref().unwrap_or("").trim().is_empty() {
            discrepancies.push(CpaDiscrepancy {
                invoice_id: inv_id.clone(),
                issue_date: issue_date.clone(),
                series: series.clone(),
                invoice_number: inv_num,
                recipient_afm: afm.clone(),
                gross_eur: gross,
                discrepancy_reason: "Ανεπιτυχής διαβίβαση myDATA (Εκκρεμεί MARK)".to_string(),
            });
        }

        // Check for outbox transmission errors
        if let Some(status) = outbox_status {
            if status == "FAILED" {
                discrepancies.push(CpaDiscrepancy {
                    invoice_id: inv_id.clone(),
                    issue_date: issue_date.clone(),
                    series: series.clone(),
                    invoice_number: inv_num,
                    recipient_afm: afm.clone(),
                    gross_eur: gross,
                    discrepancy_reason: format!(
                        "Σφάλμα αποστολής myDATA: {}",
                        last_error.unwrap_or_else(|| "Unknown error".to_string())
                    ),
                });
            }
        }

        // Mathematical integrity check (Net + VAT == Gross)
        let calc_gross = ((net + vat) * 100.0).round() / 100.0;
        if (calc_gross - gross).abs() > 0.01 {
            discrepancies.push(CpaDiscrepancy {
                invoice_id: inv_id,
                issue_date,
                series,
                invoice_number: inv_num,
                recipient_afm: afm,
                gross_eur: gross,
                discrepancy_reason: format!(
                    "Λογιστική απόκλιση: Καθαρή ({:.2}) + ΦΠΑ ({:.2}) != Σύνολο ({:.2})",
                    net, vat, gross
                ),
            });
        }
    }

    Ok(discrepancies)
}

/// Detailed invoice summary record for CPA software ingestion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CpaInvoiceRecord {
    pub invoice_id: String,
    pub issue_date: String,
    pub series: String,
    pub invoice_number: i64,
    pub invoice_type: String,
    pub recipient_afm: String,
    pub recipient_name: String,
    pub net_eur: f64,
    pub vat_eur: f64,
    pub gross_eur: f64,
    pub mydata_mark: Option<String>,
}

/// Generates a standardized CPA JSON accounting export compliant with Greek Accounting Standards (ΕΛΠ).
pub fn export_cpa_json(
    conn: &Connection,
    report: &PeriodFiscalReport,
    discrepancies: &[CpaDiscrepancy],
) -> Result<String> {
    let mut stmt = conn.prepare(
        r#"
        SELECT invoice_id, issue_date, series, invoice_number, invoice_type,
               recipient_afm, recipient_name, total_net_eur, total_vat_eur, total_gross_eur, mydata_mark
        FROM invoices
        WHERE issue_date >= ?1 AND issue_date <= ?2 AND is_cancelled = 0
        ORDER BY issue_date ASC, invoice_number ASC
        "#,
    )?;

    let invoice_rows = stmt.query_map(params![report.period_start, report.period_end], |r| {
        Ok(CpaInvoiceRecord {
            invoice_id: r.get(0)?,
            issue_date: r.get(1)?,
            series: r.get(2)?,
            invoice_number: r.get(3)?,
            invoice_type: r.get(4)?,
            recipient_afm: r.get(5)?,
            recipient_name: r.get(6)?,
            net_eur: r.get(7)?,
            vat_eur: r.get(8)?,
            gross_eur: r.get(9)?,
            mydata_mark: r.get(10)?,
        })
    })?;

    let mut invoices = Vec::new();
    for inv in invoice_rows {
        invoices.push(inv?);
    }

    #[derive(Serialize)]
    struct CpaExportPackage<'a> {
        standard: &'static str,
        generated_at: i64,
        period_start: &'a str,
        period_end: &'a str,
        total_invoices: usize,
        total_net_eur: f64,
        total_vat_eur: f64,
        total_gross_eur: f64,
        vat_breakdown: &'a [VatBucketSummary],
        audit_discrepancies: &'a [CpaDiscrepancy],
        invoices_registry: Vec<CpaInvoiceRecord>,
    }

    let package = CpaExportPackage {
        standard: "Greek Accounting Standards (Law 4308/2014) - AADE myDATA v1.0",
        generated_at: chrono::Utc::now().timestamp_millis(),
        period_start: &report.period_start,
        period_end: &report.period_end,
        total_invoices: report.total_invoices_count,
        total_net_eur: report.total_net_eur,
        total_vat_eur: report.total_vat_eur,
        total_gross_eur: report.total_gross_eur,
        vat_breakdown: &report.vat_buckets,
        audit_discrepancies: discrepancies,
        invoices_registry: invoices,
    };

    Ok(serde_json::to_string_pretty(&package).unwrap_or_default())
}

/// Generates a standardized European/Greek CSV accounting ledger.
pub fn export_cpa_csv(
    conn: &Connection,
    start_date: &str,
    end_date: &str,
) -> Result<String> {
    let mut stmt = conn.prepare(
        r#"
        SELECT issue_date, series, invoice_number, invoice_type, recipient_afm, recipient_name,
               total_net_eur, total_vat_eur, total_gross_eur, mydata_mark, mydata_uid
        FROM invoices
        WHERE issue_date >= ?1 AND issue_date <= ?2 AND is_cancelled = 0
        ORDER BY issue_date ASC, invoice_number ASC
        "#,
    )?;

    let mut csv = String::from("ΗΜΕΡΟΜΗΝΙΑ;ΣΕΙΡΑ;ΑΡΙΘΜΟΣ;ΤΥΠΟΣ;ΑΦΜ_ΛΗΠΤΗ;ΕΠΩΝΥΜΙΑ;ΚΑΘΑΡΗ_ΑΞΙΑ;ΦΠΑ;ΣΥΝΟΛΟ;MARK;UID\r\n");

    let rows = stmt.query_map(params![start_date, end_date], |r| {
        let date: String = r.get(0)?;
        let series: String = r.get(1)?;
        let num: i64 = r.get(2)?;
        let inv_type: String = r.get(3)?;
        let afm: String = r.get(4)?;
        let name: String = r.get(5)?;
        let net: f64 = r.get(6)?;
        let vat: f64 = r.get(7)?;
        let gross: f64 = r.get(8)?;
        let mark: Option<String> = r.get(9)?;
        let uid: Option<String> = r.get(10)?;
        Ok((date, series, num, inv_type, afm, name, net, vat, gross, mark, uid))
    })?;

    for row in rows {
        let (date, series, num, inv_type, afm, name, net, vat, gross, mark, uid) = row?;
        let clean_name = name.replace(';', " ");
        let net_str = format!("{:.2}", net).replace('.', ",");
        let vat_str = format!("{:.2}", vat).replace('.', ",");
        let gross_str = format!("{:.2}", gross).replace('.', ",");

        csv.push_str(&format!(
            "{};{};{};{};{};{};{};{};{};{};{}\r\n",
            date,
            series,
            num,
            inv_type,
            afm,
            clean_name,
            net_str,
            vat_str,
            gross_str,
            mark.unwrap_or_default(),
            uid.unwrap_or_default()
        ));
    }

    Ok(csv)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::invoicing::{
        calculate_totals, create_invoice, init_invoicing_schema, FiscalInvoice, InvoiceLine,
        InvoiceType, VatCategory,
    };

    #[test]
    fn test_period_fiscal_report_and_vat_buckets() {
        let conn = Connection::open_in_memory().unwrap();
        init_invoicing_schema(&conn).unwrap();

        // Invoice 1: 24% VAT
        let lines1 = vec![InvoiceLine::new(1, "Service Part", 1.0, 100.0, VatCategory::Vat24)];
        let (net1, vat1, gross1) = calculate_totals(&lines1);
        let inv1 = FiscalInvoice {
            invoice_id: "inv-cpa-01".to_string(),
            series: "A".to_string(),
            invoice_number: 1,
            invoice_type: InvoiceType::Sale1_1,
            issue_date: "2026-10-02".to_string(),
            issue_time: "10:00:00".to_string(),
            issuer_afm: "802194512".to_string(),
            recipient_afm: "099887766".to_string(),
            recipient_name: "Alpha Corp".to_string(),
            total_net_eur: net1,
            total_vat_eur: vat1,
            total_gross_eur: gross1,
            mydata_mark: Some("MARK-001".to_string()),
            mydata_uid: Some("UID-001".to_string()),
            mydata_qr_url: None,
            is_cancelled: false,
            created_at: 1710000000,
        };
        create_invoice(&conn, &inv1, &lines1).unwrap();

        // Invoice 2: 13% and 6% VAT (Unfiled / Missing MARK)
        let lines2 = vec![
            InvoiceLine::new(1, "Fresh Produce", 1.0, 50.0, VatCategory::Vat13),
            InvoiceLine::new(2, "Book", 1.0, 20.0, VatCategory::Vat6),
        ];
        let (net2, vat2, gross2) = calculate_totals(&lines2);
        let inv2 = FiscalInvoice {
            invoice_id: "inv-cpa-02".to_string(),
            series: "A".to_string(),
            invoice_number: 2,
            invoice_type: InvoiceType::RetailReceipt11_1,
            issue_date: "2026-10-03".to_string(),
            issue_time: "11:00:00".to_string(),
            issuer_afm: "802194512".to_string(),
            recipient_afm: "000000000".to_string(),
            recipient_name: "Walk-in Retail".to_string(),
            total_net_eur: net2,
            total_vat_eur: vat2,
            total_gross_eur: gross2,
            mydata_mark: None, // Missing MARK!
            mydata_uid: None,
            mydata_qr_url: None,
            is_cancelled: false,
            created_at: 1710001000,
        };
        create_invoice(&conn, &inv2, &lines2).unwrap();

        let report = generate_period_fiscal_report(&conn, "2026-10-01", "2026-10-31").unwrap();
        assert_eq!(report.total_invoices_count, 2);
        assert_eq!(report.total_net_eur, 170.0);
        // VAT: 24.0 + 6.50 + 1.20 = 31.70
        assert_eq!(report.total_vat_eur, 31.70);
        assert_eq!(report.total_gross_eur, 201.70);
        assert_eq!(report.missing_mark_count, 1);

        let discrepancies = audit_fiscal_discrepancies(&conn, "2026-10-01", "2026-10-31").unwrap();
        assert_eq!(discrepancies.len(), 1);
        assert_eq!(discrepancies[0].invoice_id, "inv-cpa-02");

        let json = export_cpa_json(&conn, &report, &discrepancies).unwrap();
        assert!(json.contains("Alpha Corp"));
        assert!(json.contains("Greek Accounting Standards"));

        let csv = export_cpa_csv(&conn, "2026-10-01", "2026-10-31").unwrap();
        assert!(csv.contains("Alpha Corp"));
        assert!(csv.contains("Walk-in Retail"));
        assert!(csv.contains(";100,00;24,00;124,00;MARK-001;UID-001"));
    }
}
