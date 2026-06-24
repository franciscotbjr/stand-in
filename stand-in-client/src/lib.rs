#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

//! # stand-in-client
//!
//! MCP client SDK — connect, discover, and call MCP servers from Rust.
//!
//! Reuses the serde types from `stand-in` and provides the client side of
//! the MCP protocol on top of the same transport seam (stdio + Streamable HTTP).
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! # use stand_in_client::prelude::*;
//! # #[tokio::main]
//! # async fn main() -> stand_in_client::error::Result<()> {
//! let client = Client::builder()
//!     .transport(StdioTransport::command("my-mcp-server", &[] as &[&str]))
//!     .client_info("my-app", "1.0")
//!     .connect()
//!     .await?;
//!
//! let tools = client.tools();
//! let result = client.call_tool("get_weather", serde_json::json!({ "city": "SP" })).await?;
//! client.disconnect().await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Features
//!
//! | Feature  | Default | Description |
//! |----------|---------|-------------|
//! | `macros` | Yes     | `#[mcp_client]` proc macro for typed stubs |
//! | `http`   | No      | Streamable HTTP client transport |

pub mod client;
mod connection;
mod correlation;
pub mod error;
pub mod notification;
pub mod prelude;
pub mod transport;

// ── Auth (gated behind http feature) ──────────────────────────────────────

#[cfg(feature = "http")]
pub mod auth;

// ── OAuth (gated behind oauth feature) ────────────────────────────────────

#[cfg(feature = "oauth")]
pub mod oauth;

// ── Crate-root re-exports ──────────────────────────────────────────────────

pub use client::{Client, ClientBuilder};
pub use error::{Error, Result};
pub use notification::Notification;
#[cfg(feature = "http")]
pub use transport::HttpTransport;
pub use transport::{ClientTransport, StdioTransport};

#[cfg(feature = "http")]
pub use auth::Credential;

#[cfg(feature = "oauth")]
pub use oauth::{OAuthConfig, OAuthFlow, OAuthTokens};

// ── Re-exported stand-in types ─────────────────────────────────────────────
// The consumer needs these types to interact with the MCP protocol, but
// should never have to type `stand_in::` — the prelude also re-exports them.

pub use stand_in::prompt::{
    GetPromptResult, PromptContent, PromptDefinition, PromptMessage, PromptRole,
};
pub use stand_in::resource::{ReadResourceResult, Resource, ResourceContents, ResourceTemplate};
pub use stand_in::server::{ServerCapabilities, ServerInfo};
pub use stand_in::tool::{CallToolResult, Content, InputSchema, ToolDefinition};

#[cfg(feature = "macros")]
pub use stand_in_client_macros::*;

/// Hidden support module for the `#[mcp_client]` proc macro.
///
/// Re-exports dependencies the generated code needs without forcing the
/// consumer to add them manually.
#[doc(hidden)]
pub mod __macros {
    pub use serde_json;
}

#[cfg(test)]
mod verify_types {
    #[allow(unused_imports)]
    use stand_in::prompt::{
        GetPromptParams, GetPromptResult, ListPromptsResult, Prompt, PromptArgument,
        PromptDefinition,
    };
    #[allow(unused_imports)]
    use stand_in::protocol::{JsonRpcError, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse};
    #[allow(unused_imports)]
    use stand_in::resource::{
        ListResourceTemplatesResult, ListResourcesResult, ReadResourceParams, ReadResourceResult,
        Resource, ResourceAnnotations, ResourceContents, ResourceTemplate, SubscribeParams,
        UnsubscribeParams,
    };
    #[allow(unused_imports)]
    use stand_in::server::{
        ClientInfo, InitializeParams, InitializeResult, ResourcesCapability, ServerCapabilities,
        ServerInfo, ToolsCapability,
    };
    #[allow(unused_imports)]
    use stand_in::tool::{
        CallToolParams, CallToolResult, Content, InputSchema, ListToolsResult, ToolDefinition,
    };
}
