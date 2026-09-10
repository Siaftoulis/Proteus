use eframe::egui;
use crate::models::{Mode, Viewport2D};
use crate::theme;
use crate::ProteusApp;

pub fn show(app: &mut ProteusApp, ctx: &egui::Context) {
    egui::TopBottomPanel::top("mode_switcher")
        .min_height(36.)
        .resizable(false)
        .frame(egui::Frame {
            fill: theme::PANEL,
            inner_margin: egui::Margin::symmetric(12, 5),
            stroke: egui::Stroke::new(1., theme::BORDER),
            ..Default::default()
        })
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                // ── 3 CORE PILLARS (Primary Navigation) ──
                let core_pillars: &[(Mode, &str, &str, &str)] = &[
                    (Mode::Designer,   "🎨", "Design Canvas", "Visual layout, drag-to-draw, layers & styling"),
                    (Mode::DataViewer, "📊", "Data Studio",   "SQLite database, custom entities & analytics"),
                    (Mode::Play,       "📱", "Devices & Run", "Multi-device simulator, live test & export"),
                ];

                for (mode, icon, label, tooltip) in core_pillars {
                    let is_active = app.mode == *mode;
                    let btn = ui.add(
                        egui::Button::new(
                            egui::RichText::new(format!("{}  {}", icon, label))
                                .size(11.5)
                                .color(if is_active { theme::TEXT } else { theme::TEXT_DIM })
                                .strong()
                        )
                        .fill(if is_active { theme::ELEVATED } else { theme::PANEL })
                        .stroke(if is_active { egui::Stroke::new(1.2, theme::ACCENT) } else { egui::Stroke::new(1., theme::BORDER) })
                        .corner_radius(egui::CornerRadius::same(5))
                        .min_size(egui::vec2(130., 26.))
                    ).on_hover_text(*tooltip);

                    if btn.clicked() {
                        switch_mode(app, mode);
                    }
                }

                ui.add_space(8.);
                ui.separator();
                ui.add_space(8.);

                // ── SECONDARY MODULES (Automations & CRM) ──
                let modules: &[(Mode, &str, &str)] = &[
                    (Mode::FlowBuilder, "⚡", "Flows"),
                    (Mode::Contacts,    "◎", "Contacts"),
                    (Mode::Pipeline,    "▤", "Pipeline"),
                    (Mode::Tasks,       "☰", "Tasks"),
                    (Mode::Studio,      "✦", "Freehand"),
                ];

                ui.label(egui::RichText::new("MODULES:").size(9.).color(theme::TEXT_MUTED).strong());

                for (mode, icon, label) in modules {
                    let is_active = app.mode == *mode;
                    let btn = ui.add(
                        egui::Button::new(
                            egui::RichText::new(format!("{} {}", icon, label))
                                .size(10.5)
                                .color(if is_active { theme::TEXT } else { theme::TEXT_DIM })
                        )
                        .fill(if is_active { theme::ELEVATED } else { theme::PANEL })
                        .stroke(if is_active { egui::Stroke::new(1., theme::FOCUS) } else { egui::Stroke::NONE })
                        .corner_radius(egui::CornerRadius::same(4))
                        .min_size(egui::vec2(75., 24.))
                    );

                    if btn.clicked() {
                        switch_mode(app, mode);
                    }
                }
            });
        });
}

fn switch_mode(app: &mut ProteusApp, mode: &Mode) {
    if *mode == Mode::Play && app.mode != Mode::Play {
        app.form_state.clear();
        app.editor_state.selected_node_ids.clear();
        app.active_play_page = app.project_doc.root_node_ids.first().cloned();
        app.play_viewport = Viewport2D::new();
    }
    if *mode == Mode::DataViewer && app.mode != Mode::DataViewer {
        app.viewer_selected_entity.clear();
        app.viewer_entity_input.clear();
        app.viewer_records.clear();
    }
    app.mode = mode.clone();
}
