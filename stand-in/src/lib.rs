//! # stand-in
//!
//! A stand-in for your MCP server boilerplate.
//!
//! You write declarative macros that look like your MCP server — tools, resources, prompts —
//! but when the compiler rolls, the macros step aside and production-ready code takes their place.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use stand_in::prelude::*;
//!
//! #[mcp_tool(
//!     name = "get_weather",
//!     description = "Returns current weather for a given city"
//! )]
//! async fn get_weather(city: String) -> Result<String> {
//!     Ok(format!("{}: sunny", city))
//! }
//!
//! #[mcp_server]
//! struct MyServer;
//!
//! #[tokio::main]
//! async fn main() {
//!     MyServer::serve(StdioTransport::default()).await;
//! }
//! ```

pub mod error;
pub mod prompt;
pub mod protocol;
pub mod resource;
pub mod server;
pub mod tool;
pub mod transport;

pub use stand_in_macros::*;

/// Prelude module — import everything you need with `use stand_in::prelude::*`.
pub mod prelude {
    pub use crate::error::{Error, Result};
    pub use crate::prompt::{Prompt, PromptMessage};
    pub use crate::protocol::{JsonRpcError, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse};
    pub use crate::resource::{
        ListResourceTemplatesResult, ListResourcesResult, McpResource, ReadResourceParams,
        ReadResourceResult, Resource, ResourceAnnotations, ResourceContents, ResourceRegistry,
        ResourceTemplate, SubscribeParams, UnsubscribeParams,
    };
    pub use crate::server::{
        ClientInfo, InitializeParams, InitializeResult, RequestHandler, ResourcesCapability,
        ServerCapabilities, ServerInfo, ToolsCapability,
    };
    pub use crate::tool::{
        CallToolParams, CallToolResult, Content, InputSchema, ListToolsResult, McpTool,
        ToolDefinition, ToolRegistry,
    };
    #[cfg(feature = "http")]
    pub use crate::transport::HttpTransport;
    #[cfg(feature = "stdio")]
    pub use crate::transport::StdioTransport;
    pub use crate::transport::Transport;
    pub use stand_in_macros::*;
}
