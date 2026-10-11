//! AADE myDATA Inbound Expense Query Generator (RequestTransmittedDocs).
//! Constructs structured query parameters, date range bounds, and AFM filters
//! for automated retrieval of inbound supplier invoices from the official AADE API.
//! Strict Rule 1 (100% Original Codebase), Rule 5 (Zero Mock Data), Rule 6 (Compliance & Security).

use serde::{Deserialize, Serialize};
use crate::mydata_sync::MyDataCredentials;

/// Date format error or invalid AFM error for myDATA expense filters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExpenseQueryError {
    InvalidDateFormat(String),
    InvalidAfm(String),
}

impl std::fmt::Display for ExpenseQueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidDateFormat(msg) => write!(f, "Μη έγκυρη μορφή ημερομηνίας: {}", msg),
            Self::InvalidAfm(afm) => write!(f, "Μη έγκυρο ΑΦΜ προμηθευτή: {}", afm),
        }
    }
}

impl std::error::Error for ExpenseQueryError {}

/// Structured query parameters for AADE `RequestTransmittedDocs` API endpoint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct InboundExpenseFilter {
    /// Starting MARK watermark for incremental pagination / polling.
    pub mark_from: Option<u64>,
    /// Start date in European `dd/MM/yyyy` format.
    pub date_from: Option<String>,
    /// End date in European `dd/MM/yyyy` format.
    pub date_to: Option<String>,
    /// Counterparty Supplier Greek VAT identification number (9 digits).
    pub supplier_afm: Option<String>,
    /// Specific invoice type code filter (e.g. "1.1", "1.2", "2.1", "13.1", "14.1").
    pub invoice_type: Option<String>,
    /// Optional pagination continuation token from previous response.
    pub next_partition_key: Option<String>,
}

impl InboundExpenseFilter {
    /// Creates a new empty inbound expense filter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the starting MARK index for forward incremental synchronization.
    pub fn with_mark(mut self, mark: u64) -> Self {
        self.mark_from = Some(mark);
        self
    }

    /// Sets a date range with validation against `dd/MM/yyyy`.
    pub fn with_date_range(mut self, from: &str, to: &str) -> Result<Self, ExpenseQueryError> {
        validate_mydata_date(from)?;
        validate_mydata_date(to)?;
        self.date_from = Some(from.to_string());
        self.date_to = Some(to.to_string());
        Ok(self)
    }

    /// Filters by specific counterparty supplier AFM (validated for 9 numeric digits).
    pub fn with_supplier_afm(mut self, afm: &str) -> Result<Self, ExpenseQueryError> {
        let trimmed = afm.trim();
        if trimmed.len() != 9 || !trimmed.chars().all(|c| c.is_ascii_digit()) {
            return Err(ExpenseQueryError::InvalidAfm(afm.to_string()));
        }
        self.supplier_afm = Some(trimmed.to_string());
        Ok(self)
    }

    /// Sets invoice type filter (e.g. "1.1" Commercial Invoice, "2.1" Service Invoice).
    pub fn with_invoice_type(mut self, inv_type: &str) -> Self {
        self.invoice_type = Some(inv_type.trim().to_string());
        self
    }

    /// Sets continuation token for AADE pagination.
    pub fn with_partition_key(mut self, key: &str) -> Self {
        self.next_partition_key = Some(key.trim().to_string());
        self
    }

    /// Builds the full HTTP GET Request URL for AADE `RequestTransmittedDocs`.
    pub fn build_request_url(&self, credentials: &MyDataCredentials) -> String {
        let base_url = credentials.environment.base_url();
        let mut query_params: Vec<String> = Vec::new();

        if let Some(mark) = self.mark_from {
            query_params.push(format!("mark={}", mark));
        }
        if let Some(ref from) = self.date_from {
            query_params.push(format!("dateFrom={}", from));
        }
        if let Some(ref to) = self.date_to {
            query_params.push(format!("dateTo={}", to));
        }
        if let Some(ref afm) = self.supplier_afm {
            query_params.push(format!("entityVatNumber={}", afm));
        }
        if let Some(ref inv_t) = self.invoice_type {
            query_params.push(format!("invType={}", inv_t));
        }
        if let Some(ref part_key) = self.next_partition_key {
            query_params.push(format!("nextPartitionKey={}", part_key));
        }

        let query_str = if query_params.is_empty() {
            String::new()
        } else {
            format!("?{}", query_params.join("&"))
        };

        format!("{}/RequestTransmittedDocs{}", base_url, query_str)
    }

    /// Generates official AADE API request HTTP headers for authenticated transmission.
    pub fn build_request_headers(&self, credentials: &MyDataCredentials) -> Vec<(&'static str, String)> {
        vec![
            ("aade-del-user-id", credentials.user_id.clone()),
            ("Ocp-Apim-Subscription-Key", credentials.subscription_key.clone()),
            ("Accept", "application/xml".to_string()),
        ]
    }
}

/// Validates that a date conforms to the strict Greek myDATA `dd/MM/yyyy` format.
fn validate_mydata_date(date_str: &str) -> Result<(), ExpenseQueryError> {
    let parts: Vec<&str> = date_str.split('/').collect();
    if parts.len() != 3 {
        return Err(ExpenseQueryError::InvalidDateFormat(format!(
            "Απαιτείται μορφή dd/MM/yyyy, δόθηκε: {}",
            date_str
        )));
    }

    let day: u32 = parts[0]
        .parse()
        .map_err(|_| ExpenseQueryError::InvalidDateFormat("Μη έγκυρη ημέρα".to_string()))?;
    let month: u32 = parts[1]
        .parse()
        .map_err(|_| ExpenseQueryError::InvalidDateFormat("Μη έγκυρος μήνας".to_string()))?;
    let year: u32 = parts[2]
        .parse()
        .map_err(|_| ExpenseQueryError::InvalidDateFormat("Μη έγκυρο έτος".to_string()))?;

    if !(1..=31).contains(&day) || !(1..=12).contains(&month) || !(2000..=2100).contains(&year) {
        return Err(ExpenseQueryError::InvalidDateFormat(format!(
            "Ημερομηνία εκτός αποδεκτών ορίων: {}",
            date_str
        )));
    }

    Ok(())
}

/// Directly queries AADE `RequestTransmittedDocs` REST endpoint and retrieves raw response XML.
pub fn dispatch_request_transmitted_docs(
    credentials: &MyDataCredentials,
    filter: &InboundExpenseFilter,
    timeout_secs: u64,
) -> Result<String, String> {
    let url = filter.build_request_url(credentials);
    let mut req = ureq::get(&url)
        .timeout(std::time::Duration::from_secs(timeout_secs));

    for (header_name, header_val) in filter.build_request_headers(credentials) {
        req = req.set(header_name, &header_val);
    }

    match req.call() {
        Ok(resp) => resp
            .into_string()
            .map_err(|e| format!("Failed to read AADE response XML: {}", e)),
        Err(ureq::Error::Status(code, resp)) => {
            let body = resp.into_string().unwrap_or_default();
            Err(format!("AADE RequestTransmittedDocs HTTP {}: {}", code, body))
        }
        Err(e) => Err(format!("Network connection error to AADE: {}", e)),
    }
}

/// Fetches inbound expenses from AADE, parses XML documents, and records them into SQLite.
pub fn fetch_and_record_inbound_expenses(
    conn: &rusqlite::Connection,
    credentials: &MyDataCredentials,
    filter: &InboundExpenseFilter,
) -> Result<Vec<crate::mydata_expenses::ledger::InboundExpenseRecord>, String> {
    let _ = crate::mydata_expenses::ledger::init_inbound_expenses_schema(conn);
    let xml_body = dispatch_request_transmitted_docs(credentials, filter, 15)?;
    let parsed_resp = crate::mydata_expenses::parser::parse_requested_doc_xml(&xml_body);

    let mut recorded = Vec::new();
    for doc in parsed_resp.invoices {
        let mut rec = crate::mydata_expenses::ledger::InboundExpenseRecord::new(
            doc.mark,
            &doc.uid,
            &doc.supplier_afm,
            &doc.invoice_type,
            &doc.series,
            &doc.aa,
            &doc.issue_date,
            doc.total_net_cents,
            doc.total_vat_cents,
            doc.total_gross_cents,
        );
        rec.supplier_name = Some(format!("Προμηθευτής ΑΦΜ {}", doc.supplier_afm));

        crate::mydata_expenses::ledger::upsert_inbound_expense(conn, &rec)
            .map_err(|e| format!("Database error recording expense: {}", e))?;

        recorded.push(rec);
    }

    Ok(recorded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mydata_sync::MyDataEnvironment;

    #[test]
    fn test_inbound_expense_query_full_url() {
        let creds = MyDataCredentials::new(
            MyDataEnvironment::Production,
            "usr_proteus_01",
            "key_sec_9999",
            "123456789",
        );

        let filter = InboundExpenseFilter::new()
            .with_mark(400500)
            .with_date_range("01/10/2026", "10/10/2026")
            .unwrap()
            .with_supplier_afm("998877665")
            .unwrap()
            .with_invoice_type("1.1");

        let url = filter.build_request_url(&creds);
        assert_eq!(
            url,
            "https://mydatapi.aade.gr/myDATA/RequestTransmittedDocs?mark=400500&dateFrom=01/10/2026&dateTo=10/10/2026&entityVatNumber=998877665&invType=1.1"
        );

        let headers = filter.build_request_headers(&creds);
        assert_eq!(headers.len(), 3);
        assert_eq!(headers[0], ("aade-del-user-id", "usr_proteus_01".to_string()));
        assert_eq!(headers[1], ("Ocp-Apim-Subscription-Key", "key_sec_9999".to_string()));
    }

    #[test]
    fn test_date_and_afm_validations() {
        assert!(validate_mydata_date("15/08/2026").is_ok());
        assert!(validate_mydata_date("32/08/2026").is_err());
        assert!(validate_mydata_date("2026-08-15").is_err());

        let invalid_afm = InboundExpenseFilter::new().with_supplier_afm("123");
        assert!(invalid_afm.is_err());

        let valid_afm = InboundExpenseFilter::new().with_supplier_afm("094123456");
        assert!(valid_afm.is_ok());
    }
}
