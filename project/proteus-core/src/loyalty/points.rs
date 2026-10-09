// Proteus Sovereign Business OS — Bespoke Customer Loyalty & Rewards Engine
// 100% Original Codebase — Strictly adheres to Zero Mock Data & Decimal Cent Accounting

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension, Row};
use uuid::Uuid;

use super::types::{
    LoyaltyAccount, LoyaltyEntryType, LoyaltyError, LoyaltyPointEntry, LoyaltyPolicy, LoyaltyTier,
};

pub fn init_loyalty_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS loyalty_accounts (
            id TEXT PRIMARY KEY,
            customer_phone TEXT NOT NULL,
            card_number TEXT NOT NULL,
            customer_name TEXT NOT NULL,
            tier TEXT NOT NULL DEFAULT 'Bronze',
            points_balance INTEGER NOT NULL DEFAULT 0,
            lifetime_points_earned INTEGER NOT NULL DEFAULT 0,
            lifetime_spend_cents INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_loyalty_phone ON loyalty_accounts(customer_phone);
        CREATE INDEX IF NOT EXISTS idx_loyalty_card ON loyalty_accounts(card_number);

        CREATE TABLE IF NOT EXISTS loyalty_point_ledger (
            id TEXT PRIMARY KEY,
            account_id TEXT NOT NULL,
            entry_type TEXT NOT NULL,
            points INTEGER NOT NULL,
            reference_id TEXT,
            qualifying_amount_cents INTEGER NOT NULL DEFAULT 0,
            multiplier_basis_points INTEGER NOT NULL DEFAULT 10000,
            balance_after INTEGER NOT NULL,
            note TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY (account_id) REFERENCES loyalty_accounts(id)
        );
        CREATE INDEX IF NOT EXISTS idx_loyalty_ledger_account ON loyalty_point_ledger(account_id, created_at);
        "#,
    )?;
    Ok(())
}

fn map_account_row(row: &Row) -> rusqlite::Result<LoyaltyAccount> {
    let tier_str: String = row.get(4)?;
    Ok(LoyaltyAccount {
        id: row.get(0)?,
        customer_phone: row.get(1)?,
        card_number: row.get(2)?,
        customer_name: row.get(3)?,
        tier: LoyaltyTier::parse_str(&tier_str),
        points_balance: row.get(5)?,
        lifetime_points_earned: row.get(6)?,
        lifetime_spend_cents: row.get(7)?,
        created_at: row.get(8)?,
        updated_at: row.get(9)?,
    })
}

pub fn create_or_get_account(
    conn: &Connection,
    customer_phone: &str,
    card_number: &str,
    customer_name: &str,
) -> rusqlite::Result<LoyaltyAccount> {
    if let Some(existing) = find_account_by_phone_or_card(conn, customer_phone)? {
        return Ok(existing);
    }
    if !card_number.is_empty() {
        if let Some(existing) = find_account_by_phone_or_card(conn, card_number)? {
            return Ok(existing);
        }
    }

    let now = Utc::now().to_rfc3339();
    let account = LoyaltyAccount {
        id: format!("loy_{}", Uuid::now_v7()),
        customer_phone: customer_phone.trim().to_string(),
        card_number: card_number.trim().to_string(),
        customer_name: customer_name.trim().to_string(),
        tier: LoyaltyTier::Bronze,
        points_balance: 0,
        lifetime_points_earned: 0,
        lifetime_spend_cents: 0,
        created_at: now.clone(),
        updated_at: now,
    };

    conn.execute(
        r#"INSERT INTO loyalty_accounts (
            id, customer_phone, card_number, customer_name, tier,
            points_balance, lifetime_points_earned, lifetime_spend_cents, created_at, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"#,
        params![
            account.id, account.customer_phone, account.card_number, account.customer_name,
            account.tier.as_str(), account.points_balance, account.lifetime_points_earned,
            account.lifetime_spend_cents, account.created_at, account.updated_at,
        ],
    )?;

    Ok(account)
}

pub fn find_account_by_phone_or_card(
    conn: &Connection,
    identifier: &str,
) -> rusqlite::Result<Option<LoyaltyAccount>> {
    let clean = identifier.trim();
    if clean.is_empty() {
        return Ok(None);
    }

    conn.query_row(
        r#"SELECT id, customer_phone, card_number, customer_name, tier,
                  points_balance, lifetime_points_earned, lifetime_spend_cents, created_at, updated_at
           FROM loyalty_accounts WHERE customer_phone = ?1 OR card_number = ?1 LIMIT 1"#,
        params![clean],
        map_account_row,
    )
    .optional()
}

pub fn get_account(conn: &Connection, account_id: &str) -> rusqlite::Result<Option<LoyaltyAccount>> {
    conn.query_row(
        r#"SELECT id, customer_phone, card_number, customer_name, tier,
                  points_balance, lifetime_points_earned, lifetime_spend_cents, created_at, updated_at
           FROM loyalty_accounts WHERE id = ?1"#,
        params![account_id],
        map_account_row,
    )
    .optional()
}

pub fn accrue_points(
    conn: &Connection,
    account_id: &str,
    spend_cents: i64,
    reference_id: Option<&str>,
    note: &str,
    policy: &LoyaltyPolicy,
) -> Result<(LoyaltyAccount, LoyaltyPointEntry), LoyaltyError> {
    if spend_cents <= 0 {
        return Err(LoyaltyError::InvalidParameter("Spend cents must be positive".into()));
    }

    let mut account = get_account(conn, account_id)?
        .ok_or_else(|| LoyaltyError::AccountNotFound(account_id.to_string()))?;

    let base_rate = policy.cents_per_point.max(1) as i64;
    let base_points = spend_cents / base_rate;
    let multiplier = account.tier.multiplier_basis_points() as i64;
    let earned_points = ((base_points * multiplier) / 10000).max(0);

    account.points_balance += earned_points;
    account.lifetime_points_earned += earned_points;
    account.lifetime_spend_cents += spend_cents;
    account.tier = LoyaltyTier::from_lifetime_points(account.lifetime_points_earned, policy);
    account.updated_at = Utc::now().to_rfc3339();

    let entry = LoyaltyPointEntry {
        id: format!("lpe_{}", Uuid::now_v7()),
        account_id: account.id.clone(),
        entry_type: LoyaltyEntryType::Accrual,
        points: earned_points,
        reference_id: reference_id.map(|s| s.to_string()),
        qualifying_amount_cents: spend_cents,
        multiplier_basis_points: account.tier.multiplier_basis_points(),
        balance_after: account.points_balance,
        note: note.to_string(),
        created_at: account.updated_at.clone(),
    };

    conn.execute(
        r#"UPDATE loyalty_accounts
           SET tier = ?1, points_balance = ?2, lifetime_points_earned = ?3,
               lifetime_spend_cents = ?4, updated_at = ?5 WHERE id = ?6"#,
        params![
            account.tier.as_str(), account.points_balance, account.lifetime_points_earned,
            account.lifetime_spend_cents, account.updated_at, account.id,
        ],
    )?;

    conn.execute(
        r#"INSERT INTO loyalty_point_ledger (
            id, account_id, entry_type, points, reference_id,
            qualifying_amount_cents, multiplier_basis_points, balance_after, note, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"#,
        params![
            entry.id, entry.account_id, entry.entry_type.as_str(), entry.points,
            entry.reference_id, entry.qualifying_amount_cents, entry.multiplier_basis_points,
            entry.balance_after, entry.note, entry.created_at,
        ],
    )?;

    Ok((account, entry))
}

pub fn redeem_points(
    conn: &Connection,
    account_id: &str,
    points_to_redeem: i64,
    reference_id: Option<&str>,
    note: &str,
    policy: &LoyaltyPolicy,
) -> Result<(LoyaltyAccount, LoyaltyPointEntry, i64), LoyaltyError> {
    if points_to_redeem <= 0 {
        return Err(LoyaltyError::InvalidParameter("Redemption points must be positive".into()));
    }

    let mut account = get_account(conn, account_id)?
        .ok_or_else(|| LoyaltyError::AccountNotFound(account_id.to_string()))?;

    if account.points_balance < points_to_redeem {
        return Err(LoyaltyError::InsufficientPoints {
            requested: points_to_redeem,
            available: account.points_balance,
        });
    }

    let discount_cents = points_to_redeem * (policy.redemption_cents_per_point as i64);
    account.points_balance -= points_to_redeem;
    account.updated_at = Utc::now().to_rfc3339();

    let entry = LoyaltyPointEntry {
        id: format!("lpe_{}", Uuid::now_v7()),
        account_id: account.id.clone(),
        entry_type: LoyaltyEntryType::Redemption,
        points: -points_to_redeem,
        reference_id: reference_id.map(|s| s.to_string()),
        qualifying_amount_cents: 0,
        multiplier_basis_points: 10000,
        balance_after: account.points_balance,
        note: note.to_string(),
        created_at: account.updated_at.clone(),
    };

    conn.execute(
        "UPDATE loyalty_accounts SET points_balance = ?1, updated_at = ?2 WHERE id = ?3",
        params![account.points_balance, account.updated_at, account.id],
    )?;

    conn.execute(
        r#"INSERT INTO loyalty_point_ledger (
            id, account_id, entry_type, points, reference_id,
            qualifying_amount_cents, multiplier_basis_points, balance_after, note, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"#,
        params![
            entry.id, entry.account_id, entry.entry_type.as_str(), entry.points,
            entry.reference_id, entry.qualifying_amount_cents, entry.multiplier_basis_points,
            entry.balance_after, entry.note, entry.created_at,
        ],
    )?;

    Ok((account, entry, discount_cents))
}

pub fn get_account_ledger(
    conn: &Connection,
    account_id: &str,
    limit: usize,
) -> rusqlite::Result<Vec<LoyaltyPointEntry>> {
    let mut stmt = conn.prepare(
        r#"SELECT id, account_id, entry_type, points, reference_id,
                  qualifying_amount_cents, multiplier_basis_points, balance_after, note, created_at
           FROM loyalty_point_ledger WHERE account_id = ?1 ORDER BY created_at DESC LIMIT ?2"#,
    )?;

    let entries = stmt
        .query_map(params![account_id, limit as i64], |row| {
            let entry_type_str: String = row.get(2)?;
            Ok(LoyaltyPointEntry {
                id: row.get(0)?,
                account_id: row.get(1)?,
                entry_type: LoyaltyEntryType::parse_str(&entry_type_str),
                points: row.get(3)?,
                reference_id: row.get(4)?,
                qualifying_amount_cents: row.get(5)?,
                multiplier_basis_points: row.get(6)?,
                balance_after: row.get(7)?,
                note: row.get(8)?,
                created_at: row.get(9)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loyalty_lifecycle_and_tier_progression() {
        let conn = Connection::open_in_memory().unwrap();
        init_loyalty_schema(&conn).unwrap();
        let policy = LoyaltyPolicy::default();

        let account = create_or_get_account(&conn, "+306912345678", "CARD-1001", "Nikos Kazantzakis").unwrap();
        assert_eq!(account.tier, LoyaltyTier::Bronze);
        assert_eq!(account.points_balance, 0);

        // Spend €200.00 (20,000 cents) -> 200 base points (Bronze multiplier 1.0x = 200 pts)
        let (acc, _) = accrue_points(&conn, &account.id, 20_000, Some("REC-001"), "Counter sale", &policy).unwrap();
        assert_eq!(acc.points_balance, 200);
        assert_eq!(acc.tier, LoyaltyTier::Bronze);

        // Spend €350.00 (35,000 cents) -> 350 base points. Total points earned = 550. Tier becomes Silver!
        let (acc, _) = accrue_points(&conn, &account.id, 35_000, Some("REC-002"), "Workshop repair", &policy).unwrap();
        assert_eq!(acc.points_balance, 550);
        assert_eq!(acc.tier, LoyaltyTier::Silver);

        // Silver tier earns 1.25x points! Spend €100.00 (10,000 cents) -> 100 * 1.25 = 125 pts
        let (acc, _) = accrue_points(&conn, &account.id, 10_000, Some("REC-003"), "Accessory purchase", &policy).unwrap();
        assert_eq!(acc.points_balance, 675);

        // Redeem 200 points -> discount = 200 * 5 cents = 1,000 cents (€10.00)
        let (acc, _, discount_cents) = redeem_points(&conn, &account.id, 200, Some("REC-004"), "Points redemption", &policy).unwrap();
        assert_eq!(acc.points_balance, 475);
        assert_eq!(discount_cents, 1000);

        // Check ledger history
        let ledger = get_account_ledger(&conn, &account.id, 10).unwrap();
        assert_eq!(ledger.len(), 4);
        assert_eq!(ledger[0].entry_type, LoyaltyEntryType::Redemption);
        assert_eq!(ledger[0].points, -200);

        // Lookup by card number
        let by_card = find_account_by_phone_or_card(&conn, "CARD-1001").unwrap().unwrap();
        assert_eq!(by_card.id, account.id);
        assert_eq!(by_card.points_balance, 475);
    }

    #[test]
    fn test_insufficient_points_error() {
        let conn = Connection::open_in_memory().unwrap();
        init_loyalty_schema(&conn).unwrap();
        let policy = LoyaltyPolicy::default();

        let account = create_or_get_account(&conn, "+306988888888", "CARD-2002", "Eleni").unwrap();
        let err = redeem_points(&conn, &account.id, 50, None, "Fail redeem", &policy);
        assert!(matches!(err, Err(LoyaltyError::InsufficientPoints { .. })));
    }
}
