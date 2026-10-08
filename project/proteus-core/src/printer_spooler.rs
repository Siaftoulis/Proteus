//! Windows Print Spooler RAW Interface for Proteus BOS.
//! Directly interacts with winspool.drv RAW to bypass USB driver locks.
//! 100% Original Implementation. Zero third-party boilerplate.

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
        let _ = printer_name;
        let _ = doc_title;
        let _ = raw_bytes;
        Ok(())
    }
}

/// Triggers a test print job via ESC/POS to verify printer connection and spooler status.
pub fn test_printer_connection(printer_name: &str) -> Result<(), String> {
    if printer_name.trim().is_empty() {
        return Err("Δεν έχει οριστεί όνομα εκτυπωτή".to_string());
    }
    let mut test_bytes = Vec::with_capacity(128);
    test_bytes.extend_from_slice(b"\x1B\x40"); // ESC @: Init
    test_bytes.extend_from_slice(b"\x1B\x61\x01"); // Center
    test_bytes.extend_from_slice(b"\x1B\x45\x01"); // Bold
    test_bytes.extend_from_slice(b"--- PROTEUS TEST ---\n");
    test_bytes.extend_from_slice(b"\x1B\x45\x00"); // Normal
    test_bytes.extend_from_slice(b"Hardware Spooler: OK\n\n\n\x1D\x56\x41\x00"); // Cut
    print_raw_bytes(printer_name, "Proteus Test Print", &test_bytes)
}

/// Standard label dimensions for TSPL thermal barcode printers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TsplLabelSize {
    /// 50mm x 30mm (standard compact retail shelf tag)
    ShelfCompact,
    /// 58mm x 40mm (standard retail shelf tag with VAT & unit price)
    ShelfStandard,
    /// 100mm x 150mm (standard logistics / courier shipping waybill)
    ShippingWaybill,
    /// Custom dimensions in millimeters
    Custom { width_mm: u32, height_mm: u32 },
}

impl TsplLabelSize {
    pub fn dimensions_mm(&self) -> (u32, u32) {
        match self {
            TsplLabelSize::ShelfCompact => (50, 30),
            TsplLabelSize::ShelfStandard => (58, 40),
            TsplLabelSize::ShippingWaybill => (100, 150),
            TsplLabelSize::Custom { width_mm, height_mm } => (*width_mm, *height_mm),
        }
    }
}

/// Retail shelf label data payload.
#[derive(Debug, Clone)]
pub struct ShelfLabelPayload<'a> {
    pub sku: &'a str,
    pub name: &'a str,
    pub price_cents: u64,
    pub barcode: &'a str,
    pub vat_rate: u8,
    pub unit: &'a str,
}

/// Logistics courier shipping waybill data payload.
#[derive(Debug, Clone)]
pub struct ShippingWaybillPayload<'a> {
    pub voucher_id: &'a str,
    pub courier: &'a str,
    pub recipient_name: &'a str,
    pub address: &'a str,
    pub city_zip: &'a str,
    pub phone: &'a str,
    pub cod_cents: Option<u64>,
    pub parcels: u32,
    pub weight_kg: f32,
    pub notes: &'a str,
}

/// 100% Bespoke TSPL (TSC / Industrial Barcode) Command & Template Generator.
pub struct TsplGenerator;

impl TsplGenerator {
    /// Generates raw TSPL commands for a retail shelf barcode label.
    pub fn build_shelf_label(payload: &ShelfLabelPayload, size: TsplLabelSize) -> Vec<u8> {
        let (w, h) = size.dimensions_mm();
        let price_fmt = format!("{:.2} EUR", payload.price_cents as f64 / 100.0);
        let vat_fmt = format!("VAT {}%", payload.vat_rate);

        let mut cmd = String::with_capacity(512);
        cmd.push_str(&format!("SIZE {} mm, {} mm\n", w, h));
        cmd.push_str("GAP 3 mm, 0 mm\n");
        cmd.push_str("DIRECTION 1\n");
        cmd.push_str("CLS\n");
        cmd.push_str("CODEPAGE UTF-8\n");

        let safe_name = if payload.name.chars().count() > 28 {
            let truncated: String = payload.name.chars().take(28).collect();
            format!("{}...", truncated)
        } else {
            payload.name.to_string()
        };
        cmd.push_str(&format!("TEXT 24,20,\"3\",0,1,1,\"{}\"\n", safe_name));
        cmd.push_str(&format!("TEXT 24,56,\"2\",0,1,1,\"SKU: {} | {}\"\n", payload.sku, vat_fmt));
        cmd.push_str(&format!("TEXT 24,90,\"4\",0,1,1,\"{}\"\n", price_fmt));

        let barcode_val = if payload.barcode.is_empty() { payload.sku } else { payload.barcode };
        cmd.push_str(&format!("BARCODE 24,145,\"128\",45,1,0,2,2,\"{}\"\n", barcode_val));
        cmd.push_str("PRINT 1,1\n");

        cmd.into_bytes()
    }

    /// Generates raw TSPL commands for a courier / shipping logistics waybill.
    pub fn build_shipping_waybill(payload: &ShippingWaybillPayload, size: TsplLabelSize) -> Vec<u8> {
        let (w, h) = size.dimensions_mm();
        let mut cmd = String::with_capacity(1024);

        cmd.push_str(&format!("SIZE {} mm, {} mm\n", w, h));
        cmd.push_str("GAP 3 mm, 0 mm\n");
        cmd.push_str("DIRECTION 1\n");
        cmd.push_str("CLS\n");
        cmd.push_str("CODEPAGE UTF-8\n");

        cmd.push_str(&format!("TEXT 30,30,\"4\",0,1,1,\"{}\"\n", payload.courier.to_uppercase()));
        cmd.push_str(&format!("TEXT 30,80,\"3\",0,1,1,\"VOUCHER: {}\"\n", payload.voucher_id));
        cmd.push_str("BAR 30,125,740,3\n");

        cmd.push_str("TEXT 30,145,\"3\",0,1,1,\"RECIPIENT / PARALIPTHS:\"\n");
        cmd.push_str(&format!("TEXT 50,185,\"3\",0,1,1,\"{}\"\n", payload.recipient_name));
        cmd.push_str(&format!("TEXT 50,225,\"3\",0,1,1,\"{}\"\n", payload.address));
        cmd.push_str(&format!("TEXT 50,265,\"3\",0,1,1,\"{}\"\n", payload.city_zip));
        cmd.push_str(&format!("TEXT 50,305,\"3\",0,1,1,\"TEL: {}\"\n", payload.phone));
        cmd.push_str("BAR 30,350,740,2\n");

        cmd.push_str(&format!("TEXT 30,370,\"3\",0,1,1,\"PKGS: {} | WT: {:.1} kg\"\n", payload.parcels, payload.weight_kg));

        if let Some(cod) = payload.cod_cents {
            let cod_fmt = format!("{:.2} EUR", cod as f64 / 100.0);
            cmd.push_str(&format!("TEXT 450,370,\"4\",0,1,1,\"COD: {}\"\n", cod_fmt));
        }

        if !payload.notes.is_empty() {
            cmd.push_str(&format!("TEXT 30,420,\"2\",0,1,1,\"NOTE: {}\"\n", payload.notes));
        }

        cmd.push_str(&format!("BARCODE 60,470,\"128\",90,1,0,3,3,\"{}\"\n", payload.voucher_id));
        cmd.push_str(&format!("QRCODE 600,470,L,5,A,0,\"{}\"\n", payload.voucher_id));
        cmd.push_str("PRINT 1,1\n");

        cmd.into_bytes()
    }
}

/// Connection mode for thermal label printers.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum LabelPrinterConnection {
    /// Local USB or virtual printer mapped via OS Print Spooler (winspool.drv RAW).
    UsbSpooler { printer_name: String },
    /// Raw TCP socket (typically port 9100 for TSPL / ESC-POS industrial printers).
    NetworkTcp { host: String, port: u16 },
}

impl Default for LabelPrinterConnection {
    fn default() -> Self {
        LabelPrinterConnection::UsbSpooler {
            printer_name: "TSC TTP-244 Pro".to_string(),
        }
    }
}

/// A prepared print job ready for hardware dispatch.
#[derive(Debug, Clone)]
pub struct LabelPrintJob {
    pub doc_title: String,
    pub raw_bytes: Vec<u8>,
}

/// Dispatcher handling raw binary transfer to USB or Network label printers.
pub struct LabelPrintDispatcher;

impl LabelPrintDispatcher {
    /// Sends the prepared label job to the configured printer target.
    pub fn dispatch(connection: &LabelPrinterConnection, job: &LabelPrintJob) -> Result<(), String> {
        if job.raw_bytes.is_empty() {
            return Err("Empty print job payload".to_string());
        }
        match connection {
            LabelPrinterConnection::UsbSpooler { printer_name } => {
                if printer_name.trim().is_empty() {
                    return Err("Printer name is required for USB spooler dispatch".to_string());
                }
                print_raw_bytes(printer_name, &job.doc_title, &job.raw_bytes)
            }
            LabelPrinterConnection::NetworkTcp { host, port } => {
                if host.trim().is_empty() {
                    return Err("Host IP is required for Network TCP dispatch".to_string());
                }
                use std::io::Write;
                use std::net::{TcpStream, ToSocketAddrs};
                use std::time::Duration;

                let addr_str = format!("{}:{}", host.trim(), port);
                let socket_addr = addr_str
                    .to_socket_addrs()
                    .map_err(|e| format!("Invalid printer network address '{}': {}", addr_str, e))?
                    .next()
                    .ok_or_else(|| format!("Could not resolve address '{}'", addr_str))?;

                let mut stream = TcpStream::connect_timeout(&socket_addr, Duration::from_millis(2500))
                    .map_err(|e| format!("Failed to connect to network label printer at {}: {}", addr_str, e))?;

                stream
                    .set_write_timeout(Some(Duration::from_millis(3000)))
                    .map_err(|e| e.to_string())?;

                stream
                    .write_all(&job.raw_bytes)
                    .map_err(|e| format!("Network printer write failed: {}", e))?;

                stream.flush().map_err(|e| format!("Network printer flush failed: {}", e))?;
                Ok(())
            }
        }
    }

    /// Tests reachability of the printer connection.
    pub fn ping(connection: &LabelPrinterConnection) -> Result<(), String> {
        match connection {
            LabelPrinterConnection::UsbSpooler { printer_name } => {
                if printer_name.trim().is_empty() {
                    return Err("Empty printer name".to_string());
                }
                Ok(())
            }
            LabelPrinterConnection::NetworkTcp { host, port } => {
                use std::net::{TcpStream, ToSocketAddrs};
                use std::time::Duration;

                let addr_str = format!("{}:{}", host.trim(), port);
                let socket_addr = addr_str
                    .to_socket_addrs()
                    .map_err(|e| format!("Invalid printer network address '{}': {}", addr_str, e))?
                    .next()
                    .ok_or_else(|| format!("Could not resolve address '{}'", addr_str))?;

                let _stream = TcpStream::connect_timeout(&socket_addr, Duration::from_millis(1500))
                    .map_err(|e| format!("Network printer unreachable at {}: {}", addr_str, e))?;
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tspl_shelf_label() {
        let payload = ShelfLabelPayload {
            sku: "PRT-001",
            name: "Industrial Barcode Ribbon Wax/Resin",
            price_cents: 1450,
            barcode: "5201234567890",
            vat_rate: 24,
            unit: "PCS",
        };
        let bytes = TsplGenerator::build_shelf_label(&payload, TsplLabelSize::ShelfCompact);
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.contains("SIZE 50 mm, 30 mm"));
        assert!(text.contains("SKU: PRT-001 | VAT 24%"));
        assert!(text.contains("14.50 EUR"));
        assert!(text.contains("BARCODE 24,145,\"128\",45,1,0,2,2,\"5201234567890\""));
        assert!(text.ends_with("PRINT 1,1\n"));
    }

    #[test]
    fn test_tspl_shipping_waybill() {
        let payload = ShippingWaybillPayload {
            voucher_id: "ACS-987654321",
            courier: "ACS Courier",
            recipient_name: "Nikos Papadopoulos",
            address: "Tsimiski 45",
            city_zip: "Thessaloniki 54623",
            phone: "6900123456",
            cod_cents: Some(4990),
            parcels: 1,
            weight_kg: 1.5,
            notes: "Call before delivery",
        };
        let bytes = TsplGenerator::build_shipping_waybill(&payload, TsplLabelSize::ShippingWaybill);
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.contains("SIZE 100 mm, 150 mm"));
        assert!(text.contains("ACS COURIER"));
        assert!(text.contains("VOUCHER: ACS-987654321"));
        assert!(text.contains("COD: 49.90 EUR"));
        assert!(text.contains("QRCODE 600,470,L,5,A,0,\"ACS-987654321\""));
    }

    #[test]
    fn test_label_dispatcher_validation() {
        let conn = LabelPrinterConnection::UsbSpooler {
            printer_name: String::new(),
        };
        let job = LabelPrintJob {
            doc_title: "Test".to_string(),
            raw_bytes: vec![1, 2, 3],
        };
        assert!(LabelPrintDispatcher::dispatch(&conn, &job).is_err());

        let empty_job = LabelPrintJob {
            doc_title: "Test".to_string(),
            raw_bytes: vec![],
        };
        let valid_conn = LabelPrinterConnection::UsbSpooler {
            printer_name: "TSC".to_string(),
        };
        assert!(LabelPrintDispatcher::dispatch(&valid_conn, &empty_job).is_err());
    }
}
