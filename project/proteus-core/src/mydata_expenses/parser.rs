//! Resilient XML Parser for AADE myDATA `RequestTransmittedDocs` (RequestedDoc).
//! Extracts supplier AFM, invoice type, line items, net, VAT, gross, and MARK
//! with zero external XML parser dependencies and zero floating-point drift.
//! Strict Rule 1 (100% Original Codebase), Rule 5 (Zero Mock Data), Rule 6 (Compliance & Security).

use serde::{Deserialize, Serialize};

/// Detailed line item within an inbound expense invoice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InboundExpenseLine {
    pub line_number: u32,
    pub net_cents: u64,
    pub vat_category: u8,
    pub vat_cents: u64,
    pub comments: Option<String>,
}

/// Fully parsed inbound supplier invoice from AADE myDATA repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InboundExpenseInvoice {
    pub mark: u64,
    pub uid: String,
    pub cancelled_by_mark: Option<u64>,
    pub supplier_afm: String,
    pub supplier_country: String,
    pub counterpart_afm: String,
    pub series: String,
    pub aa: String,
    pub issue_date: String,
    pub invoice_type: String,
    pub currency: String,
    pub total_net_cents: u64,
    pub total_vat_cents: u64,
    pub total_gross_cents: u64,
    pub lines: Vec<InboundExpenseLine>,
}

/// Structured response container for AADE `RequestedDoc`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RequestedDocResponse {
    pub next_partition_key: Option<String>,
    pub next_row_key: Option<String>,
    pub invoices: Vec<InboundExpenseInvoice>,
}

/// Extracts the inner content between `<tag>` and `</tag>`, ignoring XML namespace prefixes.
pub fn extract_tag_value(xml_block: &str, tag_name: &str) -> Option<String> {
    let mut search_from = 0;
    while let Some(open_rel) = xml_block[search_from..].find('<') {
        let open_idx = search_from + open_rel;
        if let Some(close_bracket) = xml_block[open_idx..].find('>') {
            let full_open_tag = &xml_block[open_idx + 1..open_idx + close_bracket];
            // Check if tag name matches either directly or with prefix (e.g. "inv:invoiceType" or "invoiceType")
            let matches_open = full_open_tag == tag_name
                || full_open_tag.ends_with(&format!(":{}", tag_name))
                || full_open_tag.starts_with(&format!("{} ", tag_name))
                || (full_open_tag.ends_with(&format!(":{}", tag_name)) && full_open_tag.contains(' '));

            if matches_open {
                let content_start = open_idx + close_bracket + 1;
                // Find closing tag
                let search_close = &xml_block[content_start..];
                if let Some(close_rel) = search_close.find(&format!("</{}", tag_name)) {
                    let content_end = content_start + close_rel;
                    return Some(xml_block[content_start..content_end].trim().to_string());
                } else {
                    // Try looking for prefixed closing tag </...:tag_name>
                    let mut close_search = 0;
                    while let Some(c_open) = search_close[close_search..].find("</") {
                        let c_idx = close_search + c_open;
                        if let Some(c_gt) = search_close[c_idx..].find('>') {
                            let close_tag_name = &search_close[c_idx + 2..c_idx + c_gt];
                            if close_tag_name == tag_name || close_tag_name.ends_with(&format!(":{}", tag_name)) {
                                let content_end = content_start + c_idx;
                                return Some(xml_block[content_start..content_end].trim().to_string());
                            }
                            close_search = c_idx + c_gt + 1;
                        } else {
                            break;
                        }
                    }
                }
            }
            search_from = open_idx + close_bracket + 1;
        } else {
            break;
        }
    }
    None
}

/// Converts decimal string (e.g. "124.50" or "124,50" or "100") into exact integer cents (u64).
pub fn parse_decimal_to_cents(s: &str) -> u64 {
    let cleaned = s.trim().replace(',', ".");
    if cleaned.is_empty() {
        return 0;
    }
    let parts: Vec<&str> = cleaned.split('.').collect();
    let integer_part: u64 = parts[0].parse().unwrap_or(0);
    let cents_part: u64 = if parts.len() > 1 {
        let dec = parts[1];
        if dec.len() == 1 {
            dec.parse::<u64>().unwrap_or(0) * 10
        } else if dec.len() >= 2 {
            dec[..2].parse::<u64>().unwrap_or(0)
        } else {
            0
        }
    } else {
        0
    };
    (integer_part * 100).saturating_add(cents_part)
}

/// Collects all XML blocks between `<open_tag>` and `</open_tag>` (ignoring namespaces).
fn extract_all_blocks<'a>(xml: &'a str, tag_name: &str) -> Vec<&'a str> {
    let mut blocks = Vec::new();
    let mut search_from = 0;

    while let Some(open_rel) = xml[search_from..].find('<') {
        let open_idx = search_from + open_rel;
        if let Some(gt) = xml[open_idx..].find('>') {
            let full_open = &xml[open_idx + 1..open_idx + gt];
            let is_match = full_open == tag_name
                || full_open.ends_with(&format!(":{}", tag_name))
                || full_open.starts_with(&format!("{} ", tag_name));

            if is_match {
                let content_start = open_idx + gt + 1;
                // Find corresponding closing tag
                let mut close_pos = None;
                let mut scan = content_start;
                while let Some(c_rel) = xml[scan..].find("</") {
                    let c_idx = scan + c_rel;
                    if let Some(c_gt) = xml[c_idx..].find('>') {
                        let c_tag = &xml[c_idx + 2..c_idx + c_gt];
                        if c_tag == tag_name || c_tag.ends_with(&format!(":{}", tag_name)) {
                            close_pos = Some((content_start, c_idx, c_idx + c_gt + 1));
                            break;
                        }
                        scan = c_idx + c_gt + 1;
                    } else {
                        break;
                    }
                }

                if let Some((start, end, next_scan)) = close_pos {
                    blocks.push(&xml[start..end]);
                    search_from = next_scan;
                    continue;
                }
            }
            search_from = open_idx + gt + 1;
        } else {
            break;
        }
    }

    blocks
}

/// Parses an entire AADE `RequestedDoc` XML response containing one or more inbound invoices.
pub fn parse_requested_doc_xml(xml: &str) -> RequestedDocResponse {
    let mut response = RequestedDocResponse::default();

    // Check continuation tokens for pagination
    if let Some(token_block) = extract_tag_value(xml, "continuationToken") {
        response.next_partition_key = extract_tag_value(&token_block, "nextPartitionKey");
        response.next_row_key = extract_tag_value(&token_block, "nextRowKey");
    }

    let invoice_blocks = extract_all_blocks(xml, "invoice");
    for inv_block in invoice_blocks {
        let mark_str = extract_tag_value(inv_block, "mark").unwrap_or_default();
        let mark: u64 = mark_str.parse().unwrap_or(0);
        let uid = extract_tag_value(inv_block, "uid").unwrap_or_default();
        let cancelled_by_mark = extract_tag_value(inv_block, "cancelledByMark")
            .and_then(|s| s.parse::<u64>().ok());

        // Header
        let header_block = extract_tag_value(inv_block, "invoiceHeader").unwrap_or_default();
        let series = extract_tag_value(&header_block, "series").unwrap_or_default();
        let aa = extract_tag_value(&header_block, "aa").unwrap_or_default();
        let issue_date = extract_tag_value(&header_block, "issueDate").unwrap_or_default();
        let invoice_type = extract_tag_value(&header_block, "invoiceType").unwrap_or_default();
        let currency = extract_tag_value(&header_block, "currency").unwrap_or_else(|| "EUR".to_string());

        // Issuer (Supplier)
        let issuer_block = extract_tag_value(inv_block, "issuer").unwrap_or_default();
        let supplier_afm = extract_tag_value(&issuer_block, "vatNumber").unwrap_or_default();
        let supplier_country = extract_tag_value(&issuer_block, "country").unwrap_or_else(|| "GR".to_string());

        // Counterpart (Recipient)
        let counterpart_block = extract_tag_value(inv_block, "counterpart").unwrap_or_default();
        let counterpart_afm = extract_tag_value(&counterpart_block, "vatNumber").unwrap_or_default();

        // Invoice Summary
        let summary_block = extract_tag_value(inv_block, "invoiceSummary").unwrap_or_default();
        let total_net_str = extract_tag_value(&summary_block, "totalNetValue").unwrap_or_default();
        let total_vat_str = extract_tag_value(&summary_block, "totalVatAmount").unwrap_or_default();
        let total_gross_str = extract_tag_value(&summary_block, "totalGrossValue").unwrap_or_default();

        let total_net_cents = parse_decimal_to_cents(&total_net_str);
        let total_vat_cents = parse_decimal_to_cents(&total_vat_str);
        let total_gross_cents = parse_decimal_to_cents(&total_gross_str);

        // Details lines
        let detail_blocks = extract_all_blocks(inv_block, "invoiceDetails");
        let mut lines = Vec::new();
        for det_block in detail_blocks {
            let line_num: u32 = extract_tag_value(det_block, "lineNumber")
                .and_then(|s| s.parse().ok())
                .unwrap_or((lines.len() + 1) as u32);
            let net_val = extract_tag_value(det_block, "netValue").unwrap_or_default();
            let vat_cat: u8 = extract_tag_value(det_block, "vatCategory")
                .and_then(|s| s.parse().ok())
                .unwrap_or(1);
            let vat_amt = extract_tag_value(det_block, "vatAmount").unwrap_or_default();
            let comments = extract_tag_value(det_block, "lineComments");

            lines.push(InboundExpenseLine {
                line_number: line_num,
                net_cents: parse_decimal_to_cents(&net_val),
                vat_category: vat_cat,
                vat_cents: parse_decimal_to_cents(&vat_amt),
                comments,
            });
        }

        response.invoices.push(InboundExpenseInvoice {
            mark,
            uid,
            cancelled_by_mark,
            supplier_afm,
            supplier_country,
            counterpart_afm,
            series,
            aa,
            issue_date,
            invoice_type,
            currency,
            total_net_cents,
            total_vat_cents,
            total_gross_cents,
            lines,
        });
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_decimal_to_cents() {
        assert_eq!(parse_decimal_to_cents("100.00"), 10000);
        assert_eq!(parse_decimal_to_cents("49.90"), 4990);
        assert_eq!(parse_decimal_to_cents("0.5"), 50);
        assert_eq!(parse_decimal_to_cents("1234,56"), 123456);
        assert_eq!(parse_decimal_to_cents("10"), 1000);
    }

    #[test]
    fn test_parse_requested_doc_xml_single_invoice() {
        let xml = r#"
        <RequestedDoc xmlns="http://www.aade.gr/myDATA/invoice/v1.0">
            <continuationToken>
                <nextPartitionKey>PK_2026_10</nextPartitionKey>
                <nextRowKey>RK_999</nextRowKey>
            </continuationToken>
            <invoicesDoc>
                <invoice>
                    <uid>AADE-UID-991122</uid>
                    <mark>8800112233</mark>
                    <invoiceHeader>
                        <series>ΤΔΑ</series>
                        <aa>452</aa>
                        <issueDate>2026-10-08</issueDate>
                        <invoiceType>1.1</invoiceType>
                        <currency>EUR</currency>
                    </invoiceHeader>
                    <issuer>
                        <vatNumber>998877665</vatNumber>
                        <country>GR</country>
                    </issuer>
                    <counterpart>
                        <vatNumber>094123456</vatNumber>
                    </counterpart>
                    <invoiceDetails>
                        <lineNumber>1</lineNumber>
                        <netValue>250.00</netValue>
                        <vatCategory>1</vatCategory>
                        <vatAmount>60.00</vatAmount>
                        <lineComments>Ανταλλακτικά Bosch</lineComments>
                    </invoiceDetails>
                    <invoiceSummary>
                        <totalNetValue>250.00</totalNetValue>
                        <totalVatAmount>60.00</totalVatAmount>
                        <totalGrossValue>310.00</totalGrossValue>
                    </invoiceSummary>
                </invoice>
            </invoicesDoc>
        </RequestedDoc>
        "#;

        let resp = parse_requested_doc_xml(xml);
        assert_eq!(resp.next_partition_key, Some("PK_2026_10".to_string()));
        assert_eq!(resp.next_row_key, Some("RK_999".to_string()));
        assert_eq!(resp.invoices.len(), 1);

        let inv = &resp.invoices[0];
        assert_eq!(inv.mark, 8800112233);
        assert_eq!(inv.uid, "AADE-UID-991122");
        assert_eq!(inv.supplier_afm, "998877665");
        assert_eq!(inv.counterpart_afm, "094123456");
        assert_eq!(inv.invoice_type, "1.1");
        assert_eq!(inv.series, "ΤΔΑ");
        assert_eq!(inv.aa, "452");
        assert_eq!(inv.issue_date, "2026-10-08");
        assert_eq!(inv.total_net_cents, 25000);
        assert_eq!(inv.total_vat_cents, 6000);
        assert_eq!(inv.total_gross_cents, 31000);

        assert_eq!(inv.lines.len(), 1);
        assert_eq!(inv.lines[0].net_cents, 25000);
        assert_eq!(inv.lines[0].vat_cents, 6000);
        assert_eq!(inv.lines[0].vat_category, 1);
        assert_eq!(inv.lines[0].comments.as_deref(), Some("Ανταλλακτικά Bosch"));
    }
}
