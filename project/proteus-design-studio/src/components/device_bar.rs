use eframe::egui;
use crate::models::{DevicePreset, Mode};
use crate::theme;
use crate::ProteusApp;
use crate::scene;

pub fn show(app: &mut ProteusApp, ctx: &egui::Context) {
    if app.mode != Mode::Designer || !app.show_device_toolbar {
        return;
    }

    egui::TopBottomPanel::top("device_bar")
        .min_height(32.)
        .resizable(false)
        .frame(egui::Frame { fill: theme::PANEL, inner_margin: egui::Margin::symmetric(12, 3), ..Default::default() })
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                let cur_dev = app.device_preset.clone();
                let (cw, ch) = cur_dev.size();
                ui.label(egui::RichText::new("🎯 Track:").size(10.).color(theme::TEXT_DIM));
                let is_desktop_track = matches!(cur_dev, DevicePreset::DesktopHD | DevicePreset::Desktop | DevicePreset::Laptop | DevicePreset::AppleMacBook);
                let is_thermal_track = matches!(cur_dev, DevicePreset::Thermal80mm | DevicePreset::Thermal58mm);
                let is_web_track = !is_desktop_track && !is_thermal_track;

                if ui.add(
                    egui::Button::new(
                        egui::RichText::new("🖥 PCD-App (Software)")
                            .size(10.)
                            .color(if is_desktop_track { theme::TEXT } else { theme::TEXT_DIM })
                    )
                    .fill(if is_desktop_track { theme::ELEVATED } else { theme::PANEL })
                    .min_size(egui::vec2(120., 22.))
                ).on_hover_text("PCD-App Track: Native Desktop Software, POS Cashier & Dense Layout").clicked() {
                    app.device_preset = DevicePreset::DesktopHD;
                    app.viewport_profile = crate::viewport::ViewportProfile::desktop();
                    app.toast("Mode: PCD-App Software Designer (1920×1080 Native)".to_string());
                }

                if ui.add(
                    egui::Button::new(
                        egui::RichText::new("🌐 PCD-Web (Storefront)")
                            .size(10.)
                            .color(if is_web_track { theme::TEXT } else { theme::TEXT_DIM })
                    )
                    .fill(if is_web_track { theme::ELEVATED } else { theme::PANEL })
                    .min_size(egui::vec2(125., 22.))
                ).on_hover_text("PCD-Web Track: Public Responsive Storefront, E-Commerce & Web Portal").clicked() {
                    app.device_preset = DevicePreset::AppleIPhone;
                    app.viewport_profile = crate::viewport::ViewportProfile::apple_iphone();
                    app.toast("Mode: PCD-Web Storefront Designer (Responsive Web / Mobile)".to_string());
                }

                if ui.add(
                    egui::Button::new(
                        egui::RichText::new("🖨 ESC/POS (Hardware)")
                            .size(10.)
                            .color(if is_thermal_track { theme::TEXT } else { theme::TEXT_DIM })
                    )
                    .fill(if is_thermal_track { theme::ELEVATED } else { theme::PANEL })
                    .min_size(egui::vec2(120., 22.))
                ).on_hover_text("ESC/POS Track: Thermal Receipt Hardware, 58mm/80mm Monospace Layout & Spooler").clicked() {
                    app.device_preset = DevicePreset::Thermal80mm;
                    app.viewport_profile = crate::viewport::ViewportProfile::thermal_receipt_80mm();
                    app.toast("Mode: ESC/POS Thermal Receipt Hardware Designer (80mm / 48-Col)".to_string());
                }

                ui.add_space(6.);
                ui.separator();
                ui.add_space(6.);

                ui.label(egui::RichText::new("📐 Viewport:").size(10.).color(theme::TEXT_DIM));
                ui.label(egui::RichText::new(format!("{} {} {}×{}", cur_dev.icon(), cur_dev.label(), cw, ch)).size(11.).color(theme::TEXT));
                ui.add_space(8.);
                ui.separator();
                ui.add_space(8.);
                let presets = [
                    DevicePreset::DesktopHD, DevicePreset::Desktop, DevicePreset::Laptop,
                    DevicePreset::AppleMacBook,
                    DevicePreset::TabletLandscape, DevicePreset::TabletPortrait,
                    DevicePreset::AppleIPhone, DevicePreset::Phone, DevicePreset::PhoneSmall,
                    DevicePreset::Thermal80mm, DevicePreset::Thermal58mm,
                ];
                for p in &presets {
                    let (pw, ph) = p.size();
                    let is_active = cur_dev == *p;
                    let label = format!("{} {} {}×{}", p.icon(), p.label(), pw, ph);
                    if ui.add(
                        egui::Button::new(
                            egui::RichText::new(label)
                                .size(9.)
                                .color(if is_active { theme::TEXT } else { theme::TEXT_DIM })
                        )
                        .fill(if is_active { theme::ELEVATED } else { theme::PANEL })
                        .min_size(egui::vec2(95., 22.))
                    ).clicked() {
                        app.device_preset = p.clone();
                        app.viewport_profile = match p {
                            DevicePreset::AppleIPhone => crate::viewport::ViewportProfile::apple_iphone(),
                            DevicePreset::AppleMacBook => crate::viewport::ViewportProfile::apple_macos(),
                            DevicePreset::Phone | DevicePreset::PhoneSmall => crate::viewport::ViewportProfile::mobile_touch(),
                            DevicePreset::TabletLandscape | DevicePreset::TabletPortrait => crate::viewport::ViewportProfile::tablet_touch(),
                            DevicePreset::Thermal80mm => crate::viewport::ViewportProfile::thermal_receipt_80mm(),
                            DevicePreset::Thermal58mm => crate::viewport::ViewportProfile::thermal_receipt_58mm(),
                            _ => crate::viewport::ViewportProfile::desktop(),
                        };
                        app.toast(format!("Canvas: {} ({})", p.label(), if app.viewport_profile.is_touch_device { "Touch 44pt" } else { "Desktop" }));
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(egui::RichText::new("📐 Bespoke Workspace").size(9.).color(theme::ACCENT)).clicked() {
                        app.project_doc = scene::create_crm_preset();
                        app.editor_state.selected_node_ids.clear();
                        app.designer_selected_node = None;
                        app.toast("Bespoke Workspace loaded ✓");
                    }
                    ui.add_space(4.);
                    if ui.button(egui::RichText::new(if app.show_device_toolbar { "✕ Hide" } else { "☰ Show" }).size(9.).color(theme::TEXT_DIM)).clicked() {
                        app.show_device_toolbar = !app.show_device_toolbar;
                    }
                });
            });
        });
}
