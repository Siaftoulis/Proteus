//! Background Replication Daemon for Proteus Client.
//! Periodically polls and drains the prioritized Outbox queue (Critical -> High -> Normal -> Low)
//! using an independent SQLite read connection compatible with WAL mode.

use proteus_core::replication::worker::{OutboxWorker, RemoteTransport};
use proteus_core::replication::OutboxRecord;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub struct DefaultRemoteTransport {
    pub remote_url: Option<String>,
}

impl Default for DefaultRemoteTransport {
    fn default() -> Self {
        Self {
            remote_url: std::env::var("PROTEUS_REPLICATION_URL").ok(),
        }
    }
}

impl RemoteTransport for DefaultRemoteTransport {
    fn send(&self, record: &OutboxRecord) -> Result<(), String> {
        if let Some(ref url) = self.remote_url {
            let sync_url = if url.ends_with("/api/sync/outbox") {
                url.clone()
            } else {
                format!("{}/api/sync/outbox", url.trim_end_matches('/'))
            };
            let body = serde_json::to_string(&vec![record.clone()])
                .map_err(|e| format!("Serialization error: {}", e))?;
            let resp = ureq::post(&sync_url)
                .set("Content-Type", "application/json")
                .send_string(&body)
                .map_err(|e| format!("HTTP post failed: {}", e))?;
            if resp.status() >= 200 && resp.status() < 300 {
                Ok(())
            } else {
                Err(format!("Server returned HTTP {}", resp.status()))
            }
        } else {
            // Standby / Local loopback mode: records are preserved in outbox
            // until a remote Hub endpoint is configured.
            Ok(())
        }
    }

    fn ping(&self) -> Result<(), String> {
        if let Some(ref url) = self.remote_url {
            let ping_url = format!("{}/health", url.trim_end_matches('/'));
            let resp = ureq::get(&ping_url)
                .call()
                .map_err(|e| format!("Ping failed: {}", e))?;
            if resp.status() == 200 {
                Ok(())
            } else {
                Err(format!("Ping returned HTTP {}", resp.status()))
            }
        } else {
            Ok(())
        }
    }
}

pub struct ReplicationDaemon {
    is_running: Arc<AtomicBool>,
    pushed_total: Arc<AtomicUsize>,
}

impl ReplicationDaemon {
    pub fn start(db_path: PathBuf, poll_interval_ms: u64) -> Result<Self, String> {
        let is_running = Arc::new(AtomicBool::new(true));
        let pushed_total = Arc::new(AtomicUsize::new(0));

        let running_clone = is_running.clone();
        let pushed_clone = pushed_total.clone();

        thread::Builder::new()
            .name("proteus-replication-daemon".into())
            .spawn(move || {
                let conn = match proteus_core::paths::open_tuned_connection(&db_path) {
                    Ok(c) => c,
                    Err(_) => return,
                };

                let mut worker = OutboxWorker::default();
                let transport = DefaultRemoteTransport::default();

                while running_clone.load(Ordering::Relaxed) {
                    if let Ok(count) = worker.step(&conn, &transport) {
                        if count > 0 {
                            pushed_clone.fetch_add(count, Ordering::Relaxed);
                        }
                    }
                    thread::sleep(Duration::from_millis(poll_interval_ms));
                }
            })
            .map_err(|e| format!("Failed to spawn replication daemon: {}", e))?;

        Ok(Self {
            is_running,
            pushed_total,
        })
    }

    pub fn total_pushed(&self) -> usize {
        self.pushed_total.load(Ordering::Relaxed)
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::Relaxed);
    }
}

impl Drop for ReplicationDaemon {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replication_daemon_lifecycle() {
        let temp_dir = std::env::temp_dir().join(format!("proteus_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let _ = std::fs::create_dir_all(&temp_dir);
        let db_path = temp_dir.join("test_rep.db");

        {
            let conn = proteus_core::paths::open_tuned_connection(&db_path).unwrap();
            proteus_core::replication::init_outbox_schema(&conn).unwrap();
            proteus_core::replication::enqueue_outbox(&conn, "test_ent", "REC-1", proteus_core::replication::ChangeOp::Insert, "{}").unwrap();
        }

        let daemon = ReplicationDaemon::start(db_path, 50).unwrap();
        std::thread::sleep(Duration::from_millis(150));
        assert_eq!(daemon.total_pushed(), 1);
        daemon.stop();

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_remote_transport_e2e_with_lan_receiver() {
        let test_port = 17449;
        let receiver = crate::lan_receiver::LanPackageReceiver::start(test_port).expect("Receiver starts");

        let transport = DefaultRemoteTransport {
            remote_url: Some(format!("http://127.0.0.1:{}", test_port)),
        };

        // 1. Ping health check
        assert!(transport.ping().is_ok());

        // 2. Send outbox record
        let record = OutboxRecord {
            id: "outbox-e2e-1".into(),
            entity: "service_tickets".into(),
            record_id: "TICK-E2E-1".into(),
            op: proteus_core::replication::ChangeOp::Insert,
            payload_json: r#"{"ticket_id":"TICK-E2E-1","ticket_number":"ST-9999","customer_name":"E2E Test","customer_phone":"6900000000","device_model":"Test Phone","serial_number":"SN123","reported_fault":"Battery","internal_notes":"","estimated_cost":50.0,"current_status":"NEW","created_at":"2026-10-11","updated_at":"2026-10-11","delivered_at":null}"#.into(),
            local_timestamp: chrono::Utc::now().to_rfc3339(),
            remote_version: 1,
            status: proteus_core::replication::SyncStatus::Pending,
            priority: proteus_core::replication::DataPriority::High,
        };

        let send_res = transport.send(&record);
        assert!(send_res.is_ok(), "Transport send to LanPackageReceiver must succeed: {:?}", send_res);

        receiver.stop();
    }
}
