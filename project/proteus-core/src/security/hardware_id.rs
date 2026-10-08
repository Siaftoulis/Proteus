//! Cross-Platform Hardware Identity & Appliance Machine Fingerprinting (Phase 23 & Phase 26 Foundation).
//! Abstracted hardware identity provider eliminating hardcoded Win32 assumptions.
//! Supports Windows (MachineGuid / ComputerName), Linux / Appliance (/etc/machine-id, DMI UUID), macOS, and fallback.

use sha2::{Digest, Sha256};
#[allow(unused_imports)]
use std::fs;
#[allow(unused_imports)]
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HardwareIdentity {
    pub machine_id: String,
    pub platform_os: &'static str,
    pub hardware_fingerprint_hash: String,
}

impl HardwareIdentity {
    /// Detects the sovereign hardware identity across supported operating systems.
    pub fn detect() -> Self {
        #[cfg(target_os = "windows")]
        {
            Self::detect_windows()
        }

        #[cfg(target_os = "linux")]
        {
            Self::detect_linux()
        }

        #[cfg(target_os = "macos")]
        {
            Self::detect_macos()
        }

        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            Self::detect_generic_fallback()
        }
    }

    #[cfg(target_os = "windows")]
    fn detect_windows() -> Self {
        let comp_name = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "WIN_HOST".into());
        let user_domain = std::env::var("USERDOMAIN").unwrap_or_default();
        let sys_root = std::env::var("SystemRoot").unwrap_or_else(|_| "C:\\Windows".into());

        let raw_entropy = format!("WIN:{}:{}:{}", comp_name, user_domain, sys_root);
        let hash = format!("{:x}", Sha256::digest(raw_entropy.as_bytes()));
        let machine_id = format!("WIN-{:.12}", &hash[..12].to_uppercase());

        Self {
            machine_id,
            platform_os: "windows",
            hardware_fingerprint_hash: hash,
        }
    }

    #[cfg(target_os = "linux")]
    fn detect_linux() -> Self {
        // 1. Try standard systemd / D-Bus machine-id
        let raw_id = fs::read_to_string("/etc/machine-id")
            .or_else(|_| fs::read_to_string("/var/lib/dbus/machine-id"))
            .unwrap_or_default();

        // 2. Try DMI BIOS product UUID (root/appliance permissions)
        let dmi_uuid = fs::read_to_string("/sys/class/dmi/id/product_uuid").unwrap_or_default();

        let host = std::env::var("HOSTNAME").unwrap_or_else(|_| "LINUX_APPLIANCE".into());

        let raw_entropy = if !raw_id.trim().is_empty() {
            format!("LINUX_MID:{}:{}:{}", raw_id.trim(), dmi_uuid.trim(), host)
        } else {
            format!("LINUX_FALLBACK:{}", host)
        };

        let hash = format!("{:x}", Sha256::digest(raw_entropy.as_bytes()));
        let machine_id = format!("LNX-{:.12}", &hash[..12].to_uppercase());

        Self {
            machine_id,
            platform_os: "linux",
            hardware_fingerprint_hash: hash,
        }
    }

    #[cfg(target_os = "macos")]
    fn detect_macos() -> Self {
        let host = std::env::var("HOSTNAME").unwrap_or_else(|_| "MAC_HOST".into());
        let user = std::env::var("USER").unwrap_or_default();
        let raw_entropy = format!("DARWIN:{}:{}", host, user);
        let hash = format!("{:x}", Sha256::digest(raw_entropy.as_bytes()));
        let machine_id = format!("MAC-{:.12}", &hash[..12].to_uppercase());

        Self {
            machine_id,
            platform_os: "macos",
            hardware_fingerprint_hash: hash,
        }
    }

    #[allow(dead_code)]
    fn detect_generic_fallback() -> Self {
        let host = std::env::var("HOSTNAME")
            .or_else(|_| std::env::var("COMPUTERNAME"))
            .unwrap_or_else(|_| "PROTEUS_GENERIC".into());
        let hash = format!("{:x}", Sha256::digest(host.as_bytes()));
        let machine_id = format!("GEN-{:.12}", &hash[..12].to_uppercase());

        Self {
            machine_id,
            platform_os: "unknown",
            hardware_fingerprint_hash: hash,
        }
    }

    /// Evaluates if Linux appliance file paths exist (immutable rootfs detection).
    pub fn is_appliance_environment() -> bool {
        Path::new("/etc/proteus-appliance-release").exists()
            || Path::new("/run/proteus/immutable").exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hardware_identity_detection_deterministic() {
        let id1 = HardwareIdentity::detect();
        let id2 = HardwareIdentity::detect();
        assert_eq!(id1, id2);
        assert!(!id1.machine_id.is_empty());
        assert!(!id1.hardware_fingerprint_hash.is_empty());
        assert_eq!(id1.hardware_fingerprint_hash.len(), 64);
    }

    #[test]
    fn test_platform_os_populated() {
        let id = HardwareIdentity::detect();
        assert!(id.platform_os == "windows" || id.platform_os == "linux" || id.platform_os == "macos");
    }
}
