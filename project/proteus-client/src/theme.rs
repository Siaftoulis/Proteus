#![allow(dead_code)]

//! Nordic Slate Design System for Proteus Client.
//! Follows the PDS (Proteus Design System) specification:
//! Deep Slate Navy (#27374D), Steel Slate (#526D82), Frost Gray (#9DB2BF), Icy Mist (#DDE6ED).

use egui::{Color32, CornerRadius, Margin, Stroke, Visuals};

// ── Base Palette: Nordic Slate ──
pub const DEEP_SLATE_NAVY: Color32  = Color32::from_rgb(39, 55, 77);   // #27374D
pub const STEEL_SLATE: Color32      = Color32::from_rgb(82, 109, 130);  // #526D82
pub const FROST_GRAY: Color32       = Color32::from_rgb(157, 178, 191); // #9DB2BF
pub const ICY_MIST: Color32         = Color32::from_rgb(221, 230, 237); // #DDE6ED

// ── Dark Mode Semantic Tokens ──
pub const BG_BASE: Color32    = Color32::from_rgb(27, 36, 48);   // #1B2430 (Canvas)
pub const BG_PANEL: Color32   = Color32::from_rgb(30, 42, 58);   // #1E2A3A (Secondary Surface / Recessed)
pub const BG_CARD: Color32    = DEEP_SLATE_NAVY;                 // #27374D (Surface / Elevated)
pub const BG_HOVER: Color32   = Color32::from_rgb(47, 65, 89);
pub const BG_ACTIVE: Color32  = Color32::from_rgb(56, 77, 106);

pub const BORDER_SUBTLE: Color32 = Color32::from_rgba_premultiplied(33, 44, 52, 102); // rgba(82, 109, 130, 0.4)
pub const BORDER_FOCUS: Color32  = STEEL_SLATE; // #526D82
pub const BORDER_STRONG: Color32 = STEEL_SLATE; // #526D82

pub const ACCENT_PRIMARY: Color32 = ICY_MIST;    // #DDE6ED
pub const ACCENT_CYAN: Color32    = FROST_GRAY;  // #9DB2BF
pub const ACCENT_GOLD: Color32    = Color32::from_rgb(214, 158, 46);

pub const TEXT_PRIMARY: Color32   = ICY_MIST;    // #DDE6ED
pub const TEXT_SECONDARY: Color32 = FROST_GRAY;  // #9DB2BF
pub const TEXT_MUTED: Color32     = STEEL_SLATE; // #526D82

// ── Unified Component Tokens (Zero Random Colors) ──
pub const TAB_CONTAINER_BG: Color32 = Color32::from_rgb(22, 30, 42);
pub const TAB_CONTAINER_STROKE: Color32 = Color32::from_rgb(45, 60, 80);
pub const TAB_ACTIVE_FILL: Color32 = STEEL_SLATE;      // #526D82
pub const TAB_ACTIVE_TEXT: Color32 = ICY_MIST;         // #DDE6ED
pub const TAB_HOVER_TEXT: Color32 = Color32::WHITE;
pub const TAB_INACTIVE_TEXT: Color32 = FROST_GRAY;     // #9DB2BF

pub const BADGE_MOUNTED_BG: Color32 = BG_PANEL;        // #1E2A3A
pub const BADGE_MOUNTED_BORDER: Color32 = STEEL_SLATE; // #526D82
pub const BADGE_MOUNTED_TEXT: Color32 = ICY_MIST;      // #DDE6ED

pub const LOCAL_STATUS_BG: Color32 = BG_PANEL;         // #1E2A3A
pub const LOCAL_STATUS_BORDER: Color32 = STEEL_SLATE;  // #526D82
pub const LOCAL_STATUS_TEXT: Color32 = FROST_GRAY;     // #9DB2BF
pub const LOCAL_STATUS_DOT: Color32 = FROST_GRAY;      // #9DB2BF

pub const BTN_TOP_BG: Color32 = BG_CARD;               // #27374D
pub const BTN_TOP_BORDER: Color32 = BORDER_SUBTLE;
pub const BTN_TOP_TEXT: Color32 = ICY_MIST;            // #DDE6ED


// ── Status Colors (Nordic Slate Feedback) ──
pub const STATUS_RECEIVED: Color32      = FROST_GRAY;                      // Frost Gray
pub const STATUS_IN_PROGRESS: Color32   = Color32::from_rgb(214, 158, 46); // Amber
pub const STATUS_WAITING_PARTS: Color32 = STEEL_SLATE;                     // Steel Slate
pub const STATUS_READY: Color32         = Color32::from_rgb(56, 161, 105); // Emerald Green
pub const STATUS_DELIVERED: Color32     = FROST_GRAY;                      // Slate
pub const STATUS_CANCELLED: Color32     = Color32::from_rgb(229, 62, 62);  // Red

pub fn status_color(status: proteus_core::tickets::TicketStatus) -> Color32 {
    match status {
        proteus_core::tickets::TicketStatus::Received => STATUS_RECEIVED,
        proteus_core::tickets::TicketStatus::InProgress => STATUS_IN_PROGRESS,
        proteus_core::tickets::TicketStatus::WaitingParts => STATUS_WAITING_PARTS,
        proteus_core::tickets::TicketStatus::Ready => STATUS_READY,
        proteus_core::tickets::TicketStatus::Delivered => STATUS_DELIVERED,
        proteus_core::tickets::TicketStatus::Cancelled => STATUS_CANCELLED,
    }
}

/// Applies the Nordic Slate minimalist design system to egui.
pub fn apply_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    let mut visuals = Visuals::dark();

    let rounding = CornerRadius::same(5);

    visuals.override_text_color = Some(TEXT_PRIMARY);
    visuals.panel_fill = BG_PANEL;
    visuals.window_fill = BG_CARD;
    visuals.faint_bg_color = BG_BASE;
    visuals.extreme_bg_color = BG_BASE;

    // Button styling with crisp 1px borders
    visuals.widgets.noninteractive.bg_fill = BG_PANEL;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER_SUBTLE);
    visuals.widgets.noninteractive.corner_radius = rounding;

    visuals.widgets.inactive.bg_fill = BG_CARD;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER_SUBTLE);
    visuals.widgets.inactive.corner_radius = rounding;

    visuals.widgets.hovered.bg_fill = BG_HOVER;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, BORDER_STRONG);
    visuals.widgets.hovered.corner_radius = rounding;

    visuals.widgets.active.bg_fill = BG_ACTIVE;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, BORDER_STRONG);
    visuals.widgets.active.corner_radius = rounding;

    visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(82, 109, 130, 80);
    visuals.selection.stroke = Stroke::new(1.0, BORDER_FOCUS);

    style.visuals = visuals;
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(10.0, 4.0);
    style.spacing.window_margin = Margin::same(12);

    ctx.set_style(style);
}


