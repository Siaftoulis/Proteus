//! Standalone CRM Application Extraction and Packaging Engine.
//!
//! Enables compiling and packaging customized business schemas, forms, and workflows
//! into autonomous, zero-privilege standalone application distributions (Windows, macOS, Linux, Android, iOS).

pub mod ios;

use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// Target operating system and deployment format for standalone extraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetPlatform {
    WindowsExe,
    WindowsMsi,
    MacOsDmg,
    LinuxAppImage,
    AndroidApk,
    IosAltStore,
}

impl TargetPlatform {
    pub fn extension(&self) -> &'static str {
        match self {
            Self::WindowsExe => "exe",
            Self::WindowsMsi => "msi",
            Self::MacOsDmg => "dmg",
            Self::LinuxAppImage => "AppImage",
            Self::AndroidApk => "apk",
            Self::IosAltStore => "ipa",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::WindowsExe => "Windows Standalone Binary (.exe)",
            Self::WindowsMsi => "Windows Enterprise Installer (.msi)",
            Self::MacOsDmg => "macOS Bundle Disk Image (.dmg)",
            Self::LinuxAppImage => "Linux Universal Package (.AppImage)",
            Self::AndroidApk => "Android Handheld Package (.apk)",
            Self::IosAltStore => "iOS Sideload Bundle (.ipa)",
        }
    }

    pub fn is_mobile(&self) -> bool {
        matches!(self, Self::AndroidApk | Self::IosAltStore)
    }
}

/// Window dimensions and styling metadata for standalone runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    pub title: String,
    pub width: f32,
    pub height: f32,
    pub min_width: f32,
    pub min_height: f32,
    pub resizable: bool,
    pub fullscreen: bool,
    pub accent_color_hex: String,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "Proteus Standalone CRM".to_string(),
            width: 1280.0,
            height: 800.0,
            min_width: 800.0,
            min_height: 600.0,
            resizable: true,
            fullscreen: false,
            accent_color_hex: "#2563EB".to_string(),
        }
    }
}

/// Manifest defining the metadata, security profile, and packaging parameters of a standalone app.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandaloneAppManifest {
    pub app_id: String,
    pub app_name: String,
    pub version: String,
    pub company_name: String,
    pub copyright: String,
    pub target_platform: TargetPlatform,
    pub window_config: WindowConfig,
    pub bundled_package_id: String,
    pub local_db_name: String,
    pub airgapped_mode: bool,
    pub created_at: String,
}

impl StandaloneAppManifest {
    pub fn new(
        app_id: impl Into<String>,
        app_name: impl Into<String>,
        target_platform: TargetPlatform,
        bundled_package_id: impl Into<String>,
    ) -> Self {
        Self {
            app_id: app_id.into(),
            app_name: app_name.into(),
            version: "1.0.0".to_string(),
            company_name: "Proteus Sovereign Systems".to_string(),
            copyright: format!("© {} Proteus Systems", Utc::now().format("%Y")),
            target_platform,
            window_config: WindowConfig::default(),
            bundled_package_id: bundled_package_id.into(),
            local_db_name: "app_store.db".to_string(),
            airgapped_mode: false,
            created_at: Utc::now().to_rfc3339(),
        }
    }

    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| format!("Serialization error: {}", e))
    }

    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| format!("Deserialization error: {}", e))
    }
}

/// Cryptographically sealed standalone runtime bundle containing manifest and .pr package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandaloneBundle {
    pub bundle_id: String,
    pub manifest: StandaloneAppManifest,
    pub package_bytes: Vec<u8>,
    pub package_sha256: String,
    pub created_at: String,
}

impl StandaloneBundle {
    pub fn new(manifest: StandaloneAppManifest, package_bytes: Vec<u8>) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(&package_bytes);
        let package_sha256 = format!("{:x}", hasher.finalize());

        Self {
            bundle_id: format!("SBL-{}", &package_sha256[..12]),
            manifest,
            package_bytes,
            package_sha256,
            created_at: Utc::now().to_rfc3339(),
        }
    }

    pub fn verify_integrity(&self) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(&self.package_bytes);
        let actual_hash = format!("{:x}", hasher.finalize());
        actual_hash == self.package_sha256
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        serde_json::to_vec(self).map_err(|e| format!("Bundle serialization error: {}", e))
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let bundle: Self = serde_json::from_slice(bytes)
            .map_err(|e| format!("Bundle deserialization error: {}", e))?;
        if !bundle.verify_integrity() {
            return Err("Bundle SHA-256 integrity check failed: Tampered package".to_string());
        }
        Ok(bundle)
    }
}

/// Persistent record of an extracted standalone application build.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandaloneBuildRecord {
    pub id: String,
    pub app_id: String,
    pub app_name: String,
    pub target_platform: String,
    pub version: String,
    pub package_sha256: String,
    pub output_path: String,
    pub created_at: String,
}

/// Initializes SQLite schema for standalone build tracking.
pub fn ensure_standalone_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS standalone_builds (
            id TEXT PRIMARY KEY,
            app_id TEXT NOT NULL,
            app_name TEXT NOT NULL,
            target_platform TEXT NOT NULL,
            version TEXT NOT NULL,
            package_sha256 TEXT NOT NULL,
            output_path TEXT NOT NULL,
            created_at TEXT NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_standalone_app_id ON standalone_builds(app_id);",
    )
}

/// Inserts a standalone build log into SQLite.
pub fn record_standalone_build(
    conn: &Connection,
    record: &StandaloneBuildRecord,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO standalone_builds (id, app_id, app_name, target_platform, version, package_sha256, output_path, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            record.id,
            record.app_id,
            record.app_name,
            record.target_platform,
            record.version,
            record.package_sha256,
            record.output_path,
            record.created_at
        ],
    )?;
    Ok(())
}

/// Lists all registered standalone builds ordered by date descending.
pub fn list_standalone_builds(conn: &Connection) -> rusqlite::Result<Vec<StandaloneBuildRecord>> {
    let mut stmt = conn.prepare(
        "SELECT id, app_id, app_name, target_platform, version, package_sha256, output_path, created_at
         FROM standalone_builds ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(StandaloneBuildRecord {
            id: row.get(0)?,
            app_id: row.get(1)?,
            app_name: row.get(2)?,
            target_platform: row.get(3)?,
            version: row.get(4)?,
            package_sha256: row.get(5)?,
            output_path: row.get(6)?,
            created_at: row.get(7)?,
        })
    })?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

/// Standalone Application Extractor Orchestrator.
pub struct StandaloneExtractor;

impl StandaloneExtractor {
    /// Generates the standard Cargo build command line arguments for the target platform.
    pub fn generate_cargo_build_command(target: TargetPlatform, release: bool) -> Vec<String> {
        let mut cmd = vec!["cargo".to_string(), "build".to_string()];
        if release {
            cmd.push("--release".to_string());
        }
        match target {
            TargetPlatform::WindowsExe | TargetPlatform::WindowsMsi => cmd.extend_from_slice(&["--target".into(), "x86_64-pc-windows-msvc".into()]),
            TargetPlatform::MacOsDmg => cmd.extend_from_slice(&["--target".into(), "aarch64-apple-darwin".into()]),
            TargetPlatform::LinuxAppImage => cmd.extend_from_slice(&["--target".into(), "x86_64-unknown-linux-gnu".into()]),
            TargetPlatform::AndroidApk => cmd.extend_from_slice(&["--target".into(), "aarch64-linux-android".into()]),
            TargetPlatform::IosAltStore => cmd.extend_from_slice(&["--target".into(), "aarch64-apple-ios".into()]),
        }
        cmd.extend_from_slice(&["--package".into(), "proteus-client".into()]);
        cmd
    }

    /// Bundles the manifest and package bytes into a standalone distribution artifact.
    pub fn extract_bundle(manifest: StandaloneAppManifest, package_bytes: Vec<u8>, output_dir: &Path) -> Result<(PathBuf, StandaloneBundle), String> {
        let bundle = StandaloneBundle::new(manifest.clone(), package_bytes);
        let bundle_bytes = bundle.to_bytes()?;

        let ext = manifest.target_platform.extension();
        let file_name = format!("{}-{}.{}", manifest.app_id, manifest.version, ext);
        let target_file = output_dir.join(file_name);

        std::fs::write(&target_file, &bundle_bytes)
            .map_err(|e| format!("Failed to write standalone bundle: {}", e))?;

        Ok((target_file, bundle))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_platform_properties() {
        assert_eq!(TargetPlatform::WindowsExe.extension(), "exe");
        assert_eq!(TargetPlatform::LinuxAppImage.extension(), "AppImage");
        assert_eq!(TargetPlatform::AndroidApk.extension(), "apk");
        assert!(!TargetPlatform::WindowsExe.is_mobile());
        assert!(TargetPlatform::AndroidApk.is_mobile());
        assert!(TargetPlatform::IosAltStore.is_mobile());
    }

    #[test]
    fn test_manifest_serialization_roundtrip() {
        let manifest = StandaloneAppManifest::new(
            "athens-repair-crm",
            "Athens Repair Shop",
            TargetPlatform::WindowsExe,
            "PKG-REPAIR-001",
        );
        let json = manifest.to_json().unwrap();
        let restored = StandaloneAppManifest::from_json(&json).unwrap();
        assert_eq!(restored.app_id, "athens-repair-crm");
        assert_eq!(restored.target_platform, TargetPlatform::WindowsExe);
        assert_eq!(restored.window_config.width, 1280.0);
    }

    #[test]
    fn test_standalone_bundle_integrity_and_bytes() {
        let manifest = StandaloneAppManifest::new(
            "thess-pos",
            "Thessaloniki POS",
            TargetPlatform::LinuxAppImage,
            "PKG-POS-002",
        );
        let fake_pkg = b"PRPK_SAMPLE_PACKAGE_CONTENT_V1".to_vec();
        let bundle = StandaloneBundle::new(manifest, fake_pkg);

        assert!(bundle.verify_integrity());
        let bytes = bundle.to_bytes().unwrap();

        let loaded = StandaloneBundle::from_bytes(&bytes).unwrap();
        assert_eq!(loaded.bundle_id, bundle.bundle_id);
        assert_eq!(loaded.package_sha256, bundle.package_sha256);
        assert_eq!(loaded.package_bytes, bundle.package_bytes);
    }

    #[test]
    fn test_standalone_bundle_tamper_rejection() {
        let manifest = StandaloneAppManifest::new(
            "patra-wms",
            "Patra WMS",
            TargetPlatform::WindowsExe,
            "PKG-WMS-003",
        );
        let fake_pkg = b"ORIGINAL_BYTES".to_vec();
        let mut bundle = StandaloneBundle::new(manifest, fake_pkg);
        bundle.package_bytes = b"TAMPERED_BYTES".to_vec();

        assert!(!bundle.verify_integrity());
        let bytes = serde_json::to_vec(&bundle).unwrap();
        assert!(StandaloneBundle::from_bytes(&bytes).is_err());
    }

    #[test]
    fn test_standalone_sqlite_schema_and_records() {
        let conn = Connection::open_in_memory().unwrap();
        ensure_standalone_schema(&conn).unwrap();

        let record = StandaloneBuildRecord {
            id: "BUILD-001".to_string(),
            app_id: "crete-crm".to_string(),
            app_name: "Crete Olive CRM".to_string(),
            target_platform: "WindowsExe".to_string(),
            version: "1.0.0".to_string(),
            package_sha256: "abc123def456".to_string(),
            output_path: "C:\\builds\\crete-crm.exe".to_string(),
            created_at: Utc::now().to_rfc3339(),
        };

        record_standalone_build(&conn, &record).unwrap();
        let list = list_standalone_builds(&conn).unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].app_id, "crete-crm");
        assert_eq!(list[0].target_platform, "WindowsExe");
    }

    #[test]
    fn test_cargo_build_command_generation() {
        let win_cmd = StandaloneExtractor::generate_cargo_build_command(TargetPlatform::WindowsExe, true);
        assert!(win_cmd.contains(&"--release".to_string()));
        assert!(win_cmd.contains(&"x86_64-pc-windows-msvc".to_string()));

        let android_cmd = StandaloneExtractor::generate_cargo_build_command(TargetPlatform::AndroidApk, false);
        assert!(!android_cmd.contains(&"--release".to_string()));
        assert!(android_cmd.contains(&"aarch64-linux-android".to_string()));
    }
}
