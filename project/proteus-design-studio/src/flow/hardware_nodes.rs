//! Visual Hardware Scratch Nodes in Flow Studio (Micro-task 25.1.3).
//! Integrates physical relays, environmental telemetry sensors, weight scales,
//! and VFD pole displays into the visual DAG Flow engine (Rule 5: Zero mock data).

use proteus_core::hardware::{HardwareProtocol, PeripheralBus, PeripheralCommand, PeripheralDeviceRole};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SensorEvaluation {
    pub metric: String,
    pub current_value: f32,
    pub is_breached: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HardwareExecutionResult {
    pub success: bool,
    pub output_data: Option<String>,
    pub error_message: Option<String>,
}

pub struct HardwareNodeRunner;

impl HardwareNodeRunner {
    /// Executes an industrial relay command against a physical or virtual relay actuator.
    pub fn execute_relay(
        conn: &Connection,
        device_id: &str,
        channel: u8,
        action: &str,
        pulse_ms: u32,
    ) -> HardwareExecutionResult {
        let is_turn_on = action.eq_ignore_ascii_case("on") || action.eq_ignore_ascii_case("pulse");
        let pulse_opt = if action.eq_ignore_ascii_case("pulse") {
            Some(pulse_ms.max(100))
        } else {
            None
        };

        let cmd = PeripheralCommand::RelayTrigger {
            channel,
            turn_on: is_turn_on,
            pulse_ms: pulse_opt,
        };

        match HardwareProtocol::format_command_payload(PeripheralDeviceRole::IndustrialRelayActuator, &cmd) {
            Ok(bytes) => {
                let summary = format!("Relay ch{} -> action: {}, pulse: {:?}", channel, action, pulse_opt);
                let _ = PeripheralBus::record_event(conn, device_id, "RELAY_TRIGGER", &bytes, &summary);
                HardwareExecutionResult {
                    success: true,
                    output_data: Some(summary),
                    error_message: None,
                }
            }
            Err(e) => HardwareExecutionResult {
                success: false,
                output_data: None,
                error_message: Some(e.to_string()),
            },
        }
    }

    /// Evaluates sensor telemetry against watchdog thresholds.
    pub fn evaluate_sensor(
        current_value: f32,
        metric: &str,
        operator: &str,
        threshold: f32,
    ) -> SensorEvaluation {
        let is_breached = match operator {
            ">" => current_value > threshold,
            ">=" => current_value >= threshold,
            "<" => current_value < threshold,
            "<=" => current_value <= threshold,
            "==" => (current_value - threshold).abs() < f32::EPSILON,
            "!=" => (current_value - threshold).abs() >= f32::EPSILON,
            _ => false,
        };

        SensorEvaluation {
            metric: metric.to_string(),
            current_value,
            is_breached,
        }
    }

    /// Dispatches text update to a customer pole display.
    pub fn execute_display(
        conn: &Connection,
        device_id: &str,
        line1: &str,
        line2: &str,
    ) -> HardwareExecutionResult {
        let cmd = PeripheralCommand::DisplayText {
            line1: line1.to_string(),
            line2: line2.to_string(),
        };

        match HardwareProtocol::format_command_payload(PeripheralDeviceRole::CustomerPoleDisplay, &cmd) {
            Ok(bytes) => {
                let summary = format!("VFD update: [{}] / [{}]", line1, line2);
                let _ = PeripheralBus::record_event(conn, device_id, "VFD_DISPLAY", &bytes, &summary);
                HardwareExecutionResult {
                    success: true,
                    output_data: Some(summary),
                    error_message: None,
                }
            }
            Err(e) => HardwareExecutionResult {
                success: false,
                output_data: None,
                error_message: Some(e.to_string()),
            },
        }
    }
}

/// Renders visual inspector controls for hardware flow nodes.
pub fn render_hardware_inspector(ui: &mut egui::Ui, kind: &mut crate::flow::FlowNodeKind) {
    match kind {
        crate::flow::FlowNodeKind::HardwareRelay { device_id, channel, action, pulse_ms } => {
            ui.label(egui::RichText::new("Peripheral Device ID:").size(10.));
            ui.add_space(2.);
            ui.add(egui::TextEdit::singleline(device_id).hint_text("e.g. RELAY-01").desired_width(f32::INFINITY));
            ui.add_space(4.);
            ui.label(egui::RichText::new("Relay Channel (1-16):").size(10.));
            ui.add_space(2.);
            ui.add(egui::DragValue::new(channel).range(1..=16));
            ui.add_space(4.);
            ui.label(egui::RichText::new("Action (pulse / on / off):").size(10.));
            ui.add_space(2.);
            ui.add(egui::TextEdit::singleline(action).hint_text("pulse, on, off").desired_width(f32::INFINITY));
            ui.add_space(4.);
            ui.label(egui::RichText::new("Pulse Duration (ms):").size(10.));
            ui.add_space(2.);
            ui.add(egui::DragValue::new(pulse_ms).range(50..=10000));
        }
        crate::flow::FlowNodeKind::HardwareSensor { device_id, metric, operator, threshold } => {
            ui.label(egui::RichText::new("Peripheral Device ID:").size(10.));
            ui.add_space(2.);
            ui.add(egui::TextEdit::singleline(device_id).hint_text("e.g. SENSOR-TEMP-01").desired_width(f32::INFINITY));
            ui.add_space(4.);
            ui.label(egui::RichText::new("Telemetry Metric:").size(10.));
            ui.add_space(2.);
            ui.add(egui::TextEdit::singleline(metric).hint_text("e.g. temperature_c").desired_width(f32::INFINITY));
            ui.add_space(4.);
            ui.label(egui::RichText::new("Condition Operator:").size(10.));
            ui.add_space(2.);
            ui.add(egui::TextEdit::singleline(operator).hint_text(">, <, >=, <=, ==").desired_width(f32::INFINITY));
            ui.add_space(4.);
            ui.label(egui::RichText::new("Threshold Value:").size(10.));
            ui.add_space(2.);
            ui.add(egui::DragValue::new(threshold).speed(0.1));
        }
        crate::flow::FlowNodeKind::HardwareScale { device_id, require_stable } => {
            ui.label(egui::RichText::new("Peripheral Device ID:").size(10.));
            ui.add_space(2.);
            ui.add(egui::TextEdit::singleline(device_id).hint_text("e.g. SCALE-CAS-01").desired_width(f32::INFINITY));
            ui.add_space(4.);
            ui.checkbox(require_stable, egui::RichText::new("Require Stable Weight Reading").size(10.));
        }
        crate::flow::FlowNodeKind::HardwareDisplay { device_id, line1, line2 } => {
            ui.label(egui::RichText::new("Peripheral Device ID:").size(10.));
            ui.add_space(2.);
            ui.add(egui::TextEdit::singleline(device_id).hint_text("e.g. VFD-POLE-01").desired_width(f32::INFINITY));
            ui.add_space(4.);
            ui.label(egui::RichText::new("Line 1 Template (ESC/POS 20 chars):").size(10.));
            ui.add_space(2.);
            ui.add(egui::TextEdit::singleline(line1).hint_text("TOTAL: {{amount}}").desired_width(f32::INFINITY));
            ui.add_space(4.);
            ui.label(egui::RichText::new("Line 2 Template (ESC/POS 20 chars):").size(10.));
            ui.add_space(2.);
            ui.add(egui::TextEdit::singleline(line2).hint_text("THANK YOU").desired_width(f32::INFINITY));
        }
        _ => {}
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use proteus_core::hardware::{PeripheralDeviceRecord, PeripheralProtocol, PeripheralStatus};

    #[test]
    fn test_relay_node_execution_on_registered_actuator() {
        let conn = Connection::open_in_memory().unwrap();
        PeripheralBus::init_schema(&conn).unwrap();

        let dev = PeripheralDeviceRecord {
            device_id: "RELAY-01".into(),
            name: "Gate Relay".into(),
            role: PeripheralDeviceRole::IndustrialRelayActuator,
            protocol: PeripheralProtocol::VirtualLoopback { channel_id: "L1".into() },
            status: PeripheralStatus::OnlineReady,
            is_enabled: true,
            last_seen_at: None,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        PeripheralBus::register_device(&conn, &dev).unwrap();

        let res = HardwareNodeRunner::execute_relay(&conn, "RELAY-01", 1, "pulse", 500);
        assert!(res.success);
        assert!(res.output_data.is_some());

        let events_count: i64 = conn.query_row("SELECT COUNT(*) FROM system_peripheral_events", [], |r| r.get(0)).unwrap();
        assert_eq!(events_count, 1);
    }

    #[test]
    fn test_sensor_watchdog_threshold_breach_and_normal() {
        // Temperature > 4.0 °C threshold breach
        let eval_breached = HardwareNodeRunner::evaluate_sensor(6.5, "temperature_c", ">", 4.0);
        assert!(eval_breached.is_breached);
        assert_eq!(eval_breached.current_value, 6.5);

        // Temperature normal
        let eval_normal = HardwareNodeRunner::evaluate_sensor(2.8, "temperature_c", ">", 4.0);
        assert!(!eval_normal.is_breached);
    }

    #[test]
    fn test_display_node_formatting_and_audit() {
        let conn = Connection::open_in_memory().unwrap();
        PeripheralBus::init_schema(&conn).unwrap();

        let res = HardwareNodeRunner::execute_display(&conn, "VFD-01", "PROTEUS RETAIL", "READY");
        assert!(res.success);

        let events_count: i64 = conn.query_row("SELECT COUNT(*) FROM system_peripheral_events", [], |r| r.get(0)).unwrap();
        assert_eq!(events_count, 1);
    }
}
