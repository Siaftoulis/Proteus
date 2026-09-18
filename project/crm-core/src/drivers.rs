// Proteus Core — Direct Database Drivers & Dialect Abstraction
// Designed from first principles. Zero copied third-party boilerplate.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DatabaseProtocol {
    Sqlite,
    Postgres,
    MySql,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SslMode {
    Disable,
    Prefer,
    Require,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub protocol: DatabaseProtocol,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub ssl_mode: SslMode,
    pub timeout_ms: u64,
}

impl ConnectionConfig {
    pub fn sqlite_memory() -> Self {
        Self {
            protocol: DatabaseProtocol::Sqlite,
            host: String::new(),
            port: 0,
            database: ":memory:".to_string(),
            username: None,
            password: None,
            ssl_mode: SslMode::Disable,
            timeout_ms: 5000,
        }
    }

    pub fn parse_url(raw_url: &str) -> Result<Self, String> {
        let trimmed = raw_url.trim();
        if trimmed.starts_with("sqlite://") {
            let path = trimmed.trim_start_matches("sqlite://");
            return Ok(Self {
                protocol: DatabaseProtocol::Sqlite,
                host: String::new(),
                port: 0,
                database: if path.is_empty() { ":memory:".to_string() } else { path.to_string() },
                username: None,
                password: None,
                ssl_mode: SslMode::Disable,
                timeout_ms: 5000,
            });
        }

        let (protocol, rest) = if let Some(s) = trimmed.strip_prefix("postgres://").or_else(|| trimmed.strip_prefix("postgresql://")) {
            (DatabaseProtocol::Postgres, s)
        } else if let Some(s) = trimmed.strip_prefix("mysql://") {
            (DatabaseProtocol::MySql, s)
        } else {
            return Err(format!("Unsupported database protocol scheme: {}", trimmed));
        };

        let (auth_part, host_db_part) = match rest.find('@') {
            Some(idx) => (Some(&rest[..idx]), &rest[idx + 1..]),
            None => (None, rest),
        };

        let (username, password) = match auth_part {
            Some(auth) => match auth.find(':') {
                Some(ci) => (Some(auth[..ci].to_string()), Some(auth[ci + 1..].to_string())),
                None => (Some(auth.to_string()), None),
            },
            None => (None, None),
        };

        let (host_port, db_query) = match host_db_part.find('/') {
            Some(idx) => (&host_db_part[..idx], &host_db_part[idx + 1..]),
            None => (host_db_part, ""),
        };

        let (host, port) = match host_port.find(':') {
            Some(ci) => (host_port[..ci].to_string(), host_port[ci + 1..].parse::<u16>().map_err(|e| e.to_string())?),
            None => (host_port.to_string(), match protocol {
                DatabaseProtocol::Postgres => 5432,
                DatabaseProtocol::MySql => 3306,
                DatabaseProtocol::Sqlite => 0,
            }),
        };

        let (database, ssl_mode) = match db_query.find('?') {
            Some(qi) => {
                let db = &db_query[..qi];
                let mut ssl = SslMode::Prefer;
                for pair in db_query[qi + 1..].split('&') {
                    let mut kv = pair.split('=');
                    if let (Some("sslmode"), Some(v)) = (kv.next(), kv.next()) {
                        ssl = match v.to_lowercase().as_str() {
                            "disable" => SslMode::Disable,
                            "require" => SslMode::Require,
                            _ => SslMode::Prefer,
                        };
                    }
                }
                (db.to_string(), ssl)
            }
            None => (db_query.to_string(), SslMode::Prefer),
        };

        Ok(Self { protocol, host, port, database, username, password, ssl_mode, timeout_ms: 5000 })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnMeta {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub is_primary_key: bool,
}

pub struct SqlDialect;

impl SqlDialect {
    pub fn placeholder(protocol: DatabaseProtocol, index: usize) -> String {
        match protocol {
            DatabaseProtocol::Postgres => format!("${}", index),
            DatabaseProtocol::Sqlite | DatabaseProtocol::MySql => "?".to_string(),
        }
    }

    pub fn quote_identifier(protocol: DatabaseProtocol, ident: &str) -> String {
        match protocol {
            DatabaseProtocol::Postgres | DatabaseProtocol::Sqlite => format!("\"{}\"", ident),
            DatabaseProtocol::MySql => format!("`{}`", ident),
        }
    }

    pub fn map_type(protocol: DatabaseProtocol, abstract_type: &str) -> &'static str {
        match (protocol, abstract_type.to_uppercase().as_str()) {
            (DatabaseProtocol::Postgres, "TEXT") => "TEXT",
            (DatabaseProtocol::Postgres, "INTEGER") => "BIGINT",
            (DatabaseProtocol::Postgres, "REAL") => "DOUBLE PRECISION",
            (DatabaseProtocol::Postgres, "JSON") => "JSONB",
            (DatabaseProtocol::MySql, "TEXT") => "LONGTEXT",
            (DatabaseProtocol::MySql, "INTEGER") => "BIGINT",
            (DatabaseProtocol::MySql, "REAL") => "DOUBLE",
            (DatabaseProtocol::MySql, "JSON") => "JSON",
            (DatabaseProtocol::Sqlite, "JSON") => "TEXT",
            (DatabaseProtocol::Sqlite, other) => match other {
                "INTEGER" => "INTEGER",
                "REAL" => "REAL",
                _ => "TEXT",
            },
            (_, _) => "TEXT",
        }
    }

    pub fn translate_placeholders(sql: &str, target: DatabaseProtocol) -> String {
        if target != DatabaseProtocol::Postgres {
            return sql.to_string();
        }
        let mut result = String::new();
        let (mut idx, mut in_quotes) = (1, false);
        for c in sql.chars() {
            match c {
                '\'' => { in_quotes = !in_quotes; result.push(c); }
                '?' if !in_quotes => { result.push_str(&format!("${}", idx)); idx += 1; }
                _ => result.push(c),
            }
        }
        result
    }
}

pub trait DatabaseDriver {
    fn protocol(&self) -> DatabaseProtocol;
    fn ping(&self) -> Result<bool, String>;
    fn execute_raw(&self, sql: &str) -> Result<u64, String>;
    fn inspect_tables(&self) -> Result<Vec<String>, String>;
    fn inspect_columns(&self, table: &str) -> Result<Vec<ColumnMeta>, String>;
}

pub struct SqliteDriver {
    conn: rusqlite::Connection,
}

impl SqliteDriver {
    pub fn new_memory() -> Result<Self, String> {
        Ok(Self { conn: rusqlite::Connection::open_in_memory().map_err(|e| e.to_string())? })
    }

    pub fn open(path: &str) -> Result<Self, String> {
        Ok(Self { conn: rusqlite::Connection::open(path).map_err(|e| e.to_string())? })
    }
}

impl DatabaseDriver for SqliteDriver {
    fn protocol(&self) -> DatabaseProtocol { DatabaseProtocol::Sqlite }

    fn ping(&self) -> Result<bool, String> {
        let mut stmt = self.conn.prepare("SELECT 1").map_err(|e| e.to_string())?;
        let res: i32 = stmt.query_row([], |r| r.get(0)).map_err(|e| e.to_string())?;
        Ok(res == 1)
    }

    fn execute_raw(&self, sql: &str) -> Result<u64, String> {
        self.conn.execute_batch(sql).map_err(|e| e.to_string())?;
        Ok(self.conn.changes())
    }

    fn inspect_tables(&self) -> Result<Vec<String>, String> {
        let mut stmt = self.conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name").map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |r| r.get::<_, String>(0)).map_err(|e| e.to_string())?;
        let mut tables = Vec::new();
        for r in rows { tables.push(r.map_err(|e| e.to_string())?); }
        Ok(tables)
    }

    fn inspect_columns(&self, table: &str) -> Result<Vec<ColumnMeta>, String> {
        let pragma = format!("PRAGMA table_info(\"{}\")", table.replace('"', ""));
        let mut stmt = self.conn.prepare(&pragma).map_err(|e| e.to_string())?;
        let rows = stmt.query_map([], |r| {
            Ok(ColumnMeta {
                name: r.get(1)?,
                data_type: r.get(2)?,
                is_nullable: r.get::<_, i32>(3)? == 0,
                is_primary_key: r.get::<_, i32>(5)? > 0,
            })
        }).map_err(|e| e.to_string())?;
        let mut cols = Vec::new();
        for r in rows { cols.push(r.map_err(|e| e.to_string())?); }
        Ok(cols)
    }
}

pub struct RemoteDriverMock {
    config: ConnectionConfig,
    tables: Vec<String>,
}

impl RemoteDriverMock {
    pub fn new(config: ConnectionConfig) -> Self {
        Self {
            config,
            tables: vec!["customers".into(), "invoices".into(), "inventory_items".into()],
        }
    }
}

impl DatabaseDriver for RemoteDriverMock {
    fn protocol(&self) -> DatabaseProtocol { self.config.protocol }

    fn ping(&self) -> Result<bool, String> {
        if self.config.host.is_empty() && self.config.protocol != DatabaseProtocol::Sqlite {
            return Err("Empty host in remote connection".into());
        }
        Ok(true)
    }

    fn execute_raw(&self, _sql: &str) -> Result<u64, String> { Ok(1) }

    fn inspect_tables(&self) -> Result<Vec<String>, String> { Ok(self.tables.clone()) }

    fn inspect_columns(&self, table: &str) -> Result<Vec<ColumnMeta>, String> {
        Ok(vec![
            ColumnMeta { name: "id".into(), data_type: "BIGINT".into(), is_nullable: false, is_primary_key: true },
            ColumnMeta { name: format!("{}_name", table), data_type: "VARCHAR(255)".into(), is_nullable: false, is_primary_key: false },
            ColumnMeta { name: "metadata".into(), data_type: "JSONB".into(), is_nullable: true, is_primary_key: false },
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sqlite_url() {
        let cfg = ConnectionConfig::parse_url("sqlite:///data/crm.db").unwrap();
        assert_eq!(cfg.protocol, DatabaseProtocol::Sqlite);
        assert_eq!(cfg.database, "/data/crm.db");
    }

    #[test]
    fn test_parse_postgres_url() {
        let cfg = ConnectionConfig::parse_url("postgres://admin:secret@pg.corp.net:5432/proteus_prod?sslmode=require").unwrap();
        assert_eq!(cfg.protocol, DatabaseProtocol::Postgres);
        assert_eq!(cfg.host, "pg.corp.net");
        assert_eq!(cfg.port, 5432);
        assert_eq!(cfg.database, "proteus_prod");
        assert_eq!(cfg.username.as_deref(), Some("admin"));
        assert_eq!(cfg.password.as_deref(), Some("secret"));
        assert_eq!(cfg.ssl_mode, SslMode::Require);
    }

    #[test]
    fn test_parse_mysql_url() {
        let cfg = ConnectionConfig::parse_url("mysql://root:toor@127.0.0.1:3306/erp_db").unwrap();
        assert_eq!(cfg.protocol, DatabaseProtocol::MySql);
        assert_eq!(cfg.host, "127.0.0.1");
        assert_eq!(cfg.port, 3306);
        assert_eq!(cfg.database, "erp_db");
        assert_eq!(cfg.username.as_deref(), Some("root"));
    }

    #[test]
    fn test_dialect_placeholders() {
        let sql = "SELECT * FROM tickets WHERE status = ? AND phone = ?";
        let pg_sql = SqlDialect::translate_placeholders(sql, DatabaseProtocol::Postgres);
        assert_eq!(pg_sql, "SELECT * FROM tickets WHERE status = $1 AND phone = $2");
    }

    #[test]
    fn test_sqlite_driver_lifecycle() {
        let driver = SqliteDriver::new_memory().unwrap();
        assert!(driver.ping().unwrap());

        driver.execute_raw("CREATE TABLE test_items (id TEXT PRIMARY KEY, value INTEGER NOT NULL)").unwrap();
        let tables = driver.inspect_tables().unwrap();
        assert_eq!(tables, vec!["test_items"]);

        let cols = driver.inspect_columns("test_items").unwrap();
        assert_eq!(cols.len(), 2);
        assert_eq!(cols[0].name, "id");
        assert!(cols[0].is_primary_key);
        assert_eq!(cols[1].name, "value");
        assert!(!cols[1].is_nullable);
    }

    #[test]
    fn test_remote_mock_driver() {
        let cfg = ConnectionConfig::parse_url("postgres://app:pass@cloud.db:5432/main").unwrap();
        let driver = RemoteDriverMock::new(cfg);
        assert!(driver.ping().unwrap());
        let tables = driver.inspect_tables().unwrap();
        assert!(tables.contains(&"customers".to_string()));
    }
}
