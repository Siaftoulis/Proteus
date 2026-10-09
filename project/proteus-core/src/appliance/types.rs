//! Type definitions for Proteus Sovereign Appliance (Phase 26).
//! Models bare-metal kiosk architectures, bootloaders, partition layouts, and image formats.

use serde::{Deserialize, Serialize};

/// Target processor architecture for the appliance OS image.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TargetArch {
    X86_64,
    Aarch64,
}

impl TargetArch {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::X86_64 => "x86_64",
            Self::Aarch64 => "aarch64",
        }
    }

    pub fn kernel_image_name(&self) -> &'static str {
        match self {
            Self::X86_64 => "vmlinuz",
            Self::Aarch64 => "Image.gz",
        }
    }

    pub fn default_bootloader(&self) -> BootloaderKind {
        match self {
            Self::X86_64 => BootloaderKind::SystemdBoot,
            Self::Aarch64 => BootloaderKind::RpiBootloader,
        }
    }
}

/// Operational role and UI environment profile for the bare-metal appliance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApplianceRole {
    RetailPosTerminal,
    KitchenDisplayKiosk,
    WarehouseScannerStation,
    EdgeDatabaseServer,
    CustomerSelfCheckout,
    Custom(String),
}

impl ApplianceRole {
    pub fn label(&self) -> &str {
        match self {
            Self::RetailPosTerminal => "Retail POS Terminal",
            Self::KitchenDisplayKiosk => "Kitchen Display Kiosk (KDS)",
            Self::WarehouseScannerStation => "Warehouse Scanner Station",
            Self::EdgeDatabaseServer => "Edge Database Server (Headless)",
            Self::CustomerSelfCheckout => "Customer Self-Checkout Kiosk",
            Self::Custom(name) => name.as_str(),
        }
    }

    pub fn is_headless(&self) -> bool {
        matches!(self, Self::EdgeDatabaseServer)
    }
}

/// Supported bootloader mechanism for system initialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BootloaderKind {
    SystemdBoot,
    GrubEfi,
    RpiBootloader,
}

impl BootloaderKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SystemdBoot => "systemd-boot",
            Self::GrubEfi => "grub-efi",
            Self::RpiBootloader => "rpi-bootloader",
        }
    }
}

/// Storage partition scheme for the target disk / eMMC / NVMe image.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoragePartitionScheme {
    pub efi_boot_mb: u32,
    pub rootfs_mb: u32,
    pub persistent_data_mb: u32,
    pub swap_mb: u32,
}

impl Default for StoragePartitionScheme {
    fn default() -> Self {
        Self {
            efi_boot_mb: 256,
            rootfs_mb: 2048,
            persistent_data_mb: 8192,
            swap_mb: 0,
        }
    }
}

/// Display and kiosk ergonomics configuration for the appliance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KioskDisplayConfig {
    pub resolution_width: u32,
    pub resolution_height: u32,
    pub rotation_degrees: u16,
    pub cursor_visible: bool,
    pub screensaver_timeout_secs: u32,
    pub auto_restart_on_crash: bool,
}

impl Default for KioskDisplayConfig {
    fn default() -> Self {
        Self {
            resolution_width: 1920,
            resolution_height: 1080,
            rotation_degrees: 0,
            cursor_visible: false,
            screensaver_timeout_secs: 0,
            auto_restart_on_crash: true,
        }
    }
}

/// Network bootstrap configuration for first-boot provisioning.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkBootstrapConfig {
    pub hostname: String,
    pub dhcp_enabled: bool,
    pub static_ipv4: Option<String>,
    pub gateway_ipv4: Option<String>,
    pub dns_servers: Vec<String>,
    pub fallback_cellular_hotspot: bool,
    pub ssh_enabled: bool,
    pub ssh_authorized_keys: Vec<String>,
}

impl Default for NetworkBootstrapConfig {
    fn default() -> Self {
        Self {
            hostname: "proteus-appliance".to_string(),
            dhcp_enabled: true,
            static_ipv4: None,
            gateway_ipv4: None,
            dns_servers: vec!["1.1.1.1".to_string(), "8.8.8.8".to_string()],
            fallback_cellular_hotspot: true,
            ssh_enabled: false,
            ssh_authorized_keys: Vec::new(),
        }
    }
}

/// Supported binary distribution formats for appliance deployment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApplianceImageFormat {
    HybridIso,
    RawGptImage,
    RpiSdCardImage,
}

impl ApplianceImageFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            Self::HybridIso => "iso",
            Self::RawGptImage => "img",
            Self::RpiSdCardImage => "img",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::HybridIso => "x86_64 Hybrid UEFI/BIOS ISO",
            Self::RawGptImage => "x86_64 Raw GPT Disk Image",
            Self::RpiSdCardImage => "ARM64 RPi SD/eMMC Image",
        }
    }
}

/// Description of an individual partition within the generated appliance disk image.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartitionEntryDescriptor {
    pub partition_index: u8,
    pub label: String,
    pub fs_type: String,
    pub start_sector: u64,
    pub sector_count: u64,
    pub is_bootable: bool,
}

/// Calculated disk layout and geometry plan before image compilation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageLayoutPlan {
    pub sector_size: u32,
    pub total_sectors: u64,
    pub total_bytes: u64,
    pub partitions: Vec<PartitionEntryDescriptor>,
}

/// In-memory representation of an EFI or bootloader file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VirtualFsFile {
    pub path: String,
    pub contents: Vec<u8>,
}

/// Metadata descriptor of a compiled appliance image artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildArtifactDescriptor {
    pub build_id: String,
    pub profile_id: String,
    pub format: ApplianceImageFormat,
    pub arch: TargetArch,
    pub total_size_bytes: u64,
    pub sha256_checksum: String,
    pub partition_count: usize,
    pub manifest_fingerprint: String,
    pub built_at: String,
}
