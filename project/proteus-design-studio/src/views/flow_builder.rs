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
        ("+ Condition (IF)", crate::flow::FlowNodeKind::Condition {
            field: "status".to_string(),
            operator: "==".to_string(),
            target_value: "active".to_string(),
        }),
        ("+ Notification", crate::flow::FlowNodeKind::ShowToast { message: "Action executed".to_string() }),
        ("+ Federation Bridge", crate::flow::FlowNodeKind::FederationBridge {
            endpoint: "http://partner-db:7443".to_string(),
            source_entity: "products".to_string(),
            target_entity: "inventory".to_string(),
            bidirectional: true,
        }),
        ("+ Relay Actuator", crate::flow::FlowNodeKind::HardwareRelay {
            device_id: "RELAY-01".to_string(),
            channel: 1,
            action: "pulse".to_string(),
            pulse_ms: 500,
        }),
        ("+ Sensor Watchdog", crate::flow::FlowNodeKind::HardwareSensor {
            device_id: "SENSOR-01".to_string(),
            metric: "temperature_c".to_string(),
            threshold: 30.0,
            operator: ">".to_string(),
        }),
        ("+ Scale Weigh", crate::flow::FlowNodeKind::HardwareScale {
            device_id: "SCALE-01".to_string(),
            require_stable: true,
        }),
        ("+ VFD Display", crate::flow::FlowNodeKind::HardwareDisplay {
            device_id: "VFD-01".to_string(),
            line1: "TOTAL: 0.00 EUR".to_string(),
            line2: "THANK YOU".to_string(),
        }),
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
            app.project_doc.flow_graph.nodes.insert(id.clone(), crate::flow::FlowNode::new(id.clone(), kind, pos));
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
            let mut port_hit: Option<(String, String)> = None;
            if let Some((node_id, port_id, crate::flow::PortDirection::Output)) =
                flow_renderer::find_port_at_world_pos(&app.project_doc.flow_graph, world_pos, 12.0)
            {
                port_hit = Some((node_id.to_string(), port_id.to_string()));
            } else {
                for node in app.project_doc.flow_graph.nodes.values() {
                    if flow_renderer::is_over_output_port(node.position, world_pos) {
                        let p_id = node.output_ports.first().map(|p| p.id.as_str()).unwrap_or("exec_out");
                        port_hit = Some((node.id.clone(), p_id.to_string()));
                        break;
                    }
                }
            }
            if let Some((src_id, port_id)) = port_hit {
                app.flow_wiring_source = Some(format!("{}#{}", src_id, port_id));
                app.flow_wiring_pos = Some(world_pos);
            } else {
                // Node body hit → select and drag
                let mut hit: Option<String> = None;
                for node in app.project_doc.flow_graph.nodes.values() {
                    let nx = node.position.0;
                    let ny = node.position.1;
                    let nh = flow_renderer::node_height(node);
                    if world.x >= nx && world.x <= nx + flow_renderer::NODE_W
                        && world.y >= ny && world.y <= ny + nh
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
                    // Check Bezier edge hit → click-to-delete
                    let mut edge_to_remove: Option<usize> = None;
                    for (idx, edge) in app.project_doc.flow_graph.edges.iter().enumerate() {
                        let from_node = match app.project_doc.flow_graph.nodes.get(&edge.from_node) {
                            Some(n) => n,
                            None => continue,
                        };
                        let to_node = match app.project_doc.flow_graph.nodes.get(&edge.to_node) {
                            Some(n) => n,
                            None => continue,
                        };
                        let (fx, fy) = flow_renderer::port_world_position(from_node, &edge.from_port)
                            .unwrap_or_else(|| flow_renderer::node_output_port(from_node.position));
                        let (tx, ty) = flow_renderer::port_world_position(to_node, &edge.to_port)
                            .unwrap_or_else(|| flow_renderer::node_input_port(to_node.position));
                        let curve = crate::flow::CubicBezierCurve::from_endpoints((fx, fy), (tx, ty));
                        if curve.distance_to_point(world_pos, 24) <= 8.0 {
                            edge_to_remove = Some(idx);
                            break;
                        }
                    }
                    if let Some(idx) = edge_to_remove {
                        app.project_doc.flow_graph.edges.remove(idx);
                        app.toast("Wire removed");
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
        if let Some(src_str) = app.flow_wiring_source.clone() {
            let (src_id, src_port) = match src_str.split_once('#') {
                Some((n, p)) => (n.to_string(), p.to_string()),
                None => (src_str.clone(), "exec_out".to_string()),
            };
            let mut target_hit: Option<(String, String)> = None;
            if let Some(cursor) = mpos {
                let world = app.flow_viewport.screen_to_world(cursor, canvas_origin);
                let world_pos = (world.x, world.y);

                // 1. Precise port snap
                if let Some((tgt_node, tgt_port, crate::flow::PortDirection::Input)) =
                    flow_renderer::find_port_at_world_pos(&app.project_doc.flow_graph, world_pos, 14.0)
                {
                    if tgt_node != src_id {
                        target_hit = Some((tgt_node.to_string(), tgt_port.to_string()));
                    }
                }

                // 2. Fallback to node body
                if target_hit.is_none() {
                    for node in app.project_doc.flow_graph.nodes.values() {
                        if node.id == src_id {
                            continue;
                        }
                        let nx = node.position.0;
                        let ny = node.position.1;
                        let nh = flow_renderer::node_height(node);
                        if world.x >= nx && world.x <= nx + flow_renderer::NODE_W
                            && world.y >= ny && world.y <= ny + nh
                        {
                            let def_in = node.input_ports.first().map(|p| p.id.clone()).unwrap_or_else(|| "exec_in".to_string());
                            target_hit = Some((node.id.clone(), def_in));
                            break;
                        }
                    }
                }
            }

            if let Some((tgt_id, tgt_port)) = target_hit {
                if let Ok(()) = app.project_doc.flow_graph.can_connect(&src_id, &src_port, &tgt_id, &tgt_port) {
                    let branch = if src_port == "branch_true" {
                        Some("true".to_string())
                    } else if src_port == "branch_false" {
                        Some("false".to_string())
                    } else {
                        None
                    };
                    let mut edge = crate::flow::FlowEdge::with_ports(&src_id, &src_port, &tgt_id, &tgt_port);
                    edge.branch = branch;
                    app.project_doc.flow_graph.edges.push(edge);
                    app.toast("Wire connected");
                } else {
                    app.toast("Incompatible or duplicate wire");
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
                crate::flow::FlowNodeKind::Condition { ref mut field, ref mut operator, ref mut target_value } => {
                    ui.label(egui::RichText::new("Target Field:").size(10.).color(theme::TEXT));
                    ui.add_space(2.);
                    ui.add(egui::TextEdit::singleline(field).hint_text("e.g. status or amount").desired_width(f32::INFINITY));
                    ui.add_space(4.);
                    ui.label(egui::RichText::new("Operator:").size(10.).color(theme::TEXT));
                    ui.add_space(2.);
                    ui.add(egui::TextEdit::singleline(operator).hint_text("==, !=, >, <, contains").desired_width(f32::INFINITY));
                    ui.add_space(4.);
                    ui.label(egui::RichText::new("Compare Value:").size(10.).color(theme::TEXT));
                    ui.add_space(2.);
                    ui.add(egui::TextEdit::singleline(target_value).hint_text("e.g. active or 100").desired_width(f32::INFINITY));
                }
                crate::flow::FlowNodeKind::ShowToast { ref mut message } => {
                    ui.label(egui::RichText::new("Notification Message:").size(10.).color(theme::TEXT));
                    ui.add_space(2.);
                    ui.add(egui::TextEdit::singleline(message).hint_text("e.g. Record saved successfully").desired_width(f32::INFINITY));
                }
                crate::flow::FlowNodeKind::FederationBridge {
                    ref mut endpoint,
                    ref mut source_entity,
                    ref mut target_entity,
                    ref mut bidirectional,
                } => {
                    ui.label(egui::RichText::new("Remote Endpoint:").size(10.).color(theme::TEXT));
                    ui.add_space(2.);
                    ui.add(egui::TextEdit::singleline(endpoint).hint_text("e.g. http://partner-db:7443").desired_width(f32::INFINITY));
                    ui.add_space(4.);
                    ui.label(egui::RichText::new("Source Table / Entity:").size(10.).color(theme::TEXT));
                    ui.add_space(2.);
                    ui.add(egui::TextEdit::singleline(source_entity).hint_text("e.g. products").desired_width(f32::INFINITY));
                    ui.add_space(4.);
                    ui.label(egui::RichText::new("Target Table / Entity:").size(10.).color(theme::TEXT));
                    ui.add_space(2.);
                    ui.add(egui::TextEdit::singleline(target_entity).hint_text("e.g. inventory").desired_width(f32::INFINITY));
                    ui.add_space(6.);
                    ui.checkbox(bidirectional, egui::RichText::new("Bidirectional Sync (LWW)").size(10.).color(theme::TEXT));
                }
                other => {
                    crate::flow::hardware_nodes::render_hardware_inspector(ui, other);
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
