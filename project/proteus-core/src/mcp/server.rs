//! Sovereign Model Context Protocol (MCP) Server Engine.
//! Handles JSON-RPC 2.0 dispatch, capability negotiation, tool invocations, and resource streams.

use std::collections::HashMap;
use std::sync::Arc;
use serde_json::json;

use super::types::{
    error_codes, CallToolParams, CallToolResult, InitializeResult, JsonRpcRequest,
    JsonRpcResponse, Resource, ResourceContent, ResourcesCapability, ServerCapabilities,
    ServerInfo, Tool, ToolsCapability, MCP_PROTOCOL_VERSION,
};

pub type ToolHandler = Arc<dyn Fn(serde_json::Value) -> Result<CallToolResult, String> + Send + Sync>;
pub type ResourceHandler = Arc<dyn Fn() -> Result<String, String> + Send + Sync>;

pub struct McpServer {
    server_info: ServerInfo,
    tools: HashMap<String, (Tool, ToolHandler)>,
    resources: HashMap<String, (Resource, ResourceHandler)>,
}

impl McpServer {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            server_info: ServerInfo {
                name: name.into(),
                version: version.into(),
            },
            tools: HashMap::new(),
            resources: HashMap::new(),
        }
    }

    /// Registers a callable tool with its input schema and execution closure.
    pub fn register_tool<F>(&mut self, tool: Tool, handler: F)
    where
        F: Fn(serde_json::Value) -> Result<CallToolResult, String> + Send + Sync + 'static,
    {
        self.tools.insert(tool.name.clone(), (tool, Arc::new(handler)));
    }

    /// Registers a readable context resource with a text generator.
    pub fn register_resource<F>(&mut self, resource: Resource, handler: F)
    where
        F: Fn() -> Result<String, String> + Send + Sync + 'static,
    {
        self.resources.insert(resource.uri.clone(), (resource, Arc::new(handler)));
    }

    /// Processes a single raw JSON-RPC string line (from stdio or WebSocket).
    /// Returns Some(response_json) if response is expected, or None for notifications.
    pub fn handle_raw_json(&self, raw: &str) -> Option<String> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return None;
        }

        let request: JsonRpcRequest = match serde_json::from_str(trimmed) {
            Ok(req) => req,
            Err(e) => {
                let err_resp = JsonRpcResponse::error(
                    None,
                    error_codes::PARSE_ERROR,
                    format!("JSON-RPC parse error: {}", e),
                );
                return serde_json::to_string(&err_resp).ok();
            }
        };

        let response = self.handle_request(request)?;
        serde_json::to_string(&response).ok()
    }

    /// Handles a parsed JSON-RPC 2.0 request or notification.
    pub fn handle_request(&self, req: JsonRpcRequest) -> Option<JsonRpcResponse> {
        let is_notification = req.id.is_none();
        let id = req.id.clone();

        match req.method.as_str() {
            "initialize" => {
                let result = InitializeResult {
                    protocol_version: MCP_PROTOCOL_VERSION.to_string(),
                    capabilities: ServerCapabilities {
                        tools: Some(ToolsCapability { list_changed: Some(false) }),
                        resources: Some(ResourcesCapability {
                            subscribe: Some(false),
                            list_changed: Some(false),
                        }),
                        prompts: None,
                    },
                    server_info: self.server_info.clone(),
                };
                Some(JsonRpcResponse::success(id, json!(result)))
            }

            "notifications/initialized" => {
                // Client handshake acknowledgement notification
                None
            }

            "ping" => {
                Some(JsonRpcResponse::success(id, json!({})))
            }

            "tools/list" => {
                let tools_list: Vec<&Tool> = self.tools.values().map(|(t, _)| t).collect();
                Some(JsonRpcResponse::success(id, json!({ "tools": tools_list })))
            }

            "tools/call" => {
                let params_val = req.params.unwrap_or(json!({}));
                let call_params: CallToolParams = match serde_json::from_value(params_val) {
                    Ok(p) => p,
                    Err(e) => {
                        return Some(JsonRpcResponse::error(
                            id,
                            error_codes::INVALID_PARAMS,
                            format!("Invalid tools/call params: {}", e),
                        ));
                    }
                };

                let (_, handler) = match self.tools.get(&call_params.name) {
                    Some(pair) => pair,
                    None => {
                        return Some(JsonRpcResponse::error(
                            id,
                            error_codes::INVALID_PARAMS,
                            format!("Tool '{}' not found", call_params.name),
                        ));
                    }
                };

                let args = call_params.arguments.unwrap_or(json!({}));
                match handler(args) {
                    Ok(res) => Some(JsonRpcResponse::success(id, json!(res))),
                    Err(err_msg) => {
                        let failure = CallToolResult::error(err_msg);
                        Some(JsonRpcResponse::success(id, json!(failure)))
                    }
                }
            }

            "resources/list" => {
                let res_list: Vec<&Resource> = self.resources.values().map(|(r, _)| r).collect();
                Some(JsonRpcResponse::success(id, json!({ "resources": res_list })))
            }

            "resources/read" => {
                let params_val = req.params.unwrap_or(json!({}));
                let uri = params_val.get("uri").and_then(|v| v.as_str()).unwrap_or("");

                let (res_meta, handler) = match self.resources.get(uri) {
                    Some(pair) => pair,
                    None => {
                        return Some(JsonRpcResponse::error(
                            id,
                            error_codes::INVALID_PARAMS,
                            format!("Resource '{}' not found", uri),
                        ));
                    }
                };

                match handler() {
                    Ok(text) => {
                        let content = ResourceContent {
                            uri: res_meta.uri.clone(),
                            mime_type: res_meta.mime_type.clone(),
                            text,
                        };
                        Some(JsonRpcResponse::success(id, json!({ "contents": [content] })))
                    }
                    Err(err_msg) => Some(JsonRpcResponse::error(
                        id,
                        error_codes::INTERNAL_ERROR,
                        format!("Failed reading resource '{}': {}", uri, err_msg),
                    )),
                }
            }

            unknown => {
                if is_notification {
                    None
                } else {
                    Some(JsonRpcResponse::error(
                        id,
                        error_codes::METHOD_NOT_FOUND,
                        format!("Method '{}' not found on Sovereign MCP server", unknown),
                    ))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::types::RequestId;

    #[test]
    fn test_mcp_initialize_and_ping() {
        let server = McpServer::new("proteus-core-test", "0.1.0");

        let init_raw = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        let resp_str = server.handle_raw_json(init_raw).expect("Expected response");
        let resp: JsonRpcResponse = serde_json::from_str(&resp_str).unwrap();

        assert_eq!(resp.id, Some(RequestId::Number(1)));
        assert!(resp.error.is_none());
        let res = resp.result.unwrap();
        assert_eq!(res["serverInfo"]["name"], "proteus-core-test");
        assert_eq!(res["protocolVersion"], "2024-11-05");

        let ping_raw = r#"{"jsonrpc":"2.0","id":"ping-99","method":"ping"}"#;
        let ping_resp_str = server.handle_raw_json(ping_raw).expect("Expected ping response");
        let ping_resp: JsonRpcResponse = serde_json::from_str(&ping_resp_str).unwrap();
        assert_eq!(ping_resp.id, Some(RequestId::String("ping-99".into())));
    }

    #[test]
    fn test_mcp_tool_registration_and_execution() {
        let mut server = McpServer::new("proteus-agent", "1.0.0");

        let tool = Tool {
            name: "echo_shout".into(),
            description: "Echoes upper-case string".into(),
            input_schema: json!({
                "type": "object",
                "properties": { "msg": { "type": "string" } },
                "required": ["msg"]
            }),
        };

        server.register_tool(tool, |args| {
            let msg = args.get("msg").and_then(|v| v.as_str()).unwrap_or("");
            Ok(CallToolResult::text(msg.to_uppercase()))
        });

        // 1. List tools
        let list_req = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#;
        let list_resp_str = server.handle_raw_json(list_req).unwrap();
        assert!(list_resp_str.contains("echo_shout"));

        // 2. Call tool
        let call_req = r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"echo_shout","arguments":{"msg":"hello proteus"}}}"#;
        let call_resp_str = server.handle_raw_json(call_req).unwrap();
        let resp: JsonRpcResponse = serde_json::from_str(&call_resp_str).unwrap();
        let res = resp.result.unwrap();
        assert_eq!(res["content"][0]["text"], "HELLO PROTEUS");
    }

    #[test]
    fn test_mcp_resource_read() {
        let mut server = McpServer::new("proteus-agent", "1.0.0");

        let res = Resource {
            uri: "proteus://system/status".into(),
            name: "System Status".into(),
            description: Some("Local node health".into()),
            mime_type: Some("application/json".into()),
        };

        server.register_resource(res, || Ok(r#"{"status":"healthy","uptime":3600}"#.into()));

        let read_req = r#"{"jsonrpc":"2.0","id":4,"method":"resources/read","params":{"uri":"proteus://system/status"}}"#;
        let resp_str = server.handle_raw_json(read_req).unwrap();
        assert!(resp_str.contains("healthy"));
    }

    #[test]
    fn test_mcp_invalid_method_and_malformed_json() {
        let server = McpServer::new("proteus-agent", "1.0.0");

        let unknown_req = r#"{"jsonrpc":"2.0","id":5,"method":"unknown_foo"}"#;
        let resp_str = server.handle_raw_json(unknown_req).unwrap();
        let resp: JsonRpcResponse = serde_json::from_str(&resp_str).unwrap();
        assert_eq!(resp.error.unwrap().code, error_codes::METHOD_NOT_FOUND);

        let bad_json = r#"{"jsonrpc":"2.0", broken json"#;
        let err_resp_str = server.handle_raw_json(bad_json).unwrap();
        let err_resp: JsonRpcResponse = serde_json::from_str(&err_resp_str).unwrap();
        assert_eq!(err_resp.error.unwrap().code, error_codes::PARSE_ERROR);
    }
}
