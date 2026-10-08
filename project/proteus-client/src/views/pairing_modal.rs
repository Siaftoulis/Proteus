//! Mobile QR Pairing Modal for Proteus Terminal Client.
//! Generates cryptographically sealed QR pairing payloads for instant phone-to-terminal pairing.

use egui::{CornerRadius, Frame, Margin, RichText, Stroke, Vec2};
use proteus_core::lan::pairing::{
    init_pairing_schema, list_active_paired_devices, revoke_paired_device, PairedDevice,
    PairingPayload,
};
use rusqlite::Connection;

pub struct PairingModalState {
    pub is_open: bool,
    pub current_payload: Option<PairingPayload>,
    pub qr_string: Option<String>,
    pub paired_devices: Vec<PairedDevice>,
    pub status_msg: Option<(String, bool)>,
}

impl Default for PairingModalState {
    fn default() -> Self {
        Self {
            is_open: false,
            current_payload: None,
            qr_string: None,
            paired_devices: Vec::new(),
            status_msg: None,
        }
    }
}

impl PairingModalState {
    pub fn open_and_refresh(&mut self, conn: &Connection, store_id: &str, terminal_name: &str) {
        self.is_open = true;
        let _ = init_pairing_schema(conn);
        self.generate_new_pairing(store_id, terminal_name);
        self.reload_devices(conn);
    }

    pub fn generate_new_pairing(&mut self, store_id: &str, terminal_name: &str) {
        let payload = PairingPayload::new(
            store_id,
            "terminal_pos_01",
            terminal_name,
            "127.0.0.1",
            7443,
            "sha256:d8a5...proteus_tls_cert",
            900, // 15 minutes TTL
        );
        self.qr_string = payload.to_qr_string().ok();
        self.current_payload = Some(payload);
        self.status_msg = Some(("Δημιουργήθηκε νέος κωδικός QR σύζευξης (ισχύς 15')".into(), true));
    }

    pub fn reload_devices(&mut self, conn: &Connection) {
        if let Ok(devs) = list_active_paired_devices(conn) {
            self.paired_devices = devs;
        }
    }

    pub fn revoke_device(&mut self, conn: &Connection, device_id: &str) {
        match revoke_paired_device(conn, device_id) {
            Ok(true) => {
                self.status_msg = Some((format!("✓ Η συσκευή '{}' ανακλήθηκε", device_id), true));
                self.reload_devices(conn);
            }
            Ok(false) => {
                self.status_msg = Some((format!("Η συσκευή '{}' δεν βρέθηκε", device_id), false));
            }
            Err(e) => {
                self.status_msg = Some((format!("Σφάλμα SQLite: {}", e), false));
            }
        }
    }
}

pub fn draw_pairing_modal(
    ctx: &egui::Context,
    conn: &Connection,
    state: &mut PairingModalState,
    store_id: &str,
    terminal_name: &str,
) {
    if !state.is_open {
        return;
    }

    let mut open = state.is_open;
    egui::Window::new("📱 Σύζευξη Κινητού Συνεργάτη (QR Mobile Pairing)")
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .fixed_size(Vec2::new(540.0, 520.0))
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(ctx, |ui| {
            ui.add_space(4.0);
            ui.label(
                RichText::new("Σαρώστε τον παρακάτω κωδικό με την εφαρμογή Proteus Mobile για άμεση εξουσιοδότηση και LAN συγχρονισμό.")
                    .size(12.0)
                    .color(crate::theme::TEXT_SECONDARY),
            );
            ui.add_space(8.0);

            if let Some((msg, is_ok)) = &state.status_msg {
                let col = if *is_ok {
                    crate::theme::STATUS_READY
                } else {
                    crate::theme::STATUS_CANCELLED
                };
                ui.label(RichText::new(msg).size(12.0).color(col));
                ui.add_space(6.0);
            }

            // QR Code Box
            Frame::new()
                .fill(crate::theme::BG_PANEL)
                .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::same(12))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("🔑 Ενεργό Token Σύζευξης:").strong().size(12.0).color(crate::theme::TEXT_PRIMARY));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("🔄 Ανανέωση").clicked() {
                                state.generate_new_pairing(store_id, terminal_name);
                            }
                        });
                    });
                    ui.add_space(6.0);

                    if let Some(qr) = &state.qr_string {
                        ui.add(
                            egui::TextEdit::multiline(&mut qr.as_str())
                                .desired_rows(3)
                                .font(egui::TextStyle::Monospace)
                                .desired_width(ui.available_width()),
                        );
                    }

                    if let Some(payload) = &state.current_payload {
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(format!("Host: {}:{}", payload.lan_ip, payload.port)).size(11.0).color(crate::theme::ACCENT_CYAN));
                            ui.label(RichText::new(format!("Terminal: {}", payload.terminal_name)).size(11.0).color(crate::theme::TEXT_MUTED));
                        });
                    }
                });

            ui.add_space(12.0);
            ui.label(RichText::new("Συνδεδεμένες Συσκευές (Paired Devices):").strong().size(13.0).color(crate::theme::TEXT_PRIMARY));
            ui.add_space(4.0);

            Frame::new()
                .fill(crate::theme::BG_PANEL)
                .stroke(Stroke::new(1.0, crate::theme::BORDER_SUBTLE))
                .corner_radius(CornerRadius::same(6))
                .inner_margin(Margin::same(10))
                .show(ui, |ui| {
                    if state.paired_devices.is_empty() {
                        ui.label(RichText::new("Δεν υπάρχουν συνδεδεμένες φορητές συσκευές.").size(12.0).color(crate::theme::TEXT_MUTED));
                    } else {
                        let mut dev_to_revoke = None;
                        for dev in &state.paired_devices {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("📱").size(14.0));
                                ui.vertical(|ui| {
                                    ui.label(RichText::new(&dev.device_name).strong().size(12.0).color(crate::theme::TEXT_PRIMARY));
                                    ui.label(RichText::new(format!("ID: {} | Ρόλοι: {:?}", dev.device_id, dev.allowed_roles)).size(10.5).color(crate::theme::TEXT_MUTED));
                                });
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button(RichText::new("Ανάκληση").size(11.0).color(crate::theme::STATUS_CANCELLED)).clicked() {
                                        dev_to_revoke = Some(dev.device_id.clone());
                                    }
                                });
                            });
                            ui.separator();
                        }
                        if let Some(id) = dev_to_revoke {
                            state.revoke_device(conn, &id);
                        }
                    }
                });

            ui.add_space(12.0);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(RichText::new("Κλείσιμο").strong()).clicked() {
                    state.is_open = false;
                }
            });
        });

    state.is_open = open;
}
