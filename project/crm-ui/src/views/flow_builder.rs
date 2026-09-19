use eframe::egui::{self, Color32, Pos2, Rect};
use crate::theme;
use crate::ProteusApp;
use crate::flow_renderer;

pub fn show_left(app: &mut ProteusApp, ui: &mut egui::Ui) {
    ui.add_space(6.);
    ui.label(egui::RichText::new("⚡ FLOW BUILDER").size(9.).color(theme::ACCENT));
    ui.add_space(4.);
    ui.label(egui::RichText::new("Add Nodes:").size(9.).color(theme::TEXT_DIM));
    ui.add_space(2.);
    let kinds: &[(&str, crate::flow::FlowNodeKind)] = &[
        ("+ Trigger Click", crate::flow::FlowNodeKind::TriggerClick { target_node_id: String::new() }),
        ("+ Navigate To",  crate::flow::FlowNodeKind::NavigateTo { page_id: String::new() }),
        ("+ Save To DB",   crate::flow::FlowNodeKind::SaveToDatabase { entity: String::new() }),
    ];
    for (label, kind_tpl) in kinds {
        if ui.add(egui::Button::new(egui::RichText::new(*label).size(10.).color(theme::ACCENT_GREEN))
            .fill(theme::WIDGET_BG).min_size(egui::vec2(ui.available_width(), 22.))).clicked()
        {
            let id = uuid::Uuid::new_v4().to_string();
            let kind = kind_tpl.clone();
            let pos = if app.flow_canvas_center == (100., 100.) {
                (300. + app.project_doc.flow_graph.nodes.len() as f32 * 40., 150.)
            } else {
                app.flow_canvas_center
            };
            app.project_doc.flow_graph.nodes.insert(id.clone(), crate::flow::FlowNode {
                id: id.clone(), kind, position: pos,
            });
            app.selected_flow_node = Some(id);
            app.toast("Flow node added");
        }
    }
    ui.add_space(8.);
    ui.label(egui::RichText::new("Nodes in graph:").size(9.).color(theme::TEXT_DIM));
    let count = app.project_doc.flow_graph.nodes.len();
    ui.label(egui::RichText::new(count.to_string()).size(11.).color(theme::ACCENT));
}

pub fn show_central(app: &mut ProteusApp, ctx: &egui::Context, ui: &mut egui::Ui, pnt: &egui::Painter, r: Rect, mpos: Option<Pos2>, mdown: bool) {
    let canvas_origin = r.left_top();
    app.flow_canvas_center = app.flow_viewport.screen_to_world(r.center(), canvas_origin).into();

    // Viewport pan (middle mouse or Space+drag)
    let scroll = ui.input(|i| i.raw_scroll_delta);
    if scroll.y != 0. {
        if let Some(cursor) = mpos {
            let world_before = app.flow_viewport.screen_to_world(cursor, canvas_origin);
            app.flow_viewport.zoom = (app.flow_viewport.zoom * (1.0 + scroll.y * 0.002)).clamp(0.1, 5.0);
            app.flow_viewport.pan.x = cursor.x - canvas_origin.x - world_before.x * app.flow_viewport.zoom;
            app.flow_viewport.pan.y = cursor.y - canvas_origin.y - world_before.y * app.flow_viewport.zoom;
        } else {
            app.flow_viewport.zoom = (app.flow_viewport.zoom * (1.0 + scroll.y * 0.002)).clamp(0.1, 5.0);
        }
    }
    let middle = ui.input(|i| i.pointer.middle_down());
    let space = ui.input(|i| i.key_down(egui::Key::Space));
    let is_panning = middle || (space && mdown);
    if is_panning {
        let delta = ui.input(|i| i.pointer.delta());
        app.flow_viewport.pan += delta;
    }

    // Dot-matrix grid
    let world_tl = app.flow_viewport.screen_to_world(r.left_top(), canvas_origin);
    let world_br = app.flow_viewport.screen_to_world(r.right_bottom(), canvas_origin);
    let start_x = (world_tl.x / 24.0).floor() * 24.0;
    let start_y = (world_tl.y / 24.0).floor() * 24.0;
    let mut wy = start_y;
    while wy <= world_br.y {
        let mut wx = start_x;
        while wx <= world_br.x {
            let sp = app.flow_viewport.world_to_screen(egui::pos2(wx, wy), canvas_origin);
            if r.contains(sp) {
                pnt.circle_filled(sp, (1.0_f32).max(0.5 * app.flow_viewport.zoom), Color32::from_rgb(25, 25, 25));
            }
            wx += 24.0;
        }
        wy += 24.0;
    }

    // Render flow graph
    flow_renderer::draw_flow_graph(
        ui, pnt, &app.project_doc.flow_graph,
        &app.flow_viewport, canvas_origin, r,
        app.selected_flow_node.as_deref(),
        app.flow_wiring_source.as_deref(),
        app.flow_wiring_pos,
    );

    // Node interaction (wiring, select, drag)
    let mclick = ui.input(|i| i.pointer.primary_clicked());
    let mreleased = ui.input(|i| i.pointer.primary_released());
    let mdelta = ui.input(|i| i.pointer.delta());

    // Update wiring end position while dragging
    if app.flow_wiring_source.is_some() {
        if let Some(cursor) = mpos {
            let world = app.flow_viewport.screen_to_world(cursor, canvas_origin);
            app.flow_wiring_pos = Some((world.x, world.y));
        }
    }

    if mclick {
        if is_panning {
            app.selected_flow_node = None;
        } else if let Some(cursor) = mpos {
            let world = app.flow_viewport.screen_to_world(cursor, canvas_origin);
            let world_pos = (world.x, world.y);

            // Check output port hit first (wiring)
            let mut port_hit: Option<String> = None;
            for node in app.project_doc.flow_graph.nodes.values() {
                if flow_renderer::is_over_output_port(node.position, world_pos) {
                    port_hit = Some(node.id.clone());
                    break;
                }
            }
            if let Some(src_id) = port_hit {
                app.flow_wiring_source = Some(src_id);
                app.flow_wiring_pos = Some(world_pos);
            } else {
                // Node body hit → select and drag
                let mut hit: Option<String> = None;
                for node in app.project_doc.flow_graph.nodes.values() {
                    let nx = node.position.0;
                    let ny = node.position.1;
                    if world.x >= nx && world.x <= nx + flow_renderer::NODE_W
                        && world.y >= ny && world.y <= ny + flow_renderer::NODE_H
                    {
                        hit = Some(node.id.clone());
                        break;
                    }
                }
                app.selected_flow_node = hit.clone();
                if hit.is_some() {
                    app.flow_drag_active = true;
                } else {
                    app.selected_flow_node = None;
                    // Check edge hit → click-to-delete
                    let mut edge_to_remove: Option<usize> = None;
                    for (idx, edge) in app.project_doc.flow_graph.edges.iter().enumerate() {
                        let from_pos = match app.project_doc.flow_graph.nodes.get(&edge.from_node) {
                            Some(n) => flow_renderer::node_output_port(n.position),
                            None => continue,
                        };
                        let to_pos = match app.project_doc.flow_graph.nodes.get(&edge.to_node) {
                            Some(n) => flow_renderer::node_input_port(n.position),
                            None => continue,
                        };
                        if flow_renderer::is_near_line_segment(world_pos, from_pos, to_pos, 8.0) {
                            edge_to_remove = Some(idx);
                            break;
                        }
                    }
                    if let Some(idx) = edge_to_remove {
                        app.project_doc.flow_graph.edges.remove(idx);
                        app.toast("Edge removed");
                    }
                }
            }
        } else {
            app.selected_flow_node = None;
        }
    }

    if app.flow_drag_active && mdown && !is_panning {
        if let Some(ref sel_id) = app.selected_flow_node.clone() {
            let world_delta = mdelta / app.flow_viewport.zoom;
            if let Some(node) = app.project_doc.flow_graph.nodes.get_mut(sel_id) {
                node.position.0 += world_delta.x;
                node.position.1 += world_delta.y;
            }
        }
    }

    if mreleased {
        if app.flow_wiring_source.is_some() {
            let mut target: Option<String> = None;
            if let Some(cursor) = mpos {
                let world = app.flow_viewport.screen_to_world(cursor, canvas_origin);
                for node in app.project_doc.flow_graph.nodes.values() {
                    let nx = node.position.0;
                    let ny = node.position.1;
                    if world.x >= nx && world.x <= nx + flow_renderer::NODE_W
                        && world.y >= ny && world.y <= ny + flow_renderer::NODE_H
                    {
                        target = Some(node.id.clone());
                        break;
                    }
                }
            }
            if let Some(src) = app.flow_wiring_source.clone() {
                if let Some(tgt) = target {
                    if src != tgt {
                        let already = app.project_doc.flow_graph.edges.iter()
                            .any(|e| e.from_node == src && e.to_node == tgt);
                        if !already {
                            app.project_doc.flow_graph.edges.push(crate::flow::FlowEdge {
                                from_node: src,
                                to_node: tgt,
                            });
                        }
                    }
                }
            }
            app.flow_wiring_source = None;
            app.flow_wiring_pos = None;
        }
        app.flow_drag_active = false;
    }

    // Delete flow node on keyboard shortcut
    if ctx.input(|i| i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace)) {
        if let Some(id) = app.selected_flow_node.take() {
            app.project_doc.flow_graph.remove_node(&id);
            app.toast("Flow node deleted");
        }
    }

    // Status info
    pnt.text(egui::pos2(r.left() + 8., r.bottom() - 4.), egui::Align2::LEFT_BOTTOM,
        format!("Zoom: {:.0}%  |  {} flow nodes, {} edges",
            app.flow_viewport.zoom * 100.,
            app.project_doc.flow_graph.nodes.len(),
            app.project_doc.flow_graph.edges.len(),
        ),
        egui::FontId::proportional(9.), Color32::from_rgb(130, 130, 130));

    if app.project_doc.flow_graph.nodes.is_empty() {
        pnt.text(r.center(), egui::Align2::CENTER_CENTER,
            "No flow nodes yet. Add triggers and actions to build your automation.",
            egui::FontId::proportional(14.), Color32::from_rgb(130, 130, 130));
    }
}

pub fn show_right(app: &mut ProteusApp, ui: &mut egui::Ui) {
    if let Some(sel_id) = &app.selected_flow_node.clone() {
        if let Some(node) = app.project_doc.flow_graph.nodes.get_mut(sel_id) {
            ui.label(egui::RichText::new("NODE CONFIGURATION").size(9.).color(theme::TEXT_DIM));
            ui.add_space(2.);
            ui.label(egui::RichText::new(format!("ID: {}", node.id)).size(9.).color(theme::TEXT_DIM));
            ui.add_space(4.);
            match &mut node.kind {
                crate::flow::FlowNodeKind::TriggerClick { ref mut target_node_id } => {
                    ui.label(egui::RichText::new("Target UI Node ID:").size(10.).color(theme::TEXT));
                    ui.add_space(2.);
                    ui.add(egui::TextEdit::singleline(target_node_id).hint_text("e.g. btn-1").desired_width(f32::INFINITY));
                }
                crate::flow::FlowNodeKind::NavigateTo { ref mut page_id } => {
                    ui.label(egui::RichText::new("Target Page ID:").size(10.).color(theme::TEXT));
                    ui.add_space(2.);
                    ui.add(egui::TextEdit::singleline(page_id).hint_text("e.g. page-2").desired_width(f32::INFINITY));
                }
                crate::flow::FlowNodeKind::SaveToDatabase { ref mut entity } => {
                    ui.label(egui::RichText::new("Database Entity:").size(10.).color(theme::TEXT));
                    ui.add_space(2.);
                    ui.add(egui::TextEdit::singleline(entity).hint_text("e.g. contacts").desired_width(f32::INFINITY));
                }
            }
        }
    } else {
        ui.label(egui::RichText::new("Select a node on the canvas to configure it.").size(10.).color(theme::TEXT_DIM));
    }
    ui.add_space(12.);
    ui.separator();
    ui.add_space(4.);
    let count = app.project_doc.flow_graph.nodes.len();
    let edge_count = app.project_doc.flow_graph.edges.len();
    ui.label(egui::RichText::new(format!("Graph: {} nodes, {} edges", count, edge_count)).size(9.).color(theme::TEXT_DIM));
}
