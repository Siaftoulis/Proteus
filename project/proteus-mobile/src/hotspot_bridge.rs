//! Zero-Touch Cellular LAN Bridge & POS Hotspot Failover (Micro-task 24.1.3).
//! Provides autonomous mobile 4G/5G failover routing for retail POS terminals
//! during fixed broadband/FTTH outages. Enforces Greek/EU metered cellular data caps
//! and prioritizes fiscal myDATA / EMV SaF payments over background bulk traffic.

use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

fn current_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailoverNetworkState {
    PrimaryBroadbandActive,
    ProbingFailover,
    CellularBridgeActive,
    FailbackGracePeriod,
    Disconnected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BridgeTrafficPriority {
    /// P1: AADE myDATA vouchers, EMV SaF card settlements, IRIS QR payments.
    P1CriticalFiscalPayment,
    /// P2: Stock updates, outbox mutations, customer records.
    P2OperationalData,
    /// P3: High-bandwidth diagnostics, audit logs, updates (blocked during cellular failover).
    P3TelemetryAndBulk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotspotBridgeConfig {
    pub bridge_id: String,
    pub hotspot_ssid: String,
    pub bridge_port: u16,
    pub daily_cellular_cap_bytes: u64,
    pub heartbeat_interval_secs: u64,
    pub probe_threshold_failures: u32,
    pub probe_threshold_successes: u32,
    pub metered_prioritization: bool,
}

impl Default for HotspotBridgeConfig {
    fn default() -> Self {
        Self {
            bridge_id: "BRIDGE-MOBILE-01".into(),
            hotspot_ssid: "PROTEUS_EMERGENCY_POS_LAN".into(),
            bridge_port: 8443,
            daily_cellular_cap_bytes: 250 * 1024 * 1024, // 250 MB emergency quota
            heartbeat_interval_secs: 10,
            probe_threshold_failures: 3,
            probe_threshold_successes: 3,
            metered_prioritization: true,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HotspotBridgeStats {
    pub total_failovers_count: u32,
    pub bytes_forwarded_cellular: u64,
    pub bytes_forwarded_broadband: u64,
    pub last_failover_epoch: Option<u64>,
    pub last_failback_epoch: Option<u64>,
    pub active_terminal_clients: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeLogEvent {
    pub event_id: String,
    pub event_type: String,
    pub old_state: String,
    pub new_state: String,
    pub bytes_used: u64,
    pub notes: String,
    pub created_at: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum HotspotBridgeError {
    DailyCapExceeded { current: u64, limit: u64 },
    TrafficBlockedByMeteredPolicy(String),
    Database(String),
    InvalidState(String),
}

impl From<rusqlite::Error> for HotspotBridgeError {
    fn from(err: rusqlite::Error) -> Self {
        HotspotBridgeError::Database(err.to_string())
    }
}

pub fn init_hotspot_bridge_schema(conn: &Connection) -> Result<(), HotspotBridgeError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS system_hotspot_bridge_events (
            event_id TEXT PRIMARY KEY,
            event_type TEXT NOT NULL,
            old_state TEXT NOT NULL,
            new_state TEXT NOT NULL,
            bytes_used INTEGER NOT NULL,
            notes TEXT NOT NULL,
            created_at TEXT NOT NULL
        );",
        [],
    )?;
    Ok(())
}

pub fn record_bridge_event(
    conn: &Connection,
    event_type: &str,
    old_state: FailoverNetworkState,
    new_state: FailoverNetworkState,
    bytes_used: u64,
    notes: &str,
) -> Result<(), HotspotBridgeError> {
    init_hotspot_bridge_schema(conn)?;
    let event_id = format!("evt_brg_{}", uuid::Uuid::new_v4());
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO system_hotspot_bridge_events (
            event_id, event_type, old_state, new_state, bytes_used, notes, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            event_id,
            event_type,
            format!("{:?}", old_state),
            format!("{:?}", new_state),
            bytes_used as i64,
            notes,
            now
        ],
    )?;
    Ok(())
}

pub fn fetch_bridge_events(conn: &Connection, limit: usize) -> Result<Vec<BridgeLogEvent>, HotspotBridgeError> {
    init_hotspot_bridge_schema(conn)?;
    let mut stmt = conn.prepare(
        "SELECT event_id, event_type, old_state, new_state, bytes_used, notes, created_at
         FROM system_hotspot_bridge_events
         ORDER BY rowid DESC
         LIMIT ?1",
    )?;
    let rows = stmt.query_map(params![limit as i64], |r| {
        Ok(BridgeLogEvent {
            event_id: r.get(0)?,
            event_type: r.get(1)?,
            old_state: r.get(2)?,
            new_state: r.get(3)?,
            bytes_used: r.get::<_, i64>(4)? as u64,
            notes: r.get(5)?,
            created_at: r.get(6)?,
        })
    })?;

    let mut list = Vec::new();
    for row in rows {
        list.push(row?);
    }
    Ok(list)
}

#[derive(Debug, Clone)]
pub struct HotspotBridgeController {
    pub state: FailoverNetworkState,
    pub consecutive_failures: u32,
    pub consecutive_successes: u32,
    pub stats: HotspotBridgeStats,
    pub config: HotspotBridgeConfig,
}

impl HotspotBridgeController {
    pub fn new(config: HotspotBridgeConfig) -> Self {
        Self {
            state: FailoverNetworkState::PrimaryBroadbandActive,
            consecutive_failures: 0,
            consecutive_successes: 0,
            stats: HotspotBridgeStats::default(),
            config,
        }
    }

    /// Evaluates if an outbound packet may pass through the bridge according to metered rules.
    pub fn can_forward_packet(
        &self,
        priority: BridgeTrafficPriority,
        byte_size: u64,
    ) -> Result<bool, HotspotBridgeError> {
        if self.state != FailoverNetworkState::CellularBridgeActive {
            // Broadband is active or recovering; all traffic allowed.
            return Ok(true);
        }

        // On cellular failover: enforce strict metered policy
        if self.config.metered_prioritization && priority == BridgeTrafficPriority::P3TelemetryAndBulk {
            return Err(HotspotBridgeError::TrafficBlockedByMeteredPolicy(
                "P3 bulk/telemetry blocked during active cellular failover".into(),
            ));
        }

        // Check cellular quota cap
        let projected = self.stats.bytes_forwarded_cellular.saturating_add(byte_size);
        if projected > self.config.daily_cellular_cap_bytes {
            // Only allow critical P1 if slightly over cap, otherwise reject
            if priority != BridgeTrafficPriority::P1CriticalFiscalPayment {
                return Err(HotspotBridgeError::DailyCapExceeded {
                    current: self.stats.bytes_forwarded_cellular,
                    limit: self.config.daily_cellular_cap_bytes,
                });
            }
        }

        Ok(true)
    }

    /// Records forwarded traffic volume across either cellular or broadband path.
    pub fn register_traffic(&mut self, priority: BridgeTrafficPriority, byte_size: u64) {
        if self.state == FailoverNetworkState::CellularBridgeActive {
            self.stats.bytes_forwarded_cellular = self.stats.bytes_forwarded_cellular.saturating_add(byte_size);
        } else {
            self.stats.bytes_forwarded_broadband = self.stats.bytes_forwarded_broadband.saturating_add(byte_size);
        }
        let _ = priority;
    }

    /// Processes heartbeat probe results from the fixed broadband uplink gateway.
    pub fn handle_heartbeat_result(
        &mut self,
        conn: &Connection,
        broadband_healthy: bool,
    ) -> Result<Option<FailoverNetworkState>, HotspotBridgeError> {
        let old_state = self.state;
        let mut state_changed = false;

        if broadband_healthy {
            self.consecutive_failures = 0;
            self.consecutive_successes = self.consecutive_successes.saturating_add(1);

            match self.state {
                FailoverNetworkState::ProbingFailover => {
                    self.state = FailoverNetworkState::PrimaryBroadbandActive;
                    state_changed = true;
                }
                FailoverNetworkState::CellularBridgeActive => {
                    if self.consecutive_successes >= self.config.probe_threshold_successes {
                        self.state = FailoverNetworkState::FailbackGracePeriod;
                        state_changed = true;
                    }
                }
                FailoverNetworkState::FailbackGracePeriod => {
                    if self.consecutive_successes >= self.config.probe_threshold_successes.saturating_add(2) {
                        self.state = FailoverNetworkState::PrimaryBroadbandActive;
                        self.stats.last_failback_epoch = Some(current_epoch_secs());
                        state_changed = true;
                    }
                }
                FailoverNetworkState::Disconnected => {
                    self.state = FailoverNetworkState::PrimaryBroadbandActive;
                    state_changed = true;
                }
                FailoverNetworkState::PrimaryBroadbandActive => {}
            }
        } else {
            self.consecutive_successes = 0;
            self.consecutive_failures = self.consecutive_failures.saturating_add(1);

            match self.state {
                FailoverNetworkState::PrimaryBroadbandActive => {
                    if self.consecutive_failures >= 1 {
                        self.state = FailoverNetworkState::ProbingFailover;
                        state_changed = true;
                    }
                }
                FailoverNetworkState::ProbingFailover => {
                    if self.consecutive_failures >= self.config.probe_threshold_failures {
                        self.state = FailoverNetworkState::CellularBridgeActive;
                        self.stats.total_failovers_count = self.stats.total_failovers_count.saturating_add(1);
                        self.stats.last_failover_epoch = Some(current_epoch_secs());
                        state_changed = true;
                    }
                }
                FailoverNetworkState::FailbackGracePeriod => {
                    // Instantly bounce back to cellular bridge if probe failed during grace period
                    self.state = FailoverNetworkState::CellularBridgeActive;
                    state_changed = true;
                }
                FailoverNetworkState::CellularBridgeActive | FailoverNetworkState::Disconnected => {}
            }
        }

        if state_changed {
            record_bridge_event(
                conn,
                "STATE_TRANSITION",
                old_state,
                self.state,
                self.stats.bytes_forwarded_cellular,
                &format!(
                    "Heartbeat result: healthy={}. Failures={}, Successes={}",
                    broadband_healthy, self.consecutive_failures, self.consecutive_successes
                ),
            )?;
            Ok(Some(self.state))
        } else {
            Ok(None)
        }
    }

    /// Manually triggers cellular failover (e.g. cashier notices broadband link down).
    pub fn force_failover(&mut self, conn: &Connection, reason: &str) -> Result<(), HotspotBridgeError> {
        let old = self.state;
        self.state = FailoverNetworkState::CellularBridgeActive;
        self.stats.total_failovers_count = self.stats.total_failovers_count.saturating_add(1);
        self.stats.last_failover_epoch = Some(current_epoch_secs());
        record_bridge_event(conn, "MANUAL_FAILOVER", old, self.state, self.stats.bytes_forwarded_cellular, reason)
    }

    /// Manually triggers failback to primary broadband.
    pub fn force_failback(&mut self, conn: &Connection, reason: &str) -> Result<(), HotspotBridgeError> {
        let old = self.state;
        self.state = FailoverNetworkState::PrimaryBroadbandActive;
        self.stats.last_failback_epoch = Some(current_epoch_secs());
        record_bridge_event(conn, "MANUAL_FAILBACK", old, self.state, self.stats.bytes_forwarded_cellular, reason)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hotspot_schema_init_and_event_recording() {
        let conn = Connection::open_in_memory().unwrap();
        init_hotspot_bridge_schema(&conn).unwrap();

        record_bridge_event(
            &conn,
            "TEST_EVENT",
            FailoverNetworkState::PrimaryBroadbandActive,
            FailoverNetworkState::CellularBridgeActive,
            1024,
            "Simulated WAN line cut",
        )
        .unwrap();

        let events = fetch_bridge_events(&conn, 10).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, "TEST_EVENT");
        assert_eq!(events[0].old_state, "PrimaryBroadbandActive");
        assert_eq!(events[0].new_state, "CellularBridgeActive");
        assert_eq!(events[0].bytes_used, 1024);
    }

    #[test]
    fn test_broadband_outage_triggers_cellular_failover() {
        let conn = Connection::open_in_memory().unwrap();
        let mut controller = HotspotBridgeController::new(HotspotBridgeConfig {
            probe_threshold_failures: 2,
            ..Default::default()
        });

        // 1st failure -> ProbingFailover
        let res = controller.handle_heartbeat_result(&conn, false).unwrap();
        assert_eq!(res, Some(FailoverNetworkState::ProbingFailover));
        assert_eq!(controller.state, FailoverNetworkState::ProbingFailover);

        // 2nd failure -> CellularBridgeActive (failover engaged)
        let res = controller.handle_heartbeat_result(&conn, false).unwrap();
        assert_eq!(res, Some(FailoverNetworkState::CellularBridgeActive));
        assert_eq!(controller.state, FailoverNetworkState::CellularBridgeActive);
        assert_eq!(controller.stats.total_failovers_count, 1);
        assert!(controller.stats.last_failover_epoch.is_some());
    }

    #[test]
    fn test_metered_traffic_prioritization_and_blocking() {
        let mut controller = HotspotBridgeController::new(HotspotBridgeConfig::default());
        controller.state = FailoverNetworkState::CellularBridgeActive;

        // P1 Critical Fiscal Payment must pass
        assert!(controller
            .can_forward_packet(BridgeTrafficPriority::P1CriticalFiscalPayment, 500)
            .is_ok());

        // P2 Operational Data must pass
        assert!(controller
            .can_forward_packet(BridgeTrafficPriority::P2OperationalData, 1000)
            .is_ok());

        // P3 Telemetry and Bulk must be blocked
        let res = controller.can_forward_packet(BridgeTrafficPriority::P3TelemetryAndBulk, 2000);
        assert!(matches!(res, Err(HotspotBridgeError::TrafficBlockedByMeteredPolicy(_))));
    }

    #[test]
    fn test_cellular_daily_cap_enforcement() {
        let mut controller = HotspotBridgeController::new(HotspotBridgeConfig {
            daily_cellular_cap_bytes: 10_000,
            ..Default::default()
        });
        controller.state = FailoverNetworkState::CellularBridgeActive;
        controller.register_traffic(BridgeTrafficPriority::P2OperationalData, 9_500);

        // Next 1000 bytes for P2 exceeds 10,000 cap -> Err(DailyCapExceeded)
        let res = controller.can_forward_packet(BridgeTrafficPriority::P2OperationalData, 1_000);
        assert!(matches!(res, Err(HotspotBridgeError::DailyCapExceeded { .. })));

        // However, P1 Critical Fiscal Payment is allowed through as safety override
        let res_p1 = controller.can_forward_packet(BridgeTrafficPriority::P1CriticalFiscalPayment, 1_000);
        assert!(res_p1.is_ok());
    }

    #[test]
    fn test_recovery_and_failback_grace_period() {
        let conn = Connection::open_in_memory().unwrap();
        let mut controller = HotspotBridgeController::new(HotspotBridgeConfig {
            probe_threshold_successes: 2,
            ..Default::default()
        });
        controller.state = FailoverNetworkState::CellularBridgeActive;

        // 1st success -> probe count 1 (still CellularBridgeActive)
        controller.handle_heartbeat_result(&conn, true).unwrap();
        assert_eq!(controller.state, FailoverNetworkState::CellularBridgeActive);

        // 2nd success -> FailbackGracePeriod
        let res = controller.handle_heartbeat_result(&conn, true).unwrap();
        assert_eq!(res, Some(FailoverNetworkState::FailbackGracePeriod));

        // Further consecutive successes -> PrimaryBroadbandActive
        controller.handle_heartbeat_result(&conn, true).unwrap();
        let res_final = controller.handle_heartbeat_result(&conn, true).unwrap();
        assert_eq!(res_final, Some(FailoverNetworkState::PrimaryBroadbandActive));
        assert!(controller.stats.last_failback_epoch.is_some());
    }

    #[test]
    fn test_manual_force_failover_and_failback() {
        let conn = Connection::open_in_memory().unwrap();
        let mut controller = HotspotBridgeController::new(HotspotBridgeConfig::default());

        controller.force_failover(&conn, "Emergency WAN line down").unwrap();
        assert_eq!(controller.state, FailoverNetworkState::CellularBridgeActive);
        assert_eq!(controller.stats.total_failovers_count, 1);

        controller.force_failback(&conn, "Broadband fiber link restored").unwrap();
        assert_eq!(controller.state, FailoverNetworkState::PrimaryBroadbandActive);
        assert!(controller.stats.last_failback_epoch.is_some());

        let events = fetch_bridge_events(&conn, 10).unwrap();
        assert_eq!(events.len(), 2);
    }
}
