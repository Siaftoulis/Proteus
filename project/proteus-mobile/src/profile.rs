//! Mobile Profile & Shop Configuration View.
//! Displays shop metadata, zero-privilege storage details, and manual LAN sync trigger.

use crate::{
    render_mobile_logo_widget, MobileAppState, ACCENT_GOLD, BG_CARD, BORDER_LINE,
    MIN_TOUCH_TARGET, TEXT_BODY, TEXT_TITLE,
};
use egui::{CornerRadius, Frame, Margin, RichText, Stroke, Ui, Vec2};

pub fn render_profile_tab(ui: &mut Ui, state: &mut MobileAppState) {
    state.reload_config();
    ui.label(
        RichText::new("Στοιχεία Καταστήματος (SQLite)")
            .strong()
            .size(14.0)
            .color(TEXT_TITLE),
    );
    ui.add_space(8.0);

    Frame::new()
        .fill(BG_CARD)
        .stroke(Stroke::new(1.0, BORDER_LINE))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                render_mobile_logo_widget(ui, Vec2::new(36.0, 36.0));
                ui.add_space(8.0);
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(&state.shop_config.shop_name)
                            .strong()
                            .size(13.0)
                            .color(TEXT_TITLE),
                    );
                    ui.label(
                        RichText::new("Proteus Business OS Mobile")
                            .size(11.0)
                            .color(TEXT_BODY),
                    );
                });
            });
            ui.add_space(8.0);
            ui.label(
                RichText::new(format!("Διεύθυνση: {}", state.shop_config.address))
                    .size(11.5)
                    .color(TEXT_BODY),
            );
            ui.label(
                RichText::new(format!("Τηλέφωνο: {}", state.shop_config.phone))
                    .size(11.5)
                    .color(TEXT_BODY),
            );
            ui.label(
                RichText::new(format!("Footer: {}", state.shop_config.footer_message))
                    .size(11.5)
                    .color(TEXT_BODY),
            );
            ui.label(
                RichText::new(format!("Λογότυπο / Εικονίδιο: {}", state.shop_config.logo_icon))
                    .size(11.5)
                    .color(ACCENT_GOLD),
            );
        });

    ui.add_space(10.0);
    ui.label(
        RichText::new("Ρυθμίσεις LAN Συγχρονισμού")
            .strong()
            .size(13.0)
            .color(TEXT_TITLE),
    );
    Frame::new()
        .fill(BG_CARD)
        .stroke(Stroke::new(1.0, BORDER_LINE))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::same(12))
        .show(ui, |ui| {
            if let Some(term) = &state.paired_terminal {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Συνδεδεμένο:").size(11.5).color(TEXT_BODY));
                    ui.label(RichText::new(term).strong().size(12.0).color(TEXT_TITLE));
                });
                ui.add_space(4.0);
            }

            ui.horizontal(|ui| {
                ui.label(RichText::new("Host IP:").color(TEXT_BODY).size(11.5));
                ui.add_sized(
                    Vec2::new(140.0, 30.0),
                    egui::TextEdit::singleline(&mut state.lan_host),
                );
                ui.label(RichText::new("Port:").color(TEXT_BODY).size(11.5));
                let mut port_str = state.lan_port.to_string();
                if ui
                    .add_sized(Vec2::new(60.0, 30.0), egui::TextEdit::singleline(&mut port_str))
                    .changed()
                {
                    if let Ok(p) = port_str.parse::<u16>() {
                        state.lan_port = p;
                    }
                }
            });
            ui.add_space(6.0);
            if ui
                .add_sized(
                    Vec2::new(ui.available_width(), MIN_TOUCH_TARGET),
                    egui::Button::new("⚡ Άμεσος Συγχρονισμός Outbox"),
                )
                .clicked()
            {
                match state.sync_outbox_to_lan() {
                    Ok(cnt) => {
                        state.status_message = Some((
                            format!("✓ Συγχρονίστηκαν {} εγγραφές στο κατάστημα!", cnt),
                            true,
                        ));
                    }
                    Err(e) => {
                        state.status_message = Some((e, false));
                    }
                }
            }
        });
}
