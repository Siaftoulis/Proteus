//! Studio Model Context Protocol (MCP) Bridge & Live Dispatcher.
//! Enables bidirectional communication between Proteus Design Studio and local AI agents.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use proteus_core::mcp::{register_core_tools, CallToolResult, McpServer, Tool};
use serde_json::json;

use crate::scene::{self, CanvasEvent, Layout, Node, NodeStyle, NodeType, ProjectDocument, Sizing, Styling};

pub type PendingNodeItem = (Node, Option<String>);

/// Studio AI Bridge coordinating external MCP agent commands and internal scene mutations.
pub struct StudioAiBridge {
    server: McpServer,
    incoming: Arc<Mutex<VecDeque<String>>>,
    outgoing: Arc<Mutex<VecDeque<String>>>,
    pending_nodes: Arc<Mutex<Vec<PendingNodeItem>>>,
    pending_clear: Arc<Mutex<bool>>,
    doc_summary: Arc<Mutex<serde_json::Value>>,
}

impl Default for StudioAiBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl StudioAiBridge {
    pub fn new() -> Self {
        let mut server = McpServer::new("proteus-design-studio", "0.1.0");
        register_core_tools(&mut server);

        let incoming = Arc::new(Mutex::new(VecDeque::new()));
        let outgoing = Arc::new(Mutex::new(VecDeque::new()));
        let pending_nodes = Arc::new(Mutex::new(Vec::new()));
        let pending_clear = Arc::new(Mutex::new(false));
        let doc_summary = Arc::new(Mutex::new(json!({})));

        // ── Studio Tool 1: inject_canvas_nodes ──
        let p_nodes_clone = Arc::clone(&pending_nodes);
        let inject_tool = Tool {
            name: "inject_canvas_nodes".to_string(),
            description: "Injects declarative UI node trees (frames, inputs, buttons, tables) directly into the active Studio canvas.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "node": {
                        "type": "object",
                        "description": "Root node object with optional nested children array"
                    },
                    "parent_id": {
                        "type": "string",
                        "description": "Optional parent node ID to attach under"
                    }
                },
                "required": ["node"]
            }),
        };

        server.register_tool(inject_tool, move |args| {
            let root_val = args.get("node").unwrap_or(&serde_json::Value::Null);
            let parent_id = args.get("parent_id").and_then(|v| v.as_str()).map(|s| s.to_string());

            let mut flattened = Vec::new();
            flatten_node_tree(root_val, parent_id.as_deref(), &mut flattened);

            let count = flattened.len();
            if count > 0 {
                let mut queue = p_nodes_clone.lock().unwrap();
                queue.extend(flattened);
                Ok(CallToolResult::text(format!("Queued {} node(s) for live canvas injection", count)))
            } else {
                Ok(CallToolResult::error("Failed to parse node definition"))
            }
        });

        // ── Studio Tool 2: get_active_canvas_summary ──
        let summary_clone = Arc::clone(&doc_summary);
        let summary_tool = Tool {
            name: "get_active_canvas_summary".to_string(),
            description: "Retrieves live summary of current canvas nodes, hierarchy, and selected items.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        };

        server.register_tool(summary_tool, move |_| {
            let summary = summary_clone.lock().unwrap().clone();
            Ok(CallToolResult::text(serde_json::to_string_pretty(&summary).unwrap_or_default()))
        });

        // ── Studio Tool 3: clear_canvas ──
        let clear_clone = Arc::clone(&pending_clear);
        let clear_tool = Tool {
            name: "clear_canvas".to_string(),
            description: "Clears all nodes from the active canvas document.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
        };

        server.register_tool(clear_tool, move |_| {
            *clear_clone.lock().unwrap() = true;
            Ok(CallToolResult::text("Canvas cleared successfully"))
        });

        Self {
            server,
            incoming,
            outgoing,
            pending_nodes,
            pending_clear,
            doc_summary,
        }
    }

    /// Enqueues an incoming raw JSON-RPC string line (from stdio, socket, or worker).
    pub fn enqueue_request(&self, line: &str) {
        if let Ok(mut q) = self.incoming.lock() {
            q.push_back(line.to_string());
        }
    }

    /// Pops a processed JSON-RPC response from the outgoing queue.
    pub fn pop_response(&self) -> Option<String> {
        self.outgoing.lock().ok()?.pop_front()
    }

    /// Non-blocking poll invoked during egui frame update.
    /// Drains incoming requests, updates canvas document, and queues responses.
    pub fn poll_and_dispatch(
        &mut self,
        doc: &mut ProjectDocument,
        selected_node: Option<&str>,
    ) -> Vec<CanvasEvent> {
        let mut events = Vec::new();

        // 1. Refresh live canvas summary cache for MCP readers
        if let Ok(mut sum) = self.doc_summary.lock() {
            let brief_nodes: Vec<serde_json::Value> = doc
                .nodes
                .values()
                .map(|n| {
                    json!({
                        "id": n.id,
                        "name": n.name,
                        "position": [n.position.0, n.position.1],
                        "parent_id": n.parent_id
                    })
                })
                .collect();

            *sum = json!({
                "total_nodes": doc.nodes.len(),
                "root_node_ids": doc.root_node_ids,
                "selected_node": selected_node,
                "nodes": brief_nodes
            });
        }

        // 2. Process incoming JSON-RPC lines
        let requests: Vec<String> = {
            let mut in_q = match self.incoming.lock() {
                Ok(q) => q,
                Err(_) => return events,
            };
            in_q.drain(..).collect()
        };

        for raw_line in requests {
            if let Some(resp_line) = self.server.handle_raw_json(&raw_line) {
                if let Ok(mut out_q) = self.outgoing.lock() {
                    out_q.push_back(resp_line);
                }
            }
        }

        // 3. Apply canvas clear if triggered
        if let Ok(mut clr) = self.pending_clear.lock() {
            if *clr {
                doc.nodes.clear();
                doc.root_node_ids.clear();
                *clr = false;
                events.push(CanvasEvent::ClearSelection);
            }
        }

        // 4. Inject staged nodes into the active document
        let staged = {
            let mut p_nodes = match self.pending_nodes.lock() {
                Ok(n) => n,
                Err(_) => return events,
            };
            p_nodes.drain(..).collect::<Vec<_>>()
        };

        for (node, parent_id) in staged {
            let nid = node.id.clone();
            let _ = doc.add_node(node, parent_id);
            events.push(CanvasEvent::NodeClicked { id: nid, shift_held: false });
        }

        events
    }

    /// Evaluates a single raw string synchronously (useful for integration tests & scripts).
    pub fn process_single_message(
        &mut self,
        raw: &str,
        doc: &mut ProjectDocument,
        selected_node: Option<&str>,
    ) -> Option<String> {
        self.enqueue_request(raw);
        self.poll_and_dispatch(doc, selected_node);
        self.pop_response()
    }
}

/// Recursively flattens a nested JSON node tree into `(Node, parent_id)` tuples.
fn flatten_node_tree(
    val: &serde_json::Value,
    parent_id: Option<&str>,
    out: &mut Vec<(Node, Option<String>)>,
) {
    if let Some(node) = parse_node_json(val) {
        let node_id = node.id.clone();
        out.push((node, parent_id.map(|s| s.to_string())));

        if let Some(children) = val.get("children").and_then(|c| c.as_array()) {
            for child_val in children {
                flatten_node_tree(child_val, Some(&node_id), out);
            }
        }
    }
}

/// Converts a declarative JSON node representation into an egui-compatible `Node`.
pub fn parse_node_json(val: &serde_json::Value) -> Option<Node> {
    let id = val.get("id")?.as_str()?.to_string();
    let name = val.get("name").and_then(|v| v.as_str()).unwrap_or(&id).to_string();
    let type_str = val.get("type").and_then(|v| v.as_str()).unwrap_or("Frame");

    let pos_arr = val.get("position").and_then(|v| v.as_array());
    let x = pos_arr.and_then(|a| a.first()?.as_f64()).unwrap_or(0.0) as f32;
    let y = pos_arr.and_then(|a| a.get(1)?.as_f64()).unwrap_or(0.0) as f32;

    let size_arr = val.get("size").and_then(|v| v.as_array());
    let w = size_arr.and_then(|a| a.first()?.as_f64()).unwrap_or(200.0) as f32;
    let h = size_arr.and_then(|a| a.get(1)?.as_f64()).unwrap_or(100.0) as f32;

    let props = val.get("props");
    let node_type = match type_str {
        "Text" => {
            let content = props
                .and_then(|p| p.get("content"))
                .and_then(|v| v.as_str())
                .unwrap_or(&name)
                .to_string();
            let size = props
                .and_then(|p| p.get("font_size"))
                .and_then(|v| v.as_f64())
                .unwrap_or(14.0) as f32;
            let weight = props
                .and_then(|p| p.get("font_weight"))
                .and_then(|v| v.as_u64())
                .unwrap_or(400) as u16;

            NodeType::Text {
                content,
                font: scene::FontSpec {
                    family: "Inter".to_string(),
                    size,
                    weight,
                    color: scene::Rgba::WHITE,
                },
            }
        }
        "TextInput" => {
            let placeholder = props
                .and_then(|p| p.get("placeholder"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let bound_entity = props
                .and_then(|p| p.get("bound_entity"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let bound_field = props
                .and_then(|p| p.get("bound_field"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            NodeType::TextInput {
                placeholder,
                field_type: scene::FieldType::Text,
                bound_entity,
                bound_field,
            }
        }
        "Button" => {
            let label = props
                .and_then(|p| p.get("label"))
                .and_then(|v| v.as_str())
                .unwrap_or("Submit")
                .to_string();

            NodeType::Button {
                label,
                style: scene::ButtonStyle::Primary,
            }
        }
        "Table" => {
            let bound_entity = props
                .and_then(|p| p.get("bound_entity"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let columns = props
                .and_then(|p| p.get("columns"))
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|s| s.as_str().map(|str| str.to_string())).collect())
                .unwrap_or_default();

            NodeType::Table { bound_entity, columns }
        }
        "Shape" => NodeType::Shape { kind: scene::ShapeKind::Rectangle },
        _ => NodeType::Frame,
    };

    let mut node = Node::new(id, name, node_type);
    node.position = (x, y);
    node.layout = Layout {
        width: Sizing::Fixed(w),
        height: Sizing::Fixed(h),
        ..Default::default()
    };
    node.styling = Styling {
        background: Some(scene::Rgba { r: 24, g: 27, b: 34, a: 255 }),
        corner_radius: [6.0, 6.0, 6.0, 6.0],
        border: Some(scene::Border { width: 1.0, color: scene::Rgba { r: 56, g: 65, b: 82, a: 255 } }),
        padding: [12.0, 12.0, 12.0, 12.0],
        shadow: None,
        opacity: 1.0,
    };
    node.style = NodeStyle::default();

    Some(node)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_studio_ai_bridge_tools_list() {
        let mut bridge = StudioAiBridge::new();
        let mut doc = ProjectDocument::new();

        let req = r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#;
        let resp_str = bridge.process_single_message(req, &mut doc, None).expect("Expected response");

        assert!(resp_str.contains("inject_canvas_nodes"));
        assert!(resp_str.contains("get_active_canvas_summary"));
        assert!(resp_str.contains("clear_canvas"));
        assert!(resp_str.contains("generate_schema_from_prompt"));
    }

    #[test]
    fn test_studio_ai_bridge_inject_and_summary() {
        let mut bridge = StudioAiBridge::new();
        let mut doc = ProjectDocument::new();

        // 1. Inject node tree with Frame and Child Button
        let inject_req = r#"{
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": {
                "name": "inject_canvas_nodes",
                "arguments": {
                    "node": {
                        "id": "ai-form-card",
                        "name": "Customer Card",
                        "type": "Frame",
                        "position": [50.0, 50.0],
                        "size": [400.0, 300.0],
                        "children": [
                            {
                                "id": "ai-btn-ok",
                                "name": "OK Button",
                                "type": "Button",
                                "position": [20.0, 20.0],
                                "size": [120.0, 36.0],
                                "props": { "label": "Save Customer ✓" }
                            }
                        ]
                    }
                }
            }
        }"#;

        let resp_str = bridge.process_single_message(inject_req, &mut doc, None).expect("Expected response");
        assert!(resp_str.contains("live canvas injection"));
        assert_eq!(doc.nodes.len(), 2);
        assert!(doc.get_node("ai-form-card").is_some());
        assert!(doc.get_node("ai-btn-ok").is_some());

        // 2. Query canvas summary
        let summary_req = r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"get_active_canvas_summary","arguments":{}}}"#;
        let summary_resp = bridge.process_single_message(summary_req, &mut doc, Some("ai-form-card")).expect("Expected response");
        assert!(summary_resp.contains("Customer Card"));
        assert!(summary_resp.contains("ai-btn-ok"));
    }

    #[test]
    fn test_studio_ai_bridge_clear_canvas() {
        let mut bridge = StudioAiBridge::new();
        let mut doc = ProjectDocument::new();
        let _ = doc.add_node(Node::new("n1".into(), "N1".into(), NodeType::Frame), None);
        assert_eq!(doc.nodes.len(), 1);

        let clear_req = r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"clear_canvas","arguments":{}}}"#;
        let resp = bridge.process_single_message(clear_req, &mut doc, None).expect("Expected response");
        assert!(resp.contains("Canvas cleared successfully"));
        assert_eq!(doc.nodes.len(), 0);
    }
}
