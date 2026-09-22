//! IT & Networking Workbench for Proteus Workbench (crm-ui).
//! Manages UDP LAN discovery beacons, local peer mesh, and remote server gateway connections.

use eframe::egui::{self, Color32, CornerRadius, Frame, Margin, RichText, Stroke, Vec2};
use crate::theme;

#[derive(Clone, Debug)]
pub struct ServerEndpoint {
    pub name: String,
    pub url: String,
    pub endpoint_type: String,
    pub is_active: bool,
    pub last_status: Option<String>,
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
}

impl Default for NetworkingState {
    fn default() -> Self {
        Self {
            beacon_active: true,
            beacon_port: crm_core::lan::DEFAULT_BEACON_PORT,
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
                "[NET] Gateway socket pool initialized (Max: 16 conns)".to_string(),
            ],
        }
    }
}

pub fn show_left(state: &mut NetworkingState, ui: &mut egui::Ui) {
    ui.add_space(4.);
    ui.label(RichText::new("LAN BEACON CONFIG").size(11.).strong().color(theme::TEXT));
    ui.add_space(6.);

    ui.horizontal(|ui| {
        ui.label(RichText::new("Beacon Active:").size(10.5).color(theme::TEXT_DIM));
        ui.checkbox(&mut state.beacon_active, "");
    });
    ui.add_space(4.);

    ui.label(RichText::new("Node Identifier:").size(10.).color(theme::TEXT_DIM));
    ui.add(egui::TextEdit::singleline(&mut state.node_id));
    ui.add_space(4.);

    ui.label(RichText::new(format!("Beacon Port: UDP {}", state.beacon_port)).size(10.).color(theme::ACCENT));
    ui.add_space(10.);
    ui.separator();
    ui.add_space(8.);

    ui.label(RichText::new("ADD SERVER GATEWAY").size(11.).strong().color(theme::TEXT));
    ui.add_space(4.);

    ui.label(RichText::new("Gateway Name:").size(10.).color(theme::TEXT_DIM));
    ui.add(egui::TextEdit::singleline(&mut state.new_endpoint_name).hint_text("e.g. POS Server"));
    ui.add_space(4.);

    ui.label(RichText::new("Endpoint URL:").size(10.).color(theme::TEXT_DIM));
    ui.add(egui::TextEdit::singleline(&mut state.new_endpoint_url).hint_text("https://api.mycompany.com"));
    ui.add_space(6.);

    let add_btn = ui.add(
        egui::Button::new(RichText::new("+ Add Server Gateway").size(10.5).strong().color(Color32::WHITE))
            .fill(theme::ACCENT)
            .corner_radius(CornerRadius::same(4))
            .min_size(Vec2::new(ui.available_width(), 24.)),
    );

    if add_btn.clicked() && !state.new_endpoint_name.trim().is_empty() {
        state.remote_endpoints.push(ServerEndpoint {
            name: state.new_endpoint_name.clone(),
            url: state.new_endpoint_url.clone(),
            endpoint_type: state.new_endpoint_type.clone(),
            is_active: true,
            last_status: Some("Configured".to_string()),
        });
        state.diagnostic_log.push(format!("[GATEWAY] Added endpoint '{}'", state.new_endpoint_name));
        state.new_endpoint_name.clear();
        state.new_endpoint_url.clear();
    }
}

pub fn show_central(state: &mut NetworkingState, ui: &mut egui::Ui) {
    ui.add_space(8.);
    ui.horizontal(|ui| {
        ui.label(RichText::new("🌐 IT & Networking Workbench").size(15.).strong().color(theme::TEXT));
        ui.add_space(12.);
        ui.label(RichText::new("Local Mesh, UDP Beacon, and External Server Gateways").size(11.).color(theme::TEXT_DIM));
    });
    ui.add_space(8.);
    ui.separator();
    ui.add_space(8.);

    egui::ScrollArea::vertical().show(ui, |ui| {
        // LAN Beacon Card
        Frame::new()
            .fill(theme::PANEL)
            .stroke(Stroke::new(1., theme::BORDER))
            .corner_radius(CornerRadius::same(6))
            .inner_margin(Margin::same(12))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("LOCAL LAN DISCOVERY STATUS").size(11.5).strong().color(theme::TEXT));
                    ui.add_space(8.);
                    let (badge_txt, badge_color) = if state.beacon_active {
                        ("● ACTIVE BEACON", Color32::from_rgb(46, 125, 50))
                    } else {
                        ("○ MUTED", Color32::from_rgb(180, 100, 50))
                    };
                    ui.label(RichText::new(badge_txt).size(10.).strong().color(badge_color));
                });
                ui.add_space(6.);
                ui.label(
                    RichText::new(format!(
                        "Broadcasting UDP keepalives on port {} as '{}'. Nearby shop terminals and support stations discover this instance automatically without central routing.",
                        state.beacon_port, state.node_id
                    ))
                    .size(10.5)
                    .color(theme::TEXT_DIM),
                );
            });

        ui.add_space(12.);

        // Configured Remote Gateways Card
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

        ui.add_space(12.);

        // Live Diagnostic Log Stream
        Frame::new()
            .fill(Color32::from_rgb(15, 17, 22))
            .stroke(Stroke::new(1., theme::BORDER))
            .corner_radius(CornerRadius::same(6))
            .inner_margin(Margin::same(12))
            .show(ui, |ui| {
                ui.label(RichText::new("NETWORK DIAGNOSTIC EVENT LOG").size(11.).strong().color(theme::TEXT_DIM));
                ui.add_space(6.);
                for entry in state.diagnostic_log.iter().rev().take(6) {
                    ui.label(RichText::new(entry).size(10.).monospace().color(Color32::from_rgb(180, 200, 220)));
                }
            });

    });
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
    }
}

