// Minimal synchronous DAG walker for flow execution with conditional branching
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
    #[serde(default)]
    pub source_handle: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlowGraph {
    pub nodes: Vec<FlowNode>,
    pub edges: Vec<FlowEdge>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConditionOp {
    Equals,
    NotEquals,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Contains,
    IsEmpty,
}

impl ConditionOp {
    pub fn parse(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "==" | "eq" | "equals" => Self::Equals,
            "!=" | "neq" | "not_equals" => Self::NotEquals,
            ">" | "gt" | "greater_than" => Self::GreaterThan,
            ">=" | "gte" | "greater_than_or_equal" => Self::GreaterThanOrEqual,
            "<" | "lt" | "less_than" => Self::LessThan,
            "<=" | "lte" | "less_than_or_equal" => Self::LessThanOrEqual,
            "contains" | "includes" => Self::Contains,
            "empty" | "is_empty" => Self::IsEmpty,
            _ => Self::Equals,
        }
    }

    pub fn evaluate(&self, actual: Option<&serde_json::Value>, expected: Option<&serde_json::Value>) -> bool {
        match self {
            Self::IsEmpty => match actual {
                None | Some(serde_json::Value::Null) => true,
                Some(serde_json::Value::String(s)) => s.trim().is_empty(),
                Some(serde_json::Value::Array(a)) => a.is_empty(),
                Some(serde_json::Value::Object(m)) => m.is_empty(),
                _ => false,
            },
            Self::Equals => match (actual, expected) {
                (Some(a), Some(b)) => {
                    if let (Some(na), Some(nb)) = (to_f64(a), to_f64(b)) {
                        (na - nb).abs() < 1e-9
                    } else if let (Some(sa), Some(sb)) = (a.as_str(), b.as_str()) {
                        sa == sb
                    } else if let (Some(ba), Some(bb)) = (a.as_bool(), b.as_bool()) {
                        ba == bb
                    } else {
                        a == b
                    }
                }
                _ => actual == expected,
            },
            Self::NotEquals => !Self::Equals.evaluate(actual, expected),
            Self::GreaterThan => match (actual.and_then(to_f64), expected.and_then(to_f64)) {
                (Some(a), Some(b)) => a > b,
                _ => false,
            },
            Self::GreaterThanOrEqual => match (actual.and_then(to_f64), expected.and_then(to_f64)) {
                (Some(a), Some(b)) => a >= b,
                _ => false,
            },
            Self::LessThan => match (actual.and_then(to_f64), expected.and_then(to_f64)) {
                (Some(a), Some(b)) => a < b,
                _ => false,
            },
            Self::LessThanOrEqual => match (actual.and_then(to_f64), expected.and_then(to_f64)) {
                (Some(a), Some(b)) => a <= b,
                _ => false,
            },
            Self::Contains => match (
                actual.and_then(|v| v.as_str()).map(|s| s.to_lowercase()),
                expected.and_then(|v| v.as_str()).map(|s| s.to_lowercase()),
            ) {
                (Some(a), Some(b)) => a.contains(&b),
                _ => false,
            },
        }
    }
}

fn to_f64(v: &serde_json::Value) -> Option<f64> {
    v.as_f64().or_else(|| v.as_str().and_then(|s| s.trim().parse::<f64>().ok()))
}

fn find_trigger(nodes: &[FlowNode]) -> Option<&FlowNode> {
    nodes.iter().find(|n| n.node_type == "trigger")
}

fn evaluate_node_condition(node: &FlowNode, payload: &serde_json::Value) -> bool {
    let field = node.data.get("field").and_then(|v| v.as_str()).unwrap_or("");
    let op_str = node.data.get("operator")
        .or_else(|| node.data.get("op"))
        .and_then(|v| v.as_str())
        .unwrap_or("==");
    let op = ConditionOp::parse(op_str);
    let expected = node.data.get("value").or_else(|| node.data.get("expected"));
    let actual = if field.is_empty() { Some(payload) } else { payload.get(field) };
    op.evaluate(actual, expected)
}

impl Database {
    pub fn execute_flow(&self, project_id: &str, widget_id: &str, flows_json: &str) -> Result<String, String> {
        self.execute_flow_with_context(project_id, widget_id, flows_json, &serde_json::json!({}))
    }

    pub fn execute_flow_with_context(
        &self,
        project_id: &str,
        _widget_id: &str,
        flows_json: &str,
        context: &serde_json::Value,
    ) -> Result<String, String> {
        let graph: FlowGraph = serde_json::from_str(flows_json)
            .map_err(|e| format!("Invalid flow JSON: {}", e))?;
        let trigger = find_trigger(&graph.nodes)
            .ok_or_else(|| "No trigger node found in flow".to_string())?;

        let mut results = Vec::new();
        let mut current = vec![trigger.id.clone()];
        let mut visited = std::collections::HashSet::new();
        let payload = context.clone();

        while let Some(node_id) = current.pop() {
            if !visited.insert(node_id.clone()) {
                continue;
            }
            if let Some(node) = graph.nodes.iter().find(|n| n.id == node_id) {
                match node.node_type.as_str() {
                    "trigger" => {
                        for edge in &graph.edges {
                            if edge.source == node_id {
                                current.push(edge.target.clone());
                            }
                        }
                    }
                    "action" => {
                        let label = node
                            .data
                            .get("label")
                            .and_then(|v| v.as_str())
                            .unwrap_or("action");
                        match label {
                            "Create Record" | "create_record" => {
                                let entity = node
                                    .data
                                    .get("entity")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("contact");
                                let fields_str = node
                                    .data
                                    .get("fields")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("{}");
                                let final_fields = if fields_str.trim() == "{}" && payload.is_object() {
                                    serde_json::to_string(&payload).unwrap_or_else(|_| "{}".into())
                                } else {
                                    fields_str.to_string()
                                };
                                self.create_record(project_id, entity, &final_fields)
                                    .map_err(|e| format!("Create record failed: {}", e))?;
                                results.push(format!("Created {} record", entity));
                            }
                            "Update Record" | "update_record" => {
                                let entity = node
                                    .data
                                    .get("entity")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("contact");
                                let id = node
                                    .data
                                    .get("record_id")
                                    .and_then(|v| v.as_str())
                                    .or_else(|| payload.get("id").and_then(|v| v.as_str()))
                                    .unwrap_or("");
                                let fields = node
                                    .data
                                    .get("fields")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("{}");
                                if !id.is_empty() {
                                    self.update_record(id, fields)
                                        .map_err(|e| format!("Update record failed: {}", e))?;
                                    results.push(format!(
                                        "Updated {} record (#{})",
                                        entity,
                                        &id[..6.min(id.len())]
                                    ));
                                }
                            }
                            "Notification" | "Toast" | "show_toast" | "send_notification" => {
                                let msg = node
                                    .data
                                    .get("message")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("Flow notification");
                                results.push(format!("Notification: {}", msg));
                            }
                            _ => results.push(format!("Executed: {}", label)),
                        }
                        for edge in &graph.edges {
                            if edge.source == node_id {
                                current.push(edge.target.clone());
                            }
                        }
                    }
                    "condition" | "logicalGate" => {
                        let label = node
                            .data
                            .get("label")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Condition");
                        let is_true = evaluate_node_condition(node, &payload);
                        results.push(format!(
                            "{}: {}",
                            label,
                            if is_true { "true" } else { "false" }
                        ));

                        let outgoing: Vec<&FlowEdge> =
                            graph.edges.iter().filter(|e| e.source == node_id).collect();
                        if outgoing.is_empty() {
                            continue;
                        }

                        let is_match = |edge: &&FlowEdge, target_val: &str| -> bool {
                            if let Some(ref h) = edge.source_handle {
                                if h.eq_ignore_ascii_case(target_val) {
                                    return true;
                                }
                            }
                            if let Some(ref l) = edge.label {
                                if l.eq_ignore_ascii_case(target_val) {
                                    return true;
                                }
                            }
                            false
                        };

                        if is_true {
                            if let Some(tgt) = node.data.get("true_target").and_then(|v| v.as_str()) {
                                current.push(tgt.to_string());
                            } else if let Some(edge) = outgoing
                                .iter()
                                .find(|e| is_match(e, "true") || is_match(e, "yes") || is_match(e, "pass"))
                            {
                                current.push(edge.target.clone());
                            } else {
                                current.push(outgoing[0].target.clone());
                            }
                        } else if let Some(tgt) = node.data.get("false_target").and_then(|v| v.as_str()) {
                            current.push(tgt.to_string());
                        } else if let Some(edge) = outgoing
                            .iter()
                            .find(|e| is_match(e, "false") || is_match(e, "no") || is_match(e, "fail"))
                        {
                            current.push(edge.target.clone());
                        } else if outgoing.len() >= 2 {
                            current.push(outgoing[1].target.clone());
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
        let flows = r#"{"nodes":[{"id":"t1","type":"trigger","data":{"label":"Click"}},{"id":"a1","type":"action","data":{"label":"Create Record","entity":"lead","fields":"{}"}},{"id":"a2","type":"action","data":{"label":"create_record","entity":"lead","fields":"{}"}}],"edges":[{"id":"e1","source":"t1","target":"a1"},{"id":"e2","source":"a1","target":"a2"}]}"#;
        db.execute_flow(&project.id, "w1", flows).unwrap();
        assert_eq!(db.list_records(&project.id, "lead").unwrap().len(), 2);
    }

    #[test]
    fn test_execute_flow_condition_branching_true() {
        let db = test_db();
        let project = db.create_project("branch_true").unwrap();
        let flows = r#"{"nodes":[{"id":"t1","type":"trigger","data":{"label":"Click"}},{"id":"c1","type":"condition","data":{"label":"Check Amount","field":"amount","op":">","value":100}},{"id":"a1","type":"action","data":{"label":"Create Record","entity":"vip_deal","fields":"{\"status\":\"approved\"}"}},{"id":"a2","type":"action","data":{"label":"Create Record","entity":"standard_deal","fields":"{\"status\":\"standard\"}"}}],"edges":[{"id":"e1","source":"t1","target":"c1"},{"id":"e2","source":"c1","target":"a1","source_handle":"true"},{"id":"e3","source":"c1","target":"a2","source_handle":"false"}]}"#;
        let ctx = serde_json::json!({ "amount": 250 });
        let result = db.execute_flow_with_context(&project.id, "w1", flows, &ctx).unwrap();
        assert!(result.contains("Check Amount: true"));
        assert!(result.contains("Created vip_deal record"));
        assert!(!result.contains("Created standard_deal record"));

        let vip = db.list_records(&project.id, "vip_deal").unwrap();
        assert_eq!(vip.len(), 1);
        let standard = db.list_records(&project.id, "standard_deal").unwrap();
        assert_eq!(standard.len(), 0);
    }

    #[test]
    fn test_execute_flow_condition_branching_false() {
        let db = test_db();
        let project = db.create_project("branch_false").unwrap();
        let flows = r#"{"nodes":[{"id":"t1","type":"trigger","data":{"label":"Click"}},{"id":"c1","type":"condition","data":{"label":"Check Amount","field":"amount","op":">","value":100}},{"id":"a1","type":"action","data":{"label":"Create Record","entity":"vip_deal","fields":"{}"}},{"id":"a2","type":"action","data":{"label":"Create Record","entity":"standard_deal","fields":"{}"}}],"edges":[{"id":"e1","source":"t1","target":"c1"},{"id":"e2","source":"c1","target":"a1","source_handle":"true"},{"id":"e3","source":"c1","target":"a2","source_handle":"false"}]}"#;
        let ctx = serde_json::json!({ "amount": 50 });
        let result = db.execute_flow_with_context(&project.id, "w1", flows, &ctx).unwrap();
        assert!(result.contains("Check Amount: false"));
        assert!(result.contains("Created standard_deal record"));
        assert!(!result.contains("Created vip_deal record"));

        let vip = db.list_records(&project.id, "vip_deal").unwrap();
        assert_eq!(vip.len(), 0);
        let standard = db.list_records(&project.id, "standard_deal").unwrap();
        assert_eq!(standard.len(), 1);
    }

    #[test]
    fn test_execute_flow_condition_string_contains() {
        let db = test_db();
        let project = db.create_project("contains").unwrap();
        let flows = r#"{"nodes":[{"id":"t1","type":"trigger","data":{"label":"Click"}},{"id":"c1","type":"condition","data":{"label":"Check Email","field":"email","op":"contains","value":"@enterprise.com"}},{"id":"a1","type":"action","data":{"label":"Notification","message":"Enterprise Account Detected"}}],"edges":[{"id":"e1","source":"t1","target":"c1"},{"id":"e2","source":"c1","target":"a1"}]}"#;
        let ctx = serde_json::json!({ "email": "director@enterprise.com" });
        let result = db.execute_flow_with_context(&project.id, "w1", flows, &ctx).unwrap();
        assert!(result.contains("Enterprise Account Detected"));
    }
}

