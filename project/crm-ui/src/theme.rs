use eframe::egui::{self, Color32, Stroke};

// ── Affinity / Linear / Figma Luxury Minimalist Dark Palette ──
pub const BG: Color32           = Color32::from_rgb(14, 15, 18);     // Deep obsidian background
pub const PANEL: Color32        = Color32::from_rgb(20, 21, 26);     // Sleek panel surface
pub const ELEVATED: Color32     = Color32::from_rgb(26, 28, 36);     // Elevated surfaces / cards
pub const WIDGET_BG: Color32    = Color32::from_rgb(30, 33, 42);     // Inputs, buttons base
pub const HOVER: Color32        = Color32::from_rgb(40, 44, 56);     // Interactive hover
pub const ACTIVE: Color32       = Color32::from_rgb(50, 55, 70);     // Interactive active
pub const BORDER: Color32       = Color32::from_rgb(36, 39, 50);     // Crisp 1px hairline border
pub const BORDER_LIGHT: Color32 = Color32::from_rgb(48, 52, 66);     // Subtle separator
pub const FOCUS: Color32        = Color32::from_rgb(99, 102, 241);   // Linear indigo
pub const TEXT: Color32         = Color32::from_rgb(243, 244, 246);   // Crisp primary text
pub const TEXT_DIM: Color32     = Color32::from_rgb(156, 163, 175);  // Balanced secondary text
pub const TEXT_MUTED: Color32   = Color32::from_rgb(107, 114, 128);  // Subtle captions
pub const ACCENT: Color32       = Color32::from_rgb(99, 102, 241);   // Primary Accent (Indigo)
pub const ACCENT_DIM: Color32   = Color32::from_rgb(42, 45, 80);     // Accent fill / highlight
pub const ACCENT_ORANGE: Color32 = Color32::from_rgb(245, 158, 11);  // Warm Amber
pub const ACCENT_GREEN: Color32  = Color32::from_rgb(34, 197, 94);   // Vibrant Emerald
pub const ACCENT_RED: Color32    = Color32::from_rgb(239, 68, 68);   // Clean Coral Red
pub const ACCENT_PURPLE: Color32 = Color32::from_rgb(168, 85, 247);  // Electric Purple
pub const ACCENT_CYAN: Color32   = Color32::from_rgb(14, 165, 233);  // Sky Cyan
pub const SELECTED: Color32      = Color32::from_rgb(99, 102, 241);
pub const HEADER_TRIGGER: Color32   = Color32::from_rgb(239, 68, 68);
pub const HEADER_ACTION: Color32    = Color32::from_rgb(99, 102, 241);
pub const HEADER_CONDITION: Color32 = Color32::from_rgb(245, 158, 11);
pub const HEADER_GATE: Color32      = Color32::from_rgb(34, 197, 94);

pub fn configure_egui_style(ctx: &egui::Context) {
    use egui::style::*;
    ctx.set_pixels_per_point(1.25);
    let rounding = egui::CornerRadius::same(5);
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(8., 6.);
    style.spacing.button_padding = egui::vec2(12., 5.);
    style.spacing.indent = 14.;
    style.spacing.icon_width = 16.;
    style.spacing.icon_width_inner = 12.;
    style.spacing.interact_size = egui::vec2(32., 22.);
    style.spacing.slider_width = 130.;
    style.visuals = Visuals {
        dark_mode: true,
        override_text_color: Some(TEXT),
        window_fill: ELEVATED,
        panel_fill: PANEL,
        faint_bg_color: PANEL,
        extreme_bg_color: BG,
        code_bg_color: WIDGET_BG,
        window_corner_radius: rounding,
        menu_corner_radius: rounding,
        warn_fg_color: ACCENT_ORANGE,
        error_fg_color: ACCENT_RED,
        hyperlink_color: ACCENT,
        selection: Selection { bg_fill: ACCENT_DIM, stroke: Stroke::new(1., ACCENT) },
        widgets: Widgets {
            noninteractive: WidgetVisuals { bg_fill: PANEL, weak_bg_fill: WIDGET_BG, bg_stroke: Stroke::new(1., BORDER), fg_stroke: Stroke::new(1., TEXT_DIM), expansion: 0., corner_radius: rounding },
            inactive: WidgetVisuals { bg_fill: WIDGET_BG, weak_bg_fill: PANEL, bg_stroke: Stroke::new(1., BORDER), fg_stroke: Stroke::new(1., TEXT), expansion: 0., corner_radius: rounding },
            hovered: WidgetVisuals { bg_fill: HOVER, weak_bg_fill: PANEL, bg_stroke: Stroke::new(1., BORDER_LIGHT), fg_stroke: Stroke::new(1.2, TEXT), expansion: 0.5, corner_radius: rounding },
            active: WidgetVisuals { bg_fill: ACTIVE, weak_bg_fill: PANEL, bg_stroke: Stroke::new(1., ACCENT), fg_stroke: Stroke::new(1.5, TEXT), expansion: 0.5, corner_radius: rounding },
            open: WidgetVisuals { bg_fill: ELEVATED, weak_bg_fill: PANEL, bg_stroke: Stroke::new(1., ACCENT), fg_stroke: Stroke::new(1.2, TEXT), expansion: 0., corner_radius: rounding },
        },
        popup_shadow: egui::epaint::Shadow { offset: [0, 4], blur: 16, spread: 0, color: Color32::from_black_alpha(100) },
        window_shadow: egui::epaint::Shadow { offset: [0, 6], blur: 24, spread: 0, color: Color32::from_black_alpha(130) },
        ..Default::default()
    };
    style.text_styles = [
        (egui::TextStyle::Heading, egui::FontId::proportional(20.)),
        (egui::TextStyle::Body, egui::FontId::proportional(14.)),
        (egui::TextStyle::Monospace, egui::FontId::monospace(13.)),
        (egui::TextStyle::Button, egui::FontId::proportional(13.)),
        (egui::TextStyle::Small, egui::FontId::proportional(11.)),
    ].into();
    ctx.set_style(style);
}
