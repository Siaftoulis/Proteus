//! B2B Dealer Registry & Anti-Race Time-Locked Stock Reservation Engine.
//! Strict Rule 1 (100% Original Codebase), Rule 3 (<400 lines), and Rule 5 (Zero Mock Data).
//! Enables cross-store dealer federations, shared inventory feeds, and prevents double-selling
//! across physical POS and webstore via atomic time-locked stock reservations (15-min TTL).

use rusqlite::{params, Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReservationStatus {
    Held,
    Confirmed,
    Released,
    Expired,
}

impl ReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Held => "HELD",
            Self::Confirmed => "CONFIRMED",
            Self::Released => "RELEASED",
            Self::Expired => "EXPIRED",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "CONFIRMED" => Self::Confirmed,
            "RELEASED" => Self::Released,
            "EXPIRED" => Self::Expired,
            _ => Self::Held,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct B2bDealer {
    pub dealer_id: String,
    pub legal_name: String,
    pub afm: String,
    pub contact_email: String,
    pub endpoint_url: Option<String>,
    pub p2b_terms_version: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SharedItem {
    pub sku: String,
    pub dealer_id: String,
    pub title: String,
    pub wholesale_price: f64,
    pub available_quantity: u32,
    pub reserved_quantity: u32,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StockReservation {
    pub reservation_id: String,
    pub dealer_id: String,
    pub sku: String,
    pub quantity: u32,
    pub cart_session_id: String,
    pub expires_at: String,
    pub status: ReservationStatus,
    pub created_at: String,
}

/// Initializes SQLite tables for B2B dealers and time-locked stock reservations.
pub fn init_b2b_reservation_tables(conn: &Connection) -> SqlResult<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS b2b_dealers (
            dealer_id TEXT PRIMARY KEY,
            legal_name TEXT NOT NULL,
            afm TEXT NOT NULL,
            contact_email TEXT NOT NULL,
            endpoint_url TEXT,
            p2b_terms_version TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'ACTIVE',
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS b2b_shared_inventory (
            sku TEXT PRIMARY KEY,
            dealer_id TEXT NOT NULL REFERENCES b2b_dealers(dealer_id),
            title TEXT NOT NULL,
            wholesale_price REAL NOT NULL,
            available_quantity INTEGER NOT NULL DEFAULT 0,
            reserved_quantity INTEGER NOT NULL DEFAULT 0,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS b2b_stock_reservations (
            reservation_id TEXT PRIMARY KEY,
            dealer_id TEXT NOT NULL REFERENCES b2b_dealers(dealer_id),
            sku TEXT NOT NULL REFERENCES b2b_shared_inventory(sku),
            quantity INTEGER NOT NULL,
            cart_session_id TEXT NOT NULL,
            expires_at TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'HELD',
            created_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_b2b_inventory_dealer ON b2b_shared_inventory(dealer_id);
        CREATE INDEX IF NOT EXISTS idx_b2b_reservations_status ON b2b_stock_reservations(status, expires_at);",
    )
}

/// Registers a partner dealer into the B2B federation.
pub fn register_dealer(conn: &Connection, dealer: &B2bDealer) -> SqlResult<()> {
    conn.execute(
        "INSERT INTO b2b_dealers (dealer_id, legal_name, afm, contact_email, endpoint_url, p2b_terms_version, status, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            dealer.dealer_id,
            dealer.legal_name,
            dealer.afm,
            dealer.contact_email,
            dealer.endpoint_url,
            dealer.p2b_terms_version,
            dealer.status,
            dealer.created_at,
        ],
    )?;
    Ok(())
}

/// Upserts an inventory item published by a partner dealer.
pub fn upsert_shared_inventory_item(conn: &Connection, item: &SharedItem) -> SqlResult<()> {
    conn.execute(
        "INSERT INTO b2b_shared_inventory (
            sku, dealer_id, title, wholesale_price, available_quantity, reserved_quantity, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        ON CONFLICT(sku) DO UPDATE SET
            title = excluded.title,
            wholesale_price = excluded.wholesale_price,
            available_quantity = excluded.available_quantity,
            updated_at = excluded.updated_at",
        params![
            item.sku,
            item.dealer_id,
            item.title,
            item.wholesale_price,
            item.available_quantity,
            item.reserved_quantity,
            item.updated_at,
        ],
    )?;
    Ok(())
}

/// Atomically reserves stock for a customer cart session with a monotonic timeout (TTL).
pub fn reserve_stock(
    conn: &mut Connection,
    dealer_id: &str,
    sku: &str,
    quantity: u32,
    cart_session_id: &str,
    ttl_seconds: u64,
) -> Result<StockReservation, String> {
    if quantity == 0 {
        return Err("Reservation quantity must be greater than zero".to_string());
    }

    let tx = conn.transaction().map_err(|e| e.to_string())?;

    // Check available unreserved quantity
    let (available, reserved): (u32, u32) = tx.query_row(
        "SELECT available_quantity, reserved_quantity FROM b2b_shared_inventory WHERE sku = ?1 AND dealer_id = ?2",
        params![sku, dealer_id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    ).map_err(|e| format!("Item not found or unavailable: {}", e))?;

    if available < reserved + quantity {
        return Err(format!("Insufficient unreserved stock: available {}, reserved {}, requested {}", available, reserved, quantity));
    }

    // Atomically increment reserved quantity
    tx.execute(
        "UPDATE b2b_shared_inventory SET reserved_quantity = reserved_quantity + ?1 WHERE sku = ?2",
        params![quantity, sku],
    ).map_err(|e| e.to_string())?;

    let now = chrono::Utc::now();
    let expires = now + chrono::Duration::seconds(ttl_seconds as i64);
    let reservation_id = format!("RES-{}", uuid::Uuid::new_v4());

    tx.execute(
        "INSERT INTO b2b_stock_reservations (
            reservation_id, dealer_id, sku, quantity, cart_session_id, expires_at, status, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'HELD', ?7)",
        params![
            reservation_id,
            dealer_id,
            sku,
            quantity,
            cart_session_id,
            expires.to_rfc3339(),
            now.to_rfc3339(),
        ],
    ).map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;

    Ok(StockReservation {
        reservation_id,
        dealer_id: dealer_id.to_string(),
        sku: sku.to_string(),
        quantity,
        cart_session_id: cart_session_id.to_string(),
        expires_at: expires.to_rfc3339(),
        status: ReservationStatus::Held,
        created_at: now.to_rfc3339(),
    })
}

/// Finalizes checkout: marks reservation as CONFIRMED and decrements both available and reserved stock.
pub fn confirm_reservation(conn: &mut Connection, reservation_id: &str) -> Result<(), String> {
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let (sku, qty, status): (String, u32, String) = tx.query_row(
        "SELECT sku, quantity, status FROM b2b_stock_reservations WHERE reservation_id = ?1",
        params![reservation_id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    ).map_err(|e| format!("Reservation not found: {}", e))?;

    if status != "HELD" {
        return Err(format!("Cannot confirm reservation in '{}' status", status));
    }

    // Deduct stock permanently from available & release the hold
    tx.execute(
        "UPDATE b2b_shared_inventory
         SET available_quantity = CASE WHEN available_quantity >= ?1 THEN available_quantity - ?1 ELSE 0 END,
             reserved_quantity = CASE WHEN reserved_quantity >= ?1 THEN reserved_quantity - ?1 ELSE 0 END
         WHERE sku = ?2",
        params![qty, sku],
    ).map_err(|e| e.to_string())?;

    tx.execute(
        "UPDATE b2b_stock_reservations SET status = 'CONFIRMED' WHERE reservation_id = ?1",
        params![reservation_id],
    ).map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

/// Releases an active hold back to available stock.
pub fn release_reservation(conn: &mut Connection, reservation_id: &str) -> Result<(), String> {
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let (sku, qty, status): (String, u32, String) = tx.query_row(
        "SELECT sku, quantity, status FROM b2b_stock_reservations WHERE reservation_id = ?1",
        params![reservation_id],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    ).map_err(|e| format!("Reservation not found: {}", e))?;

    if status != "HELD" {
        return Err(format!("Reservation is already '{}'", status));
    }

    tx.execute(
        "UPDATE b2b_shared_inventory
         SET reserved_quantity = CASE WHEN reserved_quantity >= ?1 THEN reserved_quantity - ?1 ELSE 0 END
         WHERE sku = ?2",
        params![qty, sku],
    ).map_err(|e| e.to_string())?;

    tx.execute(
        "UPDATE b2b_stock_reservations SET status = 'RELEASED' WHERE reservation_id = ?1",
        params![reservation_id],
    ).map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

/// Reaps all expired reservations whose TTL has elapsed and restores reserved stock.
pub fn reap_expired_reservations(conn: &mut Connection) -> Result<usize, String> {
    let now = chrono::Utc::now().to_rfc3339();
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let mut expired = Vec::new();
    {
        let mut stmt = tx.prepare(
            "SELECT reservation_id, sku, quantity FROM b2b_stock_reservations
             WHERE status = 'HELD' AND expires_at < ?1"
        ).map_err(|e| e.to_string())?;

        let rows = stmt.query_map(params![now], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, u32>(2)?))
        }).map_err(|e| e.to_string())?;

        for r in rows.flatten() {
            expired.push(r);
        }
    }

    let count = expired.len();
    for (res_id, sku, qty) in expired {
        tx.execute(
            "UPDATE b2b_shared_inventory
             SET reserved_quantity = CASE WHEN reserved_quantity >= ?1 THEN reserved_quantity - ?1 ELSE 0 END
             WHERE sku = ?2",
            params![qty, sku],
        ).map_err(|e| e.to_string())?;

        tx.execute(
            "UPDATE b2b_stock_reservations SET status = 'EXPIRED' WHERE reservation_id = ?1",
            params![res_id],
        ).map_err(|e| e.to_string())?;
    }

    tx.commit().map_err(|e| e.to_string())?;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_b2b_stock_reservation_lifecycle() {
        let mut conn = Connection::open_in_memory().unwrap();
        init_b2b_reservation_tables(&conn).unwrap();

        let dealer = B2bDealer {
            dealer_id: "dlr_patras".to_string(),
            legal_name: "Patras Industrial Supply".to_string(),
            afm: "094123456".to_string(),
            contact_email: "b2b@patras-supply.gr".to_string(),
            endpoint_url: Some("https://patras-supply.gr/api".to_string()),
            p2b_terms_version: "2026.1".to_string(),
            status: "ACTIVE".to_string(),
            created_at: "2026-10-14T10:00:00Z".to_string(),
        };
        register_dealer(&conn, &dealer).unwrap();

        let item = SharedItem {
            sku: "IND-WRENCH-01".to_string(),
            dealer_id: "dlr_patras".to_string(),
            title: "Torque Wrench 1/2".to_string(),
            wholesale_price: 45.0,
            available_quantity: 10,
            reserved_quantity: 0,
            updated_at: "2026-10-14T10:00:00Z".to_string(),
        };
        upsert_shared_inventory_item(&conn, &item).unwrap();

        // 1. Reserve 3 items
        let res = reserve_stock(&mut conn, "dlr_patras", "IND-WRENCH-01", 3, "sess_cart_abc", 900).unwrap();
        assert_eq!(res.quantity, 3);
        assert_eq!(res.status, ReservationStatus::Held);

        // 2. Reject over-reservation
        let err = reserve_stock(&mut conn, "dlr_patras", "IND-WRENCH-01", 8, "sess_cart_xyz", 900);
        assert!(err.is_err());

        // 3. Confirm reservation
        confirm_reservation(&mut conn, &res.reservation_id).unwrap();

        let (avail, resv): (u32, u32) = conn.query_row(
            "SELECT available_quantity, reserved_quantity FROM b2b_shared_inventory WHERE sku = 'IND-WRENCH-01'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        ).unwrap();

        assert_eq!(avail, 7);
        assert_eq!(resv, 0);
    }

    #[test]
    fn test_expired_stock_reservation_reaping() {
        let mut conn = Connection::open_in_memory().unwrap();
        init_b2b_reservation_tables(&conn).unwrap();

        let dealer = B2bDealer {
            dealer_id: "dlr_larissa".to_string(),
            legal_name: "Larissa Tools".to_string(),
            afm: "099888777".to_string(),
            contact_email: "info@larissa-tools.gr".to_string(),
            endpoint_url: None,
            p2b_terms_version: "2026.1".to_string(),
            status: "ACTIVE".to_string(),
            created_at: "2026-10-14T10:00:00Z".to_string(),
        };
        register_dealer(&conn, &dealer).unwrap();

        let item = SharedItem {
            sku: "DRILL-01".to_string(),
            dealer_id: "dlr_larissa".to_string(),
            title: "Hammer Drill 800W".to_string(),
            wholesale_price: 60.0,
            available_quantity: 5,
            reserved_quantity: 0,
            updated_at: "2026-10-14T10:00:00Z".to_string(),
        };
        upsert_shared_inventory_item(&conn, &item).unwrap();

        // Reserve with 0-second TTL (instant expiration)
        let res = reserve_stock(&mut conn, "dlr_larissa", "DRILL-01", 2, "sess_quick", 0).unwrap();

        // Sleep 10ms to ensure now > expires_at
        std::thread::sleep(std::time::Duration::from_millis(15));

        let reaped = reap_expired_reservations(&mut conn).unwrap();
        assert_eq!(reaped, 1);

        let resv: u32 = conn.query_row(
            "SELECT reserved_quantity FROM b2b_shared_inventory WHERE sku = 'DRILL-01'",
            [],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(resv, 0);

        let status: String = conn.query_row(
            "SELECT status FROM b2b_stock_reservations WHERE reservation_id = ?1",
            [&res.reservation_id],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(status, "EXPIRED");
    }
}
