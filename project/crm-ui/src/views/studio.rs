use eframe::egui::{self, Color32, Pos2, Rect, Stroke};
use crate::models::{tool_name, LayerContent, StudioLayer, StudioTool, STUDIO_TOOLS};
use crate::theme;
use crate::ProteusApp;

pub fn show_left(app: &mut ProteusApp, ui: &mut egui::Ui) {
    ui.add_space(6.);
    ui.label(egui::RichText::new("TOOLS").size(9.).color(theme::TEXT_DIM));
    ui.add_space(4.);
    for (tool, icon, _) in STUDIO_TOOLS {
        let sel = app.studio_tool == *tool;
        if ui.add(
            egui::Button::new(egui::RichText::new(format!(" {}  {}", icon, tool_name(tool))).size(11.).color(if sel { Color32::BLACK } else { theme::TEXT }))
                .fill(if sel { theme::ACCENT } else { theme::WIDGET_BG })
                .min_size(egui::vec2(ui.available_width(), 22.))
        ).clicked() {
            app.studio_tool = tool.clone();
        }
    }
    ui.add_space(8.);
    ui.label(egui::RichText::new("COLOR").size(9.).color(theme::TEXT_DIM));
    ui.add_space(2.);
    // Simple color display
    let cr = ui.allocate_exact_size(egui::vec2(20., 20.), egui::Sense::click());
    ui.painter().rect_filled(cr.0, 0, app.studio_color);
    ui.painter().rect_stroke(cr.0, 0, Stroke::new(1., theme::BORDER), egui::StrokeKind::Outside);
    ui.horizontal(|ui| {
        ui.add_space(24.);
        if ui.button(egui::RichText::new("Pick").size(9.)).clicked() {
            app.studio_show_color_popup = true;
            app.studio_color_picker_target = 0;
        }
    });
    ui.add_space(2.);
    ui.label(egui::RichText::new("SIZE").size(9.).color(theme::TEXT_DIM));
    ui.add(egui::Slider::new(&mut app.studio_brush_size, 1.0..=100.0).text("px").text_color(theme::TEXT_DIM));
}

pub fn show_central(app: &mut ProteusApp, ctx: &egui::Context, pnt: &egui::Painter, r: Rect, mpos: Option<Pos2>, mdown: bool, mup: bool) {
    let to_c = |pos: Pos2| egui::pos2(pos.x - r.left(), pos.y - r.top());

    // Render layers sorted by z
    let mut sorted: Vec<usize> = (0..app.studio_layers.len()).collect();
    sorted.sort_by_key(|&i| app.studio_layers[i].z);
    for &i in &sorted {
        let l = &app.studio_layers[i];
        if !l.visible { continue; }
        let lr = Rect::from_min_size(egui::pos2(r.left() + l.x, r.top() + l.y), egui::vec2(l.w, l.h));
        match &l.content {
            LayerContent::Shape { kind, x, y, w, h, fill_r, fill_g, fill_b, fill_a, stroke_r, stroke_g, stroke_b, stroke_a, stroke_width } => {
                let fill = Color32::from_rgba_premultiplied(*fill_r, *fill_g, *fill_b, *fill_a);
                let stroke = Color32::from_rgba_premultiplied(*stroke_r, *stroke_g, *stroke_b, *stroke_a);
                let sr = Rect::from_min_size(egui::pos2(r.left() + x, r.top() + y), egui::vec2(*w, *h));
                match kind.as_str() {
                    "ellipse" => {
                        pnt.circle_filled(sr.center(), sr.width().min(sr.height()) / 2., fill);
                        if *stroke_width > 0. {
                            pnt.circle_stroke(sr.center(), sr.width().min(sr.height()) / 2., Stroke::new(*stroke_width, stroke));
                        }
                    }
                    "line" => {
                        pnt.line_segment([sr.left_top(), sr.right_bottom()], Stroke::new(*stroke_width, stroke));
                    }
                    _ => {
                        pnt.rect_filled(sr, 0, fill);
                        if *stroke_width > 0. {
                            pnt.rect_stroke(sr, 0, Stroke::new(*stroke_width, stroke), egui::StrokeKind::Outside);
                        }
                    }
                }
                if app.studio_sel_layer == Some(i) {
                    pnt.rect_stroke(sr, 0, Stroke::new(1., theme::ACCENT), egui::StrokeKind::Outside);
                }
            }
            LayerContent::Stroke { points, r: pr, g: pg, b: pb, a: pa, size } => {
                if points.len() >= 2 {
                    let col = Color32::from_rgba_premultiplied(*pr, *pg, *pb, *pa);
                    for j in 1..points.len() {
                        let p1 = egui::pos2(r.left() + points[j - 1][0], r.top() + points[j - 1][1]);
                        let p2 = egui::pos2(r.left() + points[j][0], r.top() + points[j][1]);
                        pnt.line_segment([p1, p2], Stroke::new(*size, col));
                    }
                }
            }
            LayerContent::Text { content, font_size, r, g, b, a } => {
                let col = Color32::from_rgba_premultiplied(*r, *g, *b, *a);
                pnt.text(lr.left_top(), egui::Align2::LEFT_TOP, content, egui::FontId::proportional(*font_size), col);
                if app.studio_sel_layer == Some(i) {
                    pnt.rect_stroke(lr, 0, Stroke::new(1., theme::ACCENT), egui::StrokeKind::Outside);
                }
            }
            LayerContent::Image { .. } => {}
        }
    }

    // Shape preview while dragging
    if let (Some(start), Some(curr)) = (app.studio_shape_start, app.studio_shape_current) {
        let sr = Rect::from_min_max(start, curr);
        let psr = Rect::from_min_max(egui::pos2(r.left() + sr.min.x, r.top() + sr.min.y), egui::pos2(r.left() + sr.max.x, r.top() + sr.max.y));
        match app.studio_tool {
            StudioTool::Rectangle => {
                pnt.rect_filled(psr, 0, Color32::from_rgba_premultiplied(52, 152, 219, 80));
                pnt.rect_stroke(psr, 0, Stroke::new(1., theme::ACCENT), egui::StrokeKind::Outside);
            }
            StudioTool::Ellipse => {
                let rad = psr.width().min(psr.height()) / 2.;
                pnt.circle_filled(psr.center(), rad, Color32::from_rgba_premultiplied(52, 152, 219, 80));
                pnt.circle_stroke(psr.center(), rad, Stroke::new(1., theme::ACCENT));
            }
            _ => {}
        }
    }

    // Canvas interactions
    let cpos = mpos.map(to_c);
    let c_down = mdown;
    let c_up = mup;

    match app.studio_tool {
        StudioTool::Select => {
            if c_up {
                if let Some(pos) = cpos {
                    let mut hit_idx: Option<usize> = None;
                    for &i in sorted.iter().rev() {
                        let l = &app.studio_layers[i];
                        if !l.visible || l.locked { continue; }
                        let lr = Rect::from_min_size(egui::pos2(l.x, l.y), egui::vec2(l.w, l.h));
                        if lr.contains(pos) { hit_idx = Some(i); break; }
                    }
                    app.studio_sel_layer = hit_idx;
                }
            }
            if c_down {
                if let Some(pos) = cpos {
                    if app.studio_dragging.is_none() {
                        if let Some(li) = app.studio_sel_layer {
                            let l = &app.studio_layers[li];
                            let lr = Rect::from_min_size(egui::pos2(l.x, l.y), egui::vec2(l.w, l.h));
                            if lr.contains(pos) {
                                app.studio_dragging = Some((li, egui::pos2(l.x, l.y), pos));
                            }
                        }
                    }
                }
            }
            if let Some((li, start, off)) = app.studio_dragging {
                if c_down {
                    let drag_id = if app.studio_dragging.is_some() { Some(app.studio_layers[li].id.clone()) } else { None };
                    if let Some(pos) = cpos {
                        if let Some(ref lid) = drag_id {
                            if let Some(l) = app.studio_layers.iter_mut().find(|x| x.id == *lid) {
                                l.x = (start.x + pos.x - off.x).max(0.);
                                l.y = (start.y + pos.y - off.y).max(0.);
                            }
                        }
                    }
                } else {
                    app.studio_dragging = None;
                }
            }
        }
        StudioTool::Brush | StudioTool::Eraser => {
            if let Some(pos) = cpos {
                if c_down {
                    let brush_idx = app.studio_layers.iter().position(|l| matches!(l.content, LayerContent::Stroke { .. }));
                    let col = if app.studio_tool == StudioTool::Eraser { Color32::from_rgb(13, 13, 13) } else { app.studio_color };
                    let (cr, cg, cb, ca) = (col.r(), col.g(), col.b(), col.a());
                    let sz = if app.studio_tool == StudioTool::Eraser { app.studio_brush_size * 3. } else { app.studio_brush_size };
                    if let Some(si) = brush_idx {
                        if let LayerContent::Stroke { points, .. } = &mut app.studio_layers[si].content {
                            points.push([pos.x, pos.y]);
                        }
                    } else {
                        let z = app.studio_layers.iter().map(|x| x.z).max().unwrap_or(0) + 1;
                        app.studio_layers.push(StudioLayer {
                            id: format!("sl{}", app.studio_layers.len() + 1),
                            name: "Brush".into(),
                            x: 0., y: 0., w: 0., h: 0., z,
                            visible: true, opacity: 1., locked: false,
                            content: LayerContent::Stroke { points: vec![[pos.x, pos.y]], r: cr, g: cg, b: cb, a: ca, size: sz },
                        });
                        app.studio_sel_layer = Some(app.studio_layers.len() - 1);
                    }
                }
            }
        }
        StudioTool::Rectangle | StudioTool::Ellipse => {
            if c_down && app.studio_shape_start.is_none() {
                if let Some(pos) = cpos {
                    app.studio_shape_start = Some(pos);
                }
            }
            if app.studio_shape_start.is_some() {
                if let Some(pos) = cpos {
                    app.studio_shape_current = Some(pos);
                }
                if c_up {
                    if let (Some(s), Some(e)) = (app.studio_shape_start, app.studio_shape_current) {
                        let (x, y, w, h) = (
                            s.x.min(e.x), s.y.min(e.y),
                            (s.x - e.x).abs(), (s.y - e.y).abs(),
                        );
                        let kind = if app.studio_tool == StudioTool::Rectangle { "rect" } else { "ellipse" };
                        let col = app.studio_color;
                        let z = app.studio_layers.iter().map(|x| x.z).max().unwrap_or(0) + 1;
                        app.studio_layers.push(StudioLayer {
                            id: format!("sl{}", app.studio_layers.len() + 1),
                            name: kind.into(),
                            x, y, w, h, z,
                            visible: true, opacity: 1., locked: false,
                            content: LayerContent::Shape {
                                kind: kind.into(), x, y, w, h,
                                fill_r: col.r(), fill_g: col.g(), fill_b: col.b(), fill_a: 180,
                                stroke_r: 255, stroke_g: 255, stroke_b: 255, stroke_a: 255, stroke_width: 1.,
                            },
                        });
                        app.studio_sel_layer = Some(app.studio_layers.len() - 1);
                    }
                    app.studio_shape_start = None;
                    app.studio_shape_current = None;
                }
            }
        }
        StudioTool::Line => {
            if c_down && app.studio_shape_start.is_none() {
                if let Some(pos) = cpos { app.studio_shape_start = Some(pos); }
            }
            if app.studio_shape_start.is_some() {
                if let Some(pos) = cpos { app.studio_shape_current = Some(pos); }
                if c_up {
                    if let (Some(s), Some(e)) = (app.studio_shape_start, app.studio_shape_current) {
                        let col = app.studio_color;
                        let z = app.studio_layers.iter().map(|x| x.z).max().unwrap_or(0) + 1;
                        app.studio_layers.push(StudioLayer {
                            id: format!("sl{}", app.studio_layers.len() + 1),
                            name: "Line".into(),
                            x: s.x, y: s.y, w: (s.x - e.x).abs(), h: (s.y - e.y).abs(),
                            z, visible: true, opacity: 1., locked: false,
                            content: LayerContent::Shape {
                                kind: "line".into(), x: s.x, y: s.y, w: (s.x - e.x).abs(), h: (s.y - e.y).abs(),
                                fill_r: 0, fill_g: 0, fill_b: 0, fill_a: 0,
                                stroke_r: col.r(), stroke_g: col.g(), stroke_b: col.b(), stroke_a: col.a(), stroke_width: app.studio_brush_size,
                            },
                        });
                        app.studio_sel_layer = Some(app.studio_layers.len() - 1);
                    }
                    app.studio_shape_start = None;
                    app.studio_shape_current = None;
                }
            }
        }
        StudioTool::Text => {
            if c_up && app.studio_shape_start.is_none() {
                if let Some(pos) = cpos {
                    app.studio_text_input.clear();
                    app.studio_show_text_dialog = true;
                    app.studio_shape_start = Some(pos);
                }
            }
        }
        StudioTool::Picker => {
            if c_up {
                app.toast("Color picker (todo — click to sample)");
            }
        }
        StudioTool::Fill => {
            if c_up {
                if let Some(li) = app.studio_sel_layer {
                    if let Some(l) = app.studio_layers.get_mut(li) {
                        if let LayerContent::Shape { fill_r, fill_g, fill_b, fill_a, .. } = &mut l.content {
                            let c = app.studio_color;
                            *fill_r = c.r(); *fill_g = c.g(); *fill_b = c.b(); *fill_a = c.a();
                        }
                    }
                }
            }
        }
    }

    // Text dialog
    if app.studio_show_text_dialog {
        egui::Area::new("text_dialog".into()).anchor(egui::Align2::CENTER_CENTER, egui::vec2(0., 0.)).show(ctx, |ui| {
            ui.label("Enter text:");
            ui.text_edit_multiline(&mut app.studio_text_input);
            if ui.button("Add").clicked() && !app.studio_text_input.is_empty() {
                if let Some(pos) = app.studio_shape_start {
                    let col = app.studio_color;
                    let z = app.studio_layers.iter().map(|x| x.z).max().unwrap_or(0) + 1;
                    app.studio_layers.push(StudioLayer {
                        id: format!("sl{}", app.studio_layers.len() + 1),
                        name: "Text".into(),
                        x: pos.x, y: pos.y, w: 160., h: 30., z,
                        visible: true, opacity: 1., locked: false,
                        content: LayerContent::Text {
                            content: app.studio_text_input.clone(),
                            font_size: 18.,
                            r: col.r(), g: col.g(), b: col.b(), a: col.a(),
                        },
                    });
                    app.studio_sel_layer = Some(app.studio_layers.len() - 1);
                }
                app.studio_shape_start = None;
                app.studio_show_text_dialog = false;
            }
            if ui.button("Cancel").clicked() {
                app.studio_shape_start = None;
                app.studio_show_text_dialog = false;
            }
        });
    }

    // Color picker popup
    if app.studio_show_color_popup {
        egui::Area::new("color_popup".into()).anchor(egui::Align2::CENTER_CENTER, egui::vec2(0., 0.)).show(ctx, |ui| {
            ui.label(egui::RichText::new(if app.studio_color_picker_target == 0 { "Primary Color" } else { "Secondary Color" }).size(11.).color(theme::TEXT));
            ui.add_space(4.);
            let col = if app.studio_color_picker_target == 0 { &mut app.studio_color } else { &mut app.studio_color2 };
            let mut color = [col.r() as f32 / 255., col.g() as f32 / 255., col.b() as f32 / 255., col.a() as f32 / 255.];
            ui.color_edit_button_rgba_premultiplied(&mut color);
            if ui.button("Close").clicked() { app.studio_show_color_popup = false; }
        });
    }

    // Empty canvas hint
    if app.studio_layers.is_empty() {
        pnt.text(r.center(), egui::Align2::CENTER_CENTER,
            "Use the tool palette to create layers. Choose a tool and click/drag on canvas.",
            egui::FontId::proportional(14.), theme::TEXT_DIM);
    }
}

pub fn show_right(app: &mut ProteusApp, ui: &mut egui::Ui) {
    ui.add_space(4.);
    let mut idxs: Vec<usize> = (0..app.studio_layers.len()).collect();
    idxs.sort_by(|&a, &b| app.studio_layers[b].z.cmp(&app.studio_layers[a].z));
    for &i in &idxs {
        let l = &app.studio_layers[i];
        let sel = app.studio_sel_layer == Some(i);
        let resp = ui.add(
            egui::Button::new(egui::RichText::new(if l.visible { format!("◉ {}", l.name) } else { format!("○ {}", l.name) }).size(10.).color(if sel { theme::ACCENT } else { theme::TEXT }))
                .fill(if sel { Color32::from_rgb(35, 35, 50) } else { theme::WIDGET_BG })
                .min_size(egui::vec2(ui.available_width(), 20.))
        );
        if resp.clicked() { app.studio_sel_layer = Some(i); }
        resp.context_menu(|ui| {
            if ui.button("Toggle Visibility").clicked() { if let Some(l) = app.studio_layers.get_mut(i) { l.visible = !l.visible; } ui.close_menu(); }
            if ui.button("Toggle Lock").clicked() { if let Some(l) = app.studio_layers.get_mut(i) { l.locked = !l.locked; } ui.close_menu(); }
            if ui.button("Duplicate").clicked() {
                let c = app.studio_layers[i].clone();
                let new_id = format!("sl{}", app.studio_layers.len() + 1);
                app.studio_layers.push(StudioLayer { id: new_id, ..c });
                ui.close_menu();
            }
            if ui.button("Delete").clicked() {
                app.studio_layers.remove(i);
                app.studio_sel_layer = None;
                ui.close_menu();
            }
        });
    }
    ui.add_space(4.);
    if ui.button(egui::RichText::new("+ Layer").size(10.).color(theme::ACCENT_GREEN)).clicked() {
        let z = app.studio_layers.iter().map(|x| x.z).max().unwrap_or(0) + 1;
        app.studio_layers.push(StudioLayer {
            id: format!("sl{}", app.studio_layers.len() + 1),
            name: format!("Layer {}", app.studio_layers.len() + 1),
            x: 50., y: 50., w: 100., h: 100., z,
            visible: true, opacity: 1., locked: false,
            content: LayerContent::Shape {
                kind: "rect".into(), x: 50., y: 50., w: 100., h: 100.,
                fill_r: 52, fill_g: 152, fill_b: 219, fill_a: 200,
                stroke_r: 255, stroke_g: 255, stroke_b: 255, stroke_a: 255, stroke_width: 1.,
            },
        });
        app.studio_sel_layer = Some(app.studio_layers.len() - 1);
    }
    if let Some(li) = app.studio_sel_layer {
        if let Some(l) = app.studio_layers.get(li) {
            ui.add_space(8.);
            ui.label(egui::RichText::new("PROPERTIES").size(9.).color(theme::TEXT_DIM));
            ui.separator();
            match &l.content {
                LayerContent::Shape { kind, x, y, w, h, .. } => {
                    ui.label(egui::RichText::new(format!("{} at ({:.0},{:.0}) {}x{}", kind, x, y, w, h)).size(9.).color(theme::TEXT_DIM));
                }
                LayerContent::Stroke { points, .. } => {
                    ui.label(egui::RichText::new(format!("Brush stroke, {} pts", points.len())).size(9.).color(theme::TEXT_DIM));
                }
                LayerContent::Text { content, .. } => {
                    ui.label(egui::RichText::new(format!("Text: \"{}\"", content)).size(9.).color(theme::TEXT_DIM));
                }
                LayerContent::Image { .. } => {
                    ui.label(egui::RichText::new("Image layer").size(9.).color(theme::TEXT_DIM));
                }
            }
        }
    }
}
