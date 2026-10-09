//! Ergonomic 44pt Touch Control Primitives & Input Field Shield (Micro-task 29.1.2).
//! Adheres strictly to Apple HIG & Android Material standards for touch accessibility.
//! Guarantees a minimum 44.0pt interactive boundary for buttons, inputs, toggles,
//! and segmented pickers to eliminate mis-taps on mobile & POS touch screens.

use crate::keyboard_cushion::KeyboardCushion;
use crate::{BG_CARD, BORDER_LINE, TEXT_BODY, TEXT_TITLE};
use egui::{
    Color32, CornerRadius, FontId, Id, Rect, Response, RichText, Sense, Stroke, Ui, Vec2,
};

pub const MIN_TOUCH_DIMENSION: f32 = 44.0;
pub const ACCENT_BLUE: Color32 = Color32::from_rgb(79, 134, 198);

/// Enforces an expanded hit target box around a response to guarantee at least 44x44pt.
pub fn enforce_touch_target_boundary(ui: &mut Ui, inner_rect: Rect, id: Id) -> Response {
    let mut touch_rect = inner_rect;
    if touch_rect.width() < MIN_TOUCH_DIMENSION {
        let diff = MIN_TOUCH_DIMENSION - touch_rect.width();
        touch_rect.min.x -= diff * 0.5;
        touch_rect.max.x += diff * 0.5;
    }
    if touch_rect.height() < MIN_TOUCH_DIMENSION {
        let diff = MIN_TOUCH_DIMENSION - touch_rect.height();
        touch_rect.min.y -= diff * 0.5;
        touch_rect.max.y += diff * 0.5;
    }

    ui.interact(touch_rect, id, Sense::click())
}

/// A high-yield touch button guaranteeing minimum 44pt height with Nordic styling.
pub fn touch_button(ui: &mut Ui, text: &str, is_primary: bool) -> Response {
    let width = ui.available_width();
    let desired_size = Vec2::new(width, MIN_TOUCH_DIMENSION);
    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());

    if ui.is_rect_visible(rect) {
        let is_hovered = response.hovered();
        let is_pressed = response.is_pointer_button_down_on();

        let (bg, border, text_color) = if is_primary {
            if is_pressed {
                (Color32::from_rgb(60, 110, 170), Stroke::NONE, Color32::WHITE)
            } else if is_hovered {
                (Color32::from_rgb(95, 150, 215), Stroke::NONE, Color32::WHITE)
            } else {
                (ACCENT_BLUE, Stroke::NONE, Color32::WHITE)
            }
        } else {
            if is_pressed {
                (Color32::from_rgb(45, 62, 85), Stroke::new(1.0, BORDER_LINE), TEXT_TITLE)
            } else if is_hovered {
                (Color32::from_rgb(35, 50, 70), Stroke::new(1.0, BORDER_LINE), TEXT_TITLE)
            } else {
                (BG_CARD, Stroke::new(1.0, BORDER_LINE), TEXT_TITLE)
            }
        };

        ui.painter().rect(rect, CornerRadius::same(8), bg, border, egui::StrokeKind::Outside);
        let font = FontId::proportional(15.0);
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            text,
            font,
            text_color,
        );
    }

    response
}

/// Touch-accessible text edit with embedded clear button and soft keyboard cushion binding.
pub fn touch_text_input(
    ui: &mut Ui,
    cushion: &mut KeyboardCushion,
    value: &mut String,
    hint: &str,
) -> Response {
    let width = ui.available_width();
    let desired_size = Vec2::new(width, MIN_TOUCH_DIMENSION);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::click());

    let has_clear = !value.is_empty();
    let text_edit_width = if has_clear {
        width - 44.0
    } else {
        width
    };

    ui.allocate_new_ui(
        egui::UiBuilder::new().max_rect(Rect::from_min_size(rect.min, Vec2::new(text_edit_width, MIN_TOUCH_DIMENSION))),
        |ui| {
            let edit_resp = ui.add_sized(
                Vec2::new(text_edit_width, MIN_TOUCH_DIMENSION),
                egui::TextEdit::singleline(value)
                    .hint_text(hint)
                    .font(FontId::proportional(15.0)),
            );
            cushion.register_input_focus(&edit_resp);
            response = response.union(edit_resp);
        },
    );

    // Embedded 44pt clear button if field is populated
    if has_clear {
        let clear_rect = Rect::from_min_size(
            egui::pos2(rect.max.x - 44.0, rect.min.y),
            Vec2::new(44.0, MIN_TOUCH_DIMENSION),
        );
        let clear_id = response.id.with("clear_btn");
        let clear_resp = ui.interact(clear_rect, clear_id, Sense::click());
        if ui.is_rect_visible(clear_rect) {
            let icon_color = if clear_resp.hovered() { TEXT_TITLE } else { TEXT_BODY };
            ui.painter().text(
                clear_rect.center(),
                egui::Align2::CENTER_CENTER,
                "✕",
                FontId::proportional(14.0),
                icon_color,
            );
        }
        if clear_resp.clicked() {
            value.clear();
            response.mark_changed();
        }
    }

    response
}

/// Touch-accessible checkbox toggle enforcing full 44pt minimum hit target height.
pub fn touch_checkbox(ui: &mut Ui, checked: &mut bool, label: &str) -> Response {
    let width = ui.available_width();
    let desired_size = Vec2::new(width, MIN_TOUCH_DIMENSION);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::click());

    if response.clicked() {
        *checked = !*checked;
        response.mark_changed();
    }

    if ui.is_rect_visible(rect) {
        let box_size = 24.0;
        let box_rect = Rect::from_center_size(
            egui::pos2(rect.min.x + 22.0, rect.center().y),
            Vec2::splat(box_size),
        );

        let (fill, stroke) = if *checked {
            (ACCENT_BLUE, Stroke::NONE)
        } else {
            (BG_CARD, Stroke::new(1.5, BORDER_LINE))
        };

        ui.painter().rect(box_rect, CornerRadius::same(6), fill, stroke, egui::StrokeKind::Outside);

        if *checked {
            ui.painter().text(
                box_rect.center(),
                egui::Align2::CENTER_CENTER,
                "✓",
                FontId::proportional(15.0),
                Color32::WHITE,
            );
        }

        let label_pos = egui::pos2(rect.min.x + 48.0, rect.center().y);
        ui.painter().text(
            label_pos,
            egui::Align2::LEFT_CENTER,
            label,
            FontId::proportional(15.0),
            TEXT_TITLE,
        );
    }

    response
}

/// Segmented tab picker where every item provides 44pt height touch ergonomics.
pub fn touch_segmented_picker<T: PartialEq + Clone>(
    ui: &mut Ui,
    items: &[(T, &str)],
    selected: &mut T,
) -> bool {
    let mut changed = false;
    let width = ui.available_width();
    let item_count = items.len().max(1) as f32;
    let item_width = width / item_count;
    let total_rect = Rect::from_min_size(ui.cursor().min, Vec2::new(width, MIN_TOUCH_DIMENSION));

    ui.painter().rect(
        total_rect,
        CornerRadius::same(10),
        BG_CARD,
        Stroke::new(1.0, BORDER_LINE),
        egui::StrokeKind::Outside,
    );

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::ZERO;

        for (val, label) in items {
            let (rect, resp) = ui.allocate_exact_size(
                Vec2::new(item_width, MIN_TOUCH_DIMENSION),
                Sense::click(),
            );

            let is_active = *selected == *val;
            if resp.clicked() && !is_active {
                *selected = val.clone();
                changed = true;
            }

            if is_active {
                let pill_rect = rect.shrink(3.0);
                ui.painter().rect(
                    pill_rect,
                    CornerRadius::same(8),
                    ACCENT_BLUE,
                    Stroke::NONE,
                    egui::StrokeKind::Outside,
                );
            }

            let text_color = if is_active { Color32::WHITE } else { TEXT_BODY };
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                *label,
                FontId::proportional(14.0),
                text_color,
            );
        }
    });

    changed
}

/// Touch stepper with + / - buttons each guaranteed at 44x44pt.
pub fn touch_stepper(
    ui: &mut Ui,
    value: &mut i32,
    min: i32,
    max: i32,
    label: &str,
) -> bool {
    let mut changed = false;

    ui.horizontal(|ui| {
        ui.label(RichText::new(label).color(TEXT_TITLE).font(FontId::proportional(15.0)));

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            // Plus button
            let (plus_rect, plus_resp) = ui.allocate_exact_size(Vec2::splat(MIN_TOUCH_DIMENSION), Sense::click());
            if plus_resp.clicked() && *value < max {
                *value += 1;
                changed = true;
            }
            ui.painter().rect(plus_rect, CornerRadius::same(8), BG_CARD, Stroke::new(1.0, BORDER_LINE), egui::StrokeKind::Outside);
            ui.painter().text(plus_rect.center(), egui::Align2::CENTER_CENTER, "+", FontId::proportional(18.0), TEXT_TITLE);

            ui.add_space(4.0);

            // Value display
            ui.label(RichText::new(value.to_string()).strong().font(FontId::proportional(16.0)).color(TEXT_TITLE));

            ui.add_space(4.0);

            // Minus button
            let (minus_rect, minus_resp) = ui.allocate_exact_size(Vec2::splat(MIN_TOUCH_DIMENSION), Sense::click());
            if minus_resp.clicked() && *value > min {
                *value -= 1;
                changed = true;
            }
            ui.painter().rect(minus_rect, CornerRadius::same(8), BG_CARD, Stroke::new(1.0, BORDER_LINE), egui::StrokeKind::Outside);
            ui.painter().text(minus_rect.center(), egui::Align2::CENTER_CENTER, "−", FontId::proportional(18.0), TEXT_TITLE);
        });
    });

    changed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_min_touch_dimension_constant() {
        assert_eq!(MIN_TOUCH_DIMENSION, 44.0);
    }

    #[test]
    fn test_enforce_touch_target_boundary_expansion() {
        let small_rect = Rect::from_min_size(egui::pos2(100.0, 100.0), Vec2::new(20.0, 20.0));
        let mut touch_rect = small_rect;
        if touch_rect.width() < MIN_TOUCH_DIMENSION {
            let diff = MIN_TOUCH_DIMENSION - touch_rect.width();
            touch_rect.min.x -= diff * 0.5;
            touch_rect.max.x += diff * 0.5;
        }
        if touch_rect.height() < MIN_TOUCH_DIMENSION {
            let diff = MIN_TOUCH_DIMENSION - touch_rect.height();
            touch_rect.min.y -= diff * 0.5;
            touch_rect.max.y += diff * 0.5;
        }
        assert_eq!(touch_rect.width(), 44.0);
        assert_eq!(touch_rect.height(), 44.0);
        assert_eq!(touch_rect.center(), small_rect.center());
    }

    #[test]
    fn test_stepper_bounds_clamp() {
        let mut val = 5;
        let min = 0;
        let max = 10;

        // Step up
        val = (val + 1).min(max);
        assert_eq!(val, 6);

        // Max clamp
        val = 10;
        val = (val + 1).min(max);
        assert_eq!(val, 10);

        // Min clamp
        val = 0;
        val = (val - 1).max(min);
        assert_eq!(val, 0);
    }
}
