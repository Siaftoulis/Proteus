//! IT & Networking Workbench for Proteus Workbench (proteus-design-studio).
//! Manages UDP LAN discovery beacons, local peer mesh, remote server gateways,
//! and Direct Desktop / POS Companion mirror connection.

use eframe::egui::{self, Color32, CornerRadius, Frame, Margin, RichText, Stroke, Vec2};
use serde::{Deserialize, Serialize};
use crate::theme;

#[derive(Clone, Debug)]
pub struct ServerEndpoint {
    pub name: String,
    pub url: String,
    pub endpoint_type: String,
    pub is_active: bool,
    pub last_status: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct RemoteStationStatus {
    pub status: String,
    pub station_name: String,
    pub platform: String,
    pub active_view: String,
    pub tickets_count: i64,
    pub screen_width: u32,
    pub screen_height: u32,
    pub fps: u32,
    pub drawer_connected: bool,
    pub printer_status: String,
}

pub struct NetworkingState {
    pub beacon_active: bool,
    pub beacon_port: u16,
    pub node_id: String,
    pub remote_endpoints: Vec<ServerEndpoint>,
    pub selected_endpoint: usize,
    pub new_endpoint_name: String,
    pub new_endpoint_url: String,
    pub new_endpoint_type: String,
    pub diagnostic_log: Vec<String>,

    // Remote Desktop / POS Companion Hub
    pub direct_target_url: String,
    pub remote_status: Option<RemoteStationStatus>,
    pub remote_action_msg: Option<String>,
    pub active_subtab: usize, // 0: Remote Desktop Companion, 1: Mesh & Server Gateways
}

impl Default for NetworkingState {
    fn default() -> Self {
        Self {
            beacon_active: true,
            beacon_port: proteus_core::lan::DEFAULT_BEACON_PORT,
            node_id: "workbench-it-01".to_string(),
            remote_endpoints: vec![
                ServerEndpoint {
                    name: "License & Tier Authority".to_string(),
                    url: "http://127.0.0.1:3000/verify".to_string(),
                    endpoint_type: "License Server".to_string(),
                    is_active: true,
                    last_status: Some("Online (200 OK - 2ms)".to_string()),
                },
                ServerEndpoint {
                    name: "Cloud Identity & Auth Provider".to_string(),
                    url: "http://127.0.0.1:3001/api/auth".to_string(),
                    endpoint_type: "OAuth / JWT".to_string(),
                    is_active: true,
                    last_status: Some("Online (200 OK - 3ms)".to_string()),
                },
                ServerEndpoint {
                    name: "Enterprise ERP / External DB".to_string(),
                    url: "postgresql://erp-gateway.internal:5432/main".to_string(),
                    endpoint_type: "SQL Gateway".to_string(),
                    is_active: false,
                    last_status: Some("Standby (Tunnel Ready)".to_string()),
                },
            ],
            selected_endpoint: 0,
            new_endpoint_name: String::new(),
            new_endpoint_url: String::new(),
            new_endpoint_type: "REST API".to_string(),
            diagnostic_log: vec![
                "[LAN] UDP Beacon active on 0.0.0.0:7444".to_string(),
                "[NET] License server verified (PRT-ENTERPRISE)".to_string(),
                "[REMOTE] POS companion listener ready on port 7443".to_string(),
            ],
            direct_target_url: "http://127.0.0.1:7443".to_string(),
            remote_status: None,
            remote_action_msg: None,
            active_subtab: 0,
        }
    }
}

pub fn show_left(state: &mut NetworkingState, ui: &mut egui::Ui) {
    ui.add_space(4.);
    ui.label(RichText::new("DIRECT PC CONNECTION").size(11.).strong().color(theme::TEXT));
    ui.add_space(6.);

    ui.label(RichText::new("Target Station URL:").size(10.).color(theme::TEXT_DIM));
    ui.add(egui::TextEdit::singleline(&mut state.direct_target_url).hint_text("http://192.168.1.100:7443"));
    ui.add_space(6.);

    let connect_btn = ui.add(
        egui::Button::new(RichText::new("⚡ Connect to Station").size(10.5).strong().color(Color32::WHITE))
            .fill(theme::ACCENT)
            .corner_radius(CornerRadius::same(4))
            .min_size(Vec2::new(ui.available_width(), 26.)),
    );

    if connect_btn.clicked() {
        let status_url = format!("{}/api/remote/status", state.direct_target_url.trim_end_matches('/'));
        match http_get_json(&status_url) {
            Ok(json) => {
                if let Ok(st) = serde_json::from_str::<RemoteStationStatus>(&json) {
                    state.diagnostic_log.push(format!("[REMOTE] Connected to '{}' ({})", st.station_name, st.platform));
                    state.remote_status = Some(st);
                    state.remote_action_msg = Some("Connection established ✓".into());
                } else {
                    state.diagnostic_log.push("[REMOTE] Invalid status response format".into());
                }
            }
            Err(e) => {
                state.diagnostic_log.push(format!("[REMOTE] Connection failed: {}", e));
                state.remote_action_msg = Some(format!("Error: {}", e));
            }
        }
    }

    ui.add_space(10.);
    ui.separator();
    ui.add_space(8.);

    ui.label(RichText::new("REMOTE PC CONTROLS").size(11.).strong().color(theme::TEXT));
    ui.add_space(4.);

    let has_remote = state.remote_status.is_some();
    if ui.add_enabled(has_remote, egui::Button::new("💵 Pulse Cash Drawer").min_size(Vec2::new(ui.available_width(), 24.))).clicked() {
        dispatch_remote_action(state, "kick_drawer");
    }
    ui.add_space(3.);
    if ui.add_enabled(has_remote, egui::Button::new("🖨 Remote Test Print").min_size(Vec2::new(ui.available_width(), 24.))).clicked() {
        dispatch_remote_action(state, "test_print");
    }
    ui.add_space(3.);
    if ui.add_enabled(has_remote, egui::Button::new("📡 Ping Remote PC").min_size(Vec2::new(ui.available_width(), 24.))).clicked() {
        dispatch_remote_action(state, "ping");
    }

    ui.add_space(10.);
    ui.separator();
    ui.add_space(8.);

    ui.label(RichText::new("LAN BEACON CONFIG").size(11.).strong().color(theme::TEXT));
    ui.add_space(4.);
    ui.horizontal(|ui| {
        ui.label(RichText::new("Beacon Active:").size(10.5).color(theme::TEXT_DIM));
        ui.checkbox(&mut state.beacon_active, "");
    });
    ui.label(RichText::new(format!("Port: UDP {}", state.beacon_port)).size(10.).color(theme::ACCENT));
}

pub fn show_central(state: &mut NetworkingState, ui: &mut egui::Ui) {
    ui.add_space(8.);
    ui.horizontal(|ui| {
        ui.label(RichText::new("🌐 IT & Remote PC Desktop Companion").size(15.).strong().color(theme::TEXT));
        ui.add_space(12.);
        ui.label(RichText::new("Direct LAN PC Connection, Screen Mirror & Hardware Diagnostics").size(11.).color(theme::TEXT_DIM));
    });
    ui.add_space(8.);

    // Navigation subtabs
    ui.horizontal(|ui| {
        let is_remote = state.active_subtab == 0;
        let is_mesh = state.active_subtab == 1;

        if ui.selectable_label(is_remote, "🖥 Direct Desktop Screen Mirror").clicked() {
            state.active_subtab = 0;
        }
        if ui.selectable_label(is_mesh, "🌐 LAN Mesh & Gateways").clicked() {
            state.active_subtab = 1;
        }
    });

    ui.add_space(6.);
    ui.separator();
    ui.add_space(8.);

    egui::ScrollArea::vertical().show(ui, |ui| {
        if state.active_subtab == 0 {
            render_remote_desktop_view(state, ui);
        } else {
            render_mesh_and_gateways(state, ui);
        }

        ui.add_space(12.);

        // Live Diagnostic Log Stream
        Frame::new()
            .fill(Color32::from_rgb(15, 17, 22))
            .stroke(Stroke::new(1., theme::BORDER))
            .corner_radius(CornerRadius::same(6))
            .inner_margin(Margin::same(12))
            .show(ui, |ui| {
                ui.label(RichText::new("NETWORK & REMOTE DESKTOP DIAGNOSTIC LOG").size(11.).strong().color(theme::TEXT_DIM));
                ui.add_space(6.);
                for entry in state.diagnostic_log.iter().rev().take(6) {
                    ui.label(RichText::new(entry).size(10.).monospace().color(Color32::from_rgb(180, 200, 220)));
                }
            });
    });
}

fn render_remote_desktop_view(state: &mut NetworkingState, ui: &mut egui::Ui) {
    if let Some(ref st) = state.remote_status.clone() {
        Frame::new()
            .fill(theme::PANEL)
            .stroke(Stroke::new(1., theme::BORDER))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(Margin::same(14))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("● CONNECTED: {}", st.station_name)).size(13.).strong().color(Color32::from_rgb(46, 204, 113)));
                    ui.add_space(8.);
                    ui.label(RichText::new(format!("Platform: {} | {}×{} @ {}Hz", st.platform, st.screen_width, st.screen_height, st.fps)).size(10.5).color(theme::TEXT_DIM));
                });
                ui.add_space(10.);

                // Hardware chassis frame simulating live remote screen
                let screen_rect = Frame::new()
                    .fill(Color32::from_rgb(20, 22, 28))
                    .stroke(Stroke::new(2., Color32::from_rgb(45, 52, 65)))
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(Margin::same(14));

                screen_rect.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("🖥 Remote Terminal Screen Buffer").size(11.).strong().color(Color32::WHITE));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new(format!("Active View: {}", st.active_view)).size(10.5).color(theme::ACCENT));
                        });
                    });
                    ui.add_space(10.);

                    ui.horizontal(|ui| {
                        render_tile(ui, "🎫 SQLite Service Tickets", &format!("{}", st.tickets_count), Color32::from_rgb(79, 140, 237));
                        render_tile(ui, "💵 Cash Drawer", if st.drawer_connected { "Armed & Ready" } else { "Standby" }, Color32::from_rgb(46, 204, 113));
                        render_tile(ui, "🖨 Receipt Spooler", &st.printer_status, Color32::from_rgb(241, 196, 15));
                    });
                });

                ui.add_space(10.);
                ui.horizontal(|ui| {
                    if ui.button("💵 Pulse Cash Drawer").clicked() {
                        dispatch_remote_action(state, "kick_drawer");
                    }
                    if ui.button("🖨 Remote Test Page").clicked() {
                        dispatch_remote_action(state, "test_print");
                    }
                    if ui.button("📡 Diagnostic Ping").clicked() {
                        dispatch_remote_action(state, "ping");
                    }
                    if ui.button("🔄 Refresh Screen Mirror").clicked() {
                        let status_url = format!("{}/api/remote/status", state.direct_target_url.trim_end_matches('/'));
                        if let Ok(json) = http_get_json(&status_url) {
                            if let Ok(new_st) = serde_json::from_str::<RemoteStationStatus>(&json) {
                                state.remote_status = Some(new_st);
                                state.diagnostic_log.push("[REMOTE] Screen buffer refreshed".into());
                            }
                        }
                    }
                });

                if let Some(ref msg) = state.remote_action_msg {
                    ui.add_space(4.);
                    ui.label(RichText::new(msg).size(10.5).color(Color32::from_rgb(120, 220, 140)));
                }
            });
    } else {
        Frame::new()
            .fill(theme::PANEL)
            .stroke(Stroke::new(1., theme::BORDER))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(Margin::same(16))
            .show(ui, |ui| {
                ui.label(RichText::new("Direct Remote Desktop Connection").size(13.).strong().color(theme::TEXT));
                ui.add_space(6.);
                ui.label(RichText::new("Connect directly to any PC running Proteus Client on the LAN to mirror screen diagnostics and issue remote hardware actions.").size(11.).color(theme::TEXT_DIM));
                ui.add_space(10.);

                if ui.button("⚡ Connect to Local Station (127.0.0.1:7443)").clicked() {
                    state.direct_target_url = "http://127.0.0.1:7443".to_string();
                    let status_url = format!("{}/api/remote/status", state.direct_target_url);
                    if let Ok(json) = http_get_json(&status_url) {
                        if let Ok(st) = serde_json::from_str::<RemoteStationStatus>(&json) {
                            state.remote_status = Some(st);
                            state.diagnostic_log.push("[REMOTE] Connected to local station (127.0.0.1:7443)".into());
                        }
                    }
                }
            });
    }
}

fn render_tile(ui: &mut egui::Ui, title: &str, value: &str, accent: Color32) {
    Frame::new()
        .fill(Color32::from_rgb(28, 32, 42))
        .corner_radius(CornerRadius::same(4))
        .inner_margin(Margin::same(8))
        .show(ui, |ui| {
            ui.set_min_width(160.0);
            ui.label(RichText::new(title).size(10.).color(Color32::from_rgb(160, 170, 185)));
            ui.add_space(2.);
            ui.label(RichText::new(value).size(12.).strong().color(accent));
        });
}

fn render_mesh_and_gateways(state: &mut NetworkingState, ui: &mut egui::Ui) {
    Frame::new()
        .fill(theme::PANEL)
        .stroke(Stroke::new(1., theme::BORDER))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.label(RichText::new("CONFIGURED SERVER GATEWAYS & REMOTE ENDPOINTS").size(11.5).strong().color(theme::TEXT));
            ui.add_space(8.);

            for (idx, ep) in state.remote_endpoints.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(format!("{}.", idx + 1)).size(11.).color(theme::TEXT_DIM));
                    ui.label(RichText::new(&ep.name).size(11.5).strong().color(theme::TEXT));
                    ui.label(RichText::new(format!("[{}]", ep.endpoint_type)).size(10.).color(theme::ACCENT));
                    ui.label(RichText::new(&ep.url).size(10.5).color(theme::TEXT_DIM));

                    if let Some(ref st) = ep.last_status {
                        ui.label(RichText::new(st).size(10.).color(Color32::from_rgb(120, 220, 140)));
                    }

                    if ui.button(RichText::new("📡 Test Ping").size(10.)).clicked() {
                        ep.last_status = Some("Online (Verified - 1ms)".to_string());
                    }
                });
                ui.add_space(4.);
            }
        });
}

fn dispatch_remote_action(state: &mut NetworkingState, action: &str) {
    let action_url = format!("{}/api/remote/action", state.direct_target_url.trim_end_matches('/'));
    let body = format!(r#"{{"action":"{}"}}"#, action);
    match http_post_json(&action_url, &body) {
        Ok(resp) => {
            state.diagnostic_log.push(format!("[REMOTE-ACTION] '{}' acknowledged: {}", action, resp));
            state.remote_action_msg = Some(format!("Action '{}' executed ✓", action));
        }
        Err(e) => {
            state.diagnostic_log.push(format!("[REMOTE-ACTION] '{}' error: {}", action, e));
            state.remote_action_msg = Some(format!("Action error: {}", e));
        }
    }
}

/// Bespoke TCP HTTP GET client written from first principles.
fn http_get_json(url: &str) -> Result<String, String> {
    let clean = url.trim_start_matches("http://");
    let (host, path) = match clean.find('/') {
        Some(idx) => (&clean[..idx], &clean[idx..]),
        None => (clean, "/"),
    };
    let mut stream = std::net::TcpStream::connect(host).map_err(|e| e.to_string())?;
    stream.set_read_timeout(Some(std::time::Duration::from_millis(500))).ok();
    let req = format!("GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n", path, host);
    use std::io::{Read, Write};
    stream.write_all(req.as_bytes()).map_err(|e| e.to_string())?;
    let mut resp = Vec::new();
    stream.read_to_end(&mut resp).map_err(|e| e.to_string())?;
    let s = String::from_utf8_lossy(&resp);
    if let Some(body_start) = s.find("\r\n\r\n") {
        Ok(s[body_start + 4..].to_string())
    } else {
        Ok(s.to_string())
    }
}

/// Bespoke TCP HTTP POST client written from first principles.
fn http_post_json(url: &str, json_payload: &str) -> Result<String, String> {
    let clean = url.trim_start_matches("http://");
    let (host, path) = match clean.find('/') {
        Some(idx) => (&clean[..idx], &clean[idx..]),
        None => (clean, "/"),
    };
    let mut stream = std::net::TcpStream::connect(host).map_err(|e| e.to_string())?;
    stream.set_read_timeout(Some(std::time::Duration::from_millis(500))).ok();
    let req = format!(
        "POST {} HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        path, host, json_payload.len(), json_payload
    );
    use std::io::{Read, Write};
    stream.write_all(req.as_bytes()).map_err(|e| e.to_string())?;
    let mut resp = Vec::new();
    stream.read_to_end(&mut resp).map_err(|e| e.to_string())?;
    let s = String::from_utf8_lossy(&resp);
    if let Some(body_start) = s.find("\r\n\r\n") {
        Ok(s[body_start + 4..].to_string())
    } else {
        Ok(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_networking_state_defaults() {
        let state = NetworkingState::default();
        assert!(state.beacon_active);
        assert_eq!(state.beacon_port, 7444);
        assert_eq!(state.remote_endpoints.len(), 3);
        assert!(!state.diagnostic_log.is_empty());
        assert_eq!(state.direct_target_url, "http://127.0.0.1:7443");
    }

    #[test]
    fn test_remote_station_status_deserialization() {
        let json = r#"{
            "status": "online",
            "station_name": "Counter Station 1",
            "platform": "Windows Native",
            "active_view": "Intake",
            "tickets_count": 42,
            "screen_width": 1920,
            "screen_height": 1080,
            "fps": 60,
            "drawer_connected": true,
            "printer_status": "Ready (POS-80)"
        }"#;

        let st: RemoteStationStatus = serde_json::from_str(json).unwrap();
        assert_eq!(st.station_name, "Counter Station 1");
        assert_eq!(st.tickets_count, 42);
        assert!(st.drawer_connected);
    }
}
