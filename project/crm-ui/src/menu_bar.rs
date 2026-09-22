//! Native top application menu bar and action dispatcher for Proteus Designer.
//! Provides File, Edit, View, Tools, and Help menus with keyboard shortcut hints.

use eframe::egui::{self, Color32, Frame, Margin, RichText, Stroke};
use crate::models::{LayoutMode, Mode};
use crate::app_state::ProteusApp;

/// Render the native top menu bar.
pub fn show(app: &mut ProteusApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("native_menu_bar")
        .frame(
            Frame::new()
                .fill(Color32::from_rgb(18, 20, 26))
                .stroke(Stroke::new(1.0, Color32::from_rgb(38, 43, 56)))
                .inner_margin(Margin::symmetric(10, 4)),
        )
        .show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                // File Menu
                ui.menu_button("File", |ui| {
                    if ui.button("💾 Save Project (Ctrl+S)").clicked() {
                        app.save_project();
                        ui.close_menu();
                    }
                    if ui.button("📁 Open / Reload (Ctrl+O)").clicked() {
                        app.load_project();
                        app.toast("Loaded ✓");
                        ui.close_menu();
                    }
                    ui.separator();
                    let auto_deploy_txt = if app.auto_deploy_on_save {
                        "✓ Auto-Deploy LAN on Save"
                    } else {
                        "  Auto-Deploy LAN on Save"
                    };
                    if ui.button(auto_deploy_txt).clicked() {
                        app.auto_deploy_on_save = !app.auto_deploy_on_save;
                        let status = if app.auto_deploy_on_save { "Ενεργοποιήθηκε" } else { "Απενεργοποιήθηκε" };
                        app.toast(format!("Auto-Deploy LAN: {}", status));
                        ui.close_menu();
                    }
                });

                // Edit Menu
                ui.menu_button("Edit", |ui| {
                    let can_undo = !app.editor_state.undo_stack.is_empty();
                    if ui.add_enabled(can_undo, egui::Button::new("⟲ Undo (Ctrl+Z)")).clicked() {
                        app.undo();
                        ui.close_menu();
                    }
                    let can_redo = !app.editor_state.redo_stack.is_empty();
                    if ui.add_enabled(can_redo, egui::Button::new("⟳ Redo (Ctrl+Y)")).clicked() {
                        app.redo();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("➕ Insert KPI Metric Card").clicked() {
                        app.spawn_kpi_card((120.0, 120.0));
                        ui.close_menu();
                    }
                    if ui.button("📝 Insert Service Intake Form").clicked() {
                        app.spawn_form_block((150.0, 150.0));
                        ui.close_menu();
                    }
                });

                // View Menu
                ui.menu_button("View", |ui| {
                    let grid_label = if app.layout == LayoutMode::Grid { "✓ Snap-to-Grid" } else { "  Snap-to-Grid" };
                    if ui.button(grid_label).clicked() {
                        app.layout = if app.layout == LayoutMode::Grid { LayoutMode::Free } else { LayoutMode::Grid };
                        ui.close_menu();
                    }
                    let device_bar_label = if app.show_device_toolbar {
                        "✓ Device Preview Bar"
                    } else {
                        "  Device Preview Bar"
                    };
                    if ui.button(device_bar_label).clicked() {
                        app.show_device_toolbar = !app.show_device_toolbar;
                        ui.close_menu();
                    }
                    ui.separator();
                    let touch_label = if app.viewport_profile.is_touch_device {
                        "✓ Mobile Touch Profile (44pt)"
                    } else {
                        "  Mobile Touch Profile (44pt)"
                    };
                    if ui.button(touch_label).clicked() {
                        app.viewport_profile = if app.viewport_profile.is_touch_device {
                            crate::viewport::ViewportProfile::desktop()
                        } else {
                            crate::viewport::ViewportProfile::mobile_touch()
                        };
                        app.toast(if app.viewport_profile.is_touch_device {
                            "Mobile Touch Profile (44pt) Active"
                        } else {
                            "Desktop Profile Active"
                        });
                        ui.close_menu();
                    }
                });

                // Workspaces Menu
                ui.menu_button("Workspaces", |ui| {
                    if ui.selectable_label(app.mode == Mode::Designer, "🎨 Visual Designer").clicked() {
                        app.mode = Mode::Designer;
                        ui.close_menu();
                    }
                    if ui.selectable_label(app.mode == Mode::Analyst, "📊 Business & Data Analyst").clicked() {
                        app.mode = Mode::Analyst;
                        ui.close_menu();
                    }
                    if ui.selectable_label(app.mode == Mode::Networking, "🌐 IT & Networking").clicked() {
                        app.mode = Mode::Networking;
                        ui.close_menu();
                    }
                    if ui.selectable_label(app.mode == Mode::Troubleshoot, "🛠 Hardware Troubleshooting").clicked() {
                        app.mode = Mode::Troubleshoot;
                        ui.close_menu();
                    }
                    if ui.selectable_label(app.mode == Mode::ConnectedData, "🗄 Server & Connected Data").clicked() {
                        app.mode = Mode::ConnectedData;
                        ui.close_menu();
                    }
                });

                // Help Menu
                ui.menu_button("Help", |ui| {
                    if ui.button("ℹ About Proteus Engine").clicked() {
                        app.toast("Proteus Engine v0.1.0 — 100% Bespoke Architecture");
                        ui.close_menu();
                    }
                });

                // Right side: Active project name
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        RichText::new(format!("⚡ {}", app.pname))
                            .size(10.5)
                            .color(Color32::from_rgb(140, 160, 190)),
                    );
                });
            });
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_bar_toggle_options() {
        let mut app = ProteusApp::default();
        assert!(!app.auto_deploy_on_save);

        app.auto_deploy_on_save = true;
        assert!(app.auto_deploy_on_save);

        assert!(!app.show_device_toolbar);
        app.show_device_toolbar = true;
        assert!(app.show_device_toolbar);
    }
}
