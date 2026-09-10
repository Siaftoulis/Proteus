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
    ui.allocate_ui(Vec2::new(ui.available_width(), height), |ui| {
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

        ScrollArea::vertical().id_salt("inspector_layers").show(ui, |ui| {
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
                        NodeType::Image { .. } => "◫",
                        NodeType::Shape { .. } => "◻",
                    };

                    ui.horizontal(|ui| {
                        // Clickable node row
                        let row_label = format!("{} {}", icon, n.name);
                        let text_color = if is_sel { theme::ACCENT } else { theme::TEXT };
                        if ui.selectable_label(is_sel, RichText::new(row_label).size(10.).color(text_color)).clicked() {
                            events.push(CanvasEvent::NodeClicked {
                                id: nid.clone(),
                                shift_held: false,
                            });
                        }

                        // Right action buttons: Lock, Visibility, Delete
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // Delete
                            if ui.add(egui::Button::new("✕").min_size(Vec2::new(14., 14.))).clicked() {
                                events.push(CanvasEvent::DeleteNode { id: nid.clone() });
                            }
                            // Lock toggle
                            let lock_icon = if n.locked { "🔒" } else { "🔓" };
                            if ui.add(egui::Button::new(lock_icon).min_size(Vec2::new(16., 14.))).clicked() {
                                events.push(CanvasEvent::NodeModified {
                                    id: nid.clone(),
                                    update: NodeUpdate::ToggleLock,
                                });
                            }
                            // Visibility toggle
                            let vis_icon = if n.visible { "👁" } else { "🚫" };
                            if ui.add(egui::Button::new(vis_icon).min_size(Vec2::new(16., 14.))).clicked() {
                                events.push(CanvasEvent::NodeModified {
                                    id: nid.clone(),
                                    update: NodeUpdate::ToggleVisibility,
                                });
                            }
                        });
                    });
                }
            }
        });
    });
}
