//! Designer layout coordinator and views lifecycle.
//! Implements the Penpot-inspired 3-column architecture (Tools + Layers/Assets, Infinite Canvas, Design/Prototype Inspector).

pub mod canvas;
pub mod context_menu;
pub mod interaction;

pub use canvas::*;
pub use context_menu::*;
pub use interaction::*;

use eframe::egui::{self, Color32, Pos2, Rect, Response, Stroke, Vec2};
use crate::inspector;
use crate::renderer;
use crate::scene::{self, CanvasEvent};
use crate::theme;
use crate::ProteusApp;

pub fn tool_icon_btn(
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
    .min_size(Vec2::new(26., 24.));

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
    ui.add_space(2.);

    // ── Penpot Top Tool Rail ──
    ui.horizontal_wrapped(|ui| {
        if tool_icon_btn(ui, "↖", app.active_tool == crate::models::DesignerTool::Select, "Select Tool", "V", "Select and move elements").clicked() {
            app.active_tool = crate::models::DesignerTool::Select;
        }
        if tool_icon_btn(ui, "▦", false, "Add Board/Page", "P", "Add new canvas artboard/page").clicked() {
            let page_num = app.project_doc.root_node_ids.len() + 1;
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
        if tool_icon_btn(ui, "▢", app.active_tool == crate::models::DesignerTool::Rectangle, "Smart Box", "R", "Draw Box").clicked() {
            app.active_tool = crate::models::DesignerTool::Rectangle;
        }
        if tool_icon_btn(ui, "T", app.active_tool == crate::models::DesignerTool::Text, "Text Tool", "T", "Draw Text").clicked() {
            app.active_tool = crate::models::DesignerTool::Text;
        }
        if tool_icon_btn(ui, "🔘", app.active_tool == crate::models::DesignerTool::Button, "Action Button", "B", "Draw Button").clicked() {
            app.active_tool = crate::models::DesignerTool::Button;
        }
        if tool_icon_btn(ui, "⊞", app.active_tool == crate::models::DesignerTool::Table, "Data Table", "G", "Draw Table").clicked() {
            app.active_tool = crate::models::DesignerTool::Table;
        }
        if tool_icon_btn(ui, "✋", app.active_tool == crate::models::DesignerTool::Hand, "Hand Tool", "H", "Pan Canvas").clicked() {
            app.active_tool = crate::models::DesignerTool::Hand;
        }
    });

    ui.add_space(4.);
    ui.separator();
    ui.add_space(4.);

    // ── Penpot Left Sidebar Tabs (Layers vs Assets) ──
    ui.horizontal(|ui| {
        let is_layers = app.designer_left_tab == crate::models::DesignerLeftTab::Layers;
        let is_assets = app.designer_left_tab == crate::models::DesignerLeftTab::Assets;

        if ui.add(
            egui::Button::new(
                egui::RichText::new("▤ LAYERS")
                    .size(9.5)
                    .strong()
                    .color(if is_layers { theme::ACCENT } else { Color32::GRAY }),
            )
            .fill(if is_layers { Color32::from_rgba_unmultiplied(79, 140, 237, 25) } else { Color32::TRANSPARENT })
            .stroke(if is_layers { egui::Stroke::new(1., theme::ACCENT) } else { egui::Stroke::NONE })
            .min_size(Vec2::new(96., 22.)),
        ).clicked() {
            app.designer_left_tab = crate::models::DesignerLeftTab::Layers;
        }

        if ui.add(
            egui::Button::new(
                egui::RichText::new("✦ ASSETS")
                    .size(9.5)
                    .strong()
                    .color(if is_assets { theme::ACCENT } else { Color32::GRAY }),
            )
            .fill(if is_assets { Color32::from_rgba_unmultiplied(79, 140, 237, 25) } else { Color32::TRANSPARENT })
            .stroke(if is_assets { egui::Stroke::new(1., theme::ACCENT) } else { egui::Stroke::NONE })
            .min_size(Vec2::new(96., 22.)),
        ).clicked() {
            app.designer_left_tab = crate::models::DesignerLeftTab::Assets;
        }
    });

    ui.add_space(4.);

    match app.designer_left_tab {
        crate::models::DesignerLeftTab::Layers => {
            let mut layer_events = Vec::new();
            crate::components::layers_panel::draw_layers_panel(
                ui,
                &app.project_doc,
                app.designer_selected_node.as_deref(),
                ui.available_height() - 10.,
                &mut layer_events,
            );
            for ev in layer_events {
                match ev {
                    CanvasEvent::NodeClicked { id, shift_held } => {
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
                    CanvasEvent::NodeModified { id, update } => {
                        let _ = app.project_doc.update_node(&id, update);
                    }
                    CanvasEvent::DeleteNode { id } => {
                        app.push_undo();
                        app.project_doc.delete_node(&id);
                        if app.designer_selected_node.as_deref() == Some(&id) {
                            app.designer_selected_node = None;
                            app.editor_state.selected_node_ids.clear();
                        }
                    }
                    _ => {}
                }
            }
        }
        crate::models::DesignerLeftTab::Assets => {
            ui.label(egui::RichText::new("ONE-CLICK COMPONENT BLOCKS").size(8.5).color(theme::TEXT_MUTED));
            ui.add_space(4.);

            if ui.button("💳 KPI Metric Card").on_hover_text("Insert pre-styled KPI card").clicked() {
                let offset = (app.spawn_counter as f32 * 25.0) % 250.0;
                app.spawn_kpi_card((120.0 + offset, 120.0 + offset));
            }
            ui.add_space(2.);
            if ui.button("📝 Intake Form Block").on_hover_text("Insert customer intake form").clicked() {
                let offset = (app.spawn_counter as f32 * 25.0) % 250.0;
                app.spawn_form_block((160.0 + offset, 140.0 + offset));
            }
            ui.add_space(2.);
            if ui.button("⊞ Data Grid View").on_hover_text("Insert SQLite data grid").clicked() {
                let offset = (app.spawn_counter as f32 * 25.0) % 250.0;
                app.spawn_designer_node(scene::NodeType::Table {
                    bound_entity: Some("contacts".into()),
                    columns: vec!["ID".into(), "Name".into(), "Phone".into()],
                }, Pos2::new(180.0 + offset, 160.0 + offset));
            }
            ui.add_space(2.);
            if ui.button("🔘 Action Trigger").on_hover_text("Insert button with SQLite submit action").clicked() {
                let offset = (app.spawn_counter as f32 * 25.0) % 250.0;
                app.spawn_designer_node(scene::NodeType::Button {
                    label: "Submit Record".into(),
                    style: scene::ButtonStyle::Primary,
                }, Pos2::new(200.0 + offset, 180.0 + offset));
            }
        }
    }
}

pub fn show_central(
    app: &mut ProteusApp,
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    pnt: &egui::Painter,
    r: Rect,
    mpos: Option<Pos2>,
    resp: &Response,
) {
    let canvas_origin = r.left_top();

    // Palette click (fallback) → spawn at viewport center
    if let Some(nt) = app.palette_drag.take() {
        let center = r.center();
        let world_center = app.viewport.screen_to_world(center, canvas_origin);
        app.spawn_designer_node(nt, world_center);
    }

    let hover_canvas = mpos.map(|p| r.contains(p)).unwrap_or(false);

    // 1. Pan & Zoom
    let is_panning = handle_pan_and_zoom(app, ui, r, mpos, hover_canvas, canvas_origin);

    // 2. Render Grid & Device Frame
    render_grid(app, ctx, pnt, r, canvas_origin);
    render_device_frame(app, ctx, pnt, r, canvas_origin);

    // 3. Keyboard shortcuts
    handle_keyboard_shortcuts(app, ctx, ui);

    // 4. Render arena document
    let arena_events = renderer::draw_document(
        ui,
        &app.project_doc,
        &app.viewport,
        &app.editor_state,
        false,
        &mut app.form_state,
        None,
        &app.table_cache,
    );

    let mut is_resizing = false;
    for ev in &arena_events {
        if let CanvasEvent::NodeResizeStarted = ev {
            is_resizing = true;
        }
    }

    // 5. Mouse Interaction
    if !is_panning {
        if let Some(mp) = mpos {
            let is_draw_tool = matches!(
                app.active_tool,
                crate::models::DesignerTool::Rectangle
                    | crate::models::DesignerTool::Text
                    | crate::models::DesignerTool::Button
                    | crate::models::DesignerTool::Table
            );

            if is_draw_tool {
                handle_drawing_tool(app, ui, pnt, mp, canvas_origin, hover_canvas);
            } else {
                handle_mouse_selection(app, ui, pnt, mp, canvas_origin, hover_canvas, is_resizing);
            }
        }
    }

    // 6. Arena Events dispatch
    handle_arena_events(app, &arena_events);

    // 7. Bottom Canvas HUD
    render_canvas_hud(app, ui, pnt, r, mpos, canvas_origin);

    // 8. Right-click context menu
    handle_context_menu(app, resp);
}

pub fn show_right(app: &mut ProteusApp, ui: &mut egui::Ui) {
    let events = inspector::draw_inspector(
        ui,
        &app.project_doc,
        &app.editor_state,
        &mut app.inspector_tab,
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
