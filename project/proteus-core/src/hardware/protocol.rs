//! Hardware Protocol Framing, Parsing & Checksum Utilities (Micro-task 25.1.1).
//! Standardizes serial scale protocols (Toledo/CAS/Dibal), barcode scanners,
//! ESC/POS VFD pole displays, and Modbus-RTU relay controllers.

use super::types::{
    HardwareBusError, InboundPeripheralPayload, PeripheralCommand, PeripheralDeviceRole, ScaleReading,
};

pub struct HardwareProtocol;

impl HardwareProtocol {
    /// Parses standard scale formats (Toledo, Dibal, CAS): e.g. "ST,GS,+001.250kg" or "+001.250kg".
    pub fn parse_scale_frame(raw: &[u8]) -> Result<ScaleReading, HardwareBusError> {
        let text = String::from_utf8_lossy(raw);
        let trimmed = text.trim_matches(|c: char| c.is_control() || c.is_whitespace());

        if trimmed.is_empty() {
            return Err(HardwareBusError::InvalidFraming);
        }

        let is_stable = !trimmed.contains("US");
        let is_overweight = trimmed.contains("OL") || trimmed.contains("OVER");

        // Extract numeric digits and decimal point
        let mut sign: i64 = 1;
        let mut num_str = String::new();
        let mut found_unit = "kg".to_string();

        if trimmed.contains('-') {
            sign = -1;
        }
        if trimmed.to_lowercase().contains('g') && !trimmed.to_lowercase().contains("kg") {
            found_unit = "g".to_string();
        }

        for ch in trimmed.chars() {
            if ch.is_ascii_digit() || ch == '.' {
                num_str.push(ch);
            }
        }

        let val_float: f64 = num_str.parse().unwrap_or(0.0);
        let net_grams = (val_float * if found_unit == "kg" { 1000.0 } else { 1.0 }) as i64 * sign;

        Ok(ScaleReading {
            net_weight_grams: net_grams,
            tare_weight_grams: 0,
            unit: found_unit,
            is_stable,
            is_overweight,
        })
    }

    /// Strips scanner preambles, postambles (STX, ETX, CR, LF).
    pub fn parse_scanner_frame(raw: &[u8]) -> Result<String, HardwareBusError> {
        let text = String::from_utf8_lossy(raw);
        let cleaned: String = text
            .chars()
            .filter(|c| !c.is_control() && *c != '\u{2}' && *c != '\u{3}')
            .collect();

        let trimmed = cleaned.trim().to_string();
        if trimmed.is_empty() {
            return Err(HardwareBusError::InvalidFraming);
        }

        Ok(trimmed)
    }

    /// Parses environmental telemetry payload in JSON format.
    pub fn parse_sensor_frame(raw: &[u8]) -> Result<InboundPeripheralPayload, HardwareBusError> {
        #[derive(serde::Deserialize)]
        struct SensorJson {
            temp: Option<f32>,
            hum: Option<f32>,
            bat: Option<u8>,
        }

        if let Ok(parsed) = serde_json::from_slice::<SensorJson>(raw) {
            return Ok(InboundPeripheralPayload::SensorTelemetry {
                temp_c: parsed.temp.unwrap_or(0.0),
                humidity_pct: parsed.hum.unwrap_or(0.0),
                battery_pct: parsed.bat,
            });
        }

        Ok(InboundPeripheralPayload::Raw(raw.to_vec()))
    }

    /// Formats an outbound hardware command into raw protocol bytes.
    pub fn format_command_payload(
        role: PeripheralDeviceRole,
        cmd: &PeripheralCommand,
    ) -> Result<Vec<u8>, HardwareBusError> {
        match cmd {
            PeripheralCommand::RawBytes(bytes) => Ok(bytes.clone()),
            PeripheralCommand::DisplayText { line1, line2 } => {
                if role != PeripheralDeviceRole::CustomerPoleDisplay {
                    return Err(HardwareBusError::ProtocolMismatch(
                        "DisplayText only supported on CustomerPoleDisplay".into(),
                    ));
                }
                Ok(Self::format_vfd_pole_display(line1, line2))
            }
            PeripheralCommand::RelayTrigger { channel, turn_on, pulse_ms } => {
                if role != PeripheralDeviceRole::IndustrialRelayActuator {
                    return Err(HardwareBusError::ProtocolMismatch(
                        "RelayTrigger only supported on IndustrialRelayActuator".into(),
                    ));
                }
                Ok(Self::format_modbus_relay_frame(*channel, *turn_on, *pulse_ms))
            }
            PeripheralCommand::ZeroScale => Ok(vec![0x1B, b'Z', 0x0D, 0x0A]),
            PeripheralCommand::TareScale => Ok(vec![0x1B, b'T', 0x0D, 0x0A]),
        }
    }

    /// Formats ESC/POS VFD 2x20 pole display sequence.
    pub fn format_vfd_pole_display(line1: &str, line2: &str) -> Vec<u8> {
        let mut out = Vec::new();
        // ESC @ (Initialize display)
        out.extend_from_slice(&[0x1B, 0x40]);

        // Line 1: US $ 01 01 (Move cursor to row 1, col 1)
        out.extend_from_slice(&[0x1F, 0x24, 0x01, 0x01]);
        let l1_padded = format!("{:20}", line1);
        out.extend_from_slice(&l1_padded.as_bytes()[..20.min(l1_padded.len())]);

        // Line 2: US $ 01 02 (Move cursor to row 2, col 1)
        out.extend_from_slice(&[0x1F, 0x24, 0x01, 0x02]);
        let l2_padded = format!("{:20}", line2);
        out.extend_from_slice(&l2_padded.as_bytes()[..20.min(l2_padded.len())]);

        out
    }

    /// Formats standard Modbus-RTU / Serial frame for relay actuators with CRC16.
    pub fn format_modbus_relay_frame(channel: u8, turn_on: bool, pulse_ms: Option<u32>) -> Vec<u8> {
        let mut frame = Vec::new();
        frame.push(0x01); // Slave Address
        frame.push(0x05); // Function Code: Write Single Coil
        frame.push(0x00); // Coil High
        frame.push(channel); // Coil Low (Channel index)
        frame.push(if turn_on { 0xFF } else { 0x00 }); // Value High
        frame.push(pulse_ms.map(|p| (p / 100) as u8).unwrap_or(0x00)); // Value Low (pulse / 100ms or 0)

        let crc = Self::calculate_crc16_modbus(&frame);
        frame.push((crc & 0xFF) as u8);
        frame.push(((crc >> 8) & 0xFF) as u8);
        frame
    }

    /// Calculates Modbus CRC16 (poly 0xA001).
    pub fn calculate_crc16_modbus(data: &[u8]) -> u16 {
        let mut crc: u16 = 0xFFFF;
        for &byte in data {
            crc ^= byte as u16;
            for _ in 0..8 {
                if (crc & 0x0001) != 0 {
                    crc = (crc >> 1) ^ 0xA001;
                } else {
                    crc >>= 1;
                }
            }
        }
        crc
    }

    /// Calculates Longitudinal Redundancy Check (LRC) checksum.
    pub fn calculate_lrc(data: &[u8]) -> u8 {
        let mut lrc: u8 = 0;
        for &byte in data {
            lrc = lrc.wrapping_add(byte);
        }
        (!lrc).wrapping_add(1)
    }
}
