use eframe::egui::{self, Color32, Pos2, Rect, Response, Stroke, Vec2};
use crate::models::LayoutMode;
use crate::models::GRID;
use crate::scene::{self, CanvasEvent};
use crate::theme;
use crate::ProteusApp;
use crate::renderer;
use crate::inspector;

fn tool_icon_btn(
    ui: &mut egui::Ui,
    icon: &'static str,
    is_active: bool,
    title: &'static str,
    shortcut: &'static str,
    desc: &'static str,
) -> egui::Response {
    let btn = egui::Button::new(
        egui::RichText::new(icon)
            .size(13.)
            .color(if is_active { theme::ACCENT } else { Color32::from_rgb(175, 180, 190) }),
    )
    .fill(if is_active { Color32::from_rgba_unmultiplied(79, 140, 237, 30) } else { Color32::TRANSPARENT })
    .stroke(if is_active { Stroke::new(1., theme::ACCENT) } else { Stroke::NONE })
    .corner_radius(egui::CornerRadius::same(4))
    .min_size(Vec2::new(30., 28.));

    let resp = ui.add(btn);
    resp.on_hover_ui(|ui| {
        ui.set_max_width(170.);
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new(title).size(11.).strong().color(Color32::WHITE));
            ui.label(egui::RichText::new(format!("({})", shortcut)).size(9.).color(theme::ACCENT));
        });
        ui.label(egui::RichText::new(desc).size(9.).color(Color32::LIGHT_GRAY));
    })
}

pub fn show_left(app: &mut ProteusApp, ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(2.);

        // Single-column vertical tools (Affinity Designer style)
        if tool_icon_btn(ui, "↖", app.active_tool == crate::models::DesignerTool::Select, "Select Tool", "V", "Click to select, drag on canvas for marquee multi-selection.").clicked() {
            app.active_tool = crate::models::DesignerTool::Select;
        }
        ui.add_space(2.);
        if tool_icon_btn(ui, "▢", app.active_tool == crate::models::DesignerTool::Rectangle, "Smart Box", "R", "Smart Box tool. Click & drag on canvas to draw directly.").clicked() {
            app.active_tool = crate::models::DesignerTool::Rectangle;
        }
        ui.add_space(2.);
        if tool_icon_btn(ui, "T", app.active_tool == crate::models::DesignerTool::Text, "Text Tool", "T", "Text tool. Click & drag on canvas to draw text.").clicked() {
            app.active_tool = crate::models::DesignerTool::Text;
        }
        ui.add_space(2.);
        if tool_icon_btn(ui, "🔘", app.active_tool == crate::models::DesignerTool::Button, "Action Button", "B", "Action Button tool. Click & drag on canvas to draw a button.").clicked() {
            app.active_tool = crate::models::DesignerTool::Button;
        }
        ui.add_space(2.);
        if tool_icon_btn(ui, "⊞", app.active_tool == crate::models::DesignerTool::Table, "Data Table", "G", "Data Table tool. Click & drag on canvas to draw a data-bound table.").clicked() {
            app.active_tool = crate::models::DesignerTool::Table;
        }
        ui.add_space(2.);
        if tool_icon_btn(ui, "✋", app.active_tool == crate::models::DesignerTool::Hand, "Hand Tool", "H", "Pan canvas viewport freely.").clicked() {
            app.active_tool = crate::models::DesignerTool::Hand;
        }
        ui.add_space(2.);
        if tool_icon_btn(ui, "⌫", app.active_tool == crate::models::DesignerTool::Eraser, "Eraser", "E", "Delete currently selected element.").clicked() {
            if let Some(ref sel_id) = app.designer_selected_node.clone() {
                app.project_doc.delete_node(sel_id);
                app.designer_selected_node = None;
                app.editor_state.selected_node_ids.clear();
                app.toast("Element deleted");
            }
        }
        ui.add_space(2.);
        if tool_icon_btn(ui, "✂", app.active_tool == crate::models::DesignerTool::Crop, "Crop Tool", "C", "Crop canvas area (Placeholder - v1.1).").clicked() {
            app.active_tool = crate::models::DesignerTool::Crop;
        }
        ui.add_space(2.);
        if tool_icon_btn(ui, "🔍", app.active_tool == crate::models::DesignerTool::Zoom, "Zoom Tool", "Z", "Zoom canvas (Mouse wheel also zooms).").clicked() {
            app.active_tool = crate::models::DesignerTool::Zoom;
        }
        ui.add_space(2.);
        if tool_icon_btn(ui, "📏", app.active_tool == crate::models::DesignerTool::Ruler, "Ruler Guides", "M", "Measurement guides and snap indicators.").clicked() {
            app.active_tool = crate::models::DesignerTool::Ruler;
        }

        ui.add_space(8.);
        ui.separator();
        ui.add_space(4.);

        // Pages quick-add button
        let page_count = app.project_doc.root_node_ids.len();
        let pages_btn = tool_icon_btn(ui, "📄", false, "Add Page", "P", "Add a new blank canvas page.");
        if pages_btn.clicked() {
            let page_num = page_count + 1;
            let page_id = format!("page-{}", page_num);
            let page = scene::Node {
                id: page_id.clone(),
                name: format!("Page {}", page_num),
                node_type: scene::NodeType::Frame,
                parent_id: None,
                children_ids: vec![],
                position: (40., 60.),
                visible: true,
                locked: false,
                z: 0,
                styling: scene::Styling::default(),
                style: scene::NodeStyle::default(),
                layout: scene::Layout {
                    width: scene::Sizing::Fixed(500.),
                    height: scene::Sizing::Fixed(400.),
                    ..Default::default()
                },
            };
            let _ = app.project_doc.add_node(page, None);
            app.toast(format!("Page '{}' added ✓", page_id));
        }
    });
}

pub fn show_central(app: &mut ProteusApp, ctx: &egui::Context, ui: &mut egui::Ui, pnt: &egui::Painter, r: Rect, mpos: Option<Pos2>, resp: &Response) {
    let canvas_origin = r.left_top();

    // Palette click (fallback) → spawn at viewport center
    if let Some(nt) = app.palette_drag.take() {
        let center = r.center();
        let world_center = app.viewport.screen_to_world(center, canvas_origin);
        app.spawn_designer_node(nt, world_center);
    }

    // Viewport input
    let hover_canvas = mpos.map(|p| r.contains(p)).unwrap_or(false);

    let scroll = ui.input(|i| i.raw_scroll_delta);
    if scroll.y != 0. && hover_canvas {
        if let Some(cursor) = mpos {
            let world_before = app.viewport.screen_to_world(cursor, canvas_origin);
            app.viewport.zoom = (app.viewport.zoom * (1.0 + scroll.y * 0.002)).clamp(0.1, 5.0);
            app.viewport.pan.x = cursor.x - canvas_origin.x - world_before.x * app.viewport.zoom;
            app.viewport.pan.y = cursor.y - canvas_origin.y - world_before.y * app.viewport.zoom;
        } else {
            app.viewport.zoom = (app.viewport.zoom * (1.0 + scroll.y * 0.002)).clamp(0.1, 5.0);
        }
    }

    let middle = ui.input(|i| i.pointer.middle_down());
    let space = ui.input(|i| i.key_down(egui::Key::Space));
    let mdown = ui.input(|i| i.pointer.primary_down());
    let is_panning_input = middle || (space && mdown);
    let is_panning = is_panning_input && hover_canvas;

    if is_panning {
        let delta = ui.input(|i| i.pointer.delta());
        app.viewport.pan += delta;
    }

    // Render dot-matrix grid
    if app.layout == LayoutMode::Grid {
        let world_tl = app.viewport.screen_to_world(r.left_top(), canvas_origin);
        let world_br = app.viewport.screen_to_world(r.right_bottom(), canvas_origin);
        let start_x = (world_tl.x / GRID).floor() * GRID;
        let start_y = (world_tl.y / GRID).floor() * GRID;
        let mut wy = start_y;
        while wy <= world_br.y {
            let sy = app.viewport.world_to_screen(egui::pos2(0., wy), canvas_origin).y;
            pnt.line_segment([egui::pos2(r.left(), sy), egui::pos2(r.right(), sy)],
                Stroke::new(1., Color32::from_rgba_premultiplied(30, 30, 30, 255)));
            wy += GRID;
        }
        let mut wx = start_x;
        while wx <= world_br.x {
            let sx = app.viewport.world_to_screen(egui::pos2(wx, 0.), canvas_origin).x;
            pnt.line_segment([egui::pos2(sx, r.top()), egui::pos2(sx, r.bottom())],
                Stroke::new(1., Color32::from_rgba_premultiplied(30, 30, 30, 255)));
            wx += GRID;
        }
    } else {
        let world_tl = app.viewport.screen_to_world(r.left_top(), canvas_origin);
        let world_br = app.viewport.screen_to_world(r.right_bottom(), canvas_origin);
        let start_x = (world_tl.x / 24.0).floor() * 24.0;
        let start_y = (world_tl.y / 24.0).floor() * 24.0;
        let mut wy = start_y;
        while wy <= world_br.y {
            let mut wx = start_x;
            while wx <= world_br.x {
                let sp = app.viewport.world_to_screen(egui::pos2(wx, wy), canvas_origin);
                if r.contains(sp) {
                    pnt.circle_filled(sp, (1.0_f32).max(0.5 * app.viewport.zoom), Color32::from_rgb(25, 25, 25));
                }
                wx += 24.0;
            }
            wy += 24.0;
        }
    }

    // Device frame overlay
    {
        let (dw, dh) = app.device_preset.size();
        let dev_origin = app.viewport.world_to_screen(egui::pos2(0., 0.), canvas_origin);
        let dev_w = dw * app.viewport.zoom;
        let dev_h = dh * app.viewport.zoom;
        let dev_rect = Rect::from_min_size(dev_origin, egui::vec2(dev_w, dev_h));
        let outside_rects = [
            Rect::from_min_max(egui::pos2(r.left(), r.top()), egui::pos2(r.right(), dev_rect.top())),
            Rect::from_min_max(egui::pos2(r.left(), dev_rect.bottom()), egui::pos2(r.right(), r.bottom())),
            Rect::from_min_max(egui::pos2(r.left(), dev_rect.top()), egui::pos2(dev_rect.left(), dev_rect.bottom())),
            Rect::from_min_max(egui::pos2(dev_rect.right(), dev_rect.top()), egui::pos2(r.right(), dev_rect.bottom())),
        ];
        for or in &outside_rects {
            pnt.rect_filled(*or, 0, Color32::from_black_alpha(80));
        }
        pnt.rect_stroke(dev_rect, 4., Stroke::new(2., theme::ACCENT), egui::StrokeKind::Outside);
        let label = format!("{} {} — {}×{}", app.device_preset.icon(), app.device_preset.label(), dw, dh);
        pnt.text(egui::pos2(dev_rect.left() + 8., dev_rect.top() - 16.), egui::Align2::LEFT_BOTTOM,
            &label, egui::FontId::proportional(10.), theme::ACCENT);
    }

    // Keyboard shortcuts
    let ctrl = ctx.input(|i| i.modifiers.command || i.modifiers.ctrl);
    let shift = ctx.input(|i| i.modifiers.shift);

    // Delete on keyboard shortcut
    let delete_pressed = ctx.input(|i| i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace));
    if delete_pressed && !ui.ctx().wants_keyboard_input() {
        if let Some(id) = app.designer_selected_node.take() {
            app.push_undo();
            app.project_doc.delete_node(&id);
            app.editor_state.selected_node_ids.clear();
            app.toast("Node deleted");
        }
    }

    // Undo / Redo shortcuts
    if ctrl && !ui.ctx().wants_keyboard_input() {
        if ctx.input(|i| i.key_pressed(egui::Key::Z)) {
            if shift {
                app.redo();
            } else {
                app.undo();
            }
        } else if ctx.input(|i| i.key_pressed(egui::Key::Y)) {
            app.redo();
        }
    }

    // Duplicate (Ctrl+D)
    if ctrl && ctx.input(|i| i.key_pressed(egui::Key::D)) && !ui.ctx().wants_keyboard_input() {
        if let Some(id) = app.designer_selected_node.clone() {
            if let Some(node) = app.project_doc.get_node(&id).cloned() {
                app.push_undo();
                app.spawn_counter += 1;
                let new_id = format!("node-{}", app.spawn_counter);
                let mut dup = node;
                dup.id = new_id.clone();
                dup.name = format!("{} (Copy)", dup.name);
                dup.position.0 += 20.0;
                dup.position.1 += 20.0;
                dup.children_ids.clear();
                let _ = app.project_doc.add_node(dup, None);
                app.designer_selected_node = Some(new_id.clone());
                app.editor_state.selected_node_ids = vec![new_id];
                app.toast("Duplicated (Ctrl+D) ✓");
            }
        }
    }

    // Copy (Ctrl+C)
    if ctrl && ctx.input(|i| i.key_pressed(egui::Key::C)) && !ui.ctx().wants_keyboard_input() {
        if let Some(id) = app.designer_selected_node.as_ref() {
            if let Some(node) = app.project_doc.get_node(id).cloned() {
                app.editor_state.clipboard = Some(vec![node]);
                app.toast("Copied to clipboard (Ctrl+C)");
            }
        }
    }

    // Paste (Ctrl+V)
    if ctrl && ctx.input(|i| i.key_pressed(egui::Key::V)) && !ui.ctx().wants_keyboard_input() {
        if let Some(clipboard) = app.editor_state.clipboard.clone() {
            for node in clipboard {
                app.push_undo();
                app.spawn_counter += 1;
                let new_id = format!("node-{}", app.spawn_counter);
                let mut pasted = node;
                pasted.id = new_id.clone();
                pasted.name = format!("{} (Pasted)", pasted.name);
                pasted.position.0 += 24.0;
                pasted.position.1 += 24.0;
                pasted.children_ids.clear();
                let _ = app.project_doc.add_node(pasted, None);
                app.designer_selected_node = Some(new_id.clone());
                app.editor_state.selected_node_ids = vec![new_id];
                app.toast("Pasted (Ctrl+V) ✓");
            }
        }
    }

    // Escape (Deselect)
    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) && !ui.ctx().wants_keyboard_input() {
        app.designer_selected_node = None;
        app.editor_state.selected_node_ids.clear();
    }

    // Arrow keys nudge
    if !ui.ctx().wants_keyboard_input() {
        let left = ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft));
        let right = ctx.input(|i| i.key_pressed(egui::Key::ArrowRight));
        let up = ctx.input(|i| i.key_pressed(egui::Key::ArrowUp));
        let down = ctx.input(|i| i.key_pressed(egui::Key::ArrowDown));
        if left || right || up || down {
            if let Some(id) = app.designer_selected_node.clone() {
                let step = if shift { 10.0 } else { 1.0 };
                let dx = if left { -step } else if right { step } else { 0.0 };
                let dy = if up { -step } else if down { step } else { 0.0 };
                let _ = app.project_doc.move_node(&id, (dx, dy));
            }
        }
    }

    // Render arena document
    let arena_events = renderer::draw_document(ui, &app.project_doc, &app.viewport, &app.editor_state, false, &mut app.form_state, None, &app.table_cache);

    let mut is_resizing = false;
    for ev in &arena_events {
        if let CanvasEvent::NodeResizeStarted = ev {
            is_resizing = true;
        }
    }

    // Designer mouse interaction
    if !is_panning {
        let pointer = ui.input(|i| i.pointer.clone());
        if let Some(mp) = mpos {
            let wm = app.viewport.screen_to_world(mp, canvas_origin);
            let wm = (wm.x, wm.y);

            let is_draw_tool = matches!(
                app.active_tool,
                crate::models::DesignerTool::Rectangle
                    | crate::models::DesignerTool::Text
                    | crate::models::DesignerTool::Button
                    | crate::models::DesignerTool::Table
            );

            if is_draw_tool && hover_canvas {
                ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Crosshair);
                let tool_icon = match app.active_tool {
                    crate::models::DesignerTool::Rectangle => "▢",
                    crate::models::DesignerTool::Text => "T",
                    crate::models::DesignerTool::Button => "🔘",
                    crate::models::DesignerTool::Table => "⊞",
                    _ => "",
                };
                // Floating tool badge offset from cursor
                let badge_rect = Rect::from_min_size(Pos2::new(mp.x + 12., mp.y + 14.), Vec2::new(20., 20.));
                pnt.rect_filled(badge_rect, 4.0, Color32::from_rgba_unmultiplied(22, 26, 32, 230));
                pnt.rect_stroke(badge_rect, 4.0, Stroke::new(1.0, theme::ACCENT), egui::StrokeKind::Outside);
                pnt.text(badge_rect.center(), egui::Align2::CENTER_CENTER, tool_icon, egui::FontId::proportional(11.), theme::ACCENT);

                if pointer.primary_pressed() {
                    app.draw_start = Some(mp);
                    app.designer_selected_node = None;
                    app.editor_state.selected_node_ids.clear();
                }
            }

            if is_draw_tool {
                if let (Some(start), true) = (app.draw_start, pointer.primary_down()) {
                    let drag_rect = Rect::from_two_pos(start, mp);
                    pnt.rect_filled(drag_rect, 4.0, Color32::from_rgba_unmultiplied(79, 140, 237, 28));
                    pnt.rect_stroke(drag_rect, 4.0, Stroke::new(1.5, theme::ACCENT), egui::StrokeKind::Outside);

                    let ws = app.viewport.screen_to_world(start, canvas_origin);
                    let we = app.viewport.screen_to_world(mp, canvas_origin);
                    let dw = (we.x - ws.x).abs();
                    let dh = (we.y - ws.y).abs();
                    let dim_text = format!("{:.0} × {:.0}", dw, dh);
                    let dim_pos = Pos2::new(drag_rect.max.x + 8., drag_rect.max.y + 4.);
                    let dim_rect = Rect::from_min_size(dim_pos, Vec2::new(64., 18.));
                    pnt.rect_filled(dim_rect, 3.0, Color32::from_rgba_unmultiplied(20, 24, 30, 220));
                    pnt.text(dim_rect.center(), egui::Align2::CENTER_CENTER, &dim_text, egui::FontId::proportional(10.), Color32::WHITE);
                }

                if !pointer.primary_down() && app.draw_start.is_some() {
                    if let Some(start) = app.draw_start.take() {
                        let ws = app.viewport.screen_to_world(start, canvas_origin);
                        let we = app.viewport.screen_to_world(mp, canvas_origin);
                        let min_x = ws.x.min(we.x);
                        let min_y = ws.y.min(we.y);
                        let dw = (we.x - ws.x).abs();
                        let dh = (we.y - ws.y).abs();

                        let (pos, size) = if dw < 12.0 && dh < 12.0 {
                            match app.active_tool {
                                crate::models::DesignerTool::Button => ((ws.x, ws.y), (120., 36.)),
                                crate::models::DesignerTool::Text => ((ws.x, ws.y), (120., 30.)),
                                crate::models::DesignerTool::Table => ((ws.x, ws.y), (360., 160.)),
                                _ => ((ws.x, ws.y), (200., 140.)),
                            }
                        } else {
                            ((min_x, min_y), (dw.max(30.), dh.max(20.)))
                        };

                        let nt = match app.active_tool {
                            crate::models::DesignerTool::Button => scene::NodeType::Button {
                                label: "Action".into(),
                                style: scene::ButtonStyle::Primary,
                            },
                            crate::models::DesignerTool::Text => scene::NodeType::Text {
                                content: "Label".into(),
                                font: scene::FontSpec::default(),
                            },
                            crate::models::DesignerTool::Table => scene::NodeType::Table {
                                bound_entity: Some("contacts".into()),
                                columns: vec!["ID".into(), "Name".into(), "Email".into(), "Status".into()],
                            },
                            _ => scene::NodeType::Frame,
                        };

                        app.spawn_designer_node_with_size(nt, pos, size);
                        app.active_tool = crate::models::DesignerTool::Select;
                    }
                }
            } else {
                // Right click hit-testing (auto-select on right click)
                if pointer.secondary_clicked() && hover_canvas {
                    if let Some(hit_id) = scene::hit_test_nodes(&app.project_doc, wm, app.viewport.zoom) {
                        app.designer_selected_node = Some(hit_id.clone());
                        app.editor_state.selected_node_ids = vec![hit_id];
                    }
                }

                if pointer.primary_pressed() && hover_canvas {
                    let hit = scene::hit_test_nodes(&app.project_doc, wm, app.viewport.zoom);
                    if !is_resizing {
                        if let Some(hit_id) = hit {
                            app.designer_selected_node = Some(hit_id.clone());
                            app.designer_drag_active = true;
                            if let Some(node) = app.project_doc.nodes.get(&hit_id) {
                                app.designer_drag_offset = (node.position.0 - wm.0, node.position.1 - wm.1);
                            }
                            app.editor_state.selected_node_ids = vec![hit_id];
                            app.marquee_start = None;
                        } else {
                            app.designer_selected_node = None;
                            app.designer_drag_active = false;
                            app.editor_state.selected_node_ids.clear();
                            if app.active_tool == crate::models::DesignerTool::Select {
                                app.marquee_start = Some(mp);
                            }
                        }
                    }
                }

                if pointer.primary_down() && app.designer_drag_active {
                    if let Some(ref sel_id) = app.designer_selected_node.clone() {
                        let new_x = wm.0 + app.designer_drag_offset.0;
                        let new_y = wm.1 + app.designer_drag_offset.1;
                        let _ = app.project_doc.update_node(sel_id, scene::NodeUpdate::Move { x: new_x, y: new_y });
                    }
                }

                if pointer.primary_down() && app.marquee_start.is_some() {
                    if let Some(start) = app.marquee_start {
                        let sel_rect = Rect::from_two_pos(start, mp);
                        pnt.rect_filled(sel_rect, 0.0, Color32::from_rgba_unmultiplied(0, 120, 215, 35));
                        pnt.rect_stroke(sel_rect, 0.0, Stroke::new(1.0, Color32::from_rgb(0, 150, 255)), egui::StrokeKind::Outside);

                        let w_start = app.viewport.screen_to_world(start, canvas_origin);
                        let w_end = app.viewport.screen_to_world(mp, canvas_origin);
                        let world_box = Rect::from_two_pos(w_start, w_end);

                        let mut matched = Vec::new();
                        for (nid, n) in &app.project_doc.nodes {
                            let (nw, nh) = match (&n.layout.width, &n.layout.height) {
                                (scene::Sizing::Fixed(w), scene::Sizing::Fixed(h)) => (*w, *h),
                                (scene::Sizing::Fixed(w), _) => (*w, 100.),
                                (_, scene::Sizing::Fixed(h)) => (200., *h),
                                _ => (200., 100.),
                            };
                            let n_rect = Rect::from_min_size(Pos2::new(n.position.0, n.position.1), Vec2::new(nw, nh));
                            if world_box.intersects(n_rect) {
                                matched.push(nid.clone());
                            }
                        }
                        if !matched.is_empty() {
                            app.designer_selected_node = matched.first().cloned();
                        }
                        app.editor_state.selected_node_ids = matched;
                    }
                }

                if !pointer.primary_down() {
                    app.designer_drag_active = false;
                    app.marquee_start = None;
                }
            }
        }
    }

    for ev in &arena_events {
        match ev {
            CanvasEvent::NodeClicked { id, shift_held } => {
                if *shift_held {
                    if let Some(pos) = app.editor_state.selected_node_ids.iter().position(|sid| sid == id) {
                        app.editor_state.selected_node_ids.remove(pos);
                    } else {
                        app.editor_state.selected_node_ids.push(id.clone());
                    }
                } else {
                    app.editor_state.selected_node_ids = vec![id.clone()];
                }
            }
            CanvasEvent::NodeDragged(id, delta) => {
                let _ = app.project_doc.move_node(id, *delta);
                if app.editor_state.selected_node_ids.contains(id) {
                    let others: Vec<String> = app.editor_state.selected_node_ids.iter()
                        .filter(|sid| *sid != id)
                        .cloned()
                        .collect();
                    for other_id in &others {
                        let _ = app.project_doc.move_node(other_id, *delta);
                    }
                }
            }
            CanvasEvent::NodeResized { id, handle, delta } => {
                let _ = app.project_doc.resize_node(id, *handle, *delta);
            }
            CanvasEvent::NodeModified { id, update } => {
                let _ = app.project_doc.update_node(id, update.clone());
            }
            CanvasEvent::NodeResizeStarted => {}
            CanvasEvent::ClearSelection => {
                app.editor_state.selected_node_ids.clear();
            }
            CanvasEvent::ActionTriggered { .. } => {}
            CanvasEvent::DeleteNode { id } => {
                app.project_doc.delete_node(id);
                app.designer_selected_node = None;
                app.editor_state.selected_node_ids.clear();
                app.toast("Node deleted");
            }
            CanvasEvent::SetButtonAction { button_id, target_page, submit_entity } => {
                app.project_doc.set_button_action(button_id, target_page.clone(), submit_entity.clone());
            }
        }
    }

    // Viewport info
    pnt.text(egui::pos2(r.left() + 8., r.bottom() - 4.), egui::Align2::LEFT_BOTTOM,
        &format!("Zoom: {:.0}%  |  Pan: ({:.0}, {:.0})  |  {} nodes",
            app.viewport.zoom * 100., app.viewport.pan.x, app.viewport.pan.y, app.project_doc.nodes.len()),
        egui::FontId::proportional(9.), theme::TEXT_DIM);

    if app.project_doc.nodes.is_empty() {
        pnt.text(r.center(), egui::Align2::CENTER_CENTER,
            "Click widgets from the left panel to start designing",
            egui::FontId::proportional(14.), theme::TEXT_DIM);
    }

    // Right-click context menu
    resp.context_menu(|ui| {
        if let Some(ref sel_id) = app.designer_selected_node.clone() {
            let node_name = app.project_doc.nodes.get(sel_id)
                .map(|n| n.name.as_str())
                .unwrap_or("Element");
            ui.label(egui::RichText::new(format!("SELECTED: {}", node_name)).size(10.).color(theme::ACCENT).strong());
            ui.separator();

            if ui.button("📋 Copy Dimensions").clicked() {
                if let Some(n) = app.project_doc.nodes.get(sel_id) {
                    let (w, h) = match (&n.layout.width, &n.layout.height) {
                        (scene::Sizing::Fixed(w), scene::Sizing::Fixed(h)) => (*w, *h),
                        (scene::Sizing::Fixed(w), _) => (*w, 100.),
                        (_, scene::Sizing::Fixed(h)) => (200., *h),
                        _ => (200., 100.),
                    };
                    app.copied_dimensions = Some((w, h));
                    app.toast("Dimensions copied ✓");
                }
                ui.close_menu();
            }

            let can_paste = app.copied_dimensions.is_some();
            if ui.add_enabled(can_paste, egui::Button::new("📌 Paste Dimensions")).clicked() {
                if let Some((w, h)) = app.copied_dimensions {
                    let _ = app.project_doc.update_node(sel_id, scene::NodeUpdate::Dimensions { width: w, height: h });
                    app.toast("Dimensions applied ✓");
                }
                ui.close_menu();
            }

            if ui.button("🔒 Toggle Lock").clicked() {
                let _ = app.project_doc.update_node(sel_id, scene::NodeUpdate::ToggleLock);
                ui.close_menu();
            }
            if ui.button("👁 Toggle Visibility").clicked() {
                let _ = app.project_doc.update_node(sel_id, scene::NodeUpdate::ToggleVisibility);
                ui.close_menu();
            }
            if ui.button(egui::RichText::new("🗑 Delete").color(Color32::from_rgb(255, 100, 100))).clicked() {
                app.project_doc.delete_node(sel_id);
                app.designer_selected_node = None;
                app.editor_state.selected_node_ids.clear();
                app.toast("Node deleted");
                ui.close_menu();
            }
            ui.separator();
        }

        ui.label(egui::RichText::new("CANVAS TOOLS").size(9.).color(theme::TEXT_DIM));
        if ui.button("▢ Smart Box (R)").clicked() {
            app.active_tool = crate::models::DesignerTool::Rectangle;
            ui.close_menu();
        }
        if ui.button("T Text Label (T)").clicked() {
            app.active_tool = crate::models::DesignerTool::Text;
            ui.close_menu();
        }
        if ui.button("🔘 Action Button (B)").clicked() {
            app.active_tool = crate::models::DesignerTool::Button;
            ui.close_menu();
        }
    });
}

pub fn show_right(app: &mut ProteusApp, ui: &mut egui::Ui) {
    let events = inspector::draw_inspector(
        ui,
        &app.project_doc,
        &app.editor_state,
        &mut app.copied_dimensions,
    );
    for ev in events {
        match ev {
            scene::CanvasEvent::NodeModified { id, update } => {
                let _ = app.project_doc.update_node(&id, update);
            }
            scene::CanvasEvent::DeleteNode { id } => {
                app.push_undo();
                app.project_doc.delete_node(&id);
                if app.designer_selected_node.as_deref() == Some(&id) {
                    app.designer_selected_node = None;
                    app.editor_state.selected_node_ids.clear();
                }
                app.toast("Node deleted");
            }
            scene::CanvasEvent::NodeClicked { id, shift_held } => {
                if shift_held {
                    if let Some(pos) = app.editor_state.selected_node_ids.iter().position(|sid| sid == &id) {
                        app.editor_state.selected_node_ids.remove(pos);
                    } else {
                        app.editor_state.selected_node_ids.push(id.clone());
                    }
                } else {
                    app.designer_selected_node = Some(id.clone());
                    app.editor_state.selected_node_ids = vec![id];
                }
            }
            scene::CanvasEvent::SetButtonAction { button_id, target_page, submit_entity } => {
                app.project_doc.set_button_action(&button_id, target_page.clone(), submit_entity.clone());
                let msg = match (&target_page, &submit_entity) {
                    (Some(p), Some(e)) => format!("Action: Submit '{}' & Go to '{}'", e, p),
                    (Some(p), None) => format!("Action: Go to screen '{}'", p),
                    (None, Some(e)) => format!("Action: Submit to '{}'", e),
                    (None, None) => "Action cleared".into(),
                };
                app.toast(msg);
            }
            _ => {}
        }
    }
}

