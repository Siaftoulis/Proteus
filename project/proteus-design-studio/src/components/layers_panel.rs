use eframe::egui::{self, Color32, RichText, ScrollArea, Vec2};
use crate::scene::{CanvasEvent, NodeType, NodeUpdate, ProjectDocument};
use crate::theme;

pub fn draw_layers_panel(
    ui: &mut egui::Ui,
    doc: &ProjectDocument,
    selected_id: Option<&str>,
    height: f32,
    events: &mut Vec<CanvasEvent>,
) {
    let max_w = ui.available_width();
    ui.allocate_ui(Vec2::new(max_w, height), |ui| {
        ui.set_max_width(max_w);
        ui.horizontal(|ui| {
            ui.label(RichText::new("LAYERS").size(9.).color(theme::TEXT_DIM).strong());
            let resp = ui.add(
                egui::Button::new(RichText::new("ℹ").size(10.).color(Color32::from_rgb(0, 180, 255)))
                    .fill(Color32::from_rgb(35, 45, 55))
                    .min_size(Vec2::new(16., 16.)),
            );
            resp.on_hover_text("Ιεραρχία στοιχείων. Πατήστε για επιλογή, κλειδώστε (🔒) ή αποκρύψτε (👁).");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(RichText::new(format!("Total: {}", doc.nodes.len())).size(8.).color(Color32::GRAY));
            });
        });
        ui.add_space(2.);

        ScrollArea::vertical()
            .id_salt("inspector_layers")
            .max_width(max_w)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_max_width(max_w);
                ui.spacing_mut().item_spacing = Vec2::new(3.0, 2.0);
                let mut node_ids: Vec<String> = doc.nodes.keys().cloned().collect();
                node_ids.sort();

                for nid in &node_ids {
                    if let Some(n) = doc.get_node(nid) {
                        let is_sel = selected_id == Some(nid.as_str());
                        let icon = match &n.node_type {
                            NodeType::Frame | NodeType::Page | NodeType::Group => "📦",
                            NodeType::TextInput { .. } | NodeType::NumberField { .. } | NodeType::Dropdown { .. } | NodeType::Checkbox { .. } => "📝",
                            NodeType::Text { .. } => "🏷",
                            NodeType::Button { .. } => "🔘",
                            NodeType::Table { .. } => "⊞",
                            NodeType::DynamicForm { .. } => "📋",
                            NodeType::Image { .. } => "◫",
                            NodeType::Shape { .. } => "◻",
                        };

                        let lock_icon = if n.locked { "🔒" } else { "🔓" };
                        let vis_icon = if n.visible { "👁" } else { "🚫" };

                        ui.horizontal(|ui| {
                            ui.set_max_width(max_w);
                            let display_name = if n.name.chars().count() > 10 {
                                let prefix: String = n.name.chars().take(9).collect();
                                format!("{}…", prefix)
                            } else {
                                n.name.clone()
                            };
                            let row_label = format!("{} {}", icon, display_name);
                            let text_color = if is_sel { theme::ACCENT } else { theme::TEXT };

                            let btn = egui::SelectableLabel::new(
                                is_sel,
                                RichText::new(row_label).size(9.5).color(text_color),
                            );
                            let resp = ui.add_sized([92.0, 18.0], btn);
                            if resp.clicked() {
                                events.push(CanvasEvent::NodeClicked {
                                    id: nid.clone(),
                                    shift_held: false,
                                });
                            }
                            resp.on_hover_text(&n.name);

                            if ui.add_sized([16.0, 16.0], egui::Button::new(vis_icon)).on_hover_text("Toggle visibility").clicked() {
                                events.push(CanvasEvent::NodeModified {
                                    id: nid.clone(),
                                    update: NodeUpdate::ToggleVisibility,
                                });
                            }
                            if ui.add_sized([16.0, 16.0], egui::Button::new(lock_icon)).on_hover_text("Toggle lock").clicked() {
                                events.push(CanvasEvent::NodeModified {
                                    id: nid.clone(),
                                    update: NodeUpdate::ToggleLock,
                                });
                            }
                            if ui.add_sized([16.0, 16.0], egui::Button::new("✕")).on_hover_text("Delete node").clicked() {
                                events.push(CanvasEvent::DeleteNode { id: nid.clone() });
                            }
                        });
                    }
                }
            });
    });
}
