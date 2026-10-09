pub mod bezier;
pub mod data_blocks;
pub mod hardware_nodes;
pub mod nodes;
pub mod ports;

pub use bezier::*;
pub use data_blocks::*;
pub use hardware_nodes::*;
pub use nodes::*;
pub use ports::*;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FlowGraph {
    pub nodes: std::collections::HashMap<String, FlowNode>,
    pub edges: Vec<FlowEdge>,
}

impl Default for FlowGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl FlowGraph {
    pub fn new() -> Self {
        Self {
            nodes: std::collections::HashMap::new(),
            edges: Vec::new(),
        }
    }

    pub fn remove_node(&mut self, id: &str) {
        self.nodes.remove(id);
        self.edges.retain(|e| e.from_node != id && e.to_node != id);
    }

    pub fn can_connect(
        &self,
        from_node: &str,
        from_port: &str,
        to_node: &str,
        to_port: &str,
    ) -> Result<(), &'static str> {
        if from_node == to_node {
            return Err("Cannot connect a node to itself");
        }
        let src = self.nodes.get(from_node).ok_or("Source node not found")?;
        let tgt = self.nodes.get(to_node).ok_or("Target node not found")?;

        let src_p = src.find_port(from_port).ok_or("Source port not found")?;
        let tgt_p = tgt.find_port(to_port).ok_or("Target port not found")?;

        if src_p.direction != PortDirection::Output {
            return Err("Source port must be an Output");
        }
        if tgt_p.direction != PortDirection::Input {
            return Err("Target port must be an Input");
        }
        if !src_p.data_type.is_compatible_with(&tgt_p.data_type) {
            return Err("Port data types are incompatible");
        }
        if self.edges.iter().any(|e| {
            e.from_node == from_node
                && e.from_port == from_port
                && e.to_node == to_node
                && e.to_port == to_port
        }) {
            return Err("Edge already exists");
        }
        Ok(())
    }

    pub fn add_edge(&mut self, edge: FlowEdge) -> Result<(), &'static str> {
        self.can_connect(&edge.from_node, &edge.from_port, &edge.to_node, &edge.to_port)?;
        self.edges.push(edge);
        Ok(())
    }

    pub fn incoming_edges(&self, node_id: &str) -> Vec<&FlowEdge> {
        self.edges.iter().filter(|e| e.to_node == node_id).collect()
    }

    pub fn outgoing_edges(&self, node_id: &str) -> Vec<&FlowEdge> {
        self.edges.iter().filter(|e| e.from_node == node_id).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_ports_and_data_type_compatibility() {
        let (in_p, out_p) = default_ports_for_kind(&FlowNodeKind::Condition {
            field: "total".into(),
            operator: ">".into(),
            target_value: "100".into(),
        });
        assert_eq!(in_p.len(), 1);
        assert_eq!(in_p[0].id, "exec_in");
        assert_eq!(out_p.len(), 2);
        assert_eq!(out_p[0].id, "branch_true");
        assert_eq!(out_p[1].id, "branch_false");

        assert!(PortDataType::ExecutionFlow.is_compatible_with(&PortDataType::ExecutionFlow));
        assert!(PortDataType::Record.is_compatible_with(&PortDataType::Any));
        assert!(!PortDataType::Record.is_compatible_with(&PortDataType::Number));
    }

    #[test]
    fn test_cubic_bezier_sampling_and_distance() {
        let curve = CubicBezierCurve::from_endpoints((0.0, 0.0), (100.0, 100.0));
        let start = curve.sample(0.0);
        let mid = curve.sample(0.5);
        let end = curve.sample(1.0);

        assert!((start.0 - 0.0).abs() < 1e-4);
        assert!((start.1 - 0.0).abs() < 1e-4);
        assert!((end.0 - 100.0).abs() < 1e-4);
        assert!((end.1 - 100.0).abs() < 1e-4);
        assert!(mid.0 > 20.0 && mid.0 < 80.0);

        let dist = curve.distance_to_point((0.0, 0.0), 10);
        assert!(dist < 1e-2);
    }

    #[test]
    fn test_flow_graph_node_operations_and_wire_validation() {
        let mut graph = FlowGraph::new();
        let node1 = FlowNode::new(
            "n1",
            FlowNodeKind::TriggerClick { target_node_id: "btn-1".into() },
            (0.0, 0.0),
        );
        let node2 = FlowNode::new(
            "n2",
            FlowNodeKind::Condition {
                field: "amount".into(),
                operator: ">".into(),
                target_value: "100".into(),
            },
            (200.0, 0.0),
        );
        let node3 = FlowNode::new(
            "n3",
            FlowNodeKind::ShowToast { message: "High value deal!".into() },
            (400.0, 0.0),
        );

        graph.nodes.insert("n1".into(), node1);
        graph.nodes.insert("n2".into(), node2);
        graph.nodes.insert("n3".into(), node3);

        // Valid connections
        assert!(graph.can_connect("n1", "exec_out", "n2", "exec_in").is_ok());
        graph.add_edge(FlowEdge::with_ports("n1", "exec_out", "n2", "exec_in")).unwrap();

        assert!(graph.can_connect("n2", "branch_true", "n3", "exec_in").is_ok());
        graph.add_edge(FlowEdge::with_ports("n2", "branch_true", "n3", "exec_in")).unwrap();

        // Self-connection rejected
        assert_eq!(
            graph.can_connect("n1", "exec_out", "n1", "exec_in"),
            Err("Cannot connect a node to itself")
        );

        // Direction reversal rejected (Input to Output)
        assert_eq!(
            graph.can_connect("n2", "exec_in", "n1", "exec_out"),
            Err("Source port must be an Output")
        );

        // Serialize and deserialize roundtrip
        let json = serde_json::to_string(&graph).expect("serialize");
        let restored: FlowGraph = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.nodes.len(), 3);
        assert_eq!(restored.edges.len(), 2);

        // Remove node cascades
        graph.remove_node("n2");
        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 0);
    }
}
