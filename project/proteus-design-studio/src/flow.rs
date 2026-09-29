use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum FlowNodeKind {
    TriggerClick { target_node_id: String },
    NavigateTo { page_id: String },
    SaveToDatabase { entity: String },
    Condition {
        field: String,
        operator: String,
        target_value: String,
    },
    ShowToast { message: String },
    FederationBridge {
        endpoint: String,
        source_entity: String,
        target_entity: String,
        bidirectional: bool,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FlowNode {
    pub id: String,
    pub kind: FlowNodeKind,
    pub position: (f32, f32),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FlowEdge {
    pub from_node: String,
    pub to_node: String,
    #[serde(default)]
    pub branch: Option<String>,
}

impl FlowEdge {
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from_node: from.into(),
            to_node: to.into(),
            branch: None,
        }
    }

    pub fn with_branch(from: impl Into<String>, to: impl Into<String>, branch: impl Into<String>) -> Self {
        Self {
            from_node: from.into(),
            to_node: to.into(),
            branch: Some(branch.into()),
        }
    }
}

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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flow_graph_node_operations() {
        let mut graph = FlowGraph::new();
        let node1 = FlowNode {
            id: "n1".into(),
            kind: FlowNodeKind::TriggerClick { target_node_id: "btn-1".into() },
            position: (0.0, 0.0),
        };
        let node2 = FlowNode {
            id: "n2".into(),
            kind: FlowNodeKind::Condition {
                field: "amount".into(),
                operator: ">".into(),
                target_value: "100".into(),
            },
            position: (200.0, 0.0),
        };
        let node3 = FlowNode {
            id: "n3".into(),
            kind: FlowNodeKind::ShowToast {
                message: "High value deal!".into(),
            },
            position: (400.0, 0.0),
        };
        let node4 = FlowNode {
            id: "n4".into(),
            kind: FlowNodeKind::FederationBridge {
                endpoint: "http://partner:7443".into(),
                source_entity: "products".into(),
                target_entity: "inventory".into(),
                bidirectional: true,
            },
            position: (600.0, 0.0),
        };
        graph.nodes.insert("n1".into(), node1);
        graph.nodes.insert("n2".into(), node2);
        graph.nodes.insert("n3".into(), node3);
        graph.nodes.insert("n4".into(), node4);
        graph.edges.push(FlowEdge { from_node: "n1".into(), to_node: "n2".into(), branch: None });
        graph.edges.push(FlowEdge { from_node: "n2".into(), to_node: "n3".into(), branch: Some("true".into()) });
        graph.edges.push(FlowEdge { from_node: "n3".into(), to_node: "n4".into(), branch: None });

        assert_eq!(graph.nodes.len(), 4);
        assert_eq!(graph.edges.len(), 3);

        // Serialize and deserialize roundtrip
        let json = serde_json::to_string(&graph).expect("serialize");
        let restored: FlowGraph = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(restored.nodes.len(), 4);
        assert_eq!(restored.edges.len(), 3);

        // Remove condition node
        graph.remove_node("n2");
        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(graph.edges.len(), 1); // Only n3 -> n4 remains
    }
}

