//! Model Context Protocol (MCP) engine for local sovereign AI co-pilots and IDE agents.

pub mod types;
pub mod server;
pub mod tools;

pub use server::McpServer;
pub use tools::register_core_tools;
pub use types::*;
