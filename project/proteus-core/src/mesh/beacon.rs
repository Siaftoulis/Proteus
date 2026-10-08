//! Autonomous UDP Broadcast Beacon Engine & Packet Parser for Branch Mesh Discovery.
//! Zero third-party network libraries. Pure standard library socket primitives.
//! 100% Original Bespoke Implementation.

use std::net::{SocketAddr, UdpSocket};
use chrono::Utc;
use rusqlite::Connection;

use super::types::{
    upsert_peer_node, BranchBeacon, MeshError, PeerNode,
};

pub const DEFAULT_MESH_PORT: u16 = 7446;
pub const MAX_BEACON_PACKET_SIZE: usize = 1472; // Standard Ethernet MTU (1500 - 20 IP - 8 UDP)

#[derive(Debug, Clone)]
pub struct MeshBeaconConfig {
    pub broadcast_port: u16,
    pub listen_port: u16,
    pub beacon_interval_ms: u64,
    pub peer_ttl_seconds: i64,
}

impl Default for MeshBeaconConfig {
    fn default() -> Self {
        Self {
            broadcast_port: DEFAULT_MESH_PORT,
            listen_port: DEFAULT_MESH_PORT,
            beacon_interval_ms: 3000,
            peer_ttl_seconds: 15,
        }
    }
}

/// Transmits a branch discovery beacon via UDP broadcast.
pub fn broadcast_beacon(
    socket: &UdpSocket,
    target_port: u16,
    beacon: &BranchBeacon,
) -> Result<usize, MeshError> {
    let payload = beacon.to_packet_bytes()?;
    if payload.len() > MAX_BEACON_PACKET_SIZE {
        return Err(MeshError::InvalidPacket(
            format!("Beacon payload exceeds MTU: {} bytes", payload.len()),
        ));
    }

    let broadcast_addr = format!("255.255.255.255:{}", target_port);
    socket
        .send_to(&payload, broadcast_addr)
        .map_err(|e| MeshError::InvalidPacket(e.to_string()))
}

/// Attempts to receive a single discovery beacon packet in non-blocking mode.
pub fn receive_beacon_packet(
    socket: &UdpSocket,
    buffer: &mut [u8],
) -> Result<Option<(BranchBeacon, SocketAddr)>, MeshError> {
    match socket.recv_from(buffer) {
        Ok((len, src_addr)) => {
            if len == 0 {
                return Ok(None);
            }
            let beacon = BranchBeacon::from_packet_bytes(&buffer[..len])?;
            Ok(Some((beacon, src_addr)))
        }
        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
        Err(e) if e.kind() == std::io::ErrorKind::TimedOut => Ok(None),
        Err(e) => Err(MeshError::InvalidPacket(e.to_string())),
    }
}

/// Processes an incoming beacon packet and upserts the peer node into SQLite.
pub fn process_incoming_beacon(
    conn: &Connection,
    beacon: &BranchBeacon,
    sender_addr: SocketAddr,
    local_node_id: &str,
) -> Result<Option<PeerNode>, MeshError> {
    // Ignore self-announcements
    if beacon.node_id == local_node_id {
        return Ok(None);
    }

    let now_ts = Utc::now().timestamp_millis();
    let estimated_latency = (now_ts.saturating_sub(beacon.timestamp_utc)).clamp(0, 5000) as u32;

    let peer = PeerNode {
        node_id: beacon.node_id.clone(),
        branch_id: beacon.branch_id.clone(),
        branch_name: beacon.branch_name.clone(),
        ip_address: sender_addr.ip().to_string(),
        port: beacon.listen_port,
        last_seen_utc: now_ts,
        sync_epoch: beacon.sync_epoch,
        catalog_checksum: beacon.catalog_checksum.clone(),
        is_online: true,
        latency_ms: estimated_latency,
        role: beacon.role,
    };

    upsert_peer_node(conn, &peer)?;
    Ok(Some(peer))
}

/// Binds a UDP socket configured for broadcast transmission and receipt.
pub fn bind_mesh_socket(port: u16, non_blocking: bool) -> Result<UdpSocket, MeshError> {
    let bind_addr = format!("0.0.0.0:{}", port);
    let socket = UdpSocket::bind(bind_addr)
        .map_err(|e| MeshError::InvalidPacket(format!("Failed to bind socket: {}", e)))?;

    socket
        .set_broadcast(true)
        .map_err(|e| MeshError::InvalidPacket(format!("Failed to enable broadcast: {}", e)))?;

    if non_blocking {
        socket
            .set_nonblocking(true)
            .map_err(|e| MeshError::InvalidPacket(format!("Failed to set non-blocking: {}", e)))?;
    }

    Ok(socket)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::types::{init_mesh_schema, list_active_peers, MeshRole};
    use rusqlite::Connection;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_process_incoming_beacon_upsert_and_ignore_self() {
        let conn = Connection::open_in_memory().unwrap();
        init_mesh_schema(&conn).unwrap();

        let beacon = BranchBeacon::new(
            "BR-HER-03",
            "NODE-HER-01",
            "Heraklion Branch",
            7446,
            55,
            "hash_heraklion",
            MeshRole::PeerNode,
        );

        let sender: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 2, 45)), 7446);

        // Self-announcement must be ignored
        let self_res = process_incoming_beacon(&conn, &beacon, sender, "NODE-HER-01").unwrap();
        assert!(self_res.is_none());

        // Peer announcement from different node
        let peer_res = process_incoming_beacon(&conn, &beacon, sender, "LOCAL-NODE").unwrap();
        assert!(peer_res.is_some());
        let peer = peer_res.unwrap();
        assert_eq!(peer.node_id, "NODE-HER-01");
        assert_eq!(peer.ip_address, "192.168.2.45");
        assert_eq!(peer.branch_id, "BR-HER-03");

        // Verify written to SQLite
        let active = list_active_peers(&conn, Utc::now().timestamp_millis(), 30).unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].branch_name, "Heraklion Branch");
    }

    #[test]
    fn test_socket_loopback_beacon_roundtrip() {
        // Bind sender on ephemeral port
        let sender_socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        sender_socket.set_broadcast(true).unwrap();

        // Bind receiver on another ephemeral port
        let receiver_socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let receiver_port = receiver_socket.local_addr().unwrap().port();
        receiver_socket.set_nonblocking(true).unwrap();

        let beacon = BranchBeacon::new(
            "BR-TEST-01",
            "NODE-LOOPBACK",
            "Loopback Branch",
            receiver_port,
            120,
            "chk_loopback",
            MeshRole::Coordinator,
        );

        // Send directly to receiver port
        let payload = beacon.to_packet_bytes().unwrap();
        sender_socket
            .send_to(&payload, format!("127.0.0.1:{}", receiver_port))
            .unwrap();

        // Receive
        let mut buf = [0u8; MAX_BEACON_PACKET_SIZE];
        std::thread::sleep(std::time::Duration::from_millis(10));
        let res = receive_beacon_packet(&receiver_socket, &mut buf).unwrap();

        assert!(res.is_some());
        let (rec_beacon, _src) = res.unwrap();
        assert_eq!(rec_beacon.node_id, "NODE-LOOPBACK");
        assert_eq!(rec_beacon.sync_epoch, 120);
        assert_eq!(rec_beacon.role, MeshRole::Coordinator);
    }
}
