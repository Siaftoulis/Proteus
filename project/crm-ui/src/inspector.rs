use crate::scene::{
    CanvasEvent, EditorState, FieldType, NodeType, NodeUpdate,
    ProjectDocument, Sizing,
};
use crate::theme;
use eframe::egui::{self, Color32, RichText, ScrollArea, Vec2};

// ── Info helper ──

fn info_button(ui: &mut egui::Ui, tooltip: &str) {
    let resp = ui.add(
        egui::Button::new(RichText::new("ℹ").size(10.).color(Color32::from_rgb(0, 180, 255)))
            .fill(Color32::from_rgb(35, 45, 55))
            .min_size(Vec2::new(16., 16.)),
    );
    resp.on_hover_text(tooltip);
}

// ── Public entry point ──

/// Two-zone Photoshop-style Inspector:
/// - Upper 2/3: Transform, Sizing, Smart Box Role Selector, Styling & DB Binding
/// - Lower 1/3: Layers / Hierarchy Tree with Lock & Visibility toggles
pub fn draw_inspector(
    ui: &mut egui::Ui,
    doc: &ProjectDocument,
    editor_state: &EditorState,
    copied_dimensions: &mut Option<(f32, f32)>,
) -> Vec<CanvasEvent> {
    let mut events = Vec::new();
    let total_h = ui.available_height();
    let top_h = (total_h * 0.65).max(220.0);
    let bottom_h = (total_h - top_h - 14.0).max(120.0);

    let selected_id = editor_state.selected_node_ids.first().cloned();
    let selected_node = selected_id.as_ref().and_then(|id| doc.get_node(id));

    // ══════════════════════════════════════════════════════════════
    // ZONE 1: Upper 2/3 (Properties, Smart Role, Transform, Style)
    // ══════════════════════════════════════════════════════════════
    ui.allocate_ui(Vec2::new(ui.available_width(), top_h), |ui| {
        ScrollArea::vertical().id_salt("inspector_properties").show(ui, |ui| {
            if let (Some(node_id), Some(node)) = (selected_id.as_ref(), selected_node) {
                // ── Header & Rename ──
                ui.add_space(2.);
                let mut name_buf = node.name.clone();
                ui.horizontal(|ui| {
                    ui.label(RichText::new("NAME:").size(9.).color(theme::TEXT_DIM));
                    if ui.add(egui::TextEdit::singleline(&mut name_buf).desired_width(140.)).changed() {
                        events.push(CanvasEvent::NodeModified {
                            id: node_id.clone(),
                            update: NodeUpdate::Rename(name_buf),
                        });
                    }
                });
                ui.label(RichText::new(format!("ID: {}", node_id)).size(8.).color(Color32::GRAY));
                ui.separator();

                // ── Transform & Size ──
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
                    events.push(CanvasEvent::NodeModified {
                        id: node_id.clone(),
                        update: NodeUpdate::Move { x: new_x, y: new_y },
                    });
                }
                if dim_changed {
                    events.push(CanvasEvent::NodeModified {
                        id: node_id.clone(),
                        update: NodeUpdate::Dimensions { width: new_w, height: new_h },
                    });
                }

                // Copy / Paste Size buttons
                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new("📋 Copy Size").min_size(Vec2::new(70., 18.))).clicked() {
                        *copied_dimensions = Some((cur_w, cur_h));
                    }
                    let paste_enabled = copied_dimensions.is_some();
                    if ui.add_enabled(paste_enabled, egui::Button::new("📌 Paste Size").min_size(Vec2::new(70., 18.))).clicked() {
                        if let Some((cw, ch)) = *copied_dimensions {
                            events.push(CanvasEvent::NodeModified {
                                id: node_id.clone(),
                                update: NodeUpdate::Dimensions { width: cw, height: ch },
                            });
                        }
                    }
                });

                ui.separator();
                // ── Smart Box Role Selector ──
                ui.horizontal(|ui| {
                    ui.label(RichText::new("SMART BOX ROLE").size(9.).color(theme::TEXT_DIM).strong());
                    info_button(ui, "Ορίστε τη λειτουργία του Smart Box: Shape, Input, Text, Table ή Card.\nΤα Buttons διαχειρίζονται αυτόνομα από το εργαλείο Button.");
                });

                let is_static_shape = matches!(node.node_type, NodeType::Shape { .. });
                let is_input = matches!(
                    node.node_type,
                    NodeType::TextInput { .. }
                        | NodeType::Dropdown { .. }
                        | NodeType::NumberField { .. }
                        | NodeType::Checkbox { .. }
                );
                let is_text = matches!(node.node_type, NodeType::Text { .. });
                let is_frame = matches!(node.node_type, NodeType::Frame | NodeType::Page | NodeType::Group);
                let is_table = matches!(node.node_type, NodeType::Table { .. });

                ui.horizontal(|ui| {
                    if ui.selectable_label(is_static_shape, "◻ Shape").clicked() && !is_static_shape {
                        events.push(CanvasEvent::NodeModified {
                            id: node_id.clone(),
                            update: NodeUpdate::NodeTypeChange(NodeType::Shape {
                                kind: crate::scene::ShapeKind::Rectangle,
                            }),
                        });
                    }
                    if ui.selectable_label(is_input, "📝 Input").clicked() && !is_input {
                        events.push(CanvasEvent::NodeModified {
                            id: node_id.clone(),
                            update: NodeUpdate::NodeTypeChange(NodeType::TextInput {
                                placeholder: "Type...".into(),
                                field_type: FieldType::Text,
                                bound_entity: None,
                                bound_field: None,
                            }),
                        });
                    }
                    if ui.selectable_label(is_text, "🏷 Text").clicked() && !is_text {
                        events.push(CanvasEvent::NodeModified {
                            id: node_id.clone(),
                            update: NodeUpdate::NodeTypeChange(NodeType::Text {
                                content: "Label".into(),
                                font: crate::scene::FontSpec::default(),
                            }),
                        });
                    }
                    if ui.selectable_label(is_table, "⊞ Table").clicked() && !is_table {
                        events.push(CanvasEvent::NodeModified {
                            id: node_id.clone(),
                            update: NodeUpdate::NodeTypeChange(NodeType::Table {
                                bound_entity: Some("contacts".into()),
                                columns: vec!["ID".into(), "Name".into(), "Email".into(), "Status".into()],
                            }),
                        });
                    }
                    if ui.selectable_label(is_frame, "📦 Card").clicked() && !is_frame {
                        events.push(CanvasEvent::NodeModified {
                            id: node_id.clone(),
                            update: NodeUpdate::NodeTypeChange(NodeType::Frame),
                        });
                    }
                });

                if is_frame {
                    ui.add_space(2.);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("📦 CARD CONTAINER").size(9.).color(theme::ACCENT).strong());
                        info_button(ui, "Η Κάρτα (Card) είναι ένα πλαίσιο ομαδοποίησης στοιχείων (κάρτα προφίλ, στατιστικά, φόρμα) με στυλάτο φόντο και στρογγυλεμένες γωνίες.");
                    });
                }

                // Input sub-types & Simple configuration
                if is_input {
                    ui.add_space(2.);
                    ui.label(RichText::new("INPUT TYPE:").size(8.).color(Color32::GRAY));
                    ui.horizontal(|ui| {
                        let is_text_in = matches!(node.node_type, NodeType::TextInput { .. });
                        let is_drop_in = matches!(node.node_type, NodeType::Dropdown { .. });
                        let is_num_in = matches!(node.node_type, NodeType::NumberField { .. });
                        let is_chk_in = matches!(node.node_type, NodeType::Checkbox { .. });

                        if ui.selectable_label(is_text_in, "Text").clicked() && !is_text_in {
                            events.push(CanvasEvent::NodeModified {
                                id: node_id.clone(),
                                update: NodeUpdate::NodeTypeChange(NodeType::TextInput {
                                    placeholder: "Type...".into(),
                                    field_type: FieldType::Text,
                                    bound_entity: None,
                                    bound_field: None,
                                }),
                            });
                        }
                        if ui.selectable_label(is_drop_in, "Dropdown").clicked() && !is_drop_in {
                            events.push(CanvasEvent::NodeModified {
                                id: node_id.clone(),
                                update: NodeUpdate::NodeTypeChange(NodeType::Dropdown {
                                    options: vec!["Option 1".into(), "Option 2".into()],
                                    multiple: false,
                                    bound_entity: None,
                                    bound_field: None,
                                }),
                            });
                        }
                        if ui.selectable_label(is_num_in, "Number").clicked() && !is_num_in {
                            events.push(CanvasEvent::NodeModified {
                                id: node_id.clone(),
                                update: NodeUpdate::NodeTypeChange(NodeType::NumberField {
                                    min: None,
                                    max: None,
                                    step: 1.0,
                                    bound_entity: None,
                                    bound_field: None,
                                }),
                            });
                        }
                        if ui.selectable_label(is_chk_in, "Check").clicked() && !is_chk_in {
                            events.push(CanvasEvent::NodeModified {
                                id: node_id.clone(),
                                update: NodeUpdate::NodeTypeChange(NodeType::Checkbox {
                                    label: "Confirm".into(),
                                    bound_entity: None,
                                    bound_field: None,
                                }),
                            });
                        }
                    });

                    // Direct, simple properties based on input sub-type
                    match &node.node_type {
                        NodeType::TextInput { placeholder, .. } => {
                            let mut ph = placeholder.clone();
                            ui.horizontal(|ui| {
                                ui.label("Placeholder:");
                                if ui.add(egui::TextEdit::singleline(&mut ph).desired_width(130.)).changed() {
                                    events.push(CanvasEvent::NodeModified {
                                        id: node_id.clone(),
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
                                    let new_opts: Vec<String> = opts_str
                                        .split(',')
                                        .map(|s| s.trim().to_string())
                                        .filter(|s| !s.is_empty())
                                        .collect();
                                    events.push(CanvasEvent::NodeModified {
                                        id: node_id.clone(),
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
                                        id: node_id.clone(),
                                        update: NodeUpdate::CheckboxLabel(lbl),
                                    });
                                }
                            });
                        }
                        _ => {}
                    }

                    // Optional collapsible for Database Link (clean and non-intrusive for Average Joe)
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
                        .id_salt(format!("db_link_{}", node_id))
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
                        events.push(CanvasEvent::NodeModified {
                            id: node_id.clone(),
                            update: NodeUpdate::DataBinding {
                                entity: if entity.trim().is_empty() { None } else { Some(entity.trim().to_string()) },
                                field: if field.trim().is_empty() { None } else { Some(field.trim().to_string()) },
                            },
                        });
                    }
                }

                // ── Typography & Content (if role is Text) ──
                if let NodeType::Text { content, font } = &node.node_type {
                    ui.add_space(2.);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("TYPOGRAPHY & CONTENT").size(9.).color(theme::TEXT_DIM).strong());
                        info_button(ui, "Μέγεθος γραμματοσειράς, βάρος και περιεχόμενο κειμένου.");
                    });
                    let mut val = content.clone();
                    ui.horizontal(|ui| {
                        ui.label("Text:");
                        let resp = ui.add(egui::TextEdit::singleline(&mut val).desired_width(140.));
                        if resp.changed() {
                            events.push(CanvasEvent::NodeModified {
                                id: node_id.clone(),
                                update: NodeUpdate::TextContent(val),
                            });
                        }
                    });

                    // Font Size Slider
                    let mut cur_size = font.size;
                    ui.horizontal(|ui| {
                        ui.label("Size:");
                        if ui.add(egui::Slider::new(&mut cur_size, 8.0..=72.0).suffix("px")).changed() {
                            events.push(CanvasEvent::NodeModified {
                                id: node_id.clone(),
                                update: NodeUpdate::FontSize(cur_size),
                            });
                        }
                    });

                    // Quick Presets
                    ui.horizontal(|ui| {
                        for (lbl, sz) in [("H1", 32.0), ("H2", 24.0), ("H3", 18.0), ("Body", 14.0), ("Small", 11.0)] {
                            let is_active = (font.size - sz).abs() < 1.0;
                            if ui.selectable_label(is_active, lbl).clicked() {
                                events.push(CanvasEvent::NodeModified {
                                    id: node_id.clone(),
                                    update: NodeUpdate::FontSize(sz),
                                });
                            }
                        }
                    });

                    // Font Weight
                    ui.horizontal(|ui| {
                        ui.label("Weight:");
                        let is_bold = font.weight >= 700;
                        if ui.selectable_label(!is_bold, "Regular").clicked() && is_bold {
                            events.push(CanvasEvent::NodeModified {
                                id: node_id.clone(),
                                update: NodeUpdate::FontWeight(400),
                            });
                        }
                        if ui.selectable_label(is_bold, "Bold").clicked() && !is_bold {
                            events.push(CanvasEvent::NodeModified {
                                id: node_id.clone(),
                                update: NodeUpdate::FontWeight(700),
                            });
                        }
                    });
                }

                // ── Button Configuration (if role is Button) ──
                if let NodeType::Button { label, style } = &node.node_type {
                    ui.add_space(2.);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("BUTTON CONFIGURATION").size(9.).color(theme::TEXT_DIM).strong());
                        info_button(ui, "Ετικέτα και οπτικό στυλ κουμπιού (Primary, Secondary, Danger, Ghost).");
                    });
                    let mut val = label.clone();
                    ui.horizontal(|ui| {
                        ui.label("Label:");
                        let resp = ui.add(egui::TextEdit::singleline(&mut val).desired_width(140.));
                        if resp.changed() {
                            events.push(CanvasEvent::NodeModified {
                                id: node_id.clone(),
                                update: NodeUpdate::NodeTypeChange(NodeType::Button {
                                    label: val,
                                    style: style.clone(),
                                }),
                            });
                        }
                    });

                    use crate::scene::ButtonStyle;
                    ui.horizontal(|ui| {
                        let is_p = matches!(style, ButtonStyle::Primary);
                        let is_s = matches!(style, ButtonStyle::Secondary);
                        let is_d = matches!(style, ButtonStyle::Danger);
                        let is_g = matches!(style, ButtonStyle::Ghost);

                        if ui.selectable_label(is_p, "Primary").clicked() && !is_p {
                            events.push(CanvasEvent::NodeModified {
                                id: node_id.clone(),
                                update: NodeUpdate::NodeTypeChange(NodeType::Button {
                                    label: label.clone(),
                                    style: ButtonStyle::Primary,
                                }),
                            });
                        }
                        if ui.selectable_label(is_s, "Secondary").clicked() && !is_s {
                            events.push(CanvasEvent::NodeModified {
                                id: node_id.clone(),
                                update: NodeUpdate::NodeTypeChange(NodeType::Button {
                                    label: label.clone(),
                                    style: ButtonStyle::Secondary,
                                }),
                            });
                        }
                        if ui.selectable_label(is_d, "Danger").clicked() && !is_d {
                            events.push(CanvasEvent::NodeModified {
                                id: node_id.clone(),
                                update: NodeUpdate::NodeTypeChange(NodeType::Button {
                                    label: label.clone(),
                                    style: ButtonStyle::Danger,
                                }),
                            });
                        }
                        if ui.selectable_label(is_g, "Ghost").clicked() && !is_g {
                            events.push(CanvasEvent::NodeModified {
                                id: node_id.clone(),
                                update: NodeUpdate::NodeTypeChange(NodeType::Button {
                                    label: label.clone(),
                                    style: ButtonStyle::Ghost,
                                }),
                            });
                        }
                    });

                    // ── On-Click Action Configuration ──
                    ui.add_space(4.);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("⚡ ON-CLICK ACTION").size(9.).color(theme::ACCENT).strong());
                        info_button(ui, "Ενέργεια κατά το πάτημα (Μετάβαση σε οθόνη ή αποθήκευση στη βάση SQLite).");
                    });

                    let (cur_nav, cur_submit) = doc.get_button_action(node_id);

                    // Collect all available pages
                    let available_pages: Vec<(&String, &str)> = doc.root_node_ids.iter().filter_map(|pid| {
                        doc.nodes.get(pid).map(|pn| (pid, pn.name.as_str()))
                    }).collect();

                    // 1. Navigation Screen Selector
                    let current_nav_page = cur_nav.clone().unwrap_or_default();
                    let current_nav_label = if current_nav_page.is_empty() {
                        "— None (No Screen Change) —".to_string()
                    } else {
                        available_pages.iter()
                            .find(|(id, _)| **id == current_nav_page)
                            .map(|(id, name)| format!("{} ({})", name, id))
                            .unwrap_or_else(|| current_nav_page.clone())
                    };

                    ui.horizontal(|ui| {
                        ui.label("Target Screen:");
                        egui::ComboBox::from_id_salt(format!("btn_nav_{}", node_id))
                            .selected_text(current_nav_label)
                            .width(160.)
                            .show_ui(ui, |ui| {
                                if ui.selectable_label(current_nav_page.is_empty(), "— None —").clicked() {
                                    events.push(CanvasEvent::SetButtonAction {
                                        button_id: node_id.clone(),
                                        target_page: None,
                                        submit_entity: cur_submit.clone(),
                                    });
                                }
                                for (pid, pname) in &available_pages {
                                    let is_sel = **pid == current_nav_page;
                                    if ui.selectable_label(is_sel, format!("{} ({})", pname, pid)).clicked() {
                                        events.push(CanvasEvent::SetButtonAction {
                                            button_id: node_id.clone(),
                                            target_page: Some((*pid).clone()),
                                            submit_entity: cur_submit.clone(),
                                        });
                                    }
                                }
                            });
                    });

                    // 2. Submit Form Selector
                    let mut submit_val = cur_submit.clone().unwrap_or_default();
                    ui.horizontal(|ui| {
                        ui.label("Submit Entity:");
                        let resp = ui.add(egui::TextEdit::singleline(&mut submit_val).hint_text("e.g. contacts").desired_width(110.));
                        if resp.lost_focus() {
                            let new_sub = if submit_val.trim().is_empty() { None } else { Some(submit_val.trim().to_string()) };
                            if new_sub != cur_submit {
                                events.push(CanvasEvent::SetButtonAction {
                                    button_id: node_id.clone(),
                                    target_page: cur_nav.clone(),
                                    submit_entity: new_sub,
                                });
                            }
                        }
                    });

                    // Quick pills for common entities
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Pills:").size(8.5).color(theme::TEXT_MUTED));
                        for ent_name in &["contacts", "deals", "tasks", "settings"] {
                            let is_active = cur_submit.as_deref() == Some(*ent_name);
                            if ui.selectable_label(is_active, *ent_name).clicked() {
                                let new_sub = if is_active { None } else { Some((*ent_name).to_string()) };
                                events.push(CanvasEvent::SetButtonAction {
                                    button_id: node_id.clone(),
                                    target_page: cur_nav.clone(),
                                    submit_entity: new_sub,
                                });
                            }
                        }
                    });
                }

                // ── Table Data Binding (if role is Table) ──
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
                            events.push(CanvasEvent::NodeModified {
                                id: node_id.clone(),
                                update: NodeUpdate::DataBinding {
                                    entity: if ent.is_empty() { None } else { Some(ent.clone()) },
                                    field: None,
                                },
                            });
                        }
                    });

                    // Quick Entity Suggestions
                    ui.horizontal(|ui| {
                        for preset_ent in &["contacts", "deals", "tasks", "inventory"] {
                            let is_curr = ent == *preset_ent;
                            if ui.selectable_label(is_curr, *preset_ent).clicked() && !is_curr {
                                events.push(CanvasEvent::NodeModified {
                                    id: node_id.clone(),
                                    update: NodeUpdate::DataBinding {
                                        entity: Some((*preset_ent).to_string()),
                                        field: None,
                                    },
                                });
                            }
                        }
                    });
                }

                ui.add_space(4.);
                ui.separator();

                // ── Styling & Appearance ──
                ui.horizontal(|ui| {
                    ui.label(RichText::new("STYLING & APPEARANCE").size(9.).color(theme::TEXT_DIM).strong());
                    info_button(ui, "Χρώματα, πάχος περιγράμματος και στρογγυλεμένες γωνίες (Corner Radius).");
                });
                ui.add_space(2.);

                use egui::widgets::color_picker::{color_edit_button_rgba, Alpha};
                let mut new_style = node.style.clone();
                let mut style_changed = false;

                ui.horizontal(|ui| {
                    ui.label("Background:");
                    let [r, g, b, a] = new_style.bg_color;
                    let mut c = egui::Rgba::from_rgba_unmultiplied(r, g, b, a);
                    style_changed |= color_edit_button_rgba(ui, &mut c, Alpha::OnlyBlend).changed();
                    new_style.bg_color = [c.r(), c.g(), c.b(), c.a()];
                });

                if is_text {
                    ui.horizontal(|ui| {
                        ui.label("Text Color:");
                        let [r, g, b, a] = new_style.text_color;
                        let mut c = egui::Rgba::from_rgba_unmultiplied(r, g, b, a);
                        style_changed |= color_edit_button_rgba(ui, &mut c, Alpha::OnlyBlend).changed();
                        new_style.text_color = [c.r(), c.g(), c.b(), c.a()];
                    });
                }

                ui.horizontal(|ui| {
                    ui.label("Border Color:");
                    let [r, g, b, a] = new_style.border_color;
                    let mut c = egui::Rgba::from_rgba_unmultiplied(r, g, b, a);
                    style_changed |= color_edit_button_rgba(ui, &mut c, Alpha::OnlyBlend).changed();
                    new_style.border_color = [c.r(), c.g(), c.b(), c.a()];
                });

                style_changed |= ui.add(egui::Slider::new(&mut new_style.border_width, 0.0..=10.0).text("Border Width")).changed();
                style_changed |= ui.add(egui::Slider::new(&mut new_style.border_radius, 0.0..=50.0).text("Corner Radius")).changed();

                if style_changed {
                    events.push(CanvasEvent::NodeModified {
                        id: node_id.clone(),
                        update: NodeUpdate::NodeStyle(new_style.clone()),
                    });
                    let r = new_style.border_radius;
                    events.push(CanvasEvent::NodeModified {
                        id: node_id.clone(),
                        update: NodeUpdate::CornerRadius([r, r, r, r]),
                    });
                }
            } else {
                ui.add_space(30.);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("No Element Selected").size(11.).color(theme::TEXT_DIM).strong());
                    ui.label(RichText::new("Select a layer below or click on canvas").size(9.).color(Color32::DARK_GRAY));
                });
            }
        });
    });

    ui.separator();

    // ══════════════════════════════════════════════════════════════
    // ZONE 2: Lower 1/3 (Layers & Hierarchy Tree, Lock & Visibility)
    // ══════════════════════════════════════════════════════════════
    crate::components::layers_panel::draw_layers_panel(
        ui,
        doc,
        selected_id.as_deref(),
        bottom_h,
        &mut events,
    );

    events
}
