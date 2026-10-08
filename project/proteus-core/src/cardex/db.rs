//! Financial Cardex SQLite Persistence & Movement Dispatch Engine.
//! Strict Rule 1 (100% Original Codebase) and Rule 5 (Zero Mock Data).

use rusqlite::{params, Connection, Result};
use uuid::Uuid;

use crate::cardex::types::{CardexEntity, CardexEntry, CardexMovementType, EntityType};

/// Initializes the financial cardex, entries, and receipt schemas in SQLite.
pub fn init_cardex_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS cardex_entities (
            entity_id TEXT PRIMARY KEY,
            entity_type TEXT NOT NULL CHECK(entity_type IN ('CUSTOMER', 'SUPPLIER')),
            name TEXT NOT NULL,
            phone TEXT,
            email TEXT,
            credit_limit_eur REAL NOT NULL DEFAULT 0.0,
            payment_terms_days INTEGER NOT NULL DEFAULT 30,
            current_balance_eur REAL NOT NULL DEFAULT 0.0,
            created_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS cardex_entries (
            entry_id TEXT PRIMARY KEY,
            entity_id TEXT NOT NULL REFERENCES cardex_entities(entity_id),
            movement_type TEXT NOT NULL CHECK(movement_type IN ('DEBIT', 'CREDIT')),
            document_type TEXT NOT NULL,
            document_series TEXT NOT NULL,
            document_number INTEGER NOT NULL,
            description TEXT NOT NULL,
            amount_eur REAL NOT NULL,
            running_balance_eur REAL NOT NULL,
            issue_date TEXT NOT NULL,
            due_date TEXT NOT NULL,
            created_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS payment_receipts (
            receipt_id TEXT PRIMARY KEY,
            entity_id TEXT NOT NULL REFERENCES cardex_entities(entity_id),
            series TEXT NOT NULL,
            receipt_number INTEGER NOT NULL,
            amount_eur REAL NOT NULL,
            payment_method TEXT NOT NULL,
            notes TEXT,
            issue_date TEXT NOT NULL,
            created_at INTEGER NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_cardex_entries_entity ON cardex_entries(entity_id);
        CREATE INDEX IF NOT EXISTS idx_cardex_entries_date ON cardex_entries(issue_date);
        CREATE INDEX IF NOT EXISTS idx_receipts_entity ON payment_receipts(entity_id);
        "#,
    )?;
    Ok(())
}

/// Upserts an entity in the financial ledger.
pub fn save_cardex_entity(conn: &Connection, entity: &CardexEntity) -> Result<()> {
    conn.execute(
        r#"
        INSERT INTO cardex_entities (
            entity_id, entity_type, name, phone, email, credit_limit_eur,
            payment_terms_days, current_balance_eur, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        ON CONFLICT(entity_id) DO UPDATE SET
            name = excluded.name,
            phone = excluded.phone,
            email = excluded.email,
            credit_limit_eur = excluded.credit_limit_eur,
            payment_terms_days = excluded.payment_terms_days
        "#,
        params![
            entity.entity_id,
            entity.entity_type.as_str(),
            entity.name,
            entity.phone,
            entity.email,
            entity.credit_limit_eur,
            entity.payment_terms_days,
            entity.current_balance_eur,
            entity.created_at,
        ],
    )?;
    Ok(())
}

/// Retrieves an entity by its identifier (AFM or code).
pub fn get_cardex_entity(conn: &Connection, entity_id: &str) -> Result<Option<CardexEntity>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT entity_id, entity_type, name, phone, email, credit_limit_eur,
               payment_terms_days, current_balance_eur, created_at
        FROM cardex_entities
        WHERE entity_id = ?1
        "#,
    )?;

    let mut rows = stmt.query(params![entity_id])?;
    if let Some(r) = rows.next()? {
        let type_str: String = r.get(1)?;
        let ent_type = EntityType::from_str(&type_str).unwrap_or(EntityType::Customer);
        Ok(Some(CardexEntity {
            entity_id: r.get(0)?,
            entity_type: ent_type,
            name: r.get(2)?,
            phone: r.get(3)?,
            email: r.get(4)?,
            credit_limit_eur: r.get(5)?,
            payment_terms_days: r.get(6)?,
            current_balance_eur: r.get(7)?,
            created_at: r.get(8)?,
        }))
    } else {
        Ok(None)
    }
}

/// Posts a financial movement into an entity's cardex, updating their rolling balance.
pub fn post_cardex_movement(
    conn: &Connection,
    entity_id: &str,
    movement_type: CardexMovementType,
    document_type: &str,
    series: &str,
    doc_number: i64,
    description: &str,
    amount_eur: f64,
    issue_date: &str,
    due_date: &str,
) -> Result<CardexEntry> {
    let entity = get_cardex_entity(conn, entity_id)?
        .ok_or_else(|| rusqlite::Error::QueryReturnedNoRows)?;

    let amount = (amount_eur * 100.0).round() / 100.0;

    let new_balance = match (entity.entity_type, movement_type) {
        (EntityType::Customer, CardexMovementType::Debit) => entity.current_balance_eur + amount,
        (EntityType::Customer, CardexMovementType::Credit) => entity.current_balance_eur - amount,
        (EntityType::Supplier, CardexMovementType::Credit) => entity.current_balance_eur + amount,
        (EntityType::Supplier, CardexMovementType::Debit) => entity.current_balance_eur - amount,
    };
    let rounded_balance = (new_balance * 100.0).round() / 100.0;

    let entry_id = Uuid::now_v7().to_string();
    let now = chrono::Utc::now().timestamp_millis();

    conn.execute(
        r#"
        INSERT INTO cardex_entries (
            entry_id, entity_id, movement_type, document_type, document_series,
            document_number, description, amount_eur, running_balance_eur,
            issue_date, due_date, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
        "#,
        params![
            entry_id,
            entity_id,
            movement_type.as_str(),
            document_type,
            series,
            doc_number,
            description,
            amount,
            rounded_balance,
            issue_date,
            due_date,
            now,
        ],
    )?;

    conn.execute(
        "UPDATE cardex_entities SET current_balance_eur = ?1 WHERE entity_id = ?2",
        params![rounded_balance, entity_id],
    )?;

    Ok(CardexEntry {
        entry_id,
        entity_id: entity_id.to_string(),
        movement_type,
        document_type: document_type.to_string(),
        document_series: series.to_string(),
        document_number: doc_number,
        description: description.to_string(),
        amount_eur: amount,
        running_balance_eur: rounded_balance,
        issue_date: issue_date.to_string(),
        due_date: due_date.to_string(),
        created_at: now,
    })
}

/// Records an official payment collection receipt and offsets the cardex balance.
pub fn record_payment_receipt(
    conn: &Connection,
    entity_id: &str,
    series: &str,
    receipt_number: i64,
    amount_eur: f64,
    payment_method: &str,
    notes: &str,
    issue_date: &str,
) -> Result<String> {
    let receipt_id = Uuid::now_v7().to_string();
    let now = chrono::Utc::now().timestamp_millis();
    let amount = (amount_eur * 100.0).round() / 100.0;

    conn.execute(
        r#"
        INSERT INTO payment_receipts (
            receipt_id, entity_id, series, receipt_number, amount_eur, payment_method, notes, issue_date, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        "#,
        params![
            receipt_id,
            entity_id,
            series,
            receipt_number,
            amount,
            payment_method,
            notes,
            issue_date,
            now,
        ],
    )?;

    let entity = get_cardex_entity(conn, entity_id)?
        .ok_or_else(|| rusqlite::Error::QueryReturnedNoRows)?;

    let movement_type = match entity.entity_type {
        EntityType::Customer => CardexMovementType::Credit,
        EntityType::Supplier => CardexMovementType::Debit,
    };

    let desc = format!("Είσπραξη / Πληρωμή [{}] {}", payment_method, notes);
    post_cardex_movement(
        conn,
        entity_id,
        movement_type,
        "ΑΠΟΔΕΙΞΗ_ΕΙΣΠΡΑΞΗΣ",
        series,
        receipt_number,
        &desc,
        amount,
        issue_date,
        issue_date,
    )?;

    Ok(receipt_id)
}

/// Retrieves all ledger entries for an entity ordered chronologically.
pub fn list_cardex_entries(conn: &Connection, entity_id: &str) -> Result<Vec<CardexEntry>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT entry_id, entity_id, movement_type, document_type, document_series,
               document_number, description, amount_eur, running_balance_eur,
               issue_date, due_date, created_at
        FROM cardex_entries
        WHERE entity_id = ?1
        ORDER BY issue_date ASC, created_at ASC
        "#,
    )?;

    let rows = stmt.query_map(params![entity_id], |r| {
        let m_str: String = r.get(2)?;
        let m_type = CardexMovementType::from_str(&m_str).unwrap_or(CardexMovementType::Debit);
        Ok(CardexEntry {
            entry_id: r.get(0)?,
            entity_id: r.get(1)?,
            movement_type: m_type,
            document_type: r.get(3)?,
            document_series: r.get(4)?,
            document_number: r.get(5)?,
            description: r.get(6)?,
            amount_eur: r.get(7)?,
            running_balance_eur: r.get(8)?,
            issue_date: r.get(9)?,
            due_date: r.get(10)?,
            created_at: r.get(11)?,
        })
    })?;

    let mut entries = Vec::new();
    for row in rows {
        entries.push(row?);
    }
    Ok(entries)
}

/// GDPR Article 17 & Greek Law 4624/2019: Anonymizes customer PII in Cardex ledger.
/// Redacts name to '[GDPR ΑΝΩΝΥΜΟΠΟΙΗΜΕΝΟ]', phone to '[ERASED]', and email to '[ERASED]',
/// while strictly preserving financial accounting balances to satisfy statutory tax laws.
pub fn anonymize_cardex_customer(conn: &Connection, entity_id: &str, operator: &str) -> Result<bool> {
    let count = conn.execute(
        r#"
        UPDATE cardex_entities
        SET name = '[GDPR ΑΝΩΝΥΜΟΠΟΙΗΜΕΝΟ]', phone = '[ERASED]', email = '[ERASED]'
        WHERE entity_id = ?1
        "#,
        params![entity_id],
    )?;

    if count > 0 {
        let event_id = Uuid::now_v7().to_string();
        let now = chrono::Utc::now().timestamp_millis();
        let _ = conn.execute(
            "INSERT INTO system_events (event_id, entity_id, event_type, payload, created_at)
             VALUES (?1, ?2, 'GDPR_CARDEX_CUSTOMER_ANONYMIZED', ?3, ?4)",
            params![
                event_id,
                entity_id,
                format!(r#"{{"operator":"{}","action":"GDPR_ART17_ERASURE"}}"#, operator),
                now
            ],
        );
        Ok(true)
    } else {
        Ok(false)
    }
}
