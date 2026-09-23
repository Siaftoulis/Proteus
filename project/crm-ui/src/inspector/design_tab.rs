//! Penpot-inspired Design Tab for the Proteus Inspector.
//! Covers geometry, alignment, typography, role selection, and visual styles.

use eframe::egui::{self, Color32, RichText, Vec2};
use crate::scene::{
    ButtonStyle, CanvasEvent, FieldType, Node, NodeType, NodeUpdate,
    ProjectDocument, Sizing,
};
use crate::theme;
use super::controls::{color_picker_row, info_button, luxury_color_swatches};
use super::mutations::*;

pub fn draw_design_tab(
    ui: &mut egui::Ui,
    node: &Node,
    doc: &ProjectDocument,
    copied_dimensions: &mut Option<(f32, f32)>,
    events: &mut Vec<CanvasEvent>,
) {
    // 1. Transform & Size with Penpot Alignment Bar
    draw_transform_props(ui, node, doc, copied_dimensions, events);

    ui.separator();

    // 2. Smart Box Role Selector
    draw_smart_role_selector(ui, node, events);

    // 3. Type-specific visual props
    draw_input_props(ui, node, events);
    draw_typography_props(ui, node, events);
    draw_button_visual_props(ui, node, events);

    // 4. Styling & Appearance (Fills, Stroke, Radii)
    draw_styling_props(ui, node, events);
}

pub fn draw_transform_props(
    ui: &mut egui::Ui,
    node: &Node,
    doc: &ProjectDocument,
    copied_dimensions: &mut Option<(f32, f32)>,
    events: &mut Vec<CanvasEvent>,
) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("GEOMETRY & ALIGNMENT").size(9.).color(theme::TEXT_DIM).strong());
        info_button(ui, "Διαστάσεις (W, H), Θέση (X, Y) και ευθυγράμμιση Penpot.");
    });

    let (cur_w, cur_h) = match (&node.layout.width, &node.layout.height) {
        (Sizing::Fixed(w), Sizing::Fixed(h)) => (*w, *h),
        (Sizing::Fixed(w), _) => (*w, 100.),
        (_, Sizing::Fixed(h)) => (200., *h),
        _ => (200., 100.),
    };

    // Penpot Alignment Toolbar
    ui.horizontal(|ui| {
        let (ref_w, ref_h) = if let Some(pid) = &node.parent_id {
            if let Some(parent) = doc.get_node(pid) {
                match (&parent.layout.width, &parent.layout.height) {
                    (Sizing::Fixed(w), Sizing::Fixed(h)) => (*w, *h),
                    (Sizing::Fixed(w), _) => (*w, 800.),
                    (_, Sizing::Fixed(h)) => (1200., *h),
                    _ => (1200., 800.),
                }
            } else {
                (1200., 800.)
            }
        } else {
            (1200., 800.)
        };

        if ui.add(egui::Button::new("⇤").min_size(Vec2::new(20., 18.))).on_hover_text("Align Left").clicked() {
            events.push(move_node(node.id.clone(), 0.0, node.position.1));
        }
        if ui.add(egui::Button::new("⇋").min_size(Vec2::new(20., 18.))).on_hover_text("Align Center Horizontally").clicked() {
            let center_x = ((ref_w - cur_w) / 2.0).max(0.0).round();
            events.push(move_node(node.id.clone(), center_x, node.position.1));
        }
        if ui.add(egui::Button::new("⇥").min_size(Vec2::new(20., 18.))).on_hover_text("Align Right").clicked() {
            let right_x = (ref_w - cur_w).max(0.0).round();
            events.push(move_node(node.id.clone(), right_x, node.position.1));
        }
        ui.separator();
        if ui.add(egui::Button::new("⤒").min_size(Vec2::new(20., 18.))).on_hover_text("Align Top").clicked() {
            events.push(move_node(node.id.clone(), node.position.0, 0.0));
        }
        if ui.add(egui::Button::new("⥯").min_size(Vec2::new(20., 18.))).on_hover_text("Align Center Vertically").clicked() {
            let center_y = ((ref_h - cur_h) / 2.0).max(0.0).round();
            events.push(move_node(node.id.clone(), node.position.0, center_y));
        }
        if ui.add(egui::Button::new("⤓").min_size(Vec2::new(20., 18.))).on_hover_text("Align Bottom").clicked() {
            let bottom_y = (ref_h - cur_h).max(0.0).round();
            events.push(move_node(node.id.clone(), node.position.0, bottom_y));
        }
    });

    let mut new_x = node.position.0;
    let mut new_y = node.position.1;
    let mut new_w = cur_w;
    let mut new_h = cur_h;
    let mut dim_changed = false;
    let mut pos_changed = false;

    ui.horizontal(|ui| {
        ui.label(RichText::new("X:").size(10.).color(Color32::GRAY));
        pos_changed |= ui.add(egui::DragValue::new(&mut new_x).speed(1.0)).changed();
        ui.label(RichText::new("Y:").size(10.).color(Color32::GRAY));
        pos_changed |= ui.add(egui::DragValue::new(&mut new_y).speed(1.0)).changed();
    });
    ui.horizontal(|ui| {
        ui.label(RichText::new("W:").size(10.).color(Color32::GRAY));
        dim_changed |= ui.add(egui::DragValue::new(&mut new_w).speed(1.0).range(10.0..=2000.0)).changed();
        ui.label(RichText::new("H:").size(10.).color(Color32::GRAY));
        dim_changed |= ui.add(egui::DragValue::new(&mut new_h).speed(1.0).range(10.0..=2000.0)).changed();
    });

    if pos_changed {
        events.push(move_node(node.id.clone(), new_x, new_y));
    }
    if dim_changed {
        events.push(resize_node(node.id.clone(), new_w, new_h));
    }

    ui.horizontal(|ui| {
        if ui.button("📋 Copy Size").on_hover_text("Copy width and height").clicked() {
            *copied_dimensions = Some((cur_w, cur_h));
        }
        if let Some((pw, ph)) = *copied_dimensions {
            if ui.button(format!("Paste ({:.0}x{:.0})", pw, ph)).clicked() {
                events.push(resize_node(node.id.clone(), pw, ph));
            }
        }
    });
}

pub fn draw_smart_role_selector(ui: &mut egui::Ui, node: &Node, events: &mut Vec<CanvasEvent>) {
    ui.add_space(2.);
    ui.label(RichText::new("SMART COMPONENT ROLE:").size(8.).color(Color32::GRAY));

    let is_text = matches!(node.node_type, NodeType::Text { .. });
    let is_btn = matches!(node.node_type, NodeType::Button { .. });
    let is_inp = matches!(node.node_type, NodeType::TextInput { .. });
    let is_tbl = matches!(node.node_type, NodeType::Table { .. });
    let is_frame = matches!(node.node_type, NodeType::Frame | NodeType::Page);

    ui.horizontal_wrapped(|ui| {
        if ui.selectable_label(is_text, "🏷 Text").clicked() && !is_text {
            events.push(change_node_type(node.id.clone(), NodeType::Text {
                content: "Sample Text".into(),
                font: crate::scene::FontSpec::default(),
            }));
        }
        if ui.selectable_label(is_btn, "🔘 Button").clicked() && !is_btn {
            events.push(change_node_type(node.id.clone(), NodeType::Button {
                label: "Action".into(),
                style: ButtonStyle::Primary,
            }));
        }
        if ui.selectable_label(is_inp, "📝 Input").clicked() && !is_inp {
            events.push(change_node_type(node.id.clone(), NodeType::TextInput {
                placeholder: "Enter value...".into(),
                field_type: FieldType::Text,
                bound_entity: None,
                bound_field: None,
            }));
        }
        if ui.selectable_label(is_tbl, "⊞ Table").clicked() && !is_tbl {
            events.push(change_node_type(node.id.clone(), NodeType::Table {
                bound_entity: Some("contacts".into()),
                columns: vec!["ID".into(), "Name".into(), "Email".into(), "Status".into()],
            }));
        }
        if ui.selectable_label(is_frame, "📦 Card").clicked() && !is_frame {
            events.push(change_node_type(node.id.clone(), NodeType::Frame));
        }
    });
}

pub fn draw_input_props(ui: &mut egui::Ui, node: &Node, events: &mut Vec<CanvasEvent>) {
    if let NodeType::TextInput { placeholder, .. } = &node.node_type {
        let mut ph = placeholder.clone();
        ui.horizontal(|ui| {
            ui.label(RichText::new("Placeholder:").size(10.));
            if ui.add(egui::TextEdit::singleline(&mut ph).desired_width(130.)).changed() {
                events.push(CanvasEvent::NodeModified {
                    id: node.id.clone(),
                    update: NodeUpdate::Placeholder(ph),
                });
            }
        });
    }
}

pub fn draw_typography_props(ui: &mut egui::Ui, node: &Node, events: &mut Vec<CanvasEvent>) {
    if let NodeType::Text { content, font } = &node.node_type {
        ui.add_space(2.);
        ui.label(RichText::new("TYPOGRAPHY").size(9.).color(theme::TEXT_DIM).strong());

        let mut val = content.clone();
        ui.horizontal(|ui| {
            ui.label("Text:");
            if ui.add(egui::TextEdit::singleline(&mut val).desired_width(140.)).changed() {
                events.push(CanvasEvent::NodeModified {
                    id: node.id.clone(),
                    update: NodeUpdate::TextContent(val),
                });
            }
        });

        let mut sz = font.size;
        let mut b = font.weight >= 600;
        ui.horizontal(|ui| {
            if ui.add(egui::Slider::new(&mut sz, 8.0..=72.0).text("Size")).changed() {
                events.push(update_font_size(node.id.clone(), sz));
            }
            if ui.checkbox(&mut b, "Bold").changed() {
                events.push(update_font_weight(node.id.clone(), if b { 700 } else { 400 }));
            }
        });
    }
}

pub fn draw_button_visual_props(ui: &mut egui::Ui, node: &Node, events: &mut Vec<CanvasEvent>) {
    if let NodeType::Button { label, style } = &node.node_type {
        ui.add_space(2.);
        ui.label(RichText::new("BUTTON APPEARANCE").size(9.).color(theme::TEXT_DIM).strong());
        let mut val = label.clone();
        ui.horizontal(|ui| {
            ui.label("Label:");
            if ui.add(egui::TextEdit::singleline(&mut val).desired_width(140.)).changed() {
                events.push(change_node_type(node.id.clone(), NodeType::Button {
                    label: val,
                    style: style.clone(),
                }));
            }
        });

        ui.horizontal(|ui| {
            let is_p = matches!(style, ButtonStyle::Primary);
            let is_s = matches!(style, ButtonStyle::Secondary);
            let is_d = matches!(style, ButtonStyle::Danger);
            let is_g = matches!(style, ButtonStyle::Ghost);

            if ui.selectable_label(is_p, "Primary").clicked() && !is_p {
                events.push(change_node_type(node.id.clone(), NodeType::Button { label: label.clone(), style: ButtonStyle::Primary }));
            }
            if ui.selectable_label(is_s, "Secondary").clicked() && !is_s {
                events.push(change_node_type(node.id.clone(), NodeType::Button { label: label.clone(), style: ButtonStyle::Secondary }));
            }
            if ui.selectable_label(is_d, "Danger").clicked() && !is_d {
                events.push(change_node_type(node.id.clone(), NodeType::Button { label: label.clone(), style: ButtonStyle::Danger }));
            }
            if ui.selectable_label(is_g, "Ghost").clicked() && !is_g {
                events.push(change_node_type(node.id.clone(), NodeType::Button { label: label.clone(), style: ButtonStyle::Ghost }));
            }
        });
    }
}

pub fn draw_styling_props(ui: &mut egui::Ui, node: &Node, events: &mut Vec<CanvasEvent>) {
    ui.add_space(4.);
    ui.separator();
    ui.horizontal(|ui| {
        ui.label(RichText::new("FILL, STROKE & RADIUS").size(9.).color(theme::TEXT_DIM).strong());
        info_button(ui, "Χρώματα, πάχος περιγράμματος και στρογγυλεμένες γωνίες.");
    });
    ui.add_space(2.);

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
    style_changed |= ui.add(egui::Slider::new(&mut new_style.border_width, 0.0..=10.0).text("Border Width")).changed();
    style_changed |= ui.add(egui::Slider::new(&mut new_style.border_radius, 0.0..=50.0).text("Corner Radius")).changed();

    if style_changed {
        let r = new_style.border_radius;
        events.push(update_node_style(node.id.clone(), new_style));
        events.push(update_corner_radius(node.id.clone(), r));
    }
}
