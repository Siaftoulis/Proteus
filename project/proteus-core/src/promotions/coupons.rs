// Proteus Sovereign Business OS — Cryptographic Coupon Code Vault & Redundancy Guard
// 100% Original Codebase — Strictly adheres to Zero Mock Data & Decimal Cent Accounting

use chrono::{Duration, Utc};
use rand::Rng;
use rusqlite::{params, Connection, OptionalExtension, Row};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::types::{Coupon, CouponError, CouponRedemption, DiscountType};

pub fn normalize_coupon_code(raw: &str) -> String {
    raw.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_uppercase()
}

pub fn hash_coupon_code(raw_code: &str) -> String {
    let clean = normalize_coupon_code(raw_code);
    let mut hasher = Sha256::new();
    hasher.update(clean.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn init_coupon_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS coupon_vault (
            id TEXT PRIMARY KEY,
            code_hash TEXT NOT NULL UNIQUE,
            code_display TEXT NOT NULL,
            discount_json TEXT NOT NULL,
            min_spend_cents INTEGER NOT NULL DEFAULT 0,
            usage_limit_total INTEGER,
            usage_limit_per_customer INTEGER NOT NULL DEFAULT 1,
            usage_count INTEGER NOT NULL DEFAULT 0,
            valid_from TEXT NOT NULL,
            valid_until TEXT,
            is_active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL
        );
        CREATE UNIQUE INDEX IF NOT EXISTS idx_coupon_hash ON coupon_vault(code_hash);

        CREATE TABLE IF NOT EXISTS coupon_redemptions (
            id TEXT PRIMARY KEY,
            coupon_id TEXT NOT NULL,
            customer_identifier TEXT NOT NULL,
            receipt_id TEXT,
            discount_applied_cents INTEGER NOT NULL,
            redeemed_at TEXT NOT NULL,
            FOREIGN KEY (coupon_id) REFERENCES coupon_vault(id)
        );
        CREATE INDEX IF NOT EXISTS idx_coupon_customer ON coupon_redemptions(coupon_id, customer_identifier);
        "#,
    )?;
    Ok(())
}

fn map_coupon_row(row: &Row) -> rusqlite::Result<Coupon> {
    let disc_json: String = row.get(3)?;
    let discount: DiscountType = serde_json::from_str(&disc_json)
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(3, rusqlite::types::Type::Text, Box::new(e)))?;
    let active_int: i64 = row.get(10)?;

    Ok(Coupon {
        id: row.get(0)?,
        code_hash: row.get(1)?,
        code_display: row.get(2)?,
        discount,
        min_spend_cents: row.get(4)?,
        usage_limit_total: row.get(5)?,
        usage_limit_per_customer: row.get(6)?,
        usage_count: row.get(7)?,
        valid_from: row.get(8)?,
        valid_until: row.get(9)?,
        is_active: active_int == 1,
        created_at: row.get(11)?,
    })
}

pub fn create_coupon(
    conn: &Connection,
    raw_code: &str,
    discount: DiscountType,
    min_spend_cents: i64,
    usage_limit_total: Option<u32>,
    usage_limit_per_customer: u32,
    valid_from: &str,
    valid_until: Option<&str>,
) -> rusqlite::Result<Coupon> {
    let code_hash = hash_coupon_code(raw_code);
    let code_display = raw_code.trim().to_uppercase();
    let disc_json = serde_json::to_string(&discount).unwrap_or_default();
    let now = Utc::now().to_rfc3339();

    let coupon = Coupon {
        id: format!("cpn_{}", Uuid::now_v7()),
        code_hash: code_hash.clone(),
        code_display: code_display.clone(),
        discount,
        min_spend_cents,
        usage_limit_total,
        usage_limit_per_customer,
        usage_count: 0,
        valid_from: valid_from.to_string(),
        valid_until: valid_until.map(String::from),
        is_active: true,
        created_at: now.clone(),
    };

    conn.execute(
        r#"INSERT INTO coupon_vault (
            id, code_hash, code_display, discount_json, min_spend_cents,
            usage_limit_total, usage_limit_per_customer, usage_count,
            valid_from, valid_until, is_active, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)"#,
        params![
            coupon.id, code_hash, code_display, disc_json, coupon.min_spend_cents,
            coupon.usage_limit_total, coupon.usage_limit_per_customer, coupon.usage_count,
            coupon.valid_from, coupon.valid_until, 1, coupon.created_at,
        ],
    )?;

    Ok(coupon)
}

pub fn generate_single_use_coupon(
    conn: &Connection,
    prefix: &str,
    discount: DiscountType,
    min_spend_cents: i64,
    valid_days: Option<u32>,
) -> rusqlite::Result<(Coupon, String)> {
    let mut rng = rand::thread_rng();
    let alphabet: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let c1: String = (0..4).map(|_| alphabet[rng.gen_range(0..alphabet.len())] as char).collect();
    let c2: String = (0..4).map(|_| alphabet[rng.gen_range(0..alphabet.len())] as char).collect();
    let raw_code = format!("{}-{}-{}", prefix.trim().to_uppercase(), c1, c2);

    let now = Utc::now();
    let now_str = now.to_rfc3339();
    let valid_until = valid_days.map(|d| (now + Duration::days(d as i64)).to_rfc3339());

    let coupon = create_coupon(
        conn, &raw_code, discount, min_spend_cents, Some(1), 1, &now_str, valid_until.as_deref(),
    )?;

    Ok((coupon, raw_code))
}

pub fn lookup_coupon(conn: &Connection, raw_code: &str) -> rusqlite::Result<Option<Coupon>> {
    let hash = hash_coupon_code(raw_code);
    conn.query_row(
        r#"SELECT id, code_hash, code_display, discount_json, min_spend_cents,
                  usage_limit_total, usage_limit_per_customer, usage_count,
                  valid_from, valid_until, is_active, created_at
           FROM coupon_vault WHERE code_hash = ?1"#,
        params![hash],
        map_coupon_row,
    )
    .optional()
}

pub fn validate_coupon(
    conn: &Connection,
    raw_code: &str,
    customer_identifier: &str,
    cart_gross_cents: i64,
    current_time_iso: &str,
) -> Result<Coupon, CouponError> {
    let coupon = lookup_coupon(conn, raw_code)?.ok_or(CouponError::NotFound)?;

    if !coupon.is_active {
        return Err(CouponError::Inactive);
    }
    if current_time_iso < coupon.valid_from.as_str() {
        return Err(CouponError::Expired);
    }
    if let Some(ref until) = coupon.valid_until {
        if current_time_iso > until.as_str() {
            return Err(CouponError::Expired);
        }
    }
    if cart_gross_cents < coupon.min_spend_cents {
        return Err(CouponError::MinSpendNotMet {
            min_spend_cents: coupon.min_spend_cents,
            actual_cents: cart_gross_cents,
        });
    }
    if let Some(limit) = coupon.usage_limit_total {
        if coupon.usage_count >= limit {
            return Err(CouponError::TotalUsageLimitExceeded(limit));
        }
    }

    let clean_cust = customer_identifier.trim();
    if !clean_cust.is_empty() {
        let cust_usage: u32 = conn.query_row(
            "SELECT count(*) FROM coupon_redemptions WHERE coupon_id = ?1 AND customer_identifier = ?2",
            params![coupon.id, clean_cust],
            |r| r.get(0),
        )?;

        if cust_usage >= coupon.usage_limit_per_customer {
            return Err(CouponError::CustomerUsageLimitExceeded {
                used: cust_usage,
                limit: coupon.usage_limit_per_customer,
            });
        }
    }

    Ok(coupon)
}

pub fn redeem_coupon(
    conn: &Connection,
    raw_code: &str,
    customer_identifier: &str,
    receipt_id: Option<&str>,
    cart_gross_cents: i64,
    current_time_iso: &str,
) -> Result<(CouponRedemption, i64), CouponError> {
    let coupon = validate_coupon(conn, raw_code, customer_identifier, cart_gross_cents, current_time_iso)?;

    let discount_cents = match coupon.discount {
        DiscountType::PercentageBasisPoints(bp) => (cart_gross_cents * bp as i64) / 10000,
        DiscountType::FixedAmountCents(amt) => amt.min(cart_gross_cents),
        DiscountType::BuyXGetYFree { .. } => 0,
    };

    let redemption = CouponRedemption {
        id: format!("red_{}", Uuid::now_v7()),
        coupon_id: coupon.id.clone(),
        customer_identifier: customer_identifier.trim().to_string(),
        receipt_id: receipt_id.map(String::from),
        discount_applied_cents: discount_cents,
        redeemed_at: Utc::now().to_rfc3339(),
    };

    conn.execute("UPDATE coupon_vault SET usage_count = usage_count + 1 WHERE id = ?1", params![coupon.id])?;
    conn.execute(
        r#"INSERT INTO coupon_redemptions (
            id, coupon_id, customer_identifier, receipt_id, discount_applied_cents, redeemed_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)"#,
        params![
            redemption.id, redemption.coupon_id, redemption.customer_identifier,
            redemption.receipt_id, redemption.discount_applied_cents, redemption.redeemed_at,
        ],
    )?;

    Ok((redemption, discount_cents))
}

pub fn get_coupon_redemptions(
    conn: &Connection,
    coupon_id: &str,
    limit: usize,
) -> rusqlite::Result<Vec<CouponRedemption>> {
    let mut stmt = conn.prepare(
        r#"SELECT id, coupon_id, customer_identifier, receipt_id, discount_applied_cents, redeemed_at
           FROM coupon_redemptions WHERE coupon_id = ?1 ORDER BY redeemed_at DESC LIMIT ?2"#,
    )?;

    let entries = stmt
        .query_map(params![coupon_id, limit as i64], |row| {
            Ok(CouponRedemption {
                id: row.get(0)?,
                coupon_id: row.get(1)?,
                customer_identifier: row.get(2)?,
                receipt_id: row.get(3)?,
                discount_applied_cents: row.get(4)?,
                redeemed_at: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coupon_lifecycle_and_customer_redundancy_guard() {
        let conn = Connection::open_in_memory().unwrap();
        init_coupon_schema(&conn).unwrap();

        let coupon = create_coupon(
            &conn,
            "WELCOME20",
            DiscountType::PercentageBasisPoints(2000), // 20.00%
            5000,
            Some(10), // Max 10 total
            1,        // Max 1 per customer
            "2026-01-01T00:00:00Z",
            Some("2026-12-31T23:59:59Z"),
        )
        .unwrap();

        // 1. Min spend test
        let fail_min = validate_coupon(&conn, "welcome20", "+306912345678", 3000, "2026-10-09T10:00:00Z");
        assert!(matches!(fail_min, Err(CouponError::MinSpendNotMet { .. })));

        // 2. Valid cart
        let valid = validate_coupon(&conn, "WELCOME-20", "+306912345678", 10000, "2026-10-09T10:00:00Z");
        assert!(valid.is_ok());

        // 3. Redeem
        let (redemption, discount) = redeem_coupon(
            &conn, "WELCOME20", "+306912345678", Some("REC-101"), 10000, "2026-10-09T10:00:00Z",
        )
        .unwrap();
        assert_eq!(discount, 2000);
        assert_eq!(redemption.coupon_id, coupon.id);

        // 4. Redundancy Guard
        let second_try = validate_coupon(&conn, "WELCOME20", "+306912345678", 10000, "2026-10-09T10:05:00Z");
        assert!(matches!(second_try, Err(CouponError::CustomerUsageLimitExceeded { used: 1, limit: 1 })));

        // 5. Different customer
        let other_customer = validate_coupon(&conn, "WELCOME20", "+306999999999", 10000, "2026-10-09T10:05:00Z");
        assert!(other_customer.is_ok());
    }

    #[test]
    fn test_single_use_generated_coupon() {
        let conn = Connection::open_in_memory().unwrap();
        init_coupon_schema(&conn).unwrap();

        let (_coupon, raw_code) = generate_single_use_coupon(
            &conn, "REPAIR", DiscountType::FixedAmountCents(1000), 2000, Some(30),
        )
        .unwrap();

        assert!(raw_code.starts_with("REPAIR-"));

        let res1 = redeem_coupon(&conn, &raw_code, "CUST-A", None, 5000, "2026-10-09T10:00:00Z");
        assert!(res1.is_ok());

        let res2 = redeem_coupon(&conn, &raw_code, "CUST-B", None, 5000, "2026-10-09T10:01:00Z");
        assert!(matches!(res2, Err(CouponError::TotalUsageLimitExceeded(1))));
    }
}
