// ponytail: minimal synchronous DAG walker for flow execution
// upgrade: async, retry, parallel branches, condition evaluation
use crate::Database;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlowNode {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub position: Option<serde_json::Value>,
    pub data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlowEdge {
    pub id: String,
    pub source: String,
    pub target: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlowGraph {
    pub nodes: Vec<FlowNode>,
    pub edges: Vec<FlowEdge>,
}

fn find_trigger(nodes: &[FlowNode]) -> Option<&FlowNode> {
    nodes.iter().find(|n| n.node_type == "trigger")
}

impl Database {
    pub fn execute_flow(&self, project_id: &str, _widget_id: &str, flows_json: &str) -> Result<String, String> {
        let graph: FlowGraph = serde_json::from_str(flows_json)
            .map_err(|e| format!("Invalid flow JSON: {}", e))?;
        let trigger = find_trigger(&graph.nodes)
            .ok_or_else(|| "No trigger node found in flow".to_string())?;

        // walk the DAG: find edges from trigger, then follow actions
        let mut results = Vec::new();
        let mut current = vec![trigger.id.clone()];
        let mut visited = std::collections::HashSet::new();

        while let Some(node_id) = current.pop() {
            if !visited.insert(node_id.clone()) { continue; }
            if let Some(node) = graph.nodes.iter().find(|n| n.id == node_id) {
                match node.node_type.as_str() {
                    "trigger" => {
                        // pass through to connected nodes
                        for edge in &graph.edges {
                            if edge.source == node_id { current.push(edge.target.clone()); }
                        }
                    }
                    "action" => {
                        let label = node.data.get("label").and_then(|v| v.as_str()).unwrap_or("action");
                        match label {
                            "Create Record" | "create_record" => {
                                let entity = node.data.get("entity").and_then(|v| v.as_str()).unwrap_or("contact");
                                let fields = node.data.get("fields").and_then(|v| v.as_str()).unwrap_or("{}");
                                self.create_record(project_id, entity, fields)
                                    .map_err(|e| format!("Create record failed: {}", e))?;
                                results.push(format!("Created {} record", entity));
                            }
                            _ => results.push(format!("Executed: {}", label)),
                        }
                        // continue to connected nodes
                        for edge in &graph.edges {
                            if edge.source == node_id { current.push(edge.target.clone()); }
                        }
                    }
                    "condition" | "logicalGate" => {
                        // ponytail: skip conditions — always follow first outgoing edge
                        // upgrade: evaluate condition expressions
                        if let Some(edge) = graph.edges.iter().find(|e| e.source == node_id) {
                            current.push(edge.target.clone());
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(results.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use crate::Database;

    fn test_db() -> Database {
        Database::new(":memory:").expect("in-memory db")
    }

    #[test]
    fn test_execute_simple_flow() {
        let db = test_db();
        let project = db.create_project("test").unwrap();
        let flows = r#"{
            "nodes": [
                {"id": "t1", "type": "trigger", "position": {"x": 0, "y": 0}, "data": {"label": "On Button Click"}},
                {"id": "a1", "type": "action", "position": {"x": 200, "y": 0}, "data": {"label": "Create Record", "entity": "contact", "fields": "{\"source\":\"flow\"}"}}
            ],
            "edges": [
                {"id": "e1", "source": "t1", "target": "a1"}
            ]
        }"#;
        let result = db.execute_flow(&project.id, "w1", flows).unwrap();
        assert!(result.contains("Created contact record"));
        let records = db.list_records(&project.id, "contact").unwrap();
        assert_eq!(records.len(), 1);
        assert!(records[0].fields.contains("flow"));
    }

    #[test]
    fn test_execute_flow_no_trigger_fails() {
        let db = test_db();
        let flows = r#"{"nodes": [], "edges": []}"#;
        let result = db.execute_flow("p1", "w1", flows);
        assert!(result.is_err());
    }

    #[test]
    fn test_execute_flow_chain() {
        let db = test_db();
        let project = db.create_project("chain").unwrap();
        let flows = r#"{
            "nodes": [
                {"id": "t1", "type": "trigger", "data": {"label": "Click"}},
                {"id": "a1", "type": "action", "data": {"label": "Create Record", "entity": "lead", "fields": "{}"}},
                {"id": "a2", "type": "action", "data": {"label": "create_record", "entity": "lead", "fields": "{}"}}
            ],
            "edges": [
                {"id": "e1", "source": "t1", "target": "a1"},
                {"id": "e2", "source": "a1", "target": "a2"}
            ]
        }"#;
        db.execute_flow(&project.id, "w1", flows).unwrap();
        assert_eq!(db.list_records(&project.id, "lead").unwrap().len(), 2);
    }
}
