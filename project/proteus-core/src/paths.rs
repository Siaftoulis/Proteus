//! Standard system paths for Proteus BOS data storage.
//! Ensures zero-privilege execution (no Program Files writes).

use std::path::PathBuf;

/// Returns the platform-standard database file path.
///
/// - Windows: `%APPDATA%\Proteus\data\store.db`
/// - Linux: `~/.local/share/proteus/store.db`
/// - macOS: `~/Library/Application Support/Proteus/store.db`
pub fn get_database_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            let mut path = PathBuf::from(appdata);
            path.push("Proteus");
            path.push("data");
            path.push("store.db");
            return path;
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(home) = std::env::var("HOME") {
            let mut path = PathBuf::from(home);
            path.push(".local");
            path.push("share");
            path.push("proteus");
            path.push("store.db");
            return path;
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = std::env::var("HOME") {
            let mut path = PathBuf::from(home);
            path.push("Library");
            path.push("Application Support");
            path.push("Proteus");
            path.push("store.db");
            return path;
        }
    }

    // Fallback to local data folder if environment variable is missing
    PathBuf::from("data").join("store.db")
}

/// Ensures that the parent directory for the database exists.
pub fn ensure_database_dir_exists(db_path: &std::path::Path) -> std::io::Result<()> {
    if let Some(parent) = db_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

/// Returns the platform-standard backups directory path.
///
/// - Windows: `%APPDATA%\Proteus\backups`
/// - Linux: `~/.local/share/proteus/backups`
/// - macOS: `~/Library/Application Support/Proteus/backups`
pub fn get_backup_dir() -> PathBuf {
    let db_path = get_database_path();
    if let Some(parent) = db_path.parent() {
        if let Some(grandparent) = parent.parent() {
            return grandparent.join("backups");
        }
    }
    PathBuf::from("backups")
}

/// Returns the platform-standard packages export/import directory path.
///
/// - Windows: `%APPDATA%\Proteus\packages`
/// - Linux: `~/.local/share/proteus/packages`
/// - macOS: `~/Library/Application Support/Proteus/packages`
pub fn get_packages_dir() -> PathBuf {
    let db_path = get_database_path();
    if let Some(parent) = db_path.parent() {
        if let Some(grandparent) = parent.parent() {
            return grandparent.join("packages");
        }
    }
    PathBuf::from("packages")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_path_ends_with_store_db() {
        let path = get_database_path();
        assert!(path.ends_with("store.db"));
    }

    #[test]
    fn test_ensure_dir_creates_parent() {
        let temp_dir = std::env::temp_dir().join("proteus_test_path_dir");
        let test_db = temp_dir.join("sub").join("store.db");
        assert!(ensure_database_dir_exists(&test_db).is_ok());
        assert!(test_db.parent().unwrap().exists());
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_packages_dir() {
        let path = get_packages_dir();
        assert!(path.ends_with("packages"));
    }
}
