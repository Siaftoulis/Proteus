//! Stateless reusable UI controls and primitives for the Proteus Inspector.

use eframe::egui::{self, Color32, RichText, Vec2};
use crate::theme;

/// Renders a small information badge with hover tooltip.
pub fn info_button(ui: &mut egui::Ui, tooltip: &str) {
    let resp = ui.add(
        egui::Button::new(RichText::new("ℹ").size(10.).color(Color32::from_rgb(0, 180, 255)))
            .fill(Color32::from_rgb(35, 45, 55))
            .min_size(Vec2::new(16., 16.)),
    );
    resp.on_hover_text(tooltip);
}

/// Renders an RGBA color picker row.
pub fn color_picker_row(ui: &mut egui::Ui, label: &str, rgba: &mut [f32; 4]) -> bool {
    use egui::widgets::color_picker::{color_edit_button_rgba, Alpha};
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(label);
        let [r, g, b, a] = *rgba;
        let mut c = egui::Rgba::from_rgba_unmultiplied(r, g, b, a);
        changed |= color_edit_button_rgba(ui, &mut c, Alpha::OnlyBlend).changed();
        *rgba = [c.r(), c.g(), c.b(), c.a()];
    });
    changed
}

/// Renders curated luxury color swatches.
pub fn luxury_color_swatches<F>(ui: &mut egui::Ui, mut on_select: F)
where
    F: FnMut([f32; 4]),
{
    let swatches = [
        ("Obsidian", [0.05, 0.06, 0.08, 1.0]),
        ("Dark Card", [0.10, 0.11, 0.14, 1.0]),
        ("Elevated", [0.15, 0.16, 0.20, 1.0]),
        ("Border Gray", [0.22, 0.25, 0.32, 1.0]),
        ("Indigo", [0.39, 0.40, 0.95, 1.0]),
        ("Cyan", [0.06, 0.73, 0.85, 1.0]),
        ("Emerald", [0.06, 0.72, 0.50, 1.0]),
        ("Amber", [0.96, 0.62, 0.04, 1.0]),
        ("Rose", [0.96, 0.25, 0.37, 1.0]),
        ("White", [1.0, 1.0, 1.0, 1.0]),
    ];
    ui.horizontal_wrapped(|ui| {
        for (name, rgba) in swatches {
            let (rect, resp) = ui.allocate_exact_size(Vec2::splat(15.0), egui::Sense::click());
            let col = Color32::from_rgba_unmultiplied(
                (rgba[0] * 255.0) as u8,
                (rgba[1] * 255.0) as u8,
                (rgba[2] * 255.0) as u8,
                (rgba[3] * 255.0) as u8,
            );
            ui.painter().rect_filled(rect, egui::CornerRadius::same(3), col);
            ui.painter().rect_stroke(
                rect,
                egui::CornerRadius::same(3),
                egui::Stroke::new(1.0, theme::BORDER),
                egui::StrokeKind::Outside,
            );
            if resp.on_hover_text(name).clicked() {
                on_select(rgba);
            }
        }
    });
}
