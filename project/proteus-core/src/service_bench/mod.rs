//! Technical Service & Diagnostic Bench Engine for Proteus BOS.
//! Core Operational Wedge: Parts Consumption, Labor Logging & 1-Click myDATA Settlement.
//! Strict Rule 1 (100% Original Codebase), Rule 3 (<400 lines), and Rule 5 (Zero Mock Data).

pub mod db;
pub mod settlement;
pub mod types;

pub use db::{
    add_consumed_part, compute_bench_summary, get_diagnostic_checklist, init_service_bench_schema,
    list_consumed_parts, list_labor_logs, log_technician_labor, remove_consumed_part,
    save_diagnostic_checklist,
};
pub use settlement::{settle_ticket_to_invoice, BenchSettlementError, SettlementResult};
pub use types::{BenchSummary, ConsumedSparePart, DeviceDiagnosticChecklist, LaborLog};

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use crate::cardex::init_cardex_schema;
    use crate::invoicing::init_invoicing_schema;
    use crate::notifications::init_notifications_schema;
    use crate::tickets::{create_ticket, init_tickets_schema, ServiceTicket, TicketStatus};

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_tickets_schema(&conn).unwrap();
        init_service_bench_schema(&conn).unwrap();
        init_invoicing_schema(&conn).unwrap();
        init_cardex_schema(&conn).unwrap();
        init_notifications_schema(&conn).unwrap();
        conn
    }

    #[test]
    fn test_diagnostic_checklist_lifecycle() {
        let conn = setup_test_db();
        let ticket_id = "test-ticket-diag-01";

        let checklist = DeviceDiagnosticChecklist {
            ticket_id: ticket_id.to_string(),
            powers_on: true,
            display_functional: true,
            touch_responsive: false,
            liquid_ingress_detected: true,
            audio_functional: false,
            camera_functional: true,
            battery_health_pct: Some(78),
            cosmetic_condition: "Ραγισμένη οθόνη".to_string(),
            customer_accessories: "Φορτιστής Type-C".to_string(),
            inspected_by: "Nikos (Technician)".to_string(),
            inspected_at: chrono::Utc::now().timestamp_millis(),
        };

        save_diagnostic_checklist(&conn, &checklist).unwrap();

        let loaded = get_diagnostic_checklist(&conn, ticket_id).unwrap().unwrap();
        assert_eq!(loaded.ticket_id, ticket_id);
        assert!(loaded.powers_on);
        assert!(!loaded.touch_responsive);
        assert!(loaded.liquid_ingress_detected);
        assert_eq!(loaded.battery_health_pct, Some(78));
        assert_eq!(loaded.cosmetic_condition, "Ραγισμένη οθόνη");
    }

    #[test]
    fn test_bench_parts_and_labor_aggregation() {
        let conn = setup_test_db();
        let ticket_id = "test-ticket-bench-02";

        // Add 2 spare parts
        let part1 = ConsumedSparePart {
            id: "part-01".to_string(),
            ticket_id: ticket_id.to_string(),
            part_sku: "SCR-OLED-IP15".to_string(),
            description: "OLED Display Assembly".to_string(),
            quantity: 1.0,
            unit_cost_eur: 65.0,
            unit_retail_eur: 120.0,
            technician_name: "Kostas".to_string(),
            consumed_at: chrono::Utc::now().timestamp_millis(),
        };
        let part2 = ConsumedSparePart {
            id: "part-02".to_string(),
            ticket_id: ticket_id.to_string(),
            part_sku: "ADH-TAPE-01".to_string(),
            description: "Waterproof Seal Adhesive".to_string(),
            quantity: 2.0,
            unit_cost_eur: 2.5,
            unit_retail_eur: 8.0,
            technician_name: "Kostas".to_string(),
            consumed_at: chrono::Utc::now().timestamp_millis(),
        };
        add_consumed_part(&conn, &part1).unwrap();
        add_consumed_part(&conn, &part2).unwrap();

        // Add 2 labor logs: 45 min + 30 min = 75 min @ 40€/hr
        let labor1 = LaborLog {
            id: "lab-01".to_string(),
            ticket_id: ticket_id.to_string(),
            technician_name: "Kostas".to_string(),
            duration_minutes: 45,
            hourly_rate_eur: 40.0,
            work_performed: "Disassembly & screen replacement".to_string(),
            logged_at: chrono::Utc::now().timestamp_millis(),
        };
        let labor2 = LaborLog {
            id: "lab-02".to_string(),
            ticket_id: ticket_id.to_string(),
            technician_name: "Kostas".to_string(),
            duration_minutes: 30,
            hourly_rate_eur: 40.0,
            work_performed: "Adhesive cure & QA testing".to_string(),
            logged_at: chrono::Utc::now().timestamp_millis(),
        };
        log_technician_labor(&conn, &labor1).unwrap();
        log_technician_labor(&conn, &labor2).unwrap();

        let summary = compute_bench_summary(&conn, ticket_id).unwrap();
        assert_eq!(summary.parts_count, 2);
        assert_eq!(summary.parts_total_cost_eur, 70.0); // 65 + 5
        assert_eq!(summary.parts_total_retail_eur, 136.0); // 120 + 16
        assert_eq!(summary.labor_total_minutes, 75);
        assert_eq!(summary.labor_total_eur, 50.0); // 1.25 hr * 40
        assert_eq!(summary.gross_total_eur, 186.0); // 136 + 50
    }

    #[test]
    fn test_1click_settlement_to_invoice_cardex_and_notification() {
        let conn = setup_test_db();

        // Create base service ticket
        let mut ticket = ServiceTicket::new(
            "Acme Logistics",
            "+306981234567",
            "Zebra TC52 Barcode Scanner",
            "Broken scan engine window",
        );
        create_ticket(&conn, &mut ticket).unwrap();

        // Add part + labor
        let part = ConsumedSparePart {
            id: "part-zb-01".to_string(),
            ticket_id: ticket.ticket_id.clone(),
            part_sku: "ZB-SCN-WIN".to_string(),
            description: "Corning Glass Scan Window".to_string(),
            quantity: 1.0,
            unit_cost_eur: 15.0,
            unit_retail_eur: 35.0,
            technician_name: "Eleni".to_string(),
            consumed_at: chrono::Utc::now().timestamp_millis(),
        };
        add_consumed_part(&conn, &part).unwrap();

        let labor = LaborLog {
            id: "lab-zb-01".to_string(),
            ticket_id: ticket.ticket_id.clone(),
            technician_name: "Eleni".to_string(),
            duration_minutes: 60,
            hourly_rate_eur: 45.0,
            work_performed: "Optical realignment and lens calibration".to_string(),
            logged_at: chrono::Utc::now().timestamp_millis(),
        };
        log_technician_labor(&conn, &labor).unwrap();

        // Execute 1-Click Settlement
        let res = settle_ticket_to_invoice(
            &conn,
            &ticket.ticket_id,
            "TΠΥ",
            301,
            "094014201",
            "802194512",
            "Acme Logistics S.A.",
            Some("+306981234567"),
            true, // B2B
            "Επισκευή Zebra Scanner",
        ).unwrap();

        assert_eq!(res.invoice_number, 301);
        // Net: 35 + 45 = 80€. Gross @ 24% VAT: 80 * 1.24 = 99.20€
        assert_eq!(res.total_gross_eur, 99.20);
        assert!(res.cardex_movement_id.is_some());
        assert!(res.notification_id.is_some());

        // Verify ticket status transitioned to Delivered
        let updated_t = crate::tickets::get_ticket(&conn, &ticket.ticket_id).unwrap().unwrap();
        assert_eq!(updated_t.current_status, TicketStatus::Delivered);
        assert!(updated_t.delivered_at.is_some());

        // Verify myDATA outbox queue entry exists
        let outbox_count: i64 = conn
            .query_row("SELECT count(*) FROM mydata_outbox WHERE invoice_id = ?1", [&res.invoice_id], |r| r.get(0))
            .unwrap();
        assert_eq!(outbox_count, 1, "Settlement must enqueue invoice to myDATA outbox");

        // Verify Cardex debit movement exists for customer
        let cardex_count: i64 = conn
            .query_row("SELECT count(*) FROM cardex_entries WHERE entity_id = '802194512'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(cardex_count, 1, "Settlement must post debit movement to customer Cardex");
    }
}
