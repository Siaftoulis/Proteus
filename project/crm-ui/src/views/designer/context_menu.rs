//! Context-aware right-click menu dispatcher and multi-selection actions.

use eframe::egui::{self, Color32, Response};
use crate::scene::{self, NodeUpdate};
use crate::theme;
use crate::ProteusApp;

pub fn handle_context_menu(app: &mut ProteusApp, resp: &Response) {
    resp.context_menu(|ui| {
        let sel_count = app.editor_state.selected_node_ids.len();

        if sel_count > 1 {
            // ── MULTI-SELECTION CONTEXT MENU ──
            ui.label(
                egui::RichText::new(format!("MULTI-SELECTION ({} elements)", sel_count))
                    .size(10.5)
                    .color(theme::ACCENT)
                    .strong(),
            );
            ui.separator();

            ui.label(egui::RichText::new("ALIGN").size(9.).color(theme::TEXT_DIM));
            if ui.button("⇤ Align Left").clicked() {
                let mut min_x = f32::INFINITY;
                for id in &app.editor_state.selected_node_ids {
                    if let Some(n) = app.project_doc.nodes.get(id) {
                        if n.position.0 < min_x {
                            min_x = n.position.0;
                        }
                    }
                }
                if min_x.is_finite() {
                    app.push_undo();
                    for id in &app.editor_state.selected_node_ids.clone() {
                        if let Some(n) = app.project_doc.nodes.get(id) {
                            let _ = app.project_doc.update_node(id, NodeUpdate::Move { x: min_x, y: n.position.1 });
                        }
                    }
                    app.toast("Aligned Left ✓");
                }
                ui.close_menu();
            }

            if ui.button("⇥ Align Right").clicked() {
                let mut max_right = f32::NEG_INFINITY;
                for id in &app.editor_state.selected_node_ids {
                    if let Some(n) = app.project_doc.nodes.get(id) {
                        let w = match n.layout.width {
                            scene::Sizing::Fixed(w) | scene::Sizing::Fill(w) => w,
                            scene::Sizing::Hug => 200.,
                        };
                        let r = n.position.0 + w;
                        if r > max_right {
                            max_right = r;
                        }
                    }
                }
                if max_right.is_finite() {
                    app.push_undo();
                    for id in &app.editor_state.selected_node_ids.clone() {
                        if let Some(n) = app.project_doc.nodes.get(id) {
                            let w = match n.layout.width {
                                scene::Sizing::Fixed(w) | scene::Sizing::Fill(w) => w,
                                scene::Sizing::Hug => 200.,
                            };
                            let _ = app.project_doc.update_node(id, NodeUpdate::Move { x: max_right - w, y: n.position.1 });
                        }
                    }
                    app.toast("Aligned Right ✓");
                }
                ui.close_menu();
            }

            if ui.button("⤒ Align Top").clicked() {
                let mut min_y = f32::INFINITY;
                for id in &app.editor_state.selected_node_ids {
                    if let Some(n) = app.project_doc.nodes.get(id) {
                        if n.position.1 < min_y {
                            min_y = n.position.1;
                        }
                    }
                }
                if min_y.is_finite() {
                    app.push_undo();
                    for id in &app.editor_state.selected_node_ids.clone() {
                        if let Some(n) = app.project_doc.nodes.get(id) {
                            let _ = app.project_doc.update_node(id, NodeUpdate::Move { x: n.position.0, y: min_y });
                        }
                    }
                    app.toast("Aligned Top ✓");
                }
                ui.close_menu();
            }

            ui.separator();
            if ui.button(egui::RichText::new(format!("🗑 Delete All ({})", sel_count)).color(Color32::from_rgb(255, 100, 100))).clicked() {
                app.push_undo();
                for id in &app.editor_state.selected_node_ids.clone() {
                    app.project_doc.delete_node(id);
                }
                app.designer_selected_node = None;
                app.editor_state.selected_node_ids.clear();
                app.toast(format!("Deleted {} elements", sel_count));
                ui.close_menu();
            }
        } else if let Some(ref sel_id) = app.designer_selected_node.clone() {
            // ── SINGLE NODE CONTEXT MENU ──
            let node_name = app.project_doc.nodes.get(sel_id)
                .map(|n| n.name.as_str())
                .unwrap_or("Element");
            let is_locked = app.project_doc.nodes.get(sel_id).map(|n| n.locked).unwrap_or(false);
            let is_visible = app.project_doc.nodes.get(sel_id).map(|n| n.visible).unwrap_or(true);

            ui.label(egui::RichText::new(format!("ELEMENT: {}", node_name)).size(10.5).color(theme::ACCENT).strong());
            ui.separator();

            if ui.button("📋 Copy (Ctrl+C)").clicked() {
                if let Some(node) = app.project_doc.get_node(sel_id).cloned() {
                    app.editor_state.clipboard = Some(vec![node]);
                    app.toast("Copied to clipboard ✓");
                }
                ui.close_menu();
            }

            if ui.button("⎘ Duplicate (Ctrl+D)").clicked() {
                if let Some(node) = app.project_doc.get_node(sel_id).cloned() {
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
                    app.toast("Duplicated ✓");
                }
                ui.close_menu();
            }

            ui.separator();
            if ui.button("📏 Copy Dimensions").clicked() {
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
            if ui.add_enabled(can_paste, egui::Button::new("📐 Paste Dimensions")).clicked() {
                if let Some((w, h)) = app.copied_dimensions {
                    app.push_undo();
                    let _ = app.project_doc.update_node(sel_id, scene::NodeUpdate::Dimensions { width: w, height: h });
                    app.toast("Dimensions applied ✓");
                }
                ui.close_menu();
            }

            ui.separator();
            let lock_label = if is_locked { "🔓 Unlock Element" } else { "🔒 Lock Element" };
            if ui.button(lock_label).clicked() {
                let _ = app.project_doc.update_node(sel_id, scene::NodeUpdate::ToggleLock);
                ui.close_menu();
            }

            let vis_label = if is_visible { "🙈 Hide Element" } else { "👁 Show Element" };
            if ui.button(vis_label).clicked() {
                let _ = app.project_doc.update_node(sel_id, scene::NodeUpdate::ToggleVisibility);
                ui.close_menu();
            }

            if ui.button("▲ Bring Forward").clicked() {
                if let Some(n) = app.project_doc.nodes.get_mut(sel_id) {
                    n.z += 1;
                    app.toast("Moved forward ✓");
                }
                ui.close_menu();
            }

            if ui.button("▼ Send Backward").clicked() {
                if let Some(n) = app.project_doc.nodes.get_mut(sel_id) {
                    n.z = n.z.saturating_sub(1);
                    app.toast("Moved backward ✓");
                }
                ui.close_menu();
            }

            ui.separator();
            if ui.button(egui::RichText::new("🗑 Delete").color(Color32::from_rgb(255, 100, 100))).clicked() {
                app.push_undo();
                app.project_doc.delete_node(sel_id);
                app.designer_selected_node = None;
                app.editor_state.selected_node_ids.clear();
                app.toast("Node deleted");
                ui.close_menu();
            }
        } else {
            // ── EMPTY CANVAS CONTEXT MENU ──
            ui.label(egui::RichText::new("CANVAS").size(10.5).color(theme::ACCENT).strong());
            ui.separator();

            let has_clipboard = app.editor_state.clipboard.as_ref().map(|c| !c.is_empty()).unwrap_or(false);
            if ui.add_enabled(has_clipboard, egui::Button::new("📋 Paste (Ctrl+V)")).clicked() {
                if let Some(clipboard) = app.editor_state.clipboard.clone() {
                    app.push_undo();
                    for node in clipboard {
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
                    }
                    app.toast("Pasted from clipboard ✓");
                }
                ui.close_menu();
            }

            if ui.button("🗂 Select All (Ctrl+A)").clicked() {
                let all_ids: Vec<String> = app.project_doc.nodes.keys().cloned().collect();
                app.designer_selected_node = all_ids.first().cloned();
                app.editor_state.selected_node_ids = all_ids;
                app.toast("Selected all elements");
                ui.close_menu();
            }

            if ui.button("⟲ Reset Zoom (100%)").clicked() {
                app.viewport.zoom = 1.0;
                app.viewport.pan = egui::Vec2::ZERO;
                app.toast("Zoom reset to 100%");
                ui.close_menu();
            }

            ui.separator();
            ui.label(egui::RichText::new("INSERT").size(9.).color(theme::TEXT_DIM));

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
            if ui.button("⊞ Data Table (G)").clicked() {
                app.active_tool = crate::models::DesignerTool::Table;
                ui.close_menu();
            }
            if ui.button("💳 KPI Metric Card").clicked() {
                let offset = (app.spawn_counter as f32 * 25.0) % 250.0;
                app.spawn_kpi_card((120.0 + offset, 120.0 + offset));
                ui.close_menu();
            }
            if ui.button("📝 Intake Form Block").clicked() {
                let offset = (app.spawn_counter as f32 * 25.0) % 250.0;
                app.spawn_form_block((150.0 + offset, 140.0 + offset));
                ui.close_menu();
            }
        }
    });
}
