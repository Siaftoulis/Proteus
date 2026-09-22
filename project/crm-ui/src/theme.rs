use eframe::egui::{self, Color32, Stroke};
use serde::{Deserialize, Serialize};

// ── Affinity / Linear / Figma Luxury Minimalist Dark Palette (Fallback Constants) ──
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

/// Operating system / UI theme mode choice.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum ThemeMode {
    System,
    #[default]
    Dark,
    Light,
}

impl ThemeMode {
    pub fn label(&self) -> &'static str {
        match self {
            Self::System => "System Auto",
            Self::Dark => "Dark (Obsidian)",
            Self::Light => "Light (Minimalist)",
        }
    }

    pub fn resolve_is_dark(&self, ctx: &egui::Context) -> bool {
        match self {
            Self::System => !matches!(ctx.system_theme(), Some(egui::Theme::Light)),
            Self::Dark => true,
            Self::Light => false,
        }
    }
}

/// Luxury accent color presets.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum AccentPreset {
    #[default]
    Indigo,
    Sapphire,
    Emerald,
    Amber,
    Rose,
    Slate,
}

impl AccentPreset {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Indigo => "Linear Indigo",
            Self::Sapphire => "Sapphire Blue",
            Self::Emerald => "Emerald Green",
            Self::Amber => "Amber Gold",
            Self::Rose => "Rose Red",
            Self::Slate => "Minimal Slate",
        }
    }

    pub fn color(&self) -> Color32 {
        match self {
            Self::Indigo => Color32::from_rgb(99, 102, 241),
            Self::Sapphire => Color32::from_rgb(37, 99, 235),
            Self::Emerald => Color32::from_rgb(22, 163, 74),
            Self::Amber => Color32::from_rgb(217, 119, 6),
            Self::Rose => Color32::from_rgb(225, 29, 72),
            Self::Slate => Color32::from_rgb(100, 116, 139),
        }
    }

    pub fn dim_color(&self, is_dark: bool) -> Color32 {
        let c = self.color();
        if is_dark {
            Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 55)
        } else {
            Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 35)
        }
    }
}

/// Complete resolved UI color palette tokens.
#[derive(Clone, Debug)]
pub struct Palette {
    pub is_dark: bool,
    pub bg: Color32,
    pub panel: Color32,
    pub elevated: Color32,
    pub widget_bg: Color32,
    pub hover: Color32,
    pub active: Color32,
    pub border: Color32,
    pub border_light: Color32,
    pub text: Color32,
    pub text_dim: Color32,
    pub text_muted: Color32,
    pub accent: Color32,
    pub accent_dim: Color32,
    pub accent_orange: Color32,
    pub accent_green: Color32,
    pub accent_red: Color32,
    pub accent_purple: Color32,
    pub accent_cyan: Color32,
}

impl Palette {
    pub fn new(is_dark: bool, accent_preset: AccentPreset) -> Self {
        let accent = accent_preset.color();
        let accent_dim = accent_preset.dim_color(is_dark);

        if is_dark {
            Self {
                is_dark: true,
                bg: Color32::from_rgb(14, 15, 18),
                panel: Color32::from_rgb(20, 21, 26),
                elevated: Color32::from_rgb(26, 28, 36),
                widget_bg: Color32::from_rgb(30, 33, 42),
                hover: Color32::from_rgb(40, 44, 56),
                active: Color32::from_rgb(50, 55, 70),
                border: Color32::from_rgb(36, 39, 50),
                border_light: Color32::from_rgb(48, 52, 66),
                text: Color32::from_rgb(243, 244, 246),
                text_dim: Color32::from_rgb(156, 163, 175),
                text_muted: Color32::from_rgb(107, 114, 128),
                accent,
                accent_dim,
                accent_orange: Color32::from_rgb(245, 158, 11),
                accent_green: Color32::from_rgb(34, 197, 94),
                accent_red: Color32::from_rgb(239, 68, 68),
                accent_purple: Color32::from_rgb(168, 85, 247),
                accent_cyan: Color32::from_rgb(14, 165, 233),
            }
        } else {
            Self {
                is_dark: false,
                bg: Color32::from_rgb(243, 244, 246),
                panel: Color32::from_rgb(255, 255, 255),
                elevated: Color32::from_rgb(249, 250, 251),
                widget_bg: Color32::from_rgb(243, 244, 246),
                hover: Color32::from_rgb(229, 231, 235),
                active: Color32::from_rgb(209, 213, 219),
                border: Color32::from_rgb(220, 224, 230),
                border_light: Color32::from_rgb(235, 238, 242),
                text: Color32::from_rgb(17, 24, 39),
                text_dim: Color32::from_rgb(75, 85, 99),
                text_muted: Color32::from_rgb(156, 163, 175),
                accent,
                accent_dim,
                accent_orange: Color32::from_rgb(217, 119, 6),
                accent_green: Color32::from_rgb(22, 163, 74),
                accent_red: Color32::from_rgb(220, 38, 38),
                accent_purple: Color32::from_rgb(147, 51, 234),
                accent_cyan: Color32::from_rgb(2, 132, 199),
            }
        }
    }
}

pub fn configure_egui_style(ctx: &egui::Context, theme_mode: ThemeMode, accent_preset: AccentPreset) {
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

    let is_dark = theme_mode.resolve_is_dark(ctx);
    let palette = Palette::new(is_dark, accent_preset);

    style.visuals = Visuals {
        dark_mode: is_dark,
        override_text_color: Some(palette.text),
        window_fill: palette.elevated,
        panel_fill: palette.panel,
        faint_bg_color: palette.panel,
        extreme_bg_color: palette.bg,
        code_bg_color: palette.widget_bg,
        window_corner_radius: rounding,
        menu_corner_radius: rounding,
        warn_fg_color: palette.accent_orange,
        error_fg_color: palette.accent_red,
        hyperlink_color: palette.accent,
        selection: Selection {
            bg_fill: palette.accent_dim,
            stroke: Stroke::new(1., palette.accent),
        },
        widgets: Widgets {
            noninteractive: WidgetVisuals {
                bg_fill: palette.panel,
                weak_bg_fill: palette.widget_bg,
                bg_stroke: Stroke::new(1., palette.border),
                fg_stroke: Stroke::new(1., palette.text_dim),
                expansion: 0.,
                corner_radius: rounding,
            },
            inactive: WidgetVisuals {
                bg_fill: palette.widget_bg,
                weak_bg_fill: palette.panel,
                bg_stroke: Stroke::new(1., palette.border),
                fg_stroke: Stroke::new(1., palette.text),
                expansion: 0.,
                corner_radius: rounding,
            },
            hovered: WidgetVisuals {
                bg_fill: palette.hover,
                weak_bg_fill: palette.panel,
                bg_stroke: Stroke::new(1., palette.border_light),
                fg_stroke: Stroke::new(1.2, palette.text),
                expansion: 0.5,
                corner_radius: rounding,
            },
            active: WidgetVisuals {
                bg_fill: palette.active,
                weak_bg_fill: palette.panel,
                bg_stroke: Stroke::new(1., palette.accent),
                fg_stroke: Stroke::new(1.5, palette.text),
                expansion: 0.5,
                corner_radius: rounding,
            },
            open: WidgetVisuals {
                bg_fill: palette.elevated,
                weak_bg_fill: palette.panel,
                bg_stroke: Stroke::new(1., palette.accent),
                fg_stroke: Stroke::new(1.2, palette.text),
                expansion: 0.,
                corner_radius: rounding,
            },
        },
        popup_shadow: egui::epaint::Shadow {
            offset: [0, 4],
            blur: 16,
            spread: 0,
            color: if is_dark { Color32::from_black_alpha(100) } else { Color32::from_black_alpha(35) },
        },
        window_shadow: egui::epaint::Shadow {
            offset: [0, 6],
            blur: 24,
            spread: 0,
            color: if is_dark { Color32::from_black_alpha(130) } else { Color32::from_black_alpha(45) },
        },
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_mode_and_palette_tokens() {
        let p_dark = Palette::new(true, AccentPreset::Indigo);
        assert!(p_dark.is_dark);
        assert_eq!(p_dark.accent, Color32::from_rgb(99, 102, 241));

        let p_light = Palette::new(false, AccentPreset::Emerald);
        assert!(!p_light.is_dark);
        assert_eq!(p_light.accent, Color32::from_rgb(22, 163, 74));
        assert_eq!(ThemeMode::Light.label(), "Light (Minimalist)");
        assert_eq!(AccentPreset::Sapphire.label(), "Sapphire Blue");
    }
}
