use crate::flow::{
    CubicBezierCurve, FlowGraph, FlowNode, FlowNodeKind, PortDataType, PortDirection,
};
use crate::Viewport2D;
use eframe::egui::{self, epaint::PathShape, Color32, Pos2, Rect, Stroke, StrokeKind, Vec2};

pub const NODE_W: f32 = 210.;
pub const NODE_H: f32 = 54.;
pub const PORT_ROW_H: f32 = 20.;
pub const HEADER_H: f32 = 22.;

/// Calculates dynamic height of a node based on its stacked ports.
pub fn node_height(node: &FlowNode) -> f32 {
    let port_count = node.input_ports.len().max(node.output_ports.len()).max(1);
    (HEADER_H + 12.0 + (port_count as f32 * PORT_ROW_H)).max(NODE_H)
}

/// Computes the exact world position for a given port socket on a node.
pub fn port_world_position(node: &FlowNode, port_id: &str) -> Option<(f32, f32)> {
    if let Some(idx) = node.input_ports.iter().position(|p| p.id == port_id) {
        let y = node.position.1 + HEADER_H + 10.0 + (idx as f32 * PORT_ROW_H);
        return Some((node.position.0, y));
    }
    if let Some(idx) = node.output_ports.iter().position(|p| p.id == port_id) {
        let y = node.position.1 + HEADER_H + 10.0 + (idx as f32 * PORT_ROW_H);
        return Some((node.position.0 + NODE_W, y));
    }
    None
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
        return (px - ax) * (px - ax) + (py - ay) * (py - ay) <= threshold * threshold;
    }
    let t = (((px - ax) * dx + (py - ay) * dy) / len_sq).clamp(0.0, 1.0);
    let nearest_x = ax + t * dx;
    let nearest_y = ay + t * dy;
    let dist_sq = (px - nearest_x) * (px - nearest_x) + (py - nearest_y) * (py - nearest_y);
    dist_sq <= threshold * threshold
}

/// Legacy default output port center in world coordinates (right side, middle).
pub fn node_output_port(node_pos: (f32, f32)) -> (f32, f32) {
    (node_pos.0 + NODE_W, node_pos.1 + NODE_H / 2.)
}

/// Legacy default input port center in world coordinates (left side, middle).
pub fn node_input_port(node_pos: (f32, f32)) -> (f32, f32) {
    (node_pos.0, node_pos.1 + NODE_H / 2.)
}

/// Detects if mouse cursor is within hit radius of any port on any node in the graph.
pub fn find_port_at_world_pos(
    graph: &FlowGraph,
    world_pos: (f32, f32),
    radius: f32,
) -> Option<(&str, &str, PortDirection)> {
    let r_sq = radius * radius;
    for node in graph.nodes.values() {
        for p in &node.input_ports {
            if let Some((px, py)) = port_world_position(node, &p.id) {
                let dx = world_pos.0 - px;
                let dy = world_pos.1 - py;
                if dx * dx + dy * dy <= r_sq {
                    return Some((&node.id, &p.id, PortDirection::Input));
                }
            }
        }
        for p in &node.output_ports {
            if let Some((px, py)) = port_world_position(node, &p.id) {
                let dx = world_pos.0 - px;
                let dy = world_pos.1 - py;
                if dx * dx + dy * dy <= r_sq {
                    return Some((&node.id, &p.id, PortDirection::Output));
                }
            }
        }
    }
    None
}

/// Legacy check for mouse hover on default output port.
pub fn is_over_output_port(node_pos: (f32, f32), mouse_world: (f32, f32)) -> bool {
    let (px, py) = node_output_port(node_pos);
    let dx = mouse_world.0 - px;
    let dy = mouse_world.1 - py;
    dx * dx + dy * dy <= 81. // radius 9 squared
}

pub fn port_color(data_type: PortDataType) -> Color32 {
    match data_type {
        PortDataType::ExecutionFlow => Color32::from_rgb(100, 180, 255),
        PortDataType::Boolean => Color32::from_rgb(255, 190, 70),
        PortDataType::Record => Color32::from_rgb(70, 220, 130),
        PortDataType::Text => Color32::from_rgb(200, 130, 255),
        PortDataType::Number => Color32::from_rgb(255, 130, 130),
        PortDataType::Any => Color32::from_rgb(0, 210, 240),
    }
}

fn kind_label(kind: &FlowNodeKind) -> &'static str {
    match kind {
        FlowNodeKind::TriggerClick { .. } => "Trigger: Click",
        FlowNodeKind::NavigateTo { .. } => "Navigate To",
        FlowNodeKind::SaveToDatabase { .. } => "Save To DB",
        FlowNodeKind::Condition { .. } => "Condition (IF)",
        FlowNodeKind::ShowToast { .. } => "Notification",
        FlowNodeKind::FederationBridge { .. } => "Federation Bridge",
        FlowNodeKind::HardwareRelay { .. } => "IoT Relay Actuator",
        FlowNodeKind::HardwareSensor { .. } => "Sensor Watchdog",
        FlowNodeKind::HardwareScale { .. } => "Scale Weigh",
        FlowNodeKind::HardwareDisplay { .. } => "VFD Pole Display",
    }
}

fn kind_color(kind: &FlowNodeKind) -> Color32 {
    match kind {
        FlowNodeKind::TriggerClick { .. } => Color32::from_rgb(79, 140, 237),
        FlowNodeKind::NavigateTo { .. } => Color32::from_rgb(237, 180, 60),
        FlowNodeKind::SaveToDatabase { .. } => Color32::from_rgb(60, 200, 120),
        FlowNodeKind::Condition { .. } => Color32::from_rgb(230, 80, 80),
        FlowNodeKind::ShowToast { .. } => Color32::from_rgb(160, 90, 220),
        FlowNodeKind::FederationBridge { .. } => Color32::from_rgb(0, 180, 216),
        FlowNodeKind::HardwareRelay { .. } => Color32::from_rgb(245, 158, 11),
        FlowNodeKind::HardwareSensor { .. } => Color32::from_rgb(16, 185, 129),
        FlowNodeKind::HardwareScale { .. } => Color32::from_rgb(139, 92, 246),
        FlowNodeKind::HardwareDisplay { .. } => Color32::from_rgb(6, 182, 212),
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
    // 1. Bezier Connected Wires
    for edge in &graph.edges {
        let from = match graph.nodes.get(&edge.from_node) {
            Some(n) => n,
            None => continue,
        };
        let to = match graph.nodes.get(&edge.to_node) {
            Some(n) => n,
            None => continue,
        };

        let (fx, fy) = port_world_position(from, &edge.from_port)
            .unwrap_or_else(|| node_output_port(from.position));
        let (tx, ty) = port_world_position(to, &edge.to_port)
            .unwrap_or_else(|| node_input_port(to.position));

        let curve = CubicBezierCurve::from_endpoints((fx, fy), (tx, ty));
        let samples = curve.sample_points(24);
        let screen_pts: Vec<Pos2> = samples
            .into_iter()
            .map(|(wx, wy)| viewport.world_to_screen(Pos2::new(wx, wy), canvas_origin))
            .collect();

        let wire_color = match edge.branch.as_deref() {
            Some("true") => Color32::from_rgb(80, 200, 130),
            Some("false") => Color32::from_rgb(235, 90, 90),
            _ => Color32::from_rgb(110, 150, 200),
        };

        painter.add(PathShape::line(screen_pts.clone(), Stroke::new(2.5, wire_color)));

        // Arrowhead at endpoint
        if screen_pts.len() >= 2 {
            let ep = screen_pts[screen_pts.len() - 1];
            let prev = screen_pts[screen_pts.len() - 2];
            let dir = (ep - prev).normalized();
            let perp = Vec2::new(-dir.y, dir.x);
            painter.line_segment([ep, ep - dir * 8. + perp * 4.], Stroke::new(2., wire_color));
            painter.line_segment([ep, ep - dir * 8. - perp * 4.], Stroke::new(2., wire_color));
        }
    }

    // 2. Active Interactive Wire During Drag
    if let (Some(src_raw), Some(end_world)) = (wiring_source, wiring_end) {
        let (src_id, src_port_opt) = match src_raw.split_once('#') {
            Some((n, p)) => (n, Some(p)),
            None => (src_raw, None),
        };
        if let Some(src_node) = graph.nodes.get(src_id) {
            let (sx, sy) = src_port_opt
                .and_then(|p| port_world_position(src_node, p))
                .unwrap_or_else(|| node_output_port(src_node.position));
            let curve = CubicBezierCurve::from_endpoints((sx, sy), end_world);
            let samples = curve.sample_points(24);
            let screen_pts: Vec<Pos2> = samples
                .into_iter()
                .map(|(wx, wy)| viewport.world_to_screen(Pos2::new(wx, wy), canvas_origin))
                .collect();

            let active_color = Color32::from_rgb(180, 230, 255);
            painter.add(PathShape::line(screen_pts.clone(), Stroke::new(2.5, active_color)));
            if screen_pts.len() >= 2 {
                let ep = screen_pts[screen_pts.len() - 1];
                let prev = screen_pts[screen_pts.len() - 2];
                let dir = (ep - prev).normalized();
                let perp = Vec2::new(-dir.y, dir.x);
                painter.line_segment([ep, ep - dir * 8. + perp * 4.], Stroke::new(2., active_color));
                painter.line_segment([ep, ep - dir * 8. - perp * 4.], Stroke::new(2., active_color));
            }
        }
    }

    // 3. Scratch-Style Blocks & Port Sockets
    for node in graph.nodes.values() {
        let h = node_height(node);
        let screen_pos = viewport.world_to_screen(Pos2::new(node.position.0, node.position.1), canvas_origin);
        let node_rect = Rect::from_min_size(screen_pos, Vec2::new(NODE_W, h));
        let color = kind_color(&node.kind);

        painter.rect_filled(node_rect, 6., Color32::from_rgb(26, 28, 36));
        painter.rect_stroke(node_rect, 6., Stroke::new(1.5, color), StrokeKind::Outside);

        if selected_id == Some(node.id.as_str()) {
            painter.rect_stroke(node_rect, 6., Stroke::new(2.5, Color32::YELLOW), StrokeKind::Outside);
        }

        // Header bar
        let header_rect = Rect::from_min_size(node_rect.min, Vec2::new(NODE_W, HEADER_H));
        painter.rect_filled(header_rect, 6., color);
        painter.rect_filled(
            Rect::from_min_max(Pos2::new(node_rect.min.x, header_rect.max.y - 4.), node_rect.max),
            0,
            Color32::from_rgb(26, 28, 36),
        );

        let label = kind_label(&node.kind);
        painter.text(
            Pos2::new(header_rect.center().x, header_rect.center().y),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(10.5),
            Color32::from_rgb(10, 10, 15),
        );

        // Input Ports on Left Edge
        for p in &node.input_ports {
            if let Some((pw_x, pw_y)) = port_world_position(node, &p.id) {
                let p_screen = viewport.world_to_screen(Pos2::new(pw_x, pw_y), canvas_origin);
                let p_col = port_color(p.data_type);

                painter.circle_filled(p_screen, 5., p_col);
                painter.circle_stroke(p_screen, 5., Stroke::new(1.5, Color32::from_rgb(220, 220, 220)));

                painter.text(
                    Pos2::new(p_screen.x + 8., p_screen.y),
                    egui::Align2::LEFT_CENTER,
                    &p.label,
                    egui::FontId::proportional(9.),
                    Color32::from_rgb(180, 190, 205),
                );
            }
        }

        // Output Ports on Right Edge
        for p in &node.output_ports {
            if let Some((pw_x, pw_y)) = port_world_position(node, &p.id) {
                let p_screen = viewport.world_to_screen(Pos2::new(pw_x, pw_y), canvas_origin);
                let p_col = port_color(p.data_type);

                painter.circle_filled(p_screen, 5., p_col);
                painter.circle_stroke(p_screen, 5., Stroke::new(1.5, Color32::from_rgb(220, 220, 220)));

                painter.text(
                    Pos2::new(p_screen.x - 8., p_screen.y),
                    egui::Align2::RIGHT_CENTER,
                    &p.label,
                    egui::FontId::proportional(9.),
                    Color32::from_rgb(180, 190, 205),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_height_and_port_positions() {
        let cond_node = FlowNode::new(
            "c1",
            FlowNodeKind::Condition {
                field: "amount".into(),
                operator: ">".into(),
                target_value: "100".into(),
            },
            (100.0, 100.0),
        );
        let h = node_height(&cond_node);
        assert!(h >= NODE_H);

        let p_in = port_world_position(&cond_node, "exec_in").expect("exec_in port exists");
        assert_eq!(p_in.0, 100.0); // Left edge

        let p_true = port_world_position(&cond_node, "branch_true").expect("branch_true exists");
        assert_eq!(p_true.0, 100.0 + NODE_W); // Right edge

        let p_false = port_world_position(&cond_node, "branch_false").expect("branch_false exists");
        assert_eq!(p_false.0, 100.0 + NODE_W);
        assert!(p_false.1 > p_true.1); // Stacked vertically below true
    }

    #[test]
    fn test_port_color_mapping() {
        assert_ne!(port_color(PortDataType::ExecutionFlow), port_color(PortDataType::Boolean));
        assert_ne!(port_color(PortDataType::Record), port_color(PortDataType::Number));
    }
}
