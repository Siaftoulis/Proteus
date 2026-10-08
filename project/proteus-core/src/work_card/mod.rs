//! Ergani II Digital Work Card (Ψηφιακή Κάρτα Εργασίας) Engine for Proteus BOS.
//! Statutory compliance under Greek Law 4808/2021 & Circular 47319/2023.
//! 100% Original Implementation. Zero third-party boilerplate.

pub mod db;
pub mod reconcile;
pub mod types;

pub use db::{
    authenticate_employee_by_pin, calculate_shift_duration_hours, init_work_card_schema,
    list_employees, list_today_events, record_clock_event, save_employee,
};
pub use reconcile::{flush_outage_sync_queue, verify_merkle_chain};
pub use types::{
    EmployeeProfile, ErganiSyncSummary, WorkCardError, WorkCardEvent, WorkCardEventType,
};

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::{params, Connection};

    #[test]
    fn test_work_card_pin_auth_and_offline_clock_in() {
        let conn = Connection::open_in_memory().unwrap();
        init_work_card_schema(&conn).unwrap();

        let emp = EmployeeProfile {
            employee_id: "EMP-001".to_string(),
            afm: "094014201".to_string(),
            amka: "01019012345".to_string(),
            full_name: "Κωνσταντίνος Δημητρίου".to_string(),
            job_title: "Τεχνικός Υπολογιστών".to_string(),
            pin_code: "1234".to_string(),
            qr_badge_token: "QR-SEC-001".to_string(),
            is_active: true,
        };
        save_employee(&conn, &emp).unwrap();

        // 1. PIN Authentication
        let auth_res = authenticate_employee_by_pin(&conn, "094014201", "1234");
        assert!(auth_res.is_ok());
        assert_eq!(auth_res.unwrap().full_name, "Κωνσταντίνος Δημητρίου");

        let bad_auth = authenticate_employee_by_pin(&conn, "094014201", "9999");
        assert!(matches!(bad_auth, Err(WorkCardError::InvalidPin(_))));

        // 2. Offline Clock-In with statutory telecom outage marker
        let event = record_clock_event(
            &conn,
            "EMP-001",
            WorkCardEventType::ClockIn,
            true, // Offline fallback active
            Some("TELECOM_PROVIDER_OUTAGE"),
        )
        .unwrap();

        assert!(event.is_offline_fallback);
        assert_eq!(event.outage_reason.as_deref(), Some("TELECOM_PROVIDER_OUTAGE"));
        assert_eq!(event.sync_status, "PENDING_OUTAGE_SYNC");
        assert!(!event.merkle_hash.is_empty());
    }

    #[test]
    fn test_merkle_chain_verification_and_outage_flush() {
        let conn = Connection::open_in_memory().unwrap();
        init_work_card_schema(&conn).unwrap();

        let emp = EmployeeProfile {
            employee_id: "EMP-010".to_string(),
            afm: "099887766".to_string(),
            amka: "11223344556".to_string(),
            full_name: "Μαρία Γεωργίου".to_string(),
            job_title: "Ταμίας".to_string(),
            pin_code: "4321".to_string(),
            qr_badge_token: "QR-SEC-010".to_string(),
            is_active: true,
        };
        save_employee(&conn, &emp).unwrap();

        // Record two offline events
        record_clock_event(&conn, "EMP-010", WorkCardEventType::ClockIn, true, Some("OUTAGE")).unwrap();
        record_clock_event(&conn, "EMP-010", WorkCardEventType::ClockOut, true, Some("OUTAGE")).unwrap();

        // Verify Merkle integrity passes
        let root = verify_merkle_chain(&conn).unwrap();
        assert!(!root.is_empty());

        // Flush outage queue
        let summary = flush_outage_sync_queue(&conn).unwrap();
        assert_eq!(summary.total_queued, 2);
        assert_eq!(summary.synced_count, 2);
        assert_eq!(summary.failed_count, 0);

        // Verify all events are now SYNCED with submission IDs
        let events = list_today_events(&conn, "20").unwrap();
        assert_eq!(events.len(), 2);
        for (ev, _) in events {
            assert_eq!(ev.sync_status, "SYNCED");
            assert!(ev.ergani_submission_id.is_some());
            assert!(ev.ergani_submission_id.unwrap().starts_with("ERG-"));
        }
    }

    #[test]
    fn test_shift_duration_and_statutory_overtime() {
        let conn = Connection::open_in_memory().unwrap();
        init_work_card_schema(&conn).unwrap();

        let emp = EmployeeProfile {
            employee_id: "EMP-002".to_string(),
            afm: "090000045".to_string(),
            amka: "15038598765".to_string(),
            full_name: "Ελένη Βασιλείου".to_string(),
            job_title: "Υπεύθυνη Καταστήματος".to_string(),
            pin_code: "5678".to_string(),
            qr_badge_token: "QR-SEC-002".to_string(),
            is_active: true,
        };
        save_employee(&conn, &emp).unwrap();

        let base_ts = 1727942400000i64;
        let break_start_ts = base_ts + (4 * 3600 * 1000);
        let break_end_ts = break_start_ts + (1800 * 1000);
        let clock_out_ts = base_ts + (34200 * 1000);

        conn.execute(
            "INSERT INTO work_card_events (event_id, employee_id, event_type, timestamp_utc, local_time_str, merkle_hash, sync_status) VALUES ('E1', 'EMP-002', 'CLOCK_IN', ?1, '2026-10-03 08:00:00', 'h1', 'PENDING')",
            params![base_ts],
        ).unwrap();

        conn.execute(
            "INSERT INTO work_card_events (event_id, employee_id, event_type, timestamp_utc, local_time_str, merkle_hash, sync_status) VALUES ('E2', 'EMP-002', 'BREAK_START', ?1, '2026-10-03 12:00:00', 'h2', 'PENDING')",
            params![break_start_ts],
        ).unwrap();

        conn.execute(
            "INSERT INTO work_card_events (event_id, employee_id, event_type, timestamp_utc, local_time_str, merkle_hash, sync_status) VALUES ('E3', 'EMP-002', 'BREAK_END', ?1, '2026-10-03 12:30:00', 'h3', 'PENDING')",
            params![break_end_ts],
        ).unwrap();

        conn.execute(
            "INSERT INTO work_card_events (event_id, employee_id, event_type, timestamp_utc, local_time_str, merkle_hash, sync_status) VALUES ('E4', 'EMP-002', 'CLOCK_OUT', ?1, '2026-10-03 17:30:00', 'h4', 'PENDING')",
            params![clock_out_ts],
        ).unwrap();

        let (regular, overtime) = calculate_shift_duration_hours(&conn, "EMP-002", "2026-10-03").unwrap();
        assert_eq!(regular, 8.0);
        assert_eq!(overtime, 1.0);
    }
}
