//! Mobile Barcode & QR Optical Scanner with Companion Terminal Pairing.
//! Parses proteus-pair:// payloads for instant zero-configuration LAN sync.

use crate::{
    MobileAppState, MobileTab, ACCENT_GOLD, BG_BASE, BG_CARD, MIN_TOUCH_TARGET,
    TEXT_BODY, TEXT_TITLE,
};
use egui::{Color32, CornerRadius, Frame, Margin, RichText, Stroke, Ui, Vec2};
use proteus_core::lan::pairing::PairingPayload;

pub fn render_scanner_tab(ui: &mut Ui, state: &mut MobileAppState) {
    ui.label(
        RichText::new("Barcode & QR Scanner (Handheld)")
            .strong()
            .size(15.0)
            .color(TEXT_TITLE),
    );
    ui.label(
        RichText::new("Υποδομή κάμερας για σάρωση εντολών επισκευής ή κωδικού σύζευξης τερματικού.")
            .size(11.5)
            .color(TEXT_BODY),
    );
    ui.add_space(10.0);

    // Active Pairing Banner if paired
    if let Some(term) = &state.paired_terminal {
        Frame::new()
            .fill(BG_CARD)
            .stroke(Stroke::new(1.0, Color32::from_rgb(52, 211, 153)))
            .corner_radius(CornerRadius::same(6))
            .inner_margin(Margin::same(10))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("✓ Συνδεδεμένο Τερματικό:").strong().size(12.0).color(Color32::from_rgb(52, 211, 153)));
                    ui.label(RichText::new(term).strong().size(12.0).color(TEXT_TITLE));
                });
                ui.label(
                    RichText::new(format!("Endpoint: {}:{}", state.lan_host, state.lan_port))
                        .size(11.0)
                        .color(TEXT_BODY),
                );
            });
        ui.add_space(8.0);
    }

    // Optical Viewfinder Simulation
    Frame::new()
        .fill(BG_BASE)
        .stroke(Stroke::new(1.5, ACCENT_GOLD))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::same(18))
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("📷").size(32.0));
                ui.label(
                    RichText::new("[ Optical QR / Barcode Scanner ]")
                        .color(TEXT_BODY)
                        .size(12.0),
                );
                ui.add_space(4.0);
                ui.label(
                    RichText::new("Τοποθετήστε το QR κωδικό σύζευξης ή barcode στο πλαίσιο")
                        .color(Color32::from_rgb(148, 163, 184))
                        .size(10.5),
                );
            });
        });

    ui.add_space(10.0);
    ui.label(
        RichText::new("Χειροκίνητη εισαγωγή ή προσομοίωση κάμερας:")
            .size(11.5)
            .color(TEXT_BODY),
    );
    ui.add_sized(
        Vec2::new(ui.available_width(), 36.0),
        egui::TextEdit::singleline(&mut state.scanned_code)
            .hint_text("proteus-pair://... ή κωδικός εντολής"),
    );

    ui.add_space(6.0);

    // Primary Action Button
    if ui
        .add_sized(
            Vec2::new(ui.available_width(), MIN_TOUCH_TARGET),
            egui::Button::new("⚡ Επεξεργασία Σάρωσης"),
        )
        .clicked()
    {
        process_scanned_input(state);
    }
}

pub fn process_scanned_input(state: &mut MobileAppState) {
    let input = state.scanned_code.trim();
    if input.is_empty() {
        return;
    }

    if input.starts_with("proteus-pair://") || input.contains("\"pairing_id\"") {
        match PairingPayload::from_qr_string(input) {
            Ok(payload) => {
                state.lan_host = payload.lan_ip.clone();
                state.lan_port = payload.port;
                state.paired_terminal = Some(payload.terminal_name.clone());
                state.store_id = Some(payload.store_id.clone());
                state.terminal_id = Some(payload.terminal_id.clone());
                state.auth_token = Some(payload.auth_token.clone());
                state.status_message = Some((
                    format!(
                        "✓ Επιτυχής σύζευξη με τερματικό '{}' ({}:{})!",
                        payload.terminal_name, payload.lan_ip, payload.port
                    ),
                    true,
                ));
                state.scanned_code.clear();
            }
            Err(e) => {
                state.status_message = Some((format!("❌ Σφάλμα QR σύζευξης: {}", e), false));
            }
        }
    } else {
        // Normal ticket / item code lookup
        state.search_query = state.scanned_code.clone();
        state.active_tab = MobileTab::Tickets;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_scanned_pairing_payload() {
        let mut state = MobileAppState::default();
        let payload = PairingPayload::new(
            "store_01",
            "pos_1",
            "Main Terminal POS",
            "192.168.1.88",
            7443,
            "dummy_cert",
            600,
        );
        let qr = payload.to_qr_string().unwrap();
        state.scanned_code = qr;

        process_scanned_input(&mut state);

        assert_eq!(state.lan_host, "192.168.1.88");
        assert_eq!(state.lan_port, 7443);
        assert_eq!(state.paired_terminal.as_deref(), Some("Main Terminal POS"));
        assert!(state.auth_token.is_some());
        assert!(state.status_message.as_ref().unwrap().1);
    }

    #[test]
    fn test_process_scanned_normal_barcode() {
        let mut state = MobileAppState::default();
        state.scanned_code = "TICK-2026-9999".into();

        process_scanned_input(&mut state);

        assert_eq!(state.search_query, "TICK-2026-9999");
        assert_eq!(state.active_tab, MobileTab::Tickets);
    }
}
