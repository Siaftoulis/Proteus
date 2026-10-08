//! Serial Number Tracking, Component Genealogy & RMA Engine for Proteus BOS.
//! Tracks the complete physical lifecycle of high-value components and devices:
//! Supplier -> Warehouse -> Installed in Customer Ticket -> RMA Claim -> Supplier Replacement.
//! Adheres strictly to Rule 1 (100% Original Codebase) and Rule 5 (Zero Mock Data).
//! Modularized under Rule 3 (<400 lines per file).

pub mod types;
pub mod db;

pub use types::*;
pub use db::*;

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_mem_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_genealogy_schema(&conn).unwrap();
        conn
    }

    #[test]
    fn test_genealogy_intake_and_fetch() {
        let conn = setup_mem_db();
        let g = record_component_intake(
            &conn,
            "SN-BOSCH-99120",
            "BSH-ALT-01",
            "Δυναμό Αυτοκινήτου 12V 90A",
            "Bosch Hellas",
            Some("INV-2026-441"),
            24,
            "Νίκος (Αποθηκάριος)",
        )
        .unwrap();

        assert_eq!(g.serial_number, "SN-BOSCH-99120");
        assert_eq!(g.sku, "BSH-ALT-01");
        assert_eq!(g.warranty_months, 24);
        assert_eq!(g.current_stage.display_name(), "Παραλαβή Προμηθευτή");

        let fetched = get_genealogy(&conn, "SN-BOSCH-99120").unwrap().unwrap();
        assert_eq!(fetched.name, "Δυναμό Αυτοκινήτου 12V 90A");

        let timeline = get_genealogy_timeline(&conn, "SN-BOSCH-99120").unwrap();
        assert_eq!(timeline.len(), 1);
        assert!(timeline[0].description.contains("INV-2026-441"));
    }

    #[test]
    fn test_lifecycle_and_rma_workflow() {
        let conn = setup_mem_db();
        record_component_intake(
            &conn,
            "SN-SCR-4040",
            "SCR-OLED-14",
            "Οθόνη OLED 14.1\"",
            "Global Parts EU",
            None,
            12,
            "Μαρία (Παραλαβές)",
        )
        .unwrap();

        // 1. Install in customer device ticket #1042
        install_in_ticket(
            &conn,
            "SN-SCR-4040",
            "Γιώργος Αντωνίου",
            "ThinkPad T14 Gen 3",
            1042,
            "Κώστας (Τεχνικός)",
        )
        .unwrap();

        let installed = get_genealogy(&conn, "SN-SCR-4040").unwrap().unwrap();
        assert_eq!(installed.current_stage.display_name(), "Εγκατεστημένο σε Συσκευή");

        // 2. Customer returns device with fault -> initiate RMA
        initiate_rma(
            &conn,
            "SN-SCR-4040",
            "Κάθετες πράσινες γραμμές μετά από 2 εβδομάδες χρήσης",
            "Κώστας (Τεχνικός)",
        )
        .unwrap();

        let rma_active = list_active_rma_claims(&conn).unwrap();
        assert_eq!(rma_active.len(), 1);
        assert_eq!(rma_active[0].serial_number, "SN-SCR-4040");

        // 3. Supplier sends replacement part
        resolve_rma_replacement(
            &conn,
            "SN-SCR-4040",
            "SN-SCR-5099",
            Some("RMA-CR-8812"),
            "Νίκος (Αποθήκη)",
        )
        .unwrap();

        let resolved = get_genealogy(&conn, "SN-SCR-4040").unwrap().unwrap();
        assert_eq!(resolved.current_stage.display_name(), "Αντικατάσταση από Προμηθευτή");

        let active_after = list_active_rma_claims(&conn).unwrap();
        assert!(active_after.is_empty());

        let timeline = get_genealogy_timeline(&conn, "SN-SCR-4040").unwrap();
        assert_eq!(timeline.len(), 4);
    }

    #[test]
    fn test_warranty_status_calculation() {
        let conn = setup_mem_db();
        let g = record_component_intake(
            &conn,
            "SN-WARR-TEST",
            "PART-01",
            "Μίζα Αυτοκινήτου",
            "Valeo Direct",
            None,
            12,
            "Admin",
        )
        .unwrap();

        // Same time as intake: should have ~360 days remaining
        let status = g.warranty_status(g.intake_date);
        match status {
            WarrantyStatus::Valid { days_remaining } => {
                assert!(days_remaining >= 355 && days_remaining <= 365);
            }
            _ => panic!("Expected valid warranty"),
        }

        // 400 days in the future: should be expired
        let future_time = g.intake_date + 400 * 24 * 3600 * 1000;
        let expired_status = g.warranty_status(future_time);
        match expired_status {
            WarrantyStatus::Expired { days_expired } => {
                assert!(days_expired >= 35 && days_expired <= 45);
            }
            _ => panic!("Expected expired warranty"),
        }
    }

    #[test]
    fn test_search_genealogies() {
        let conn = setup_mem_db();
        record_component_intake(&conn, "SN-ALPHA-01", "SKU-A", "Alternator", "Denso", None, 24, "User").unwrap();
        record_component_intake(&conn, "SN-BETA-02", "SKU-B", "Brake Pad", "Brembo", None, 12, "User").unwrap();

        let res_a = search_genealogies(&conn, "ALPHA").unwrap();
        assert_eq!(res_a.len(), 1);
        assert_eq!(res_a[0].serial_number, "SN-ALPHA-01");

        let res_b = search_genealogies(&conn, "Brembo").unwrap();
        assert_eq!(res_b.len(), 1);
        assert_eq!(res_b[0].name, "Brake Pad");
    }
}
