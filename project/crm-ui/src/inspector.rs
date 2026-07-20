use crate::scene::{CanvasEvent, EditorState, NodeType, NodeUpdate, ProjectDocument, Rgba};
use eframe::egui::{self, Color32, ScrollArea};

// ── Colour bridge helpers ──

fn rgba_to_color32(c: &Rgba) -> Color32 {
    Color32::from_rgba_premultiplied(c.r, c.g, c.b, c.a)
}

fn color32_to_rgba(c: Color32) -> Rgba {
    Rgba { r: c.r(), g: c.g(), b: c.b(), a: c.a() }
}

// ── Public entry point ──

/// Show the inspector panel for the currently selected arena node.
/// Returns events to be processed by the caller.
pub fn draw_inspector(
    ui: &mut egui::Ui,
    doc: &ProjectDocument,
    editor_state: &EditorState,
) -> Vec<CanvasEvent> {
    let mut events = Vec::new();

    let node_id = match editor_state.selected_node_ids.first() {
        Some(id) => id.clone(),
        None => {
            ui.label(egui::RichText::new("No selection").size(10.).color(Color32::DARK_GRAY));
            return events;
        }
    };

    let node = match doc.get_node(&node_id) {
        Some(n) => n,
        None => {
            ui.label(egui::RichText::new("Node not found").size(10.).color(Color32::DARK_GRAY));
            return events;
        }
    };

    ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(4.);
        ui.label(egui::RichText::new(&node.name).size(13.).color(Color32::WHITE).strong());
        ui.label(egui::RichText::new(format!("id: {}", node_id)).size(8.).color(Color32::GRAY));
        ui.separator();

        // ── Content section ──
        ui.add_space(2.);
        ui.label(egui::RichText::new("CONTENT").size(9.).color(Color32::GRAY));
        match &node.node_type {
            NodeType::Text { content, .. } => {
                let mut val = content.clone();
                ui.add_space(2.);
                ui.label("Text:");
                let resp = ui.add(egui::TextEdit::singleline(&mut val).desired_width(f32::INFINITY));
                if resp.lost_focus() && val != *content {
                    events.push(CanvasEvent::NodeModified {
                        id: node_id.clone(),
                        update: NodeUpdate::TextContent(val),
                    });
                }
            }
            NodeType::Button { label, .. } => {
                let mut val = label.clone();
                ui.add_space(2.);
                ui.label("Label:");
                let resp = ui.add(egui::TextEdit::singleline(&mut val).desired_width(f32::INFINITY));
                if resp.lost_focus() && val != *label {
                    events.push(CanvasEvent::NodeModified {
                        id: node_id.clone(),
                        update: NodeUpdate::TextContent(val),
                    });
                }
            }
            _ => {
                ui.label(egui::RichText::new("(no editable content)").size(9.).color(Color32::DARK_GRAY));
            }
        }
        ui.add_space(4.);
        ui.separator();

        // ── Styling section ──
        ui.add_space(2.);
        ui.label(egui::RichText::new("STYLING").size(9.).color(Color32::GRAY));
        ui.add_space(2.);

        use egui::widgets::color_picker::{color_edit_button_rgba, Alpha};
        let mut new_style = node.style.clone();
        let mut style_changed = false;

        ui.horizontal(|ui| {
            ui.label("Background:");
            let [r,g,b,a] = new_style.bg_color;
            let mut c = egui::Rgba::from_rgba_unmultiplied(r, g, b, a);
            style_changed |= color_edit_button_rgba(ui, &mut c, Alpha::OnlyBlend).changed();
            new_style.bg_color = [c.r(), c.g(), c.b(), c.a()];
        });
        ui.horizontal(|ui| {
            ui.label("Text Color:");
            let [r,g,b,a] = new_style.text_color;
            let mut c = egui::Rgba::from_rgba_unmultiplied(r, g, b, a);
            style_changed |= color_edit_button_rgba(ui, &mut c, Alpha::OnlyBlend).changed();
            new_style.text_color = [c.r(), c.g(), c.b(), c.a()];
        });
        ui.horizontal(|ui| {
            ui.label("Border Color:");
            let [r,g,b,a] = new_style.border_color;
            let mut c = egui::Rgba::from_rgba_unmultiplied(r, g, b, a);
            style_changed |= color_edit_button_rgba(ui, &mut c, Alpha::OnlyBlend).changed();
            new_style.border_color = [c.r(), c.g(), c.b(), c.a()];
        });
        style_changed |= ui.add(egui::Slider::new(&mut new_style.border_radius, 0.0..=50.0).text("Border Radius")).changed();
        style_changed |= ui.add(egui::Slider::new(&mut new_style.border_width, 0.0..=10.0).text("Border Width")).changed();

        if style_changed {
            events.push(CanvasEvent::NodeModified {
                id: node_id.clone(),
                update: NodeUpdate::NodeStyle(new_style),
            });
        }

        // ── Data Binding ──
        let is_input = matches!(node.node_type,
            NodeType::TextInput { .. } | NodeType::Dropdown { .. }
            | NodeType::NumberField { .. } | NodeType::Checkbox { .. }
        );
        if is_input {
            ui.add_space(4.);
            ui.separator();
            ui.add_space(2.);
            ui.label(egui::RichText::new("DATABASE BINDING").size(9.).color(Color32::GRAY));

            let (cur_entity, cur_field) = match &node.node_type {
                NodeType::TextInput { bound_entity, bound_field, .. }
                | NodeType::Dropdown { bound_entity, bound_field, .. }
                | NodeType::NumberField { bound_entity, bound_field, .. }
                | NodeType::Checkbox { bound_entity, bound_field, .. } => {
                    (bound_entity.clone().unwrap_or_default(), bound_field.clone().unwrap_or_default())
                }
                _ => (String::new(), String::new()),
            };

            let mut entity = cur_entity.clone();
            let mut field = cur_field.clone();
            let mut changed = false;
            ui.add_space(2.);
            ui.horizontal(|ui| {
                ui.label("Entity (Table):");
                if ui.add(egui::TextEdit::singleline(&mut entity).desired_width(f32::INFINITY)).lost_focus() {
                    changed = true;
                }
            });
            ui.horizontal(|ui| {
                ui.label("Field (Column):");
                if ui.add(egui::TextEdit::singleline(&mut field).desired_width(f32::INFINITY)).lost_focus() {
                    changed = true;
                }
            });
            if changed {
                events.push(CanvasEvent::NodeModified {
                    id: node_id.clone(),
                    update: NodeUpdate::DataBinding {
                        entity: if entity.is_empty() { None } else { Some(entity.clone()) },
                        field: if field.is_empty() { None } else { Some(field.clone()) },
                    },
                });
            }
        }

        // ── Corner radius ──
        ui.add_space(2.);
        let uniform = node.styling.corner_radius[0];
        let mut new_r = uniform;
        if ui.add(egui::Slider::new(&mut new_r, 0.0..=40.0).text("Radius")).changed()
            && (new_r - uniform).abs() > f32::EPSILON
        {
            events.push(CanvasEvent::NodeModified {
                id: node_id.clone(),
                update: NodeUpdate::CornerRadius([new_r, new_r, new_r, new_r]),
            });
        }

        // ── Delete ──
        ui.add_space(8.);
        ui.separator();
        ui.add_space(4.);
        if ui.add(egui::Button::new("🗑 Delete Node").fill(egui::Color32::from_rgb(200, 50, 50)).min_size(egui::vec2(ui.available_width(), 28.))).clicked() {
            events.push(CanvasEvent::DeleteNode { id: node_id.clone() });
        }
    });

    events
}
