//! Vector Shape & Line/Arrow GPU Canvas Renderer.
//! Provides mathematical geometry calculation, sub-pixel rasterization,
//! directional arrowhead synthesis, dashed strokes, and geometric polygon fills.

use eframe::egui::{self, Color32, CornerRadius, Pos2, Rect, Sense, Stroke, Vec2};
use crate::scene::{Node, NodeStyle, ShapeKind};

/// Extract RGBA fill color from NodeStyle.
fn style_fill(s: &NodeStyle) -> Color32 {
    Color32::from_rgba_unmultiplied(
        (s.bg_color[0] * 255.0) as u8,
        (s.bg_color[1] * 255.0) as u8,
        (s.bg_color[2] * 255.0) as u8,
        (s.bg_color[3] * 255.0) as u8,
    )
}

/// Extract RGBA border/stroke color from NodeStyle.
fn style_border_color(s: &NodeStyle) -> Color32 {
    Color32::from_rgba_unmultiplied(
        (s.border_color[0] * 255.0) as u8,
        (s.border_color[1] * 255.0) as u8,
        (s.border_color[2] * 255.0) as u8,
        (s.border_color[3] * 255.0) as u8,
    )
}

/// Calculate primary start and end points of a vector line from bounding box.
pub fn calculate_line_endpoints(rect: Rect) -> (Pos2, Pos2) {
    if rect.width() >= rect.height() {
        (rect.left_center(), rect.right_center())
    } else {
        (rect.center_top(), rect.center_bottom())
    }
}

/// Calculate symmetrical arrowhead wing endpoints at target position `p2`.
pub fn calculate_arrow_head(p1: Pos2, p2: Pos2, head_len: f32, head_width: f32) -> (Pos2, Pos2) {
    let d = p2 - p1;
    let len = d.length();
    if len < 0.001 {
        return (p2, p2);
    }
    let tangent = d / len;
    let normal = Vec2::new(-tangent.y, tangent.x);

    let wing1 = p2 - tangent * head_len + normal * (head_width * 0.5);
    let wing2 = p2 - tangent * head_len - normal * (head_width * 0.5);
    (wing1, wing2)
}

/// Render a dashed line segment along vector endpoints.
pub fn render_dashed_line(
    painter: &egui::Painter,
    p1: Pos2,
    p2: Pos2,
    stroke: Stroke,
    dash_len: f32,
    gap_len: f32,
) {
    let d = p2 - p1;
    let total_len = d.length();
    if total_len < 0.001 {
        return;
    }
    let dir = d / total_len;

    let mut current_dist = 0.0;
    while current_dist < total_len {
        let seg_start = p1 + dir * current_dist;
        let seg_end_dist = (current_dist + dash_len).min(total_len);
        let seg_end = p1 + dir * seg_end_dist;

        painter.line_segment([seg_start, seg_end], stroke);
        current_dist += dash_len + gap_len;
    }
}

/// Render a vector shape node (Ellipse, Line, Arrow, Triangle, Rectangle).
pub fn render_vector_shape(
    ui: &mut egui::Ui,
    node: &Node,
    kind: &ShapeKind,
    z: f32,
) -> bool {
    let fill_col = style_fill(&node.style);
    let border_col = style_border_color(&node.style);
    let stroke_w = (node.style.border_width * z).max(1.0);
    let stroke = Stroke::new(stroke_w, border_col);

    let (resp, pnt) = ui.allocate_painter(ui.available_size(), Sense::hover());
    let r = resp.rect;

    match kind {
        ShapeKind::Ellipse => {
            let rad = (r.width().min(r.height()) / 2.0).max(1.0);
            pnt.circle_filled(r.center(), rad, fill_col);
            if node.style.border_width > 0.0 {
                pnt.circle_stroke(r.center(), rad, stroke);
            }
        }
        ShapeKind::Line => {
            let (p1, p2) = calculate_line_endpoints(r);
            if node.style.border_radius > 10.0 {
                render_dashed_line(&pnt, p1, p2, stroke, 8.0 * z.max(0.5), 5.0 * z.max(0.5));
            } else {
                pnt.line_segment([p1, p2], stroke);
            }
        }
        ShapeKind::Arrow => {
            let (p1, p2) = calculate_line_endpoints(r);
            pnt.line_segment([p1, p2], stroke);

            let head_len = (12.0 * z).clamp(8.0, 24.0);
            let head_width = (10.0 * z).clamp(6.0, 20.0);
            let (w1, w2) = calculate_arrow_head(p1, p2, head_len, head_width);

            // Render solid geometric arrow head tip
            pnt.add(egui::Shape::convex_polygon(vec![p2, w1, w2], border_col, Stroke::NONE));
        }
        ShapeKind::Triangle => {
            let top = r.center_top();
            let bl = r.left_bottom();
            let br = r.right_bottom();
            pnt.add(egui::Shape::convex_polygon(vec![top, br, bl], fill_col, stroke));
        }
        ShapeKind::Rectangle => {
            let cr = CornerRadius::same((node.style.border_radius * z) as u8);
            pnt.rect_filled(r, cr, fill_col);
            if node.style.border_width > 0.0 {
                pnt.rect_stroke(r, cr, stroke, egui::StrokeKind::Outside);
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_line_endpoints_horizontal_and_vertical() {
        let horiz_rect = Rect::from_min_size(Pos2::new(10.0, 20.0), Vec2::new(100.0, 10.0));
        let (p1, p2) = calculate_line_endpoints(horiz_rect);
        assert_eq!(p1, Pos2::new(10.0, 25.0));
        assert_eq!(p2, Pos2::new(110.0, 25.0));

        let vert_rect = Rect::from_min_size(Pos2::new(10.0, 20.0), Vec2::new(10.0, 100.0));
        let (vp1, vp2) = calculate_line_endpoints(vert_rect);
        assert_eq!(vp1, Pos2::new(15.0, 20.0));
        assert_eq!(vp2, Pos2::new(15.0, 120.0));
    }

    #[test]
    fn test_calculate_arrow_head_symmetry() {
        let p1 = Pos2::new(0.0, 0.0);
        let p2 = Pos2::new(100.0, 0.0);
        let (w1, w2) = calculate_arrow_head(p1, p2, 10.0, 8.0);

        assert_eq!(w1.x, 90.0);
        assert_eq!(w2.x, 90.0);
        assert_eq!(w1.y, 4.0);
        assert_eq!(w2.y, -4.0);
    }
}
