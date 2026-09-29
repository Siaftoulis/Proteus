//! PDS (Proteus Design System) Logo & Store Brand Renderer for Proteus Client.
//! Renders the official Proteus identity or custom store branding icons.

use egui::{self, Color32, CornerRadius, FontId, Pos2, Rect, Stroke, Vec2};

pub const PC_GOLD: Color32 = Color32::from_rgb(197, 168, 128); // #C5A880 warm ochre gold
pub const PC_CYAN: Color32 = Color32::from_rgb(56, 189, 248);  // Sky cyan

pub const STORE_ICONS: &[(&str, &str)] = &[
    ("default", "King Proteus Emblem (Official)"),
    ("wrench", "Service & Workshop / Επισκευές"),
    ("hardware", "Hardware & Fasteners / Σιδηρικά"),
    ("anchor", "Marine & Nautical / Ναυτιλιακά"),
    ("auto", "Auto Garage / Συνεργείο Αυτοκινήτων"),
    ("store", "General Store & Trades / Εμπορικό"),
];

/// Renders the legacy Proteus Client (PC) monogram vector fallback.
#[allow(dead_code)]
pub fn paint_pc_logo(painter: &egui::Painter, rect: Rect, is_dark: bool) {
    let w = rect.width();
    let h = rect.height();
    let min = rect.min;

    let p_color = if is_dark {
        crate::theme::ICY_MIST
    } else {
        crate::theme::DEEP_SLATE_NAVY
    };

    let c_color = PC_GOLD;
    let wave_color = p_color;

    let stroke_width = (w * 0.10).max(1.8);
    let p_stroke = Stroke::new(stroke_width, p_color);
    let c_stroke = Stroke::new(stroke_width, c_color);
    let wave_stroke = Stroke::new((w * 0.085).max(1.5), wave_color);

    // Stem of P
    let stem_top = Pos2::new(min.x + w * 0.32, min.y + h * 0.10);
    let stem_bot = Pos2::new(min.x + w * 0.32, min.y + h * 0.62);
    painter.line_segment([stem_top, stem_bot], p_stroke);

    // Top serif on P stem
    let serif_left = Pos2::new(min.x + w * 0.22, min.y + h * 0.16);
    painter.line_segment([serif_left, stem_top], p_stroke);

    // Mid-stem spur on P
    let spur_left = Pos2::new(min.x + w * 0.18, min.y + h * 0.44);
    let spur_right = Pos2::new(min.x + w * 0.32, min.y + h * 0.44);
    painter.line_segment([spur_left, spur_right], p_stroke);

    // Top horizontal bar of P loop
    let top_bar_right = Pos2::new(min.x + w * 0.60, min.y + h * 0.10);
    painter.line_segment([stem_top, top_bar_right], p_stroke);

    // Middle bar of P loop
    let mid_bar_right = Pos2::new(min.x + w * 0.58, min.y + h * 0.36);
    let mid_bar_left = Pos2::new(min.x + w * 0.32, min.y + h * 0.36);
    painter.line_segment([mid_bar_left, mid_bar_right], p_stroke);

    // Right curve of P loop
    let p_arc_control = Pos2::new(min.x + w * 0.82, min.y + h * 0.23);
    painter.add(egui::epaint::QuadraticBezierShape::from_points_stroke(
        [top_bar_right, p_arc_control, mid_bar_right],
        false,
        Color32::TRANSPARENT,
        p_stroke,
    ));

    // Interlocking 'C' in warm gold
    let c_center = Pos2::new(min.x + w * 0.66, min.y + h * 0.42);
    let c_radius = w * 0.22;
    let mut c_points = Vec::new();
    let num_pts = 16;
    let start_angle = std::f32::consts::PI * 0.30;
    let end_angle = std::f32::consts::PI * 1.70;
    for i in 0..=num_pts {
        let frac = i as f32 / num_pts as f32;
        let angle = start_angle + frac * (end_angle - start_angle);
        c_points.push(Pos2::new(
            c_center.x + c_radius * angle.cos(),
            c_center.y - c_radius * angle.sin(),
        ));
    }
    for pair in c_points.windows(2) {
        painter.line_segment([pair[0], pair[1]], c_stroke);
    }

    // Twin Marine Waves
    let w1_p0 = Pos2::new(min.x + w * 0.12, min.y + h * 0.72);
    let w1_c0 = Pos2::new(min.x + w * 0.36, min.y + h * 0.62);
    let w1_c1 = Pos2::new(min.x + w * 0.64, min.y + h * 0.80);
    let w1_p1 = Pos2::new(min.x + w * 0.90, min.y + h * 0.70);

    painter.add(egui::epaint::CubicBezierShape::from_points_stroke(
        [w1_p0, w1_c0, w1_c1, w1_p1],
        false,
        Color32::TRANSPARENT,
        wave_stroke,
    ));

    let w2_p0 = Pos2::new(min.x + w * 0.12, min.y + h * 0.87);
    let w2_c0 = Pos2::new(min.x + w * 0.36, min.y + h * 0.77);
    let w2_c1 = Pos2::new(min.x + w * 0.64, min.y + h * 0.95);
    let w2_p1 = Pos2::new(min.x + w * 0.90, min.y + h * 0.85);

    painter.add(egui::epaint::CubicBezierShape::from_points_stroke(
        [w2_p0, w2_c0, w2_c1, w2_p1],
        false,
        Color32::TRANSPARENT,
        wave_stroke,
    ));
}

/// Renders a sleek repair/mechanic workshop icon (diagonal wrench with open jaw).
pub fn paint_wrench_logo(painter: &egui::Painter, rect: Rect, is_dark: bool) {
    let w = rect.width();
    let min = rect.min;
    let stroke_color = if is_dark { PC_GOLD } else { crate::theme::DEEP_SLATE_NAVY };
    let stroke = Stroke::new((w * 0.12).max(1.8), stroke_color);

    // Shaft of wrench (diagonal)
    let p_start = Pos2::new(min.x + w * 0.25, min.y + w * 0.75);
    let p_end = Pos2::new(min.x + w * 0.68, min.y + w * 0.32);
    painter.line_segment([p_start, p_end], stroke);

    // Open jaw head
    let jaw_left = Pos2::new(min.x + w * 0.60, min.y + w * 0.18);
    let jaw_top = Pos2::new(min.x + w * 0.82, min.y + w * 0.20);
    let jaw_right = Pos2::new(min.x + w * 0.82, min.y + w * 0.42);
    painter.line_segment([p_end, jaw_left], stroke);
    painter.line_segment([jaw_left, jaw_top], stroke);
    painter.line_segment([jaw_top, jaw_right], stroke);

    // Base ring
    let ring_center = Pos2::new(min.x + w * 0.20, min.y + w * 0.80);
    painter.circle_stroke(ring_center, w * 0.10, stroke);
}

/// Renders a hardware bolt & nut icon for hardware stores & technical supplies.
pub fn paint_hardware_logo(painter: &egui::Painter, rect: Rect, is_dark: bool) {
    let w = rect.width();
    let min = rect.min;
    let bolt_color = if is_dark { PC_CYAN } else { crate::theme::DEEP_SLATE_NAVY };
    let stroke = Stroke::new((w * 0.10).max(1.5), bolt_color);

    // Hexagonal head
    let c = Pos2::new(min.x + w * 0.50, min.y + w * 0.30);
    let r = w * 0.24;
    let mut pts = Vec::with_capacity(6);
    for i in 0..6 {
        let angle = std::f32::consts::PI / 3.0 * i as f32;
        pts.push(Pos2::new(c.x + r * angle.cos(), c.y + r * angle.sin()));
    }
    for pair in pts.windows(2) {
        painter.line_segment([pair[0], pair[1]], stroke);
    }
    if let (Some(first), Some(last)) = (pts.first(), pts.last()) {
        painter.line_segment([*last, *first], stroke);
    }

    // Threaded shaft
    let shaft_left = min.x + w * 0.38;
    let shaft_right = min.x + w * 0.62;
    painter.line_segment([Pos2::new(shaft_left, min.y + w * 0.52), Pos2::new(shaft_left, min.y + w * 0.88)], stroke);
    painter.line_segment([Pos2::new(shaft_right, min.y + w * 0.52), Pos2::new(shaft_right, min.y + w * 0.88)], stroke);
    painter.line_segment([Pos2::new(shaft_left, min.y + w * 0.88), Pos2::new(shaft_right, min.y + w * 0.88)], stroke);

    // Thread grooves
    painter.line_segment([Pos2::new(shaft_left, min.y + w * 0.64), Pos2::new(shaft_right, min.y + w * 0.64)], stroke);
    painter.line_segment([Pos2::new(shaft_left, min.y + w * 0.76), Pos2::new(shaft_right, min.y + w * 0.76)], stroke);
}

/// Renders a nautical anchor icon for marine and shipping suppliers.
pub fn paint_anchor_logo(painter: &egui::Painter, rect: Rect, is_dark: bool) {
    let w = rect.width();
    let min = rect.min;
    let col = if is_dark { crate::theme::ICY_MIST } else { crate::theme::DEEP_SLATE_NAVY };
    let stroke = Stroke::new((w * 0.10).max(1.5), col);

    // Center shaft
    let top = Pos2::new(min.x + w * 0.50, min.y + w * 0.25);
    let bot = Pos2::new(min.x + w * 0.50, min.y + w * 0.78);
    painter.line_segment([top, bot], stroke);

    // Top ring
    painter.circle_stroke(Pos2::new(min.x + w * 0.50, min.y + w * 0.16), w * 0.08, stroke);

    // Crossbar
    painter.line_segment([Pos2::new(min.x + w * 0.30, min.y + w * 0.36), Pos2::new(min.x + w * 0.70, min.y + w * 0.36)], stroke);

    // Curved flukes
    let p_left = Pos2::new(min.x + w * 0.18, min.y + w * 0.65);
    let p_ctrl = Pos2::new(min.x + w * 0.50, min.y + w * 0.90);
    let p_right = Pos2::new(min.x + w * 0.82, min.y + w * 0.65);
    painter.add(egui::epaint::QuadraticBezierShape::from_points_stroke(
        [p_left, p_ctrl, p_right],
        false,
        Color32::TRANSPARENT,
        stroke,
    ));
}

/// Renders an automobile silhouette / garage icon.
pub fn paint_auto_logo(painter: &egui::Painter, rect: Rect, is_dark: bool) {
    let w = rect.width();
    let min = rect.min;
    let col = if is_dark { PC_GOLD } else { crate::theme::DEEP_SLATE_NAVY };
    let stroke = Stroke::new((w * 0.10).max(1.5), col);

    // Car profile lines
    let p1 = Pos2::new(min.x + w * 0.12, min.y + w * 0.65);
    let p2 = Pos2::new(min.x + w * 0.28, min.y + w * 0.40);
    let p3 = Pos2::new(min.x + w * 0.65, min.y + w * 0.40);
    let p4 = Pos2::new(min.x + w * 0.88, min.y + w * 0.65);
    painter.line_segment([p1, p2], stroke);
    painter.line_segment([p2, p3], stroke);
    painter.line_segment([p3, p4], stroke);
    painter.line_segment([p1, Pos2::new(min.x + w * 0.24, min.y + w * 0.65)], stroke);
    painter.line_segment([Pos2::new(min.x + w * 0.42, min.y + w * 0.65), Pos2::new(min.x + w * 0.68, min.y + w * 0.65)], stroke);
    painter.line_segment([Pos2::new(min.x + w * 0.82, min.y + w * 0.65), p4], stroke);

    // Wheels
    painter.circle_stroke(Pos2::new(min.x + w * 0.33, min.y + w * 0.66), w * 0.09, stroke);
    painter.circle_stroke(Pos2::new(min.x + w * 0.75, min.y + w * 0.66), w * 0.09, stroke);
}

/// Renders a retail storefront / market awning icon.
pub fn paint_store_logo(painter: &egui::Painter, rect: Rect, is_dark: bool) {
    let w = rect.width();
    let min = rect.min;
    let col = if is_dark { PC_CYAN } else { crate::theme::DEEP_SLATE_NAVY };
    let stroke = Stroke::new((w * 0.10).max(1.5), col);

    // Awning roof
    let roof_top = Pos2::new(min.x + w * 0.50, min.y + w * 0.15);
    let roof_l = Pos2::new(min.x + w * 0.15, min.y + w * 0.42);
    let roof_r = Pos2::new(min.x + w * 0.85, min.y + w * 0.42);
    painter.line_segment([roof_l, roof_top], stroke);
    painter.line_segment([roof_top, roof_r], stroke);
    painter.line_segment([roof_l, roof_r], stroke);

    // Pillars and base
    painter.line_segment([Pos2::new(min.x + w * 0.22, min.y + w * 0.42), Pos2::new(min.x + w * 0.22, min.y + w * 0.82)], stroke);
    painter.line_segment([Pos2::new(min.x + w * 0.78, min.y + w * 0.42), Pos2::new(min.x + w * 0.78, min.y + w * 0.82)], stroke);
    painter.line_segment([Pos2::new(min.x + w * 0.12, min.y + w * 0.82), Pos2::new(min.x + w * 0.88, min.y + w * 0.82)], stroke);

    // Doorway
    painter.line_segment([Pos2::new(min.x + w * 0.42, min.y + w * 0.82), Pos2::new(min.x + w * 0.42, min.y + w * 0.56)], stroke);
    painter.line_segment([Pos2::new(min.x + w * 0.42, min.y + w * 0.56), Pos2::new(min.x + w * 0.58, min.y + w * 0.56)], stroke);
    painter.line_segment([Pos2::new(min.x + w * 0.58, min.y + w * 0.56), Pos2::new(min.x + w * 0.58, min.y + w * 0.82)], stroke);
}

/// Renders a 2-letter clean monogram badge for custom store initials.
pub fn paint_custom_monogram(painter: &egui::Painter, rect: Rect, initials: &str, is_dark: bool) {
    let bg = if is_dark { crate::theme::BG_CARD } else { crate::theme::BG_PANEL };
    let border = if is_dark { PC_GOLD } else { crate::theme::DEEP_SLATE_NAVY };
    painter.rect_filled(rect, CornerRadius::same(5), bg);
    painter.rect_stroke(rect, CornerRadius::same(5), Stroke::new(1.0, border), egui::StrokeKind::Outside);

    let text_col = if is_dark { crate::theme::ICY_MIST } else { crate::theme::DEEP_SLATE_NAVY };
    let font_size = (rect.height() * 0.50).max(10.0);
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        initials,
        FontId::proportional(font_size),
        text_col,
    );
}

static EMBLEM_RGBA_128: &[u8] = include_bytes!("../../../assets/proteus_emblem_128.rgba");

/// Obtains or caches the King Proteus emblem texture handle for Proteus Client.
pub fn get_or_load_pc_emblem_texture(ctx: &egui::Context) -> egui::TextureHandle {
    let id = egui::Id::new("pc_king_proteus_emblem_tex");
    ctx.data_mut(|d| {
        if let Some(tex) = d.get_temp::<egui::TextureHandle>(id) {
            tex
        } else {
            let img = egui::ColorImage::from_rgba_unmultiplied([128, 128], EMBLEM_RGBA_128);
            let tex = ctx.load_texture("pc_king_proteus_emblem", img, egui::TextureOptions::LINEAR);
            d.insert_temp(id, tex.clone());
            tex
        }
    })
}

/// Master widget to render either the official King Proteus emblem or custom store brand icon.
pub fn render_store_logo_widget(
    ui: &mut egui::Ui,
    size: Vec2,
    logo_icon: &str,
    shop_name: &str,
    is_dark: bool,
) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::hover());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        match logo_icon.trim().to_lowercase().as_str() {
            "wrench" | "tools" | "repair" => paint_wrench_logo(painter, rect, is_dark),
            "hardware" | "bolt" | "tools_hardware" => paint_hardware_logo(painter, rect, is_dark),
            "anchor" | "nautical" | "marine" => paint_anchor_logo(painter, rect, is_dark),
            "auto" | "car" | "garage" => paint_auto_logo(painter, rect, is_dark),
            "store" | "retail" | "shop" => paint_store_logo(painter, rect, is_dark),
            "default" | "" | "proteus" => {
                let tex = get_or_load_pc_emblem_texture(ui.ctx());
                painter.image(
                    tex.id(),
                    rect,
                    Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );
            }
            custom => {
                // If custom 2-letter monogram provided, render initials
                let initials = if custom.len() <= 3 && !custom.is_empty() {
                    custom.to_uppercase()
                } else {
                    let words: Vec<&str> = shop_name.split_whitespace().collect();
                    if words.len() >= 2 {
                        let c1 = words[0].chars().next().unwrap_or('P');
                        let c2 = words[1].chars().next().unwrap_or('S');
                        format!("{}{}", c1, c2).to_uppercase()
                    } else {
                        shop_name.chars().take(2).collect::<String>().to_uppercase()
                    }
                };
                paint_custom_monogram(painter, rect, &initials, is_dark);
            }
        }
    }
    resp
}

/// Helper widget to render the default Proteus Client King Proteus emblem.
#[allow(dead_code)]
pub fn pc_logo_widget(ui: &mut egui::Ui, size: Vec2, _is_dark: bool) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::hover());
    if ui.is_rect_visible(rect) {
        let tex = get_or_load_pc_emblem_texture(ui.ctx());
        ui.painter().image(
            tex.id(),
            rect,
            Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
    }
    resp
}
