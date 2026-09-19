//! Native canvas space calculations, virtual grid, device frames, and HUD overlays.

use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};
use crate::models::{LayoutMode, GRID};
use crate::theme;
use crate::ProteusApp;

pub fn handle_pan_and_zoom(
    app: &mut ProteusApp,
    ui: &mut egui::Ui,
    _r: Rect,
    mpos: Option<Pos2>,
    hover_canvas: bool,
    canvas_origin: Pos2,
) -> bool {
    let scroll = ui.input(|i| i.raw_scroll_delta);
    if scroll.y != 0. && hover_canvas {
        if let Some(cursor) = mpos {
            let world_before = app.viewport.screen_to_world(cursor, canvas_origin);
            app.viewport.zoom = (app.viewport.zoom * (1.0 + scroll.y * 0.002)).clamp(0.1, 5.0);
            app.viewport.pan.x = cursor.x - canvas_origin.x - world_before.x * app.viewport.zoom;
            app.viewport.pan.y = cursor.y - canvas_origin.y - world_before.y * app.viewport.zoom;
        } else {
            app.viewport.zoom = (app.viewport.zoom * (1.0 + scroll.y * 0.002)).clamp(0.1, 5.0);
        }
    }

    let middle = ui.input(|i| i.pointer.middle_down());
    let space = ui.input(|i| i.key_down(egui::Key::Space));
    let mdown = ui.input(|i| i.pointer.primary_down());
    let is_panning_input = middle || (space && mdown);
    let is_panning = is_panning_input && hover_canvas;

    if is_panning {
        let delta = ui.input(|i| i.pointer.delta());
        app.viewport.pan += delta;
    }

    is_panning
}

pub fn render_grid(app: &ProteusApp, pnt: &egui::Painter, r: Rect, canvas_origin: Pos2) {
    if app.layout == LayoutMode::Grid {
        let world_tl = app.viewport.screen_to_world(r.left_top(), canvas_origin);
        let world_br = app.viewport.screen_to_world(r.right_bottom(), canvas_origin);
        let start_x = (world_tl.x / GRID).floor() * GRID;
        let start_y = (world_tl.y / GRID).floor() * GRID;
        let mut wy = start_y;
        while wy <= world_br.y {
            let sy = app.viewport.world_to_screen(egui::pos2(0., wy), canvas_origin).y;
            pnt.line_segment(
                [egui::pos2(r.left(), sy), egui::pos2(r.right(), sy)],
                Stroke::new(1., Color32::from_rgba_premultiplied(30, 30, 30, 255)),
            );
            wy += GRID;
        }
        let mut wx = start_x;
        while wx <= world_br.x {
            let sx = app.viewport.world_to_screen(egui::pos2(wx, 0.), canvas_origin).x;
            pnt.line_segment(
                [egui::pos2(sx, r.top()), egui::pos2(sx, r.bottom())],
                Stroke::new(1., Color32::from_rgba_premultiplied(30, 30, 30, 255)),
            );
            wx += GRID;
        }
    } else {
        let world_tl = app.viewport.screen_to_world(r.left_top(), canvas_origin);
        let world_br = app.viewport.screen_to_world(r.right_bottom(), canvas_origin);
        let start_x = (world_tl.x / 24.0).floor() * 24.0;
        let start_y = (world_tl.y / 24.0).floor() * 24.0;
        let mut wy = start_y;
        while wy <= world_br.y {
            let mut wx = start_x;
            while wx <= world_br.x {
                let sp = app.viewport.world_to_screen(egui::pos2(wx, wy), canvas_origin);
                if r.contains(sp) {
                    pnt.circle_filled(sp, (1.0_f32).max(0.5 * app.viewport.zoom), Color32::from_rgb(25, 25, 25));
                }
                wx += 24.0;
            }
            wy += 24.0;
        }
    }
}

pub fn render_device_frame(app: &ProteusApp, pnt: &egui::Painter, r: Rect, canvas_origin: Pos2) {
    let (dw, dh) = app.device_preset.size();
    let dev_origin = app.viewport.world_to_screen(egui::pos2(0., 0.), canvas_origin);
    let dev_w = dw * app.viewport.zoom;
    let dev_h = dh * app.viewport.zoom;
    let dev_rect = Rect::from_min_size(dev_origin, egui::vec2(dev_w, dev_h));
    let outside_rects = [
        Rect::from_min_max(egui::pos2(r.left(), r.top()), egui::pos2(r.right(), dev_rect.top())),
        Rect::from_min_max(egui::pos2(r.left(), dev_rect.bottom()), egui::pos2(r.right(), r.bottom())),
        Rect::from_min_max(egui::pos2(r.left(), dev_rect.top()), egui::pos2(dev_rect.left(), dev_rect.bottom())),
        Rect::from_min_max(egui::pos2(dev_rect.right(), dev_rect.top()), egui::pos2(r.right(), dev_rect.bottom())),
    ];
    for or in &outside_rects {
        pnt.rect_filled(*or, 0, Color32::from_black_alpha(80));
    }
    pnt.rect_stroke(dev_rect, 4., Stroke::new(2., theme::ACCENT), egui::StrokeKind::Outside);
    let label = format!("{} {} — {}×{}", app.device_preset.icon(), app.device_preset.label(), dw, dh);
    pnt.text(
        egui::pos2(dev_rect.left() + 8., dev_rect.top() - 16.),
        egui::Align2::LEFT_BOTTOM,
        &label,
        egui::FontId::proportional(10.),
        theme::ACCENT,
    );
}

pub fn render_canvas_hud(
    app: &mut ProteusApp,
    ui: &mut egui::Ui,
    pnt: &egui::Painter,
    r: Rect,
    mpos: Option<Pos2>,
    canvas_origin: Pos2,
) {
    let hud_y = r.bottom() - 36.0;

    // 1. Bottom-Left Status & Cursor Coordinates Pill
    let status_rect = Rect::from_min_size(Pos2::new(r.left() + 14.0, hud_y), Vec2::new(260.0, 26.0));
    pnt.rect_filled(status_rect, egui::CornerRadius::same(6), Color32::from_rgb(18, 20, 26));
    pnt.rect_stroke(status_rect, egui::CornerRadius::same(6), Stroke::new(1.0, theme::BORDER), egui::StrokeKind::Outside);

    let (cur_x, cur_y) = if let Some(mp) = mpos {
        let wm = app.viewport.screen_to_world(mp, canvas_origin);
        (wm.x.round() as i32, wm.y.round() as i32)
    } else {
        (0, 0)
    };

    let sel_info = if let Some(sid) = &app.designer_selected_node {
        let name = app.project_doc.nodes.get(sid).map(|n| n.name.as_str()).unwrap_or("Node");
        format!("{} ({})", name, sid)
    } else {
        format!("{} Nodes", app.project_doc.nodes.len())
    };

    let status_text = format!("X: {:<4} Y: {:<4} | {}", cur_x, cur_y, sel_info);
    pnt.text(
        Pos2::new(status_rect.left() + 10.0, status_rect.center().y),
        egui::Align2::LEFT_CENTER,
        &status_text,
        egui::FontId::monospace(10.5),
        theme::TEXT_DIM,
    );

    // 2. Bottom-Right Floating Zoom & View Controller
    let zoom_rect = Rect::from_min_size(Pos2::new(r.right() - 212.0, hud_y), Vec2::new(198.0, 26.0));
    pnt.rect_filled(zoom_rect, egui::CornerRadius::same(6), Color32::from_rgb(18, 20, 26));
    pnt.rect_stroke(zoom_rect, egui::CornerRadius::same(6), Stroke::new(1.0, theme::BORDER), egui::StrokeKind::Outside);

    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(zoom_rect), |ui| {
        ui.horizontal_centered(|ui| {
            ui.add_space(4.0);
            if ui.add(egui::Button::new("−").min_size(Vec2::new(22.0, 20.0))).clicked() {
                app.viewport.zoom = (app.viewport.zoom * 0.85).clamp(0.1, 5.0);
            }
            let zoom_pct = format!("{:.0}%", app.viewport.zoom * 100.0);
            if ui.add(egui::Button::new(egui::RichText::new(zoom_pct).size(11.0)).min_size(Vec2::new(50.0, 20.0)))
                .on_hover_text("Click to reset zoom to 100%")
                .clicked()
            {
                app.viewport.zoom = 1.0;
                app.viewport.pan = egui::Vec2::ZERO;
            }
            if ui.add(egui::Button::new("+").min_size(Vec2::new(22.0, 20.0))).clicked() {
                app.viewport.zoom = (app.viewport.zoom * 1.15).clamp(0.1, 5.0);
            }
            ui.separator();
            if ui.add(egui::Button::new(egui::RichText::new("⛶ Fit").size(11.0)).min_size(Vec2::new(42.0, 20.0)))
                .on_hover_text("Center canvas & fit")
                .clicked()
            {
                app.viewport.zoom = 1.0;
                app.viewport.pan = egui::Vec2::ZERO;
            }
        });
    });

    if app.project_doc.nodes.is_empty() {
        pnt.text(
            r.center(),
            egui::Align2::CENTER_CENTER,
            "Click widgets from the left panel to start designing",
            egui::FontId::proportional(14.),
            theme::TEXT_DIM,
        );
    }
}
