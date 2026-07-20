use crate::scene::{CanvasEvent, EditorState, Layout, NodeStyle, NodeType, ProjectDocument, ResizeHandle, Rgba, Sizing, Styling};
use crate::Viewport2D;
use eframe::egui::{self, Color32, CornerRadius, CursorIcon, Margin, Pos2, Rect, Sense, Stroke, UiBuilder, Vec2};

// ── Colour bridge ──

pub fn zoom_font(size: f32, zoom: f32) -> f32 {
    (size * zoom).max(3.0)
}

fn rgba(c: &Rgba) -> Color32 {
    Color32::from_rgba_premultiplied(c.r, c.g, c.b, c.a)
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

fn style_stroke(s: &NodeStyle) -> Stroke {
    Stroke::new(s.border_width, style_border_color(s))
}

fn style_radius(s: &NodeStyle) -> CornerRadius {
    let r = s.border_radius as u8;
    CornerRadius { nw: r, ne: r, se: r, sw: r }
}

// ── Styling → egui Frame ──

fn build_frame(styling: &Styling, style: &NodeStyle) -> egui::Frame {
    let mut frame = egui::Frame::default();
    frame = frame.fill(style_fill(style));
    frame = frame.stroke(style_stroke(style));
    frame = frame.corner_radius(style_radius(style));
    let [l, t, r, b] = styling.padding;
    frame = frame.inner_margin(Margin { left: l as i8, right: r as i8, top: t as i8, bottom: b as i8 });
    frame
}

fn input_frame(style: &NodeStyle) -> egui::Frame {
    egui::Frame::default()
        .fill(style_fill(style))
        .corner_radius(style_radius(style))
        .stroke(style_stroke(style))
}

fn node_size(layout: &Layout) -> Vec2 {
    let w = match &layout.width { Sizing::Fixed(v) => *v, Sizing::Fill(v) => *v, Sizing::Hug => 200. };
    let h = match &layout.height { Sizing::Fixed(v) => *v, Sizing::Fill(v) => *v, Sizing::Hug => 100. };
    Vec2::new(w, h)
}

fn world_to_local(pos: (f32, f32), viewport: &Viewport2D, canvas_origin: Pos2) -> Pos2 {
    Pos2::new(pos.0 * viewport.zoom + viewport.pan.x + canvas_origin.x, pos.1 * viewport.zoom + viewport.pan.y + canvas_origin.y)
}

fn draw_selection(ui: &egui::Ui, rect: Rect) {
    ui.painter().rect_stroke(
        rect.expand(2.), CornerRadius::same(2),
        Stroke::new(2., Color32::from_rgb(0, 120, 215)),
        egui::StrokeKind::Outside,
    );
}

const HANDLE_SIZE: f32 = 8.0;

fn handle_rect(center: Pos2) -> Rect {
    Rect::from_center_size(center, Vec2::splat(HANDLE_SIZE))
}

fn cursor_for_handle(handle: ResizeHandle) -> CursorIcon {
    use ResizeHandle::*;
    match handle {
        TopLeft | BottomRight => CursorIcon::ResizeNwSe,
        TopRight | BottomLeft => CursorIcon::ResizeNeSw,
        Top | Bottom => CursorIcon::ResizeVertical,
        Left | Right => CursorIcon::ResizeHorizontal,
    }
}

/// Returns true if a handle was dragged (caller should suppress node drag).
fn render_handles(
    ui: &mut egui::Ui,
    rect: Rect,
    events: &mut Vec<CanvasEvent>,
    node_id: &str,
    viewport: &Viewport2D,
) -> bool {
    use ResizeHandle::*;
    let centers = [
        (TopLeft,     Pos2::new(rect.left(),  rect.top())),
        (Top,         Pos2::new(rect.center().x, rect.top())),
        (TopRight,    Pos2::new(rect.right(), rect.top())),
        (Right,       Pos2::new(rect.right(), rect.center().y)),
        (BottomRight, Pos2::new(rect.right(), rect.bottom())),
        (Bottom,      Pos2::new(rect.center().x, rect.bottom())),
        (BottomLeft,  Pos2::new(rect.left(),  rect.bottom())),
        (Left,        Pos2::new(rect.left(),  rect.center().y)),
    ];

    let mut any_dragged = false;
    for (handle, center) in centers {
        let hr = handle_rect(center);
        let sense_id = ui.id().with(("resize", node_id, handle));
        let response = ui.interact(hr, sense_id, Sense::drag())
            .on_hover_cursor(cursor_for_handle(handle));

        // Draw handle
        ui.painter().rect_filled(hr, 1., Color32::WHITE);
        ui.painter().rect_stroke(hr, 1., Stroke::new(1., Color32::from_rgb(0, 120, 215)), egui::StrokeKind::Outside);

        if response.drag_started() {
            events.push(CanvasEvent::NodeResizeStarted);
        }

        if response.dragged() {
            any_dragged = true;
            let world_delta = response.drag_delta() / viewport.zoom;
            events.push(CanvasEvent::NodeResized {
                id: node_id.to_owned(),
                handle,
                delta: (world_delta.x, world_delta.y),
            });
        }
    }
    any_dragged
}

/// Draw selection highlight and resize handles.
/// Returns true if a resize handle was dragged.
fn sense_interaction(
    ui: &mut egui::Ui,
    node_id: &str,
    content_rect: Rect,
    events: &mut Vec<CanvasEvent>,
    viewport: &Viewport2D,
    editor_state: &EditorState,
) -> bool {
    let is_selected = editor_state.selected_node_ids.iter().any(|sid| sid == node_id);
    if !is_selected { return false; }

    // Guard against tiny rects that would crash handle rendering
    if content_rect.width() < 4.0 || content_rect.height() < 4.0 { return false; }
    if !content_rect.is_positive() { return false; }

    draw_selection(ui, content_rect);
    render_handles(ui, content_rect, events, node_id, viewport)
}

// ── Public entry point ──

pub fn draw_document(
    ui: &mut egui::Ui,
    doc: &ProjectDocument,
    viewport: &Viewport2D,
    editor_state: &EditorState,
    play_mode: bool,
    form_state: &mut std::collections::HashMap<String, String>,
    active_play_page: Option<&str>,
) -> Vec<CanvasEvent> {
    let mut events = Vec::new();
    let canvas_origin = ui.max_rect().min;

    for root_id in &doc.root_node_ids {
        let node = match doc.nodes.get(root_id) {
            Some(n) => n,
            None => { eprintln!("[renderer] root '{}' not found", root_id); continue; }
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
            render_node(ui, root_id, doc, &mut events, viewport, editor_state, play_mode, form_state);
        });
    }

    events
}

// ── Recursive traversal (returns true if this node or any descendant was clicked) ──

fn render_node(
    ui: &mut egui::Ui,
    node_id: &str,
    doc: &ProjectDocument,
    events: &mut Vec<CanvasEvent>,
    viewport: &Viewport2D,
    editor_state: &EditorState,
    play_mode: bool,
    form_state: &mut std::collections::HashMap<String, String>,
) -> bool {
    let node = match doc.nodes.get(node_id) {
        Some(n) => n,
        None => { eprintln!("[renderer] node '{}' not found", node_id); return false; }
    };
    if !node.visible { return false; }

    let (content_rect, child_clicked): (Rect, bool) = match &node.node_type {
        // ── Containers: render children FIRST, then check parent ──
        NodeType::Frame | NodeType::Group | NodeType::Page => {
            let mut child_clicked = false;
            let frame = build_frame(&node.styling, &node.style);
            let inner = frame.show(ui, |ui| {
                ui.set_min_size(ui.available_size());
                let parent_rect = ui.min_rect();
                for child_id in &node.children_ids {
                    if let Some(child) = doc.nodes.get(child_id) {
                        let child_w = match child.layout.width { Sizing::Fixed(v) => v, Sizing::Fill(v) => v, Sizing::Hug => 200. };
                        let child_h = match child.layout.height { Sizing::Fixed(v) => v, Sizing::Fill(v) => v, Sizing::Hug => 100. };
                        
                        let child_screen_pos = Pos2::new(
                            parent_rect.min.x + child.position.0 * viewport.zoom,
                            parent_rect.min.y + child.position.1 * viewport.zoom
                        );
                        let child_screen_size = Vec2::new(child_w * viewport.zoom, child_h * viewport.zoom);
                        let child_rect = Rect::from_min_size(child_screen_pos, child_screen_size);
                        
                        ui.allocate_new_ui(UiBuilder::new().max_rect(child_rect), |child_ui| {
                            if render_node(child_ui, child_id, doc, events, viewport, editor_state, play_mode, form_state) {
                                child_clicked = true;
                            }
                        });
                    }
                }
            });
            (inner.response.rect, child_clicked)
        }

        // ── Text / Labels ──
        NodeType::Text { content, font } => {
            let scaled_size = (font.size * viewport.zoom).max(4.0);
            let mut rt = egui::RichText::new(content).size(scaled_size).color(style_text_color(&node.style));
            if font.weight >= 700 { rt = rt.strong(); }
            
            let rect = ui.max_rect();
            let mut child_ui = ui.new_child(UiBuilder::new().max_rect(rect).layout(*ui.layout()));
            child_ui.set_min_size(rect.size());
            child_ui.set_max_size(rect.size());
            child_ui.label(rt);
            (rect, false)
        }

        // ── Inputs (frames with children inside) ──
        NodeType::TextInput { placeholder, .. } => {
            let mut child_clicked = false;
            let inner = input_frame(&node.style).show(ui, |ui| {
                ui.set_min_size(ui.available_size());
                let parent_rect = ui.min_rect();
                if play_mode {
                    let text = form_state.entry(node_id.to_owned()).or_default();
                    ui.add(egui::TextEdit::singleline(text).hint_text(placeholder).desired_width(f32::INFINITY));
                } else {
                    ui.label(egui::RichText::new(placeholder).size(zoom_font(12., viewport.zoom)).color(Color32::from_rgb(130, 130, 130)));
                }

                for child_id in &node.children_ids {
                    if let Some(child) = doc.nodes.get(child_id) {
                        let child_w = match child.layout.width { Sizing::Fixed(v) => v, Sizing::Fill(v) => v, Sizing::Hug => 200. };
                        let child_h = match child.layout.height { Sizing::Fixed(v) => v, Sizing::Fill(v) => v, Sizing::Hug => 100. };
                        let child_screen_pos = Pos2::new(
                            parent_rect.min.x + child.position.0 * viewport.zoom,
                            parent_rect.min.y + child.position.1 * viewport.zoom
                        );
                        let child_screen_size = Vec2::new(child_w * viewport.zoom, child_h * viewport.zoom);
                        let child_rect = Rect::from_min_size(child_screen_pos, child_screen_size);
                        ui.allocate_new_ui(UiBuilder::new().max_rect(child_rect), |child_ui| {
                            if render_node(child_ui, child_id, doc, events, viewport, editor_state, play_mode, form_state) {
                                child_clicked = true;
                            }
                        });
                    }
                }
            });
            (inner.response.rect, child_clicked)
        }
        NodeType::Dropdown { options, .. } => {
            let inner = input_frame(&node.style).show(ui, |ui| {
                ui.set_min_size(ui.available_size());
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
                    ui.label(egui::RichText::new(preview).size(zoom_font(12., viewport.zoom)).color(Color32::from_rgb(215, 215, 215)));
                    ui.label(egui::RichText::new(" ▾").size(zoom_font(10., viewport.zoom)).color(Color32::from_rgb(130, 130, 130)));
                }
            });
            (inner.response.rect, false)
        }
        NodeType::NumberField { min, max, .. } => {
            let inner = input_frame(&node.style).show(ui, |ui| {
                ui.set_min_size(ui.available_size());
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
                    ui.label(egui::RichText::new("123").size(zoom_font(12., viewport.zoom)).color(Color32::from_rgb(130, 130, 130)));
                }
            });
            (inner.response.rect, false)
        }
        NodeType::Checkbox { label, .. } => {
            let inner = input_frame(&node.style).show(ui, |ui| {
                ui.set_min_size(ui.available_size());
                if play_mode {
                    let text = form_state.entry(node_id.to_owned()).or_default();
                    let mut checked = text == "true";
                    let resp = ui.checkbox(&mut checked, label);
                    if resp.changed() {
                        *text = if checked { "true".to_string() } else { "false".to_string() };
                    }
                } else {
                    ui.add_enabled_ui(false, |ui| {
                        ui.checkbox(&mut false, label);
                    });
                }
            });
            (inner.response.rect, false)
        }

        // ── Actions ──
        NodeType::Button { label, style: _ } => {
            let inner = ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                let resp = ui.add_sized(
                    ui.available_size(),
                    egui::Button::new(egui::RichText::new(label).color(style_text_color(&node.style)))
                        .fill(style_fill(&node.style))
                        .corner_radius(style_radius(&node.style))
                        .sense(if play_mode { Sense::click() } else { Sense::hover() }),
                );
                if play_mode && resp.clicked() {
                    events.push(CanvasEvent::ActionTriggered { source_node_id: node_id.to_string() });
                }
            });
            (inner.response.rect, false)
        }

        // ── Static ──
        NodeType::Image { url, .. } => {
            let frame = egui::Frame::default()
                .fill(style_fill(&node.style))
                .stroke(style_stroke(&node.style));
            let inner = frame.show(ui, |ui| {
                ui.set_min_size(ui.available_size());
                ui.centered_and_justified(|ui| {
                    ui.label(egui::RichText::new(if url.is_empty() { "◫ Image" } else { url })
                        .size(zoom_font(11., viewport.zoom)).color(style_text_color(&node.style)));
                });
            });
            (inner.response.rect, false)
        }
        NodeType::Shape { kind } => {
            let inner = ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                ui.set_min_size(ui.available_size());
                ui.label(egui::RichText::new(format!("⬡ {:?}", kind)).size(zoom_font(11., viewport.zoom)).color(style_text_color(&node.style)))
            });
            (inner.response.rect, false)
        }
    };

    if !play_mode {
        sense_interaction(ui, node_id, content_rect, events, viewport, editor_state);
    }

    // Return true if this node's click OR any descendant's click fired
    let this_clicked = events.iter().any(|e| match e {
        CanvasEvent::NodeClicked { id, .. } => id == node_id,
        _ => false,
    });
    this_clicked || child_clicked
}
