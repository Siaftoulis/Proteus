use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum FlowNodeKind {
    TriggerClick { target_node_id: String },
    NavigateTo { page_id: String },
    SaveToDatabase { entity: String },
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
