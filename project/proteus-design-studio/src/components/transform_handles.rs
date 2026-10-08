//! Lunacy-grade visual transform handles: 8 resize handles, rotation lollipop stem & knob, and live HUD badges.

use eframe::egui::{self, Color32, CornerRadius, CursorIcon, Pos2, Rect, Sense, Stroke, Vec2};
use crate::scene::{CanvasEvent, NodeUpdate, ResizeHandle};
use crate::theme;
use crate::Viewport2D;

pub const HANDLE_SIZE: f32 = 10.0;
pub const ROTATION_STEM_LENGTH: f32 = 24.0;
pub const ROTATION_KNOB_RADIUS: f32 = 4.0;

/// Calculates the normalized rotation angle in degrees (-180.0..=180.0) from center to cursor.
/// 0 degrees corresponds to 12 o'clock (pointing straight up).
pub fn calculate_rotation_angle(center: Pos2, mouse_pos: Pos2, shift_held: bool) -> f32 {
    let delta = mouse_pos - center;
    let raw_rad = delta.y.atan2(delta.x) + std::f32::consts::FRAC_PI_2;
    let mut deg = raw_rad.to_degrees();

    // Normalize to [-180.0, 180.0]
    while deg > 180.0 {
        deg -= 360.0;
    }
    while deg < -180.0 {
        deg += 360.0;
    }

    if shift_held {
        // Constrain to 15-degree steps for CAD/Figma-like precision
        (deg / 15.0).round() * 15.0
    } else {
        deg.round()
    }
}

fn cursor_for_handle(handle: ResizeHandle) -> CursorIcon {
    use ResizeHandle::*;
    match handle {
        TopLeft | BottomRight => CursorIcon::ResizeNwSe,
        TopRight | BottomLeft => CursorIcon::ResizeNeSw,
        Top | Bottom => CursorIcon::ResizeVertical,
        Left | Right => CursorIcon::ResizeHorizontal,
    }
}

/// Renders the accent bounding stroke surrounding the selected node.
pub fn render_selection_box(pnt: &egui::Painter, rect: Rect, stroke_color: Color32) {
    pnt.rect_stroke(
        rect.expand(2.0),
        CornerRadius::same(2),
        Stroke::new(1.5, stroke_color),
        egui::StrokeKind::Outside,
    );
}

/// Renders the floating dimension & transform HUD pill below the bounding box.
pub fn render_dimension_badge(pnt: &egui::Painter, rect: Rect, viewport: &Viewport2D, rotation: f32) {
    let world_w = (rect.width() / viewport.zoom).round() as i32;
    let world_h = (rect.height() / viewport.zoom).round() as i32;

    let badge_text = if rotation.abs() > 0.001 {
        format!("{} × {} • ↻ {:.0}°", world_w, world_h, rotation)
    } else {
        format!("{} × {}", world_w, world_h)
    };

    let font_id = egui::FontId::monospace(10.0);
    let galley = pnt.layout_no_wrap(badge_text, font_id, Color32::WHITE);
    let badge_w = galley.size().x + 12.0;
    let badge_h = 17.0;
    let badge_pos = Pos2::new(rect.center().x - badge_w / 2.0, rect.bottom() + 6.0);
    let badge_rect = Rect::from_min_size(badge_pos, Vec2::new(badge_w, badge_h));

    pnt.rect_filled(badge_rect, CornerRadius::same(4), Color32::from_rgb(18, 20, 26));
    pnt.rect_stroke(
        badge_rect,
        CornerRadius::same(4),
        Stroke::new(1.0, theme::BORDER),
        egui::StrokeKind::Outside,
    );
    pnt.galley(Pos2::new(badge_pos.x + 6.0, badge_pos.y + 2.0), galley, Color32::WHITE);
}

/// Renders the rotation lollipop handle (stem line and circular knob) above the top center.
pub fn render_rotation_handle(
    ui: &mut egui::Ui,
    rect: Rect,
    node_id: &str,
    current_rotation: f32,
    events: &mut Vec<CanvasEvent>,
) -> bool {
    let top_center = Pos2::new(rect.center().x, rect.top());
    let knob_center = Pos2::new(top_center.x, top_center.y - ROTATION_STEM_LENGTH);

    // Stem line from top-center edge to rotation knob
    ui.painter().line_segment(
        [top_center, knob_center],
        Stroke::new(1.0, theme::ACCENT),
    );

    let interact_rect = Rect::from_center_size(knob_center, Vec2::splat(HANDLE_SIZE + 4.0));
    let sense_id = ui.id().with(("rotate_handle", node_id));
    let resp = ui.interact(interact_rect, sense_id, Sense::drag())
        .on_hover_cursor(CursorIcon::Crosshair);

    let is_active = resp.hovered() || resp.dragged();
    let knob_radius = if is_active { ROTATION_KNOB_RADIUS + 1.5 } else { ROTATION_KNOB_RADIUS };
    let knob_fill = if resp.dragged() { theme::ACCENT } else { Color32::WHITE };

    ui.painter().circle_filled(knob_center, knob_radius, knob_fill);
    ui.painter().circle_stroke(knob_center, knob_radius, Stroke::new(1.5, theme::ACCENT));

    if resp.drag_started() {
        events.push(CanvasEvent::NodeResizeStarted);
    }

    if resp.dragged() {
        if let Some(mouse_pos) = ui.input(|i| i.pointer.interact_pos()) {
            let shift = ui.input(|i| i.modifiers.shift);
            let angle = calculate_rotation_angle(rect.center(), mouse_pos, shift);
            events.push(CanvasEvent::NodeModified {
                id: node_id.to_string(),
                update: NodeUpdate::Rotate(angle),
            });
        }
    }

    // Live Degree Angle HUD Pill above knob when hovered or dragging
    if is_active || current_rotation.abs() > 0.001 {
        let angle_label = format!("↻ {:.0}°", current_rotation);
        let font_id = egui::FontId::monospace(9.5);
        let galley = ui.painter().layout_no_wrap(angle_label, font_id, Color32::WHITE);
        let hud_w = galley.size().x + 10.0;
        let hud_h = 16.0;
        let hud_pos = Pos2::new(knob_center.x - hud_w / 2.0, knob_center.y - 18.0);
        let hud_rect = Rect::from_min_size(hud_pos, Vec2::new(hud_w, hud_h));

        ui.painter().rect_filled(hud_rect, CornerRadius::same(3), Color32::from_rgb(18, 20, 26));
        ui.painter().rect_stroke(hud_rect, CornerRadius::same(3), Stroke::new(1.0, theme::ACCENT), egui::StrokeKind::Outside);
        ui.painter().galley(Pos2::new(hud_pos.x + 5.0, hud_pos.y + 1.5), galley, Color32::WHITE);
    }

    resp.dragged()
}

/// Renders the 8 resize handles around the perimeter of the bounding box.
pub fn render_resize_handles(
    ui: &mut egui::Ui,
    rect: Rect,
    events: &mut Vec<CanvasEvent>,
    node_id: &str,
    viewport: &Viewport2D,
) -> bool {
    use ResizeHandle::*;
    let centers = [
        (TopLeft,     Pos2::new(rect.left(),  rect.top())),
        (Top,         Pos2::new(rect.center().x, rect.top())),
        (TopRight,    Pos2::new(rect.right(), rect.top())),
        (Right,       Pos2::new(rect.right(), rect.center().y)),
        (BottomRight, Pos2::new(rect.right(), rect.bottom())),
        (Bottom,      Pos2::new(rect.center().x, rect.bottom())),
        (BottomLeft,  Pos2::new(rect.left(),  rect.bottom())),
        (Left,        Pos2::new(rect.left(),  rect.center().y)),
    ];

    let mut any_dragged = false;
    for (handle, center) in centers {
        let hr = Rect::from_center_size(center, Vec2::splat(HANDLE_SIZE));
        let sense_id = ui.id().with(("resize", node_id, handle));
        let response = ui.interact(hr, sense_id, Sense::drag())
            .on_hover_cursor(cursor_for_handle(handle));

        let is_hovered = response.hovered();
        let handle_radius = if is_hovered { 4.5 } else { 3.5 };
        ui.painter().circle_filled(center, handle_radius, Color32::WHITE);
        ui.painter().circle_stroke(center, handle_radius, Stroke::new(1.5, theme::ACCENT));

        if response.drag_started() {
            events.push(CanvasEvent::NodeResizeStarted);
        }

        if response.dragged() {
            any_dragged = true;
            let world_delta = response.drag_delta() / viewport.zoom;
            events.push(CanvasEvent::NodeResized {
                id: node_id.to_owned(),
                handle,
                delta: (world_delta.x, world_delta.y),
            });
        }
    }
    any_dragged
}

/// Comprehensive transform overlay: selection stroke, dimension badge, 8 resize handles, and rotation lollipop.
pub fn render_transform_handles(
    ui: &mut egui::Ui,
    rect: Rect,
    node_id: &str,
    events: &mut Vec<CanvasEvent>,
    viewport: &Viewport2D,
    rotation: f32,
) -> bool {
    render_selection_box(ui.painter(), rect, theme::ACCENT);
    render_dimension_badge(ui.painter(), rect, viewport, rotation);
    let rotating = render_rotation_handle(ui, rect, node_id, rotation, events);
    let resizing = render_resize_handles(ui, rect, events, node_id, viewport);
    rotating || resizing
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_rotation_angle_cardinal_directions() {
        let center = Pos2::new(100.0, 100.0);

        // Pointing straight up -> 0°
        let up = Pos2::new(100.0, 50.0);
        let ang_up = calculate_rotation_angle(center, up, false);
        assert_eq!(ang_up, 0.0);

        // Pointing straight right -> 90°
        let right = Pos2::new(150.0, 100.0);
        let ang_right = calculate_rotation_angle(center, right, false);
        assert_eq!(ang_right, 90.0);

        // Pointing straight down -> 180° or -180°
        let down = Pos2::new(100.0, 150.0);
        let ang_down = calculate_rotation_angle(center, down, false);
        assert_eq!(ang_down.abs(), 180.0);

        // Pointing straight left -> -90°
        let left = Pos2::new(50.0, 100.0);
        let ang_left = calculate_rotation_angle(center, left, false);
        assert_eq!(ang_left, -90.0);
    }

    #[test]
    fn test_calculate_rotation_angle_shift_snap_15_degrees() {
        let center = Pos2::new(0.0, 0.0);
        // Approximately 33 degrees (cos(33°)≈0.54, sin(33°)≈0.84)
        let pos = Pos2::new(54.0, -84.0);

        let smooth = calculate_rotation_angle(center, pos, false);
        assert!((smooth - 33.0).abs() <= 1.0);

        let snapped = calculate_rotation_angle(center, pos, true);
        assert_eq!(snapped, 30.0); // 33° snaps to nearest 15° multiple (30°)
    }

    #[test]
    fn test_selection_box_expansion_geometry() {
        let r = Rect::from_min_max(Pos2::new(10.0, 20.0), Pos2::new(110.0, 120.0));
        assert_eq!(r.width(), 100.0);
        assert_eq!(r.height(), 100.0);
        let top_center = Pos2::new(r.center().x, r.top());
        assert_eq!(top_center, Pos2::new(60.0, 20.0));
    }
}
