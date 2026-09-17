//! Premium Dark Design System for Proteus Client.
//! Curated HSL colors, crisp 1px borders, elevated surfaces, and status palettes.

use egui::{Color32, CornerRadius, Margin, Stroke, Visuals};

pub const BG_BASE: Color32 = Color32::from_rgb(13, 15, 18);
pub const BG_PANEL: Color32 = Color32::from_rgb(18, 21, 27);
pub const BG_CARD: Color32 = Color32::from_rgb(24, 27, 34);
pub const BG_HOVER: Color32 = Color32::from_rgb(32, 37, 46);
pub const BG_ACTIVE: Color32 = Color32::from_rgb(40, 46, 58);

pub const BORDER_SUBTLE: Color32 = Color32::from_rgb(37, 41, 50);
pub const BORDER_FOCUS: Color32 = Color32::from_rgb(99, 102, 241); // Indigo

pub const ACCENT_PRIMARY: Color32 = Color32::from_rgb(99, 102, 241); // Indigo
pub const ACCENT_CYAN: Color32 = Color32::from_rgb(56, 189, 248);
pub const ACCENT_GOLD: Color32 = Color32::from_rgb(251, 191, 36);

pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(243, 244, 246);
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(156, 163, 175);
pub const TEXT_MUTED: Color32 = Color32::from_rgb(107, 114, 128);

// ── Status Colors ──
pub const STATUS_RECEIVED: Color32 = Color32::from_rgb(56, 189, 248);   // Sky Blue
pub const STATUS_IN_PROGRESS: Color32 = Color32::from_rgb(251, 146, 60); // Orange
pub const STATUS_WAITING_PARTS: Color32 = Color32::from_rgb(192, 132, 252); // Purple
pub const STATUS_READY: Color32 = Color32::from_rgb(52, 211, 153);      // Emerald Mint
pub const STATUS_DELIVERED: Color32 = Color32::from_rgb(148, 163, 184); // Slate
pub const STATUS_CANCELLED: Color32 = Color32::from_rgb(244, 63, 94);   // Rose

pub fn status_color(status: crm_core::tickets::TicketStatus) -> Color32 {
    match status {
        crm_core::tickets::TicketStatus::Received => STATUS_RECEIVED,
        crm_core::tickets::TicketStatus::InProgress => STATUS_IN_PROGRESS,
        crm_core::tickets::TicketStatus::WaitingParts => STATUS_WAITING_PARTS,
        crm_core::tickets::TicketStatus::Ready => STATUS_READY,
        crm_core::tickets::TicketStatus::Delivered => STATUS_DELIVERED,
        crm_core::tickets::TicketStatus::Cancelled => STATUS_CANCELLED,
    }
}

/// Applies the dark minimalist design system to egui.
pub fn apply_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    let mut visuals = Visuals::dark();

    let rounding = CornerRadius::same(6);

    visuals.override_text_color = Some(TEXT_PRIMARY);
    visuals.panel_fill = BG_PANEL;
    visuals.window_fill = BG_CARD;
    visuals.faint_bg_color = BG_BASE;
    visuals.extreme_bg_color = BG_BASE;

    // Button styling
    visuals.widgets.noninteractive.bg_fill = BG_CARD;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER_SUBTLE);
    visuals.widgets.noninteractive.corner_radius = rounding;

    visuals.widgets.inactive.bg_fill = BG_CARD;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER_SUBTLE);
    visuals.widgets.inactive.corner_radius = rounding;

    visuals.widgets.hovered.bg_fill = BG_HOVER;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, BORDER_FOCUS);
    visuals.widgets.hovered.corner_radius = rounding;

    visuals.widgets.active.bg_fill = BG_ACTIVE;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT_PRIMARY);
    visuals.widgets.active.corner_radius = rounding;

    visuals.selection.bg_fill = ACCENT_PRIMARY.linear_multiply(0.3);
    visuals.selection.stroke = Stroke::new(1.0, ACCENT_PRIMARY);

    style.visuals = visuals;
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.window_margin = Margin::same(12);

    ctx.set_style(style);
}
