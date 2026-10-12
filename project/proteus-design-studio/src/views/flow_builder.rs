use eframe::egui::{self, Color32, Pos2, Rect};
use crate::theme;
use crate::ProteusApp;
use crate::flow_renderer;

pub fn show_left(app: &mut ProteusApp, ui: &mut egui::Ui) {
    ui.add_space(6.);
    ui.label(egui::RichText::new("⚡ FLOW BUILDER").size(9.).color(theme::ACCENT));
    ui.add_space(4.);

    if ui.add(egui::Button::new(egui::RichText::new("📂 Import .SQL Script").size(10.5).color(Color32::from_rgb(0, 210, 240)).strong())
        .fill(Color32::from_rgb(20, 36, 48))
        .stroke(egui::Stroke::new(1.0, Color32::from_rgb(0, 168, 204)))
        .min_size(egui::vec2(ui.available_width(), 26.)))
        .clicked()
    {
        app.show_sql_import_modal = true;
        app.sql_import_error = None;
    }

    ui.add_space(4.);
    if ui.add(egui::Button::new(egui::RichText::new("💾 Export Schema (.SQL)").size(10.5).color(theme::TEXT).strong())
        .fill(theme::WIDGET_BG)
        .stroke(egui::Stroke::new(1.0, theme::BORDER))
        .min_size(egui::vec2(ui.available_width(), 26.)))
        .clicked()
    {
        let ddl = app.project_doc.flow_graph.export_sql_schema_ddl();
        if ddl.is_empty() {
            app.toast("No table schema nodes found on canvas");
        } else {
            ui.ctx().copy_text(ddl.clone());
            let vault_path = std::path::Path::new("vault/exported_schema.sql");
            let _ = std::fs::write(vault_path, &ddl);
            app.toast("Schema exported & copied to clipboard ✓");
        }
    }

    ui.add_space(6.);
    ui.separator();
    ui.add_space(4.);
    ui.label(egui::RichText::new("Add Nodes:").size(9.).color(theme::TEXT_DIM));
    ui.add_space(2.);
    let kinds = crate::flow::data_block_ui::flow_palette_kinds();
    for (label, kind_tpl) in &kinds {
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
                    let is_table_relation = src_id.starts_with("tbl_") || tgt_id.starts_with("tbl_")
                        || src_port.starts_with("out_") || tgt_port.starts_with("in_");
                    if is_table_relation {
                        edge.color_hue = Some(crate::flow::FlowEdge::compute_relation_hue(&src_id, &tgt_id));
                    }
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

    // Modal dialog for importing external .SQL relational DDL script
    if app.show_sql_import_modal {
        let mut open = app.show_sql_import_modal;
        egui::Window::new(egui::RichText::new("📂 Import SQL Relational Schema (.SQL)").size(13.).color(theme::ACCENT).strong())
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_width(520.0)
            .default_height(400.0)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.add_space(4.);
                ui.label(egui::RichText::new("Paste a standard SQL DDL script (CREATE TABLE, FOREIGN KEY ...) or load from file. Visual table nodes and multi-colored relation wires will be automatically constructed on the canvas.").size(10.).color(theme::TEXT_DIM));
                ui.add_space(8.);

                ui.label(egui::RichText::new("SQL DDL Script:").size(10.5).color(theme::TEXT));
                ui.add_space(2.);
                egui::ScrollArea::vertical()
                    .max_height(220.0)
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut app.sql_import_buffer)
                                .font(egui::TextStyle::Monospace)
                                .desired_width(f32::INFINITY)
                                .desired_rows(12)
                                .hint_text("CREATE TABLE customers (\n    id INTEGER PRIMARY KEY,\n    name TEXT NOT NULL\n);\n\nCREATE TABLE orders (\n    id TEXT PRIMARY KEY,\n    customer_id INTEGER NOT NULL,\n    FOREIGN KEY (customer_id) REFERENCES customers(id)\n);")
                        );
                    });

                if let Some(ref err) = app.sql_import_error {
                    ui.add_space(4.);
                    ui.label(egui::RichText::new(format!("⚠ {}", err)).size(10.).color(Color32::from_rgb(255, 90, 90)));
                }

                ui.add_space(6.);
                ui.checkbox(
                    &mut app.sql_import_execute_in_db,
                    egui::RichText::new("Execute & create tables in SQLite database (store.db)").size(10.).color(theme::TEXT),
                );
                ui.add_space(8.);
                ui.separator();
                ui.add_space(6.);

                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new(egui::RichText::new("⚡ Parse & Build Nodes").size(11.).color(Color32::BLACK).strong())
                        .fill(theme::ACCENT_GREEN)
                        .min_size(egui::vec2(150., 28.)))
                        .clicked()
                    {
                        match app.import_sql_ddl_into_flow(&app.sql_import_buffer.clone()) {
                            Ok(tables_count) => {
                                app.toast(format!("Successfully generated {} table nodes and relation wires", tables_count));
                                app.show_sql_import_modal = false;
                                app.sql_import_error = None;
                            }
                            Err(e) => {
                                app.sql_import_error = Some(e);
                            }
                        }
                    }

                    if ui.add(egui::Button::new(egui::RichText::new("📄 Load Sample ERP").size(10.5).color(theme::TEXT))
                        .fill(theme::WIDGET_BG)
                        .min_size(egui::vec2(130., 28.)))
                        .clicked()
                    {
                        let sample_path = std::path::Path::new("vault/synthetic_erp.sql");
                        if let Ok(content) = std::fs::read_to_string(sample_path) {
                            app.sql_import_buffer = content;
                        } else {
                            app.sql_import_buffer = String::from(
                                "CREATE TABLE customers (\n    id INTEGER PRIMARY KEY AUTOINCREMENT,\n    vat_number TEXT UNIQUE NOT NULL,\n    company_name TEXT NOT NULL,\n    credit_limit REAL NOT NULL DEFAULT 5000.0\n);\n\nCREATE TABLE products (\n    sku TEXT PRIMARY KEY,\n    name TEXT NOT NULL,\n    unit_price REAL NOT NULL\n);\n\nCREATE TABLE orders (\n    order_number TEXT PRIMARY KEY,\n    customer_id INTEGER NOT NULL,\n    total_amount REAL NOT NULL,\n    FOREIGN KEY (customer_id) REFERENCES customers(id)\n);\n\nCREATE TABLE order_items (\n    id INTEGER PRIMARY KEY AUTOINCREMENT,\n    order_number TEXT NOT NULL,\n    product_sku TEXT NOT NULL,\n    quantity INTEGER NOT NULL,\n    FOREIGN KEY (order_number) REFERENCES orders(order_number),\n    FOREIGN KEY (product_sku) REFERENCES products(sku)\n);\n\nCREATE TABLE invoices (\n    invoice_number TEXT PRIMARY KEY,\n    order_number TEXT NOT NULL,\n    mydata_mark TEXT UNIQUE NOT NULL,\n    total_payable REAL NOT NULL,\n    FOREIGN KEY (order_number) REFERENCES orders(order_number)\n);"
                            );
                        }
                        app.sql_import_error = None;
                    }

                    if ui.add(egui::Button::new(egui::RichText::new("Cancel").size(10.5).color(theme::TEXT_DIM))
                        .fill(theme::WIDGET_BG)
                        .min_size(egui::vec2(70., 28.)))
                        .clicked()
                    {
                        app.show_sql_import_modal = false;
                        app.sql_import_error = None;
                    }
                });
            });
        app.show_sql_import_modal = open;
    }
}

pub fn show_right(app: &mut ProteusApp, ui: &mut egui::Ui) {
    let mut action_register_entity: Option<(String, Vec<proteus_core::sql_ingest::SqlColumnMeta>)> = None;
    let mut action_toast: Option<String> = None;
    let mut action_remove_edge_idx: Option<usize> = None;

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
                crate::flow::FlowNodeKind::TableSchema {
                    ref mut table_name,
                    ref mut columns,
                } => {
                    ui.label(egui::RichText::new("RELATIONAL TABLE SCHEMA").size(10.).color(theme::ACCENT).strong());
                    ui.add_space(4.);
                    ui.label(egui::RichText::new("Table Name:").size(9.5).color(theme::TEXT));
                    ui.add_space(2.);
                    ui.add(
                        egui::TextEdit::singleline(table_name)
                            .hint_text("table_name")
                            .desired_width(f32::INFINITY),
                    );
                    ui.add_space(8.);

                    // Search filter & quick case transforms
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(format!("COLUMNS ({})", columns.len())).size(9.5).color(theme::TEXT_DIM).strong());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.add(egui::Button::new(egui::RichText::new("➕ Add").size(9.5).color(theme::ACCENT_GREEN))).clicked() {
                                let new_idx = columns.len() + 1;
                                columns.push(proteus_core::sql_ingest::SqlColumnMeta {
                                    name: format!("col_{}", new_idx),
                                    data_type: "TEXT".to_string(),
                                    is_nullable: true,
                                    is_primary_key: false,
                                    is_unique: false,
                                    is_indexed: false,
                                });
                            }
                            if ui.add(egui::Button::new(egui::RichText::new("a-z").size(9.).color(theme::TEXT_DIM))).on_hover_text("Transform column names to lowercase").clicked() {
                                for c in columns.iter_mut() {
                                    c.name = c.name.to_lowercase();
                                }
                            }
                            if ui.add(egui::Button::new(egui::RichText::new("A-Z").size(9.).color(theme::TEXT_DIM))).on_hover_text("Transform column names to UPPERCASE").clicked() {
                                for c in columns.iter_mut() {
                                    c.name = c.name.to_uppercase();
                                }
                            }
                        });
                    });
                    ui.add_space(2.);

                    // Quick Column Template Pills
                    ui.horizontal_wrapped(|ui| {
                        if ui.button(egui::RichText::new("+ UUID PK").size(8.5)).clicked() {
                            columns.push(proteus_core::sql_ingest::SqlColumnMeta {
                                name: "id".to_string(),
                                data_type: "UUID".to_string(),
                                is_nullable: false,
                                is_primary_key: true,
                                is_unique: true,
                                is_indexed: true,
                            });
                        }
                        if ui.button(egui::RichText::new("+ Text").size(8.5)).clicked() {
                            let idx = columns.len() + 1;
                            columns.push(proteus_core::sql_ingest::SqlColumnMeta {
                                name: format!("name_{}", idx),
                                data_type: "TEXT".to_string(),
                                is_nullable: true,
                                is_primary_key: false,
                                is_unique: false,
                                is_indexed: false,
                            });
                        }
                        if ui.button(egui::RichText::new("+ Decimal").size(8.5)).clicked() {
                            columns.push(proteus_core::sql_ingest::SqlColumnMeta {
                                name: "amount_eur".to_string(),
                                data_type: "DECIMAL".to_string(),
                                is_nullable: false,
                                is_primary_key: false,
                                is_unique: false,
                                is_indexed: false,
                            });
                        }
                        if ui.button(egui::RichText::new("+ Time").size(8.5)).clicked() {
                            columns.push(proteus_core::sql_ingest::SqlColumnMeta {
                                name: "created_at".to_string(),
                                data_type: "TIMESTAMP".to_string(),
                                is_nullable: false,
                                is_primary_key: false,
                                is_unique: false,
                                is_indexed: true,
                            });
                        }
                    });
                    ui.add_space(2.);

                    ui.add(
                        egui::TextEdit::singleline(&mut app.flow_col_search_query)
                            .hint_text("🔍 Filter columns...")
                            .desired_width(f32::INFINITY),
                    );
                    ui.add_space(4.);

                    let search_q = app.flow_col_search_query.trim().to_lowercase();
                    let mut remove_col_idx: Option<usize> = None;

                    egui::ScrollArea::vertical()
                        .max_height(240.0)
                        .show(ui, |ui| {
                            for (idx, col) in columns.iter_mut().enumerate() {
                                if !search_q.is_empty() && !col.name.to_lowercase().contains(&search_q) {
                                    continue;
                                }
                                ui.group(|ui| {
                                    ui.horizontal(|ui| {
                                        let pk_badge = if col.is_primary_key { "🔑" } else { "•" };
                                        ui.label(egui::RichText::new(pk_badge).size(10.).color(if col.is_primary_key { theme::ACCENT } else { theme::TEXT_DIM }));
                                        ui.add(egui::TextEdit::singleline(&mut col.name).hint_text("column").desired_width(70.));

                                        egui::ComboBox::from_id_salt(format!("col_dt_{}", idx))
                                            .selected_text(&col.data_type)
                                            .width(65.)
                                            .show_ui(ui, |ui| {
                                                for dt in ["TEXT", "INTEGER", "DECIMAL", "UUID", "TIMESTAMP", "JSON", "BOOLEAN"] {
                                                    ui.selectable_value(&mut col.data_type, dt.to_string(), dt);
                                                }
                                            });

                                        if ui.small_button("×").on_hover_text("Delete column").clicked() {
                                            remove_col_idx = Some(idx);
                                        }
                                    });

                                    let is_pk = col.is_primary_key;
                                    let mut is_req = !col.is_nullable;
                                    let pk_color = if is_pk { theme::ACCENT } else { theme::TEXT_DIM };
                                    let req_color = if is_req { theme::ACCENT_GREEN } else { theme::TEXT_DIM };
                                    let uniq_color = if col.is_unique { theme::ACCENT } else { theme::TEXT_DIM };
                                    let idx_color = if col.is_indexed { theme::ACCENT } else { theme::TEXT_DIM };

                                    ui.horizontal(|ui| {
                                        if ui.checkbox(&mut col.is_primary_key, egui::RichText::new("PK").size(8.5).color(pk_color)).changed() {
                                            if col.is_primary_key {
                                                col.is_nullable = false;
                                            }
                                        }
                                        if ui.checkbox(&mut is_req, egui::RichText::new("Req").size(8.5).color(req_color)).changed() {
                                            col.is_nullable = !is_req;
                                        }
                                        ui.checkbox(&mut col.is_unique, egui::RichText::new("Uniq").size(8.5).color(uniq_color));
                                        ui.checkbox(&mut col.is_indexed, egui::RichText::new("Idx").size(8.5).color(idx_color));
                                    });
                                });
                                ui.add_space(2.);
                            }
                        });

                    if let Some(del_idx) = remove_col_idx {
                        if del_idx < columns.len() {
                            columns.remove(del_idx);
                        }
                    }

                    // Synchronize visual node ports with columns
                    let (new_in, new_out) = crate::flow::ports::table_schema_ports(columns);
                    node.input_ports = new_in;
                    node.output_ports = new_out;

                    ui.add_space(8.);
                    ui.separator();
                    ui.add_space(4.);
                    ui.label(egui::RichText::new("VISUAL ETL & CODE ACTIONS").size(9.5).color(theme::TEXT_DIM).strong());
                    ui.add_space(4.);

                    if ui.add(egui::Button::new(egui::RichText::new("📋 Copy SQL DDL").size(10.).color(theme::TEXT))
                        .fill(theme::WIDGET_BG)
                        .min_size(egui::vec2(ui.available_width(), 24.)))
                        .clicked()
                    {
                        let mut ddl = format!("CREATE TABLE {} (\n", table_name);
                        let col_defs: Vec<String> = columns
                            .iter()
                            .map(|c| {
                                let pk = if c.is_primary_key { " PRIMARY KEY" } else { "" };
                                let not_null = if !c.is_nullable && !c.is_primary_key { " NOT NULL" } else { "" };
                                let uniq = if c.is_unique && !c.is_primary_key { " UNIQUE" } else { "" };
                                format!("    {} {}{}{}{}", c.name, c.data_type, pk, not_null, uniq)
                            })
                            .collect();
                        ddl.push_str(&col_defs.join(",\n"));
                        ddl.push_str("\n);");
                        for c in columns.iter() {
                            if c.is_indexed && !c.is_primary_key {
                                ddl.push_str(&format!("\nCREATE INDEX IF NOT EXISTS idx_{}_{} ON {}({});", table_name, c.name, table_name, c.name));
                            }
                        }
                        ui.ctx().copy_text(ddl);
                        action_toast = Some("Table DDL copied to clipboard ✓".to_string());
                    }

                    ui.add_space(3.);
                    if ui.add(egui::Button::new(egui::RichText::new("⚡ Copy Proteus DSL").size(10.).color(Color32::from_rgb(0, 210, 240)).strong())
                        .fill(Color32::from_rgb(20, 36, 48))
                        .stroke(egui::Stroke::new(1.0, Color32::from_rgb(0, 168, 204)))
                        .min_size(egui::vec2(ui.available_width(), 24.)))
                        .clicked()
                    {
                        let mut dsl = format!("entity {} {{\n", table_name);
                        let mut fields = Vec::new();
                        for c in columns.iter() {
                            let ftype = match c.data_type.to_uppercase().as_str() {
                                "UUID" => "uuid",
                                "INT" | "INTEGER" | "BIGINT" => "integer",
                                "REAL" | "FLOAT" | "DECIMAL" => "decimal",
                                "BOOL" | "BOOLEAN" => "bool",
                                "TIMESTAMP" | "DATETIME" => "timestamp",
                                "JSON" | "JSONB" => "json",
                                _ => "text",
                            };
                            let mut flags = Vec::new();
                            if c.is_primary_key { flags.push("primary"); }
                            if !c.is_nullable && !c.is_primary_key { flags.push("required"); }
                            if c.is_unique && !c.is_primary_key { flags.push("unique"); }
                            let flag_str = if flags.is_empty() { String::new() } else { format!(" {}", flags.join(" ")) };
                            fields.push(format!("    {}: {}{}", c.name, ftype, flag_str));
                        }
                        dsl.push_str(&fields.join(",\n"));
                        dsl.push_str("\n}");
                        ui.ctx().copy_text(dsl);
                        action_toast = Some("Proteus DSL snippet copied to clipboard ✓".to_string());
                    }

                    ui.add_space(3.);
                    if ui.add(egui::Button::new(egui::RichText::new("⚡ Register as Proteus Entity").size(10.).color(Color32::BLACK).strong())
                        .fill(theme::ACCENT_GREEN)
                        .min_size(egui::vec2(ui.available_width(), 24.)))
                        .clicked()
                    {
                        action_register_entity = Some((table_name.clone(), columns.clone()));
                    }
                }
                other => {
                    crate::flow::hardware_nodes::render_hardware_inspector(ui, other);
                }
            }

            // Display Connected ETL Relations & Wires for this node
            let node_id = sel_id.clone();
            let connected_edges: Vec<(usize, crate::flow::FlowEdge)> = app
                .project_doc
                .flow_graph
                .edges
                .iter()
                .enumerate()
                .filter(|(_, e)| e.from_node == node_id || e.to_node == node_id)
                .map(|(i, e)| (i, e.clone()))
                .collect();

            if !connected_edges.is_empty() {
                ui.add_space(8.);
                ui.separator();
                ui.add_space(4.);
                ui.label(egui::RichText::new(format!("ETL RELATIONS ({})", connected_edges.len())).size(9.5).color(theme::TEXT_DIM).strong());
                ui.add_space(4.);

                for (edge_idx, edge) in &connected_edges {
                    let color = match edge.color_hue {
                        Some(h) => crate::flow_renderer::hue_to_color32(h),
                        None => theme::ACCENT,
                    };
                    let (rel_text, _is_outgoing) = if edge.from_node == node_id {
                        let _from_col = edge.from_port.strip_prefix("out_").unwrap_or(&edge.from_port);
                        let to_col = edge.to_port.strip_prefix("in_").unwrap_or(&edge.to_port);
                        (format!("➔ {}.{}", edge.to_node.strip_prefix("tbl_").unwrap_or(&edge.to_node), to_col), true)
                    } else {
                        let from_col = edge.from_port.strip_prefix("out_").unwrap_or(&edge.from_port);
                        let _to_col = edge.to_port.strip_prefix("in_").unwrap_or(&edge.to_port);
                        (format!(" {}.{}", edge.from_node.strip_prefix("tbl_").unwrap_or(&edge.from_node), from_col), false)
                    };

                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("●").size(10.).color(color));
                        ui.label(egui::RichText::new(rel_text).size(9.5).color(theme::TEXT));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("×").on_hover_text("Disconnect wire").clicked() {
                                action_remove_edge_idx = Some(*edge_idx);
                            }
                        });
                    });
                }

                ui.add_space(4.);
                if ui.add(egui::Button::new(egui::RichText::new("🔗 Generate ETL JOIN View").size(9.5).color(theme::TEXT))
                    .fill(theme::WIDGET_BG)
                    .min_size(egui::vec2(ui.available_width(), 22.)))
                    .clicked()
                {
                    let mut joins = Vec::new();
                    let current_tbl = node_id.strip_prefix("tbl_").unwrap_or(&node_id);
                    for (_, edge) in &connected_edges {
                        let src_tbl = edge.from_node.strip_prefix("tbl_").unwrap_or(&edge.from_node);
                        let tgt_tbl = edge.to_node.strip_prefix("tbl_").unwrap_or(&edge.to_node);
                        let src_col = edge.from_port.strip_prefix("out_").unwrap_or(&edge.from_port);
                        let tgt_col = edge.to_port.strip_prefix("in_").unwrap_or(&edge.to_port);
                        if edge.from_node == node_id {
                            joins.push(format!("JOIN {} ON {}.{} = {}.{}", tgt_tbl, current_tbl, src_col, tgt_tbl, tgt_col));
                        } else {
                            joins.push(format!("JOIN {} ON {}.{} = {}.{}", src_tbl, current_tbl, tgt_col, src_tbl, src_col));
                        }
                    }
                    let query = format!("SELECT * FROM {} {}", current_tbl, joins.join(" "));
                    ui.ctx().copy_text(query);
                    action_toast = Some("ETL JOIN query copied to clipboard".to_string());
                }
            }
        }
    } else {
        ui.label(egui::RichText::new("Select a node on the canvas to configure it.").size(10.).color(theme::TEXT_DIM));
    }

    if let Some(idx) = action_remove_edge_idx {
        if idx < app.project_doc.flow_graph.edges.len() {
            app.project_doc.flow_graph.edges.remove(idx);
            app.toast("Wire removed");
        }
    }
    if let Some((tbl, cols)) = action_register_entity {
        match app.register_table_as_entity(&tbl, &cols) {
            Ok(()) => app.toast(format!("Entity '{}' registered into Proteus DataEngine", tbl)),
            Err(e) => app.toast(format!("Failed to register entity: {}", e)),
        }
    }
    if let Some(msg) = action_toast {
        app.toast(msg);
    }

    ui.add_space(12.);
    ui.separator();
    ui.add_space(4.);
    let count = app.project_doc.flow_graph.nodes.len();
    let edge_count = app.project_doc.flow_graph.edges.len();
    ui.label(egui::RichText::new(format!("Graph: {} nodes, {} edges", count, edge_count)).size(9.).color(theme::TEXT_DIM));
}
