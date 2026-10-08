//! Thermal Receipt Printing & ESC/POS Generator for Proteus BOS.
//! Directly integrates with Windows Print Spooler (winspool.drv RAW) to bypass USB driver locks.

use crate::tickets::ServiceTicket;
use serde::{Deserialize, Serialize};

/// Paper width specification for thermal receipt printers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaperWidth {
    Width58mm, // 32 characters per line
    Width80mm, // 42-48 characters per line
}

impl PaperWidth {
    pub fn columns(&self) -> usize {
        match self {
            PaperWidth::Width58mm => 32,
            PaperWidth::Width80mm => 48,
        }
    }
}

/// Metadata about the store printed in the ticket header and top navigation bar.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopReceiptConfig {
    pub shop_name: String,
    pub legal_name: String,
    pub afm: String,
    pub doy: String,
    pub activity_description: String,
    pub address: String,
    pub phone: String,
    pub branch_code: i32,
    pub mydata_user_id: String,
    pub mydata_subscription_key: String,
    pub mydata_sandbox: bool,
    pub footer_message: String,
    pub paper_width: PaperWidth,
    pub logo_icon: String,
}

impl Default for ShopReceiptConfig {
    fn default() -> Self {
        Self {
            shop_name: "PROTEUS SERVICE LAB".to_string(),
            legal_name: "PROTEUS MONOPROSOPI IKE".to_string(),
            afm: "802194512".to_string(),
            doy: "Δ' ΑΘΗΝΩΝ".to_string(),
            activity_description: "ΥΠΗΡΕΣΙΕΣ ΠΛΗΡΟΦΟΡΙΚΗΣ & ΤΕΧΝΙΚΗ ΥΠΟΣΤΗΡΙΞΗ".to_string(),
            address: "Τεχνικό Κέντρο Επισκευών".to_string(),
            phone: "+30 210 1234567".to_string(),
            branch_code: 0,
            mydata_user_id: String::new(),
            mydata_subscription_key: String::new(),
            mydata_sandbox: true,
            footer_message: "Ευχαριστούμε για την προτίμηση!\nΦυλάξτε το παρόν δελτίο παραλαβής.".to_string(),
            paper_width: PaperWidth::Width80mm,
            logo_icon: "default".to_string(),
        }
    }
}

/// Initializes the shop_settings table in SQLite for 100% real persistence of store branding & fiscal profile.
pub fn init_shop_settings_schema(conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS shop_settings (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            shop_name TEXT NOT NULL,
            legal_name TEXT NOT NULL DEFAULT '',
            afm TEXT NOT NULL DEFAULT '',
            doy TEXT NOT NULL DEFAULT '',
            activity_description TEXT NOT NULL DEFAULT '',
            address TEXT NOT NULL,
            phone TEXT NOT NULL,
            branch_code INTEGER NOT NULL DEFAULT 0,
            mydata_user_id TEXT NOT NULL DEFAULT '',
            mydata_subscription_key TEXT NOT NULL DEFAULT '',
            mydata_sandbox INTEGER NOT NULL DEFAULT 1,
            footer_message TEXT NOT NULL,
            paper_width TEXT NOT NULL,
            logo_icon TEXT NOT NULL DEFAULT 'default',
            updated_at TEXT NOT NULL
        );"
    )?;
    let _ = conn.execute("ALTER TABLE shop_settings ADD COLUMN legal_name TEXT NOT NULL DEFAULT ''", []);
    let _ = conn.execute("ALTER TABLE shop_settings ADD COLUMN afm TEXT NOT NULL DEFAULT ''", []);
    let _ = conn.execute("ALTER TABLE shop_settings ADD COLUMN doy TEXT NOT NULL DEFAULT ''", []);
    let _ = conn.execute("ALTER TABLE shop_settings ADD COLUMN activity_description TEXT NOT NULL DEFAULT ''", []);
    let _ = conn.execute("ALTER TABLE shop_settings ADD COLUMN branch_code INTEGER NOT NULL DEFAULT 0", []);
    let _ = conn.execute("ALTER TABLE shop_settings ADD COLUMN mydata_user_id TEXT NOT NULL DEFAULT ''", []);
    let _ = conn.execute("ALTER TABLE shop_settings ADD COLUMN mydata_subscription_key TEXT NOT NULL DEFAULT ''", []);
    let _ = conn.execute("ALTER TABLE shop_settings ADD COLUMN mydata_sandbox INTEGER NOT NULL DEFAULT 1", []);
    Ok(())
}

/// Loads the persistent store configuration from SQLite, falling back to defaults if not yet created.
pub fn load_shop_config(conn: &rusqlite::Connection) -> ShopReceiptConfig {
    let _ = init_shop_settings_schema(conn);
    let mut stmt = match conn.prepare(
        "SELECT shop_name, address, phone, footer_message, paper_width, logo_icon,
                legal_name, afm, doy, activity_description, branch_code,
                mydata_user_id, mydata_subscription_key, mydata_sandbox
         FROM shop_settings WHERE id = 1"
    ) {
        Ok(s) => s,
        Err(_) => return ShopReceiptConfig::default(),
    };

    let res = stmt.query_row([], |row| {
        let width_str: String = row.get(4)?;
        let paper_width = if width_str == "58mm" {
            PaperWidth::Width58mm
        } else {
            PaperWidth::Width80mm
        };
        let sandbox_int: i32 = row.get(13).unwrap_or(1);
        Ok(ShopReceiptConfig {
            shop_name: row.get(0)?,
            address: row.get(1)?,
            phone: row.get(2)?,
            footer_message: row.get(3)?,
            paper_width,
            logo_icon: row.get(5).unwrap_or_else(|_| "default".to_string()),
            legal_name: row.get(6).unwrap_or_default(),
            afm: row.get(7).unwrap_or_default(),
            doy: row.get(8).unwrap_or_default(),
            activity_description: row.get(9).unwrap_or_default(),
            branch_code: row.get(10).unwrap_or(0),
            mydata_user_id: row.get(11).unwrap_or_default(),
            mydata_subscription_key: row.get(12).unwrap_or_default(),
            mydata_sandbox: sandbox_int != 0,
        })
    });

    res.unwrap_or_default()
}

/// Saves the store configuration and custom branding permanently into SQLite.
pub fn save_shop_config(conn: &rusqlite::Connection, config: &ShopReceiptConfig) -> Result<(), rusqlite::Error> {
    let _ = init_shop_settings_schema(conn);
    let width_str = match config.paper_width {
        PaperWidth::Width58mm => "58mm",
        PaperWidth::Width80mm => "80mm",
    };
    let now = chrono::Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO shop_settings (
            id, shop_name, address, phone, footer_message, paper_width, logo_icon,
            legal_name, afm, doy, activity_description, branch_code,
            mydata_user_id, mydata_subscription_key, mydata_sandbox, updated_at
        ) VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
        ON CONFLICT(id) DO UPDATE SET
            shop_name = excluded.shop_name,
            address = excluded.address,
            phone = excluded.phone,
            footer_message = excluded.footer_message,
            paper_width = excluded.paper_width,
            logo_icon = excluded.logo_icon,
            legal_name = excluded.legal_name,
            afm = excluded.afm,
            doy = excluded.doy,
            activity_description = excluded.activity_description,
            branch_code = excluded.branch_code,
            mydata_user_id = excluded.mydata_user_id,
            mydata_subscription_key = excluded.mydata_subscription_key,
            mydata_sandbox = excluded.mydata_sandbox,
            updated_at = excluded.updated_at;",
        rusqlite::params![
            config.shop_name,
            config.address,
            config.phone,
            config.footer_message,
            width_str,
            config.logo_icon,
            config.legal_name,
            config.afm,
            config.doy,
            config.activity_description,
            config.branch_code,
            config.mydata_user_id,
            config.mydata_subscription_key,
            if config.mydata_sandbox { 1 } else { 0 },
            now,
        ],
    )?;
    Ok(())
}

/// Formats text into a horizontal key-value row with dots padding.
/// E.g. "Συσκευή ........ Samsung S22"
pub fn format_row(key: &str, val: &str, max_width: usize) -> String {
    let key_len = key.chars().count();
    let val_len = val.chars().count();
    if key_len + val_len + 2 >= max_width {
        // Wrap on two lines
        return format!("{}\n  {}\n", key, val);
    }
    let pad_len = max_width.saturating_sub(key_len + val_len + 2);
    let dots: String = ".".repeat(pad_len);
    format!("{} {} {}\n", key, dots, val)
}

/// Generates raw ESC/POS bytes for a Service Intake Ticket.
pub fn generate_intake_receipt(ticket: &ServiceTicket, config: &ShopReceiptConfig) -> Vec<u8> {
    let mut out = Vec::with_capacity(512);
    let cols = config.paper_width.columns();
    let divider = "-".repeat(cols);

    // ESC @: Initialize printer
    out.extend_from_slice(b"\x1B\x40");

    // Center alignment
    out.extend_from_slice(b"\x1B\x61\x01");

    // Sovereign Brand Tagline
    out.extend_from_slice(b"[ PROTEUS BUSINESS OS ]\n");

    // Double-height header for shop name
    out.extend_from_slice(b"\x1B\x45\x01"); // Bold on
    out.extend_from_slice(b"\x1D\x21\x11"); // Double size
    out.extend_from_slice(config.shop_name.as_bytes());
    out.extend_from_slice(b"\n");
    out.extend_from_slice(b"\x1D\x21\x00"); // Normal size
    out.extend_from_slice(b"\x1B\x45\x00"); // Bold off

    if !config.afm.is_empty() {
        let tax_hdr = if config.doy.is_empty() {
            format!("ΑΦΜ: {}\n", config.afm)
        } else {
            format!("ΑΦΜ: {} - ΔΟΥ: {}\n", config.afm, config.doy)
        };
        out.extend_from_slice(tax_hdr.as_bytes());
    }
    if !config.address.is_empty() {
        out.extend_from_slice(config.address.as_bytes());
        out.extend_from_slice(b"\n");
    }
    if !config.phone.is_empty() {
        out.extend_from_slice(format!("Τηλ: {}\n", config.phone).as_bytes());
    }

    out.extend_from_slice(divider.as_bytes());
    out.extend_from_slice(b"\n");

    // Ticket Number Box
    out.extend_from_slice(b"\x1B\x45\x01");
    out.extend_from_slice(b"\x1D\x21\x11");
    out.extend_from_slice(format!("ΔΕΛΤΙΟ # {}\n", ticket.ticket_number).as_bytes());
    out.extend_from_slice(b"\x1D\x21\x00");
    out.extend_from_slice(b"\x1B\x45\x00");

    let dt = chrono::DateTime::from_timestamp_millis(ticket.created_at)
        .map(|d| d.format("%d/%m/%Y %H:%M").to_string())
        .unwrap_or_else(|| "N/A".to_string());
    out.extend_from_slice(format!("Ημερομηνία: {}\n", dt).as_bytes());

    out.extend_from_slice(divider.as_bytes());
    out.extend_from_slice(b"\n");

    // Left alignment for customer/device details
    out.extend_from_slice(b"\x1B\x61\x00");

    out.extend_from_slice(format_row("Πελάτης", &ticket.customer_name, cols).as_bytes());
    out.extend_from_slice(format_row("Τηλέφωνο", &ticket.customer_phone, cols).as_bytes());
    out.extend_from_slice(format_row("Συσκευή", &ticket.device_model, cols).as_bytes());
    if let Some(sn) = &ticket.serial_number {
        if !sn.is_empty() {
            out.extend_from_slice(format_row("S/N", sn, cols).as_bytes());
        }
    }

    out.extend_from_slice(divider.as_bytes());
    out.extend_from_slice(b"\n");

    out.extend_from_slice(b"\x1B\x45\x01");
    out.extend_from_slice("Περιγραφή Βλάβης:\n".as_bytes());
    out.extend_from_slice(b"\x1B\x45\x00");
    out.extend_from_slice(format!("  {}\n", ticket.reported_fault).as_bytes());

    if ticket.estimated_cost > 0.0 {
        out.extend_from_slice(divider.as_bytes());
        out.extend_from_slice(b"\n");
        out.extend_from_slice(format_row("Εκτίμηση Κόστους", &format!("{:.2} EUR", ticket.estimated_cost), cols).as_bytes());
    }

    out.extend_from_slice(divider.as_bytes());
    out.extend_from_slice(b"\n");

    // Center alignment for barcode / footer
    out.extend_from_slice(b"\x1B\x61\x01");

    // Footer message
    if !config.footer_message.is_empty() {
        out.extend_from_slice(config.footer_message.as_bytes());
        out.extend_from_slice(b"\n");
    }
    out.extend_from_slice(b"-- Powered by Proteus Business OS --\n");

    // Feed lines & Cut paper (GS V 66 0)
    out.extend_from_slice(b"\n\n\n\x1D\x56\x41\x00");

    out
}

pub use crate::printer_spooler::{print_raw_bytes, test_printer_connection};

/// Triggers a pulse on the cash drawer RJ-11/RJ-12 port connected to the receipt printer.
/// ESC p m t1 t2 command: 0x1B 0x70 0x00 0x19 0xFA (pulse to pin 2)
pub fn kick_cash_drawer(printer_name: &str) -> Result<(), String> {
    if printer_name.trim().is_empty() {
        return Err("Δεν έχει οριστεί όνομα εκτυπωτή για το συρτάρι".to_string());
    }
    let pulse_bytes = b"\x1B\x70\x00\x19\xFA";
    crate::printer_spooler::print_raw_bytes(printer_name, "Proteus Cash Drawer Kick", pulse_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_intake_receipt_contains_key_data() {
        let ticket = ServiceTicket::new("Γιώργος", "6900000000", "iPhone 14", "Μπαταρία");
        let config = ShopReceiptConfig::default();
        let bytes = generate_intake_receipt(&ticket, &config);

        assert!(!bytes.is_empty());
        // Check for ESC @ init command
        assert_eq!(&bytes[0..2], b"\x1B\x40");
        // Check for cut command at end
        assert!(bytes.ends_with(b"\x1D\x56\x41\x00"));
    }

    #[test]
    fn test_format_row_padding() {
        let row = format_row("Συσκευή", "iPhone", 32);
        assert!(row.contains("Συσκευή"));
        assert!(row.contains("iPhone"));
        assert_eq!(row.chars().filter(|&c| c == '\n').count(), 1);
    }

    #[test]
    fn test_printer_connection_empty_name() {
        assert!(test_printer_connection("").is_err());
        assert!(test_printer_connection("   ").is_err());
    }

    #[test]
    fn test_cash_drawer_empty_name() {
        assert!(kick_cash_drawer("").is_err());
    }

    #[test]
    fn test_shop_config_sqlite_persistence() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        let mut config = load_shop_config(&conn);
        assert_eq!(config.shop_name, "PROTEUS SERVICE LAB");
        assert_eq!(config.logo_icon, "default");

        config.shop_name = "AUTO MOTO HELLAS".to_string();
        config.logo_icon = "wrench".to_string();
        config.afm = "094014201".to_string();
        config.doy = "A' ATHINON".to_string();
        config.legal_name = "AUTO MOTO HELLAS AE".to_string();
        config.mydata_user_id = "user123".to_string();
        save_shop_config(&conn, &config).unwrap();

        let loaded = load_shop_config(&conn);
        assert_eq!(loaded.shop_name, "AUTO MOTO HELLAS");
        assert_eq!(loaded.logo_icon, "wrench");
        assert_eq!(loaded.afm, "094014201");
        assert_eq!(loaded.doy, "A' ATHINON");
        assert_eq!(loaded.legal_name, "AUTO MOTO HELLAS AE");
        assert_eq!(loaded.mydata_user_id, "user123");
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn test_printer_and_drawer_mock_success() {
        assert!(test_printer_connection("MockPrinter").is_ok());
        assert!(kick_cash_drawer("MockPrinter").is_ok());
    }
}
