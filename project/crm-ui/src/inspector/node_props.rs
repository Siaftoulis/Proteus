//! Specific property editors for each node type in the Proteus Inspector.

use eframe::egui::{self, Color32, RichText, Vec2};
use crate::scene::{
    ButtonStyle, CanvasEvent, FieldType, Node, NodeType, NodeUpdate,
    ProjectDocument, Sizing,
};
use crate::theme;
use super::controls::{color_picker_row, info_button, luxury_color_swatches};
use super::mutations::*;

pub fn draw_transform_props(
    ui: &mut egui::Ui,
    node: &Node,
    doc: &ProjectDocument,
    copied_dimensions: &mut Option<(f32, f32)>,
    events: &mut Vec<CanvasEvent>,
) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("TRANSFORM & SIZE").size(9.).color(theme::TEXT_DIM).strong());
        info_button(ui, "Διαστάσεις (W, H) και Θέση (X, Y). Μπορείτε να αντιγράψετε το μέγεθος.");
    });

    let (cur_w, cur_h) = match (&node.layout.width, &node.layout.height) {
        (Sizing::Fixed(w), Sizing::Fixed(h)) => (*w, *h),
        (Sizing::Fixed(w), _) => (*w, 100.),
        (_, Sizing::Fixed(h)) => (200., *h),
        _ => (200., 100.),
    };

    let mut new_x = node.position.0;
    let mut new_y = node.position.1;
    let mut new_w = cur_w;
    let mut new_h = cur_h;
    let mut dim_changed = false;
    let mut pos_changed = false;

    ui.horizontal(|ui| {
        ui.label("X:");
        pos_changed |= ui.add(egui::DragValue::new(&mut new_x).speed(1.0)).changed();
        ui.label("Y:");
        pos_changed |= ui.add(egui::DragValue::new(&mut new_y).speed(1.0)).changed();
    });
    ui.horizontal(|ui| {
        ui.label("W:");
        dim_changed |= ui.add(egui::DragValue::new(&mut new_w).speed(1.0).range(10.0..=2000.0)).changed();
        ui.label("H:");
        dim_changed |= ui.add(egui::DragValue::new(&mut new_h).speed(1.0).range(10.0..=2000.0)).changed();
    });

    if pos_changed {
        events.push(move_node(node.id.clone(), new_x, new_y));
    }
    if dim_changed {
        events.push(resize_node(node.id.clone(), new_w, new_h));
    }

    // Quick Alignment Bar
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

        if ui.add(egui::Button::new("⇤").min_size(Vec2::new(22., 20.))).on_hover_text("Align Left (X = 0)").clicked() {
            events.push(move_node(node.id.clone(), 0.0, node.position.1));
        }
        if ui.add(egui::Button::new("⇋").min_size(Vec2::new(22., 20.))).on_hover_text("Align Center Horizontally").clicked() {
            let center_x = ((ref_w - cur_w) / 2.0).max(0.0).round();
            events.push(move_node(node.id.clone(), center_x, node.position.1));
        }
        if ui.add(egui::Button::new("⇥").min_size(Vec2::new(22., 20.))).on_hover_text("Align Right").clicked() {
            let right_x = (ref_w - cur_w).max(0.0).round();
            events.push(move_node(node.id.clone(), right_x, node.position.1));
        }

        ui.separator();

        if ui.add(egui::Button::new("⤒").min_size(Vec2::new(22., 20.))).on_hover_text("Align Top (Y = 0)").clicked() {
            events.push(move_node(node.id.clone(), node.position.0, 0.0));
        }
        if ui.add(egui::Button::new("⥯").min_size(Vec2::new(22., 20.))).on_hover_text("Align Center Vertically").clicked() {
            let center_y = ((ref_h - cur_h) / 2.0).max(0.0).round();
            events.push(move_node(node.id.clone(), node.position.0, center_y));
        }
        if ui.add(egui::Button::new("⤓").min_size(Vec2::new(22., 20.))).on_hover_text("Align Bottom").clicked() {
            let bottom_y = (ref_h - cur_h).max(0.0).round();
            events.push(move_node(node.id.clone(), node.position.0, bottom_y));
        }
    });

    // Copy / Paste Size buttons
    ui.horizontal(|ui| {
        if ui.add(egui::Button::new("📋 Copy Size").min_size(Vec2::new(70., 18.))).clicked() {
            *copied_dimensions = Some((cur_w, cur_h));
        }
        let paste_enabled = copied_dimensions.is_some();
        if ui.add_enabled(paste_enabled, egui::Button::new("📌 Paste Size").min_size(Vec2::new(70., 18.))).clicked() {
            if let Some((cw, ch)) = *copied_dimensions {
                events.push(resize_node(node.id.clone(), cw, ch));
            }
        }
    });
}

pub fn draw_smart_role_selector(ui: &mut egui::Ui, node: &Node, events: &mut Vec<CanvasEvent>) {
    ui.horizontal(|ui| {
        ui.label(RichText::new("SMART BOX ROLE").size(9.).color(theme::TEXT_DIM).strong());
        info_button(ui, "Ορίστε τη λειτουργία του Smart Box: Shape, Input, Text, Table ή Card.\nΤα Buttons διαχειρίζονται αυτόνομα από το εργαλείο Button.");
    });

    let is_static_shape = matches!(node.node_type, NodeType::Shape { .. });
    let is_input = matches!(
        node.node_type,
        NodeType::TextInput { .. } | NodeType::Dropdown { .. } | NodeType::NumberField { .. } | NodeType::Checkbox { .. }
    );
    let is_text = matches!(node.node_type, NodeType::Text { .. });
    let is_frame = matches!(node.node_type, NodeType::Frame | NodeType::Page | NodeType::Group);
    let is_table = matches!(node.node_type, NodeType::Table { .. });

    ui.horizontal(|ui| {
        if ui.selectable_label(is_static_shape, "◻ Shape").clicked() && !is_static_shape {
            events.push(change_node_type(node.id.clone(), NodeType::Shape { kind: crate::scene::ShapeKind::Rectangle }));
        }
        if ui.selectable_label(is_input, "📝 Input").clicked() && !is_input {
            events.push(change_node_type(node.id.clone(), NodeType::TextInput {
                placeholder: "Type...".into(),
                field_type: FieldType::Text,
                bound_entity: None,
                bound_field: None,
            }));
        }
        if ui.selectable_label(is_text, "🏷 Text").clicked() && !is_text {
            events.push(change_node_type(node.id.clone(), NodeType::Text {
                content: "Label".into(),
                font: crate::scene::FontSpec::default(),
            }));
        }
        if ui.selectable_label(is_table, "⊞ Table").clicked() && !is_table {
            events.push(change_node_type(node.id.clone(), NodeType::Table {
                bound_entity: Some("contacts".into()),
                columns: vec!["ID".into(), "Name".into(), "Email".into(), "Status".into()],
            }));
        }
        if ui.selectable_label(is_frame, "📦 Card").clicked() && !is_frame {
            events.push(change_node_type(node.id.clone(), NodeType::Frame));
        }
    });

    if is_frame {
        ui.add_space(2.);
        ui.horizontal(|ui| {
            ui.label(RichText::new("📦 CARD CONTAINER").size(9.).color(theme::ACCENT).strong());
            info_button(ui, "Η Κάρτα (Card) είναι ένα πλαίσιο ομαδοποίησης στοιχείων με στυλάτο φόντο και στρογγυλεμένες γωνίες.");
        });
    }
}

pub fn draw_input_props(ui: &mut egui::Ui, node: &Node, events: &mut Vec<CanvasEvent>) {
    ui.add_space(2.);
    ui.label(RichText::new("INPUT TYPE:").size(8.).color(Color32::GRAY));
    ui.horizontal(|ui| {
        let is_text_in = matches!(node.node_type, NodeType::TextInput { .. });
        let is_drop_in = matches!(node.node_type, NodeType::Dropdown { .. });
        let is_num_in = matches!(node.node_type, NodeType::NumberField { .. });
        let is_chk_in = matches!(node.node_type, NodeType::Checkbox { .. });

        if ui.selectable_label(is_text_in, "Text").clicked() && !is_text_in {
            events.push(change_node_type(node.id.clone(), NodeType::TextInput {
                placeholder: "Type...".into(),
                field_type: FieldType::Text,
                bound_entity: None,
                bound_field: None,
            }));
        }
        if ui.selectable_label(is_drop_in, "Dropdown").clicked() && !is_drop_in {
            events.push(change_node_type(node.id.clone(), NodeType::Dropdown {
                options: vec!["Option 1".into(), "Option 2".into()],
                multiple: false,
                bound_entity: None,
                bound_field: None,
            }));
        }
        if ui.selectable_label(is_num_in, "Number").clicked() && !is_num_in {
            events.push(change_node_type(node.id.clone(), NodeType::NumberField {
                min: None,
                max: None,
                step: 1.0,
                bound_entity: None,
                bound_field: None,
            }));
        }
        if ui.selectable_label(is_chk_in, "Check").clicked() && !is_chk_in {
            events.push(change_node_type(node.id.clone(), NodeType::Checkbox {
                label: "Confirm".into(),
                bound_entity: None,
                bound_field: None,
            }));
        }
    });

    match &node.node_type {
        NodeType::TextInput { placeholder, .. } => {
            let mut ph = placeholder.clone();
            ui.horizontal(|ui| {
                ui.label("Placeholder:");
                if ui.add(egui::TextEdit::singleline(&mut ph).desired_width(130.)).changed() {
                    events.push(CanvasEvent::NodeModified {
                        id: node.id.clone(),
                        update: NodeUpdate::Placeholder(ph),
                    });
                }
            });
        }
        NodeType::Dropdown { options, .. } => {
            let mut opts_str = options.join(", ");
            ui.horizontal(|ui| {
                ui.label("Options:");
                if ui.add(egui::TextEdit::singleline(&mut opts_str).hint_text("Option 1, Option 2, ...").desired_width(140.)).changed() {
                    let new_opts: Vec<String> = opts_str.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                    events.push(CanvasEvent::NodeModified {
                        id: node.id.clone(),
                        update: NodeUpdate::DropdownOptions(new_opts),
                    });
                }
            });
            ui.label(RichText::new("Χωρίστε τις επιλογές με κόμμα (π.χ. Option 1, Option 2)").size(8.).color(theme::TEXT_MUTED));
        }
        NodeType::Checkbox { label, .. } => {
            let mut lbl = label.clone();
            ui.horizontal(|ui| {
                ui.label("Label:");
                if ui.add(egui::TextEdit::singleline(&mut lbl).desired_width(130.)).changed() {
                    events.push(CanvasEvent::NodeModified {
                        id: node.id.clone(),
                        update: NodeUpdate::CheckboxLabel(lbl),
                    });
                }
            });
        }
        _ => {}
    }

    // Database Link
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
    let mut db_changed = false;

    egui::CollapsingHeader::new(RichText::new("⚙ Database Link (Optional)").size(8.5).color(theme::TEXT_MUTED))
        .id_salt(format!("db_link_{}", node.id))
        .default_open(false)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Table:");
                db_changed |= ui.add(egui::TextEdit::singleline(&mut entity).desired_width(70.)).changed();
                ui.label("Column:");
                db_changed |= ui.add(egui::TextEdit::singleline(&mut field).desired_width(70.)).changed();
            });
        });

    if db_changed {
        events.push(bind_data(
            node.id.clone(),
            if entity.trim().is_empty() { None } else { Some(entity.trim().to_string()) },
            if field.trim().is_empty() { None } else { Some(field.trim().to_string()) },
        ));
    }
}

pub fn draw_typography_props(ui: &mut egui::Ui, node: &Node, events: &mut Vec<CanvasEvent>) {
    if let NodeType::Text { content, font } = &node.node_type {
        ui.add_space(2.);
        ui.horizontal(|ui| {
            ui.label(RichText::new("TYPOGRAPHY & CONTENT").size(9.).color(theme::TEXT_DIM).strong());
            info_button(ui, "Μέγεθος γραμματοσειράς, βάρος και περιεχόμενο κειμένου.");
        });
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

        let mut cur_size = font.size;
        ui.horizontal(|ui| {
            ui.label("Size:");
            if ui.add(egui::Slider::new(&mut cur_size, 8.0..=72.0).suffix("px")).changed() {
                events.push(CanvasEvent::NodeModified {
                    id: node.id.clone(),
                    update: NodeUpdate::FontSize(cur_size),
                });
            }
        });

        ui.horizontal(|ui| {
            for (lbl, sz) in [("H1", 32.0), ("H2", 24.0), ("H3", 18.0), ("Body", 14.0), ("Small", 11.0)] {
                let is_active = (font.size - sz).abs() < 1.0;
                if ui.selectable_label(is_active, lbl).clicked() {
                    events.push(CanvasEvent::NodeModified {
                        id: node.id.clone(),
                        update: NodeUpdate::FontSize(sz),
                    });
                }
            }
        });

        ui.horizontal(|ui| {
            ui.label("Weight:");
            let is_bold = font.weight >= 700;
            if ui.selectable_label(!is_bold, "Regular").clicked() && is_bold {
                events.push(CanvasEvent::NodeModified {
                    id: node.id.clone(),
                    update: NodeUpdate::FontWeight(400),
                });
            }
            if ui.selectable_label(is_bold, "Bold").clicked() && !is_bold {
                events.push(CanvasEvent::NodeModified {
                    id: node.id.clone(),
                    update: NodeUpdate::FontWeight(700),
                });
            }
        });
    }
}

pub fn draw_button_props(ui: &mut egui::Ui, node: &Node, doc: &ProjectDocument, events: &mut Vec<CanvasEvent>) {
    if let NodeType::Button { label, style } = &node.node_type {
        ui.add_space(2.);
        ui.horizontal(|ui| {
            ui.label(RichText::new("BUTTON CONFIGURATION").size(9.).color(theme::TEXT_DIM).strong());
            info_button(ui, "Ετικέτα και οπτικό στυλ κουμπιού (Primary, Secondary, Danger, Ghost).");
        });
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

        // On-Click Action Configuration
        ui.add_space(4.);
        ui.horizontal(|ui| {
            ui.label(RichText::new("⚡ ON-CLICK ACTION").size(9.).color(theme::ACCENT).strong());
            info_button(ui, "Ενέργεια κατά το πάτημα (Μετάβαση σε οθόνη ή αποθήκευση στη βάση SQLite).");
        });

        let (cur_nav, cur_submit) = doc.get_button_action(&node.id);
        let available_pages: Vec<(&String, &str)> = doc.root_node_ids.iter().filter_map(|pid| {
            doc.nodes.get(pid).map(|pn| (pid, pn.name.as_str()))
        }).collect();

        let current_nav_page = cur_nav.clone().unwrap_or_default();
        let current_nav_label = if current_nav_page.is_empty() {
            "— None (No Screen Change) —".to_string()
        } else {
            available_pages.iter().find(|(id, _)| **id == current_nav_page).map(|(id, name)| format!("{} ({})", name, id)).unwrap_or_else(|| current_nav_page.clone())
        };

        ui.horizontal(|ui| {
            ui.label("Target Screen:");
            egui::ComboBox::from_id_salt(format!("btn_nav_{}", node.id))
                .selected_text(current_nav_label)
                .width(160.)
                .show_ui(ui, |ui| {
                    if ui.selectable_label(current_nav_page.is_empty(), "— None —").clicked() {
                        events.push(set_button_action(node.id.clone(), None, cur_submit.clone()));
                    }
                    for (pid, pname) in &available_pages {
                        let is_sel = **pid == current_nav_page;
                        if ui.selectable_label(is_sel, format!("{} ({})", pname, pid)).clicked() {
                            events.push(set_button_action(node.id.clone(), Some((*pid).clone()), cur_submit.clone()));
                        }
                    }
                });
        });

        let mut submit_val = cur_submit.clone().unwrap_or_default();
        ui.horizontal(|ui| {
            ui.label("Submit Entity:");
            let resp = ui.add(egui::TextEdit::singleline(&mut submit_val).hint_text("e.g. contacts").desired_width(110.));
            if resp.lost_focus() {
                let new_sub = if submit_val.trim().is_empty() { None } else { Some(submit_val.trim().to_string()) };
                if new_sub != cur_submit {
                    events.push(set_button_action(node.id.clone(), cur_nav.clone(), new_sub));
                }
            }
        });

        ui.horizontal(|ui| {
            ui.label(RichText::new("Pills:").size(8.5).color(theme::TEXT_MUTED));
            for ent_name in &["contacts", "deals", "tasks", "settings"] {
                let is_active = cur_submit.as_deref() == Some(*ent_name);
                if ui.selectable_label(is_active, *ent_name).clicked() {
                    let new_sub = if is_active { None } else { Some((*ent_name).to_string()) };
                    events.push(set_button_action(node.id.clone(), cur_nav.clone(), new_sub));
                }
            }
        });
    }
}

pub fn draw_table_props(ui: &mut egui::Ui, node: &Node, events: &mut Vec<CanvasEvent>) {
    if let NodeType::Table { bound_entity, .. } = &node.node_type {
        ui.add_space(2.);
        ui.horizontal(|ui| {
            ui.label(RichText::new("DATA-BOUND TABLE").size(9.).color(theme::TEXT_DIM).strong());
            info_button(ui, "Συνδέστε τον πίνακα με SQLite Entity (π.χ. contacts, deals, tasks).");
        });
        let mut ent = bound_entity.clone().unwrap_or_default();
        ui.horizontal(|ui| {
            ui.label("Entity/Table:");
            let resp = ui.add(egui::TextEdit::singleline(&mut ent).hint_text("e.g. contacts").desired_width(120.));
            if resp.lost_focus() && ent != bound_entity.clone().unwrap_or_default() {
                events.push(bind_data(node.id.clone(), if ent.is_empty() { None } else { Some(ent.clone()) }, None));
            }
        });

        ui.horizontal(|ui| {
            for preset_ent in &["contacts", "deals", "tasks", "inventory"] {
                let is_curr = ent == *preset_ent;
                if ui.selectable_label(is_curr, *preset_ent).clicked() && !is_curr {
                    events.push(bind_data(node.id.clone(), Some((*preset_ent).to_string()), None));
                }
            }
        });
    }
}

pub fn draw_styling_props(ui: &mut egui::Ui, node: &Node, events: &mut Vec<CanvasEvent>) {
    ui.add_space(4.);
    ui.separator();
    ui.horizontal(|ui| {
        ui.label(RichText::new("STYLING & APPEARANCE").size(9.).color(theme::TEXT_DIM).strong());
        info_button(ui, "Χρώματα, πάχος περιγράμματος και στρογγυλεμένες γωνίες (Corner Radius).");
    });
    ui.add_space(2.);

    let mut new_style = node.style.clone();
    let mut style_changed = false;

    style_changed |= color_picker_row(ui, "Background:", &mut new_style.bg_color);

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
