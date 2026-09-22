//! Designer interaction mechanics: keyboard shortcuts, tool drawing, marquee selection, and context menus.

use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};
use crate::scene::{self, CanvasEvent};
use crate::theme;
use crate::ProteusApp;

pub fn handle_keyboard_shortcuts(app: &mut ProteusApp, ctx: &egui::Context, ui: &egui::Ui) {
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

    // Select All (Ctrl+A)
    if ctrl && ctx.input(|i| i.key_pressed(egui::Key::A)) && !ui.ctx().wants_keyboard_input() {
        let all_ids: Vec<String> = app.project_doc.nodes.keys().cloned().collect();
        app.designer_selected_node = all_ids.first().cloned();
        app.editor_state.selected_node_ids = all_ids;
        app.toast("Selected all elements (Ctrl+A)");
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
}

pub fn handle_drawing_tool(
    app: &mut ProteusApp,
    ui: &mut egui::Ui,
    pnt: &egui::Painter,
    mp: Pos2,
    canvas_origin: Pos2,
    hover_canvas: bool,
) {
    let pointer = ui.input(|i| i.pointer.clone());

    if pointer.secondary_clicked() {
        app.draw_start = None;
        app.active_tool = crate::models::DesignerTool::Select;
        return;
    }

    if hover_canvas {
        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Crosshair);
        let tool_icon = match app.active_tool {
            crate::models::DesignerTool::Rectangle => "▢",
            crate::models::DesignerTool::Text => "T",
            crate::models::DesignerTool::Button => "🔘",
            crate::models::DesignerTool::Table => "⊞",
            _ => "",
        };
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
}

pub fn handle_mouse_selection(
    app: &mut ProteusApp,
    ui: &egui::Ui,
    pnt: &egui::Painter,
    mp: Pos2,
    canvas_origin: Pos2,
    hover_canvas: bool,
    is_resizing: bool,
) {
    let pointer = ui.input(|i| i.pointer.clone());
    let wm = app.viewport.screen_to_world(mp, canvas_origin);
    let wm = (wm.x, wm.y);

    if pointer.secondary_clicked() && hover_canvas {
        if let Some(hit_id) = scene::hit_test_nodes(&app.project_doc, wm, app.viewport.zoom) {
            // Hit-Test First Dispatcher:
            // If already inside active multi-selection, preserve selection.
            // If right-clicking a different element, auto-select it immediately.
            if !app.editor_state.selected_node_ids.contains(&hit_id) {
                app.designer_selected_node = Some(hit_id.clone());
                app.editor_state.selected_node_ids = vec![hit_id];
            }
        } else {
            // Right-clicking empty canvas clears selection so canvas actions are shown
            app.designer_selected_node = None;
            app.editor_state.selected_node_ids.clear();
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

pub fn handle_arena_events(app: &mut ProteusApp, arena_events: &[CanvasEvent]) {
    for ev in arena_events {
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
}

