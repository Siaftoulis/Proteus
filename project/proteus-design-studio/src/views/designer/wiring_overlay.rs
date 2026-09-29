//! Visual Interactive Button-to-Flow Wiring Overlay on Canvas.
//! Renders smooth cubic bezier wires connecting trigger buttons directly to
//! their target destination screens and SQLite entity mutations.

use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};
use crate::models::InspectorTab;
use crate::scene::NodeType;
use crate::ProteusApp;

/// Renders visual wiring overlay for all interactive triggers in the document.
pub fn render_flow_wiring(
    app: &ProteusApp,
    ctx: &egui::Context,
    pnt: &egui::Painter,
    canvas_origin: Pos2,
) {
    let p = app.palette(ctx);
    let zoom = app.viewport.zoom;

    // Check if we should render wires: show if Prototype tab active, or if buttons exist with actions
    let is_proto_active = app.inspector_tab == InspectorTab::Prototype;

    for (node_id, node) in &app.project_doc.nodes {
        if let NodeType::Button { label, .. } = &node.node_type {
            let (target_page, submit_entity) = app.project_doc.get_button_action(node_id);
            let has_action = target_page.is_some() || submit_entity.is_some();

            let is_selected = app.editor_state.selected_node_ids.contains(node_id);

            // If no action and not in proto/selected, skip
            if !has_action && !is_proto_active && !is_selected {
                continue;
            }

            let Some((bx, by, bw, bh)) = app.project_doc.node_world_rect(node_id) else {
                continue;
            };

            let b_screen_tl = app.viewport.world_to_screen(Pos2::new(bx, by), canvas_origin);
            let b_screen_w = bw * zoom;
            let b_screen_h = bh * zoom;
            let p1 = Pos2::new(b_screen_tl.x + b_screen_w, b_screen_tl.y + b_screen_h * 0.5);

            // 1. Draw source anchor port on the button's right edge
            let port_color = if is_selected {
                p.accent
            } else if has_action {
                Color32::from_rgb(56, 189, 248) // Sky blue
            } else {
                Color32::from_rgb(148, 163, 184) // Slate
            };

            pnt.circle_filled(p1, 4.5, port_color);
            pnt.circle_stroke(p1, 4.5, Stroke::new(1.2, Color32::WHITE));

            // 2. If target screen is configured, draw the cubic bezier wire
            if let Some(ref target_id) = target_page {
                if let Some((tx, ty, _tw, th)) = app.project_doc.node_world_rect(target_id) {
                    let t_screen_tl = app.viewport.world_to_screen(Pos2::new(tx, ty), canvas_origin);
                    let t_screen_h = th * zoom;
                    let p2 = Pos2::new(t_screen_tl.x, t_screen_tl.y + t_screen_h * 0.5);

                    // Compute smooth horizontal cubic bezier control points
                    let dx = (p2.x - p1.x).abs().max(40.0) * 0.5;
                    let cp1 = Pos2::new(p1.x + dx, p1.y);
                    let cp2 = Pos2::new(p2.x - dx, p2.y);

                    let wire_stroke = if is_selected {
                        Stroke::new(2.5, p.accent)
                    } else {
                        Stroke::new(1.6, Color32::from_rgba_unmultiplied(p.accent.r(), p.accent.g(), p.accent.b(), 180))
                    };

                    // Draw curve
                    pnt.add(egui::epaint::CubicBezierShape::from_points_stroke(
                        [p1, cp1, cp2, p2],
                        false,
                        Color32::TRANSPARENT,
                        wire_stroke,
                    ));

                    // Target Arrow / Pin
                    pnt.circle_filled(p2, 4.0, p.accent);
                    pnt.circle_stroke(p2, 4.0, Stroke::new(1.0, Color32::WHITE));

                    // Midpoint badge
                    let mid_x = 0.125 * p1.x + 0.375 * cp1.x + 0.375 * cp2.x + 0.125 * p2.x;
                    let mid_y = 0.125 * p1.y + 0.375 * cp1.y + 0.375 * cp2.y + 0.125 * p2.y;
                    let mid_pos = Pos2::new(mid_x, mid_y);

                    let target_name = app.project_doc.nodes.get(target_id)
                        .map(|n| n.name.as_str())
                        .unwrap_or(target_id.as_str());

                    let badge_text = if let Some(ref entity) = submit_entity {
                        format!("💾 {} ➔ ⚡ {}", entity, target_name)
                    } else {
                        format!("⚡ ➔ {}", target_name)
                    };

                    let badge_font = egui::FontId::proportional(9.0);
                    let text_color = if is_selected { p.text } else { p.text_dim };
                    let galley = pnt.layout_no_wrap(badge_text, badge_font, text_color);
                    let badge_rect = Rect::from_center_size(
                        mid_pos,
                        Vec2::new(galley.size().x + 12.0, galley.size().y + 6.0),
                    );

                    pnt.rect_filled(badge_rect, egui::CornerRadius::same(4), p.elevated);
                    pnt.rect_stroke(
                        badge_rect,
                        egui::CornerRadius::same(4),
                        Stroke::new(1.0, if is_selected { p.accent } else { p.border }),
                        egui::StrokeKind::Outside,
                    );
                    pnt.galley(
                        Pos2::new(badge_rect.left() + 6.0, badge_rect.top() + 3.0),
                        galley,
                        text_color,
                    );
                }
            } else if let Some(ref entity) = submit_entity {
                // Submit only (no navigation) - draw database mutation badge on right of port
                let badge_pos = Pos2::new(p1.x + 14.0, p1.y);
                let badge_text = format!("💾 Submit '{}'", entity);
                let badge_font = egui::FontId::proportional(9.0);
                let text_color = p.text;
                let galley = pnt.layout_no_wrap(badge_text, badge_font, text_color);
                let badge_rect = Rect::from_min_size(
                    Pos2::new(badge_pos.x, badge_pos.y - galley.size().y * 0.5 - 3.0),
                    Vec2::new(galley.size().x + 12.0, galley.size().y + 6.0),
                );

                pnt.line_segment([p1, Pos2::new(badge_rect.left(), badge_rect.center().y)], Stroke::new(1.5, p.accent));
                pnt.rect_filled(badge_rect, egui::CornerRadius::same(4), p.elevated);
                pnt.rect_stroke(
                    badge_rect,
                    egui::CornerRadius::same(4),
                    Stroke::new(1.0, p.accent),
                    egui::StrokeKind::Outside,
                );
                pnt.galley(
                    Pos2::new(badge_rect.left() + 6.0, badge_rect.top() + 3.0),
                    galley,
                    text_color,
                );
            } else if is_selected && is_proto_active {
                // Selected button with no action yet - show unlinked port hint
                let badge_pos = Pos2::new(p1.x + 10.0, p1.y);
                let badge_text = format!("⚡ Wire '{}'", label);
                let badge_font = egui::FontId::proportional(8.5);
                let text_color = p.text_dim;
                let galley = pnt.layout_no_wrap(badge_text, badge_font, text_color);
                let badge_rect = Rect::from_min_size(
                    Pos2::new(badge_pos.x, badge_pos.y - galley.size().y * 0.5 - 2.0),
                    Vec2::new(galley.size().x + 8.0, galley.size().y + 4.0),
                );

                pnt.rect_filled(badge_rect, egui::CornerRadius::same(3), Color32::from_black_alpha(100));
                pnt.galley(
                    Pos2::new(badge_rect.left() + 4.0, badge_rect.top() + 2.0),
                    galley,
                    text_color,
                );
            }
        }
    }
}
