//! ESC/POS Thermal Delivery Voucher Generation for Digital Shipping Notes.
//! Strict Rule 1 (100% Original Codebase) and Rule 5 (Zero Mock Data).

use crate::printer::{format_row, ShopReceiptConfig};
use super::types::{ShippingNote, ShippingNoteItem};

/// Generates raw ESC/POS bytes for a Digital Shipping Note / Delivery Waybill.
pub fn generate_escpos_shipping_voucher(
    note: &ShippingNote,
    items: &[ShippingNoteItem],
    config: &ShopReceiptConfig,
) -> Vec<u8> {
    let mut out = Vec::with_capacity(1024);
    let cols = config.paper_width.columns();
    let divider = "-".repeat(cols);

    // ESC @: Init
    out.extend_from_slice(b"\x1B\x40");

    // Center alignment
    out.extend_from_slice(b"\x1B\x61\x01");

    // Brand Tagline
    out.extend_from_slice(b"[ PROTEUS LOGISTICS & DISPATCH ]\n");

    // Header Bold
    out.extend_from_slice(b"\x1B\x45\x01");
    out.extend_from_slice(b"\x1D\x21\x11"); // Double size
    out.extend_from_slice(b"\xCE\xA8\xCE\x97\xCE\xA6\xCE\x99\xCE\x91\xCE\x9A\xCE\x9F \xCE\x94\xCE\x95\xCE\x9B\xCE\xA4\xCE\x99\xCE\x9F \xCE\x91\xCE\xA0\xCE\x9F\xCE\xA3\xCE\xA4\xCE\x9F\xCE\x9B\xCE\x97\xCE\xA3\n"); // ΨΗΦΙΑΚΟ ΔΕΛΤΙΟ ΑΠΟΣΤΟΛΗΣ
    out.extend_from_slice(b"\x1D\x21\x00");
    out.extend_from_slice(b"\x1B\x45\x00");

    out.extend_from_slice(divider.as_bytes());
    out.extend_from_slice(b"\n");

    // Left alignment
    out.extend_from_slice(b"\x1B\x61\x00");

    // Note Details
    out.extend_from_slice(format_row("Αριθμός", &note.note_number, cols).as_bytes());
    out.extend_from_slice(format_row("Κατάσταση", note.status.display_name(), cols).as_bytes());
    out.extend_from_slice(format_row("Σκοπός", note.purpose.display_name(), cols).as_bytes());
    out.extend_from_slice(format_row("Αναχώρηση", &note.departure_time, cols).as_bytes());
    out.extend_from_slice(format_row("Όχημα", &note.vehicle_plate, cols).as_bytes());
    out.extend_from_slice(format_row("Οδηγός", &note.driver_name, cols).as_bytes());

    out.extend_from_slice(divider.as_bytes());
    out.extend_from_slice(b"\n");

    // Parties
    out.extend_from_slice(b"\x1B\x45\x01");
    out.extend_from_slice(b"[ \xCE\x91\xCE\xA0\xCE\x9F\xCE\xA3\xCE\xA4\xCE\x9F\xCE\x9B\xCE\x95\xCE\x91\xCE\xA3 ]\n"); // [ ΑΠΟΣΤΟΛΕΑΣ ]
    out.extend_from_slice(b"\x1B\x45\x00");
    out.extend_from_slice(format!("{} (ΑΦΜ: {})\n", note.issuer_name, note.issuer_afm).as_bytes());
    out.extend_from_slice(format!("Διεύθυνση: {}\n\n", note.issuer_address).as_bytes());

    out.extend_from_slice(b"\x1B\x45\x01");
    out.extend_from_slice(b"[ \xCE\xA0\xCE\x91\xCE\xA1\xCE\x91\xCE\x9B\xCE\x97\xCE\xA0\xCE\xA4\xCE\x97\xCE\xA3 ]\n"); // [ ΠΑΡΑΛΗΠΤΗΣ ]
    out.extend_from_slice(b"\x1B\x45\x00");
    out.extend_from_slice(format!("{} (ΑΦΜ: {})\n", note.recipient_name, note.recipient_afm).as_bytes());
    out.extend_from_slice(format!("Διεύθυνση: {}\n", note.recipient_address).as_bytes());

    out.extend_from_slice(divider.as_bytes());
    out.extend_from_slice(b"\n");

    // Items table header
    out.extend_from_slice(b"\x1B\x45\x01");
    out.extend_from_slice(b"\xCE\x95\xCE\x99\xCE\x94\xCE\x97 \xCE\x94\xCE\x99\xCE\x91\xCE\x9A\xCE\x99\xCE\x9D\xCE\x97\xCE\xA3\xCE\x97\xCE\xA3:\n"); // ΕΙΔΗ ΔΙΑΚΙΝΗΣΗΣ:
    out.extend_from_slice(b"\x1B\x45\x00");

    for (i, item) in items.iter().enumerate() {
        let line_hdr = format!("{}. {} (x{:.0} {})\n", i + 1, item.description, item.quantity, item.unit.display_name());
        out.extend_from_slice(line_hdr.as_bytes());
        if !item.sku.is_empty() {
            out.extend_from_slice(format!("   SKU: {}\n", item.sku).as_bytes());
        }
        if !item.serial_numbers.is_empty() {
            out.extend_from_slice(format!("   S/N: {}\n", item.serial_numbers.join(", ")).as_bytes());
        }
        if let Some(lot) = &item.batch_lot {
            out.extend_from_slice(format!("   LOT: {}\n", lot).as_bytes());
        }
    }

    out.extend_from_slice(divider.as_bytes());
    out.extend_from_slice(b"\n");

    // Totals
    let weight_str = note.gross_weight_kg.map(|w| format!("{:.2} kg", w)).unwrap_or_else(|| "-".to_string());
    out.extend_from_slice(format_row("Δέματα", &note.packages_count.to_string(), cols).as_bytes());
    out.extend_from_slice(format_row("Μικτό Βάρος", &weight_str, cols).as_bytes());
    if let Some(mark) = &note.mydata_mark {
        out.extend_from_slice(format_row("myDATA MARK", mark, cols).as_bytes());
    }

    out.extend_from_slice(divider.as_bytes());
    out.extend_from_slice(b"\n");

    // QR Code / Verification Seal
    out.extend_from_slice(b"\x1B\x61\x01"); // Center
    out.extend_from_slice(b"[ IAPR / myDATA & e-CMR QR PAYLOAD ]\n");
    out.extend_from_slice(format!("{}\n\n", note.qr_payload).as_bytes());

    // Receiver Signature Box
    out.extend_from_slice(b"\x1B\x61\x00"); // Left
    out.extend_from_slice(b"----------------------------------------\n");
    out.extend_from_slice(b"    \xCE\xA5\xCE\xA0\xCE\x9F\xCE\x93\xCE\xA1\xCE\x91\xCE\xA6\xCE\x97 & \xCE\xA3\xCE\xA6\xCE\xA1\xCE\x91\xCE\x93\xCE\x99\xCE\x94\xCE\x91 \xCE\xA0\xCE\x91\xCE\xA1\xCE\x91\xCE\x9B\xCE\x91\xCE\x92\xCE\x97\xCE\xA3\n\n\n"); // ΥΠΟΓΡΑΦΗ & ΣΦΡΑΓΙΔΑ ΠΑΡΑΛΑΒΗΣ
    out.extend_from_slice(b"----------------------------------------\n\n");

    // Cut Paper: GS V 66 0
    out.extend_from_slice(b"\x1D\x56\x42\x00");

    out
}
