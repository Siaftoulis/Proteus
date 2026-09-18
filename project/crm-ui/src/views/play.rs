use eframe::egui::{self, Rect};
use crate::scene::{self, CanvasEvent};
use crate::theme;
use crate::ProteusApp;
use crate::renderer;
use crate::db;

use crate::models::DevicePreset;

pub fn show_left(app: &mut ProteusApp, ui: &mut egui::Ui) {
    ui.add_space(6.);
    ui.label(egui::RichText::new("📱 DEVICE PREVIEWS").size(9.).color(theme::ACCENT).strong());
    ui.add_space(4.);
    ui.label(egui::RichText::new("Select target viewport to simulate your app on different devices:").size(9.).color(theme::TEXT_DIM));
    ui.add_space(6.);

    let presets = [
        DevicePreset::DesktopHD,
        DevicePreset::Desktop,
        DevicePreset::Laptop,
        DevicePreset::TabletLandscape,
        DevicePreset::TabletPortrait,
        DevicePreset::Phone,
    ];

    for p in &presets {
        let is_active = app.device_preset == *p;
        let (w, h) = p.size();
        let label = format!("{} {} ({}×{})", p.icon(), p.label(), w as u32, h as u32);
        let btn = ui.add(
            egui::Button::new(
                egui::RichText::new(label)
                    .size(10.)
                    .color(if is_active { theme::TEXT } else { theme::TEXT_DIM })
            )
            .fill(if is_active { theme::ELEVATED } else { theme::WIDGET_BG })
            .stroke(if is_active { egui::Stroke::new(1., theme::ACCENT) } else { egui::Stroke::NONE })
            .min_size(egui::vec2(ui.available_width(), 24.))
        );
        if btn.clicked() {
            app.device_preset = p.clone();
            app.toast(format!("Target: {}", p.label()));
        }
    }

    ui.add_space(10.);
    ui.separator();
    ui.add_space(8.);

    ui.label(egui::RichText::new("INTERACTION CONTROLS").size(9.).color(theme::TEXT_DIM).strong());
    ui.add_space(4.);

    if ui.add(egui::Button::new("🔄 Reset Form State").min_size(egui::vec2(ui.available_width(), 22.))).clicked() {
        app.form_state.clear();
        app.toast("Form reset ✓");
    }

    ui.add_space(8.);
    ui.separator();
    ui.add_space(8.);

    ui.label(egui::RichText::new("LAN AUTO-DISCOVERY & DEPLOY").size(9.).color(theme::TEXT_DIM).strong());
    ui.add_space(4.);

    ui.checkbox(&mut app.auto_deploy_on_save, egui::RichText::new("⚡ Auto-Deploy on Save").size(10.).strong());
    ui.add_space(4.);

    let live_peers = app.lan_daemon.as_ref().map(|d| d.get_live_peers()).unwrap_or_default();

    if live_peers.is_empty() {
        ui.label(egui::RichText::new("📡 Listening (UDP:7444)... No terminals found").size(9.).color(theme::TEXT_DIM));
        ui.add_space(2.);
        if ui.add(egui::Button::new(egui::RichText::new("🚀 Deploy to Localhost (7443)").strong().color(egui::Color32::WHITE))
            .fill(theme::ACCENT)
            .min_size(egui::vec2(ui.available_width(), 24.))).clicked()
        {
            let mut pkg = crm_core::package::PrPackage::new(
                format!("PKG-{}", app.pid),
                if app.pname.is_empty() { "Custom Template".to_string() } else { app.pname.clone() },
                "Designer (PCD)",
            );
            pkg.views.push(crm_core::package::PrViewLayout {
                view_id: app.pid.clone(),
                name: app.pname.clone(),
                view_type: "designer_canvas".to_string(),
                layout_json: serde_json::to_string(&app.project_doc).unwrap_or_default(),
            });

            match pkg.deploy_to_client("http://127.0.0.1:7443") {
                Ok(summary) => {
                    app.toast(format!("✓ Παραδόθηκε στο τοπικό τερματικό: '{}'!", summary.package_name));
                }
                Err(e) => {
                    app.toast(format!("❌ Σφάλμα σύνδεσης (127.0.0.1:7443): {}", e));
                }
            }
        }
    } else {
        ui.label(egui::RichText::new(format!("🟢 {} Τερματικά Ενεργά στο LAN:", live_peers.len())).size(9.).color(theme::ACCENT_GREEN).strong());
        ui.add_space(2.);

        for peer in &live_peers {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("●").size(10.).color(theme::ACCENT_GREEN));
                ui.label(egui::RichText::new(format!("{} ({})", peer.device_name, peer.ip)).size(9.).color(theme::TEXT));
            });
        }
        ui.add_space(4.);

        if ui.add(egui::Button::new(egui::RichText::new(format!("🚀 Deploy to All Terminals ({})", live_peers.len())).strong().color(egui::Color32::WHITE))
            .fill(theme::ACCENT)
            .min_size(egui::vec2(ui.available_width(), 26.))).clicked()
        {
            let mut pkg = crm_core::package::PrPackage::new(
                format!("PKG-{}", app.pid),
                if app.pname.is_empty() { "Custom Template".to_string() } else { app.pname.clone() },
                "Designer (PCD)",
            );
            pkg.views.push(crm_core::package::PrViewLayout {
                view_id: app.pid.clone(),
                name: app.pname.clone(),
                view_type: "designer_canvas".to_string(),
                layout_json: serde_json::to_string(&app.project_doc).unwrap_or_default(),
            });

            if let Some(ref daemon) = app.lan_daemon {
                let results = daemon.deploy_to_all_peers(&pkg);
                let ok = results.iter().filter(|(_, r)| r.is_ok()).count();
                app.toast(format!("✓ Παραδόθηκε σε {}/{} τερματικά LAN!", ok, results.len()));
            }
        }
    }

    ui.add_space(6.);
    ui.separator();
    ui.add_space(4.);

    ui.label(egui::RichText::new("PACKAGES & MARKETPLACE").size(9.).color(theme::TEXT_DIM).strong());
    ui.add_space(4.);

    if ui.add(egui::Button::new(egui::RichText::new("📦 Export .pr Package").color(theme::ACCENT_GREEN)).min_size(egui::vec2(ui.available_width(), 22.))).clicked() {
        app.save_project();
        app.toast("Package .pr generated & ready for Marketplace ✓");
    }
    ui.add_space(4.);

    if ui.add(egui::Button::new(egui::RichText::new("💾 Export Standalone (.crmb)").color(theme::TEXT_DIM)).min_size(egui::vec2(ui.available_width(), 20.))).clicked() {
        app.save_project();
        app.toast("Project exported as standalone bundle ✓");
    }
}

pub fn show_central(app: &mut ProteusApp, ui: &mut egui::Ui, pnt: &egui::Painter, r: Rect) {
    if app.active_play_page.is_none() {
        app.active_play_page = app.project_doc.root_node_ids.first().cloned();
    }

    if let Some(ref pid) = app.active_play_page {
        if app.play_viewport.pan == egui::Vec2::ZERO {
            if let Some(pnode) = app.project_doc.nodes.get(pid) {
                let (pw, ph) = match (&pnode.layout.width, &pnode.layout.height) {
                    (scene::Sizing::Fixed(w), scene::Sizing::Fixed(h)) => (*w, *h),
                    _ => (800., 600.),
                };
                let center = r.center();
                let canvas_origin = ui.max_rect().min;
                app.play_viewport.pan = egui::vec2(
                    center.x - canvas_origin.x - (pnode.position.0 + pw * 0.5) * app.play_viewport.zoom,
                    center.y - canvas_origin.y - (pnode.position.1 + ph * 0.5) * app.play_viewport.zoom,
                );
            }
        }
    }

    let play_events = renderer::draw_document(
        ui,
        &app.project_doc,
        &app.play_viewport,
        &app.editor_state,
        true,
        &mut app.form_state,
        app.active_play_page.as_deref(),
        &app.table_cache,
    );

    for ev in &play_events {
        match ev {
            CanvasEvent::ActionTriggered { source_node_id } => {
                let trigger_nodes: Vec<String> = app.project_doc.flow_graph.nodes.values().filter_map(|n| {
                    if let crate::flow::FlowNodeKind::TriggerClick { ref target_node_id } = n.kind {
                        if target_node_id == source_node_id {
                            Some(n.id.clone())
                        } else { None }
                    } else { None }
                }).collect();

                for trigger_id in trigger_nodes {
                    let mut curr_ids = vec![trigger_id];
                    let mut visited = std::collections::HashSet::new();

                    while let Some(curr) = curr_ids.pop() {
                        if !visited.insert(curr.clone()) { continue; }
                        let next_edges: Vec<String> = app.project_doc.flow_graph.edges.iter()
                            .filter(|e| e.from_node == curr)
                            .map(|e| e.to_node.clone())
                            .collect();

                        for target_id in next_edges {
                            if let Some(target_node) = app.project_doc.flow_graph.nodes.get(&target_id).cloned() {
                                match target_node.kind {
                                    crate::flow::FlowNodeKind::SaveToDatabase { ref entity } => {
                                        if entity.is_empty() {
                                            app.toast("Flow Error: Target entity not configured".to_string());
                                            continue;
                                        }
                                        let mut data = serde_json::Map::new();
                                        for (nid, node) in &app.project_doc.nodes {
                                            let (_bound_entity, bound_field) = match &node.node_type {
                                                scene::NodeType::TextInput { bound_entity, bound_field, .. }
                                                | scene::NodeType::Dropdown { bound_entity, bound_field, .. }
                                                | scene::NodeType::NumberField { bound_entity, bound_field, .. }
                                                | scene::NodeType::Checkbox { bound_entity, bound_field, .. } => (bound_entity, bound_field),
                                                _ => continue,
                                            };
                                            if let Some(field) = bound_field {
                                                if !field.is_empty() {
                                                    let value = app.form_state.get(nid).cloned().unwrap_or_default();
                                                    if !value.is_empty() {
                                                        data.insert(field.clone(), serde_json::Value::String(value));
                                                    }
                                                }
                                            }
                                        }
                                        let json_data = serde_json::Value::Object(data);
                                        match &app.db_conn {
                                            Some(conn) => {
                                                match db::upsert_record(conn, entity, None, &json_data) {
                                                    Ok(id) => {
                                                        app.form_state.clear();
                                                        app.toast(format!("Saved {} ✓ (#{})", entity, &id[..6.min(id.len())]));
                                                        app.reload_table_cache(entity);
                                                    }
                                                    Err(e) => app.toast(format!("Save failed: {}", e)),
                                                }
                                            }
                                            None => app.toast("DB not connected".to_string()),
                                        }
                                    }
                                    crate::flow::FlowNodeKind::NavigateTo { ref page_id } => {
                                        if app.project_doc.nodes.contains_key(page_id) {
                                            app.active_play_page = Some(page_id.clone());
                                            app.form_state.clear();
                                            // Smoothly center the target page in the play viewport
                                            if let Some(pnode) = app.project_doc.nodes.get(page_id) {
                                                let (pw, ph) = match (&pnode.layout.width, &pnode.layout.height) {
                                                    (scene::Sizing::Fixed(w), scene::Sizing::Fixed(h)) => (*w, *h),
                                                    _ => (800., 600.),
                                                };
                                                let center = r.center();
                                                let canvas_origin = ui.max_rect().min;
                                                app.play_viewport.pan = egui::vec2(
                                                    center.x - canvas_origin.x - (pnode.position.0 + pw * 0.5) * app.play_viewport.zoom,
                                                    center.y - canvas_origin.y - (pnode.position.1 + ph * 0.5) * app.play_viewport.zoom,
                                                );
                                            }
                                            let page_name = app.project_doc.nodes.get(page_id).map(|n| n.name.as_str()).unwrap_or(page_id);
                                            app.toast(format!("Screen: {}", page_name));
                                        } else {
                                            app.toast(format!("Flow Error: Page '{}' not found", page_id));
                                        }
                                    }
                                    crate::flow::FlowNodeKind::TriggerClick { .. } => {}
                                }
                                curr_ids.push(target_id);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    if app.project_doc.nodes.is_empty() {
        pnt.text(r.center(), egui::Align2::CENTER_CENTER,
            "Design your form in Designer mode, then switch to Play to interact",
            egui::FontId::proportional(14.), theme::TEXT_DIM);
    }
}

pub fn show_right(app: &mut ProteusApp, ui: &mut egui::Ui) {
    ui.label(egui::RichText::new("RUN MONITOR").size(9.).color(theme::TEXT_DIM).strong());
    ui.add_space(4.);
    ui.label(egui::RichText::new("Interact with form elements and buttons in real-time.").size(10.).color(theme::TEXT_DIM));
    ui.add_space(8.);
    ui.separator();
    ui.add_space(8.);

    ui.label(egui::RichText::new("DATABASE ENGINE").size(9.).color(theme::ACCENT).strong());
    ui.add_space(4.);
    if let Some(conn) = &app.db_conn {
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Status:").size(9.).color(theme::TEXT_DIM));
            ui.label(egui::RichText::new("● Connected (SQLite)").size(9.).color(theme::ACCENT_GREEN));
        });
        ui.add_space(2.);
        if let Ok(recs) = db::get_records(conn, "") {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Records in DB:").size(9.).color(theme::TEXT_DIM));
                ui.label(egui::RichText::new(format!("{}", recs.len())).size(10.).color(theme::TEXT).strong());
            });
        }
    } else {
        ui.label(egui::RichText::new("Offline / Standalone Cache").size(9.).color(theme::ACCENT_ORANGE));
    }

    ui.add_space(8.);
    ui.separator();
    ui.add_space(8.);

    ui.label(egui::RichText::new("ACTIVE FORM STATE").size(9.).color(theme::TEXT_DIM).strong());
    ui.add_space(4.);
    if app.form_state.is_empty() {
        ui.label(egui::RichText::new("(no form inputs modified yet)").size(9.).color(theme::TEXT_MUTED));
    } else {
        for (field_id, val) in &app.form_state {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("{}:", field_id)).size(9.).color(theme::ACCENT_CYAN));
                ui.label(egui::RichText::new(val).size(9.).color(theme::TEXT));
            });
        }
    }
}
