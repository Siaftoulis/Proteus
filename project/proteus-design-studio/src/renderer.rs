use crate::scene::{CanvasEvent, EditorState, Layout, NodeStyle, NodeType, ProjectDocument, Sizing, Styling};
use crate::Viewport2D;
use eframe::egui::{self, Color32, CornerRadius, Margin, Pos2, Rect, Sense, Stroke, UiBuilder, Vec2};

// ── Colour & Style Bridges ──

pub fn zoom_font(size: f32, zoom: f32) -> f32 {
    (size * zoom).max(1.0)
}

fn style_fill(s: &NodeStyle) -> Color32 {
    Color32::from_rgba_unmultiplied(
        (s.bg_color[0] * 255.) as u8, (s.bg_color[1] * 255.) as u8,
        (s.bg_color[2] * 255.) as u8, (s.bg_color[3] * 255.) as u8,
    )
}

fn style_text_color(s: &NodeStyle) -> Color32 {
    Color32::from_rgba_unmultiplied(
        (s.text_color[0] * 255.) as u8, (s.text_color[1] * 255.) as u8,
        (s.text_color[2] * 255.) as u8, (s.text_color[3] * 255.) as u8,
    )
}

fn style_border_color(s: &NodeStyle) -> Color32 {
    Color32::from_rgba_unmultiplied(
        (s.border_color[0] * 255.) as u8, (s.border_color[1] * 255.) as u8,
        (s.border_color[2] * 255.) as u8, (s.border_color[3] * 255.) as u8,
    )
}

fn style_radius(s: &NodeStyle, zoom: f32) -> CornerRadius {
    let r = (s.border_radius * zoom).round() as u8;
    CornerRadius { nw: r, ne: r, se: r, sw: r }
}

fn build_frame(styling: &Styling, style: &NodeStyle, zoom: f32) -> egui::Frame {
    let stroke_w = (style.border_width * zoom).max(0.5);
    let r = (style.border_radius * zoom).round() as u8;
    let l = ((styling.padding[0] * zoom).round() as i8).max(0);
    let t = ((styling.padding[1] * zoom).round() as i8).max(0);
    let r_pad = ((styling.padding[2] * zoom).round() as i8).max(0);
    let b = ((styling.padding[3] * zoom).round() as i8).max(0);

    egui::Frame::default()
        .fill(style_fill(style))
        .stroke(Stroke::new(stroke_w, style_border_color(style)))
        .corner_radius(CornerRadius { nw: r, ne: r, se: r, sw: r })
        .inner_margin(Margin { left: l, right: r_pad, top: t, bottom: b })
}

fn input_frame(style: &NodeStyle, zoom: f32) -> egui::Frame {
    let stroke_w = (style.border_width * zoom).max(0.5);
    let r = (style.border_radius * zoom).round() as u8;
    let h_pad = (((style.border_radius * 0.35 + 8.0) * zoom).round() as i8).max(4);
    let v_pad = ((2.0 * zoom).round() as i8).max(1);

    egui::Frame::default()
        .fill(style_fill(style))
        .corner_radius(CornerRadius { nw: r, ne: r, se: r, sw: r })
        .stroke(Stroke::new(stroke_w, style_border_color(style)))
        .inner_margin(Margin { left: h_pad, right: h_pad, top: v_pad, bottom: v_pad })
}

fn node_size(layout: &Layout) -> Vec2 {
    let w = match &layout.width { Sizing::Fixed(v) => *v, Sizing::Fill(v) => *v, Sizing::Hug => 200. };
    let h = match &layout.height { Sizing::Fixed(v) => *v, Sizing::Fill(v) => *v, Sizing::Hug => 100. };
    Vec2::new(w, h)
}

fn world_to_local(pos: (f32, f32), viewport: &Viewport2D, canvas_origin: Pos2) -> Pos2 {
    Pos2::new(pos.0 * viewport.zoom + viewport.pan.x + canvas_origin.x, pos.1 * viewport.zoom + viewport.pan.y + canvas_origin.y)
}

fn sense_interaction(
    ui: &mut egui::Ui,
    node_id: &str,
    content_rect: Rect,
    events: &mut Vec<CanvasEvent>,
    viewport: &Viewport2D,
    editor_state: &EditorState,
    rotation: f32,
) -> bool {
    let is_selected = editor_state.selected_node_ids.iter().any(|sid| sid == node_id);
    if !is_selected { return false; }
    if content_rect.width() < 4.0 || content_rect.height() < 4.0 { return false; }
    if !content_rect.is_positive() { return false; }

    crate::components::transform_handles::render_transform_handles(
        ui,
        content_rect,
        node_id,
        events,
        viewport,
        rotation,
    )
}

// ── Public entry point ──

#[allow(clippy::too_many_arguments)]
pub fn draw_document(
    ui: &mut egui::Ui,
    doc: &ProjectDocument,
    viewport: &Viewport2D,
    editor_state: &EditorState,
    play_mode: bool,
    form_state: &mut std::collections::HashMap<String, String>,
    active_play_page: Option<&str>,
    table_cache: &std::collections::HashMap<String, Vec<(String, serde_json::Value)>>,
) -> Vec<CanvasEvent> {
    let mut events = Vec::new();
    let canvas_origin = ui.max_rect().min;

    for root_id in &doc.root_node_ids {
        let node = match doc.nodes.get(root_id) {
            Some(n) => n,
            None => continue,
        };
        if !node.visible { continue; }
        if play_mode {
            if let Some(page) = active_play_page {
                if root_id != page { continue; }
            }
        }

        let world_pos = node.position;
        let screen_pos = world_to_local(world_pos, viewport, canvas_origin);
        let size = node_size(&node.layout);
        let screen_size = Vec2::new(size.x * viewport.zoom, size.y * viewport.zoom);
        let screen_rect = Rect::from_min_size(screen_pos, screen_size);

        ui.allocate_new_ui(UiBuilder::new().max_rect(screen_rect), |ui| {
            render_node(ui, root_id, doc, &mut events, viewport, editor_state, play_mode, form_state, screen_rect, table_cache);
        });
    }

    events
}

// ── Recursive Node Renderer ──

#[allow(clippy::too_many_arguments)]
fn render_node(
    ui: &mut egui::Ui,
    node_id: &str,
    doc: &ProjectDocument,
    events: &mut Vec<CanvasEvent>,
    viewport: &Viewport2D,
    editor_state: &EditorState,
    play_mode: bool,
    form_state: &mut std::collections::HashMap<String, String>,
    screen_rect: Rect,
    table_cache: &std::collections::HashMap<String, Vec<(String, serde_json::Value)>>,
) -> bool {
    let node = match doc.nodes.get(node_id) {
        Some(n) => n,
        None => return false,
    };
    if !node.visible { return false; }

    let z = viewport.zoom;
    ui.set_clip_rect(ui.clip_rect().intersect(screen_rect));
    ui.spacing_mut().item_spacing = Vec2::new(6.0 * z, 4.0 * z);
    ui.spacing_mut().button_padding = Vec2::new(6.0 * z, 3.0 * z);
    ui.spacing_mut().interact_size = Vec2::new(8.0 * z, 8.0 * z);
    ui.spacing_mut().icon_width = (16.0 * z).max(3.0);
    ui.spacing_mut().icon_width_inner = (12.0 * z).max(2.0);

    let child_clicked = match &node.node_type {
        // ── Containers: render children inside frame ──
        NodeType::Frame | NodeType::Group | NodeType::Page => {
            let mut any_child_clicked = false;
            let frame = build_frame(&node.styling, &node.style, z);
            frame.show(ui, |ui| {
                ui.set_min_size(ui.available_size());
                let parent_origin = screen_rect.min;
                for child_id in &node.children_ids {
                    if let Some(child) = doc.nodes.get(child_id) {
                        let child_w = match child.layout.width { Sizing::Fixed(v) => v, Sizing::Fill(v) => v, Sizing::Hug => 200. };
                        let child_h = match child.layout.height { Sizing::Fixed(v) => v, Sizing::Fill(v) => v, Sizing::Hug => 100. };

                        let child_screen_pos = Pos2::new(
                            parent_origin.x + child.position.0 * z,
                            parent_origin.y + child.position.1 * z,
                        );
                        let child_screen_size = Vec2::new(child_w * z, child_h * z);
                        let child_rect = Rect::from_min_size(child_screen_pos, child_screen_size);

                        ui.allocate_new_ui(UiBuilder::new().max_rect(child_rect), |child_ui| {
                            if render_node(child_ui, child_id, doc, events, viewport, editor_state, play_mode, form_state, child_rect, table_cache) {
                                any_child_clicked = true;
                            }
                        });
                    }
                }
            });
            any_child_clicked
        }

        // ── Text / Labels ──
        NodeType::Text { content, font } => {
            let frame = build_frame(&node.styling, &node.style, z);
            frame.show(ui, |ui| {
                ui.set_min_size(ui.available_size());
                let scaled_size = (font.size * z).max(1.0);
                let mut rt = egui::RichText::new(content).size(scaled_size).color(style_text_color(&node.style));
                if font.weight >= 700 { rt = rt.strong(); }
                ui.label(rt);
            });
            false
        }

        // ── Inputs ──
        NodeType::TextInput { placeholder, .. } => {
            let mut any_child_clicked = false;
            let inner = input_frame(&node.style, z);
            inner.show(ui, |ui| {
                ui.set_min_size(ui.available_size());
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    if play_mode {
                        let text = form_state.entry(node_id.to_owned()).or_default();
                        ui.add(egui::TextEdit::singleline(text).hint_text(placeholder).desired_width(f32::INFINITY));
                    } else {
                        ui.label(egui::RichText::new(placeholder).size(zoom_font(12., z)).color(Color32::from_rgb(130, 130, 130)));
                    }
                });

                let parent_origin = screen_rect.min;
                for child_id in &node.children_ids {
                    if let Some(child) = doc.nodes.get(child_id) {
                        let child_w = match child.layout.width { Sizing::Fixed(v) => v, Sizing::Fill(v) => v, Sizing::Hug => 200. };
                        let child_h = match child.layout.height { Sizing::Fixed(v) => v, Sizing::Fill(v) => v, Sizing::Hug => 100. };
                        let child_screen_pos = Pos2::new(
                            parent_origin.x + child.position.0 * z,
                            parent_origin.y + child.position.1 * z,
                        );
                        let child_screen_size = Vec2::new(child_w * z, child_h * z);
                        let child_rect = Rect::from_min_size(child_screen_pos, child_screen_size);
                        ui.allocate_new_ui(UiBuilder::new().max_rect(child_rect), |child_ui| {
                            if render_node(child_ui, child_id, doc, events, viewport, editor_state, play_mode, form_state, child_rect, table_cache) {
                                any_child_clicked = true;
                            }
                        });
                    }
                }
            });
            any_child_clicked
        }

        NodeType::Dropdown { options, .. } => {
            let inner = input_frame(&node.style, z);
            inner.show(ui, |ui| {
                ui.set_min_size(ui.available_size());
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    if play_mode {
                        let text = form_state.entry(node_id.to_owned()).or_default();
                        let preview = text.clone();
                        egui::ComboBox::from_id_salt(node_id)
                            .selected_text(if preview.is_empty() { "Select..." } else { &preview })
                            .show_ui(ui, |ui| {
                                for opt in options {
                                    let opt_str = opt.clone();
                                    if ui.selectable_label(opt_str == *text, &opt_str).clicked() {
                                        *text = opt_str;
                                    }
                                }
                            });
                    } else {
                        let preview = options.first().map(|s| s.as_str()).unwrap_or("Select...");
                        ui.label(egui::RichText::new(preview).size(zoom_font(12., z)).color(Color32::from_rgb(215, 215, 215)));
                        ui.label(egui::RichText::new(" ▾").size(zoom_font(10., z)).color(Color32::from_rgb(130, 130, 130)));
                    }
                });
            });
            false
        }

        NodeType::NumberField { min, max, .. } => {
            let inner = input_frame(&node.style, z);
            inner.show(ui, |ui| {
                ui.set_min_size(ui.available_size());
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    if play_mode {
                        let text = form_state.entry(node_id.to_owned()).or_default();
                        let mut val: f64 = text.parse().unwrap_or(0.0);
                        let resp = ui.add(egui::DragValue::new(&mut val)
                            .speed(1.0)
                            .range(min.unwrap_or(f64::NEG_INFINITY)..=max.unwrap_or(f64::INFINITY)));
                        if resp.changed() || resp.lost_focus() {
                            *text = val.to_string();
                        }
                    } else {
                        ui.label(egui::RichText::new("123").size(zoom_font(12., z)).color(Color32::from_rgb(130, 130, 130)));
                    }
                });
            });
            false
        }

        NodeType::Checkbox { label, .. } => {
            let inner = input_frame(&node.style, z);
            inner.show(ui, |ui| {
                ui.set_min_size(ui.available_size());
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    if play_mode {
                        let text = form_state.entry(node_id.to_owned()).or_default();
                        let mut checked = text == "true";
                        let resp = ui.checkbox(&mut checked, egui::RichText::new(label).size(zoom_font(12., z)));
                        if resp.changed() {
                            *text = if checked { "true".to_string() } else { "false".to_string() };
                        }
                    } else {
                        ui.add_enabled_ui(false, |ui| {
                            ui.checkbox(&mut false, egui::RichText::new(label).size(zoom_font(12., z)));
                        });
                    }
                });
            });
            false
        }

        // ── Actions ──
        NodeType::Button { label, style: _ } => {
            ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                let resp = ui.add_sized(
                    ui.available_size(),
                    egui::Button::new(
                        egui::RichText::new(label)
                            .size(zoom_font(13., z))
                            .color(style_text_color(&node.style))
                    )
                    .fill(style_fill(&node.style))
                    .corner_radius(style_radius(&node.style, z))
                    .sense(if play_mode { Sense::click() } else { Sense::hover() }),
                );
                if play_mode && resp.clicked() {
                    events.push(CanvasEvent::ActionTriggered { source_node_id: node_id.to_string() });
                }
            });
            false
        }

        // ── Static ──
        NodeType::Image { url, .. } => {
            let stroke_w = (node.style.border_width * z).max(0.5);
            let frame = egui::Frame::default()
                .fill(style_fill(&node.style))
                .stroke(Stroke::new(stroke_w, style_border_color(&node.style)));
            frame.show(ui, |ui| {
                ui.set_min_size(ui.available_size());
                ui.centered_and_justified(|ui| {
                    ui.label(egui::RichText::new(if url.is_empty() { "◫ Image" } else { url })
                        .size(zoom_font(11., z)).color(style_text_color(&node.style)));
                });
            });
            false
        }

        NodeType::Shape { kind } => {
            crate::components::vector_renderer::render_vector_shape(ui, node, kind, z);
            false
        }

        // ── Data-Bound Table ──
        NodeType::Table { bound_entity, columns } => {
            crate::components::data_widgets::render_data_bound_table(
                ui,
                node_id,
                node,
                bound_entity.as_deref(),
                columns,
                z,
                play_mode,
                form_state,
                table_cache,
                events,
            );
            false
        }

        // ── Dynamic Form ──
        NodeType::DynamicForm { bound_entity, title, submit_label } => {
            crate::components::data_widgets::render_dynamic_form(
                ui,
                node_id,
                node,
                bound_entity,
                title,
                submit_label,
                z,
                play_mode,
                form_state,
                events,
            );
            false
        }
    };

    if !play_mode {
        sense_interaction(ui, node_id, screen_rect, events, viewport, editor_state, node.rotation);
    }

    let this_clicked = events.iter().any(|e| match e {
        CanvasEvent::NodeClicked { id, .. } => id == node_id,
        _ => false,
    });
    this_clicked || child_clicked
}
