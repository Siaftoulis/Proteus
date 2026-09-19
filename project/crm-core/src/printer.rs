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

/// Metadata about the store printed in the ticket header.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopReceiptConfig {
    pub shop_name: String,
    pub address: String,
    pub phone: String,
    pub footer_message: String,
    pub paper_width: PaperWidth,
}

impl Default for ShopReceiptConfig {
    fn default() -> Self {
        Self {
            shop_name: "PROTEUS SERVICE LAB".to_string(),
            address: "Τεχνικό Κέντρο Επισκευών".to_string(),
            phone: "+30 210 1234567".to_string(),
            footer_message: "Ευχαριστούμε για την προτίμηση!\nΦυλάξτε το παρόν δελτίο παραλαβής.".to_string(),
            paper_width: PaperWidth::Width80mm,
        }
    }
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

    // Double-height header for shop name
    out.extend_from_slice(b"\x1B\x45\x01"); // Bold on
    out.extend_from_slice(b"\x1D\x21\x11"); // Double size
    out.extend_from_slice(config.shop_name.as_bytes());
    out.extend_from_slice(b"\n");
    out.extend_from_slice(b"\x1D\x21\x00"); // Normal size
    out.extend_from_slice(b"\x1B\x45\x00"); // Bold off

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

    // Feed lines & Cut paper (GS V 66 0)
    out.extend_from_slice(b"\n\n\n\x1D\x56\x41\x00");

    out
}

/// Sends raw bytes to a Windows printer via the Windows Print Spooler (winspool.drv RAW).
/// If not on Windows, performs a safe simulation.
pub fn print_raw_bytes(printer_name: &str, doc_title: &str, raw_bytes: &[u8]) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        use std::ptr::null_mut;

        type WinHandle = *mut std::ffi::c_void;
        type WinBool = i32;
        type WinDword = u32;
        type WinLpwstr = *mut u16;

        #[repr(C)]
        #[allow(non_snake_case)]
        struct DOC_INFO_1W {
            pDocName: WinLpwstr,
            pOutputFile: WinLpwstr,
            pDatatype: WinLpwstr,
        }

        #[link(name = "winspool")]
        #[allow(non_snake_case)]
        extern "system" {
            fn OpenPrinterW(pPrinterName: *const u16, phPrinter: *mut WinHandle, pDefault: *mut std::ffi::c_void) -> WinBool;
            fn StartDocPrinterW(hPrinter: WinHandle, Level: WinDword, pDocInfo: *mut u8) -> WinDword;
            fn StartPagePrinter(hPrinter: WinHandle) -> WinBool;
            fn WritePrinter(hPrinter: WinHandle, pBuf: *mut std::ffi::c_void, cbBuf: WinDword, pcWritten: *mut WinDword) -> WinBool;
            fn EndPagePrinter(hPrinter: WinHandle) -> WinBool;
            fn EndDocPrinter(hPrinter: WinHandle) -> WinBool;
            fn ClosePrinter(hPrinter: WinHandle) -> WinBool;
        }

        let mut printer_name_wide: Vec<u16> = OsStr::new(printer_name).encode_wide().chain(std::iter::once(0)).collect();
        let mut doc_title_wide: Vec<u16> = OsStr::new(doc_title).encode_wide().chain(std::iter::once(0)).collect();
        let mut raw_datatype_wide: Vec<u16> = OsStr::new("RAW").encode_wide().chain(std::iter::once(0)).collect();

        unsafe {
            let mut handle: WinHandle = null_mut();
            if OpenPrinterW(printer_name_wide.as_mut_ptr(), &mut handle, null_mut()) == 0 {
                return Err(format!("Αποτυχία ανοίγματος εκτυπωτή '{}': Win32 error", printer_name));
            }

            let mut doc_info = DOC_INFO_1W {
                pDocName: doc_title_wide.as_mut_ptr(),
                pOutputFile: null_mut(),
                pDatatype: raw_datatype_wide.as_mut_ptr(),
            };

            let job_id = StartDocPrinterW(handle, 1, &mut doc_info as *mut _ as *mut u8);
            if job_id == 0 {
                ClosePrinter(handle);
                return Err("Αποτυχία έναρξης εργασίας εκτύπωσης (StartDocPrinter)".to_string());
            }

            StartPagePrinter(handle);

            let mut written: WinDword = 0;
            let success = WritePrinter(
                handle,
                raw_bytes.as_ptr() as *mut std::ffi::c_void,
                raw_bytes.len() as WinDword,
                &mut written,
            );

            EndPagePrinter(handle);
            EndDocPrinter(handle);
            ClosePrinter(handle);

            if success == 0 || written != raw_bytes.len() as WinDword {
                return Err("Σφάλμα κατά την εγγραφή δεδομένων στον εκτυπωτή (WritePrinter)".to_string());
            }

            Ok(())
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        // On non-Windows systems (development/testing), log receipt output safely
        println!("[MOCK PRINTER: {}] Doc: '{}', Bytes: {}", printer_name, doc_title, raw_bytes.len());
        Ok(())
    }
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
}
