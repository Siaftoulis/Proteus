use chrono::Utc;
use crate::models::{Contact, Deal};

/// Escapes a CSV cell according to standard CSV rules (RFC 4180).
pub fn escape_csv_field(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// Parses a single CSV line into a list of field values, handling escaped quotes.
pub fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '"' => {
                if in_quotes && chars.peek() == Some(&'"') {
                    current.push('"');
                    chars.next(); // Skip escaped quote
                } else {
                    in_quotes = !in_quotes;
                }
            }
            ',' if !in_quotes => {
                fields.push(current.trim().to_string());
                current.clear();
            }
            _ => {
                current.push(c);
            }
        }
    }
    fields.push(current.trim().to_string());
    fields
}

/// Exports a slice of Contacts to a CSV formatted string.
pub fn export_contacts_csv(contacts: &[Contact]) -> String {
    let mut csv = String::from("id,name,email,phone,company,tags\n");
    for c in contacts {
        let tags_joined = c.tags.join(";");
        csv.push_str(&format!(
            "{},{},{},{},{},{}\n",
            escape_csv_field(&c.id),
            escape_csv_field(&c.name),
            escape_csv_field(&c.email),
            escape_csv_field(&c.phone),
            escape_csv_field(&c.company),
            escape_csv_field(&tags_joined),
        ));
    }
    csv
}

fn find_col(fields: &[String], names: &[&str]) -> Option<usize> {
    fields.iter().position(|f| {
        let lower = f.trim().to_lowercase();
        names.iter().any(|&n| lower == n)
    })
}

struct ContactIndices {
    id: Option<usize>,
    name: Option<usize>,
    email: Option<usize>,
    phone: Option<usize>,
    company: Option<usize>,
    tags: Option<usize>,
}

impl ContactIndices {
    fn from_header(f: &[String]) -> Self {
        Self {
            id: find_col(f, &["id"]),
            name: find_col(f, &["name", "full_name", "contact_name"]),
            email: find_col(f, &["email", "email_address"]),
            phone: find_col(f, &["phone", "telephone"]),
            company: find_col(f, &["company", "organization"]),
            tags: find_col(f, &["tags", "tag"]),
        }
    }
    fn positional() -> Self {
        Self { id: Some(0), name: Some(1), email: Some(2), phone: Some(3), company: Some(4), tags: Some(5) }
    }
}

fn parse_contact_fields(fields: &[String], indices: &ContactIndices) -> Option<Contact> {
    if fields.is_empty() || (fields.len() == 1 && fields[0].trim().is_empty()) {
        return None;
    }
    let (id, name) = if indices.name.is_none() && fields.len() == 1 {
        (uuid::Uuid::new_v4().to_string(), fields[0].clone())
    } else {
        let id = indices.id
            .and_then(|i| fields.get(i))
            .filter(|s| !s.trim().is_empty())
            .cloned()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let name = indices.name.and_then(|i| fields.get(i)).cloned().unwrap_or_default();
        (id, name)
    };
    let email = indices.email.and_then(|i| fields.get(i)).cloned().unwrap_or_default();
    let phone = indices.phone.and_then(|i| fields.get(i)).cloned().unwrap_or_default();
    let company = indices.company.and_then(|i| fields.get(i)).cloned().unwrap_or_default();
    let tags = indices.tags
        .and_then(|i| fields.get(i))
        .map(|s| s.split(';').map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect())
        .unwrap_or_default();

    Some(Contact {
        id,
        name,
        email,
        phone,
        company,
        tags,
        notes: vec![],
        created_at: Utc::now().to_rfc3339(),
        updated_at: Utc::now().to_rfc3339(),
    })
}

struct DealIndices {
    id: Option<usize>,
    title: Option<usize>,
    value: Option<usize>,
    stage: Option<usize>,
    contact_id: Option<usize>,
    expected_close: Option<usize>,
    notes: Option<usize>,
}

impl DealIndices {
    fn from_header(f: &[String]) -> Self {
        Self {
            id: find_col(f, &["id"]),
            title: find_col(f, &["title", "deal", "deal_name"]),
            value: find_col(f, &["value", "amount"]),
            stage: find_col(f, &["stage", "status"]),
            contact_id: find_col(f, &["contact_id", "contact"]),
            expected_close: find_col(f, &["expected_close", "close_date"]),
            notes: find_col(f, &["notes", "note"]),
        }
    }
    fn positional() -> Self {
        Self { id: Some(0), title: Some(1), value: Some(2), stage: Some(3), contact_id: Some(4), expected_close: Some(5), notes: Some(6) }
    }
}

fn parse_deal_fields(fields: &[String], indices: &DealIndices) -> Option<Deal> {
    if fields.is_empty() || (fields.len() == 1 && fields[0].trim().is_empty()) {
        return None;
    }
    let (id, title) = if indices.title.is_none() && fields.len() == 1 {
        (uuid::Uuid::new_v4().to_string(), fields[0].clone())
    } else {
        let id = indices.id
            .and_then(|i| fields.get(i))
            .filter(|s| !s.trim().is_empty())
            .cloned()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let title = indices.title.and_then(|i| fields.get(i)).cloned().unwrap_or_else(|| "Untitled Deal".into());
        (id, title)
    };
    let value = indices.value
        .and_then(|i| fields.get(i))
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0);
    let stage = indices.stage
        .and_then(|i| fields.get(i))
        .filter(|s| !s.trim().is_empty())
        .cloned()
        .unwrap_or_else(|| "New".into());
    let contact_id = indices.contact_id.and_then(|i| fields.get(i)).filter(|s| !s.trim().is_empty()).cloned();
    let expected_close = indices.expected_close.and_then(|i| fields.get(i)).cloned().unwrap_or_default();
    let notes = indices.notes.and_then(|i| fields.get(i)).cloned().unwrap_or_default();

    Some(Deal {
        id,
        title,
        value,
        stage,
        contact_id,
        expected_close,
        notes,
        created_at: Utc::now().to_rfc3339(),
        updated_at: Utc::now().to_rfc3339(),
    })
}

fn parse_csv_rows<T, I, F>(
    csv_text: &str,
    is_header_fn: impl Fn(&str) -> bool,
    build_indices: impl Fn(&[String], bool) -> I,
    mut parse_row: F,
) -> Vec<T>
where
    F: FnMut(&[String], &I) -> Option<T>,
{
    let mut items = Vec::new();
    let mut lines = csv_text.lines();
    let first = match lines.next() {
        Some(l) => l,
        None => return items,
    };
    let first_fields = parse_csv_line(first);
    let is_header = is_header_fn(first);
    let indices = build_indices(&first_fields, is_header);
    if !is_header {
        if let Some(item) = parse_row(&first_fields, &indices) {
            items.push(item);
        }
    }
    for line in lines {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            let fields = parse_csv_line(trimmed);
            if let Some(item) = parse_row(&fields, &indices) {
                items.push(item);
            }
        }
    }
    items
}

/// Imports Contacts from a CSV formatted string.
pub fn import_contacts_csv(csv_text: &str) -> Vec<Contact> {
    parse_csv_rows(
        csv_text,
        |h| h.to_lowercase().contains("name"),
        |f, is_hdr| if is_hdr { ContactIndices::from_header(f) } else { ContactIndices::positional() },
        parse_contact_fields,
    )
}

/// Exports a slice of Deals to a CSV formatted string.
pub fn export_deals_csv(deals: &[Deal]) -> String {
    let mut csv = String::from("id,title,value,stage,contact_id,expected_close,notes\n");
    for d in deals {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{}\n",
            escape_csv_field(&d.id),
            escape_csv_field(&d.title),
            d.value,
            escape_csv_field(&d.stage),
            escape_csv_field(d.contact_id.as_deref().unwrap_or("")),
            escape_csv_field(&d.expected_close),
            escape_csv_field(&d.notes),
        ));
    }
    csv
}

/// Imports Deals from a CSV formatted string.
pub fn import_deals_csv(csv_text: &str) -> Vec<Deal> {
    parse_csv_rows(
        csv_text,
        |h| {
            let l = h.to_lowercase();
            l.contains("title") || l.contains("deal")
        },
        |f, is_hdr| if is_hdr { DealIndices::from_header(f) } else { DealIndices::positional() },
        parse_deal_fields,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_csv_field() {
        assert_eq!(escape_csv_field("simple"), "simple");
        assert_eq!(escape_csv_field("with,comma"), "\"with,comma\"");
        assert_eq!(escape_csv_field("with \"quotes\""), "\"with \"\"quotes\"\"\"");
    }

    #[test]
    fn test_parse_csv_line() {
        let line = "c1,\"Acme, Inc.\",info@acme.com,\"tag1;tag2\"";
        let fields = parse_csv_line(line);
        assert_eq!(fields, vec!["c1", "Acme, Inc.", "info@acme.com", "tag1;tag2"]);
    }

    #[test]
    fn test_contacts_export_import_roundtrip() {
        let contacts = vec![
            Contact {
                id: "c1".into(),
                name: "Alice Smith".into(),
                email: "alice@example.com".into(),
                phone: "123-456".into(),
                company: "Tech Corp".into(),
                tags: vec!["lead".into(), "vip".into()],
                notes: vec![],
                created_at: "2026-01-01T00:00:00Z".into(),
                updated_at: "2026-01-01T00:00:00Z".into(),
            },
            Contact {
                id: "c2".into(),
                name: "Bob, \"The Builder\"".into(),
                email: "bob@builder.com".into(),
                phone: "".into(),
                company: "Construction Co.".into(),
                tags: vec!["partner".into()],
                notes: vec![],
                created_at: "2026-01-01T00:00:00Z".into(),
                updated_at: "2026-01-01T00:00:00Z".into(),
            },
        ];

        let csv = export_contacts_csv(&contacts);
        let imported = import_contacts_csv(&csv);

        assert_eq!(imported.len(), 2);
        assert_eq!(imported[0].id, "c1");
        assert_eq!(imported[0].name, "Alice Smith");
        assert_eq!(imported[0].email, "alice@example.com");
        assert_eq!(imported[0].company, "Tech Corp");
        assert_eq!(imported[0].tags, vec!["lead", "vip"]);

        assert_eq!(imported[1].id, "c2");
        assert_eq!(imported[1].name, "Bob, \"The Builder\"");
        assert_eq!(imported[1].company, "Construction Co.");
        assert_eq!(imported[1].tags, vec!["partner"]);
    }

    #[test]
    fn test_deals_export_import_roundtrip() {
        let deals = vec![
            Deal {
                id: "d1".into(),
                title: "Big Software Sale".into(),
                value: 25000.0,
                stage: "Proposal".into(),
                contact_id: Some("c1".into()),
                expected_close: "2026-12-31".into(),
                notes: "Close before year end".into(),
                created_at: "2026-01-01T00:00:00Z".into(),
                updated_at: "2026-01-01T00:00:00Z".into(),
            },
        ];

        let csv = export_deals_csv(&deals);
        let imported = import_deals_csv(&csv);

        assert_eq!(imported.len(), 1);
        assert_eq!(imported[0].id, "d1");
        assert_eq!(imported[0].title, "Big Software Sale");
        assert_eq!(imported[0].value, 25000.0);
        assert_eq!(imported[0].stage, "Proposal");
        assert_eq!(imported[0].contact_id, Some("c1".into()));
    }
}
