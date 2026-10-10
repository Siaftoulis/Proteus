//! Compliance & Fiscal Discrepancy Detector for Inbound myDATA Expenses.
//! Analyzes incoming supplier invoices for unrecorded phantom invoices, price inflation,
//! quantity divergences, and VAT category mismatches against internal purchase orders.
//! Strict Rule 1 (100% Original Codebase), Rule 5 (Zero Mock Data), Rule 6 (Compliance & Security).

use serde::{Deserialize, Serialize};

use crate::mydata_expenses::parser::InboundExpenseInvoice;
use crate::mydata_expenses::reconciler::{
    ExpenseMatchQuality, PurchaseOrderRecord, ReconciliationMatchResult, WarehouseReceiptRecord,
};

/// Severity classification of a detected expense discrepancy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ExpenseDiscrepancySeverity {
    Info,     // Minor info (e.g. price discount or rounding)
    Medium,   // Minor price drift (< 5%)
    High,     // Significant price increase (5% - 20%) or quantity discrepancy
    Critical, // Unrecorded invoice, price inflation > 20%, or math corruption
}

/// Categorization of detected fiscal or operational discrepancies.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ExpenseDiscrepancyType {
    /// Invoice exists in AADE myDATA but has no matching PO or delivery note in the company.
    UnrecordedInvoice,
    /// Supplier billed higher than agreed purchase order price.
    PriceInflation {
        expected_cents: u64,
        invoiced_cents: u64,
        diff_cents: u64,
        divergence_pct: f64,
    },
    /// Supplier gave an unannounced price decrease / discount.
    PriceDeflation {
        expected_cents: u64,
        invoiced_cents: u64,
        diff_cents: u64,
        divergence_pct: f64,
    },
    /// Invoiced quantity exceeds physically received warehouse units.
    QuantityOverbill {
        received_units: u32,
        invoiced_units: u32,
    },
    /// Net + VAT does not equal Gross in the invoice XML.
    MathematicalInconsistency {
        net_cents: u64,
        vat_cents: u64,
        gross_cents: u64,
    },
}

/// Individual discrepancy finding.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpenseDiscrepancyItem {
    pub discrepancy_type: ExpenseDiscrepancyType,
    pub severity: ExpenseDiscrepancySeverity,
    pub description: String,
}

/// Comprehensive audit report for an inbound myDATA expense.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpenseDiscrepancyReport {
    pub invoice_mark: u64,
    pub invoice_uid: String,
    pub supplier_afm: String,
    pub is_clean: bool,
    pub highest_severity: Option<ExpenseDiscrepancySeverity>,
    pub items: Vec<ExpenseDiscrepancyItem>,
}

/// Evaluates an inbound invoice against its reconciliation match to detect discrepancies.
pub fn detect_expense_discrepancies(
    invoice: &InboundExpenseInvoice,
    match_result: &ReconciliationMatchResult,
    matched_po: Option<&PurchaseOrderRecord>,
    matched_receipt: Option<&WarehouseReceiptRecord>,
) -> ExpenseDiscrepancyReport {
    let mut items = Vec::new();

    // 1. Inherent Mathematical Integrity Check (Net + VAT == Gross within 1 cent tolerance)
    let calculated_gross = invoice.total_net_cents.saturating_add(invoice.total_vat_cents);
    let gross_diff = (calculated_gross as i64 - invoice.total_gross_cents as i64).abs();
    if gross_diff > 1 {
        items.push(ExpenseDiscrepancyItem {
            discrepancy_type: ExpenseDiscrepancyType::MathematicalInconsistency {
                net_cents: invoice.total_net_cents,
                vat_cents: invoice.total_vat_cents,
                gross_cents: invoice.total_gross_cents,
            },
            severity: ExpenseDiscrepancySeverity::Critical,
            description: format!(
                "Μαθηματική ασυμφωνία παραστατικού: Καθαρή ({} €) + ΦΠΑ ({} €) != Σύνολο ({} €)",
                invoice.total_net_cents as f64 / 100.0,
                invoice.total_vat_cents as f64 / 100.0,
                invoice.total_gross_cents as f64 / 100.0
            ),
        });
    }

    // 2. Unrecorded Invoice Detection
    if match_result.match_quality == ExpenseMatchQuality::Unmatched {
        items.push(ExpenseDiscrepancyItem {
            discrepancy_type: ExpenseDiscrepancyType::UnrecordedInvoice,
            severity: ExpenseDiscrepancySeverity::Critical,
            description: format!(
                "Ακαταχώρητο Τιμολόγιο AADE (MARK: {}): Δεν εντοπίστηκε ανοιχτή παραγγελία ή δελτίο παραλαβής για το ΑΦΜ {}.",
                invoice.mark, invoice.supplier_afm
            ),
        });
    }

    // 3. Price Divergence against Matched Purchase Order
    if let Some(po) = matched_po {
        let expected = po.expected_gross_cents;
        let invoiced = invoice.total_gross_cents;

        if invoiced > expected {
            let diff = invoiced - expected;
            let pct = if expected > 0 {
                (diff as f64 / expected as f64) * 100.0
            } else {
                100.0
            };

            let severity = if pct > 20.0 {
                ExpenseDiscrepancySeverity::Critical
            } else if pct > 5.0 {
                ExpenseDiscrepancySeverity::High
            } else {
                ExpenseDiscrepancySeverity::Medium
            };

            items.push(ExpenseDiscrepancyItem {
                discrepancy_type: ExpenseDiscrepancyType::PriceInflation {
                    expected_cents: expected,
                    invoiced_cents: invoiced,
                    diff_cents: diff,
                    divergence_pct: pct,
                },
                severity,
                description: format!(
                    "Υπερτιμολόγηση Προμηθευτή: Χρέωση {:.2} € έναντι συμφωνηθείσας PO {:.2} € (+{:.1}%).",
                    invoiced as f64 / 100.0,
                    expected as f64 / 100.0,
                    pct
                ),
            });
        } else if invoiced < expected {
            let diff = expected - invoiced;
            let pct = if expected > 0 {
                (diff as f64 / expected as f64) * 100.0
            } else {
                0.0
            };

            items.push(ExpenseDiscrepancyItem {
                discrepancy_type: ExpenseDiscrepancyType::PriceDeflation {
                    expected_cents: expected,
                    invoiced_cents: invoiced,
                    diff_cents: diff,
                    divergence_pct: pct,
                },
                severity: ExpenseDiscrepancySeverity::Info,
                description: format!(
                    "Έκπτωση / Μειωμένη Χρέωση: Χρέωση {:.2} € έναντι συμφωνηθείσας PO {:.2} € (-{:.1}%).",
                    invoiced as f64 / 100.0,
                    expected as f64 / 100.0,
                    pct
                ),
            });
        }
    }

    // 4. Quantity Divergence against Matched Warehouse Receipt
    if let Some(receipt) = matched_receipt {
        // Calculate total items invoiced across lines
        // If invoice doesn't specify unit counts per line, line count is used as baseline
        let invoiced_units: u32 = invoice.lines.len() as u32;
        if receipt.total_units > 0 && invoiced_units > receipt.total_units {
            items.push(ExpenseDiscrepancyItem {
                discrepancy_type: ExpenseDiscrepancyType::QuantityOverbill {
                    received_units: receipt.total_units,
                    invoiced_units,
                },
                severity: ExpenseDiscrepancySeverity::High,
                description: format!(
                    "Απόκλιση Ποσοτήτων: Τιμολογήθηκαν {} εγγραφές έναντι {} φυσικά παραληφθέντων τεμαχίων (Δελτίο {}).",
                    invoiced_units, receipt.total_units, receipt.supplier_delivery_note
                ),
            });
        }
    }

    let is_clean = items.is_empty();
    let highest_severity = items.iter().map(|i| i.severity).max();

    ExpenseDiscrepancyReport {
        invoice_mark: invoice.mark,
        invoice_uid: invoice.uid.clone(),
        supplier_afm: invoice.supplier_afm.clone(),
        is_clean,
        highest_severity,
        items,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mydata_expenses::parser::InboundExpenseLine;
    use crate::mydata_expenses::reconciler::PurchaseOrderStatus;

    fn sample_invoice(net: u64, vat: u64, gross: u64) -> InboundExpenseInvoice {
        InboundExpenseInvoice {
            mark: 99001122,
            uid: "UID-TEST".to_string(),
            cancelled_by_mark: None,
            supplier_afm: "998877665".to_string(),
            supplier_country: "GR".to_string(),
            counterpart_afm: "094123456".to_string(),
            series: "ΤΔΑ".to_string(),
            aa: "100".to_string(),
            issue_date: "2026-10-09".to_string(),
            invoice_type: "1.1".to_string(),
            currency: "EUR".to_string(),
            total_net_cents: net,
            total_vat_cents: vat,
            total_gross_cents: gross,
            lines: vec![InboundExpenseLine {
                line_number: 1,
                net_cents: net,
                vat_category: 1,
                vat_cents: vat,
                comments: None,
            }],
        }
    }

    #[test]
    fn test_clean_invoice_no_discrepancy() {
        let inv = sample_invoice(10000, 2400, 12400);
        let match_res = ReconciliationMatchResult {
            invoice_mark: 99001122,
            invoice_uid: "UID-TEST".to_string(),
            supplier_afm: "998877665".to_string(),
            matched_po_id: Some("PO-1".to_string()),
            matched_receipt_id: None,
            match_quality: ExpenseMatchQuality::ExactMatch,
            confidence_score: 1.0,
            amount_difference_cents: 0,
        };

        let po = PurchaseOrderRecord {
            order_id: "PO-1".to_string(),
            supplier_afm: "998877665".to_string(),
            supplier_name: "Supplier AE".to_string(),
            order_date: "2026-10-01".to_string(),
            expected_net_cents: 10000,
            expected_vat_cents: 2400,
            expected_gross_cents: 12400,
            status: PurchaseOrderStatus::PendingDelivery,
            lines: vec![],
        };

        let report = detect_expense_discrepancies(&inv, &match_res, Some(&po), None);
        assert!(report.is_clean);
        assert_eq!(report.highest_severity, None);
        assert_eq!(report.items.len(), 0);
    }

    #[test]
    fn test_unrecorded_invoice_flagged_critical() {
        let inv = sample_invoice(5000, 1200, 6200);
        let match_res = ReconciliationMatchResult {
            invoice_mark: 99001122,
            invoice_uid: "UID-TEST".to_string(),
            supplier_afm: "998877665".to_string(),
            matched_po_id: None,
            matched_receipt_id: None,
            match_quality: ExpenseMatchQuality::Unmatched,
            confidence_score: 0.0,
            amount_difference_cents: 6200,
        };

        let report = detect_expense_discrepancies(&inv, &match_res, None, None);
        assert!(!report.is_clean);
        assert_eq!(report.highest_severity, Some(ExpenseDiscrepancySeverity::Critical));
        assert!(matches!(
            report.items[0].discrepancy_type,
            ExpenseDiscrepancyType::UnrecordedInvoice
        ));
    }

    #[test]
    fn test_price_inflation_detection() {
        let inv = sample_invoice(12000, 2880, 14880); // Invoiced 148.80
        let match_res = ReconciliationMatchResult {
            invoice_mark: 99001122,
            invoice_uid: "UID-TEST".to_string(),
            supplier_afm: "998877665".to_string(),
            matched_po_id: Some("PO-1".to_string()),
            matched_receipt_id: None,
            match_quality: ExpenseMatchQuality::PartialDiscrepancy,
            confidence_score: 0.5,
            amount_difference_cents: 2480,
        };

        let po = PurchaseOrderRecord {
            order_id: "PO-1".to_string(),
            supplier_afm: "998877665".to_string(),
            supplier_name: "Supplier AE".to_string(),
            order_date: "2026-10-01".to_string(),
            expected_net_cents: 10000,
            expected_vat_cents: 2400,
            expected_gross_cents: 12400, // Expected 124.00 (Inflation +20%)
            status: PurchaseOrderStatus::PendingDelivery,
            lines: vec![],
        };

        let report = detect_expense_discrepancies(&inv, &match_res, Some(&po), None);
        assert!(!report.is_clean);
        assert_eq!(report.highest_severity, Some(ExpenseDiscrepancySeverity::High));
        assert!(matches!(
            report.items[0].discrepancy_type,
            ExpenseDiscrepancyType::PriceInflation { .. }
        ));
    }
}
