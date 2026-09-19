// Proteus Client — LAN Package Receiver Daemon (port 7443)
// Listens for 1-Click "Deploy to Store" packages from Proteus Designer.
// 100% original code, zero third-party web frameworks.

use crm_core::package::{MountSummary, PrPackage};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[allow(dead_code)]
pub struct LanPackageReceiver {
    rx: Receiver<PrPackage>,
    is_running: Arc<AtomicBool>,
    port: u16,
}

impl LanPackageReceiver {
    pub fn start(port: u16) -> Result<Self, String> {
        let (tx, rx) = channel();
        let is_running = Arc::new(AtomicBool::new(true));
        let running_clone = is_running.clone();

        let listener = TcpListener::bind(format!("0.0.0.0:{}", port))
            .or_else(|_| TcpListener::bind(format!("127.0.0.1:{}", port)))
            .map_err(|e| format!("Failed to bind LAN receiver on port {}: {}", port, e))?;

        listener.set_nonblocking(true)
            .map_err(|e| format!("Failed to set non-blocking: {}", e))?;

        thread::spawn(move || {
            while running_clone.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        handle_connection(stream, &tx);
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(50));
                    }
                    Err(_) => {
                        thread::sleep(Duration::from_millis(100));
                    }
                }
            }
        });

        Ok(Self { rx, is_running, port })
    }

    pub fn try_recv(&self) -> Option<PrPackage> {
        self.rx.try_recv().ok()
    }

    #[allow(dead_code)]
    pub fn port(&self) -> u16 {
        self.port
    }

    #[allow(dead_code)]
    pub fn stop(&self) {
        self.is_running.store(false, Ordering::Relaxed);
    }
}

fn handle_connection(mut stream: TcpStream, tx: &Sender<PrPackage>) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(3)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(3)));

    let mut header_buf = Vec::new();
    let mut byte = [0u8; 1];
    let mut content_length = 0;

    // Read HTTP headers until \r\n\r\n
    while let Ok(1) = stream.read(&mut byte) {
        header_buf.push(byte[0]);
        if header_buf.ends_with(b"\r\n\r\n") {
            break;
        }
        if header_buf.len() > 8192 {
            send_response(&mut stream, 400, "Headers too large");
            return;
        }
    }

    let headers_str = String::from_utf8_lossy(&header_buf);
    for line in headers_str.lines() {
        let lower = line.to_lowercase();
        if lower.starts_with("content-length:") {
            if let Some(val) = line.split(':').nth(1) {
                content_length = val.trim().parse::<usize>().unwrap_or(0);
            }
        }
    }

    if content_length == 0 || content_length > 50 * 1024 * 1024 {
        send_response(&mut stream, 400, "Invalid Content-Length");
        return;
    }

    // Read binary body payload
    let mut body = vec![0u8; content_length];
    if stream.read_exact(&mut body).is_err() {
        send_response(&mut stream, 400, "Failed to read body bytes");
        return;
    }

    // Validate package integrity & SHA-256 seal
    match PrPackage::from_bytes(&body) {
        Ok(package) => {
            let summary = MountSummary {
                bundle_id: package.manifest.bundle_id.clone(),
                package_name: package.manifest.name.clone(),
                applied_ddl_count: package.schema.ddl_statements.len(),
                loaded_views_count: package.views.len(),
                bound_flows_count: package.flows.len(),
                snapshot_created: Some("LIVE-HOT-APPLIED".into()),
            };
            let _ = tx.send(package);
            let resp_json = serde_json::to_string(&summary).unwrap_or_else(|_| "{}".into());
            send_json_response(&mut stream, 200, &resp_json);
        }
        Err(e) => {
            send_response(&mut stream, 400, &format!("Corrupted .pr package: {}", e));
        }
    }
}

fn send_response(stream: &mut TcpStream, code: u16, msg: &str) {
    let resp = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        code, if code == 200 { "OK" } else { "Bad Request" }, msg.len(), msg
    );
    let _ = stream.write_all(resp.as_bytes());
}

fn send_json_response(stream: &mut TcpStream, code: u16, json: &str) {
    let resp = format!(
        "HTTP/1.1 {} OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        code, json.len(), json
    );
    let _ = stream.write_all(resp.as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lan_receiver_startup_and_shutdown() {
        let receiver = LanPackageReceiver::start(7499);
        assert!(receiver.is_ok());
        let rec = receiver.unwrap();
        assert_eq!(rec.port(), 7499);
        rec.stop();
    }

    #[test]
    fn test_pcda_to_pcds_hot_deploy_e2e() {
        use crm_core::inference::{InferredColumn, InferredTable, InferredType};
        use crm_core::package::PrPackage;

        // 1. PCDA infers a table from raw business data
        let table = InferredTable {
            table_name: "parts_inventory".into(),
            columns: vec![
                InferredColumn {
                    name: "part_sku".into(),
                    col_type: InferredType::Text,
                    is_nullable: false,
                    is_primary_key: true,
                },
                InferredColumn {
                    name: "stock_quantity".into(),
                    col_type: InferredType::Integer,
                    is_nullable: false,
                    is_primary_key: false,
                },
            ],
            primary_key: Some("part_sku".into()),
            sample_rows_count: 2,
        };

        // 2. PCDA exports table to .pr package
        let package = PrPackage::from_inferred_table(&table, "PCDA Analyst");
        assert_eq!(package.manifest.bundle_id, "PKG-PARTS_INVENTORY");
        assert_eq!(package.schema.ddl_statements.len(), 1);

        // 3. Start local LAN receiver on a dedicated test port
        let test_port = 17445;
        let receiver = LanPackageReceiver::start(test_port).expect("Failed to start receiver");

        // 4. PCDS deploys package to target terminal endpoint
        let target_url = format!("http://127.0.0.1:{}", test_port);
        let summary = package.deploy_to_client(&target_url).expect("Failed to deploy package to client");

        assert_eq!(summary.bundle_id, "PKG-PARTS_INVENTORY");
        assert_eq!(summary.applied_ddl_count, 1);

        // 5. Receiver receives and unpacks package
        let received_pkg = receiver.try_recv().expect("Receiver must have received package");
        assert_eq!(received_pkg.manifest.bundle_id, "PKG-PARTS_INVENTORY");
        assert_eq!(received_pkg.manifest.name, "Dataset: parts_inventory");

        // 6. Simulate terminal mounting into SQLite DB
        let mut conn = rusqlite::Connection::open_in_memory().unwrap();
        let mount_res = received_pkg.mount(&mut conn, None, "LAN-Deployer");
        assert!(mount_res.is_ok(), "Package must mount cleanly into SQLite");

        // 7. Verify table created in SQLite
        let mut stmt = conn.prepare("SELECT count(*) FROM sqlite_master WHERE type='table' AND name='parts_inventory'").unwrap();
        let count: i64 = stmt.query_row([], |r| r.get(0)).unwrap();
        assert_eq!(count, 1);

        receiver.stop();
    }
}
