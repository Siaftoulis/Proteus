//! Multi-Protocol Peripheral Bus Engine (Micro-task 25.1.1).
//! Orchestrates physical and virtual hardware channels across Serial, USB-HID, BLE, and MQTT.
//! Persists device registration, health states, and audit frames directly to SQLite (Rule 5).

use chrono::Utc;
use rusqlite::{params, Connection};

use super::protocol::HardwareProtocol;
use super::types::{
    HardwareBusError, InboundPeripheralPayload, PeripheralCommand, PeripheralDeviceRecord,
    PeripheralDeviceRole, PeripheralStatus,
};

pub struct PeripheralBus;

impl PeripheralBus {
    /// Initializes SQLite tables for hardware peripherals and telemetry event logs.
    pub fn init_schema(conn: &Connection) -> Result<(), HardwareBusError> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS system_peripheral_devices (
                device_id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                role TEXT NOT NULL,
                protocol_json TEXT NOT NULL,
                status TEXT NOT NULL,
                is_enabled INTEGER NOT NULL DEFAULT 1,
                last_seen_at TEXT,
                created_at TEXT NOT NULL
            );",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS system_peripheral_events (
                event_id TEXT PRIMARY KEY,
                device_id TEXT NOT NULL,
                event_kind TEXT NOT NULL,
                payload_hex TEXT NOT NULL,
                parsed_summary TEXT NOT NULL,
                created_at TEXT NOT NULL
            );",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_peripheral_events_device 
             ON system_peripheral_events(device_id, created_at);",
            [],
        )?;

        Ok(())
    }

    /// Registers or updates a peripheral device in SQLite.
    pub fn register_device(conn: &Connection, dev: &PeripheralDeviceRecord) -> Result<(), HardwareBusError> {
        Self::init_schema(conn)?;
        let role_str = serde_json::to_string(&dev.role)?;
        let protocol_str = serde_json::to_string(&dev.protocol)?;
        let status_str = serde_json::to_string(&dev.status)?;

        conn.execute(
            "INSERT INTO system_peripheral_devices (
                device_id, name, role, protocol_json, status, is_enabled, last_seen_at, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(device_id) DO UPDATE SET
                name = excluded.name,
                role = excluded.role,
                protocol_json = excluded.protocol_json,
                status = excluded.status,
                is_enabled = excluded.is_enabled,
                last_seen_at = excluded.last_seen_at",
            params![
                dev.device_id,
                dev.name,
                role_str,
                protocol_str,
                status_str,
                if dev.is_enabled { 1 } else { 0 },
                dev.last_seen_at,
                dev.created_at
            ],
        )?;

        Ok(())
    }

    /// Updates status and heartbeat timestamp of a registered device.
    pub fn update_device_status(
        conn: &Connection,
        device_id: &str,
        status: PeripheralStatus,
    ) -> Result<(), HardwareBusError> {
        Self::init_schema(conn)?;
        let status_str = serde_json::to_string(&status)?;
        let now = Utc::now().to_rfc3339();

        let updated = conn.execute(
            "UPDATE system_peripheral_devices 
             SET status = ?1, last_seen_at = ?2 
             WHERE device_id = ?3",
            params![status_str, now, device_id],
        )?;

        if updated == 0 {
            return Err(HardwareBusError::DeviceNotFound(device_id.to_string()));
        }

        Ok(())
    }

    /// Retrieves all registered devices on the peripheral bus.
    pub fn list_devices(conn: &Connection) -> Result<Vec<PeripheralDeviceRecord>, HardwareBusError> {
        Self::init_schema(conn)?;
        let mut stmt = conn.prepare(
            "SELECT device_id, name, role, protocol_json, status, is_enabled, last_seen_at, created_at 
             FROM system_peripheral_devices 
             ORDER BY created_at ASC",
        )?;

        let rows = stmt.query_map([], |r| {
            let role_raw: String = r.get(2)?;
            let proto_raw: String = r.get(3)?;
            let stat_raw: String = r.get(4)?;
            let is_enabled: i32 = r.get(5)?;

            let role = serde_json::from_str(&role_raw)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, Box::new(e)))?;
            let protocol = serde_json::from_str(&proto_raw)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(3, rusqlite::types::Type::Text, Box::new(e)))?;
            let status = serde_json::from_str(&stat_raw)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, Box::new(e)))?;

            Ok(PeripheralDeviceRecord {
                device_id: r.get(0)?,
                name: r.get(1)?,
                role,
                protocol,
                status,
                is_enabled: is_enabled == 1,
                last_seen_at: r.get(6)?,
                created_at: r.get(7)?,
            })
        })?;

        let mut list = Vec::new();
        for row in rows {
            list.push(row?);
        }
        Ok(list)
    }

    /// Records an audit telemetry frame in SQLite.
    pub fn record_event(
        conn: &Connection,
        device_id: &str,
        event_kind: &str,
        raw_bytes: &[u8],
        summary: &str,
    ) -> Result<(), HardwareBusError> {
        Self::init_schema(conn)?;
        let event_id = format!("evt_hw_{}", uuid::Uuid::new_v4());
        let hex_payload: String = raw_bytes.iter().map(|b| format!("{:02x}", b)).collect();
        let now = Utc::now().to_rfc3339();

        conn.execute(
            "INSERT INTO system_peripheral_events (
                event_id, device_id, event_kind, payload_hex, parsed_summary, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![event_id, device_id, event_kind, hex_payload, summary, now],
        )?;

        Ok(())
    }

    /// Parses inbound raw bytes based on device role.
    pub fn parse_inbound_frame(
        role: PeripheralDeviceRole,
        raw_bytes: &[u8],
    ) -> Result<InboundPeripheralPayload, HardwareBusError> {
        match role {
            PeripheralDeviceRole::WeightScale => {
                let reading = HardwareProtocol::parse_scale_frame(raw_bytes)?;
                Ok(InboundPeripheralPayload::Weight(reading))
            }
            PeripheralDeviceRole::BarcodeScanner | PeripheralDeviceRole::CustomMacroPad => {
                let barcode = HardwareProtocol::parse_scanner_frame(raw_bytes)?;
                Ok(InboundPeripheralPayload::Barcode(barcode))
            }
            PeripheralDeviceRole::EnvironmentalSensor => {
                HardwareProtocol::parse_sensor_frame(raw_bytes)
            }
            PeripheralDeviceRole::IndustrialRelayActuator => {
                if raw_bytes.len() >= 2 {
                    Ok(InboundPeripheralPayload::RelayAck {
                        channel: raw_bytes[0],
                        state: raw_bytes[1] != 0,
                    })
                } else {
                    Ok(InboundPeripheralPayload::Raw(raw_bytes.to_vec()))
                }
            }
            PeripheralDeviceRole::CustomerPoleDisplay | PeripheralDeviceRole::FiscalSignatureBox => {
                Ok(InboundPeripheralPayload::Raw(raw_bytes.to_vec()))
            }
        }
    }

    /// Formats an outbound hardware command into raw protocol bytes.
    pub fn format_command_payload(
        role: PeripheralDeviceRole,
        cmd: &PeripheralCommand,
    ) -> Result<Vec<u8>, HardwareBusError> {
        HardwareProtocol::format_command_payload(role, cmd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hardware::types::{PeripheralProtocol, SerialParity};

    #[test]
    fn test_init_schema_and_crud_devices() {
        let conn = Connection::open_in_memory().unwrap();
        PeripheralBus::init_schema(&conn).unwrap();

        let dev = PeripheralDeviceRecord {
            device_id: "SCALE-01".into(),
            name: "Mettler Toledo Deli Scale".into(),
            role: PeripheralDeviceRole::WeightScale,
            protocol: PeripheralProtocol::SerialUart {
                port_name: "COM3".into(),
                baud_rate: 9600,
                data_bits: 8,
                stop_bits: 1,
                parity: SerialParity::None,
            },
            status: PeripheralStatus::OnlineReady,
            is_enabled: true,
            last_seen_at: Some(Utc::now().to_rfc3339()),
            created_at: Utc::now().to_rfc3339(),
        };

        PeripheralBus::register_device(&conn, &dev).unwrap();
        let list = PeripheralBus::list_devices(&conn).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].device_id, "SCALE-01");
        assert_eq!(list[0].role, PeripheralDeviceRole::WeightScale);

        PeripheralBus::update_device_status(&conn, "SCALE-01", PeripheralStatus::Disconnected).unwrap();
        let list2 = PeripheralBus::list_devices(&conn).unwrap();
        assert_eq!(list2[0].status, PeripheralStatus::Disconnected);
    }

    #[test]
    fn test_parse_weight_scale_frame() {
        let raw = b"\x02ST,GS,+001.250kg\x03";
        let reading = HardwareProtocol::parse_scale_frame(raw).unwrap();
        assert_eq!(reading.net_weight_grams, 1250);
        assert_eq!(reading.unit, "kg");
        assert!(reading.is_stable);
        assert!(!reading.is_overweight);

        let raw_neg = b"-000.050kg";
        let reading_neg = HardwareProtocol::parse_scale_frame(raw_neg).unwrap();
        assert_eq!(reading_neg.net_weight_grams, -50);
    }

    #[test]
    fn test_parse_barcode_scanner_frame() {
        let raw = b"\x025201234567890\r\n\x03";
        let barcode = HardwareProtocol::parse_scanner_frame(raw).unwrap();
        assert_eq!(barcode, "5201234567890");
    }

    #[test]
    fn test_format_vfd_pole_display() {
        let bytes = HardwareProtocol::format_vfd_pole_display("TOTAL: 24.50 EUR", "THANK YOU");
        assert!(bytes.starts_with(&[0x1B, 0x40]));
        assert!(bytes.contains(&b'T'));
        // 2 init + 4 line1 pos + 20 line1 text + 4 line2 pos + 20 line2 text = 50 bytes
        assert_eq!(bytes.len(), 50);
    }

    #[test]
    fn test_modbus_relay_frame_and_crc() {
        let frame = HardwareProtocol::format_modbus_relay_frame(1, true, Some(500));
        assert_eq!(frame[0], 0x01);
        assert_eq!(frame[1], 0x05);
        assert_eq!(frame[3], 0x01);
        assert_eq!(frame[4], 0xFF);
        assert_eq!(frame[5], 0x05);

        let crc = HardwareProtocol::calculate_crc16_modbus(&frame[..6]);
        assert_eq!(frame[6], (crc & 0xFF) as u8);
        assert_eq!(frame[7], ((crc >> 8) & 0xFF) as u8);
    }

    #[test]
    fn test_sensor_telemetry_json_parsing() {
        let raw = br#"{"temp": 3.8, "hum": 68.2, "bat": 88}"#;
        let payload = HardwareProtocol::parse_sensor_frame(raw).unwrap();
        match payload {
            InboundPeripheralPayload::SensorTelemetry { temp_c, humidity_pct, battery_pct } => {
                assert!((temp_c - 3.8).abs() < 0.01);
                assert!((humidity_pct - 68.2).abs() < 0.01);
                assert_eq!(battery_pct, Some(88));
            }
            _ => panic!("Expected SensorTelemetry"),
        }
    }

    #[test]
    fn test_event_recording_in_real_sqlite() {
        let conn = Connection::open_in_memory().unwrap();
        PeripheralBus::init_schema(&conn).unwrap();

        PeripheralBus::record_event(&conn, "DEV-01", "FRAME_RX", b"TEST1234", "Received test barcode").unwrap();

        let count: i64 = conn.query_row("SELECT COUNT(*) FROM system_peripheral_events", [], |r| r.get(0)).unwrap();
        assert_eq!(count, 1);
    }
}
