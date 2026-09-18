// Proteus Core — Autonomous Local LAN Auto-Discovery & UDP Beacon Protocol
// Designed from first principles. Zero third-party network libraries.

use crate::package::{MountSummary, PrPackage};
use serde::{Deserialize, Serialize};
use std::net::{SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const DEFAULT_BEACON_PORT: u16 = 7444;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BeaconPacket {
    pub node_id: String,
    pub app_type: String,
    pub device_name: String,
    pub http_port: u16,
    pub epoch_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiscoveredPeer {
    pub node_id: String,
    pub app_type: String,
    pub device_name: String,
    pub ip: String,
    pub http_port: u16,
    pub last_seen_epoch: u64,
}

impl DiscoveredPeer {
    pub fn is_alive(&self, current_epoch: u64, ttl_secs: u64) -> bool {
        current_epoch.saturating_sub(self.last_seen_epoch) <= ttl_secs
    }

    pub fn endpoint_url(&self) -> String {
        format!("http://{}:{}", self.ip, self.http_port)
    }
}

pub struct LanDiscoveryDaemon {
    peers: Arc<Mutex<Vec<DiscoveredPeer>>>,
    is_running: Arc<AtomicBool>,
}

impl LanDiscoveryDaemon {
    pub fn start(
        node_id: String,
        app_type: String,
        device_name: Arc<Mutex<String>>,
        http_port: u16,
        listen_port: u16,
        broadcast: bool,
    ) -> Result<Self, String> {
        let peers = Arc::new(Mutex::new(Vec::<DiscoveredPeer>::new()));
        let is_running = Arc::new(AtomicBool::new(true));

        let peers_clone = peers.clone();
        let running_clone = is_running.clone();

        // Bind UDP socket
        let socket = UdpSocket::bind(format!("0.0.0.0:{}", listen_port))
            .or_else(|_| UdpSocket::bind("0.0.0.0:0"))
            .map_err(|e| format!("Failed to bind UDP discovery socket: {}", e))?;

        socket.set_broadcast(true).map_err(|e| e.to_string())?;
        socket.set_read_timeout(Some(Duration::from_millis(200))).map_err(|e| e.to_string())?;

        let socket_send = socket.try_clone().map_err(|e| e.to_string())?;
        let rx_node_id = node_id.clone();

        // Background receiver thread
        thread::spawn(move || {
            let mut buf = [0u8; 1500];
            while running_clone.load(Ordering::Relaxed) {
                if let Ok((len, src_addr)) = socket.recv_from(&mut buf) {
                    if let Ok(packet) = serde_json::from_slice::<BeaconPacket>(&buf[..len]) {
                        if packet.node_id != rx_node_id {
                            let ip = match src_addr {
                                SocketAddr::V4(v4) => v4.ip().to_string(),
                                SocketAddr::V6(v6) => v6.ip().to_string(),
                            };
                            let mut list = peers_clone.lock().unwrap();
                            if let Some(existing) = list.iter_mut().find(|p| p.node_id == packet.node_id) {
                                existing.last_seen_epoch = current_epoch();
                                existing.device_name = packet.device_name;
                                existing.http_port = packet.http_port;
                                existing.ip = ip;
                            } else {
                                list.push(DiscoveredPeer {
                                    node_id: packet.node_id,
                                    app_type: packet.app_type,
                                    device_name: packet.device_name,
                                    ip,
                                    http_port: packet.http_port,
                                    last_seen_epoch: current_epoch(),
                                });
                            }
                        }
                    }
                }
            }
        });

        // Background beacon sender thread (if broadcasting enabled)
        if broadcast {
            let running_send = is_running.clone();
            let target_port = if listen_port == 0 { DEFAULT_BEACON_PORT } else { listen_port };
            thread::spawn(move || {
                let broadcast_target = format!("255.255.255.255:{}", target_port);
                while running_send.load(Ordering::Relaxed) {
                    let d_name = device_name.lock().map(|g| g.clone()).unwrap_or_else(|_| "ProteusTerminal".into());
                    let pkt = BeaconPacket {
                        node_id: node_id.clone(),
                        app_type: app_type.clone(),
                        device_name: d_name,
                        http_port,
                        epoch_secs: current_epoch(),
                    };
                    if let Ok(bytes) = serde_json::to_vec(&pkt) {
                        let _ = socket_send.send_to(&bytes, &broadcast_target);
                        // Also ping localhost loopback for single-machine simulation
                        let _ = socket_send.send_to(&bytes, format!("127.0.0.1:{}", target_port));
                    }
                    thread::sleep(Duration::from_secs(2));
                }
            });
        }

        Ok(Self { peers, is_running })
    }

    pub fn get_live_peers(&self) -> Vec<DiscoveredPeer> {
        let now = current_epoch();
        let mut list = self.peers.lock().unwrap();
        // Prune peers inactive for more than 8 seconds
        list.retain(|p| p.is_alive(now, 8));
        list.clone()
    }

    pub fn peers_count(&self) -> usize {
        self.get_live_peers().len()
    }

    pub fn deploy_to_all_peers(&self, package: &PrPackage) -> Vec<(String, Result<MountSummary, String>)> {
        let peers = self.get_live_peers();
        let mut results = Vec::new();
        for peer in peers {
            let target_url = peer.endpoint_url();
            let res = package.deploy_to_client(&target_url);
            results.push((format!("{} ({})", peer.device_name, target_url), res));
        }
        results
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::Relaxed);
    }
}

impl Drop for LanDiscoveryDaemon {
    fn drop(&mut self) {
        self.stop();
    }
}

fn current_epoch() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_alive_pruning() {
        let peer = DiscoveredPeer {
            node_id: "node-1".into(),
            app_type: "ProteusClient".into(),
            device_name: "Cashier Terminal".into(),
            ip: "192.168.1.50".into(),
            http_port: 7443,
            last_seen_epoch: 100,
        };

        assert!(peer.is_alive(105, 8));
        assert!(!peer.is_alive(115, 8));
        assert_eq!(peer.endpoint_url(), "http://192.168.1.50:7443");
    }

    #[test]
    fn test_beacon_packet_serialization() {
        let pkt = BeaconPacket {
            node_id: "test-node".into(),
            app_type: "ProteusClient".into(),
            device_name: "Service Desk".into(),
            http_port: 7443,
            epoch_secs: 12345,
        };

        let json = serde_json::to_string(&pkt).unwrap();
        let decoded: BeaconPacket = serde_json::from_str(&json).unwrap();
        assert_eq!(pkt, decoded);
    }

    #[test]
    fn test_lan_discovery_loopback() {
        let test_port = 17444;
        let listener = LanDiscoveryDaemon::start(
            "listener-node".into(),
            "ProteusDesigner".into(),
            Arc::new(Mutex::new("Designer".into())),
            0,
            test_port,
            false,
        ).expect("Failed to start test listener");

        let broadcaster_name = Arc::new(Mutex::new("Store Terminal 1".into()));
        let broadcaster = LanDiscoveryDaemon::start(
            "store-node-1".into(),
            "ProteusClient".into(),
            broadcaster_name,
            7443,
            test_port,
            true,
        ).expect("Failed to start test broadcaster");

        // Give UDP packet a moment to transmit across localhost
        std::thread::sleep(Duration::from_millis(300));

        let peers = listener.get_live_peers();
        assert!(!peers.is_empty(), "Listener should have discovered the broadcaster peer");
        let found = peers.iter().find(|p| p.node_id == "store-node-1").expect("Peer store-node-1 must be found");
        assert_eq!(found.device_name, "Store Terminal 1");
        assert_eq!(found.http_port, 7443);

        broadcaster.stop();
        listener.stop();
    }
}

