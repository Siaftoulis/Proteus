//! SQLite Persistence and Offline Outbox for Shipping Notes.
//! Strict Rule 1 (100% Original Codebase) and Rule 5 (Zero Mock Data).

use chrono::Utc;
use rusqlite::{params, Connection, Result};

use super::types::{
    DispatchStatus, ShippingNote, ShippingNoteItem, ShippingUnit, TransportPurpose,
};

/// Initializes the SQLite schema for digital shipping notes, items, and outbox.
pub fn init_shipping_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS shipping_notes (
            id TEXT PRIMARY KEY,
            note_number TEXT NOT NULL UNIQUE,
            issuer_afm TEXT NOT NULL,
            issuer_name TEXT NOT NULL,
            issuer_address TEXT NOT NULL,
            recipient_afm TEXT NOT NULL,
            recipient_name TEXT NOT NULL,
            recipient_address TEXT NOT NULL,
            vehicle_plate TEXT NOT NULL,
            driver_name TEXT NOT NULL,
            departure_time TEXT NOT NULL,
            estimated_arrival TEXT,
            actual_arrival TEXT,
            purpose TEXT NOT NULL,
            gross_weight_kg REAL,
            packages_count INTEGER NOT NULL,
            mydata_mark TEXT,
            qr_payload TEXT NOT NULL,
            digital_signature_hash TEXT NOT NULL,
            status TEXT NOT NULL,
            recipient_signature_note TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_shipping_status ON shipping_notes(status);
        CREATE INDEX IF NOT EXISTS idx_shipping_date ON shipping_notes(created_at);
        CREATE INDEX IF NOT EXISTS idx_shipping_vehicle ON shipping_notes(vehicle_plate);

        CREATE TABLE IF NOT EXISTS shipping_note_items (
            id TEXT PRIMARY KEY,
            note_id TEXT NOT NULL,
            line_number INTEGER NOT NULL,
            sku TEXT NOT NULL,
            description TEXT NOT NULL,
            unit TEXT NOT NULL,
            quantity REAL NOT NULL,
            serial_numbers TEXT NOT NULL DEFAULT '[]',
            batch_lot TEXT,
            notes TEXT,
            FOREIGN KEY(note_id) REFERENCES shipping_notes(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_shipping_items_note ON shipping_note_items(note_id);

        CREATE TABLE IF NOT EXISTS shipping_outbox (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            note_id TEXT NOT NULL,
            action TEXT NOT NULL,
            payload_json TEXT NOT NULL,
            queued_at TEXT NOT NULL,
            synced_at TEXT,
            retry_count INTEGER NOT NULL DEFAULT 0,
            last_error TEXT
        );
        CREATE INDEX IF NOT EXISTS idx_shipping_outbox_pending ON shipping_outbox(synced_at);",
    )
}

/// Persists a new ShippingNote into SQLite.
pub fn create_shipping_note(conn: &Connection, note: &ShippingNote) -> Result<()> {
    init_shipping_schema(conn)?;
    conn.execute(
        "INSERT INTO shipping_notes (
            id, note_number, issuer_afm, issuer_name, issuer_address,
            recipient_afm, recipient_name, recipient_address, vehicle_plate,
            driver_name, departure_time, estimated_arrival, actual_arrival,
            purpose, gross_weight_kg, packages_count, mydata_mark,
            qr_payload, digital_signature_hash, status, recipient_signature_note,
            created_at, updated_at
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
            ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20,
            ?21, ?22, ?23
        )",
        params![
            note.id,
            note.note_number,
            note.issuer_afm,
            note.issuer_name,
            note.issuer_address,
            note.recipient_afm,
            note.recipient_name,
            note.recipient_address,
            note.vehicle_plate,
            note.driver_name,
            note.departure_time,
            note.estimated_arrival,
            note.actual_arrival,
            note.purpose.code(),
            note.gross_weight_kg,
            note.packages_count,
            note.mydata_mark,
            note.qr_payload,
            note.digital_signature_hash,
            note.status.code(),
            note.recipient_signature_note,
            note.created_at,
            note.updated_at,
        ],
    )?;
    Ok(())
}

/// Retrieves a ShippingNote by its unique ID.
pub fn get_shipping_note(conn: &Connection, id: &str) -> Result<Option<ShippingNote>> {
    init_shipping_schema(conn)?;
    let mut stmt = conn.prepare(
        "SELECT id, note_number, issuer_afm, issuer_name, issuer_address,
                recipient_afm, recipient_name, recipient_address, vehicle_plate,
                driver_name, departure_time, estimated_arrival, actual_arrival,
                purpose, gross_weight_kg, packages_count, mydata_mark,
                qr_payload, digital_signature_hash, status, recipient_signature_note,
                created_at, updated_at
         FROM shipping_notes WHERE id = ?1",
    )?;

    let mut rows = stmt.query(params![id])?;
    if let Some(row) = rows.next()? {
        let purpose_str: String = row.get(13)?;
        let status_str: String = row.get(19)?;
        Ok(Some(ShippingNote {
            id: row.get(0)?,
            note_number: row.get(1)?,
            issuer_afm: row.get(2)?,
            issuer_name: row.get(3)?,
            issuer_address: row.get(4)?,
            recipient_afm: row.get(5)?,
            recipient_name: row.get(6)?,
            recipient_address: row.get(7)?,
            vehicle_plate: row.get(8)?,
            driver_name: row.get(9)?,
            departure_time: row.get(10)?,
            estimated_arrival: row.get(11)?,
            actual_arrival: row.get(12)?,
            purpose: TransportPurpose::from_code(&purpose_str),
            gross_weight_kg: row.get(14)?,
            packages_count: row.get(15)?,
            mydata_mark: row.get(16)?,
            qr_payload: row.get(17)?,
            digital_signature_hash: row.get(18)?,
            status: DispatchStatus::from_code(&status_str),
            recipient_signature_note: row.get(20)?,
            created_at: row.get(21)?,
            updated_at: row.get(22)?,
        }))
    } else {
        Ok(None)
    }
}

/// Lists all ShippingNotes ordered by creation date descending.
pub fn list_shipping_notes(conn: &Connection) -> Result<Vec<ShippingNote>> {
    init_shipping_schema(conn)?;
    let mut stmt = conn.prepare(
        "SELECT id, note_number, issuer_afm, issuer_name, issuer_address,
                recipient_afm, recipient_name, recipient_address, vehicle_plate,
                driver_name, departure_time, estimated_arrival, actual_arrival,
                purpose, gross_weight_kg, packages_count, mydata_mark,
                qr_payload, digital_signature_hash, status, recipient_signature_note,
                created_at, updated_at
         FROM shipping_notes ORDER BY created_at DESC",
    )?;

    let rows = stmt.query_map([], |row| {
        let purpose_str: String = row.get(13)?;
        let status_str: String = row.get(19)?;
        Ok(ShippingNote {
            id: row.get(0)?,
            note_number: row.get(1)?,
            issuer_afm: row.get(2)?,
            issuer_name: row.get(3)?,
            issuer_address: row.get(4)?,
            recipient_afm: row.get(5)?,
            recipient_name: row.get(6)?,
            recipient_address: row.get(7)?,
            vehicle_plate: row.get(8)?,
            driver_name: row.get(9)?,
            departure_time: row.get(10)?,
            estimated_arrival: row.get(11)?,
            actual_arrival: row.get(12)?,
            purpose: TransportPurpose::from_code(&purpose_str),
            gross_weight_kg: row.get(14)?,
            packages_count: row.get(15)?,
            mydata_mark: row.get(16)?,
            qr_payload: row.get(17)?,
            digital_signature_hash: row.get(18)?,
            status: DispatchStatus::from_code(&status_str),
            recipient_signature_note: row.get(20)?,
            created_at: row.get(21)?,
            updated_at: row.get(22)?,
        })
    })?;

    let mut notes = Vec::new();
    for row in rows {
        notes.push(row?);
    }
    Ok(notes)
}

/// Adds an item line to an existing shipping note.
pub fn add_shipping_note_item(conn: &Connection, item: &ShippingNoteItem) -> Result<()> {
    init_shipping_schema(conn)?;
    let serials_json = serde_json::to_string(&item.serial_numbers).unwrap_or_else(|_| "[]".to_string());
    conn.execute(
        "INSERT INTO shipping_note_items (
            id, note_id, line_number, sku, description, unit, quantity, serial_numbers, batch_lot, notes
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            item.id,
            item.note_id,
            item.line_number,
            item.sku,
            item.description,
            item.unit.code(),
            item.quantity,
            serials_json,
            item.batch_lot,
            item.notes,
        ],
    )?;
    Ok(())
}

/// Retrieves all item lines belonging to a shipping note.
pub fn list_shipping_note_items(conn: &Connection, note_id: &str) -> Result<Vec<ShippingNoteItem>> {
    init_shipping_schema(conn)?;
    let mut stmt = conn.prepare(
        "SELECT id, note_id, line_number, sku, description, unit, quantity, serial_numbers, batch_lot, notes
         FROM shipping_note_items WHERE note_id = ?1 ORDER BY line_number ASC",
    )?;

    let rows = stmt.query_map(params![note_id], |row| {
        let unit_str: String = row.get(5)?;
        let serials_json: String = row.get(7)?;
        let serials: Vec<String> = serde_json::from_str(&serials_json).unwrap_or_default();
        Ok(ShippingNoteItem {
            id: row.get(0)?,
            note_id: row.get(1)?,
            line_number: row.get(2)?,
            sku: row.get(3)?,
            description: row.get(4)?,
            unit: ShippingUnit::from_code(&unit_str),
            quantity: row.get(6)?,
            serial_numbers: serials,
            batch_lot: row.get(8)?,
            notes: row.get(9)?,
        })
    })?;

    let mut items = Vec::new();
    for row in rows {
        items.push(row?);
    }
    Ok(items)
}

/// Updates the status of a shipping note.
pub fn update_shipping_note_status(conn: &Connection, id: &str, status: DispatchStatus) -> Result<()> {
    init_shipping_schema(conn)?;
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE shipping_notes SET status = ?1, updated_at = ?2 WHERE id = ?3",
        params![status.code(), now, id],
    )?;
    Ok(())
}

/// Records delivery completion with timestamp and receiver sign-off notes.
pub fn record_delivery_completion(
    conn: &Connection,
    id: &str,
    recipient_signature_note: &str,
    arrival_time: &str,
) -> Result<()> {
    init_shipping_schema(conn)?;
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE shipping_notes SET
            status = ?1,
            actual_arrival = ?2,
            recipient_signature_note = ?3,
            updated_at = ?4
         WHERE id = ?5",
        params![
            DispatchStatus::Delivered.code(),
            arrival_time,
            recipient_signature_note,
            now,
            id
        ],
    )?;
    Ok(())
}

/// Queues an action into the offline dispatch outbox.
pub fn queue_offline_dispatch(
    conn: &Connection,
    note_id: &str,
    action: &str,
    payload_json: &str,
) -> Result<i64> {
    init_shipping_schema(conn)?;
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO shipping_outbox (note_id, action, payload_json, queued_at)
         VALUES (?1, ?2, ?3, ?4)",
        params![note_id, action, payload_json, now],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Lists all pending unsynced actions from the offline dispatch outbox.
pub fn list_pending_offline_dispatches(conn: &Connection) -> Result<Vec<(i64, String, String, String)>> {
    init_shipping_schema(conn)?;
    let mut stmt = conn.prepare(
        "SELECT id, note_id, action, payload_json FROM shipping_outbox WHERE synced_at IS NULL ORDER BY id ASC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    })?;

    let mut out = Vec::new();
    for row in rows {
        out.push(row?);
    }
    Ok(out)
}

/// Marks an offline outbox item as successfully synced to the central node.
pub fn mark_offline_dispatch_synced(conn: &Connection, outbox_id: i64) -> Result<()> {
    init_shipping_schema(conn)?;
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE shipping_outbox SET synced_at = ?1 WHERE id = ?2",
        params![now, outbox_id],
    )?;
    Ok(())
}
