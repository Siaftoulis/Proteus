use eframe::egui;
use crate::models::{Mode, Viewport2D};
use crate::ProteusApp;

pub fn show(app: &mut ProteusApp, ctx: &egui::Context) {
    let p = app.palette(ctx);
    egui::TopBottomPanel::top("mode_switcher")
        .min_height(36.)
        .resizable(false)
        .frame(egui::Frame {
            fill: p.panel,
            inner_margin: egui::Margin::symmetric(12, 5),
            stroke: egui::Stroke::new(1., p.border),
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

                // Affinity-style floating segmented capsule for Primary Workspaces
                let bar_id = ui.make_persistent_id("affinity_primary_workspaces_bar");
                let font_id = egui::FontId::proportional(11.5);
                let pill_cyan = egui::Color32::from_rgb(0, 212, 255);
                let text_dark = egui::Color32::from_rgb(10, 15, 26);
                let text_muted = egui::Color32::from_rgb(160, 168, 182);
                let text_hover = egui::Color32::from_rgb(240, 245, 255);

                let mut clicked_mode: Option<Mode> = None;

                egui::Frame::new()
                    .fill(egui::Color32::from_rgb(14, 16, 22))
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(34, 38, 50)))
                    .corner_radius(egui::CornerRadius::same(20))
                    .inner_margin(egui::Margin::symmetric(3, 3))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 2.0;

                            let bg_shape_idx = ui.painter().add(egui::Shape::Noop);
                            let mut target_rect: Option<egui::Rect> = None;

                            for (mode, icon, label, tooltip) in primary_workspaces {
                                let is_active = app.mode == *mode;
                                let display_text = format!("{} {}", icon, label);

                                let text_width = ui.painter()
                                    .layout_no_wrap(display_text.clone(), font_id.clone(), egui::Color32::WHITE)
                                    .size()
                                    .x;
                                let item_size = egui::vec2(text_width + 20.0, 27.0);

                                let (rect, resp) = ui.allocate_exact_size(item_size, egui::Sense::click());
                                let resp = resp.on_hover_text(*tooltip);

                                if resp.hovered() {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                }

                                if is_active {
                                    target_rect = Some(rect);
                                }

                                if resp.clicked() && !is_active {
                                    clicked_mode = Some(mode.clone());
                                }

                                let text_color = if is_active {
                                    text_dark
                                } else if resp.hovered() {
                                    text_hover
                                } else {
                                    text_muted
                                };

                                ui.painter().text(
                                    rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    display_text,
                                    font_id.clone(),
                                    text_color,
                                );
                            }

                            if let Some(target) = target_rect {
                                let anim_min_x = ui.ctx().animate_value_with_time(bar_id.with("min_x"), target.min.x, 0.18);
                                let anim_max_x = ui.ctx().animate_value_with_time(bar_id.with("max_x"), target.max.x, 0.18);
                                let anim_min_y = ui.ctx().animate_value_with_time(bar_id.with("min_y"), target.min.y, 0.18);
                                let anim_max_y = ui.ctx().animate_value_with_time(bar_id.with("max_y"), target.max.y, 0.18);

                                let anim_rect = egui::Rect::from_min_max(
                                    egui::pos2(anim_min_x, anim_min_y),
                                    egui::pos2(anim_max_x, anim_max_y),
                                );

                                ui.painter().set(
                                    bg_shape_idx,
                                    egui::Shape::rect_filled(
                                        anim_rect,
                                        egui::CornerRadius::same(15),
                                        pill_cyan,
                                    ),
                                );
                            }
                        });
                    });

                if let Some(mode) = clicked_mode {
                    switch_mode(app, &mode);
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
                                .color(if is_active { p.text } else { p.text_dim })
                        )
                        .fill(if is_active { p.elevated } else { p.panel })
                        .stroke(if is_active { egui::Stroke::new(1., p.accent) } else { egui::Stroke::NONE })
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
