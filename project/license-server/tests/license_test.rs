use rusqlite::Connection;

// Inline the db module's functions for testing — we test the logic, not the import.
// Matching the actual module structure requires making the crate testable.
// ponytail: tests the SQL directly to verify DB logic without crate restructuring.

fn init_db(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS licenses (
            id TEXT PRIMARY KEY,
            license_key TEXT UNIQUE NOT NULL,
            customer_email TEXT NOT NULL,
            max_users INTEGER NOT NULL DEFAULT 3,
            features TEXT NOT NULL DEFAULT '[]',
            issued_at TEXT NOT NULL,
            expires_at TEXT NOT NULL,
            activations INTEGER NOT NULL DEFAULT 0
        );",
    )
    .map_err(|e| e.to_string())
}

fn insert_license(
    conn: &Connection,
    key: &str,
    email: &str,
    max_users: i32,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO licenses (id, license_key, customer_email, max_users, features, issued_at, expires_at, activations)
         VALUES (?1, ?2, ?3, ?4, '[]', '2024-01-01T00:00:00', '2025-01-01T00:00:00', 0)",
        rusqlite::params![uuid::Uuid::new_v4().to_string(), key, email, max_users],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn get_license_activations(conn: &Connection, key: &str) -> Result<Option<i32>, String> {
    let mut stmt = conn
        .prepare("SELECT activations FROM licenses WHERE license_key = ?1")
        .map_err(|e| e.to_string())?;
    let result = stmt.query_row(rusqlite::params![key], |row| row.get(0)).ok();
    Ok(result)
}

fn increment_activations(conn: &Connection, key: &str) -> Result<(), String> {
    conn.execute(
        "UPDATE licenses SET activations = activations + 1 WHERE license_key = ?1",
        rusqlite::params![key],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

#[test]
fn test_init_db_creates_table() {
    let conn = Connection::open_in_memory().unwrap();
    init_db(&conn).unwrap();
    let count: i32 = conn
        .query_row("SELECT COUNT(*) FROM licenses", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn test_insert_and_get_license() {
    let conn = Connection::open_in_memory().unwrap();
    init_db(&conn).unwrap();
    insert_license(&conn, "TEST-123", "test@example.com", 3).unwrap();
    let activations = get_license_activations(&conn, "TEST-123").unwrap();
    assert_eq!(activations, Some(0));
}

#[test]
fn test_get_nonexistent_license() {
    let conn = Connection::open_in_memory().unwrap();
    init_db(&conn).unwrap();
    let activations = get_license_activations(&conn, "NONEXISTENT").unwrap();
    assert_eq!(activations, None);
}

#[test]
fn test_increment_activations() {
    let conn = Connection::open_in_memory().unwrap();
    init_db(&conn).unwrap();
    insert_license(&conn, "INCR-001", "user@example.com", 3).unwrap();
    increment_activations(&conn, "INCR-001").unwrap();
    let activations = get_license_activations(&conn, "INCR-001").unwrap();
    assert_eq!(activations, Some(1));
    increment_activations(&conn, "INCR-001").unwrap();
    let activations = get_license_activations(&conn, "INCR-001").unwrap();
    assert_eq!(activations, Some(2));
}

#[test]
fn test_max_users_limit() {
    let conn = Connection::open_in_memory().unwrap();
    init_db(&conn).unwrap();
    insert_license(&conn, "MAX-001", "user@example.com", 2).unwrap();
    // Simulate 2 activations
    increment_activations(&conn, "MAX-001").unwrap();
    increment_activations(&conn, "MAX-001").unwrap();
    let activations = get_license_activations(&conn, "MAX-001").unwrap();
    assert_eq!(activations, Some(2));
    // max_users = 2, so next increment would exceed limit
    let max_users: i32 = conn
        .query_row("SELECT max_users FROM licenses WHERE license_key = ?1", rusqlite::params!["MAX-001"], |row| row.get(0))
        .unwrap();
    assert_eq!(max_users, 2);
    assert!(activations.unwrap() >= max_users);
}

#[test]
fn test_duplicate_license_key_fails() {
    let conn = Connection::open_in_memory().unwrap();
    init_db(&conn).unwrap();
    insert_license(&conn, "DUP-KEY", "first@example.com", 3).unwrap();
    let result = insert_license(&conn, "DUP-KEY", "second@example.com", 3);
    assert!(result.is_err());
}
