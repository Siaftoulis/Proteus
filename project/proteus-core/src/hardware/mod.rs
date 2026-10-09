//! Universal Custom Hardware & Open-Source Peripheral Bus (Phase 25).
//! Provides plug-and-play hardware abstraction for retail peripherals,
//! serial scales, barcode scanners, customer displays, and IoT relay controllers.

pub mod bus;
pub mod profile;
pub mod protocol;
pub mod types;

pub use bus::PeripheralBus;
pub use profile::{
    HardwareBinding, HardwareComplianceReport, HardwareProfileEngine, HardwareProfileManifest,
    HardwareRequirement,
};
pub use protocol::HardwareProtocol;
pub use types::{
    HardwareBusError, InboundPeripheralPayload, PeripheralCommand, PeripheralDeviceRecord,
    PeripheralDeviceRole, PeripheralProtocol, PeripheralStatus, ScaleReading, SerialParity,
};
