use rusqlite::{params, Connection};
use std::sync::Mutex;
use tracing::instrument;

use crate::models::{CreateUserRequest, User, VersionInfo};

pub struct AppDb {
    pub conn: Mutex<Connection>,
}

impl AppDb {
    pub fn new(path: &str) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|e| e.to_string())?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                email TEXT NOT NULL UNIQUE,
                password_hash TEXT,
                name TEXT NOT NULL DEFAULT '',
                avatar_url TEXT NOT NULL DEFAULT '',
                provider TEXT NOT NULL DEFAULT 'email',
                provider_id TEXT NOT NULL DEFAULT '',
                role TEXT NOT NULL DEFAULT 'editor',
                created_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS refresh_tokens (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                token TEXT NOT NULL UNIQUE,
                expires_at TEXT NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users(id)
            );
            CREATE TABLE IF NOT EXISTS user_licenses (
                user_id TEXT PRIMARY KEY,
                license_key TEXT NOT NULL,
                FOREIGN KEY (user_id) REFERENCES users(id)
            );
            CREATE TABLE IF NOT EXISTS app_versions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                version TEXT NOT NULL UNIQUE,
                url TEXT NOT NULL,
                checksum TEXT NOT NULL,
                released_at TEXT NOT NULL
            );
            ",
        )
        .map_err(|e| e.to_string())?;
        Ok(AppDb {
            conn: Mutex::new(conn),
        })
    }

    #[instrument(skip(self))]
    pub fn create_user(&self, req: &CreateUserRequest) -> Result<User, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        let role = req.role.as_deref().unwrap_or("editor");
        conn.execute(
            "INSERT INTO users (id, email, password_hash, name, avatar_url, provider, provider_id, role, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                id,
                req.email,
                req.password,
                req.name,
                req.avatar_url.as_deref().unwrap_or(""),
                req.provider,
                req.provider_id,
                role,
                now,
            ],
        )
        .map_err(|e| e.to_string())?;
        Ok(User {
            id,
            email: req.email.clone(),
            name: req.name.clone(),
            avatar_url: req.avatar_url.clone().unwrap_or_default(),
            provider: req.provider.clone(),
            role: role.to_string(),
            created_at: now,
        })
    }

    #[instrument(skip(self))]
    pub fn get_user_by_email(&self, email: &str) -> Result<Option<(User, Option<String>)>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT id, email, password_hash, name, avatar_url, provider, provider_id, role, created_at
                 FROM users WHERE email = ?1",
            )
            .map_err(|e| e.to_string())?;
        let result = stmt
            .query_row(params![email], |row| {
                Ok((
                    User {
                        id: row.get(0)?,
                        email: row.get(1)?,
                        name: row.get(3)?,
                        avatar_url: row.get(4)?,
                        provider: row.get(5)?,
                        role: row.get(7)?,
                        created_at: row.get(8)?,
                    },
                    row.get::<_, Option<String>>(2)?,
                ))
            })
            .ok();
        Ok(result)
    }

    #[instrument(skip(self))]
    pub fn get_user_by_id(&self, id: &str) -> Result<Option<User>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT id, email, name, avatar_url, provider, provider_id, role, created_at
                 FROM users WHERE id = ?1",
            )
            .map_err(|e| e.to_string())?;
        let result = stmt
            .query_row(params![id], |row| {
                Ok(User {
                    id: row.get(0)?,
                    email: row.get(1)?,
                    name: row.get(2)?,
                    avatar_url: row.get(3)?,
                    provider: row.get(4)?,
                    role: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })
            .ok();
        Ok(result)
    }

    #[instrument(skip(self))]
    pub fn store_refresh_token(&self, user_id: &str, token: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let id = uuid::Uuid::new_v4().to_string();
        let expires = (chrono::Utc::now() + chrono::Duration::days(30)).to_rfc3339();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO refresh_tokens (id, user_id, token, expires_at, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, user_id, token, expires, now],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn validate_refresh_token(&self, token: &str) -> Result<Option<String>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT user_id, expires_at FROM refresh_tokens WHERE token = ?1",
            )
            .map_err(|e| e.to_string())?;
        let result = stmt
            .query_row(params![token], |row| {
                let user_id: String = row.get(0)?;
                let expires_at: String = row.get(1)?;
                Ok((user_id, expires_at))
            })
            .ok();
        match result {
            Some((user_id, expires_at)) => {
                let expires = chrono::DateTime::parse_from_rfc3339(&expires_at)
                    .map_err(|e| e.to_string())?;
                if expires < chrono::Utc::now() {
                    conn.execute("DELETE FROM refresh_tokens WHERE token = ?1", params![token])
                        .map_err(|e| e.to_string())?;
                    return Ok(None);
                }
                Ok(Some(user_id))
            }
            None => Ok(None),
        }
    }

    #[instrument(skip(self))]
    pub fn delete_refresh_token(&self, token: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute("DELETE FROM refresh_tokens WHERE token = ?1", params![token])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn get_license_key(&self, user_id: &str) -> Result<Option<String>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT license_key FROM user_licenses WHERE user_id = ?1")
            .map_err(|e| e.to_string())?;
        let result = stmt.query_row(params![user_id], |row| row.get::<_, String>(0)).ok();
        Ok(result)
    }

    #[instrument(skip(self))]
    pub fn set_license_key(&self, user_id: &str, license_key: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT OR REPLACE INTO user_licenses (user_id, license_key) VALUES (?1, ?2)",
            params![user_id, license_key],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    #[instrument(skip(self))]
    pub fn get_user_by_provider(&self, provider: &str, provider_id: &str) -> Result<Option<User>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT id, email, name, avatar_url, provider, provider_id, role, created_at
                 FROM users WHERE provider = ?1 AND provider_id = ?2",
            )
            .map_err(|e| e.to_string())?;
        let result = stmt
            .query_row(params![provider, provider_id], |row| {
                Ok(User {
                    id: row.get(0)?,
                    email: row.get(1)?,
                    name: row.get(2)?,
                    avatar_url: row.get(3)?,
                    provider: row.get(4)?,
                    role: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })
            .ok();
        Ok(result)
    }

    #[instrument(skip(self))]
    pub fn get_latest_version(&self) -> Result<Option<VersionInfo>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare(
                "SELECT version, url, checksum, released_at FROM app_versions
                 ORDER BY id DESC LIMIT 1",
            )
            .map_err(|e| e.to_string())?;
        let result = stmt
            .query_row([], |row| {
                Ok(VersionInfo {
                    version: row.get(0)?,
                    url: row.get(1)?,
                    checksum: row.get(2)?,
                    released_at: row.get(3)?,
                })
            })
            .ok();
        Ok(result)
    }
}
