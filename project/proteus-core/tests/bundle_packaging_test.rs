use proteus_core::paths::get_database_path;
use std::fs;
use std::path::Path;

#[test]
fn test_bundle_packaging_script_and_artifacts() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let project_root = Path::new(manifest_dir).parent().expect("Failed to get project root");
    let installer_dir = project_root.join("deploy").join("installer");

    // 1. Verify bundle.ps1 exists and has correct directives
    let bundle_script = installer_dir.join("bundle.ps1");
    assert!(bundle_script.exists(), "bundle.ps1 must exist in deploy/installer");
    let script_content = fs::read_to_string(&bundle_script).expect("Failed to read bundle.ps1");
    assert!(script_content.contains("proteus-client.exe"));
    assert!(script_content.contains("proteus-design-studio.exe"));
    assert!(script_content.contains("Run-Proteus-POS.bat"));
    assert!(script_content.contains("Run-Proteus-Studio.bat"));
    assert!(script_content.contains("SHA256SUMS.txt"));

    // 2. Verify proteus.default.toml exists and has all required sections
    let config_file = installer_dir.join("proteus.default.toml");
    assert!(config_file.exists(), "proteus.default.toml must exist");
    let config_content = fs::read_to_string(&config_file).expect("Failed to read proteus.default.toml");
    assert!(config_content.contains("[store]"));
    assert!(config_content.contains("[database]"));
    assert!(config_content.contains("[mydata]"));
    assert!(config_content.contains("[pos]"));
    assert!(config_content.contains("[hardware]"));
    assert!(config_content.contains("[mesh]"));
    assert!(config_content.contains("tin = \"000000000\""));
    assert!(config_content.contains("wal_mode = true"));

    // 3. Test portable path detection with PROTEUS_DATA_DIR environment variable
    let test_dir = project_root.join("target").join("test_portable_dir");
    std::env::set_var("PROTEUS_DATA_DIR", &test_dir);
    let resolved_db = get_database_path();
    assert_eq!(resolved_db, test_dir.join("store.db"));
    std::env::remove_var("PROTEUS_DATA_DIR");

    // 4. Test portable mode with PROTEUS_PORTABLE environment variable
    std::env::set_var("PROTEUS_PORTABLE", "1");
    let resolved_portable = get_database_path();
    assert_eq!(resolved_portable, Path::new("data").join("store.db"));
    std::env::remove_var("PROTEUS_PORTABLE");
}
