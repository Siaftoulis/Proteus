use eframe::egui::{self, Color32, Stroke};
use serde::{Deserialize, Serialize};

// ── King Proteus Sovereign Brand Identity ──
pub const PETROL_KING_TEAL: Color32 = Color32::from_rgb(38, 88, 99);    // #265863
pub const PALE_CREAM_SHELL: Color32 = Color32::from_rgb(245, 238, 212); // #F5EED4
pub const WARM_OCHRE_GOLD: Color32  = Color32::from_rgb(197, 168, 128); // #C5A880
pub const SKY_CYAN: Color32         = Color32::from_rgb(56, 189, 248);  // #38BDF8

// ── Obsidian Dark Foundations (Compatibility & Extended Depth) ──
pub const OBSIDIAN_BASE: Color32    = Color32::from_rgb(12, 14, 20);     // #0C0E14
pub const OBSIDIAN_SURFACE: Color32 = Color32::from_rgb(21, 24, 33);     // #151821
pub const OBSIDIAN_BORDER: Color32  = Color32::from_rgb(38, 44, 61);     // #262C3D

// ── Base Palette: Nordic Slate (Authoritative Tokens) ──
pub const DEEP_SLATE_NAVY: Color32  = Color32::from_rgb(39, 55, 77);   // #27374D
pub const STEEL_SLATE: Color32      = Color32::from_rgb(82, 109, 130);  // #526D82
pub const FROST_GRAY: Color32       = Color32::from_rgb(157, 178, 191); // #9DB2BF
pub const ICY_MIST: Color32         = Color32::from_rgb(221, 230, 237); // #DDE6ED
pub const CANVAS_DARK: Color32      = Color32::from_rgb(27, 36, 48);   // #1B2430
pub const CANVAS_LIGHT: Color32     = Color32::from_rgb(244, 247, 249); // #F4F7F9
pub const SURFACE_WHITE: Color32    = Color32::from_rgb(255, 255, 255); // #FFFFFF
pub const SURFACE_SEC_DARK: Color32 = Color32::from_rgb(30, 42, 58);  // #1E2A3A

// ── Dark Mode Semantic Tokens (Default) ──
pub const DARK_BG: Color32             = CANVAS_DARK;          // #1B2430
pub const DARK_SURFACE: Color32        = DEEP_SLATE_NAVY;      // #27374D
pub const DARK_SURFACE_SEC: Color32    = SURFACE_SEC_DARK;     // #1E2A3A
pub const DARK_BORDER_SUBTLE: Color32  = Color32::from_rgba_premultiplied(33, 44, 52, 102); // rgba(82, 109, 130, 0.4)
pub const DARK_BORDER_STRONG: Color32  = STEEL_SLATE;          // #526D82
pub const DARK_TEXT_PRIMARY: Color32   = ICY_MIST;             // #DDE6ED
pub const DARK_TEXT_MUTED: Color32     = FROST_GRAY;           // #9DB2BF
pub const DARK_CTA_FILL: Color32       = ICY_MIST;             // #DDE6ED
pub const DARK_CTA_TEXT: Color32       = DEEP_SLATE_NAVY;      // #27374D

// ── Light Mode Semantic Tokens ──
pub const LIGHT_BG: Color32            = CANVAS_LIGHT;         // #F4F7F9
pub const LIGHT_SURFACE: Color32       = SURFACE_WHITE;        // #FFFFFF
pub const LIGHT_SURFACE_SEC: Color32   = ICY_MIST;             // #DDE6ED
pub const LIGHT_BORDER_SUBTLE: Color32 = Color32::from_rgba_premultiplied(71, 80, 86, 115); // rgba(157, 178, 191, 0.45)
pub const LIGHT_BORDER_STRONG: Color32 = FROST_GRAY;           // #9DB2BF
pub const LIGHT_TEXT_PRIMARY: Color32  = DEEP_SLATE_NAVY;      // #27374D
pub const LIGHT_TEXT_MUTED: Color32    = STEEL_SLATE;          // #526D82
pub const LIGHT_CTA_FILL: Color32      = DEEP_SLATE_NAVY;      // #27374D
pub const LIGHT_CTA_TEXT: Color32      = ICY_MIST;             // #DDE6ED

// ── Fallback Constants (Default Dark Mode) ──
pub const BG: Color32           = DARK_BG;
pub const PANEL: Color32        = DARK_SURFACE_SEC;
pub const ELEVATED: Color32     = DARK_SURFACE;
pub const WIDGET_BG: Color32    = DARK_SURFACE_SEC;
pub const HOVER: Color32        = Color32::from_rgb(47, 65, 89);
pub const ACTIVE: Color32       = Color32::from_rgb(56, 77, 106);
pub const BORDER: Color32       = DARK_BORDER_SUBTLE;
pub const BORDER_LIGHT: Color32 = DARK_BORDER_STRONG;
pub const FOCUS: Color32        = STEEL_SLATE;
pub const TEXT: Color32         = DARK_TEXT_PRIMARY;
pub const TEXT_DIM: Color32     = DARK_TEXT_MUTED;
pub const TEXT_MUTED: Color32   = STEEL_SLATE;
pub const ACCENT: Color32       = ICY_MIST;
pub const ACCENT_DIM: Color32   = Color32::from_rgba_premultiplied(26, 34, 41, 80);
pub const ACCENT_ORANGE: Color32 = Color32::from_rgb(214, 158, 46);
pub const ACCENT_GREEN: Color32  = Color32::from_rgb(56, 161, 105);
pub const ACCENT_RED: Color32    = Color32::from_rgb(229, 62, 62);
pub const ACCENT_PURPLE: Color32 = STEEL_SLATE;
pub const ACCENT_CYAN: Color32   = FROST_GRAY;
pub const SELECTED: Color32      = STEEL_SLATE;
pub const HEADER_TRIGGER: Color32   = Color32::from_rgb(229, 62, 62);
pub const HEADER_ACTION: Color32    = STEEL_SLATE;
pub const HEADER_CONDITION: Color32 = Color32::from_rgb(214, 158, 46);
pub const HEADER_GATE: Color32      = Color32::from_rgb(56, 161, 105);

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
            Self::Dark => "Dark (Nordic Slate)",
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

/// Accent color presets adhering to Nordic Slate & King Proteus Brand principles.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub enum AccentPreset {
    #[default]
    NordicSlate,
    KingTeal,
    SkyCyan,
    OchreGold,
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
            Self::NordicSlate => "Nordic Slate",
            Self::KingTeal => "King Proteus Teal",
            Self::SkyCyan => "Sky Cyan Pulse",
            Self::OchreGold => "Warm Ochre Gold",
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
            Self::NordicSlate => STEEL_SLATE,
            Self::KingTeal => PETROL_KING_TEAL,
            Self::SkyCyan => SKY_CYAN,
            Self::OchreGold => WARM_OCHRE_GOLD,
            Self::Indigo => Color32::from_rgb(99, 102, 241),
            Self::Sapphire => Color32::from_rgb(37, 99, 235),
            Self::Emerald => Color32::from_rgb(56, 161, 105),
            Self::Amber => Color32::from_rgb(214, 158, 46),
            Self::Rose => Color32::from_rgb(229, 62, 62),
            Self::Slate => STEEL_SLATE,
        }
    }

    pub fn dim_color(&self, is_dark: bool) -> Color32 {
        let c = self.color();
        if is_dark {
            Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 60)
        } else {
            Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), 40)
        }
    }
}

/// Complete resolved UI color palette tokens.
#[derive(Clone, Debug)]
pub struct Palette {
    pub is_dark: bool,
    pub bg: Color32,
    pub surface: Color32,
    pub surface_secondary: Color32,
    pub panel: Color32,
    pub elevated: Color32,
    pub widget_bg: Color32,
    pub hover: Color32,
    pub active: Color32,
    pub border: Color32,
    pub border_subtle: Color32,
    pub border_strong: Color32,
    pub border_light: Color32,
    pub text: Color32,
    pub text_primary: Color32,
    pub text_dim: Color32,
    pub text_muted: Color32,
    pub cta_primary_fill: Color32,
    pub cta_primary_text: Color32,
    pub cta_secondary_fill: Color32,
    pub cta_secondary_text: Color32,
    pub cta_secondary_border: Color32,
    pub accent: Color32,
    pub accent_dim: Color32,
    pub accent_orange: Color32,
    pub accent_green: Color32,
    pub accent_red: Color32,
    pub accent_purple: Color32,
    pub accent_cyan: Color32,
    pub brand_teal: Color32,
    pub brand_shell: Color32,
    pub brand_gold: Color32,
    pub brand_cyan: Color32,
}

impl Palette {
    pub fn new(is_dark: bool, accent_preset: AccentPreset) -> Self {
        let preset_color = accent_preset.color();
        let preset_dim = accent_preset.dim_color(is_dark);

        if is_dark {
            let accent = if accent_preset == AccentPreset::NordicSlate {
                DARK_TEXT_PRIMARY
            } else {
                preset_color
            };
            Self {
                is_dark: true,
                bg: DARK_BG,
                surface: DARK_SURFACE,
                surface_secondary: DARK_SURFACE_SEC,
                panel: DARK_SURFACE_SEC,
                elevated: DARK_SURFACE,
                widget_bg: DARK_SURFACE_SEC,
                hover: Color32::from_rgb(47, 65, 89),
                active: Color32::from_rgb(56, 77, 106),
                border: DARK_BORDER_SUBTLE,
                border_subtle: DARK_BORDER_SUBTLE,
                border_strong: DARK_BORDER_STRONG,
                border_light: DARK_BORDER_STRONG,
                text: DARK_TEXT_PRIMARY,
                text_primary: DARK_TEXT_PRIMARY,
                text_dim: DARK_TEXT_MUTED,
                text_muted: STEEL_SLATE,
                cta_primary_fill: DARK_CTA_FILL,
                cta_primary_text: DARK_CTA_TEXT,
                cta_secondary_fill: DARK_SURFACE,
                cta_secondary_text: DARK_TEXT_PRIMARY,
                cta_secondary_border: DARK_BORDER_STRONG,
                accent,
                accent_dim: preset_dim,
                accent_orange: Color32::from_rgb(214, 158, 46),
                accent_green: Color32::from_rgb(56, 161, 105),
                accent_red: Color32::from_rgb(229, 62, 62),
                accent_purple: STEEL_SLATE,
                accent_cyan: FROST_GRAY,
                brand_teal: PETROL_KING_TEAL,
                brand_shell: PALE_CREAM_SHELL,
                brand_gold: WARM_OCHRE_GOLD,
                brand_cyan: SKY_CYAN,
            }
        } else {
            let accent = if accent_preset == AccentPreset::NordicSlate {
                LIGHT_TEXT_PRIMARY
            } else {
                preset_color
            };
            Self {
                is_dark: false,
                bg: LIGHT_BG,
                surface: LIGHT_SURFACE,
                surface_secondary: LIGHT_SURFACE_SEC,
                panel: LIGHT_SURFACE,
                elevated: LIGHT_SURFACE,
                widget_bg: LIGHT_SURFACE_SEC,
                hover: Color32::from_rgb(228, 233, 238),
                active: Color32::from_rgb(215, 222, 230),
                border: LIGHT_BORDER_SUBTLE,
                border_subtle: LIGHT_BORDER_SUBTLE,
                border_strong: LIGHT_BORDER_STRONG,
                border_light: LIGHT_BORDER_STRONG,
                text: LIGHT_TEXT_PRIMARY,
                text_primary: LIGHT_TEXT_PRIMARY,
                text_dim: LIGHT_TEXT_MUTED,
                text_muted: STEEL_SLATE,
                cta_primary_fill: LIGHT_CTA_FILL,
                cta_primary_text: LIGHT_CTA_TEXT,
                cta_secondary_fill: LIGHT_SURFACE_SEC,
                cta_secondary_text: LIGHT_TEXT_PRIMARY,
                cta_secondary_border: LIGHT_BORDER_STRONG,
                accent,
                accent_dim: preset_dim,
                accent_orange: Color32::from_rgb(214, 158, 46),
                accent_green: Color32::from_rgb(56, 161, 105),
                accent_red: Color32::from_rgb(229, 62, 62),
                accent_purple: STEEL_SLATE,
                accent_cyan: FROST_GRAY,
                brand_teal: PETROL_KING_TEAL,
                brand_shell: PALE_CREAM_SHELL,
                brand_gold: WARM_OCHRE_GOLD,
                brand_cyan: SKY_CYAN,
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
    style.spacing.button_padding = egui::vec2(10., 4.);
    style.spacing.indent = 14.;
    style.spacing.icon_width = 16.;
    style.spacing.icon_width_inner = 12.;
    style.spacing.interact_size = egui::vec2(32., 22.);
    style.spacing.slider_width = 130.;

    let is_dark = theme_mode.resolve_is_dark(ctx);
    let palette = Palette::new(is_dark, accent_preset);

    style.visuals = Visuals {
        dark_mode: is_dark,
        override_text_color: Some(palette.text_primary),
        window_fill: palette.elevated,
        panel_fill: palette.panel,
        faint_bg_color: palette.panel,
        extreme_bg_color: palette.bg,
        code_bg_color: palette.widget_bg,
        window_corner_radius: rounding,
        menu_corner_radius: rounding,
        warn_fg_color: palette.accent_orange,
        error_fg_color: palette.accent_red,
        hyperlink_color: palette.border_strong,
        selection: Selection {
            bg_fill: palette.accent_dim,
            stroke: Stroke::new(1., palette.border_strong),
        },
        widgets: Widgets {
            noninteractive: WidgetVisuals {
                bg_fill: palette.panel,
                weak_bg_fill: palette.widget_bg,
                bg_stroke: Stroke::new(1., palette.border_subtle),
                fg_stroke: Stroke::new(1., palette.text_dim),
                expansion: 0.,
                corner_radius: rounding,
            },
            inactive: WidgetVisuals {
                bg_fill: palette.widget_bg,
                weak_bg_fill: palette.panel,
                bg_stroke: Stroke::new(1., palette.border_subtle),
                fg_stroke: Stroke::new(1., palette.text_primary),
                expansion: 0.,
                corner_radius: rounding,
            },
            hovered: WidgetVisuals {
                bg_fill: palette.hover,
                weak_bg_fill: palette.panel,
                bg_stroke: Stroke::new(1., palette.border_strong),
                fg_stroke: Stroke::new(1.1, palette.text_primary),
                expansion: 0.,
                corner_radius: rounding,
            },
            active: WidgetVisuals {
                bg_fill: palette.active,
                weak_bg_fill: palette.panel,
                bg_stroke: Stroke::new(1., palette.border_strong),
                fg_stroke: Stroke::new(1.2, palette.text_primary),
                expansion: 0.,
                corner_radius: rounding,
            },
            open: WidgetVisuals {
                bg_fill: palette.elevated,
                weak_bg_fill: palette.panel,
                bg_stroke: Stroke::new(1., palette.border_strong),
                fg_stroke: Stroke::new(1.1, palette.text_primary),
                expansion: 0.,
                corner_radius: rounding,
            },
        },
        popup_shadow: egui::epaint::Shadow {
            offset: [0, 2],
            blur: 4,
            spread: 0,
            color: if is_dark { Color32::from_black_alpha(60) } else { Color32::from_black_alpha(20) },
        },
        window_shadow: egui::epaint::Shadow {
            offset: [0, 4],
            blur: 8,
            spread: 0,
            color: if is_dark { Color32::from_black_alpha(80) } else { Color32::from_black_alpha(30) },
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
        let p_dark = Palette::new(true, AccentPreset::NordicSlate);
        assert!(p_dark.is_dark);
        assert_eq!(p_dark.bg, DARK_BG);
        assert_eq!(p_dark.surface, DARK_SURFACE);
        assert_eq!(p_dark.text_primary, DARK_TEXT_PRIMARY);

        let p_light = Palette::new(false, AccentPreset::NordicSlate);
        assert!(!p_light.is_dark);
        assert_eq!(p_light.bg, LIGHT_BG);
        assert_eq!(p_light.surface, LIGHT_SURFACE);
        assert_eq!(p_light.text_primary, LIGHT_TEXT_PRIMARY);
        assert_eq!(p_dark.brand_teal, PETROL_KING_TEAL);
        assert_eq!(p_dark.brand_shell, PALE_CREAM_SHELL);
        assert_eq!(ThemeMode::Light.label(), "Light (Minimalist)");
        assert_eq!(AccentPreset::NordicSlate.label(), "Nordic Slate");
        assert_eq!(AccentPreset::KingTeal.label(), "King Proteus Teal");
        assert_eq!(AccentPreset::KingTeal.color(), PETROL_KING_TEAL);
    }
}


