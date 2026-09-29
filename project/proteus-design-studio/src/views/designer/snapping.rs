//! Mac-style smart magnetic snapping ("Aimbot") and dynamic visual alignment guidelines engine.
//!
//! Provides ultra-responsive, magnetic snapping to sibling bounding boxes,
//! centers, edges, and grid coordinates during canvas interaction.

use eframe::egui::{Color32, Pos2, Stroke};
use crate::models::{LayoutMode, GRID};
use crate::scene::{ProjectDocument, Sizing};
use crate::Viewport2D;

/// Visual alignment guide produced by magnetic snapping.
#[derive(Debug, Clone, PartialEq)]
pub enum SnapGuideKind {
    /// Vertical alignment line at world coordinate `x`, spanning from `y_start` to `y_end`.
    Vertical {
        x: f32,
        y_start: f32,
        y_end: f32,
        label: Option<String>,
    },
    /// Horizontal alignment line at world coordinate `y`, spanning from `x_start` to `x_end`.
    Horizontal {
        y: f32,
        x_start: f32,
        x_end: f32,
        label: Option<String>,
    },
}

/// Result of smart snapping calculation.
#[derive(Debug, Clone, Default)]
pub struct SnapResult {
    pub snapped_pos: (f32, f32),
    pub did_snap_x: bool,
    pub did_snap_y: bool,
    pub guides: Vec<SnapGuideKind>,
}

/// Calculates magnetic snap targets for a moving or resizing node.
///
/// Compares candidate edges and centers (Left, Center, Right, Top, Middle, Bottom)
/// against all other active nodes and the virtual layout grid.
pub fn calculate_smart_snap(
    moving_id: &str,
    proposed_pos: (f32, f32),
    moving_size: (f32, f32),
    doc: &ProjectDocument,
    layout_mode: LayoutMode,
    threshold: f32,
) -> SnapResult {
    let mut result = SnapResult {
        snapped_pos: proposed_pos,
        did_snap_x: false,
        did_snap_y: false,
        guides: Vec::new(),
    };

    let (mw, mh) = moving_size;
    let (px, py) = proposed_pos;

    // Moving anchor points
    let m_left = px;
    let m_center_x = px + mw / 2.0;
    let m_right = px + mw;

    let m_top = py;
    let m_center_y = py + mh / 2.0;
    let m_bottom = py + mh;

    let mut best_dx: Option<(f32, f32, f32, f32, &'static str)> = None; // (offset, snap_x, y_min, y_max, tag)
    let mut best_dy: Option<(f32, f32, f32, f32, &'static str)> = None; // (offset, snap_y, x_min, x_max, tag)

    for (id, node) in &doc.nodes {
        if id == moving_id {
            continue;
        }

        let (nw, nh) = match (&node.layout.width, &node.layout.height) {
            (Sizing::Fixed(w), Sizing::Fixed(h)) => (*w, *h),
            (Sizing::Fixed(w), _) => (*w, 100.0),
            (_, Sizing::Fixed(h)) => (200.0, *h),
            _ => (200.0, 100.0),
        };

        let s_left = node.position.0;
        let s_center_x = s_left + nw / 2.0;
        let s_right = s_left + nw;

        let s_top = node.position.1;
        let s_center_y = s_top + nh / 2.0;
        let s_bottom = s_top + nh;

        // X-axis alignment pairs: [moving_val, sibling_val, tag]
        let x_pairs = [
            (m_left, s_left, "Left"),
            (m_left, s_right, "Right Edge"),
            (m_center_x, s_center_x, "Center"),
            (m_right, s_left, "Left Edge"),
            (m_right, s_right, "Right"),
        ];

        for (m_val, s_val, tag) in x_pairs {
            let diff = s_val - m_val;
            if diff.abs() <= threshold {
                let cur_best = best_dx.as_ref().map(|(d, ..)| d.abs()).unwrap_or(f32::INFINITY);
                if diff.abs() < cur_best {
                    let y_min = m_top.min(s_top) - 16.0;
                    let y_max = m_bottom.max(s_bottom) + 16.0;
                    best_dx = Some((diff, s_val, y_min, y_max, tag));
                }
            }
        }

        // Y-axis alignment pairs: [moving_val, sibling_val, tag]
        let y_pairs = [
            (m_top, s_top, "Top"),
            (m_top, s_bottom, "Bottom Edge"),
            (m_center_y, s_center_y, "Middle"),
            (m_bottom, s_top, "Top Edge"),
            (m_bottom, s_bottom, "Bottom"),
        ];

        for (m_val, s_val, tag) in y_pairs {
            let diff = s_val - m_val;
            if diff.abs() <= threshold {
                let cur_best = best_dy.as_ref().map(|(d, ..)| d.abs()).unwrap_or(f32::INFINITY);
                if diff.abs() < cur_best {
                    let x_min = m_left.min(s_left) - 16.0;
                    let x_max = m_right.max(s_right) + 16.0;
                    best_dy = Some((diff, s_val, x_min, x_max, tag));
                }
            }
        }
    }

    // Apply best X sibling snap or fallback to grid snap
    if let Some((diff, snap_x, y_min, y_max, tag)) = best_dx {
        result.snapped_pos.0 = px + diff;
        result.did_snap_x = true;
        result.guides.push(SnapGuideKind::Vertical {
            x: snap_x,
            y_start: y_min,
            y_end: y_max,
            label: Some(tag.to_string()),
        });
    } else if layout_mode == LayoutMode::Grid {
        let grid_target = (px / GRID).round() * GRID;
        let diff = grid_target - px;
        if diff.abs() <= threshold {
            result.snapped_pos.0 = grid_target;
            result.did_snap_x = true;
            result.guides.push(SnapGuideKind::Vertical {
                x: grid_target,
                y_start: py - 20.0,
                y_end: py + mh + 20.0,
                label: Some(format!("{:.0}", grid_target)),
            });
        }
    }

    // Apply best Y sibling snap or fallback to grid snap
    if let Some((diff, snap_y, x_min, x_max, tag)) = best_dy {
        result.snapped_pos.1 = py + diff;
        result.did_snap_y = true;
        result.guides.push(SnapGuideKind::Horizontal {
            y: snap_y,
            x_start: x_min,
            x_end: x_max,
            label: Some(tag.to_string()),
        });
    } else if layout_mode == LayoutMode::Grid {
        let grid_target = (py / GRID).round() * GRID;
        let diff = grid_target - py;
        if diff.abs() <= threshold {
            result.snapped_pos.1 = grid_target;
            result.did_snap_y = true;
            result.guides.push(SnapGuideKind::Horizontal {
                y: grid_target,
                x_start: px - 20.0,
                x_end: px + mw + 20.0,
                label: Some(format!("{:.0}", grid_target)),
            });
        }
    }

    result
}

/// Renders luxury magnetic snap guidelines on top of the canvas viewport.
pub fn render_snap_guides(
    pnt: &egui::Painter,
    guides: &[SnapGuideKind],
    viewport: &Viewport2D,
    canvas_origin: Pos2,
) {
    // Apple / Figma magenta-pink luxury alignment color
    let guide_color = Color32::from_rgb(255, 45, 85);
    let guide_stroke = Stroke::new(1.0, guide_color);

    for guide in guides {
        match guide {
            SnapGuideKind::Vertical { x, y_start, y_end, label } => {
                let start_scr = viewport.world_to_screen(Pos2::new(*x, *y_start), canvas_origin);
                let end_scr = viewport.world_to_screen(Pos2::new(*x, *y_end), canvas_origin);
                pnt.line_segment([start_scr, end_scr], guide_stroke);

                // End cap diamond marks
                pnt.circle_filled(start_scr, 2.5, guide_color);
                pnt.circle_filled(end_scr, 2.5, guide_color);

                if let Some(lbl) = label {
                    let mid_scr = Pos2::new(start_scr.x + 6.0, (start_scr.y + end_scr.y) / 2.0);
                    let font = egui::FontId::proportional(9.0);
                    let galley = pnt.layout_no_wrap(lbl.clone(), font, Color32::WHITE);
                    let pill_rect = egui::Rect::from_min_size(
                        Pos2::new(mid_scr.x, mid_scr.y - 8.0),
                        egui::vec2(galley.size().x + 8.0, 16.0),
                    );
                    pnt.rect_filled(pill_rect, egui::CornerRadius::same(3), guide_color);
                    pnt.galley(Pos2::new(pill_rect.left() + 4.0, pill_rect.top() + 1.5), galley, Color32::WHITE);
                }
            }
            SnapGuideKind::Horizontal { y, x_start, x_end, label } => {
                let start_scr = viewport.world_to_screen(Pos2::new(*x_start, *y), canvas_origin);
                let end_scr = viewport.world_to_screen(Pos2::new(*x_end, *y), canvas_origin);
                pnt.line_segment([start_scr, end_scr], guide_stroke);

                // End cap diamond marks
                pnt.circle_filled(start_scr, 2.5, guide_color);
                pnt.circle_filled(end_scr, 2.5, guide_color);

                if let Some(lbl) = label {
                    let mid_scr = Pos2::new((start_scr.x + end_scr.x) / 2.0, start_scr.y - 10.0);
                    let font = egui::FontId::proportional(9.0);
                    let galley = pnt.layout_no_wrap(lbl.clone(), font, Color32::WHITE);
                    let pill_rect = egui::Rect::from_min_size(
                        Pos2::new(mid_scr.x - galley.size().x / 2.0 - 4.0, mid_scr.y - 8.0),
                        egui::vec2(galley.size().x + 8.0, 16.0),
                    );
                    pnt.rect_filled(pill_rect, egui::CornerRadius::same(3), guide_color);
                    pnt.galley(Pos2::new(pill_rect.left() + 4.0, pill_rect.top() + 1.5), galley, Color32::WHITE);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::{Node, NodeType};

    #[test]
    fn test_smart_snapping_to_sibling_left_edge() {
        let mut doc = ProjectDocument::default();
        let mut target_node = Node::new("target".into(), "Target".into(), NodeType::Frame);
        target_node.position = (100.0, 100.0);
        let _ = doc.add_node(target_node, None);

        // Moving node proposes pos at (103.0, 350.0) with threshold 5.0 (Y is far from sibling)
        let moving_size = (80.0, 40.0);
        let snap = calculate_smart_snap(
            "moving",
            (103.0, 350.0),
            moving_size,
            &doc,
            LayoutMode::Free,
            5.0,
        );

        assert!(snap.did_snap_x);
        assert_eq!(snap.snapped_pos.0, 100.0); // Magnetically snapped from 103.0 to 100.0!
        assert_eq!(snap.guides.len(), 1);
    }

    #[test]
    fn test_smart_snapping_to_sibling_center() {
        let mut doc = ProjectDocument::default();
        let mut target_node = Node::new("target".into(), "Target".into(), NodeType::Frame);
        target_node.position = (100.0, 100.0);
        target_node.layout.width = Sizing::Fixed(100.0); // Center is 150.0
        let _ = doc.add_node(target_node, None);

        // Moving node width is 60.0. If positioned at 122.0, its center is 152.0.
        // Diff between centers is 2.0 <= threshold 6.0
        let snap = calculate_smart_snap(
            "moving",
            (122.0, 300.0),
            (60.0, 40.0),
            &doc,
            LayoutMode::Free,
            6.0,
        );

        assert!(snap.did_snap_x);
        assert_eq!(snap.snapped_pos.0, 120.0); // Center snaps to 150.0 -> pos = 120.0!
    }

    #[test]
    fn test_smart_snapping_to_grid() {
        let doc = ProjectDocument::default();
        // Proposed pos is 22.0, grid is 20.0 (diff 2.0 <= threshold 4.0)
        let snap = calculate_smart_snap(
            "moving",
            (22.0, 50.0),
            (50.0, 50.0),
            &doc,
            LayoutMode::Grid,
            4.0,
        );

        assert!(snap.did_snap_x);
        assert_eq!(snap.snapped_pos.0, 20.0);
    }
}
