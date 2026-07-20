use rusqlite::{params, Connection};

use crate::models::License;

pub fn init_db(conn: &Connection) -> Result<(), String> {
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

pub fn get_license(conn: &Connection, license_key: &str) -> Result<Option<License>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, license_key, customer_email, max_users, features, issued_at, expires_at, activations
             FROM licenses WHERE license_key = ?1",
        )
        .map_err(|e| e.to_string())?;

    let result = stmt
        .query_row(params![license_key], |row| {
            Ok(License {
                id: row.get(0)?,
                license_key: row.get(1)?,
                customer_email: row.get(2)?,
                max_users: row.get(3)?,
                features: row.get(4)?,
                issued_at: row.get(5)?,
                expires_at: row.get(6)?,
                activations: row.get(7)?,
            })
        })
        .ok();

    Ok(result)
}

pub fn insert_license(conn: &Connection, license: &License) -> Result<(), String> {
    conn.execute(
        "INSERT INTO licenses (id, license_key, customer_email, max_users, features, issued_at, expires_at, activations)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            license.id,
            license.license_key,
            license.customer_email,
            license.max_users,
            license.features,
            license.issued_at,
            license.expires_at,
            license.activations,
        ],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn increment_activations(conn: &Connection, license_key: &str) -> Result<(), String> {
    conn.execute(
        "UPDATE licenses SET activations = activations + 1 WHERE license_key = ?1",
        params![license_key],
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
