//! Cold Chain & HACCP Telemetry Subsystem.
//! Strict Rule 1 (100% Original Codebase) and Rule 5 (Zero Mock Data).
//! Modularized under Rule 3 (<400 lines per file).

pub mod cert;
pub mod db;
pub mod types;

pub use cert::*;
pub use db::*;
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use rusqlite::Connection;

    fn setup_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_cold_chain_schema(&conn).unwrap();
        conn
    }

    #[test]
    fn test_cold_sensor_registration_and_listing() {
        let conn = setup_db();
        let sensor = ColdChainSensor::new("SEN-FREEZER-01", "VAN-FREEZE-10", ColdStorageType::DeepFreeze);
        register_cold_sensor(&conn, &sensor).unwrap();

        let sensors = list_cold_sensors(&conn).unwrap();
        assert_eq!(sensors.len(), 1);
        assert_eq!(sensors[0].sensor_id, "SEN-FREEZER-01");
        assert_eq!(sensors[0].storage_type, ColdStorageType::DeepFreeze);
        assert_eq!(sensors[0].min_temp_celsius, -25.0);
        assert_eq!(sensors[0].max_temp_celsius, -18.0);
    }

    #[test]
    fn test_in_spec_telemetry_reading_no_breach() {
        let conn = setup_db();
        let sensor = ColdChainSensor::new("SEN-CHILL-01", "VAN-4433", ColdStorageType::Chilled);
        register_cold_sensor(&conn, &sensor).unwrap();

        // 2.5°C is well within [0.0°C, 4.0°C]
        let (reading, breach) = ingest_telemetry_reading(&conn, &sensor.sensor_id, 2.5, Some(75.0), false, Some(98)).unwrap();
        assert!(breach.is_none());
        assert_eq!(reading.temperature_celsius, 2.5);
        assert!(!reading.merkle_hash.is_empty());

        let logs = list_recent_readings(&conn, Some("VAN-4433"), 10).unwrap();
        assert_eq!(logs.len(), 1);
    }

    #[test]
    fn test_temperature_excursion_generates_haccp_breach() {
        let conn = setup_db();
        let sensor = ColdChainSensor::new("SEN-CHILL-02", "COLD-ROOM-B", ColdStorageType::Chilled);
        register_cold_sensor(&conn, &sensor).unwrap();

        // 10.5°C is 6.5°C above max 4.0°C => CriticalSpoilage
        let (_reading, breach_opt) = ingest_telemetry_reading(&conn, &sensor.sensor_id, 10.5, Some(80.0), true, Some(85)).unwrap();
        assert!(breach_opt.is_some());
        let breach = breach_opt.unwrap();
        assert_eq!(breach.severity, BreachSeverity::CriticalSpoilage);
        assert_eq!(breach.status, BreachStatus::Active);
        assert_eq!(breach.threshold_limit, 4.0);

        let active = list_active_breaches(&conn).unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].breach_id, breach.breach_id);

        // Resolve breach with operator note
        resolve_breach_event(&conn, &breach.breach_id, "Κώστας Ψυκτικός", "Επανεκκίνηση συμπιεστή & έλεγχος φρέον").unwrap();
        let active_after = list_active_breaches(&conn).unwrap();
        assert!(active_after.is_empty());
    }

    #[test]
    fn test_haccp_compliance_certificate_generation() {
        let conn = setup_db();
        let sensor = ColdChainSensor::new("SEN-PHARMA-01", "BOX-VACCINE-09", ColdStorageType::PharmaCold);
        register_cold_sensor(&conn, &sensor).unwrap();

        let start = Utc::now().timestamp() - 10;
        // Ingest 5 good readings within 2.0°C - 8.0°C
        for i in 0..5 {
            ingest_telemetry_reading(&conn, &sensor.sensor_id, 4.0 + (i as f64 * 0.5), Some(50.0), false, Some(100)).unwrap();
        }
        let end = Utc::now().timestamp() + 10;

        let cert = generate_haccp_certificate(&conn, "BOX-VACCINE-09", start, end).unwrap();
        assert_eq!(cert.total_readings, 5);
        assert_eq!(cert.in_spec_count, 5);
        assert_eq!(cert.breach_count, 0);
        assert_eq!(cert.in_spec_percentage, 100.0);
        assert!(cert.is_compliant);
        assert!(!cert.merkle_root_hash.is_empty());
    }
}
