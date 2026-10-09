// Proteus Sovereign Business OS — Cryptographic Gift Card & Prepaid Store Credit Vault
// 100% Original Codebase — Strictly adheres to Zero Mock Data & Decimal Cent Accounting

use chrono::{Duration, Utc};
use rand::Rng;
use rusqlite::{params, Connection, OptionalExtension, Row};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use super::types::{GiftCard, GiftCardError, GiftCardStatus, GiftCardTransaction, GiftCardTxType};

pub fn normalize_code(raw: &str) -> String {
    raw.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect::<String>()
        .to_ascii_uppercase()
}

pub fn hash_gift_code(raw_code: &str) -> String {
    let clean = normalize_code(raw_code);
    let mut hasher = Sha256::new();
    hasher.update(clean.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn generate_secure_gift_code() -> (String, String, String) {
    let mut rng = rand::thread_rng();
    let alphabet: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
    let c1: String = (0..4).map(|_| alphabet[rng.gen_range(0..alphabet.len())] as char).collect();
    let c2: String = (0..4).map(|_| alphabet[rng.gen_range(0..alphabet.len())] as char).collect();
    let c3: String = (0..4).map(|_| alphabet[rng.gen_range(0..alphabet.len())] as char).collect();

    let raw_code = format!("PR-{}-{}-{}", c1, c2, c3);
    let masked = format!("PR-****-****-{}", c3);
    let hash = hash_gift_code(&raw_code);
    (raw_code, masked, hash)
}

pub fn init_gift_card_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS gift_cards (
            id TEXT PRIMARY KEY,
            code_hash TEXT NOT NULL UNIQUE,
            masked_code TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'Active',
            initial_amount_cents INTEGER NOT NULL,
            current_balance_cents INTEGER NOT NULL,
            currency TEXT NOT NULL DEFAULT 'EUR',
            purchaser_customer_id TEXT,
            recipient_name TEXT,
            recipient_contact TEXT,
            expires_at TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE UNIQUE INDEX IF NOT EXISTS idx_gift_cards_hash ON gift_cards(code_hash);

        CREATE TABLE IF NOT EXISTS gift_card_ledger (
            id TEXT PRIMARY KEY,
            gift_card_id TEXT NOT NULL,
            tx_type TEXT NOT NULL,
            amount_cents INTEGER NOT NULL,
            balance_after_cents INTEGER NOT NULL,
            reference_receipt_id TEXT,
            pos_terminal_id TEXT,
            note TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY (gift_card_id) REFERENCES gift_cards(id)
        );
        CREATE INDEX IF NOT EXISTS idx_gift_card_ledger_card ON gift_card_ledger(gift_card_id, created_at);
        "#,
    )?;
    Ok(())
}

fn map_gift_card_row(row: &Row) -> rusqlite::Result<GiftCard> {
    let status_str: String = row.get(3)?;
    Ok(GiftCard {
        id: row.get(0)?,
        code_hash: row.get(1)?,
        masked_code: row.get(2)?,
        status: GiftCardStatus::parse_str(&status_str),
        initial_amount_cents: row.get(4)?,
        current_balance_cents: row.get(5)?,
        currency: row.get(6)?,
        purchaser_customer_id: row.get(7)?,
        recipient_name: row.get(8)?,
        recipient_contact: row.get(9)?,
        expires_at: row.get(10)?,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
    })
}

fn insert_ledger_tx(conn: &Connection, tx: &GiftCardTransaction) -> rusqlite::Result<()> {
    conn.execute(
        r#"INSERT INTO gift_card_ledger (
            id, gift_card_id, tx_type, amount_cents, balance_after_cents,
            reference_receipt_id, pos_terminal_id, note, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"#,
        params![
            tx.id, tx.gift_card_id, tx.tx_type.as_str(), tx.amount_cents,
            tx.balance_after_cents, tx.reference_receipt_id, tx.pos_terminal_id, tx.note, tx.created_at,
        ],
    )?;
    Ok(())
}

pub fn issue_gift_card(
    conn: &Connection,
    initial_balance_cents: i64,
    purchaser_id: Option<&str>,
    recipient_name: Option<&str>,
    recipient_contact: Option<&str>,
    expires_in_days: Option<u32>,
    note: &str,
) -> Result<(GiftCard, String), GiftCardError> {
    if initial_balance_cents <= 0 {
        return Err(GiftCardError::InvalidAmount("Initial balance must be positive".into()));
    }

    let (raw_code, masked_code, code_hash) = generate_secure_gift_code();
    let now = Utc::now();
    let expires_at = expires_in_days.map(|days| (now + Duration::days(days as i64)).to_rfc3339());
    let now_str = now.to_rfc3339();

    let card = GiftCard {
        id: format!("gc_{}", Uuid::now_v7()),
        code_hash,
        masked_code,
        status: GiftCardStatus::Active,
        initial_amount_cents: initial_balance_cents,
        current_balance_cents: initial_balance_cents,
        currency: "EUR".to_string(),
        purchaser_customer_id: purchaser_id.map(String::from),
        recipient_name: recipient_name.map(String::from),
        recipient_contact: recipient_contact.map(String::from),
        expires_at,
        created_at: now_str.clone(),
        updated_at: now_str.clone(),
    };

    conn.execute(
        r#"INSERT INTO gift_cards (
            id, code_hash, masked_code, status, initial_amount_cents, current_balance_cents,
            currency, purchaser_customer_id, recipient_name, recipient_contact, expires_at,
            created_at, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)"#,
        params![
            card.id, card.code_hash, card.masked_code, card.status.as_str(),
            card.initial_amount_cents, card.current_balance_cents, card.currency,
            card.purchaser_customer_id, card.recipient_name, card.recipient_contact,
            card.expires_at, card.created_at, card.updated_at,
        ],
    )?;

    let tx = GiftCardTransaction {
        id: format!("gcl_{}", Uuid::now_v7()),
        gift_card_id: card.id.clone(),
        tx_type: GiftCardTxType::Issue,
        amount_cents: card.initial_amount_cents,
        balance_after_cents: card.current_balance_cents,
        reference_receipt_id: None,
        pos_terminal_id: None,
        note: note.to_string(),
        created_at: now_str,
    };
    insert_ledger_tx(conn, &tx)?;

    Ok((card, raw_code))
}

pub fn lookup_gift_card(conn: &Connection, raw_code: &str) -> rusqlite::Result<Option<GiftCard>> {
    let hash = hash_gift_code(raw_code);
    conn.query_row(
        r#"SELECT id, code_hash, masked_code, status, initial_amount_cents, current_balance_cents,
                  currency, purchaser_customer_id, recipient_name, recipient_contact, expires_at,
                  created_at, updated_at
           FROM gift_cards WHERE code_hash = ?1"#,
        params![hash],
        map_gift_card_row,
    )
    .optional()
}

pub fn top_up_gift_card(
    conn: &Connection,
    raw_code: &str,
    amount_cents: i64,
    reference_id: Option<&str>,
    terminal_id: Option<&str>,
    note: &str,
) -> Result<(GiftCard, GiftCardTransaction), GiftCardError> {
    if amount_cents <= 0 {
        return Err(GiftCardError::InvalidAmount("Top-up amount must be positive".into()));
    }

    let mut card = lookup_gift_card(conn, raw_code)?
        .ok_or(GiftCardError::CardNotFound)?;

    if card.status != GiftCardStatus::Active {
        return Err(GiftCardError::CardInactive(card.status.as_str().to_string()));
    }

    card.current_balance_cents += amount_cents;
    card.updated_at = Utc::now().to_rfc3339();

    let tx = GiftCardTransaction {
        id: format!("gcl_{}", Uuid::now_v7()),
        gift_card_id: card.id.clone(),
        tx_type: GiftCardTxType::TopUp,
        amount_cents,
        balance_after_cents: card.current_balance_cents,
        reference_receipt_id: reference_id.map(String::from),
        pos_terminal_id: terminal_id.map(String::from),
        note: note.to_string(),
        created_at: card.updated_at.clone(),
    };

    conn.execute(
        "UPDATE gift_cards SET current_balance_cents = ?1, updated_at = ?2 WHERE id = ?3",
        params![card.current_balance_cents, card.updated_at, card.id],
    )?;
    insert_ledger_tx(conn, &tx)?;

    Ok((card, tx))
}

pub fn redeem_gift_card(
    conn: &Connection,
    raw_code: &str,
    amount_cents: i64,
    reference_id: Option<&str>,
    terminal_id: Option<&str>,
    note: &str,
) -> Result<(GiftCard, GiftCardTransaction, i64), GiftCardError> {
    if amount_cents <= 0 {
        return Err(GiftCardError::InvalidAmount("Redemption amount must be positive".into()));
    }

    let mut card = lookup_gift_card(conn, raw_code)?
        .ok_or(GiftCardError::CardNotFound)?;

    if card.status != GiftCardStatus::Active {
        return Err(GiftCardError::CardInactive(card.status.as_str().to_string()));
    }

    if let Some(exp) = &card.expires_at {
        if Utc::now().to_rfc3339() > *exp {
            return Err(GiftCardError::CardExpired(exp.clone()));
        }
    }

    if card.current_balance_cents <= 0 {
        return Err(GiftCardError::InsufficientBalance {
            requested: amount_cents,
            available: 0,
        });
    }

    let actual_redeemed = amount_cents.min(card.current_balance_cents);
    card.current_balance_cents -= actual_redeemed;
    if card.current_balance_cents == 0 {
        card.status = GiftCardStatus::Redeemed;
    }
    card.updated_at = Utc::now().to_rfc3339();

    let tx = GiftCardTransaction {
        id: format!("gcl_{}", Uuid::now_v7()),
        gift_card_id: card.id.clone(),
        tx_type: GiftCardTxType::Redeem,
        amount_cents: -actual_redeemed,
        balance_after_cents: card.current_balance_cents,
        reference_receipt_id: reference_id.map(String::from),
        pos_terminal_id: terminal_id.map(String::from),
        note: note.to_string(),
        created_at: card.updated_at.clone(),
    };

    conn.execute(
        "UPDATE gift_cards SET status = ?1, current_balance_cents = ?2, updated_at = ?3 WHERE id = ?4",
        params![card.status.as_str(), card.current_balance_cents, card.updated_at, card.id],
    )?;
    insert_ledger_tx(conn, &tx)?;

    Ok((card, tx, actual_redeemed))
}

pub fn get_card_ledger(
    conn: &Connection,
    gift_card_id: &str,
    limit: usize,
) -> rusqlite::Result<Vec<GiftCardTransaction>> {
    let mut stmt = conn.prepare(
        r#"SELECT id, gift_card_id, tx_type, amount_cents, balance_after_cents,
                  reference_receipt_id, pos_terminal_id, note, created_at
           FROM gift_card_ledger WHERE gift_card_id = ?1 ORDER BY created_at DESC LIMIT ?2"#,
    )?;

    let entries = stmt
        .query_map(params![gift_card_id, limit as i64], |row| {
            let tx_type_str: String = row.get(2)?;
            Ok(GiftCardTransaction {
                id: row.get(0)?,
                gift_card_id: row.get(1)?,
                tx_type: GiftCardTxType::parse_str(&tx_type_str),
                amount_cents: row.get(3)?,
                balance_after_cents: row.get(4)?,
                reference_receipt_id: row.get(5)?,
                pos_terminal_id: row.get(6)?,
                note: row.get(7)?,
                created_at: row.get(8)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gift_card_issuance_redemption_and_topup() {
        let conn = Connection::open_in_memory().unwrap();
        init_gift_card_schema(&conn).unwrap();

        let (card, raw_code) = issue_gift_card(
            &conn,
            5000,
            Some("CUST-100"),
            Some("Maria Papadopoulou"),
            Some("+306912345678"),
            Some(365),
            "Birthday gift",
        )
        .unwrap();

        assert_eq!(card.initial_amount_cents, 5000);
        assert_eq!(card.current_balance_cents, 5000);
        assert_eq!(card.status, GiftCardStatus::Active);
        assert!(card.masked_code.starts_with("PR-****-****-"));

        let looked_up = lookup_gift_card(&conn, &raw_code).unwrap().unwrap();
        assert_eq!(looked_up.id, card.id);

        let (card_after_redeem, tx, redeemed_cents) = redeem_gift_card(
            &conn,
            &raw_code,
            2000,
            Some("REC-9001"),
            Some("TERM-01"),
            "Partial receipt tender",
        )
        .unwrap();
        assert_eq!(redeemed_cents, 2000);
        assert_eq!(card_after_redeem.current_balance_cents, 3000);
        assert_eq!(card_after_redeem.status, GiftCardStatus::Active);
        assert_eq!(tx.amount_cents, -2000);

        let (card_after_topup, topup_tx) = top_up_gift_card(
            &conn,
            &raw_code,
            1500,
            Some("REC-9002"),
            Some("TERM-01"),
            "Store credit recharge",
        )
        .unwrap();
        assert_eq!(card_after_topup.current_balance_cents, 4500);
        assert_eq!(topup_tx.amount_cents, 1500);

        let (card_exhausted, _, final_redeemed) = redeem_gift_card(
            &conn,
            &raw_code,
            5000,
            Some("REC-9003"),
            Some("TERM-01"),
            "Full redemption",
        )
        .unwrap();
        assert_eq!(final_redeemed, 4500);
        assert_eq!(card_exhausted.current_balance_cents, 0);
        assert_eq!(card_exhausted.status, GiftCardStatus::Redeemed);

        let history = get_card_ledger(&conn, &card.id, 10).unwrap();
        assert_eq!(history.len(), 4);
    }
}
