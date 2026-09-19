use crate::flow::{FlowGraph, FlowNodeKind};
use crate::Viewport2D;
use eframe::egui::{self, Color32, Pos2, Rect, Stroke, StrokeKind, Vec2};

pub const NODE_W: f32 = 200.;
pub const NODE_H: f32 = 50.;

/// Output port center in world coordinates (right side, middle)
pub fn node_output_port(node_pos: (f32, f32)) -> (f32, f32) {
    (node_pos.0 + NODE_W, node_pos.1 + NODE_H / 2.)
}

/// Input port center in world coordinates (left side, middle)
pub fn node_input_port(node_pos: (f32, f32)) -> (f32, f32) {
    (node_pos.0, node_pos.1 + NODE_H / 2.)
}

/// Returns true if `point` is within `threshold` distance of the line segment (a, b).
pub fn is_near_line_segment(point: (f32, f32), a: (f32, f32), b: (f32, f32), threshold: f32) -> bool {
    let (px, py) = point;
    let (ax, ay) = a;
    let (bx, by) = b;
    let dx = bx - ax;
    let dy = by - ay;
    let len_sq = dx * dx + dy * dy;
    if len_sq == 0.0 {
        // Degenerate line — just distance to point
        return (px - ax) * (px - ax) + (py - ay) * (py - ay) <= threshold * threshold;
    }
    let t = ((px - ax) * dx + (py - ay) * dy) / len_sq;
    let t = t.clamp(0.0, 1.0);
    let nearest_x = ax + t * dx;
    let nearest_y = ay + t * dy;
    let dist_sq = (px - nearest_x) * (px - nearest_x) + (py - nearest_y) * (py - nearest_y);
    dist_sq <= threshold * threshold
}

pub fn is_over_output_port(node_pos: (f32, f32), mouse_world: (f32, f32)) -> bool {
    let (px, py) = node_output_port(node_pos);
    let dx = mouse_world.0 - px;
    let dy = mouse_world.1 - py;
    dx * dx + dy * dy <= 64. // radius 8 squared
}

fn kind_label(kind: &FlowNodeKind) -> &'static str {
    match kind {
        FlowNodeKind::TriggerClick { .. } => "Trigger: Click",
        FlowNodeKind::NavigateTo { .. } => "Navigate To",
        FlowNodeKind::SaveToDatabase { .. } => "Save To DB",
    }
}

fn kind_color(kind: &FlowNodeKind) -> Color32 {
    match kind {
        FlowNodeKind::TriggerClick { .. } => Color32::from_rgb(79, 140, 237),
        FlowNodeKind::NavigateTo { .. } => Color32::from_rgb(237, 180, 60),
        FlowNodeKind::SaveToDatabase { .. } => Color32::from_rgb(60, 200, 120),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn draw_flow_graph(
    _ui: &mut egui::Ui,
    painter: &egui::Painter,
    graph: &FlowGraph,
    viewport: &Viewport2D,
    canvas_origin: Pos2,
    _canvas_rect: Rect,
    selected_id: Option<&str>,
    wiring_source: Option<&str>,
    wiring_end: Option<(f32, f32)>,
) {
    // ── Edges ──
    for edge in &graph.edges {
        let from = match graph.nodes.get(&edge.from_node) {
            Some(n) => n,
            None => continue,
        };
        let to = match graph.nodes.get(&edge.to_node) {
            Some(n) => n,
            None => continue,
        };
        let (fx, fy) = node_output_port(from.position);
        let (tx, ty) = node_input_port(to.position);
        let sp = viewport.world_to_screen(Pos2::new(fx, fy), canvas_origin);
        let ep = viewport.world_to_screen(Pos2::new(tx, ty), canvas_origin);
        painter.line_segment([sp, ep], Stroke::new(2., Color32::from_rgb(100, 100, 100)));
        let dir = (ep - sp).normalized();
        let perp = Vec2::new(-dir.y, dir.x);
        painter.line_segment([ep, ep - dir * 8. + perp * 4.], Stroke::new(2., Color32::from_rgb(100, 100, 100)));
        painter.line_segment([ep, ep - dir * 8. - perp * 4.], Stroke::new(2., Color32::from_rgb(100, 100, 100)));
    }

    // ── Active wire (drawn below nodes) ──
    if let (Some(src_id), Some(end_world)) = (wiring_source, wiring_end) {
        if let Some(src_node) = graph.nodes.get(src_id) {
            let (sx, sy) = node_output_port(src_node.position);
            let sp = viewport.world_to_screen(Pos2::new(sx, sy), canvas_origin);
            let ep = viewport.world_to_screen(Pos2::new(end_world.0, end_world.1), canvas_origin);
            painter.line_segment([sp, ep], Stroke::new(2., Color32::from_rgb(180, 230, 255)));
            // Arrowhead at cursor
            let dir = (ep - sp).normalized();
            let perp = Vec2::new(-dir.y, dir.x);
            painter.line_segment([ep, ep - dir * 8. + perp * 4.], Stroke::new(2., Color32::from_rgb(180, 230, 255)));
            painter.line_segment([ep, ep - dir * 8. - perp * 4.], Stroke::new(2., Color32::from_rgb(180, 230, 255)));
        }
    }

    // ── Nodes ──
    for node in graph.nodes.values() {
        let screen_pos = viewport.world_to_screen(Pos2::new(node.position.0, node.position.1), canvas_origin);
        let node_rect = Rect::from_min_size(screen_pos, Vec2::new(NODE_W, NODE_H));
        let color = kind_color(&node.kind);

        painter.rect_filled(node_rect, 4., Color32::from_rgb(30, 30, 40));
        painter.rect_stroke(node_rect, 4., Stroke::new(1.5, color), StrokeKind::Outside);

        if selected_id == Some(node.id.as_str()) {
            painter.rect_stroke(node_rect, 4., Stroke::new(2.5, Color32::YELLOW), StrokeKind::Outside);
        }

        // Header bar
        let header_rect = Rect::from_min_size(node_rect.min, Vec2::new(NODE_W, 20.));
        painter.rect_filled(header_rect, 4., color);
        painter.rect_filled(
            Rect::from_min_max(Pos2::new(node_rect.min.x, header_rect.max.y - 4.), node_rect.max),
            0,
            Color32::from_rgb(30, 30, 40),
        );

        let label = kind_label(&node.kind);
        painter.text(
            Pos2::new(header_rect.center().x, header_rect.center().y),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(10.),
            Color32::from_rgb(0, 0, 0),
        );

        let detail = match &node.kind {
            FlowNodeKind::TriggerClick { target_node_id } => format!("target: {}", target_node_id),
            FlowNodeKind::NavigateTo { page_id } => format!("page: {}", page_id),
            FlowNodeKind::SaveToDatabase { entity } => format!("entity: {}", entity),
        };
        painter.text(
            Pos2::new(node_rect.min.x + 6., node_rect.min.y + 30.),
            egui::Align2::LEFT_TOP,
            &detail,
            egui::FontId::proportional(9.),
            Color32::from_rgb(180, 180, 180),
        );

        // Output port (right side)
        let (op_wx, op_wy) = node_output_port(node.position);
        let op_screen = viewport.world_to_screen(Pos2::new(op_wx, op_wy), canvas_origin);
        painter.circle_filled(op_screen, 6., color);
        painter.circle_stroke(op_screen, 6., Stroke::new(1.5, Color32::from_rgb(200, 200, 200)));

        // Input port (left side)
        let (ip_wx, ip_wy) = node_input_port(node.position);
        let ip_screen = viewport.world_to_screen(Pos2::new(ip_wx, ip_wy), canvas_origin);
        painter.circle_filled(ip_screen, 4., Color32::from_rgb(40, 40, 50));
        painter.circle_stroke(ip_screen, 4., Stroke::new(1., Color32::from_rgb(120, 120, 120)));
    }
}
