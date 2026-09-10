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
                ui.label(egui::RichText::new("📐 Canvas:").size(10.).color(theme::TEXT_DIM));
                let cur_dev = app.device_preset.clone();
                let (cw, ch) = cur_dev.size();
                ui.label(egui::RichText::new(format!("{} {} {}×{}", cur_dev.icon(), cur_dev.label(), cw, ch)).size(11.).color(theme::TEXT));
                ui.add_space(8.);
                ui.separator();
                ui.add_space(8.);
                let presets = [
                    DevicePreset::DesktopHD, DevicePreset::Desktop, DevicePreset::Laptop,
                    DevicePreset::TabletLandscape, DevicePreset::TabletPortrait,
                    DevicePreset::Phone, DevicePreset::PhoneSmall,
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
                        app.toast(format!("Canvas: {}", p.label()));
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(egui::RichText::new("📋 Load CRM Template").size(9.).color(theme::ACCENT)).clicked() {
                        app.project_doc = scene::create_dummy_document();
                        app.project_doc = scene::create_crm_preset();
                        app.editor_state.selected_node_ids.clear();
                        app.designer_selected_node = None;
                        app.toast("CRM Template loaded ✓");
                    }
                    ui.add_space(4.);
                    if ui.button(egui::RichText::new(if app.show_device_toolbar { "✕ Hide" } else { "☰ Show" }).size(9.).color(theme::TEXT_DIM)).clicked() {
                        app.show_device_toolbar = !app.show_device_toolbar;
                    }
                });
            });
        });
}
