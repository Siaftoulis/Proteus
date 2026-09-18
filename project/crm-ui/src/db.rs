pub fn init_db(db_path: &std::path::Path) -> Result<rusqlite::Connection, String> {
    let conn = rusqlite::Connection::open(db_path).map_err(|e| e.to_string())?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS records (
            id TEXT PRIMARY KEY,
            entity_type TEXT NOT NULL,
            data TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        CREATE INDEX IF NOT EXISTS idx_entity ON records(entity_type);"
    ).map_err(|e| e.to_string())?;
    Ok(conn)
}

pub fn upsert_record(
    conn: &rusqlite::Connection,
    entity_type: &str,
    record_id: Option<&str>,
    data_json: &serde_json::Value,
) -> Result<String, String> {
    let data_str = serde_json::to_string(data_json).map_err(|e| e.to_string())?;
    let id = match record_id {
        Some(id) => id.to_string(),
        None => uuid::Uuid::new_v4().to_string(),
    };
    conn.execute(
        "INSERT INTO records (id, entity_type, data) VALUES (?1, ?2, ?3)
         ON CONFLICT(id) DO UPDATE SET data = excluded.data, entity_type = excluded.entity_type",
        rusqlite::params![id, entity_type, data_str],
    ).map_err(|e| e.to_string())?;
    Ok(id)
}

pub fn delete_record(conn: &rusqlite::Connection, id: &str) -> Result<(), String> {
    conn.execute("DELETE FROM records WHERE id = ?1", rusqlite::params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_records(
    conn: &rusqlite::Connection,
    entity_type: &str,
) -> Result<Vec<(String, serde_json::Value)>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, data FROM records WHERE entity_type = ?1 ORDER BY created_at DESC"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map(rusqlite::params![entity_type], |row| {
        let id: String = row.get(0)?;
        let data_str: String = row.get(1)?;
        Ok((id, data_str))
    }).map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    for row in rows {
        let (id, data_str) = row.map_err(|e| e.to_string())?;
        let data: serde_json::Value = serde_json::from_str(&data_str).map_err(|e| e.to_string())?;
        result.push((id, data));
    }
    Ok(result)
}

pub fn import_contacts_csv(conn: &rusqlite::Connection, csv_data: &str) -> Result<Vec<crate::models::Contact>, String> {
    let contacts = crate::csv_utils::import_contacts_csv(csv_data);
    for c in &contacts {
        let data = serde_json::json!({
            "name": c.name,
            "email": c.email,
            "phone": c.phone,
            "company": c.company,
            "tags": c.tags,
            "notes": c.notes,
            "created_at": c.created_at,
            "updated_at": c.updated_at,
        });
        upsert_record(conn, "contacts", Some(&c.id), &data)?;
    }
    Ok(contacts)
}

pub fn get_contacts(conn: &rusqlite::Connection) -> Result<Vec<crate::models::Contact>, String> {
    let raw_records = get_records(conn, "contacts")?;
    let mut contacts = Vec::new();
    for (id, val) in raw_records {
        let name = val["name"].as_str().unwrap_or("").to_string();
        let email = val["email"].as_str().unwrap_or("").to_string();
        let phone = val["phone"].as_str().unwrap_or("").to_string();
        let company = val["company"].as_str().unwrap_or("").to_string();
        let tags: Vec<String> = val["tags"].as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();
        let created_at = val["created_at"].as_str().unwrap_or("").to_string();
        let updated_at = val["updated_at"].as_str().unwrap_or("").to_string();
        contacts.push(crate::models::Contact {
            id,
            name,
            email,
            phone,
            company,
            tags,
            notes: Vec::new(),
            created_at,
            updated_at,
        });
    }
    Ok(contacts)
}

pub fn import_deals_csv(conn: &rusqlite::Connection, csv_data: &str) -> Result<Vec<crate::models::Deal>, String> {
    let deals = crate::csv_utils::import_deals_csv(csv_data);
    for d in &deals {
        let data = serde_json::json!({
            "title": d.title,
            "value": d.value,
            "stage": d.stage,
            "contact_id": d.contact_id,
            "expected_close": d.expected_close,
            "notes": d.notes,
            "created_at": d.created_at,
            "updated_at": d.updated_at,
        });
        upsert_record(conn, "deals", Some(&d.id), &data)?;
    }
    Ok(deals)
}

pub fn get_deals(conn: &rusqlite::Connection) -> Result<Vec<crate::models::Deal>, String> {
    let raw_records = get_records(conn, "deals")?;
    let mut deals = Vec::new();
    for (id, val) in raw_records {
        let title = val["title"].as_str().unwrap_or("").to_string();
        let value = val["value"].as_f64().unwrap_or(0.0);
        let stage = val["stage"].as_str().unwrap_or("").to_string();
        let contact_id = val["contact_id"].as_str().map(|s| s.to_string());
        let expected_close = val["expected_close"].as_str().unwrap_or("").to_string();
        let notes = val["notes"].as_str().unwrap_or("").to_string();
        let created_at = val["created_at"].as_str().unwrap_or("").to_string();
        let updated_at = val["updated_at"].as_str().unwrap_or("").to_string();
        deals.push(crate::models::Deal {
            id,
            title,
            value,
            stage,
            contact_id,
            expected_close,
            notes,
            created_at,
            updated_at,
        });
    }
    Ok(deals)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mem_db() -> rusqlite::Connection {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS records (
                id TEXT PRIMARY KEY,
                entity_type TEXT NOT NULL,
                data TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            CREATE INDEX IF NOT EXISTS idx_entity ON records(entity_type);"
        ).unwrap();
        conn
    }

    #[test]
    fn init_creates_table() {
        let conn = mem_db();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM records", [], |r| r.get(0)
        ).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn insert_and_fetch() {
        let conn = mem_db();
        let data = serde_json::json!({"name": "Alice", "email": "a@b.com"});
        let id = upsert_record(&conn, "contacts", None, &data).unwrap();
        assert!(!id.is_empty());
        uuid::Uuid::parse_str(&id).expect("should be valid uuid");

        let records = get_records(&conn, "contacts").unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].0, id);
        assert_eq!(records[0].1, data);
    }
    #[test]
    fn import_contacts_csv_test() {
        let conn = mem_db();
        let contacts = get_contacts(&conn).unwrap();
        assert_eq!(contacts.len(), 0);
        // add csv data to conn
        let csv_data = "name,email\nAlice,a@b.com";
        let contacts = import_contacts_csv(&conn, csv_data).unwrap();
        assert_eq!(contacts.len(), 1);
        assert_eq!(contacts[0].name, "Alice");
        assert_eq!(contacts[0].email, "a@b.com");
    }

    #[test]
    fn import_deals_csv_test() {
        let conn = mem_db();
        let deals = get_deals(&conn).unwrap();
        assert_eq!(deals.len(), 0);
        // add csv data to conn
        let csv_data = "title,value,stage,contact_id,expected_close,notes\nDeal1,100,New,1,2022-12-31,notes1\nDeal2,200,New,2,2022-12-31,notes2";
        let deals = import_deals_csv(&conn, csv_data).unwrap();
        assert_eq!(deals.len(), 2);
        assert_eq!(deals[0].title, "Deal1");
        assert_eq!(deals[0].value, 100.0);
        assert_eq!(deals[0].stage, "New");
        assert_eq!(deals[0].contact_id.as_deref(), Some("1"));
        assert_eq!(deals[0].expected_close, "2022-12-31");
        assert_eq!(deals[0].notes, "notes1");
        assert_eq!(deals[1].title, "Deal2");
        assert_eq!(deals[1].value, 200.0);
        assert_eq!(deals[1].stage, "New");
        assert_eq!(deals[1].contact_id.as_deref(), Some("2"));
        assert_eq!(deals[1].expected_close, "2022-12-31");
        assert_eq!(deals[1].notes, "notes2");
    }

    #[test]
    fn update_existing() {
        let conn = mem_db();
        let data = serde_json::json!({"val": 1});
        let id = upsert_record(&conn, "items", None, &data).unwrap();

        let updated_data = serde_json::json!({"val": 2});
        let same_id = upsert_record(&conn, "items", Some(&id), &updated_data).unwrap();
        assert_eq!(id, same_id);

        let records = get_records(&conn, "items").unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].1, updated_data);
    }

    #[test]
    fn fetch_empty_entity() {
        let conn = mem_db();
        let records = get_records(&conn, "nonexistent").unwrap();
        assert!(records.is_empty());
    }

    #[test]
    fn delete_removes_record() {
        let conn = mem_db();
        let data = serde_json::json!({"x": 1});
        let id = upsert_record(&conn, "test", None, &data).unwrap();
        assert_eq!(get_records(&conn, "test").unwrap().len(), 1);
        delete_record(&conn, &id).unwrap();
        assert!(get_records(&conn, "test").unwrap().is_empty());
    }

    #[test]
    fn upsert_custom_id_creates_and_updates() {
        let conn = mem_db();
        let data1 = serde_json::json!({"name": "Initial"});
        let id1 = upsert_record(&conn, "contacts", Some("custom-c1"), &data1).unwrap();
        assert_eq!(id1, "custom-c1");
        let recs = get_records(&conn, "contacts").unwrap();
        assert_eq!(recs.len(), 1);
        assert_eq!(recs[0].1["name"], "Initial");

        let data2 = serde_json::json!({"name": "Updated"});
        let id2 = upsert_record(&conn, "contacts", Some("custom-c1"), &data2).unwrap();
        assert_eq!(id2, "custom-c1");
        let recs2 = get_records(&conn, "contacts").unwrap();
        assert_eq!(recs2.len(), 1);
        assert_eq!(recs2[0].1["name"], "Updated");
    }
}
