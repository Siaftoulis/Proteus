//! Anti-SAP Multi-Tenant & Hierarchical Multi-Store Engine.
//! Manages Enterprise HQ, store branches, departments, and delegated user quotas.

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
    pub dept_code: String, // "WAREHOUSE", "MOBILE", "BOOKS", "SERVICE", "POS"
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
            enterprise_id TEXT PRIMARY KEY,
            legal_name TEXT NOT NULL,
            tax_id TEXT NOT NULL UNIQUE,
            total_seat_quota INTEGER NOT NULL DEFAULT 10,
            used_seats INTEGER NOT NULL DEFAULT 0,
            cloud_tier TEXT NOT NULL DEFAULT 'EnterpriseCore',
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS enterprise_stores (
            store_id TEXT PRIMARY KEY,
            enterprise_id TEXT NOT NULL REFERENCES enterprises(enterprise_id),
            store_code TEXT NOT NULL UNIQUE,
            store_name TEXT NOT NULL,
            address TEXT NOT NULL,
            phone TEXT NOT NULL,
            allocated_seats INTEGER NOT NULL DEFAULT 5,
            active_seats INTEGER NOT NULL DEFAULT 0,
            is_active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS store_departments (
            department_id TEXT PRIMARY KEY,
            store_id TEXT NOT NULL REFERENCES enterprise_stores(store_id),
            dept_code TEXT NOT NULL,
            name TEXT NOT NULL,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS enterprise_users (
            user_id TEXT PRIMARY KEY,
            enterprise_id TEXT NOT NULL REFERENCES enterprises(enterprise_id),
            store_id TEXT NOT NULL REFERENCES enterprise_stores(store_id),
            department_id TEXT REFERENCES store_departments(department_id),
            full_name TEXT NOT NULL,
            email TEXT NOT NULL UNIQUE,
            role_code TEXT NOT NULL,
            is_active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_stores_ent ON enterprise_stores(enterprise_id);
        CREATE INDEX IF NOT EXISTS idx_depts_store ON store_departments(store_id);
        CREATE INDEX IF NOT EXISTS idx_users_store ON enterprise_users(store_id);
        "#,
    )?;
    Ok(())
}

/// Seeds a default primary enterprise and flagship stores if none exists.
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

        let store_syntagma = EnterpriseStore {
            store_id: "str-001-syntagma".into(),
            enterprise_id: ent.enterprise_id.clone(),
            store_code: "STR-001-SYNTAGMA".into(),
            store_name: "Κατάστημα 01: Σύνταγμα (Flagship)".into(),
            address: "Πλατεία Συντάγματος 4, Αθήνα".into(),
            phone: "+30 210 3210001".into(),
            allocated_seats: 15,
            active_seats: 8,
            is_active: true,
            created_at: now_iso(),
        };
        create_enterprise_store(conn, &store_syntagma)?;

        create_store_department(conn, &StoreDepartment {
            department_id: "dept-001-wh".into(),
            store_id: store_syntagma.store_id.clone(),
            dept_code: "WAREHOUSE".into(),
            name: "Κεντρική Αποθήκη & Barcode In".into(),
            created_at: now_iso(),
        })?;
        create_store_department(conn, &StoreDepartment {
            department_id: "dept-002-mob".into(),
            store_id: store_syntagma.store_id.clone(),
            dept_code: "MOBILE".into(),
            name: "Τηλεφωνία & Συμβόλαια".into(),
            created_at: now_iso(),
        })?;
        create_store_department(conn, &StoreDepartment {
            department_id: "dept-003-srv".into(),
            store_id: store_syntagma.store_id.clone(),
            dept_code: "SERVICE".into(),
            name: "Τεχνικό Εργαστήριο & Επισκευές".into(),
            created_at: now_iso(),
        })?;
        create_store_department(conn, &StoreDepartment {
            department_id: "dept-004-pos".into(),
            store_id: store_syntagma.store_id.clone(),
            dept_code: "POS".into(),
            name: "Κεντρικό Ταμείο & Λιανική".into(),
            created_at: now_iso(),
        })?;

        let store_thessaloniki = EnterpriseStore {
            store_id: "str-002-tsimiski".into(),
            enterprise_id: ent.enterprise_id.clone(),
            store_code: "STR-002-TSIMISKI".into(),
            store_name: "Κατάστημα 02: Τσιμισκή Θεσσαλονίκη".into(),
            address: "Τσιμισκή 45, Θεσσαλονίκη".into(),
            phone: "+30 2310 220002".into(),
            allocated_seats: 10,
            active_seats: 4,
            is_active: true,
            created_at: now_iso(),
        };
        create_enterprise_store(conn, &store_thessaloniki)?;

        create_store_department(conn, &StoreDepartment {
            department_id: "dept-005-pos".into(),
            store_id: store_thessaloniki.store_id.clone(),
            dept_code: "POS".into(),
            name: "Ταμείο & Λιανική".into(),
            created_at: now_iso(),
        })?;
        create_store_department(conn, &StoreDepartment {
            department_id: "dept-006-srv".into(),
            store_id: store_thessaloniki.store_id.clone(),
            dept_code: "SERVICE".into(),
            name: "Τεχνικό Service".into(),
            created_at: now_iso(),
        })?;
    }
    Ok(())
}

pub fn create_enterprise(conn: &Connection, ent: &Enterprise) -> Result<()> {
    conn.execute(
        "INSERT INTO enterprises (enterprise_id, legal_name, tax_id, total_seat_quota, used_seats, cloud_tier, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            ent.enterprise_id,
            ent.legal_name,
            ent.tax_id,
            ent.total_seat_quota,
            ent.used_seats,
            ent.cloud_tier,
            ent.created_at,
        ],
    )?;
    Ok(())
}

pub fn get_primary_enterprise(conn: &Connection) -> Result<Option<Enterprise>> {
    let mut stmt = conn.prepare(
        "SELECT enterprise_id, legal_name, tax_id, total_seat_quota, used_seats, cloud_tier, created_at
         FROM enterprises LIMIT 1",
    )?;
    let mut rows = stmt.query([])?;
    if let Some(row) = rows.next()? {
        Ok(Some(Enterprise {
            enterprise_id: row.get(0)?,
            legal_name: row.get(1)?,
            tax_id: row.get(2)?,
            total_seat_quota: row.get(3)?,
            used_seats: row.get(4)?,
            cloud_tier: row.get(5)?,
            created_at: row.get(6)?,
        }))
    } else {
        Ok(None)
    }
}

pub fn create_enterprise_store(conn: &Connection, store: &EnterpriseStore) -> Result<()> {
    conn.execute(
        "INSERT INTO enterprise_stores (store_id, enterprise_id, store_code, store_name, address, phone, allocated_seats, active_seats, is_active, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            store.store_id,
            store.enterprise_id,
            store.store_code,
            store.store_name,
            store.address,
            store.phone,
            store.allocated_seats,
            store.active_seats,
            if store.is_active { 1 } else { 0 },
            store.created_at,
        ],
    )?;
    Ok(())
}

pub fn list_enterprise_stores(conn: &Connection, enterprise_id: &str) -> Result<Vec<EnterpriseStore>> {
    let mut stmt = conn.prepare(
        "SELECT store_id, enterprise_id, store_code, store_name, address, phone, allocated_seats, active_seats, is_active, created_at
         FROM enterprise_stores WHERE enterprise_id = ?1 ORDER BY store_code ASC",
    )?;
    let rows = stmt.query_map(params![enterprise_id], |row| {
        let is_act: i64 = row.get(8)?;
        Ok(EnterpriseStore {
            store_id: row.get(0)?,
            enterprise_id: row.get(1)?,
            store_code: row.get(2)?,
            store_name: row.get(3)?,
            address: row.get(4)?,
            phone: row.get(5)?,
            allocated_seats: row.get(6)?,
            active_seats: row.get(7)?,
            is_active: is_act != 0,
            created_at: row.get(9)?,
        })
    })?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn create_store_department(conn: &Connection, dept: &StoreDepartment) -> Result<()> {
    conn.execute(
        "INSERT INTO store_departments (department_id, store_id, dept_code, name, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            dept.department_id,
            dept.store_id,
            dept.dept_code,
            dept.name,
            dept.created_at,
        ],
    )?;
    Ok(())
}

pub fn list_store_departments(conn: &Connection, store_id: &str) -> Result<Vec<StoreDepartment>> {
    let mut stmt = conn.prepare(
        "SELECT department_id, store_id, dept_code, name, created_at
         FROM store_departments WHERE store_id = ?1 ORDER BY name ASC",
    )?;
    let rows = stmt.query_map(params![store_id], |row| {
        Ok(StoreDepartment {
            department_id: row.get(0)?,
            store_id: row.get(1)?,
            dept_code: row.get(2)?,
            name: row.get(3)?,
            created_at: row.get(4)?,
        })
    })?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// Computes the total allocated seats across all branches vs enterprise quota.
pub fn calculate_seat_utilization(conn: &Connection, enterprise_id: &str) -> Result<(i64, i64)> {
    let total_quota: i64 = conn
        .query_row(
            "SELECT total_seat_quota FROM enterprises WHERE enterprise_id = ?1",
            params![enterprise_id],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let allocated_sum: i64 = conn
        .query_row(
            "SELECT COALESCE(SUM(allocated_seats), 0) FROM enterprise_stores WHERE enterprise_id = ?1",
            params![enterprise_id],
            |r| r.get(0),
        )
        .unwrap_or(0);

    Ok((allocated_sum, total_quota))
}

fn now_iso() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("{}", now)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enterprise_schema_creation_and_seeding() {
        let conn = Connection::open_in_memory().unwrap();
        init_enterprise_schema(&conn).unwrap();
        seed_default_enterprise_if_empty(&conn).unwrap();

        let ent = get_primary_enterprise(&conn).unwrap().expect("Enterprise must exist");
        assert_eq!(ent.tax_id, "099887766");
        assert_eq!(ent.total_seat_quota, 30);

        let stores = list_enterprise_stores(&conn, &ent.enterprise_id).unwrap();
        assert_eq!(stores.len(), 2);

        let depts = list_store_departments(&conn, &stores[0].store_id).unwrap();
        assert_eq!(depts.len(), 4);

        let (allocated, total) = calculate_seat_utilization(&conn, &ent.enterprise_id).unwrap();
        assert_eq!(allocated, 25); // 15 + 10
        assert_eq!(total, 30);
    }

    #[test]
    fn test_store_and_department_hierarchy() {
        let conn = Connection::open_in_memory().unwrap();
        init_enterprise_schema(&conn).unwrap();

        let ent = Enterprise {
            enterprise_id: "test-ent".into(),
            legal_name: "Test Chain".into(),
            tax_id: "112233445".into(),
            total_seat_quota: 50,
            used_seats: 0,
            cloud_tier: "EnterpriseCore".into(),
            created_at: now_iso(),
        };
        create_enterprise(&conn, &ent).unwrap();

        let store = EnterpriseStore {
            store_id: "test-str".into(),
            enterprise_id: ent.enterprise_id.clone(),
            store_code: "STR-PATRA".into(),
            store_name: "Κατάστημα Πάτρας".into(),
            address: "Κορίνθου 10".into(),
            phone: "2610123456".into(),
            allocated_seats: 8,
            active_seats: 3,
            is_active: true,
            created_at: now_iso(),
        };
        create_enterprise_store(&conn, &store).unwrap();

        create_store_department(&conn, &StoreDepartment {
            department_id: "dept-p1".into(),
            store_id: store.store_id.clone(),
            dept_code: "POS".into(),
            name: "Ταμείο".into(),
            created_at: now_iso(),
        }).unwrap();

        let stores = list_enterprise_stores(&conn, &ent.enterprise_id).unwrap();
        assert_eq!(stores.len(), 1);
        assert_eq!(stores[0].store_code, "STR-PATRA");

        let depts = list_store_departments(&conn, &store.store_id).unwrap();
        assert_eq!(depts.len(), 1);
        assert_eq!(depts[0].name, "Ταμείο");
    }
}
