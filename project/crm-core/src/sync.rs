use crate::{Database, DbError, Project};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::thread;

const MAX_CONCURRENT_CONNECTIONS: u32 = 8;

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncPayload {
    pub projects: Vec<Project>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncResponse {
    pub ok: bool,
    pub projects: Vec<Project>,
}

pub fn merge_projects(db: &Database, incoming: &[Project]) -> Result<usize, DbError> {
    let mut count = 0;
    for proj in incoming {
        let should_update = match db.load_project(&proj.id) {
            Ok(existing) => proj.updated_at > existing.updated_at,
            Err(_) => true,
        };
        if should_update {
            db.upsert_project(proj)?;
            count += 1;
        }
    }
    Ok(count)
}

pub fn start_sync_server(db: Arc<Database>, port: u16) -> Result<(), String> {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
        .map_err(|e| format!("Failed to bind sync server: {}", e))?;
    listener
        .set_nonblocking(true)
        .map_err(|e| format!("Failed to set nonblocking: {}", e))?;

    let conn_count = Arc::new(AtomicU32::new(0));

    // ponytail: global thread limit per sync server, per-account limits if needed
    thread::spawn(move || {
        loop {
        match listener.accept() {
                Ok((stream, _)) => {
                    if conn_count.load(Ordering::Relaxed) >= MAX_CONCURRENT_CONNECTIONS {
                        let _ = stream.shutdown(std::net::Shutdown::Both);
                        continue;
                    }
                    conn_count.fetch_add(1, Ordering::Relaxed);
                    let db = db.clone();
                    let cc = conn_count.clone();
                    thread::spawn(move || {
                        handle_connection(stream, &db);
                        cc.fetch_sub(1, Ordering::Relaxed);
                    });
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(std::time::Duration::from_millis(50));
                }
                Err(_) => break,
            }
        }
    });

    Ok(())
}

fn handle_connection(mut stream: TcpStream, db: &Database) {
    // ponytail: read up to 64KB initially, loop for larger Content-Length
    let mut buf = Vec::new();
    let mut tmp = [0u8; 65536];
    let n = match stream.read(&mut tmp) {
        Ok(n) if n > 0 => n,
        _ => return,
    };
    buf.extend_from_slice(&tmp[..n]);

    // Parse Content-Length from raw bytes (no UTF-8 lossy)
    let header_end = buf.windows(4).position(|w| w == b"\r\n\r\n").map(|p| p + 4).unwrap_or(n);
    let header_str = String::from_utf8_lossy(&buf[..header_end]);
    let content_length = header_str
        .lines()
        .find_map(|l| l.strip_prefix("Content-Length:").or_else(|| l.strip_prefix("content-length:")))
        .and_then(|v| v.trim().parse::<usize>().ok());

    // Read full body if Content-Length exceeds what we have
    if let Some(expected) = content_length {
        let body_have = n.saturating_sub(header_end);
        if expected > body_have {
            let mut remaining = vec![0u8; expected - body_have];
            let mut read_total = 0;
            while read_total < remaining.len() {
                match stream.read(&mut remaining[read_total..]) {
                    Ok(0) => break,
                    Ok(m) => read_total += m,
                    Err(_) => break,
                }
            }
            buf.extend_from_slice(&remaining[..read_total]);
        }
    }

    // Parse request line from raw bytes
    let request_str = String::from_utf8_lossy(&buf);
    let mut lines = request_str.lines();
    let request_line = match lines.next() {
        Some(l) => l,
        None => return,
    };
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 3 {
        return;
    }
    let method = parts[0];
    let path = parts[1];

    if path != "/sync" {
        send_response(&mut stream, 404, "Not Found", b"");
        return;
    }

    match method {
        "GET" => {
            let projects = match db.list_projects() {
                Ok(p) => p,
                Err(_) => {
                    send_response(&mut stream, 500, "Server Error", b"");
                    return;
                }
            };
            let payload = SyncResponse { ok: true, projects };
            let body = serde_json::to_vec(&payload).unwrap_or_default();
            send_response(&mut stream, 200, "OK", &body);
        }
        "PUT" => {
            // Parse body from raw bytes to preserve UTF-8 validity
            let body_bytes = if header_end < buf.len() { &buf[header_end..] } else { &[] };
            let payload: SyncPayload = match serde_json::from_slice(body_bytes) {
                Ok(p) => p,
                Err(_) => {
                    send_response(&mut stream, 400, "Bad Request", b"");
                    return;
                }
            };
            let _ = merge_projects(db, &payload.projects);
            let projects = match db.list_projects() {
                Ok(p) => p,
                Err(_) => {
                    send_response(&mut stream, 500, "Server Error", b"");
                    return;
                }
            };
            let response = SyncResponse { ok: true, projects };
            let resp_body = serde_json::to_vec(&response).unwrap_or_default();
            send_response(&mut stream, 200, "OK", &resp_body);
        }
        _ => {
            send_response(&mut stream, 405, "Method Not Allowed", b"");
        }
    }
}

fn send_response(stream: &mut TcpStream, status: u16, reason: &str, body: &[u8]) {
    let response = format!(
        "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n",
        status, reason, body.len()
    );
    let _ = stream.write_all(response.as_bytes());
    if !body.is_empty() {
        let _ = stream.write_all(body);
    }
    let _ = stream.flush();
}

pub fn sync_pull(url: &str) -> Result<Vec<Project>, String> {
    let response = ureq::get(&format!("{}/sync", url))
        .call()
        .map_err(|e| format!("Sync pull failed: {}", e))?;
    let payload: SyncResponse = response
        .into_json()
        .map_err(|e| format!("Invalid sync response: {}", e))?;
    if payload.ok {
        Ok(payload.projects)
    } else {
        Err("Sync response not OK".to_string())
    }
}

pub fn sync_push(url: &str, projects: &[Project]) -> Result<Vec<Project>, String> {
    let payload = SyncPayload {
        projects: projects.to_vec(),
    };
    let response = ureq::put(&format!("{}/sync", url))
        .send_json(&payload)
        .map_err(|e| format!("Sync push failed: {}", e))?;
    let result: SyncResponse = response
        .into_json()
        .map_err(|e| format!("Invalid sync response: {}", e))?;
    if result.ok {
        Ok(result.projects)
    } else {
        Err("Sync response not OK".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Database;

    fn test_db() -> Database {
        Database::new(":memory:").expect("Failed to create in-memory DB")
    }

    #[test]
    fn test_merge_projects_empty() {
        let db = test_db();
        let count = merge_projects(&db, &[]).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_merge_new_project() {
        let db = test_db();
        let proj = db.create_project("From Peer").unwrap();
        let db2 = test_db();
        let count = merge_projects(&db2, &[proj]).unwrap();
        assert_eq!(count, 1);
        assert_eq!(db2.list_projects().unwrap().len(), 1);
    }

    #[test]
    fn test_merge_newer_wins() {
        let db = test_db();
        let original = db.create_project("Conflict").unwrap();

        let incoming = Project {
            updated_at: "9999-12-31T23:59:59Z".to_string(), // future timestamp
            ..original.clone()
        };
        let count = merge_projects(&db, &[incoming]).unwrap();
        assert_eq!(count, 1);
        let loaded = db.load_project(&original.id).unwrap();
        assert_eq!(loaded.updated_at, "9999-12-31T23:59:59Z");
    }

    #[test]
    fn test_merge_older_skipped() {
        let db = test_db();
        let original = db.create_project("Older Skip").unwrap();

        let incoming = Project {
            updated_at: "2000-01-01T00:00:00Z".to_string(), // past timestamp
            ..original.clone()
        };
        let count = merge_projects(&db, &[incoming]).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_merge_equal_timestamp_skipped() {
        let db = test_db();
        let original = db.create_project("Equal TS").unwrap();
        let ts = original.updated_at.clone();

        let incoming = Project {
            updated_at: ts.clone(),
            name: "Would Overwrite".to_string(),
            ..original.clone()
        };
        // Same timestamp: incoming is NOT >, so merge should skip
        let count = merge_projects(&db, &[incoming]).unwrap();
        assert_eq!(count, 0);
        let loaded = db.load_project(&original.id).unwrap();
        assert_eq!(loaded.name, "Equal TS");
    }

    #[test]
    fn test_sync_payload_roundtrip() {
        let payload = SyncPayload {
            projects: vec![Project {
                id: "1".into(),
                name: "Test".into(),
                widgets: "[]".into(),
                flows: "[]".into(),
                created_at: "now".into(),
                updated_at: "now".into(),
            }],
        };
        let json = serde_json::to_vec(&payload).unwrap();
        let restored: SyncPayload = serde_json::from_slice(&json).unwrap();
        assert_eq!(restored.projects.len(), 1);
        assert_eq!(restored.projects[0].name, "Test");
    }
}
