//! Bare-Metal Appliance Profile Manifest & Immutable RootFS Engine (Micro-task 26.1.1).
//! Generates tamper-proof read-only OS specifications, systemd service units,
//! fstab overlays, and stores appliance profiles in SQLite.

use super::types::{
    ApplianceRole, BootloaderKind, KioskDisplayConfig, NetworkBootstrapConfig,
    StoragePartitionScheme, TargetArch,
};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Immutable read-only rootfs specification with dm-verity / checksum enforcement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImmutableRootFsSpec {
    /// Read-only filesystem format ("squashfs" or "erofs").
    pub fs_type: String,
    /// Compression algorithm ("zstd" or "lz4").
    pub compression: String,
    /// Cryptographic SHA-256 root integrity digest for tamper detection.
    pub root_hash_sha256: String,
    /// Whether dm-verity kernel verification block is attached.
    pub verity_enabled: bool,
    /// Volatile memory-backed write overlay directories mounted via tmpfs.
    pub tmpfs_overlays: Vec<String>,
    /// Persistent encrypted mount path for sovereign database.
    pub persistent_mount_point: String,
}

impl Default for ImmutableRootFsSpec {
    fn default() -> Self {
        Self {
            fs_type: "squashfs".to_string(),
            compression: "zstd".to_string(),
            root_hash_sha256: String::new(),
            verity_enabled: true,
            tmpfs_overlays: vec![
                "/tmp".to_string(),
                "/run".to_string(),
                "/var/volatile".to_string(),
                "/var/log".to_string(),
            ],
            persistent_mount_point: "/var/lib/proteus".to_string(),
        }
    }
}

/// Declarative profile manifest for building or provisioning a Proteus Appliance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApplianceManifest {
    pub profile_id: String,
    pub name: String,
    pub role: ApplianceRole,
    pub target_arch: TargetArch,
    pub bootloader: BootloaderKind,
    pub partitions: StoragePartitionScheme,
    pub rootfs_spec: ImmutableRootFsSpec,
    pub display: KioskDisplayConfig,
    pub network: NetworkBootstrapConfig,
    pub hardware_profile_id: Option<String>,
    pub pr_package_id: Option<String>,
    pub created_at: String,
}

impl ApplianceManifest {
    /// Calculates the cryptographic SHA-256 fingerprint of the manifest definition.
    pub fn compute_fingerprint(&self) -> String {
        let json = serde_json::to_string(self).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Generates a hardened systemd unit file for launching Proteus in kiosk mode.
    pub fn generate_systemd_kiosk_service(&self) -> String {
        let binary_path = "/usr/bin/proteus-appliance";
        let restart_sec = if self.display.auto_restart_on_crash { "2s" } else { "infinity" };
        let kiosk_env = if self.role.is_headless() {
            "Environment=PROTEUS_HEADLESS=1"
        } else {
            "Environment=DISPLAY=:0\nEnvironment=PROTEUS_KIOSK=1"
        };

        format!(
            "[Unit]\n\
             Description=Proteus Sovereign Appliance Kiosk ({name})\n\
             After=network-online.target local-fs.target\n\
             Wants=network-online.target\n\
             Conflicts=getty@tty1.service\n\n\
             [Service]\n\
             Type=simple\n\
             User=proteus\n\
             Group=proteus\n\
             WorkingDirectory=/var/lib/proteus\n\
             {kiosk_env}\n\
             ExecStart={binary_path}\n\
             Restart=always\n\
             RestartSec={restart_sec}\n\
             StandardInput=tty\n\
             TTYPath=/dev/tty1\n\
             TTYReset=yes\n\
             TTYVHangup=yes\n\
             ProtectSystem=strict\n\
             ProtectHome=read-only\n\
             ReadWritePaths=/var/lib/proteus /tmp /run\n\n\
             [Install]\n\
             WantedBy=multi-user.target\n",
            name = self.name,
            kiosk_env = kiosk_env,
            binary_path = binary_path,
            restart_sec = restart_sec,
        )
    }

    /// Generates the immutable /etc/fstab configuration with read-only root and tmpfs overlays.
    pub fn generate_fstab(&self) -> String {
        let mut fstab = String::new();
        fstab.push_str("# Proteus Sovereign Appliance Immutable Filesystem Table\n");
        fstab.push_str("LABEL=PROTEUS_BOOT  /boot           vfat    ro,defaults,noatime         0 2\n");
        fstab.push_str("LABEL=PROTEUS_ROOT  /               squashfs ro,defaults,noatime        0 1\n");
        fstab.push_str("LABEL=PROTEUS_DATA  /var/lib/proteus ext4   rw,defaults,noatime,barrier=1 0 2\n");
        for overlay in &self.rootfs_spec.tmpfs_overlays {
            fstab.push_str(&format!("tmpfs               {}          tmpfs   rw,nosuid,nodev,mode=1777   0 0\n", overlay));
        }
        fstab
    }

    /// Generates bootloader kernel cmdline configuration.
    pub fn generate_kernel_cmdline(&self) -> String {
        let mut cmd = format!(
            "root=LABEL=PROTEUS_ROOT ro rootfstype={} quiet splash console=tty1",
            self.rootfs_spec.fs_type
        );
        if !self.rootfs_spec.root_hash_sha256.is_empty() && self.rootfs_spec.verity_enabled {
            cmd.push_str(&format!(" dm-verity.roothash={}", self.rootfs_spec.root_hash_sha256));
        }
        if self.display.rotation_degrees > 0 {
            let fbcon_rot = match self.display.rotation_degrees {
                90 => "1",
                180 => "2",
                270 => "3",
                _ => "0",
            };
            cmd.push_str(&format!(" fbcon=rotate:{}", fbcon_rot));
        }
        cmd
    }
}

/// SQLite persistence manager for appliance profiles in `store.db`.
pub struct ApplianceEngine;

impl ApplianceEngine {
    /// Initializes the `system_appliance_profiles` table.
    pub fn init_schema(conn: &Connection) -> Result<(), rusqlite::Error> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS system_appliance_profiles (
                profile_id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                role TEXT NOT NULL,
                target_arch TEXT NOT NULL,
                bootloader TEXT NOT NULL,
                manifest_json TEXT NOT NULL,
                fingerprint TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    /// Saves or updates an appliance profile.
    pub fn save_profile(conn: &Connection, manifest: &ApplianceManifest) -> Result<(), rusqlite::Error> {
        let manifest_json = serde_json::to_string(manifest).map_err(|e| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(e))
        })?;
        let fingerprint = manifest.compute_fingerprint();
        let role_str = manifest.role.label().to_string();
        let arch_str = manifest.target_arch.as_str().to_string();
        let boot_str = manifest.bootloader.as_str().to_string();

        conn.execute(
            "INSERT INTO system_appliance_profiles (
                profile_id, name, role, target_arch, bootloader, manifest_json, fingerprint, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(profile_id) DO UPDATE SET
                name = excluded.name,
                role = excluded.role,
                target_arch = excluded.target_arch,
                bootloader = excluded.bootloader,
                manifest_json = excluded.manifest_json,
                fingerprint = excluded.fingerprint,
                created_at = excluded.created_at",
            params![
                manifest.profile_id,
                manifest.name,
                role_str,
                arch_str,
                boot_str,
                manifest_json,
                fingerprint,
                manifest.created_at,
            ],
        )?;
        Ok(())
    }

    /// Loads an appliance profile by ID.
    pub fn load_profile(conn: &Connection, profile_id: &str) -> Result<Option<ApplianceManifest>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT manifest_json FROM system_appliance_profiles WHERE profile_id = ?1",
        )?;
        let mut rows = stmt.query(params![profile_id])?;
        if let Some(row) = rows.next()? {
            let json_str: String = row.get(0)?;
            let manifest: ApplianceManifest = serde_json::from_str(&json_str).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
            })?;
            Ok(Some(manifest))
        } else {
            Ok(None)
        }
    }

    /// Lists all registered appliance profiles.
    pub fn list_profiles(conn: &Connection) -> Result<Vec<ApplianceManifest>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT manifest_json FROM system_appliance_profiles ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            let json_str: String = row.get(0)?;
            Ok(json_str)
        })?;

        let mut list = Vec::new();
        for r in rows {
            let json_str = r?;
            if let Ok(manifest) = serde_json::from_str::<ApplianceManifest>(&json_str) {
                list.push(manifest);
            }
        }
        Ok(list)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_fingerprint_deterministic() {
        let mut m = ApplianceManifest {
            profile_id: "APPLIANCE-POS-01".into(),
            name: "Athens Flagship Checkout".into(),
            role: ApplianceRole::RetailPosTerminal,
            target_arch: TargetArch::X86_64,
            bootloader: BootloaderKind::SystemdBoot,
            partitions: StoragePartitionScheme::default(),
            rootfs_spec: ImmutableRootFsSpec::default(),
            display: KioskDisplayConfig::default(),
            network: NetworkBootstrapConfig::default(),
            hardware_profile_id: Some("HW-PROFILE-RETAIL-01".into()),
            pr_package_id: Some("pkg-retail-athens".into()),
            created_at: "2026-10-09T00:00:00Z".into(),
        };
        m.rootfs_spec.root_hash_sha256 = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".into();

        let fp1 = m.compute_fingerprint();
        let fp2 = m.compute_fingerprint();
        assert_eq!(fp1, fp2);
        assert_eq!(fp1.len(), 64);
    }

    #[test]
    fn test_systemd_kiosk_service_generation() {
        let m = ApplianceManifest {
            profile_id: "APPLIANCE-KDS-01".into(),
            name: "Kitchen Display Unit".into(),
            role: ApplianceRole::KitchenDisplayKiosk,
            target_arch: TargetArch::Aarch64,
            bootloader: BootloaderKind::RpiBootloader,
            partitions: StoragePartitionScheme::default(),
            rootfs_spec: ImmutableRootFsSpec::default(),
            display: KioskDisplayConfig::default(),
            network: NetworkBootstrapConfig::default(),
            hardware_profile_id: None,
            pr_package_id: None,
            created_at: "2026-10-09T00:00:00Z".into(),
        };

        let unit = m.generate_systemd_kiosk_service();
        assert!(unit.contains("Description=Proteus Sovereign Appliance Kiosk (Kitchen Display Unit)"));
        assert!(unit.contains("PROTEUS_KIOSK=1"));
        assert!(unit.contains("ProtectSystem=strict"));
        assert!(unit.contains("ReadWritePaths=/var/lib/proteus /tmp /run"));
    }

    #[test]
    fn test_fstab_and_kernel_cmdline_generation() {
        let mut m = ApplianceManifest {
            profile_id: "APPLIANCE-02".into(),
            name: "Warehouse Scanner".into(),
            role: ApplianceRole::WarehouseScannerStation,
            target_arch: TargetArch::X86_64,
            bootloader: BootloaderKind::SystemdBoot,
            partitions: StoragePartitionScheme::default(),
            rootfs_spec: ImmutableRootFsSpec::default(),
            display: KioskDisplayConfig {
                rotation_degrees: 90,
                ..KioskDisplayConfig::default()
            },
            network: NetworkBootstrapConfig::default(),
            hardware_profile_id: None,
            pr_package_id: None,
            created_at: "2026-10-09T00:00:00Z".into(),
        };
        m.rootfs_spec.root_hash_sha256 = "abcdef1234567890".into();

        let fstab = m.generate_fstab();
        assert!(fstab.contains("LABEL=PROTEUS_ROOT  /               squashfs ro"));
        assert!(fstab.contains("tmpfs               /var/volatile          tmpfs"));

        let cmdline = m.generate_kernel_cmdline();
        assert!(cmdline.contains("dm-verity.roothash=abcdef1234567890"));
        assert!(cmdline.contains("fbcon=rotate:1"));
    }

    #[test]
    fn test_appliance_engine_sqlite_roundtrip() {
        let conn = Connection::open_in_memory().unwrap();
        ApplianceEngine::init_schema(&conn).unwrap();

        let m = ApplianceManifest {
            profile_id: "APP-SQLITE-01".into(),
            name: "Edge Server".into(),
            role: ApplianceRole::EdgeDatabaseServer,
            target_arch: TargetArch::X86_64,
            bootloader: BootloaderKind::SystemdBoot,
            partitions: StoragePartitionScheme::default(),
            rootfs_spec: ImmutableRootFsSpec::default(),
            display: KioskDisplayConfig::default(),
            network: NetworkBootstrapConfig::default(),
            hardware_profile_id: None,
            pr_package_id: None,
            created_at: "2026-10-09T01:00:00Z".into(),
        };

        ApplianceEngine::save_profile(&conn, &m).unwrap();

        let loaded = ApplianceEngine::load_profile(&conn, "APP-SQLITE-01").unwrap().unwrap();
        assert_eq!(loaded.name, "Edge Server");
        assert_eq!(loaded.role, ApplianceRole::EdgeDatabaseServer);

        let list = ApplianceEngine::list_profiles(&conn).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].profile_id, "APP-SQLITE-01");
    }
}
