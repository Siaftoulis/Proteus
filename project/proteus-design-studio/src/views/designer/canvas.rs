//! Native canvas space calculations, virtual grid, device frames, and HUD overlays.

use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};
use crate::models::{LayoutMode, GRID};
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

pub fn render_grid(app: &ProteusApp, ctx: &egui::Context, pnt: &egui::Painter, r: Rect, canvas_origin: Pos2) {
    let p = app.palette(ctx);
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
                Stroke::new(1., p.border),
            );
            wy += GRID;
        }
        let mut wx = start_x;
        while wx <= world_br.x {
            let sx = app.viewport.world_to_screen(egui::pos2(wx, 0.), canvas_origin).x;
            pnt.line_segment(
                [egui::pos2(sx, r.top()), egui::pos2(sx, r.bottom())],
                Stroke::new(1., p.border),
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
                    pnt.circle_filled(sp, (1.0_f32).max(0.5 * app.viewport.zoom), p.border_light);
                }
                wx += 24.0;
            }
            wy += 24.0;
        }
    }
}

pub fn render_device_frame(app: &ProteusApp, ctx: &egui::Context, pnt: &egui::Painter, r: Rect, canvas_origin: Pos2) {
    let p = app.palette(ctx);
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
    let overlay_fill = if p.is_dark { Color32::from_black_alpha(80) } else { Color32::from_black_alpha(25) };
    for or in &outside_rects {
        pnt.rect_filled(*or, 0, overlay_fill);
    }
    pnt.rect_stroke(dev_rect, 4., Stroke::new(2., p.accent), egui::StrokeKind::Outside);
    let label = format!("{} {} — {}×{}", app.device_preset.icon(), app.device_preset.label(), dw, dh);
    pnt.text(
        egui::pos2(dev_rect.left() + 8., dev_rect.top() - 16.),
        egui::Align2::LEFT_BOTTOM,
        &label,
        egui::FontId::proportional(10.),
        p.accent,
    );

    // Apple Device Chrome Overlays (Dynamic Island / macOS Traffic Lights)
    match app.device_preset {
        crate::models::DevicePreset::AppleIPhone => {
            let island_w = 110.0 * app.viewport.zoom;
            let island_h = 28.0 * app.viewport.zoom;
            let island_rect = Rect::from_center_size(
                Pos2::new(dev_rect.center().x, dev_rect.top() + 18.0 * app.viewport.zoom),
                egui::vec2(island_w, island_h),
            );
            pnt.rect_filled(island_rect, egui::CornerRadius::same((island_h * 0.5) as u8), Color32::BLACK);
            // Camera lens indicator
            pnt.circle_filled(
                Pos2::new(island_rect.right() - 16.0 * app.viewport.zoom, island_rect.center().y),
                4.0 * app.viewport.zoom,
                Color32::from_rgb(20, 25, 45),
            );

            // iOS Home Indicator Bar
            let home_w = 120.0 * app.viewport.zoom;
            let home_h = 4.0 * app.viewport.zoom;
            let home_rect = Rect::from_center_size(
                Pos2::new(dev_rect.center().x, dev_rect.bottom() - 10.0 * app.viewport.zoom),
                egui::vec2(home_w, home_h),
            );
            pnt.rect_filled(home_rect, egui::CornerRadius::same(2), Color32::from_white_alpha(180));
        }
        crate::models::DevicePreset::AppleMacBook => {
            let cy = dev_rect.top() + 12.0 * app.viewport.zoom;
            let start_x = dev_rect.left() + 16.0 * app.viewport.zoom;
            let r = 5.0 * app.viewport.zoom;
            let spacing = 18.0 * app.viewport.zoom;

            pnt.circle_filled(Pos2::new(start_x, cy), r, Color32::from_rgb(255, 95, 86));
            pnt.circle_filled(Pos2::new(start_x + spacing, cy), r, Color32::from_rgb(255, 189, 46));
            pnt.circle_filled(Pos2::new(start_x + spacing * 2.0, cy), r, Color32::from_rgb(39, 201, 63));
        }
        crate::models::DevicePreset::Thermal80mm | crate::models::DevicePreset::Thermal58mm => {
            let is_58 = matches!(app.device_preset, crate::models::DevicePreset::Thermal58mm);
            let zoom = app.viewport.zoom;

            // Top Paper Dispenser Slot (Thermal Printer Mouth)
            let slot_rect = Rect::from_center_size(
                Pos2::new(dev_rect.center().x, dev_rect.top() - 4.0 * zoom),
                egui::vec2(dev_rect.width() + 16.0 * zoom, 6.0 * zoom),
            );
            pnt.rect_filled(slot_rect, egui::CornerRadius::same(3), Color32::from_rgb(45, 52, 64));
            pnt.rect_stroke(slot_rect, egui::CornerRadius::same(3), Stroke::new(1.0, Color32::from_rgb(70, 80, 95)), egui::StrokeKind::Outside);

            // Column Specification Subtitle
            let col_label = if is_58 { "🖨 58mm Roll • 32 Cols (Font A) • 203 DPI" } else { "🖨 80mm Roll • 48 Cols (Font A) • 203 DPI" };
            pnt.text(
                egui::pos2(dev_rect.right(), dev_rect.top() - 16.),
                egui::Align2::RIGHT_BOTTOM,
                col_label,
                egui::FontId::monospace(9.),
                p.accent,
            );

            // Bottom Serrated Tear-Off Cutter (Zig-zag Teeth)
            let tooth_w = 8.0 * zoom;
            let tooth_h = 4.0 * zoom;
            let cut_y = dev_rect.bottom();
            let mut cur_x = dev_rect.left();
            let mut up = true;
            let tooth_stroke = Stroke::new(1.5, p.accent);

            while cur_x < dev_rect.right() {
                let next_x = (cur_x + tooth_w).min(dev_rect.right());
                let next_y = if up { cut_y + tooth_h } else { cut_y };
                let prev_y = if up { cut_y } else { cut_y + tooth_h };
                pnt.line_segment([Pos2::new(cur_x, prev_y), Pos2::new(next_x, next_y)], tooth_stroke);
                cur_x = next_x;
                up = !up;
            }

            // Tear-off cutter label badge below the jagged edge
            pnt.text(
                egui::pos2(dev_rect.center().x, cut_y + 12.0 * zoom),
                egui::Align2::CENTER_TOP,
                "✂ AUTO-CUTTER / SERRATED TEAR-OFF LINE (ESC/POS 0x1D 0x56)",
                egui::FontId::monospace(8.5),
                p.accent,
            );
        }
        _ => {}
    }
}

pub fn render_canvas_hud(
    app: &mut ProteusApp,
    ui: &mut egui::Ui,
    pnt: &egui::Painter,
    r: Rect,
    mpos: Option<Pos2>,
    canvas_origin: Pos2,
) {
    let p = app.palette(ui.ctx());
    let hud_y = r.bottom() - 36.0;

    // 1. Bottom-Left Status & Cursor Coordinates Pill
    let status_rect = Rect::from_min_size(Pos2::new(r.left() + 14.0, hud_y), Vec2::new(420.0, 26.0));
    pnt.rect_filled(status_rect, egui::CornerRadius::same(6), p.elevated);
    pnt.rect_stroke(status_rect, egui::CornerRadius::same(6), Stroke::new(1.0, p.border), egui::StrokeKind::Outside);

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

    let touch_ind = if app.viewport_profile.is_touch_device {
        "📱 Touch (44pt)"
    } else {
        "💻 Desktop"
    };

    let vrr_ind = app.vrr_mode.fps_badge();
    let status_text = format!("X: {:<4} Y: {:<4} | {} | {} | {}", cur_x, cur_y, sel_info, touch_ind, vrr_ind);
    pnt.text(
        Pos2::new(status_rect.left() + 10.0, status_rect.center().y),
        egui::Align2::LEFT_CENTER,
        &status_text,
        egui::FontId::monospace(10.5),
        p.text_dim,
    );

    // 2. Bottom-Right Floating Zoom & View Controller
    let zoom_rect = Rect::from_min_size(Pos2::new(r.right() - 284.0, hud_y), Vec2::new(270.0, 26.0));
    pnt.rect_filled(zoom_rect, egui::CornerRadius::same(6), p.elevated);
    pnt.rect_stroke(zoom_rect, egui::CornerRadius::same(6), Stroke::new(1.0, p.border), egui::StrokeKind::Outside);

    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(zoom_rect), |ui| {
        ui.horizontal_centered(|ui| {
            ui.add_space(4.0);
            let snap_label = if app.smart_snap_enabled { "🧲 Snap" } else { "Snap" };
            if ui.add(
                egui::Button::new(
                    egui::RichText::new(snap_label)
                        .size(11.0)
                        .color(if app.smart_snap_enabled { p.accent } else { p.text_dim })
                )
                .fill(if app.smart_snap_enabled { Color32::from_rgba_unmultiplied(79, 140, 237, 25) } else { Color32::TRANSPARENT })
                .min_size(Vec2::new(54.0, 20.0))
            )
            .on_hover_text("Toggle Mac-style Magnetic Smart Snapping ('Aimbot')")
            .clicked()
            {
                app.smart_snap_enabled = !app.smart_snap_enabled;
                if !app.smart_snap_enabled {
                    app.active_snap_guides.clear();
                }
            }
            ui.separator();
            if ui.add(egui::Button::new("−").min_size(Vec2::new(20.0, 20.0))).clicked() {
                app.viewport.zoom = (app.viewport.zoom * 0.85).clamp(0.1, 5.0);
            }
            let zoom_pct = format!("{:.0}%", app.viewport.zoom * 100.0);
            if ui.add(egui::Button::new(egui::RichText::new(zoom_pct).size(11.0)).min_size(Vec2::new(46.0, 20.0)))
                .on_hover_text("Click to reset zoom to 100%")
                .clicked()
            {
                app.viewport.zoom = 1.0;
                app.viewport.pan = egui::Vec2::ZERO;
            }
            if ui.add(egui::Button::new("+").min_size(Vec2::new(20.0, 20.0))).clicked() {
                app.viewport.zoom = (app.viewport.zoom * 1.15).clamp(0.1, 5.0);
            }
            ui.separator();
            if ui.add(egui::Button::new(egui::RichText::new("⛶ Fit").size(11.0)).min_size(Vec2::new(38.0, 20.0)))
                .on_hover_text("Center canvas & fit")
                .clicked()
            {
                app.viewport.zoom = 1.0;
                app.viewport.pan = egui::Vec2::ZERO;
            }
        });
    });
}

/// Renders live collaborative multiplayer peer cursors and peer selection boxes over the canvas.
pub fn render_collaborator_cursors(
    app: &ProteusApp,
    pnt: &egui::Painter,
    r: Rect,
    canvas_origin: Pos2,
) {
    for peer in &app.live_collaborators {
        // 1. Draw Peer Selection Box if node exists
        if let Some(ref sel_id) = peer.selected_node_id {
            if let Some(node) = app.project_doc.nodes.get(sel_id) {
                let (nw, nh) = match (&node.layout.width, &node.layout.height) {
                    (crate::scene::Sizing::Fixed(w), crate::scene::Sizing::Fixed(h)) => (*w, *h),
                    (crate::scene::Sizing::Fixed(w), _) => (*w, 100.0),
                    (_, crate::scene::Sizing::Fixed(h)) => (200.0, *h),
                    _ => (200.0, 100.0),
                };
                let w_min = Pos2::new(node.position.0, node.position.1);
                let s_min = app.viewport.world_to_screen(w_min, canvas_origin);
                let s_max = app.viewport.world_to_screen(Pos2::new(node.position.0 + nw, node.position.1 + nh), canvas_origin);
                let node_screen_rect = Rect::from_min_max(s_min, s_max);

                let peer_color = Color32::from_rgb(peer.color_rgb[0], peer.color_rgb[1], peer.color_rgb[2]);
                pnt.rect_stroke(node_screen_rect, 0.0, Stroke::new(1.5, peer_color), egui::StrokeKind::Outside);

                // Small peer tag at top-left of node
                let tag_rect = Rect::from_min_size(Pos2::new(node_screen_rect.left(), node_screen_rect.top() - 14.0), Vec2::new(80.0, 14.0));
                pnt.rect_filled(tag_rect, egui::CornerRadius::same(2), peer_color);
                pnt.text(tag_rect.center(), egui::Align2::CENTER_CENTER, &peer.user_name, egui::FontId::proportional(9.0), Color32::WHITE);
            }
        }

        // 2. Draw Peer Multiplayer Cursor
        let scr = app.viewport.world_to_screen(Pos2::new(peer.cursor_world.0, peer.cursor_world.1), canvas_origin);
        if r.contains(scr) {
            let peer_color = Color32::from_rgb(peer.color_rgb[0], peer.color_rgb[1], peer.color_rgb[2]);
            let tip = scr;
            let p1 = Pos2::new(scr.x + 12.0, scr.y + 12.0);
            let p2 = Pos2::new(scr.x + 4.0, scr.y + 15.0);
            pnt.add(egui::Shape::convex_polygon(
                vec![tip, p1, p2],
                peer_color,
                Stroke::new(1.0, Color32::WHITE),
            ));

            let badge_text = format!("{} ({})", peer.user_name, peer.user_role);
            let badge_w = (badge_text.len() as f32 * 6.2).max(60.0);
            let badge_rect = Rect::from_min_size(Pos2::new(scr.x + 14.0, scr.y + 12.0), Vec2::new(badge_w, 16.0));
            pnt.rect_filled(badge_rect, egui::CornerRadius::same(3), peer_color);
            pnt.text(badge_rect.center(), egui::Align2::CENTER_CENTER, &badge_text, egui::FontId::proportional(9.5), Color32::WHITE);
        }
    }
}
