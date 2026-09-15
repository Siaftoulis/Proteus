//! Service tickets and repair tracking engine for Proteus BOS.
//! Core operational wedge: «Παραλαβή -> Επισκευή -> Παράδοση».

use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::fmt;

/// The 6 canonical lifecycle states of a service repair ticket.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TicketStatus {
    Received,
    InProgress,
    WaitingParts,
    Ready,
    Delivered,
    Cancelled,
}

impl TicketStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TicketStatus::Received => "received",
            TicketStatus::InProgress => "in_progress",
            TicketStatus::WaitingParts => "waiting_parts",
            TicketStatus::Ready => "ready",
            TicketStatus::Delivered => "delivered",
            TicketStatus::Cancelled => "cancelled",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            TicketStatus::Received => "Παραλήφθηκε",
            TicketStatus::InProgress => "Σε Επισκευή",
            TicketStatus::WaitingParts => "Αναμονή Ανταλλακτικών",
            TicketStatus::Ready => "Έτοιμο",
            TicketStatus::Delivered => "Παραδόθηκε",
            TicketStatus::Cancelled => "Ακυρώθηκε",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "in_progress" => TicketStatus::InProgress,
            "waiting_parts" => TicketStatus::WaitingParts,
            "ready" => TicketStatus::Ready,
            "delivered" => TicketStatus::Delivered,
            "cancelled" => TicketStatus::Cancelled,
            _ => TicketStatus::Received,
        }
    }

    pub fn all() -> &'static [TicketStatus] {
        &[
            TicketStatus::Received,
            TicketStatus::InProgress,
            TicketStatus::WaitingParts,
            TicketStatus::Ready,
            TicketStatus::Delivered,
            TicketStatus::Cancelled,
        ]
    }
}

impl fmt::Display for TicketStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Core domain record representing an intake / service repair ticket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceTicket {
    pub ticket_id: String,
    pub ticket_number: i64,
    pub customer_name: String,
    pub customer_phone: String,
    pub device_model: String,
    pub serial_number: Option<String>,
    pub reported_fault: String,
    pub internal_notes: String,
    pub estimated_cost: f64,
    pub current_status: TicketStatus,
    pub created_at: i64,
    pub updated_at: i64,
    pub delivered_at: Option<i64>,
}

impl ServiceTicket {
    /// Creates a new ticket with UUIDv7 ID and monotonic timestamp.
    pub fn new(
        customer_name: impl Into<String>,
        customer_phone: impl Into<String>,
        device_model: impl Into<String>,
        reported_fault: impl Into<String>,
    ) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        let ticket_id = uuid::Uuid::now_v7().to_string();

        Self {
            ticket_id,
            ticket_number: 0, // Assigned on DB insert
            customer_name: customer_name.into(),
            customer_phone: customer_phone.into(),
            device_model: device_model.into(),
            serial_number: None,
            reported_fault: reported_fault.into(),
            internal_notes: String::new(),
            estimated_cost: 0.0,
            current_status: TicketStatus::Received,
            created_at: now,
            updated_at: now,
            delivered_at: None,
        }
    }
}

/// Initializes the standard Service BOS database schema in SQLite.
pub fn init_tickets_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS service_tickets (
            ticket_id TEXT PRIMARY KEY,
            ticket_number INTEGER NOT NULL,
            customer_name TEXT NOT NULL,
            customer_phone TEXT NOT NULL,
            device_model TEXT NOT NULL,
            serial_number TEXT,
            reported_fault TEXT NOT NULL,
            internal_notes TEXT DEFAULT '',
            estimated_cost REAL DEFAULT 0.0,
            current_status TEXT NOT NULL CHECK(
                current_status IN ('received', 'in_progress', 'waiting_parts', 'ready', 'delivered', 'cancelled')
            ),
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            delivered_at INTEGER
        );

        CREATE TABLE IF NOT EXISTS system_events (
            event_id TEXT PRIMARY KEY,
            entity_id TEXT NOT NULL,
            event_type TEXT NOT NULL,
            payload JSON NOT NULL,
            created_at INTEGER NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_tickets_status ON service_tickets(current_status);
        CREATE INDEX IF NOT EXISTS idx_tickets_phone ON service_tickets(customer_phone);
        CREATE INDEX IF NOT EXISTS idx_tickets_updated ON service_tickets(updated_at);
        "#,
    )?;
    Ok(())
}

/// Inserts a new ticket and automatically computes the next sequential ticket number.
pub fn create_ticket(conn: &Connection, ticket: &mut ServiceTicket) -> Result<()> {
    let now = chrono::Utc::now().timestamp_millis();
    ticket.updated_at = now;
    if ticket.created_at == 0 {
        ticket.created_at = now;
    }

    // Determine next sequential ticket number for the shop
    let next_num: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(ticket_number), 0) + 1 FROM service_tickets",
            [],
            |row| row.get(0),
        )
        .unwrap_or(1);

    ticket.ticket_number = next_num;

    conn.execute(
        r#"
        INSERT INTO service_tickets (
            ticket_id, ticket_number, customer_name, customer_phone,
            device_model, serial_number, reported_fault, internal_notes,
            estimated_cost, current_status, created_at, updated_at, delivered_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
        "#,
        params![
            ticket.ticket_id,
            ticket.ticket_number,
            ticket.customer_name,
            ticket.customer_phone,
            ticket.device_model,
            ticket.serial_number,
            ticket.reported_fault,
            ticket.internal_notes,
            ticket.estimated_cost,
            ticket.current_status.as_str(),
            ticket.created_at,
            ticket.updated_at,
            ticket.delivered_at,
        ],
    )?;

    // Audit log event
    let event_id = uuid::Uuid::now_v7().to_string();
    let payload = serde_json::to_string(ticket).unwrap_or_default();
    let _ = conn.execute(
        "INSERT INTO system_events (event_id, entity_id, event_type, payload, created_at) VALUES (?1, ?2, 'TICKET_CREATED', ?3, ?4)",
        params![event_id, ticket.ticket_id, payload, now],
    );

    Ok(())
}

/// Updates the status of an existing ticket. If Delivered, stamps delivered_at.
pub fn update_ticket_status(
    conn: &Connection,
    ticket_id: &str,
    status: TicketStatus,
) -> Result<()> {
    let now = chrono::Utc::now().timestamp_millis();
    let delivered_at = if status == TicketStatus::Delivered {
        Some(now)
    } else {
        None
    };

    conn.execute(
        r#"
        UPDATE service_tickets
        SET current_status = ?1, updated_at = ?2, delivered_at = COALESCE(?3, delivered_at)
        WHERE ticket_id = ?4
        "#,
        params![status.as_str(), now, delivered_at, ticket_id],
    )?;

    let event_id = uuid::Uuid::now_v7().to_string();
    let _ = conn.execute(
        "INSERT INTO system_events (event_id, entity_id, event_type, payload, created_at) VALUES (?1, ?2, 'STATUS_CHANGED', ?3, ?4)",
        params![event_id, ticket_id, format!(r#"{{"status":"{}"}}"#, status.as_str()), now],
    );

    Ok(())
}

/// Updates notes and estimated cost for a ticket.
pub fn update_ticket_details(
    conn: &Connection,
    ticket_id: &str,
    internal_notes: &str,
    estimated_cost: f64,
) -> Result<()> {
    let now = chrono::Utc::now().timestamp_millis();
    conn.execute(
        r#"
        UPDATE service_tickets
        SET internal_notes = ?1, estimated_cost = ?2, updated_at = ?3
        WHERE ticket_id = ?4
        "#,
        params![internal_notes, estimated_cost, now, ticket_id],
    )?;
    Ok(())
}

/// Fetches a single ticket by its UUID.
pub fn get_ticket(conn: &Connection, ticket_id: &str) -> Result<Option<ServiceTicket>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT ticket_id, ticket_number, customer_name, customer_phone,
               device_model, serial_number, reported_fault, internal_notes,
               estimated_cost, current_status, created_at, updated_at, delivered_at
        FROM service_tickets
        WHERE ticket_id = ?1
        "#,
    )?;

    let mut rows = stmt.query(params![ticket_id])?;
    if let Some(row) = rows.next()? {
        let status_str: String = row.get(9)?;
        Ok(Some(ServiceTicket {
            ticket_id: row.get(0)?,
            ticket_number: row.get(1)?,
            customer_name: row.get(2)?,
            customer_phone: row.get(3)?,
            device_model: row.get(4)?,
            serial_number: row.get(5)?,
            reported_fault: row.get(6)?,
            internal_notes: row.get(7)?,
            estimated_cost: row.get(8)?,
            current_status: TicketStatus::from_str(&status_str),
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
            delivered_at: row.get(12)?,
        }))
    } else {
        Ok(None)
    }
}

/// Lists all tickets sorted by updated_at descending.
pub fn list_tickets(conn: &Connection) -> Result<Vec<ServiceTicket>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT ticket_id, ticket_number, customer_name, customer_phone,
               device_model, serial_number, reported_fault, internal_notes,
               estimated_cost, current_status, created_at, updated_at, delivered_at
        FROM service_tickets
        ORDER BY updated_at DESC
        "#,
    )?;

    let iter = stmt.query_map([], |row| {
        let status_str: String = row.get(9)?;
        Ok(ServiceTicket {
            ticket_id: row.get(0)?,
            ticket_number: row.get(1)?,
            customer_name: row.get(2)?,
            customer_phone: row.get(3)?,
            device_model: row.get(4)?,
            serial_number: row.get(5)?,
            reported_fault: row.get(6)?,
            internal_notes: row.get(7)?,
            estimated_cost: row.get(8)?,
            current_status: TicketStatus::from_str(&status_str),
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
            delivered_at: row.get(12)?,
        })
    })?;

    let mut tickets = Vec::new();
    for t in iter {
        tickets.push(t?);
    }
    Ok(tickets)
}

/// Searches tickets by customer name, phone, device model, or ticket number.
pub fn search_tickets(conn: &Connection, query: &str) -> Result<Vec<ServiceTicket>> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return list_tickets(conn);
    }

    let pattern = format!("%{}%", trimmed);
    let mut stmt = conn.prepare(
        r#"
        SELECT ticket_id, ticket_number, customer_name, customer_phone,
               device_model, serial_number, reported_fault, internal_notes,
               estimated_cost, current_status, created_at, updated_at, delivered_at
        FROM service_tickets
        WHERE customer_name LIKE ?1
           OR customer_phone LIKE ?1
           OR device_model LIKE ?1
           OR CAST(ticket_number AS TEXT) LIKE ?1
        ORDER BY updated_at DESC
        "#,
    )?;

    let iter = stmt.query_map(params![pattern], |row| {
        let status_str: String = row.get(9)?;
        Ok(ServiceTicket {
            ticket_id: row.get(0)?,
            ticket_number: row.get(1)?,
            customer_name: row.get(2)?,
            customer_phone: row.get(3)?,
            device_model: row.get(4)?,
            serial_number: row.get(5)?,
            reported_fault: row.get(6)?,
            internal_notes: row.get(7)?,
            estimated_cost: row.get(8)?,
            current_status: TicketStatus::from_str(&status_str),
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
            delivered_at: row.get(12)?,
        })
    })?;

    let mut tickets = Vec::new();
    for t in iter {
        tickets.push(t?);
    }
    Ok(tickets)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ticket_lifecycle() {
        let conn = Connection::open_in_memory().unwrap();
        init_tickets_schema(&conn).unwrap();

        let mut t1 = ServiceTicket::new("Νίκος Παπαδόπουλος", "6971234567", "Samsung Galaxy S22", "Σπασμένη οθόνη");
        create_ticket(&conn, &mut t1).unwrap();
        assert_eq!(t1.ticket_number, 1);
        assert_eq!(t1.current_status, TicketStatus::Received);

        let mut t2 = ServiceTicket::new("Μαρία Δημητρίου", "6987654321", "Dell XPS 15", "Δεν ανάβει");
        create_ticket(&conn, &mut t2).unwrap();
        assert_eq!(t2.ticket_number, 2);

        // Update status
        update_ticket_status(&conn, &t1.ticket_id, TicketStatus::InProgress).unwrap();
        let fetched = get_ticket(&conn, &t1.ticket_id).unwrap().unwrap();
        assert_eq!(fetched.current_status, TicketStatus::InProgress);

        // Update notes & cost
        update_ticket_details(&conn, &t1.ticket_id, "Αντικαταστάθηκε οθόνη OLED", 120.0).unwrap();
        let fetched2 = get_ticket(&conn, &t1.ticket_id).unwrap().unwrap();
        assert_eq!(fetched2.estimated_cost, 120.0);
        assert_eq!(fetched2.internal_notes, "Αντικαταστάθηκε οθόνη OLED");

        // Search
        let results = search_tickets(&conn, "Dell").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].customer_name, "Μαρία Δημητρίου");
    }
}
