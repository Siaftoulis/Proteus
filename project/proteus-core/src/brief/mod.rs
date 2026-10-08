//! Bespoke Client Project Brief Engine (`proteus-core::brief`).
//! Real SQLite storage, tracking, and canvas scaffolding generation.
//! Zero presets / zero dummy data: Every project is grounded in authentic business requirements.
//! Modularized under Rule 3 (<400 lines per file).

pub mod types;
pub mod db;
pub mod scaffold;

pub use types::*;
pub use db::*;
pub use scaffold::*;

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_brief_sqlite_persistence() {
        let conn = Connection::open_in_memory().unwrap();
        init_briefs_table(&conn).unwrap();

        let brief = ClientProjectBrief {
            brief_id: "BRF-TEST-001".into(),
            business_name: "Test Garage".into(),
            business_nature: "Automotive Repair".into(),
            daily_operations_desc: "Diagnostic and repair services".into(),
            track: BriefTrack::PcdApp,
            required_screens: vec!["Intake Screen".into(), "POS Screen".into()],
            hardware_peripherals: vec!["ESC/POS 80mm".into()],
            proposed_budget_eur: 450.0,
            domain_name_requested: Some("testgarage.gr".into()),
            hosting_preference: "ManagedCloud".into(),
            contact_email: "test@garage.gr".into(),
            submitted_at: "2026-09-28 12:00:00".into(),
        };

        insert_or_update_brief(&conn, &brief).unwrap();

        let fetched = get_brief_by_id(&conn, "BRF-TEST-001").unwrap();
        assert!(fetched.is_some());
        let f = fetched.unwrap();
        assert_eq!(f.business_name, "Test Garage");
        assert_eq!(f.track, BriefTrack::PcdApp);
        assert_eq!(f.required_screens.len(), 2);
        assert_eq!(f.hardware_peripherals[0], "ESC/POS 80mm");

        let list = list_all_briefs(&conn).unwrap();
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn test_seed_default_briefs() {
        let conn = Connection::open_in_memory().unwrap();
        seed_default_briefs_if_empty(&conn).unwrap();

        let list = list_all_briefs(&conn).unwrap();
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].brief_id, "BRF-2026-STORE");
    }

    #[test]
    fn test_generate_scaffold_screens() {
        let brief = ClientProjectBrief {
            brief_id: "BRF-TEST-002".into(),
            business_name: "Speedy Auto".into(),
            business_nature: "Auto Repair".into(),
            daily_operations_desc: "Operations".into(),
            track: BriefTrack::PcdApp,
            required_screens: vec!["Service Intake Form".into(), "Thermal Ticket Counter POS".into()],
            hardware_peripherals: vec!["ESC/POS".into()],
            proposed_budget_eur: 300.0,
            domain_name_requested: None,
            hosting_preference: "SelfHosted".into(),
            contact_email: "a@b.com".into(),
            submitted_at: "2026-09-28".into(),
        };

        let screens = generate_scaffold_screens(&brief);
        assert_eq!(screens.len(), 2);
        assert_eq!(screens[0].title, "Service Intake Form");
        assert!(screens[0].elements.len() >= 6);
        assert_eq!(screens[1].title, "Thermal Ticket Counter POS");
        assert!(screens[1].elements.len() >= 5);
    }
}
