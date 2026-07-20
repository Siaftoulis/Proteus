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
    match record_id {
        Some(id) => {
            conn.execute(
                "UPDATE records SET data = ?1 WHERE id = ?2",
                rusqlite::params![data_str, id],
            ).map_err(|e| e.to_string())?;
            Ok(id.to_string())
        }
        None => {
            let id = uuid::Uuid::new_v4().to_string();
            conn.execute(
                "INSERT INTO records (id, entity_type, data) VALUES (?1, ?2, ?3)",
                rusqlite::params![id, entity_type, data_str],
            ).map_err(|e| e.to_string())?;
            Ok(id)
        }
    }
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
}
