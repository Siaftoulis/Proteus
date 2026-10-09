//! Multi-Protocol Peripheral Bus Types & Data Definitions (Micro-task 25.1.1).
//! Standardizes hardware interfaces across Serial UART, USB-HID, BLE, and MQTT brokers.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SerialParity {
    None,
    Even,
    Odd,
    Mark,
    Space,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PeripheralProtocol {
    SerialUart {
        port_name: String,
        baud_rate: u32,
        data_bits: u8,
        stop_bits: u8,
        parity: SerialParity,
    },
    UsbHid {
        vendor_id: u16,
        product_id: u16,
        serial_number: Option<String>,
        usage_page: u16,
    },
    BluetoothBle {
        mac_address: String,
        service_uuid: String,
        characteristic_uuid: String,
    },
    MqttBroker {
        broker_url: String,
        port: u16,
        topic_prefix: String,
        client_id: String,
    },
    VirtualLoopback {
        channel_id: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PeripheralDeviceRole {
    WeightScale,
    BarcodeScanner,
    CustomerPoleDisplay,
    IndustrialRelayActuator,
    EnvironmentalSensor,
    FiscalSignatureBox,
    CustomMacroPad,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PeripheralStatus {
    Disconnected,
    Connecting,
    OnlineReady,
    Transmitting,
    HardwareError(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeripheralDeviceRecord {
    pub device_id: String,
    pub name: String,
    pub role: PeripheralDeviceRole,
    pub protocol: PeripheralProtocol,
    pub status: PeripheralStatus,
    pub is_enabled: bool,
    pub last_seen_at: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScaleReading {
    pub net_weight_grams: i64,
    pub tare_weight_grams: i64,
    pub unit: String,
    pub is_stable: bool,
    pub is_overweight: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InboundPeripheralPayload {
    Raw(Vec<u8>),
    Barcode(String),
    Weight(ScaleReading),
    SensorTelemetry {
        temp_c: f32,
        humidity_pct: f32,
        battery_pct: Option<u8>,
    },
    RelayAck {
        channel: u8,
        state: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PeripheralCommand {
    RawBytes(Vec<u8>),
    DisplayText {
        line1: String,
        line2: String,
    },
    RelayTrigger {
        channel: u8,
        turn_on: bool,
        pulse_ms: Option<u32>,
    },
    ZeroScale,
    TareScale,
}

#[derive(Debug, Error, PartialEq)]
pub enum HardwareBusError {
    #[error("Device not found on hardware bus: {0}")]
    DeviceNotFound(String),
    #[error("Device is disabled: {0}")]
    DeviceDisabled(String),
    #[error("Device offline or disconnected: {0}")]
    DeviceOffline(String),
    #[error("Protocol error: {0}")]
    ProtocolMismatch(String),
    #[error("Invalid framing or checksum mismatch in frame")]
    InvalidFraming,
    #[error("Database error: {0}")]
    Database(String),
    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl From<rusqlite::Error> for HardwareBusError {
    fn from(err: rusqlite::Error) -> Self {
        HardwareBusError::Database(err.to_string())
    }
}

impl From<serde_json::Error> for HardwareBusError {
    fn from(err: serde_json::Error) -> Self {
        HardwareBusError::Serialization(err.to_string())
    }
}
