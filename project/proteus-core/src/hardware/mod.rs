//! Universal Custom Hardware & Open-Source Peripheral Bus (Phase 25).
//! Provides plug-and-play hardware abstraction for retail peripherals,
//! serial scales, barcode scanners, customer displays, and IoT relay controllers.

pub mod bus;
pub mod label;
pub mod profile;
pub mod protocol;
pub mod templates;
pub mod tspl;
pub mod types;
pub mod zpl;
pub mod spooler;
pub mod dispatcher;
pub mod scales;

pub use bus::PeripheralBus;
pub use label::{
    BarcodeSymbology, IndustrialLabel, LabelDimensions, LabelElement, LabelRotation, PrintDensity,
};
pub use templates::{ShelfEdgePriceTemplate, SmartToteWaybillTemplate};
pub use tspl::TsplBuilder;
pub use zpl::ZplBuilder;
pub use spooler::{
    enqueue_print_job, get_next_pending_job, init_label_spooler_schema, list_recent_jobs,
    purge_completed_jobs, update_job_status, LabelPrintJob, PrintJobStatus, PrintTargetProtocol,
};
pub use dispatcher::{dispatch_payload_tcp, dispatch_print_job, PrintEndpoint};
pub use scales::{
    compute_metrology_tamper_hash, compute_tare_deduction, init_scale_metrology_schema,
    list_scale_metrology_logs, log_scale_metrology_record, record_weighed_cart_item,
    verify_weighed_item_interlock, LiveScaleState, ScaleBroadcastHub, ScaleBridgeError,
    ScaleFrameParser, ScaleMetrologyError, ScaleMetrologyRecord, ScaleStreamProcessor,
    ScaleVendorProtocol, TareEngine, WeighedCartItem,
};
pub use profile::{
    HardwareBinding, HardwareComplianceReport, HardwareProfileEngine, HardwareProfileManifest,
    HardwareRequirement,
};
pub use protocol::HardwareProtocol;
pub use types::{
    HardwareBusError, InboundPeripheralPayload, PeripheralCommand, PeripheralDeviceRecord,
    PeripheralDeviceRole, PeripheralProtocol, PeripheralStatus, ScaleReading, SerialParity,
};
