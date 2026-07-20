use crate::encryption;
use crate::Database;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportData {
    pub version: u32,
    pub created_at: String,
    pub projects: Vec<crate::Project>,
}

impl Database {
    pub fn export_all(&self, password: &str, salt: &str) -> Result<Vec<u8>, String> {
        let projects = self.list_projects().map_err(|e| e.to_string())?;
        let data = ExportData {
            version: 1,
            created_at: chrono::Utc::now().to_rfc3339(),
            projects,
        };
        let json = serde_json::to_vec(&data).map_err(|e| e.to_string())?;
        let key = encryption::derive_key(password, salt)?;
        encryption::encrypt(&json, &key)
    }

    pub fn import_all(&self, encrypted_data: &[u8], password: &str, salt: &str) -> Result<usize, String> {
        let key = encryption::derive_key(password, salt)?;
        let json = encryption::decrypt(encrypted_data, &key)?;
        let data: ExportData = serde_json::from_slice(&json).map_err(|e| e.to_string())?;
        if data.version != 1 {
            return Err(format!("Unsupported export version: {}", data.version));
        }
        let mut count = 0;
        for project in &data.projects {
            self.upsert_project(project).map_err(|e| e.to_string())?;
            count += 1;
        }
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use crate::Database;

    fn test_db() -> Database {
        Database::new(":memory:").expect("Failed to create in-memory DB")
    }

    #[test]
    fn test_export_import_roundtrip() {
        let db = test_db();
        db.create_project("Project A").unwrap();
        db.create_project("Project B").unwrap();

        let exported = db.export_all("password123", "device-salt").unwrap();
        assert!(!exported.is_empty());

        let db2 = test_db();
        let count = db2.import_all(&exported, "password123", "device-salt").unwrap();
        assert_eq!(count, 2);

        let projects = db2.list_projects().unwrap();
        assert_eq!(projects.len(), 2);
    }

    #[test]
    fn test_import_wrong_password_fails() {
        let db = test_db();
        db.create_project("Secret").unwrap();
        let exported = db.export_all("correct", "salt").unwrap();
        let db2 = test_db();
        let result = db2.import_all(&exported, "wrong", "salt");
        assert!(result.is_err());
    }

    #[test]
    fn test_export_empty_db() {
        let db = test_db();
        let exported = db.export_all("pwd", "dev").unwrap();
        let db2 = test_db();
        let count = db2.import_all(&exported, "pwd", "dev").unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_import_wrong_version_fails() {
        let data = crate::export::ExportData {
            version: 999,
            created_at: "now".to_string(),
            projects: vec![],
        };
        let json = serde_json::to_vec(&data).unwrap();
        let key = crate::encryption::derive_key("pwd", "salt").unwrap();
        let encrypted = crate::encryption::encrypt(&json, &key).unwrap();
        let db = test_db();
        let result = db.import_all(&encrypted, "pwd", "salt");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("999"));
    }

    #[test]
    fn test_import_corrupted_data_fails() {
        let db = test_db();
        let garbage = vec![0xde, 0xad, 0xbe, 0xef];
        let result = db.import_all(&garbage, "pwd", "salt");
        assert!(result.is_err());
    }
}
