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
                // ── 5 PRIMARY ROLE WORKSPACES ──
                let primary_workspaces: &[(Mode, &str, &str, &str)] = &[
                    (Mode::Designer,      "🎨", "Designer",       "Visual layout, drag-to-draw, layers & styling"),
                    (Mode::Analyst,       "📊", "Analyst",        "Schema inference, data modeling & .pr package export"),
                    (Mode::Networking,    "🌐", "IT & Network",   "LAN auto-discovery, UDP beacon map & server gateways"),
                    (Mode::Troubleshoot,  "🛠", "Troubleshoot",   "ESC/POS printer spooler, cash drawer & diagnostic logs"),
                    (Mode::ConnectedData, "🗄", "Connected Data", "Live SQLite browser, priority outbox queue & audit Merkle"),
                ];

                for (mode, icon, label, tooltip) in primary_workspaces {
                    let is_active = app.mode == *mode;
                    let btn = ui.add(
                        egui::Button::new(
                            egui::RichText::new(format!("{} {}", icon, label))
                                .size(11.)
                                .color(if is_active { theme::TEXT } else { theme::TEXT_DIM })
                                .strong()
                        )
                        .fill(if is_active { theme::ELEVATED } else { theme::PANEL })
                        .stroke(if is_active { egui::Stroke::new(1.2, theme::ACCENT) } else { egui::Stroke::new(1., theme::BORDER) })
                        .corner_radius(egui::CornerRadius::same(5))
                        .min_size(egui::vec2(115., 26.))
                    ).on_hover_text(*tooltip);

                    if btn.clicked() {
                        switch_mode(app, mode);
                    }
                }

                ui.add_space(6.);
                ui.separator();
                ui.add_space(6.);

                // ── SECONDARY TOOLS & RUNTIMES ──
                let secondary_tools: &[(Mode, &str, &str)] = &[
                    (Mode::FlowBuilder, "⚡", "Flows"),
                    (Mode::Play,        "📱", "Simulate"),
                    (Mode::Contacts,    "◎", "Contacts"),
                    (Mode::Pipeline,    "▤", "Pipeline"),
                    (Mode::Tasks,       "☰", "Tasks"),
                    (Mode::Studio,      "✦", "Freehand"),
                ];

                for (mode, icon, label) in secondary_tools {
                    let is_active = app.mode == *mode;
                    let btn = ui.add(
                        egui::Button::new(
                            egui::RichText::new(format!("{} {}", icon, label))
                                .size(10.)
                                .color(if is_active { theme::TEXT } else { theme::TEXT_DIM })
                        )
                        .fill(if is_active { theme::ELEVATED } else { theme::PANEL })
                        .stroke(if is_active { egui::Stroke::new(1., theme::FOCUS) } else { egui::Stroke::NONE })
                        .corner_radius(egui::CornerRadius::same(4))
                        .min_size(egui::vec2(70., 24.))
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
