//! Lunacy-Grade Precision Inspector for Geometry, Alignment, Fills & Typography.
//! Provides pixel-perfect coordinate scrubbers, aspect-ratio locking, 4-corner radii,
//! multi-selection alignment/distribution, typography controls, and luxury color swatches.

use eframe::egui::{self, RichText, Vec2};
use crate::scene::{CanvasEvent, Node, NodeType, NodeUpdate, ProjectDocument, Sizing};
use crate::theme;
use super::controls::{color_picker_row, info_button, luxury_color_swatches};
use super::mutations::*;

/// Multi-selection bounding box in canvas coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SelectionBounds {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

impl SelectionBounds {
    pub fn width(&self) -> f32 { (self.max_x - self.min_x).max(1.0) }
    pub fn height(&self) -> f32 { (self.max_y - self.min_y).max(1.0) }
}

/// Calculate bounding box covering all nodes in the selection.
pub fn compute_selection_bounds(nodes: &[&Node]) -> Option<SelectionBounds> {
    if nodes.is_empty() { return None; }
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;

    for n in nodes {
        let (w, h) = match (&n.layout.width, &n.layout.height) {
            (Sizing::Fixed(w), Sizing::Fixed(h)) => (*w, *h),
            (Sizing::Fixed(w), _) => (*w, 40.0),
            (_, Sizing::Fixed(h)) => (100.0, *h),
            _ => (100.0, 40.0),
        };
        min_x = min_x.min(n.position.0);
        min_y = min_y.min(n.position.1);
        max_x = max_x.max(n.position.0 + w);
        max_y = max_y.max(n.position.1 + h);
    }
    Some(SelectionBounds { min_x, min_y, max_x, max_y })
}

/// Calculate even distribution positions along a 1D axis.
pub fn calculate_even_distribution(mut items: Vec<(String, f32, f32)>, total_span: (f32, f32)) -> Vec<(String, f32)> {
    if items.len() < 2 { return items.into_iter().map(|(id, pos, _)| (id, pos)).collect(); }
    items.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    let count = items.len();
    let total_item_size: f32 = items.iter().map(|(_, _, size)| size).sum();
    let remaining_gap = ((total_span.1 - total_span.0) - total_item_size).max(0.0);
    let gap = remaining_gap / (count - 1) as f32;

    let mut current_pos = total_span.0;
    let mut result = Vec::with_capacity(count);
    for (id, _, size) in items {
        result.push((id, current_pos));
        current_pos += size + gap;
    }
    result
}

/// Calculate proportional dimension when aspect ratio is constrained.
pub fn calculate_constrained_dimension(old_dim: (f32, f32), new_w: Option<f32>, new_h: Option<f32>) -> (f32, f32) {
    let ratio = if old_dim.1 > 0.0 { old_dim.0 / old_dim.1 } else { 1.0 };
    if let Some(w) = new_w {
        let h = if ratio > 0.0 { (w / ratio).round() } else { old_dim.1 };
        (w, h.max(5.0))
    } else if let Some(h) = new_h {
        ((h * ratio).round().max(5.0), h)
    } else {
        old_dim
    }
}

/// Draw Precision Geometry with X, Y, W, H, aspect ratio lock, and rotation.
pub fn draw_precision_geometry(ui: &mut egui::Ui, node: &Node, events: &mut Vec<CanvasEvent>) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("GEOMETRY").size(9.0).color(theme::TEXT_DIM).strong());
        info_button(ui, "Pixel-exact placement, constrained aspect ratio and rotation.");
    });

    let (cur_w, cur_h) = match (&node.layout.width, &node.layout.height) {
        (Sizing::Fixed(w), Sizing::Fixed(h)) => (*w, *h),
        (Sizing::Fixed(w), _) => (*w, 100.0),
        (_, Sizing::Fixed(h)) => (200.0, *h),
        _ => (200.0, 100.0),
    };

    let lock_id = ui.make_persistent_id(format!("aspect_lock_{}", node.id));
    let mut aspect_locked = ui.data_mut(|d| *d.get_temp_mut_or_default::<bool>(lock_id));

    let mut new_x = node.position.0;
    let mut new_y = node.position.1;
    let mut new_w = cur_w;
    let mut new_h = cur_h;
    let mut pos_changed = false;
    let mut w_changed = false;
    let mut h_changed = false;

    ui.horizontal(|ui| {
        ui.label(RichText::new("X").size(10.0).color(theme::FROST_GRAY));
        pos_changed |= ui.add(egui::DragValue::new(&mut new_x).speed(1.0).custom_formatter(|n, _| format!("{:.0} px", n))).changed();
        ui.add_space(4.0);
        ui.label(RichText::new("Y").size(10.0).color(theme::FROST_GRAY));
        pos_changed |= ui.add(egui::DragValue::new(&mut new_y).speed(1.0).custom_formatter(|n, _| format!("{:.0} px", n))).changed();
    });

    ui.horizontal(|ui| {
        ui.label(RichText::new("W").size(10.0).color(theme::FROST_GRAY));
        w_changed |= ui.add(egui::DragValue::new(&mut new_w).speed(1.0).range(5.0..=4000.0).custom_formatter(|n, _| format!("{:.0} px", n))).changed();
        let lock_icon = if aspect_locked { "🔗" } else { "🔓" };
        if ui.button(lock_icon).on_hover_text("Lock/Unlock Aspect Ratio").clicked() {
            aspect_locked = !aspect_locked;
            ui.data_mut(|d| d.insert_temp(lock_id, aspect_locked));
        }
        ui.label(RichText::new("H").size(10.0).color(theme::FROST_GRAY));
        h_changed |= ui.add(egui::DragValue::new(&mut new_h).speed(1.0).range(5.0..=4000.0).custom_formatter(|n, _| format!("{:.0} px", n))).changed();
    });

    let mut new_rot = node.rotation;
    let mut rot_changed = false;

    ui.horizontal(|ui| {
        ui.label(RichText::new("↻").size(11.0).color(theme::FROST_GRAY));
        rot_changed |= ui.add(
            egui::DragValue::new(&mut new_rot)
                .speed(1.0)
                .range(-180.0..=180.0)
                .custom_formatter(|n, _| format!("{:.0}°", n))
        ).on_hover_text("Rotation angle (-180° to +180°). Shift-drag handles on canvas for 15° snap.").changed();

        if ui.add(
            egui::Button::new(RichText::new("0°").size(9.5))
                .min_size(Vec2::new(24.0, 18.0))
        ).on_hover_text("Reset rotation to 0°").clicked() {
            new_rot = 0.0;
            rot_changed = true;
        }
    });

    if pos_changed { events.push(move_node(node.id.clone(), new_x, new_y)); }
    if rot_changed {
        events.push(CanvasEvent::NodeModified {
            id: node.id.clone(),
            update: crate::scene::NodeUpdate::Rotate(new_rot),
        });
    }
    if w_changed || h_changed {
        if aspect_locked {
            let (fw, fh) = if w_changed {
                calculate_constrained_dimension((cur_w, cur_h), Some(new_w), None)
            } else {
                calculate_constrained_dimension((cur_w, cur_h), None, Some(new_h))
            };
            events.push(resize_node(node.id.clone(), fw, fh));
        } else {
            events.push(resize_node(node.id.clone(), new_w, new_h));
        }
    }
}

/// Draw Lunacy-Grade Precision Alignment Bar (6 alignment buttons).
pub fn draw_precision_alignment_bar(ui: &mut egui::Ui, node: &Node, doc: &ProjectDocument, events: &mut Vec<CanvasEvent>) {
    let (cur_w, cur_h) = match (&node.layout.width, &node.layout.height) {
        (Sizing::Fixed(w), Sizing::Fixed(h)) => (*w, *h),
        (Sizing::Fixed(w), _) => (*w, 100.0),
        (_, Sizing::Fixed(h)) => (200.0, *h),
        _ => (200.0, 100.0),
    };
    let (ref_w, ref_h) = if let Some(pid) = &node.parent_id {
        doc.get_node(pid).map(|p| match (&p.layout.width, &p.layout.height) {
            (Sizing::Fixed(w), Sizing::Fixed(h)) => (*w, *h),
            _ => (1200.0, 800.0),
        }).unwrap_or((1200.0, 800.0))
    } else {
        (1200.0, 800.0)
    };

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        if ui.add(egui::Button::new("⇤").min_size(Vec2::new(22.0, 20.0))).on_hover_text("Align Left").clicked() {
            events.push(move_node(node.id.clone(), 0.0, node.position.1));
        }
        if ui.add(egui::Button::new("⇋").min_size(Vec2::new(22.0, 20.0))).on_hover_text("Align Center Horizontally").clicked() {
            events.push(move_node(node.id.clone(), ((ref_w - cur_w) / 2.0).max(0.0).round(), node.position.1));
        }
        if ui.add(egui::Button::new("⇥").min_size(Vec2::new(22.0, 20.0))).on_hover_text("Align Right").clicked() {
            events.push(move_node(node.id.clone(), (ref_w - cur_w).max(0.0).round(), node.position.1));
        }
        ui.separator();
        if ui.add(egui::Button::new("⤒").min_size(Vec2::new(22.0, 20.0))).on_hover_text("Align Top").clicked() {
            events.push(move_node(node.id.clone(), node.position.0, 0.0));
        }
        if ui.add(egui::Button::new("⥯").min_size(Vec2::new(22.0, 20.0))).on_hover_text("Align Center Vertically").clicked() {
            events.push(move_node(node.id.clone(), node.position.0, ((ref_h - cur_h) / 2.0).max(0.0).round()));
        }
        if ui.add(egui::Button::new("⤓").min_size(Vec2::new(22.0, 20.0))).on_hover_text("Align Bottom").clicked() {
            events.push(move_node(node.id.clone(), node.position.0, (ref_h - cur_h).max(0.0).round()));
        }
    });
}

/// Draw multi-selection inspector for batch alignment and even distribution.
pub fn draw_multi_selection_inspector(ui: &mut egui::Ui, doc: &ProjectDocument, selected_ids: &[String], events: &mut Vec<CanvasEvent>) {
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label(RichText::new(format!("{} LAYERS SELECTED", selected_ids.len())).size(10.0).strong().color(theme::SKY_CYAN));
    });

    let selected_nodes: Vec<&Node> = selected_ids.iter().filter_map(|id| doc.get_node(id)).collect();
    if let Some(bounds) = compute_selection_bounds(&selected_nodes) {
        ui.add_space(2.0);
        ui.label(RichText::new("SELECTION ALIGNMENT").size(8.5).color(theme::TEXT_DIM));
        ui.horizontal(|ui| {
            if ui.button("⇤ Left").clicked() {
                for n in &selected_nodes { events.push(move_node(n.id.clone(), bounds.min_x, n.position.1)); }
            }
            if ui.button("⇋ Center").clicked() {
                for n in &selected_nodes {
                    let w = match &n.layout.width { Sizing::Fixed(w) => *w, _ => 100.0 };
                    events.push(move_node(n.id.clone(), (bounds.min_x + (bounds.width() - w) / 2.0).round(), n.position.1));
                }
            }
            if ui.button("⇥ Right").clicked() {
                for n in &selected_nodes {
                    let w = match &n.layout.width { Sizing::Fixed(w) => *w, _ => 100.0 };
                    events.push(move_node(n.id.clone(), (bounds.max_x - w).round(), n.position.1));
                }
            }
        });

        ui.horizontal(|ui| {
            if ui.button("⤒ Top").clicked() {
                for n in &selected_nodes { events.push(move_node(n.id.clone(), n.position.0, bounds.min_y)); }
            }
            if ui.button("⥯ Middle").clicked() {
                for n in &selected_nodes {
                    let h = match &n.layout.height { Sizing::Fixed(h) => *h, _ => 40.0 };
                    events.push(move_node(n.id.clone(), n.position.0, (bounds.min_y + (bounds.height() - h) / 2.0).round()));
                }
            }
            if ui.button("⤓ Bottom").clicked() {
                for n in &selected_nodes {
                    let h = match &n.layout.height { Sizing::Fixed(h) => *h, _ => 40.0 };
                    events.push(move_node(n.id.clone(), n.position.0, (bounds.max_y - h).round()));
                }
            }
        });

        ui.add_space(4.0);
        ui.separator();
        ui.label(RichText::new("DISTRIBUTION").size(8.5).color(theme::TEXT_DIM));
        ui.horizontal(|ui| {
            if ui.button("⫢ Distribute Horizontally").clicked() {
                let items: Vec<(String, f32, f32)> = selected_nodes.iter().map(|n| {
                    (n.id.clone(), n.position.0, match &n.layout.width { Sizing::Fixed(w) => *w, _ => 100.0 })
                }).collect();
                for (id, nx) in calculate_even_distribution(items, (bounds.min_x, bounds.max_x)) {
                    if let Some(n) = doc.get_node(&id) { events.push(move_node(id, nx.round(), n.position.1)); }
                }
            }
        });
        ui.horizontal(|ui| {
            if ui.button("⫯ Distribute Vertically").clicked() {
                let items: Vec<(String, f32, f32)> = selected_nodes.iter().map(|n| {
                    (n.id.clone(), n.position.1, match &n.layout.height { Sizing::Fixed(h) => *h, _ => 40.0 })
                }).collect();
                for (id, ny) in calculate_even_distribution(items, (bounds.min_y, bounds.max_y)) {
                    if let Some(n) = doc.get_node(&id) { events.push(move_node(id, n.position.0, ny.round())); }
                }
            }
        });
    }
}

/// Draw Typography Section with font family, size, weights, and color.
pub fn draw_precision_typography(ui: &mut egui::Ui, node: &Node, events: &mut Vec<CanvasEvent>) {
    if let NodeType::Text { content, font } = &node.node_type {
        ui.add_space(2.0);
        ui.label(RichText::new("TYPOGRAPHY").size(9.0).color(theme::TEXT_DIM).strong());
        let mut val = content.clone();
        ui.horizontal(|ui| {
            ui.label(RichText::new("Content:").size(10.0));
            if ui.add(egui::TextEdit::singleline(&mut val).desired_width(140.0)).changed() {
                events.push(CanvasEvent::NodeModified { id: node.id.clone(), update: NodeUpdate::TextContent(val) });
            }
        });

        let families = ["Inter", "SF Pro Display", "Roboto", "JetBrains Mono", "Outfit"];
        let mut cur_fam = font.family.clone();
        ui.horizontal(|ui| {
            ui.label(RichText::new("Family:").size(10.0));
            egui::ComboBox::from_id_salt(format!("font_fam_{}", node.id))
                .selected_text(&cur_fam)
                .show_ui(ui, |ui| {
                    for fam in families {
                        if ui.selectable_value(&mut cur_fam, fam.to_string(), fam).clicked() {
                            events.push(update_font_family(node.id.clone(), cur_fam.clone()));
                        }
                    }
                });
        });

        let mut sz = font.size;
        let mut b = font.weight >= 600;
        ui.horizontal(|ui| {
            if ui.add(egui::Slider::new(&mut sz, 8.0..=96.0).text("Size")).changed() {
                events.push(update_font_size(node.id.clone(), sz));
            }
            if ui.checkbox(&mut b, "Bold").changed() {
                events.push(update_font_weight(node.id.clone(), if b { 700 } else { 400 }));
            }
        });
    }
}

/// Draw Appearance Section with Fills, Borders, 4-Corner Radii & Opacity.
pub fn draw_precision_appearance(ui: &mut egui::Ui, node: &Node, events: &mut Vec<CanvasEvent>) {
    ui.add_space(4.0);
    ui.separator();
    ui.horizontal(|ui| {
        ui.label(RichText::new("FILL, STROKE & CORNERS").size(9.0).color(theme::TEXT_DIM).strong());
        info_button(ui, "RGBA fills, border thickness, discrete 4-corner radii and opacity.");
    });

    let mut new_style = node.style.clone();
    let mut style_changed = false;
    style_changed |= color_picker_row(ui, "Fill Color:", &mut new_style.bg_color);
    luxury_color_swatches(ui, |rgba| {
        new_style.bg_color = rgba;
        style_changed = true;
    });

    if matches!(node.node_type, NodeType::Text { .. }) {
        style_changed |= color_picker_row(ui, "Text Color:", &mut new_style.text_color);
    }

    style_changed |= color_picker_row(ui, "Border Color:", &mut new_style.border_color);
    style_changed |= ui.add(egui::Slider::new(&mut new_style.border_width, 0.0..=12.0).text("Border Width")).changed();

    let uniform_id = ui.make_persistent_id(format!("uniform_corners_{}", node.id));
    let mut uniform_mode = ui.data_mut(|d| *d.get_temp_mut_or_default::<bool>(uniform_id));

    ui.horizontal(|ui| {
        let mode_btn = if uniform_mode { "◱ Uniform" } else { "◰ 4-Corners" };
        if ui.button(mode_btn).on_hover_text("Toggle uniform or discrete corner radii").clicked() {
            uniform_mode = !uniform_mode;
            ui.data_mut(|d| d.insert_temp(uniform_id, uniform_mode));
        }
        if uniform_mode {
            let mut r = new_style.border_radius;
            if ui.add(egui::Slider::new(&mut r, 0.0..=60.0).text("Radius")).changed() {
                new_style.border_radius = r;
                style_changed = true;
            }
        }
    });

    if !uniform_mode {
        let mut cr = node.styling.corner_radius;
        let mut cr_changed = false;
        ui.horizontal(|ui| {
            ui.label("TL"); cr_changed |= ui.add(egui::DragValue::new(&mut cr[0]).speed(1.0).range(0.0..=100.0)).changed();
            ui.label("TR"); cr_changed |= ui.add(egui::DragValue::new(&mut cr[1]).speed(1.0).range(0.0..=100.0)).changed();
        });
        ui.horizontal(|ui| {
            ui.label("BL"); cr_changed |= ui.add(egui::DragValue::new(&mut cr[3]).speed(1.0).range(0.0..=100.0)).changed();
            ui.label("BR"); cr_changed |= ui.add(egui::DragValue::new(&mut cr[2]).speed(1.0).range(0.0..=100.0)).changed();
        });
        if cr_changed { events.push(CanvasEvent::NodeModified { id: node.id.clone(), update: NodeUpdate::CornerRadius(cr) }); }
    }

    let mut op = node.styling.opacity;
    ui.horizontal(|ui| {
        ui.label(RichText::new("Opacity").size(10.0));
        if ui.add(egui::Slider::new(&mut op, 0.0..=1.0).custom_formatter(|n, _| format!("{:.0}%", n * 100.0))).changed() {
            events.push(update_opacity(node.id.clone(), op));
        }
    });

    if style_changed {
        let r = new_style.border_radius;
        events.push(update_node_style(node.id.clone(), new_style));
        events.push(update_corner_radius(node.id.clone(), r));
    }
}

/// Primary entry point for rendering the precision design tab.
pub fn draw_precision_design_tab(
    ui: &mut egui::Ui,
    node: &Node,
    doc: &ProjectDocument,
    copied_dimensions: &mut Option<(f32, f32)>,
    events: &mut Vec<CanvasEvent>,
) {
    draw_precision_geometry(ui, node, events);
    ui.separator();
    draw_precision_alignment_bar(ui, node, doc, events);
    ui.separator();

    crate::inspector::design_tab::draw_smart_role_selector(ui, node, events);
    crate::inspector::design_tab::draw_input_props(ui, node, events);
    draw_precision_typography(ui, node, events);
    crate::inspector::design_tab::draw_button_visual_props(ui, node, events);
    crate::inspector::design_tab::draw_dynamic_form_props(ui, node, events);

    draw_precision_appearance(ui, node, events);

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        let (cw, ch) = match (&node.layout.width, &node.layout.height) {
            (Sizing::Fixed(w), Sizing::Fixed(h)) => (*w, *h),
            _ => (100.0, 40.0),
        };
        if ui.button("📋 Copy Size").clicked() { *copied_dimensions = Some((cw, ch)); }
        if let Some((pw, ph)) = *copied_dimensions {
            if ui.button(format!("Paste ({:.0}×{:.0})", pw, ph)).clicked() {
                events.push(resize_node(node.id.clone(), pw, ph));
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constrained_dimension_aspect_ratio() {
        let old = (200.0, 100.0);
        let (new_w, new_h) = calculate_constrained_dimension(old, Some(400.0), None);
        assert_eq!((new_w, new_h), (400.0, 200.0));
        let (new_w2, new_h2) = calculate_constrained_dimension(old, None, Some(50.0));
        assert_eq!((new_w2, new_h2), (100.0, 50.0));
    }

    #[test]
    fn test_even_distribution_calculation() {
        let items = vec![
            ("item1".to_string(), 0.0, 20.0),
            ("item2".to_string(), 30.0, 20.0),
            ("item3".to_string(), 80.0, 20.0),
        ];
        let res = calculate_even_distribution(items, (0.0, 100.0));
        assert_eq!(res.len(), 3);
        assert_eq!(res[0], ("item1".to_string(), 0.0));
        assert_eq!(res[1], ("item2".to_string(), 40.0));
        assert_eq!(res[2], ("item3".to_string(), 80.0));
    }

    #[test]
    fn test_selection_bounds_computation() {
        let mut n1 = Node::new("n1".into(), "N1".into(), NodeType::Frame);
        n1.position = (10.0, 20.0);
        n1.layout.width = Sizing::Fixed(100.0);
        n1.layout.height = Sizing::Fixed(50.0);

        let mut n2 = Node::new("n2".into(), "N2".into(), NodeType::Frame);
        n2.position = (150.0, 40.0);
        n2.layout.width = Sizing::Fixed(80.0);
        n2.layout.height = Sizing::Fixed(100.0);

        let bounds = compute_selection_bounds(&[&n1, &n2]).unwrap();
        assert_eq!(bounds.min_x, 10.0);
        assert_eq!(bounds.min_y, 20.0);
        assert_eq!(bounds.max_x, 230.0);
        assert_eq!(bounds.max_y, 140.0);
        assert_eq!(bounds.width(), 220.0);
        assert_eq!(bounds.height(), 120.0);
    }
}
