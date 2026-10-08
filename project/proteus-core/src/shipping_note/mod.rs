//! Digital Shipping Note, Dispatch Companion & myDATA / e-CMR Transport Subsystem.
//! Manages verifiable electronic waybills (Δελτία Αποστολής / Διακίνησης) with:
//! - Greek Tax ID (ΑΦΜ) validation
//! - Standardized IAPR / myDATA & e-CMR QR payload generation
//! - SHA-256 tamper-proof transport seals
//! - Component serial number linkage (Genealogy)
//! - Van sales offline store-and-forward outbox
//! - Direct ESC/POS thermal delivery voucher generation.
//! Adheres strictly to Rule 1 (100% Original Codebase) and Rule 5 (Zero Mock Data).
//! Modularized under Rule 3 (<400 lines per file).

pub mod db;
pub mod escpos;
pub mod types;

pub use db::*;
pub use escpos::*;
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::printer::{PaperWidth, ShopReceiptConfig};
    use rusqlite::Connection;
    use uuid::Uuid;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_shipping_schema(&conn).unwrap();
        conn
    }

    #[test]
    fn test_shipping_schema_init() {
        let conn = setup_test_db();
        // Check that tables exist
        let count: i64 = conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name IN ('shipping_notes', 'shipping_note_items', 'shipping_outbox')",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_shipping_note_creation_and_retrieval() {
        let conn = setup_test_db();
        let note = ShippingNote::new(
            "ΔΑ-2026-0001".to_string(),
            "094014201".to_string(),
            "PROTEUS INDUSTRIAL SUPPLIES".to_string(),
            "Πειραιώς 100, Αθήνα".to_string(),
            "090000045".to_string(),
            "ERGODOMIKI ATE".to_string(),
            "Λεωφ. Κηφισίας 20, Μαρούσι".to_string(),
            "IEZ-1234".to_string(),
            "Γιώργος Παπαδόπουλος".to_string(),
            TransportPurpose::Sale,
            4,
            Some(125.50),
        );

        create_shipping_note(&conn, &note).unwrap();

        let loaded = get_shipping_note(&conn, &note.id).unwrap().expect("Note should exist");
        assert_eq!(loaded.note_number, "ΔΑ-2026-0001");
        assert_eq!(loaded.issuer_afm, "094014201");
        assert_eq!(loaded.recipient_afm, "090000045");
        assert_eq!(loaded.vehicle_plate, "IEZ-1234");
        assert_eq!(loaded.status, DispatchStatus::Draft);
        assert_eq!(loaded.packages_count, 4);
        assert_eq!(loaded.gross_weight_kg, Some(125.50));
    }

    #[test]
    fn test_shipping_note_items_and_genealogy_serials() {
        let conn = setup_test_db();
        let note = ShippingNote::new(
            "ΔΑ-2026-0002".to_string(),
            "094014201".to_string(),
            "PROTEUS HARDWARE".to_string(),
            "Αθήνα".to_string(),
            "090000045".to_string(),
            "TECH CORP".to_string(),
            "Θεσσαλονίκη".to_string(),
            "NHI-8899".to_string(),
            "Νίκος Οδηγός".to_string(),
            TransportPurpose::Repair,
            2,
            Some(15.0),
        );
        create_shipping_note(&conn, &note).unwrap();

        let item1 = ShippingNoteItem {
            id: Uuid::new_v4().to_string(),
            note_id: note.id.clone(),
            line_number: 1,
            sku: "SVR-DELL-R750".to_string(),
            description: "Dell PowerEdge Server R750".to_string(),
            unit: ShippingUnit::Piece,
            quantity: 1.0,
            serial_numbers: vec!["SN-DELL-998822".to_string()],
            batch_lot: Some("LOT-2026-Q3".to_string()),
            notes: Some("Επισκευασμένο τροφοδοτικό".to_string()),
        };

        let item2 = ShippingNoteItem {
            id: Uuid::new_v4().to_string(),
            note_id: note.id.clone(),
            line_number: 2,
            sku: "RAM-ECC-32GB".to_string(),
            description: "DDR4 32GB ECC Reg Module".to_string(),
            unit: ShippingUnit::Piece,
            quantity: 4.0,
            serial_numbers: vec![
                "ECC-RAM-001".to_string(),
                "ECC-RAM-002".to_string(),
                "ECC-RAM-003".to_string(),
                "ECC-RAM-004".to_string(),
            ],
            batch_lot: None,
            notes: None,
        };

        add_shipping_note_item(&conn, &item1).unwrap();
        add_shipping_note_item(&conn, &item2).unwrap();

        let items = list_shipping_note_items(&conn, &note.id).unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].sku, "SVR-DELL-R750");
        assert_eq!(items[0].serial_numbers, vec!["SN-DELL-998822"]);
        assert_eq!(items[1].quantity, 4.0);
        assert_eq!(items[1].serial_numbers.len(), 4);
    }

    #[test]
    fn test_signature_and_qr_verification() {
        let sig = compute_transport_signature(
            "094014201",
            "090000045",
            "IEZ-1234",
            "2026-09-30T10:00:00Z",
            "SALE",
            5,
        );
        assert!(!sig.is_empty());
        assert_eq!(sig.len(), 64); // SHA-256 hex string

        let qr = generate_iapr_qr_payload(
            Some("MARK-2026-7889901"),
            "094014201",
            "090000045",
            "2026-09-30T10:00:00Z",
            "IEZ-1234",
            5,
            &sig,
        );
        assert!(qr.contains("AADE-CMR"));
        assert!(qr.contains("MARK-2026-7889901"));
        assert!(qr.contains("ISSUER:094014201"));
        assert!(qr.contains("RECV:090000045"));
        assert!(qr.contains("VEH:IEZ-1234"));
    }

    #[test]
    fn test_status_transitions_and_delivery_completion() {
        let conn = setup_test_db();
        let note = ShippingNote::new(
            "ΔΑ-2026-0003".to_string(),
            "094014201".to_string(),
            "SENDER".to_string(),
            "ATHENS".to_string(),
            "090000045".to_string(),
            "RECEIVER".to_string(),
            "PATRA".to_string(),
            "AX-5544".to_string(),
            "Κώστας".to_string(),
            TransportPurpose::TransferBetweenBranches,
            1,
            Some(50.0),
        );
        create_shipping_note(&conn, &note).unwrap();

        // Dispatch
        update_shipping_note_status(&conn, &note.id, DispatchStatus::Dispatched).unwrap();
        let loaded = get_shipping_note(&conn, &note.id).unwrap().unwrap();
        assert_eq!(loaded.status, DispatchStatus::Dispatched);

        // In Transit
        update_shipping_note_status(&conn, &note.id, DispatchStatus::InTransit).unwrap();
        let loaded = get_shipping_note(&conn, &note.id).unwrap().unwrap();
        assert_eq!(loaded.status, DispatchStatus::InTransit);

        // Delivery Completion
        record_delivery_completion(
            &conn,
            &note.id,
            "Παρελήφθη ανεπιφύλακτα από Μ. Αντωνίου",
            "2026-09-30T14:30:00Z",
        )
        .unwrap();

        let final_note = get_shipping_note(&conn, &note.id).unwrap().unwrap();
        assert_eq!(final_note.status, DispatchStatus::Delivered);
        assert_eq!(final_note.actual_arrival.as_deref(), Some("2026-09-30T14:30:00Z"));
        assert_eq!(
            final_note.recipient_signature_note.as_deref(),
            Some("Παρελήφθη ανεπιφύλακτα από Μ. Αντωνίου")
        );
    }

    #[test]
    fn test_offline_outbox_queue_and_sync() {
        let conn = setup_test_db();
        let outbox_id = queue_offline_dispatch(
            &conn,
            "note-uuid-1234",
            "MARK_DELIVERED",
            r#"{"signed_by":"Customer","time":"2026-09-30T12:00:00Z"}"#,
        )
        .unwrap();

        let pending = list_pending_offline_dispatches(&conn).unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].0, outbox_id);
        assert_eq!(pending[0].1, "note-uuid-1234");
        assert_eq!(pending[0].2, "MARK_DELIVERED");

        // Sync
        mark_offline_dispatch_synced(&conn, outbox_id).unwrap();
        let pending_after = list_pending_offline_dispatches(&conn).unwrap();
        assert_eq!(pending_after.len(), 0);
    }

    #[test]
    fn test_escpos_shipping_voucher_generation() {
        let note = ShippingNote::new(
            "ΔΑ-2026-0004".to_string(),
            "094014201".to_string(),
            "PROTEUS WAREHOUSE".to_string(),
            "Πειραιάς".to_string(),
            "090000045".to_string(),
            "CLIENT INDUSTRIAL".to_string(),
            "Λάρισα".to_string(),
            "KHA-7788".to_string(),
            "Δημήτρης".to_string(),
            TransportPurpose::Sale,
            3,
            Some(45.0),
        );

        let items = vec![ShippingNoteItem {
            id: "it-1".to_string(),
            note_id: note.id.clone(),
            line_number: 1,
            sku: "MOT-01".to_string(),
            description: "Ηλεκτροκινητήρας 3-Φασικός 5kW".to_string(),
            unit: ShippingUnit::Piece,
            quantity: 2.0,
            serial_numbers: vec!["MOT-SN-100".to_string(), "MOT-SN-101".to_string()],
            batch_lot: None,
            notes: None,
        }];

        let config_80 = ShopReceiptConfig {
            paper_width: PaperWidth::Width80mm,
            ..Default::default()
        };
        let bytes_80 = generate_escpos_shipping_voucher(&note, &items, &config_80);
        assert!(!bytes_80.is_empty());
        assert!(bytes_80.starts_with(b"\x1B\x40")); // ESC @
        assert!(bytes_80.ends_with(b"\x1D\x56\x42\x00")); // Cut

        let config_58 = ShopReceiptConfig {
            paper_width: PaperWidth::Width58mm,
            ..Default::default()
        };
        let bytes_58 = generate_escpos_shipping_voucher(&note, &items, &config_58);
        assert!(!bytes_58.is_empty());
    }
}
