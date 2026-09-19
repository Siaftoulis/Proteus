//! Anti-SAP Multi-Tenant & Hierarchical Multi-Store Engine.
//! Manages Enterprise HQ, store branches, departments, and delegated user quotas with Merkle auditing.

use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Enterprise {
    pub enterprise_id: String,
    pub legal_name: String,
    pub tax_id: String, // ΑΦΜ
    pub total_seat_quota: i64,
    pub used_seats: i64,
    pub cloud_tier: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnterpriseStore {
    pub store_id: String,
    pub enterprise_id: String,
    pub store_code: String,
    pub store_name: String,
    pub address: String,
    pub phone: String,
    pub allocated_seats: i64,
    pub active_seats: i64,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoreDepartment {
    pub department_id: String,
    pub store_id: String,
    pub dept_code: String, // "WAREHOUSE", "MOBILE", "SERVICE", "POS"
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnterpriseUser {
    pub user_id: String,
    pub enterprise_id: String,
    pub store_id: String,
    pub department_id: Option<String>,
    pub full_name: String,
    pub email: String,
    pub role_code: String,
    pub is_active: bool,
    pub created_at: String,
}

/// Initializes the Multi-Store and Enterprise DDL schema.
pub fn init_enterprise_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS enterprises (
            enterprise_id TEXT PRIMARY KEY, legal_name TEXT NOT NULL, tax_id TEXT NOT NULL UNIQUE,
            total_seat_quota INTEGER NOT NULL DEFAULT 10, used_seats INTEGER NOT NULL DEFAULT 0,
            cloud_tier TEXT NOT NULL DEFAULT 'EnterpriseCore', created_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS enterprise_stores (
            store_id TEXT PRIMARY KEY, enterprise_id TEXT NOT NULL REFERENCES enterprises(enterprise_id),
            store_code TEXT NOT NULL UNIQUE, store_name TEXT NOT NULL, address TEXT NOT NULL,
            phone TEXT NOT NULL, allocated_seats INTEGER NOT NULL DEFAULT 5, active_seats INTEGER NOT NULL DEFAULT 0,
            is_active INTEGER NOT NULL DEFAULT 1, created_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS store_departments (
            department_id TEXT PRIMARY KEY, store_id TEXT NOT NULL REFERENCES enterprise_stores(store_id),
            dept_code TEXT NOT NULL, name TEXT NOT NULL, created_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS enterprise_users (
            user_id TEXT PRIMARY KEY, enterprise_id TEXT NOT NULL REFERENCES enterprises(enterprise_id),
            store_id TEXT NOT NULL REFERENCES enterprise_stores(store_id), department_id TEXT REFERENCES store_departments(department_id),
            full_name TEXT NOT NULL, email TEXT NOT NULL UNIQUE, role_code TEXT NOT NULL,
            is_active INTEGER NOT NULL DEFAULT 1, created_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_stores_ent ON enterprise_stores(enterprise_id);
        CREATE INDEX IF NOT EXISTS idx_depts_store ON store_departments(store_id);
        CREATE INDEX IF NOT EXISTS idx_users_store ON enterprise_users(store_id);
        "#,
    )
}

/// Seeds default primary enterprise and flagship stores if none exists.
pub fn seed_default_enterprise_if_empty(conn: &Connection) -> Result<()> {
    let count: i64 = conn.query_row("SELECT count(*) FROM enterprises", [], |r| r.get(0))?;
    if count == 0 {
        let ent = Enterprise {
            enterprise_id: "ent-001-default".into(),
            legal_name: "Proteus Retail & Services Hellas A.E.".into(),
            tax_id: "099887766".into(),
            total_seat_quota: 30,
            used_seats: 12,
            cloud_tier: "EnterpriseCore".into(),
            created_at: now_iso(),
        };
        create_enterprise(conn, &ent)?;

        let store1 = EnterpriseStore {
            store_id: "str-001-syntagma".into(), enterprise_id: ent.enterprise_id.clone(),
            store_code: "STR-001-SYNTAGMA".into(), store_name: "Κατάστημα 01: Σύνταγμα (Flagship)".into(),
            address: "Πλατεία Συντάγματος 4, Αθήνα".into(), phone: "+30 210 3210001".into(),
            allocated_seats: 15, active_seats: 0, is_active: true, created_at: now_iso(),
        };
        create_enterprise_store(conn, &store1)?;

        for (code, name) in [("WAREHOUSE", "Κεντρική Αποθήκη"), ("MOBILE", "Τηλεφωνία"), ("SERVICE", "Service"), ("POS", "Ταμείο")] {
            create_store_department(conn, &StoreDepartment {
                department_id: format!("dept-001-{}", code.to_lowercase()),
                store_id: store1.store_id.clone(), dept_code: code.into(), name: name.into(), created_at: now_iso(),
            })?;
        }

        let store2 = EnterpriseStore {
            store_id: "str-002-tsimiski".into(), enterprise_id: ent.enterprise_id.clone(),
            store_code: "STR-002-TSIMISKI".into(), store_name: "Κατάστημα 02: Τσιμισκή Θεσσαλονίκη".into(),
            address: "Τσιμισκή 45, Θεσσαλονίκη".into(), phone: "+30 2310 220002".into(),
            allocated_seats: 10, active_seats: 0, is_active: true, created_at: now_iso(),
        };
        create_enterprise_store(conn, &store2)?;
        for (code, name) in [("POS", "Ταμείο"), ("SERVICE", "Τεχνικό Service")] {
            create_store_department(conn, &StoreDepartment {
                department_id: format!("dept-002-{}", code.to_lowercase()),
                store_id: store2.store_id.clone(), dept_code: code.into(), name: name.into(), created_at: now_iso(),
            })?;
        }
    }
    Ok(())
}

pub fn create_enterprise(conn: &Connection, ent: &Enterprise) -> Result<()> {
    conn.execute(
        "INSERT INTO enterprises (enterprise_id, legal_name, tax_id, total_seat_quota, used_seats, cloud_tier, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![ent.enterprise_id, ent.legal_name, ent.tax_id, ent.total_seat_quota, ent.used_seats, ent.cloud_tier, ent.created_at],
    )?;
    Ok(())
}

pub fn get_primary_enterprise(conn: &Connection) -> Result<Option<Enterprise>> {
    let mut stmt = conn.prepare("SELECT enterprise_id, legal_name, tax_id, total_seat_quota, used_seats, cloud_tier, created_at FROM enterprises LIMIT 1")?;
    let mut rows = stmt.query([])?;
    if let Some(row) = rows.next()? {
        Ok(Some(Enterprise {
            enterprise_id: row.get(0)?, legal_name: row.get(1)?, tax_id: row.get(2)?,
            total_seat_quota: row.get(3)?, used_seats: row.get(4)?, cloud_tier: row.get(5)?, created_at: row.get(6)?,
        }))
    } else {
        Ok(None)
    }
}

pub fn create_enterprise_store(conn: &Connection, store: &EnterpriseStore) -> Result<()> {
    conn.execute(
        "INSERT INTO enterprise_stores (store_id, enterprise_id, store_code, store_name, address, phone, allocated_seats, active_seats, is_active, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![store.store_id, store.enterprise_id, store.store_code, store.store_name, store.address, store.phone, store.allocated_seats, store.active_seats, if store.is_active { 1 } else { 0 }, store.created_at],
    )?;
    Ok(())
}

pub fn list_enterprise_stores(conn: &Connection, enterprise_id: &str) -> Result<Vec<EnterpriseStore>> {
    let mut stmt = conn.prepare("SELECT store_id, enterprise_id, store_code, store_name, address, phone, allocated_seats, active_seats, is_active, created_at FROM enterprise_stores WHERE enterprise_id = ?1 ORDER BY store_code ASC")?;
    let rows = stmt.query_map(params![enterprise_id], |row| {
        let is_act: i64 = row.get(8)?;
        Ok(EnterpriseStore {
            store_id: row.get(0)?, enterprise_id: row.get(1)?, store_code: row.get(2)?, store_name: row.get(3)?,
            address: row.get(4)?, phone: row.get(5)?, allocated_seats: row.get(6)?, active_seats: row.get(7)?,
            is_active: is_act != 0, created_at: row.get(9)?,
        })
    })?;
    rows.collect()
}

pub fn create_store_department(conn: &Connection, dept: &StoreDepartment) -> Result<()> {
    conn.execute(
        "INSERT INTO store_departments (department_id, store_id, dept_code, name, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![dept.department_id, dept.store_id, dept.dept_code, dept.name, dept.created_at],
    )?;
    Ok(())
}

pub fn list_store_departments(conn: &Connection, store_id: &str) -> Result<Vec<StoreDepartment>> {
    let mut stmt = conn.prepare("SELECT department_id, store_id, dept_code, name, created_at FROM store_departments WHERE store_id = ?1 ORDER BY name ASC")?;
    let rows = stmt.query_map(params![store_id], |row| {
        Ok(StoreDepartment {
            department_id: row.get(0)?, store_id: row.get(1)?, dept_code: row.get(2)?, name: row.get(3)?, created_at: row.get(4)?,
        })
    })?;
    rows.collect()
}

pub fn calculate_seat_utilization(conn: &Connection, enterprise_id: &str) -> Result<(i64, i64)> {
    let total_quota: i64 = conn.query_row("SELECT total_seat_quota FROM enterprises WHERE enterprise_id = ?1", params![enterprise_id], |r| r.get(0)).unwrap_or(0);
    let allocated_sum: i64 = conn.query_row("SELECT COALESCE(SUM(allocated_seats), 0) FROM enterprise_stores WHERE enterprise_id = ?1", params![enterprise_id], |r| r.get(0)).unwrap_or(0);
    Ok((allocated_sum, total_quota))
}

/// Delegated User Provisioning: Enforces store seat quota and appends to Merkle audit chain.
pub fn provision_store_user(conn: &Connection, user: &EnterpriseUser) -> std::result::Result<(), String> {
    let (allocated, active): (i64, i64) = conn.query_row(
        "SELECT allocated_seats, active_seats FROM enterprise_stores WHERE store_id = ?1",
        params![user.store_id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    ).map_err(|e| format!("Κατάστημα μη διαθέσιμο: {}", e))?;

    if active >= allocated {
        return Err(format!("Υπέρβαση ορίου θέσεων καταστήματος: {}/{} κατειλημμένες", active, allocated));
    }

    conn.execute(
        "INSERT INTO enterprise_users (user_id, enterprise_id, store_id, department_id, full_name, email, role_code, is_active, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1, ?8)",
        params![user.user_id, user.enterprise_id, user.store_id, user.department_id, user.full_name, user.email, user.role_code, user.created_at],
    ).map_err(|e| format!("Αποτυχία καταχώρισης χρήστη: {}", e))?;

    let _ = conn.execute("UPDATE enterprise_stores SET active_seats = active_seats + 1 WHERE store_id = ?1", params![user.store_id]);
    let _ = conn.execute("UPDATE enterprises SET used_seats = used_seats + 1 WHERE enterprise_id = ?1", params![user.enterprise_id]);

    let payload = serde_json::to_string(user).unwrap_or_default();
    let _ = crate::merkle::append_merkle_block(conn, "USER", &user.user_id, "PROVISION", &payload);

    Ok(())
}

pub fn list_store_users(conn: &Connection, store_id: &str) -> Result<Vec<EnterpriseUser>> {
    let mut stmt = conn.prepare("SELECT user_id, enterprise_id, store_id, department_id, full_name, email, role_code, is_active, created_at FROM enterprise_users WHERE store_id = ?1 ORDER BY full_name ASC")?;
    let rows = stmt.query_map(params![store_id], |row| {
        let is_act: i64 = row.get(7)?;
        Ok(EnterpriseUser {
            user_id: row.get(0)?, enterprise_id: row.get(1)?, store_id: row.get(2)?, department_id: row.get(3)?,
            full_name: row.get(4)?, email: row.get(5)?, role_code: row.get(6)?, is_active: is_act != 0, created_at: row.get(8)?,
        })
    })?;
    rows.collect()
}

pub fn deactivate_store_user(conn: &Connection, user_id: &str) -> std::result::Result<(), String> {
    let (store_id, ent_id, was_active): (String, String, i64) = conn.query_row(
        "SELECT store_id, enterprise_id, is_active FROM enterprise_users WHERE user_id = ?1",
        params![user_id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    ).map_err(|e| e.to_string())?;

    if was_active == 0 {
        return Ok(());
    }

    conn.execute("UPDATE enterprise_users SET is_active = 0 WHERE user_id = ?1", params![user_id]).map_err(|e| e.to_string())?;
    let _ = conn.execute("UPDATE enterprise_stores SET active_seats = MAX(0, active_seats - 1) WHERE store_id = ?1", params![store_id]);
    let _ = conn.execute("UPDATE enterprises SET used_seats = MAX(0, used_seats - 1) WHERE enterprise_id = ?1", params![ent_id]);

    let _ = crate::merkle::append_merkle_block(conn, "USER", user_id, "DEACTIVATE", "{}");
    Ok(())
}

fn now_iso() -> String {
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0);
    format!("{}", now)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enterprise_schema_and_delegated_provisioning() {
        let conn = Connection::open_in_memory().unwrap();
        crate::merkle::init_merkle_schema(&conn).unwrap();
        init_enterprise_schema(&conn).unwrap();
        seed_default_enterprise_if_empty(&conn).unwrap();

        let ent = get_primary_enterprise(&conn).unwrap().unwrap();
        let stores = list_enterprise_stores(&conn, &ent.enterprise_id).unwrap();
        let store = &stores[0];

        let u1 = EnterpriseUser {
            user_id: "usr-1".into(), enterprise_id: ent.enterprise_id.clone(), store_id: store.store_id.clone(),
            department_id: None, full_name: "Κώστας Ταμίας".into(), email: "kostas@shop.gr".into(),
            role_code: "PosCashier".into(), is_active: true, created_at: now_iso(),
        };
        assert!(provision_store_user(&conn, &u1).is_ok());

        let users = list_store_users(&conn, &store.store_id).unwrap();
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].full_name, "Κώστας Ταμίας");

        // Deactivate
        assert!(deactivate_store_user(&conn, "usr-1").is_ok());
        let updated = list_store_users(&conn, &store.store_id).unwrap();
        assert!(!updated[0].is_active);

        // Verify Merkle audit blocks were recorded
        let report = crate::merkle::verify_chain_integrity(&conn).unwrap();
        assert!(report.is_valid);
        assert_eq!(report.total_blocks, 2); // 1 PROVISION + 1 DEACTIVATE
    }

    #[test]
    fn test_provision_quota_exceeded_rejected() {
        let conn = Connection::open_in_memory().unwrap();
        crate::merkle::init_merkle_schema(&conn).unwrap();
        init_enterprise_schema(&conn).unwrap();

        let ent = Enterprise {
            enterprise_id: "e-tiny".into(), legal_name: "Tiny Shop".into(), tax_id: "123".into(),
            total_seat_quota: 1, used_seats: 0, cloud_tier: "Core".into(), created_at: now_iso(),
        };
        create_enterprise(&conn, &ent).unwrap();

        let store = EnterpriseStore {
            store_id: "s-tiny".into(), enterprise_id: ent.enterprise_id.clone(), store_code: "STR-TINY".into(),
            store_name: "Tiny".into(), address: "".into(), phone: "".into(), allocated_seats: 1, active_seats: 0,
            is_active: true, created_at: now_iso(),
        };
        create_enterprise_store(&conn, &store).unwrap();

        let u1 = EnterpriseUser {
            user_id: "u1".into(), enterprise_id: ent.enterprise_id.clone(), store_id: store.store_id.clone(),
            department_id: None, full_name: "User 1".into(), email: "u1@test.com".into(), role_code: "Pos".into(),
            is_active: true, created_at: now_iso(),
        };
        assert!(provision_store_user(&conn, &u1).is_ok());

        let u2 = EnterpriseUser {
            user_id: "u2".into(), enterprise_id: ent.enterprise_id.clone(), store_id: store.store_id.clone(),
            department_id: None, full_name: "User 2".into(), email: "u2@test.com".into(), role_code: "Pos".into(),
            is_active: true, created_at: now_iso(),
        };
        // Exceeds quota! Must be rejected
        let res = provision_store_user(&conn, &u2);
        assert!(res.is_err(), "Must reject provisioning when quota exceeded");
    }
}
