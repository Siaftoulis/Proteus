//! PDS (Proteus Design System) Logo Renderer.
//! Renders the official Proteus identity: stylized 'P' monogram with twin marine waves.

use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};

pub fn paint_pds_logo(painter: &egui::Painter, rect: Rect, color: Color32) {
    let w = rect.width();
    let h = rect.height();
    let min = rect.min;

    let stroke_width = (w * 0.11).max(1.8);
    let stroke = Stroke::new(stroke_width, color);

    // ── Upper section: Stylized 'P' ──
    // Stem of P
    let stem_top = Pos2::new(min.x + w * 0.28, min.y + h * 0.10);
    let stem_bot = Pos2::new(min.x + w * 0.28, min.y + h * 0.62);
    painter.line_segment([stem_top, stem_bot], stroke);

    // Loop/Bowl of P: Top bar, right curve, middle bar
    let top_bar_right = Pos2::new(min.x + w * 0.58, min.y + h * 0.10);
    painter.line_segment([stem_top, top_bar_right], stroke);

    let mid_bar_right = Pos2::new(min.x + w * 0.58, min.y + h * 0.38);
    let mid_bar_left = Pos2::new(min.x + w * 0.28, min.y + h * 0.38);
    painter.line_segment([mid_bar_left, mid_bar_right], stroke);

    // Right arc of P bowl
    let p_arc_control = Pos2::new(min.x + w * 0.84, min.y + h * 0.24);
    painter.add(egui::epaint::QuadraticBezierShape::from_points_stroke(
        [top_bar_right, p_arc_control, mid_bar_right],
        false,
        Color32::TRANSPARENT,
        stroke,
    ));

    // ── Lower section: Twin Marine Waves ──
    let wave_stroke_width = (w * 0.09).max(1.4);
    let wave_stroke = Stroke::new(wave_stroke_width, color);

    // Upper wave (cubic bezier)
    let w1_p0 = Pos2::new(min.x + w * 0.14, min.y + h * 0.72);
    let w1_c0 = Pos2::new(min.x + w * 0.38, min.y + h * 0.62);
    let w1_c1 = Pos2::new(min.x + w * 0.64, min.y + h * 0.80);
    let w1_p1 = Pos2::new(min.x + w * 0.88, min.y + h * 0.70);

    painter.add(egui::epaint::CubicBezierShape::from_points_stroke(
        [w1_p0, w1_c0, w1_c1, w1_p1],
        false,
        Color32::TRANSPARENT,
        wave_stroke,
    ));

    // Lower wave (cubic bezier)
    let w2_p0 = Pos2::new(min.x + w * 0.14, min.y + h * 0.87);
    let w2_c0 = Pos2::new(min.x + w * 0.38, min.y + h * 0.77);
    let w2_c1 = Pos2::new(min.x + w * 0.64, min.y + h * 0.95);
    let w2_p1 = Pos2::new(min.x + w * 0.88, min.y + h * 0.85);

    painter.add(egui::epaint::CubicBezierShape::from_points_stroke(
        [w2_p0, w2_c0, w2_c1, w2_p1],
        false,
        Color32::TRANSPARENT,
        wave_stroke,
    ));
}

static EMBLEM_RGBA_128: &[u8] = include_bytes!("../../../../assets/proteus_emblem_128.rgba");

/// Obtains or caches the King Proteus emblem texture handle.
pub fn get_or_load_emblem_texture(ctx: &egui::Context) -> egui::TextureHandle {
    let id = egui::Id::new("pds_king_proteus_emblem_tex");
    ctx.data_mut(|d| {
        if let Some(tex) = d.get_temp::<egui::TextureHandle>(id) {
            tex
        } else {
            let img = egui::ColorImage::from_rgba_unmultiplied([128, 128], EMBLEM_RGBA_128);
            let tex = ctx.load_texture("pds_king_proteus_emblem", img, egui::TextureOptions::LINEAR);
            d.insert_temp(id, tex.clone());
            tex
        }
    })
}

/// Helper widget to render the official King Proteus logo emblem.
pub fn pds_logo_widget(ui: &mut egui::Ui, size: Vec2, _color: Color32) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::hover());
    if ui.is_rect_visible(rect) {
        let tex = get_or_load_emblem_texture(ui.ctx());
        ui.painter().image(
            tex.id(),
            rect,
            Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
    }
    resp
}
