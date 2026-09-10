use eframe::egui;
use crate::models::LayoutMode;
use crate::theme;
use crate::ProteusApp;

pub fn show(app: &mut ProteusApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("cmd_bar")
        .min_height(44.)
        .resizable(false)
        .frame(egui::Frame { fill: theme::PANEL, inner_margin: egui::Margin::symmetric(12, 8), ..Default::default() })
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("PROTEUS").size(16.).color(theme::ACCENT).strong());
                ui.label(egui::RichText::new("Workspace").size(10.).color(theme::TEXT_DIM).strong());
                ui.add_space(12.); ui.separator(); ui.add_space(12.);
                ui.label(egui::RichText::new(format!("📂 {}", app.pname)).size(13.).color(theme::TEXT));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if let Some(u) = &app.auth {
                        ui.label(egui::RichText::new(format!("👤 {}", u)).size(12.).color(theme::TEXT_DIM));
                    } else if ui.add(egui::Button::new("Sign In").min_size(egui::vec2(75., 26.))).clicked() {
                        app.show_login = true;
                    }
                    ui.add_space(8.);
                    ui.separator();
                    ui.add_space(8.);

                    if ui.add(egui::Button::new(egui::RichText::new("💾 Save").strong()).fill(theme::ACCENT).min_size(egui::vec2(75., 26.))).clicked() {
                        app.save_project();
                    }
                    if ui.add(egui::Button::new("📁 Open").min_size(egui::vec2(65., 26.))).clicked() {
                        app.load_project();
                        app.toast("Loaded ✓");
                    }
                    let can_undo = !app.editor_state.undo_stack.is_empty();
                    if ui.add_enabled(can_undo, egui::Button::new("⟲ Undo").min_size(egui::vec2(65., 26.)))
                        .on_hover_text("Undo (Ctrl+Z)")
                        .clicked()
                    {
                        app.undo();
                    }
                    let can_redo = !app.editor_state.redo_stack.is_empty();
                    if ui.add_enabled(can_redo, egui::Button::new("⟳ Redo").min_size(egui::vec2(65., 26.)))
                        .on_hover_text("Redo (Ctrl+Y)")
                        .clicked()
                    {
                        app.redo();
                    }
                    if ui.selectable_label(app.layout == LayoutMode::Grid, "🌐 Grid").clicked() {
                        app.layout = if app.layout == LayoutMode::Grid { LayoutMode::Free } else { LayoutMode::Grid };
                    }
                });
            });
        });
}
