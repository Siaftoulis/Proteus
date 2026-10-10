//! Native Serial RS-232 / USB Weigh Scale Protocol Engine (Phase 13).
//! Provides multi-vendor scale protocol framing, real-time tare/stability compensation,
//! and continuous serial streaming integration with Touch POS and Smart Tote stations.
//! Strict Rule 1 (100% Original Codebase), Rule 5 (Zero Mock Data), Rule 6 (Compliance & Security).

pub mod protocol;
pub mod stream;

pub use protocol::{ScaleFrameParser, ScaleVendorProtocol};
pub use stream::{LiveScaleState, ScaleStreamProcessor, TareEngine};

