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

/// Imports Contacts from a CSV formatted string.
pub fn import_contacts_csv(csv_text: &str) -> Vec<Contact> {
    let mut contacts = Vec::new();
    let mut lines = csv_text.lines();
    
    // Skip header line if present
    if let Some(header) = lines.next() {
        if !header.to_lowercase().contains("name") {
            // First line was not a header, try parsing it as a row
            if let Some(c) = parse_contact_row(header) {
                contacts.push(c);
            }
        }
    }

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(c) = parse_contact_row(trimmed) {
            contacts.push(c);
        }
    }

    contacts
}

fn parse_contact_row(line: &str) -> Option<Contact> {
    let fields = parse_csv_line(line);
    if fields.is_empty() || (fields.len() == 1 && fields[0].is_empty()) {
        return None;
    }

    // Header order: id,name,email,phone,company,tags
    let (id, name, email, phone, company, tags) = if fields.len() >= 6 {
        (
            if fields[0].is_empty() { uuid::Uuid::new_v4().to_string() } else { fields[0].clone() },
            fields[1].clone(),
            fields[2].clone(),
            fields[3].clone(),
            fields[4].clone(),
            fields[5].split(';').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
        )
    } else if fields.len() >= 2 {
        (
            if fields[0].is_empty() { uuid::Uuid::new_v4().to_string() } else { fields[0].clone() },
            fields[1].clone(),
            fields.get(2).cloned().unwrap_or_default(),
            fields.get(3).cloned().unwrap_or_default(),
            fields.get(4).cloned().unwrap_or_default(),
            Vec::new(),
        )
    } else {
        (
            uuid::Uuid::new_v4().to_string(),
            fields[0].clone(),
            String::new(),
            String::new(),
            String::new(),
            Vec::new(),
        )
    };

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
    let mut deals = Vec::new();
    let mut lines = csv_text.lines();

    if let Some(header) = lines.next() {
        if !header.to_lowercase().contains("title") {
            if let Some(d) = parse_deal_row(header) {
                deals.push(d);
            }
        }
    }

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(d) = parse_deal_row(trimmed) {
            deals.push(d);
        }
    }

    deals
}

fn parse_deal_row(line: &str) -> Option<Deal> {
    let fields = parse_csv_line(line);
    if fields.is_empty() || (fields.len() == 1 && fields[0].is_empty()) {
        return None;
    }

    // Header order: id,title,value,stage,contact_id,expected_close,notes
    let id = if !fields[0].is_empty() { fields[0].clone() } else { uuid::Uuid::new_v4().to_string() };
    let title = fields.get(1).cloned().unwrap_or_else(|| "Untitled Deal".into());
    let value = fields.get(2).and_then(|v| v.parse::<f64>().ok()).unwrap_or(0.0);
    let stage = fields.get(3).cloned().unwrap_or_else(|| "New".into());
    let contact_id = fields.get(4).filter(|s| !s.is_empty()).cloned();
    let expected_close = fields.get(5).cloned().unwrap_or_default();
    let notes = fields.get(6).cloned().unwrap_or_default();

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
