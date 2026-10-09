//! Proteus Sovereign Appliance & Bootable ISO/IMG Engine (Phase 26).
//! Provides turnkey bare-metal OS manifests, immutable read-only rootfs configurations,
//! and hardware kiosk deployment profiles.

pub mod builder;
pub mod manifest;
pub mod types;

pub use builder::ApplianceImageBuilder;
pub use manifest::{ApplianceEngine, ApplianceManifest, ImmutableRootFsSpec};
pub use types::{
    ApplianceImageFormat, ApplianceRole, BootloaderKind, BuildArtifactDescriptor,
    ImageLayoutPlan, KioskDisplayConfig, NetworkBootstrapConfig, PartitionEntryDescriptor,
    StoragePartitionScheme, TargetArch, VirtualFsFile,
};
