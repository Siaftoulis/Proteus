//! Native Serial RS-232 / USB Weigh Scale Protocol Engine (Phase 13).
//! Provides multi-vendor scale protocol framing, real-time tare/stability compensation,
//! and continuous serial streaming integration with Touch POS and Smart Tote stations.
//! Strict Rule 1 (100% Original Codebase), Rule 5 (Zero Mock Data), Rule 6 (Compliance & Security).

pub mod protocol;
pub mod stream;
pub mod bridge;
pub mod metrology;

pub use protocol::{ScaleFrameParser, ScaleVendorProtocol};
pub use stream::{LiveScaleState, ScaleStreamProcessor, TareEngine};
pub use bridge::{ScaleBroadcastHub, ScaleBridgeError, WeighedCartItem};
pub use metrology::{
    compute_metrology_tamper_hash, compute_tare_deduction, init_scale_metrology_schema,
    list_scale_metrology_logs, log_scale_metrology_record, record_weighed_cart_item,
    verify_weighed_item_interlock, ScaleMetrologyError, ScaleMetrologyRecord,
};


