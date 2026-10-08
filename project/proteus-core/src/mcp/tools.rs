//! Declarative Canvas Layout & Schema Generation MCP Tools.
//! Empowers external AI agents (Claude, Gemini, IDEs) to scaffold schemas and 2D canvas designs natively.

use std::path::PathBuf;
use rusqlite::Connection;
use serde_json::json;

use super::server::McpServer;
use super::types::{CallToolResult, Tool};
use crate::dsl::{parse_dsl, DslTranspiler, TargetDialect};

/// Registers the canonical sovereign MCP tool suite onto an McpServer instance.
pub fn register_core_tools(server: &mut McpServer) {
    register_generate_schema_tool(server);
    register_scaffold_canvas_tool(server);
    register_transpile_dsl_tool(server);
    register_query_database_schema_tool(server);
}

/// Tool: generate_schema_from_prompt
fn register_generate_schema_tool(server: &mut McpServer) {
    let tool = Tool {
        name: "generate_schema_from_prompt".to_string(),
        description: "Generates relational SQLite DDL tables and entity definitions from a natural language domain prompt.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "prompt": { "type": "string", "description": "Natural language prompt describing the business entities" },
                "industry": { "type": "string", "description": "Optional sector e.g. technology_repairs, retail, dental, logistics" }
            },
            "required": ["prompt"]
        }),
    };

    server.register_tool(tool, |args| {
        let prompt = args.get("prompt").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
        let industry = args.get("industry").and_then(|v| v.as_str()).unwrap_or("");

        let mut tables = Vec::new();

        if prompt.contains("repair") || prompt.contains("ticket") || industry.contains("repair") {
            tables.push(json!({
                "table_name": "service_tickets",
                "ddl": "CREATE TABLE IF NOT EXISTS service_tickets (\n    id TEXT PRIMARY KEY,\n    ticket_number INTEGER NOT NULL,\n    customer_name TEXT NOT NULL,\n    customer_phone TEXT NOT NULL,\n    device_model TEXT NOT NULL,\n    fault_description TEXT NOT NULL,\n    status TEXT NOT NULL DEFAULT 'received',\n    estimated_cost REAL NOT NULL DEFAULT 0.0,\n    created_at TEXT NOT NULL\n);",
                "columns": ["id", "ticket_number", "customer_name", "customer_phone", "device_model", "fault_description", "status", "estimated_cost", "created_at"]
            }));
        }

        if prompt.contains("customer") || prompt.contains("contact") || prompt.contains("crm") {
            tables.push(json!({
                "table_name": "contacts",
                "ddl": "CREATE TABLE IF NOT EXISTS contacts (\n    id TEXT PRIMARY KEY,\n    name TEXT NOT NULL,\n    email TEXT,\n    phone TEXT,\n    company TEXT,\n    vat_number TEXT,\n    created_at TEXT NOT NULL\n);",
                "columns": ["id", "name", "email", "phone", "company", "vat_number", "created_at"]
            }));
        }

        if prompt.contains("product") || prompt.contains("item") || prompt.contains("inventory") || prompt.contains("stock") {
            tables.push(json!({
                "table_name": "products",
                "ddl": "CREATE TABLE IF NOT EXISTS products (\n    id TEXT PRIMARY KEY,\n    sku TEXT UNIQUE NOT NULL,\n    title TEXT NOT NULL,\n    price REAL NOT NULL DEFAULT 0.0,\n    stock_quantity INTEGER NOT NULL DEFAULT 0,\n    tax_rate REAL NOT NULL DEFAULT 0.24,\n    created_at TEXT NOT NULL\n);",
                "columns": ["id", "sku", "title", "price", "stock_quantity", "tax_rate", "created_at"]
            }));
        }

        // Generic fallback entity if prompt had bespoke terms
        if tables.is_empty() {
            let entity_slug = prompt
                .split_whitespace()
                .next()
                .unwrap_or("records")
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '_')
                .collect::<String>();
            let table_name = if entity_slug.is_empty() { "domain_records".to_string() } else { format!("{}_records", entity_slug) };

            tables.push(json!({
                "table_name": table_name,
                "ddl": format!("CREATE TABLE IF NOT EXISTS {} (\n    id TEXT PRIMARY KEY,\n    title TEXT NOT NULL,\n    description TEXT,\n    status TEXT NOT NULL DEFAULT 'active',\n    created_at TEXT NOT NULL\n);", table_name),
                "columns": ["id", "title", "description", "status", "created_at"]
            }));
        }

        let result = json!({
            "status": "success",
            "prompt": prompt,
            "table_count": tables.len(),
            "tables": tables
        });

        Ok(CallToolResult::text(serde_json::to_string_pretty(&result).unwrap_or_default()))
    });
}

/// Tool: scaffold_canvas_layout
fn register_scaffold_canvas_tool(server: &mut McpServer) {
    let tool = Tool {
        name: "scaffold_canvas_layout".to_string(),
        description: "Scaffolds declarative 2D UI canvas layout JSON with containers, input controls, action buttons, and tables.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "entity_name": { "type": "string", "description": "Entity name to bind inputs and table to" },
                "screen_title": { "type": "string", "description": "Title label for the screen card" },
                "fields": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "List of input fields to generate (e.g. name, email, phone)"
                },
                "include_table": { "type": "boolean", "description": "Whether to include a live data-bound table below inputs" }
            },
            "required": ["entity_name", "screen_title", "fields"]
        }),
    };

    server.register_tool(tool, |args| {
        let entity = args.get("entity_name").and_then(|v| v.as_str()).unwrap_or("entity");
        let title = args.get("screen_title").and_then(|v| v.as_str()).unwrap_or("New Form");
        let fields = args.get("fields").and_then(|v| v.as_array()).cloned().unwrap_or_default();
        let include_table = args.get("include_table").and_then(|v| v.as_bool()).unwrap_or(true);

        let mut children = Vec::new();
        let mut cur_y = 60.0;

        // 1. Header Text Node
        children.push(json!({
            "id": format!("{}-title", entity),
            "type": "Text",
            "position": [24.0, 20.0],
            "size": [400.0, 28.0],
            "props": {
                "content": title,
                "font_size": 18.0,
                "font_weight": 700
            }
        }));

        // 2. Input Fields
        for (idx, f_val) in fields.iter().enumerate() {
            let field_name = f_val.as_str().unwrap_or("field");
            let field_label = field_name.replace('_', " ").to_uppercase();
            let is_col2 = idx % 2 == 1;
            let col_x = if is_col2 { 260.0 } else { 24.0 };

            children.push(json!({
                "id": format!("{}-in-{}", entity, field_name),
                "type": "TextInput",
                "position": [col_x, cur_y],
                "size": [220.0, 36.0],
                "props": {
                    "placeholder": field_label,
                    "bound_entity": entity,
                    "bound_field": field_name
                }
            }));

            if is_col2 || idx == fields.len() - 1 {
                cur_y += 48.0;
            }
        }

        // 3. Action Submit Button
        children.push(json!({
            "id": format!("{}-btn-submit", entity),
            "type": "Button",
            "position": [24.0, cur_y],
            "size": [160.0, 38.0],
            "props": {
                "label": format!("Save {} ✓", title),
                "style": "Primary",
                "bound_entity": entity
            }
        }));
        cur_y += 56.0;

        // 4. Data-bound Table (Optional)
        if include_table {
            let cols: Vec<String> = fields
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();

            children.push(json!({
                "id": format!("{}-table", entity),
                "type": "Table",
                "position": [24.0, cur_y],
                "size": [560.0, 220.0],
                "props": {
                    "bound_entity": entity,
                    "columns": cols
                }
            }));
            cur_y += 240.0;
        }

        let frame_node = json!({
            "id": format!("frame-{}", entity),
            "name": title,
            "type": "Frame",
            "position": [40.0, 60.0],
            "size": [600.0, cur_y + 20.0],
            "children": children
        });

        Ok(CallToolResult::text(serde_json::to_string_pretty(&frame_node).unwrap_or_default()))
    });
}

/// Tool: transpile_dsl_script
fn register_transpile_dsl_tool(server: &mut McpServer) {
    let tool = Tool {
        name: "transpile_dsl_script".to_string(),
        description: "Transpiles a declarative Proteus Database DSL script into SQL (sqlite, postgres, mysql, mongodb).".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "dsl_script": { "type": "string", "description": "Proteus declarative DSL source script" },
                "dialect": {
                    "type": "string",
                    "enum": ["sqlite", "postgres", "mysql", "mongodb"],
                    "description": "Target SQL dialect (defaults to sqlite)"
                }
            },
            "required": ["dsl_script"]
        }),
    };

    server.register_tool(tool, |args| {
        let script = args.get("dsl_script").and_then(|v| v.as_str()).unwrap_or("");
        let dialect_str = args.get("dialect").and_then(|v| v.as_str()).unwrap_or("sqlite");

        let dialect = match dialect_str {
            "postgres" | "postgresql" => TargetDialect::Postgres,
            "mysql" => TargetDialect::Mysql,
            "mongodb" | "mongo" => TargetDialect::MongoDb,
            _ => TargetDialect::Sqlite,
        };

        match parse_dsl(script) {
            Ok(schema) => {
                let sql = DslTranspiler::transpile(&schema, dialect);
                let result = json!({
                    "status": "success",
                    "dialect": dialect_str,
                    "entity_count": schema.entities.len(),
                    "output_code": sql
                });
                Ok(CallToolResult::text(serde_json::to_string_pretty(&result).unwrap_or_default()))
            }
            Err(e) => {
                Ok(CallToolResult::error(format!("DSL Parsing failed: {}", e)))
            }
        }
    });
}

/// Tool: query_database_schema
fn register_query_database_schema_tool(server: &mut McpServer) {
    let tool = Tool {
        name: "query_database_schema".to_string(),
        description: "Inspects live SQLite database schema, listing real tables and columns without mock data.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "db_path": { "type": "string", "description": "Path to SQLite database file. Defaults to Proteus standard store." }
            }
        }),
    };

    server.register_tool(tool, |args| {
        let path_str = args.get("db_path").and_then(|v| v.as_str());
        let path = path_str
            .map(PathBuf::from)
            .unwrap_or_else(crate::paths::get_database_path);

        if !path.exists() {
            // Return clean schema status if store file not yet initialized
            let msg = json!({
                "status": "not_found",
                "db_path": path.to_string_lossy(),
                "tables": []
            });
            return Ok(CallToolResult::text(serde_json::to_string_pretty(&msg).unwrap_or_default()));
        }

        let conn = match Connection::open(&path) {
            Ok(c) => c,
            Err(e) => return Ok(CallToolResult::error(format!("Could not open SQLite database at '{}': {}", path.display(), e))),
        };

        let mut stmt = match conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name;") {
            Ok(s) => s,
            Err(e) => return Ok(CallToolResult::error(format!("Failed to prepare sqlite_master query: {}", e))),
        };

        let table_names: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .map(|rows| rows.filter_map(Result::ok).collect())
            .unwrap_or_default();

        let mut tables_info = Vec::new();
        for t in table_names {
            let pragma_sql = format!("PRAGMA table_info(\"{}\");", t);
            if let Ok(mut p_stmt) = conn.prepare(&pragma_sql) {
                let cols: Vec<serde_json::Value> = p_stmt.query_map([], |row| {
                    let cid: i64 = row.get(0)?;
                    let name: String = row.get(1)?;
                    let col_type: String = row.get(2)?;
                    let notnull: i64 = row.get(3)?;
                    let pk: i64 = row.get(5)?;
                    Ok(json!({
                        "cid": cid,
                        "name": name,
                        "type": col_type,
                        "notnull": notnull == 1,
                        "is_pk": pk == 1
                    }))
                }).map(|rows| rows.filter_map(Result::ok).collect()).unwrap_or_default();

                tables_info.push(json!({
                    "table_name": t,
                    "columns": cols
                }));
            }
        }

        let out = json!({
            "status": "success",
            "db_path": path.to_string_lossy(),
            "table_count": tables_info.len(),
            "tables": tables_info
        });

        Ok(CallToolResult::text(serde_json::to_string_pretty(&out).unwrap_or_default()))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_schema_tool() {
        let mut server = McpServer::new("test", "1.0");
        register_generate_schema_tool(&mut server);

        let req = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"generate_schema_from_prompt","arguments":{"prompt":"I need an auto repair workshop ticket tracker"}}}"#;
        let res_str = server.handle_raw_json(req).expect("Expected response");
        assert!(res_str.contains("service_tickets"));
        assert!(res_str.contains("device_model"));
    }

    #[test]
    fn test_scaffold_canvas_tool() {
        let mut server = McpServer::new("test", "1.0");
        register_scaffold_canvas_tool(&mut server);

        let req = r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"scaffold_canvas_layout","arguments":{"entity_name":"customers","screen_title":"New Customer Intake","fields":["name","phone","email"],"include_table":true}}}"#;
        let res_str = server.handle_raw_json(req).expect("Expected response");
        assert!(res_str.contains("TextInput"));
        assert!(res_str.contains("Table"));
        assert!(res_str.contains("New Customer Intake"));
    }

    #[test]
    fn test_transpile_dsl_tool() {
        let mut server = McpServer::new("test", "1.0");
        register_transpile_dsl_tool(&mut server);

        let dsl = "entity Invoice { id: uuid primary, total: decimal required }";
        let req = format!(
            r#"{{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{{"name":"transpile_dsl_script","arguments":{{"dsl_script":"{}","dialect":"postgres"}}}}}}"#,
            dsl
        );
        let res_str = server.handle_raw_json(&req).expect("Expected response");
        let resp: crate::mcp::types::JsonRpcResponse = serde_json::from_str(&res_str).unwrap();
        let text = resp.result.unwrap()["content"][0]["text"].as_str().unwrap().to_string();
        let payload: serde_json::Value = serde_json::from_str(&text).unwrap();
        let sql = payload["output_code"].as_str().unwrap();
        assert!(sql.contains("CREATE TABLE IF NOT EXISTS \"Invoice\""));
        assert!(sql.contains("\"id\" UUID PRIMARY KEY"));
    }

    #[test]
    fn test_query_database_schema_tool_in_memory() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join(format!("test_mcp_db_{}.db", uuid::Uuid::new_v4()));

        let conn = Connection::open(&db_path).unwrap();
        conn.execute("CREATE TABLE test_mcp_records (id TEXT PRIMARY KEY, title TEXT NOT NULL);", []).unwrap();

        let mut server = McpServer::new("test", "1.0");
        register_query_database_schema_tool(&mut server);

        let req = format!(
            r#"{{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{{"name":"query_database_schema","arguments":{{"db_path":"{}"}}}}}}"#,
            db_path.to_string_lossy().replace('\\', "/")
        );
        let res_str = server.handle_raw_json(&req).expect("Expected response");
        assert!(res_str.contains("test_mcp_records"));
        assert!(res_str.contains("title"));

        let _ = std::fs::remove_file(db_path);
    }
}
