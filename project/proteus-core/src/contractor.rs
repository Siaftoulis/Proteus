//! Frontline Hardware Store & Contractor Job-Site Sub-ledger Engine for Proteus.
//! Standardizes dual unit-of-measure conversions (pieces, boxes, kg, meters)
//! and project-based running ledgers for professional contractors (Μάστορες ανά Έργο/Οικοδομή).
//! Strict Rule 5 compliance: All records persist directly to real SQLite tables.

use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};

/// Standard retail and trade units of measure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnitOfMeasure {
    Piece,    // Τεμάχιο (τεμ)
    Pack,     // Πακέτο (π.χ. 50 τεμ)
    Box,      // Κουτί (π.χ. 500 τεμ)
    Kilogram, // Κιλό (kg)
    Meter,    // Μέτρο (m)
}

impl UnitOfMeasure {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Piece => "τεμ",
            Self::Pack => "πακέτο",
            Self::Box => "κουτί",
            Self::Kilogram => "kg",
            Self::Meter => "m",
        }
    }

    /// Converts quantity from one unit to another given pack/box multipliers.
    pub fn convert_quantity(
        qty: f64,
        from: UnitOfMeasure,
        to: UnitOfMeasure,
        pieces_per_pack: f64,
        pieces_per_box: f64,
    ) -> f64 {
        if from == to {
            return qty;
        }

        // Convert from source to base pieces
        let in_pieces = match from {
            UnitOfMeasure::Piece => qty,
            UnitOfMeasure::Pack => qty * pieces_per_pack,
            UnitOfMeasure::Box => qty * pieces_per_box,
            UnitOfMeasure::Kilogram | UnitOfMeasure::Meter => qty, // Direct scale for continuous units
        };

        // Convert from pieces to destination
        match to {
            UnitOfMeasure::Piece => in_pieces,
            UnitOfMeasure::Pack => {
                if pieces_per_pack > 0.0 {
                    in_pieces / pieces_per_pack
                } else {
                    in_pieces
                }
            }
            UnitOfMeasure::Box => {
                if pieces_per_box > 0.0 {
                    in_pieces / pieces_per_box
                } else {
                    in_pieces
                }
            }
            UnitOfMeasure::Kilogram | UnitOfMeasure::Meter => in_pieces,
        }
    }
}

/// Catalog item designed for Touch Quick-Pills (non-barcoded bulk hardware items).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuickHardwarePill {
    pub id: &'static str,
    pub name: &'static str,
    pub category: &'static str,
    pub price_cents_per_piece: i64,
    pub pieces_per_pack: usize,
    pub pieces_per_box: usize,
    pub icon: &'static str,
}

pub const DEFAULT_HARDWARE_PILLS: &[QuickHardwarePill] = &[
    QuickHardwarePill {
        id: "screw-wood-4x40",
        name: "Βίδες Νοβοπάν 4×40",
        category: "Βίδες & Στερέωση",
        price_cents_per_piece: 4, // 0.04€
        pieces_per_pack: 50,
        pieces_per_box: 500,
        icon: "🔩",
    },
    QuickHardwarePill {
        id: "screw-drywall-35",
        name: "Βίδες Γυψοσανίδας 3.5×35",
        category: "Βίδες & Στερέωση",
        price_cents_per_piece: 3,
        pieces_per_pack: 100,
        pieces_per_box: 1000,
        icon: "🔩",
    },
    QuickHardwarePill {
        id: "dowel-fisher-6",
        name: "Ούπατ Fischer SX 6mm",
        category: "Βίδες & Στερέωση",
        price_cents_per_piece: 8,
        pieces_per_pack: 50,
        pieces_per_box: 300,
        icon: "🧱",
    },
    QuickHardwarePill {
        id: "cable-nym-3x15",
        name: "Καλώδιο NYM 3×1.5mm (m)",
        category: "Ηλεκτρολογικά",
        price_cents_per_piece: 95, // 0.95€ / m
        pieces_per_pack: 10,
        pieces_per_box: 100,
        icon: "⚡",
    },
    QuickHardwarePill {
        id: "cable-nym-3x25",
        name: "Καλώδιο NYM 3×2.5mm (m)",
        category: "Ηλεκτρολογικά",
        price_cents_per_piece: 145, // 1.45€ / m
        pieces_per_pack: 10,
        pieces_per_box: 100,
        icon: "⚡",
    },
    QuickHardwarePill {
        id: "pipe-fitting-half",
        name: "Μαστός Γαλβανιζέ 1/2\"",
        category: "Υδραυλικά",
        price_cents_per_piece: 120,
        pieces_per_pack: 10,
        pieces_per_box: 50,
        icon: "🚿",
    },
    QuickHardwarePill {
        id: "teflon-tape",
        name: "Τεφλόν Υδραυλικών 12mm",
        category: "Υδραυλικά",
        price_cents_per_piece: 60,
        pieces_per_pack: 10,
        pieces_per_box: 50,
        icon: "⚪",
    },
    QuickHardwarePill {
        id: "sandpaper-p120",
        name: "Γυαλόχαρτο P120 (φύλλο)",
        category: "Χρώματα & Εργαλεία",
        price_cents_per_piece: 50,
        pieces_per_pack: 25,
        pieces_per_box: 100,
        icon: "📜",
    },
];

/// A registered contractor account with current debit balance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContractorAccount {
    pub id: String,
    pub contractor_name: String,
    pub phone: String,
    pub trade: String,
    pub current_balance_cents: i64,
    pub created_at: i64,
}

/// A line item transaction recorded on a specific job site / worksite.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContractorLedgerEntry {
    pub id: String,
    pub contractor_id: String,
    pub worksite: String,
    pub description: String,
    pub debit_cents: i64,  // Material charge
    pub credit_cents: i64, // Payment against balance
    pub created_at: i64,
}

impl ContractorLedgerEntry {
    pub fn new_debit(
        contractor_id: impl Into<String>,
        worksite: impl Into<String>,
        description: impl Into<String>,
        debit_cents: i64,
    ) -> Self {
        Self {
            id: uuid::Uuid::now_v7().to_string(),
            contractor_id: contractor_id.into(),
            worksite: worksite.into(),
            description: description.into(),
            debit_cents,
            credit_cents: 0,
            created_at: chrono::Utc::now().timestamp_millis(),
        }
    }

    pub fn new_credit(
        contractor_id: impl Into<String>,
        worksite: impl Into<String>,
        description: impl Into<String>,
        credit_cents: i64,
    ) -> Self {
        Self {
            id: uuid::Uuid::now_v7().to_string(),
            contractor_id: contractor_id.into(),
            worksite: worksite.into(),
            description: description.into(),
            debit_cents: 0,
            credit_cents,
            created_at: chrono::Utc::now().timestamp_millis(),
        }
    }
}

/// Initializes contractor accounts and job-site sub-ledger SQLite tables.
pub fn init_contractor_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS contractors (
            id TEXT PRIMARY KEY,
            contractor_name TEXT NOT NULL,
            phone TEXT NOT NULL,
            trade TEXT NOT NULL,
            created_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS contractor_ledger (
            id TEXT PRIMARY KEY,
            contractor_id TEXT NOT NULL,
            worksite TEXT NOT NULL,
            description TEXT NOT NULL,
            debit_cents INTEGER NOT NULL DEFAULT 0,
            credit_cents INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL,
            FOREIGN KEY(contractor_id) REFERENCES contractors(id)
        );

        CREATE INDEX IF NOT EXISTS idx_contractor_ledger_cid ON contractor_ledger(contractor_id);
        CREATE INDEX IF NOT EXISTS idx_contractor_ledger_worksite ON contractor_ledger(worksite);
        "#,
    )
}

/// Registers a new contractor in the real SQLite database.
pub fn create_contractor(
    conn: &Connection,
    name: &str,
    phone: &str,
    trade: &str,
) -> Result<ContractorAccount> {
    let id = uuid::Uuid::now_v7().to_string();
    let now = chrono::Utc::now().timestamp_millis();

    conn.execute(
        "INSERT INTO contractors (id, contractor_name, phone, trade, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, name.trim(), phone.trim(), trade.trim(), now],
    )?;

    Ok(ContractorAccount {
        id,
        contractor_name: name.trim().to_string(),
        phone: phone.trim().to_string(),
        trade: trade.trim().to_string(),
        current_balance_cents: 0,
        created_at: now,
    })
}

/// Lists all registered contractors with calculated running balance.
pub fn list_contractors(conn: &Connection) -> Result<Vec<ContractorAccount>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT c.id, c.contractor_name, c.phone, c.trade, c.created_at,
               COALESCE(SUM(l.debit_cents), 0) - COALESCE(SUM(l.credit_cents), 0) AS balance
        FROM contractors c
        LEFT JOIN contractor_ledger l ON c.id = l.contractor_id
        GROUP BY c.id
        ORDER BY c.contractor_name ASC
        "#,
    )?;

    let rows = stmt.query_map([], |row| {
        Ok(ContractorAccount {
            id: row.get(0)?,
            contractor_name: row.get(1)?,
            phone: row.get(2)?,
            trade: row.get(3)?,
            created_at: row.get(4)?,
            current_balance_cents: row.get(5)?,
        })
    })?;

    let mut result = Vec::new();
    for r in rows {
        result.push(r?);
    }
    Ok(result)
}

/// Records a job-site transaction (purchase debit or payment credit) in the sub-ledger.
pub fn add_contractor_entry(conn: &Connection, entry: &ContractorLedgerEntry) -> Result<()> {
    conn.execute(
        r#"
        INSERT INTO contractor_ledger (id, contractor_id, worksite, description, debit_cents, credit_cents, created_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        "#,
        params![
            entry.id,
            entry.contractor_id,
            entry.worksite.trim(),
            entry.description.trim(),
            entry.debit_cents,
            entry.credit_cents,
            entry.created_at,
        ],
    )?;
    Ok(())
}

/// Retrieves all ledger entries for a given contractor, ordered chronologically.
pub fn list_contractor_entries(conn: &Connection, contractor_id: &str) -> Result<Vec<ContractorLedgerEntry>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, contractor_id, worksite, description, debit_cents, credit_cents, created_at
        FROM contractor_ledger
        WHERE contractor_id = ?1
        ORDER BY created_at DESC
        "#,
    )?;

    let rows = stmt.query_map(params![contractor_id], |row| {
        Ok(ContractorLedgerEntry {
            id: row.get(0)?,
            contractor_id: row.get(1)?,
            worksite: row.get(2)?,
            description: row.get(3)?,
            debit_cents: row.get(4)?,
            credit_cents: row.get(5)?,
            created_at: row.get(6)?,
        })
    })?;

    let mut result = Vec::new();
    for r in rows {
        result.push(r?);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_of_measure_conversions() {
        // 500 screws in box of 500 = 1 box
        let boxes = UnitOfMeasure::convert_quantity(500.0, UnitOfMeasure::Piece, UnitOfMeasure::Box, 50.0, 500.0);
        assert!((boxes - 1.0).abs() < 1e-6);

        // 2 packs of 50 = 100 pieces
        let pieces = UnitOfMeasure::convert_quantity(2.0, UnitOfMeasure::Pack, UnitOfMeasure::Piece, 50.0, 500.0);
        assert!((pieces - 100.0).abs() < 1e-6);
    }

    #[test]
    fn test_contractor_ledger_sqlite_workflow() {
        let conn = Connection::open_in_memory().unwrap();
        init_contractor_schema(&conn).unwrap();

        // 1. Create contractor
        let c = create_contractor(&conn, "Γιώργος Ηλεκτρολόγος", "6981234567", "Ηλεκτρολόγος").unwrap();
        assert_eq!(c.contractor_name, "Γιώργος Ηλεκτρολόγος");

        // 2. Add purchase entry for worksite 1
        let entry1 = ContractorLedgerEntry {
            id: uuid::Uuid::now_v7().to_string(),
            contractor_id: c.id.clone(),
            worksite: "Οικοδομή Παγκράτι".into(),
            description: "100m Καλώδιο NYM + Πίνακας Hager".into(),
            debit_cents: 14500, // 145.00€
            credit_cents: 0,
            created_at: chrono::Utc::now().timestamp_millis(),
        };
        add_contractor_entry(&conn, &entry1).unwrap();

        // 3. Add payment entry
        let entry2 = ContractorLedgerEntry {
            id: uuid::Uuid::now_v7().to_string(),
            contractor_id: c.id.clone(),
            worksite: "Οικοδομή Παγκράτι".into(),
            description: "Πληρωμή έναντι μετρητά".into(),
            debit_cents: 0,
            credit_cents: 10000, // 100.00€
            created_at: chrono::Utc::now().timestamp_millis() + 10,
        };
        add_contractor_entry(&conn, &entry2).unwrap();

        // 4. Verify balance is 45.00€ (4500 cents)
        let list = list_contractors(&conn).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].current_balance_cents, 4500);

        // 5. Verify entries count
        let entries = list_contractor_entries(&conn, &c.id).unwrap();
        assert_eq!(entries.len(), 2);
    }
}
