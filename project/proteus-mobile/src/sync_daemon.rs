//! Mobile Companion Outbox Auto-Sync Daemon & Resilient Hybrid LAN/Relay Sync Controller.
//! Designed from first principles for zero-privilege, offline-first mobile operations.

use proteus_core::lan::relay::{RelayEnvelope, RelayMessageType};
use proteus_core::replication::{fetch_pending_outbox, mark_outbox_status, SyncStatus};
use rusqlite::Connection;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn current_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncTransport {
    LanDirect,
    CloudRelay,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelaySyncParams {
    pub relay_url: String,
    pub session_id: String,
    pub client_id: String,
    pub store_id: String,
    pub recipient_terminal_id: String,
    pub pairing_secret: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncStatusInfo {
    Synced { transport: SyncTransport },
    Pending(usize),
    Syncing,
    Offline { retrying_in_secs: u64 },
}

#[derive(Debug, Clone)]
pub struct MobileSyncController {
    pub last_attempt_epoch: u64,
    pub base_interval_secs: u64,
    pub current_backoff_secs: u64,
    pub consecutive_failures: u32,
    pub is_syncing: bool,
    pub last_synced_count: usize,
    pub active_transport: SyncTransport,
    pub last_error: Option<String>,
    pub needs_immediate_trigger: bool,
}

impl Default for MobileSyncController {
    fn default() -> Self {
        Self {
            last_attempt_epoch: 0,
            base_interval_secs: 5,
            current_backoff_secs: 0,
            consecutive_failures: 0,
            is_syncing: false,
            last_synced_count: 0,
            active_transport: SyncTransport::LanDirect,
            last_error: None,
            needs_immediate_trigger: false,
        }
    }
}

impl MobileSyncController {
    pub fn trigger_immediate(&mut self) {
        self.needs_immediate_trigger = true;
    }

    pub fn should_sync(&self, current_epoch: u64, pending_count: usize) -> bool {
        if pending_count == 0 {
            return false;
        }
        if self.needs_immediate_trigger {
            return true;
        }
        let elapsed = current_epoch.saturating_sub(self.last_attempt_epoch);
        elapsed >= self.base_interval_secs + self.current_backoff_secs
    }

    pub fn record_success(&mut self, count: usize, transport: SyncTransport) {
        self.consecutive_failures = 0;
        self.current_backoff_secs = 0;
        self.last_synced_count = count;
        self.active_transport = transport;
        self.last_error = None;
        self.needs_immediate_trigger = false;
        self.is_syncing = false;
    }

    pub fn record_failure(&mut self, err: String) {
        self.consecutive_failures = self.consecutive_failures.saturating_add(1);
        let multiplier = 1u64 << self.consecutive_failures.min(4);
        self.current_backoff_secs = (multiplier * 2).clamp(5, 60);
        self.last_error = Some(err);
        self.needs_immediate_trigger = false;
        self.is_syncing = false;
    }

    pub fn status_display(&self, pending_count: usize, current_epoch: u64) -> SyncStatusInfo {
        if self.is_syncing {
            return SyncStatusInfo::Syncing;
        }
        if pending_count == 0 {
            return SyncStatusInfo::Synced {
                transport: self.active_transport,
            };
        }
        if self.consecutive_failures > 0 {
            let elapsed = current_epoch.saturating_sub(self.last_attempt_epoch);
            let wait_target = self.base_interval_secs + self.current_backoff_secs;
            let retrying_in_secs = wait_target.saturating_sub(elapsed);
            return SyncStatusInfo::Offline { retrying_in_secs };
        }
        SyncStatusInfo::Pending(pending_count)
    }
}

pub fn parse_relay_address(raw: &str) -> Result<(String, u16), String> {
    let t = raw.trim();
    let s = t.strip_prefix("https://").or_else(|| t.strip_prefix("http://")).unwrap_or(t);
    let host_port = s.split('/').next().unwrap_or(s);
    if let Some((h, p)) = host_port.split_once(':') {
        let port: u16 = p.parse().map_err(|_| format!("Μη έγκυρη θύρα relay: {}", p))?;
        Ok((h.to_string(), port))
    } else {
        Ok((host_port.to_string(), 8443))
    }
}

fn post_http_json(
    target: &str,
    path: &str,
    body: &str,
    auth_hdr: &str,
    timeout_secs: u64,
) -> Result<String, String> {
    let addr = target
        .parse()
        .map_err(|_| format!("Μη έγκυρη διεύθυνση {}", target))?;
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_secs(timeout_secs))
        .map_err(|e| format!("Αποτυχία σύνδεσης σε {}: {}", target, e))?;

    let req = format!(
        "POST {} HTTP/1.1\r\nHost: {}\r\n{}Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        path, target, auth_hdr, body.len(), body
    );

    stream
        .write_all(req.as_bytes())
        .map_err(|e| format!("Σφάλμα αποστολής: {}", e))?;
    let mut resp = String::new();
    let _ = stream.read_to_string(&mut resp);
    Ok(resp)
}

pub fn perform_outbox_sync(
    conn: &Connection,
    host: &str,
    port: u16,
    auth_token: Option<&str>,
) -> Result<usize, String> {
    let pending = fetch_pending_outbox(conn, 50).map_err(|e| e.to_string())?;
    if pending.is_empty() {
        return Ok(0);
    }

    let payload = serde_json::to_string(&pending).map_err(|e| e.to_string())?;
    let target = format!("{}:{}", host, port);
    let auth_hdr = auth_token
        .map(|t| format!("Authorization: Bearer {}\r\n", t))
        .unwrap_or_default();
    let resp = post_http_json(&target, "/api/sync/outbox", &payload, &auth_hdr, 2)?;

    if resp.contains("200 OK") {
        for rec in &pending {
            let _ = mark_outbox_status(conn, &rec.id, SyncStatus::Synced);
        }
        Ok(pending.len())
    } else {
        Err(format!(
            "Σφάλμα συγχρονισμού: {}",
            resp.lines().next().unwrap_or("Άγνωστο")
        ))
    }
}

pub fn perform_relay_outbox_sync(
    conn: &Connection,
    params: &RelaySyncParams,
) -> Result<usize, String> {
    let pending = fetch_pending_outbox(conn, 50).map_err(|e| e.to_string())?;
    if pending.is_empty() {
        return Ok(0);
    }

    let payload_bytes = serde_json::to_vec(&pending).map_err(|e| e.to_string())?;
    let envelope = RelayEnvelope::new(
        &params.session_id,
        &params.client_id,
        &params.recipient_terminal_id,
        RelayMessageType::SyncOutbox,
        payload_bytes,
        &params.pairing_secret,
    );

    let envelope_json = serde_json::to_string(&envelope).map_err(|e| e.to_string())?;
    let (host, port) = parse_relay_address(&params.relay_url)?;
    let target = format!("{}:{}", host, port);
    let resp = post_http_json(&target, "/api/relay/envelope", &envelope_json, "", 3)?;

    if resp.contains("200 OK") {
        for rec in &pending {
            let _ = mark_outbox_status(conn, &rec.id, SyncStatus::Synced);
        }
        Ok(pending.len())
    } else {
        Err(format!(
            "Σφάλμα relay: {}",
            resp.lines().next().unwrap_or("Άγνωστο")
        ))
    }
}

pub fn perform_resilient_sync(
    conn: &Connection,
    host: &str,
    port: u16,
    auth_token: Option<&str>,
    relay_params: Option<&RelaySyncParams>,
) -> Result<(usize, SyncTransport), String> {
    match perform_outbox_sync(conn, host, port, auth_token) {
        Ok(cnt) => Ok((cnt, SyncTransport::LanDirect)),
        Err(lan_err) => {
            if let Some(r_params) = relay_params {
                match perform_relay_outbox_sync(conn, r_params) {
                    Ok(cnt) => Ok((cnt, SyncTransport::CloudRelay)),
                    Err(relay_err) => {
                        Err(format!("LAN: {} | Relay: {}", lan_err, relay_err))
                    }
                }
            } else {
                Err(lan_err)
            }
        }
    }
}

pub fn step_auto_sync(
    conn: &Connection,
    controller: &mut MobileSyncController,
    host: &str,
    port: u16,
    auth_token: Option<&str>,
) -> Option<Result<usize, String>> {
    step_resilient_auto_sync(conn, controller, host, port, auth_token, None)
        .map(|res| res.map(|(cnt, _)| cnt))
}

pub fn step_resilient_auto_sync(
    conn: &Connection,
    controller: &mut MobileSyncController,
    host: &str,
    port: u16,
    auth_token: Option<&str>,
    relay_params: Option<&RelaySyncParams>,
) -> Option<Result<(usize, SyncTransport), String>> {
    let pending = fetch_pending_outbox(conn, 100).map(|v| v.len()).unwrap_or(0);
    let now = current_epoch_secs();

    if !controller.should_sync(now, pending) {
        return None;
    }

    controller.last_attempt_epoch = now;
    controller.is_syncing = true;

    match perform_resilient_sync(conn, host, port, auth_token, relay_params) {
        Ok((cnt, transport)) => {
            controller.record_success(cnt, transport);
            Some(Ok((cnt, transport)))
        }
        Err(e) => {
            controller.record_failure(e.clone());
            Some(Err(e))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_controller_lifecycle_and_backoff() {
        let mut ctrl = MobileSyncController::default();
        let now = 1000;

        assert!(!ctrl.should_sync(now, 0));
        assert!(ctrl.should_sync(now, 5));

        ctrl.last_attempt_epoch = now;
        ctrl.record_failure("Connection refused".into());
        assert_eq!(ctrl.consecutive_failures, 1);
        assert!(ctrl.current_backoff_secs >= 5);

        assert!(!ctrl.should_sync(now + 2, 5));

        ctrl.trigger_immediate();
        assert!(ctrl.should_sync(now + 2, 5));

        ctrl.record_success(3, SyncTransport::LanDirect);
        assert_eq!(ctrl.consecutive_failures, 0);
        assert_eq!(ctrl.current_backoff_secs, 0);
        assert_eq!(ctrl.last_synced_count, 3);
        assert_eq!(ctrl.active_transport, SyncTransport::LanDirect);
    }

    #[test]
    fn test_sync_status_display_transitions() {
        let mut ctrl = MobileSyncController::default();
        let now = 1000;

        assert_eq!(
            ctrl.status_display(0, now),
            SyncStatusInfo::Synced {
                transport: SyncTransport::LanDirect
            }
        );
        assert_eq!(ctrl.status_display(3, now), SyncStatusInfo::Pending(3));

        ctrl.is_syncing = true;
        assert_eq!(ctrl.status_display(3, now), SyncStatusInfo::Syncing);
        ctrl.is_syncing = false;

        ctrl.record_success(5, SyncTransport::CloudRelay);
        assert_eq!(
            ctrl.status_display(0, now),
            SyncStatusInfo::Synced {
                transport: SyncTransport::CloudRelay
            }
        );

        ctrl.last_attempt_epoch = now;
        ctrl.record_failure("Network timeout".into());
        match ctrl.status_display(3, now) {
            SyncStatusInfo::Offline { retrying_in_secs } => {
                assert!(retrying_in_secs > 0);
            }
            _ => panic!("Expected Offline status"),
        }
    }

    #[test]
    fn test_parse_relay_address_formats() {
        assert_eq!(parse_relay_address("https://relay.proteus-bos.gr:8443").unwrap(), ("relay.proteus-bos.gr".into(), 8443));
        assert_eq!(parse_relay_address("http://192.168.1.100:9000/api").unwrap(), ("192.168.1.100".into(), 9000));
        assert_eq!(parse_relay_address("custom-relay.internal").unwrap(), ("custom-relay.internal".into(), 8443));
    }

    #[test]
    fn test_resilient_sync_fallback_aggregation() {
        let conn = Connection::open_in_memory().unwrap();
        proteus_core::replication::init_outbox_schema(&conn).unwrap();
        proteus_core::replication::enqueue_outbox(
            &conn, "service_tickets", "rec-01", proteus_core::replication::ChangeOp::Insert, "{}"
        ).unwrap();

        let relay_params = RelaySyncParams {
            relay_url: "127.0.0.1:59999".into(),
            session_id: "test-sess".into(),
            client_id: "mobile-01".into(),
            store_id: "store-01".into(),
            recipient_terminal_id: "term-01".into(),
            pairing_secret: "secret-key".into(),
        };

        let res = perform_resilient_sync(&conn, "127.0.0.1", 59998, None, Some(&relay_params));
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(err.contains("LAN:") && err.contains("Relay:"));
    }
}
