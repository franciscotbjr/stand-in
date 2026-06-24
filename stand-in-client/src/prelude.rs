//! Prelude module — import everything you need with `use stand_in_client::prelude::*`.

pub use crate::client::{Client, ClientBuilder};
pub use crate::error::{Error, Result};
pub use crate::notification::Notification;
pub use crate::transport::ClientTransport;
#[cfg(feature = "http")]
pub use crate::transport::HttpTransport;
pub use crate::transport::StdioTransport;

#[cfg(feature = "http")]
pub use crate::auth::Credential;

#[cfg(feature = "oauth")]
pub use crate::oauth::{OAuthConfig, OAuthFlow, OAuthTokens};

// ── Re-exported stand-in types — consumers never type stand_in:: ──────────

pub use stand_in::prompt::{
    GetPromptResult, PromptContent, PromptDefinition, PromptMessage, PromptRole,
};
pub use stand_in::resource::{ReadResourceResult, Resource, ResourceContents, ResourceTemplate};
pub use stand_in::server::{ServerCapabilities, ServerInfo};
pub use stand_in::tool::{CallToolResult, Content, InputSchema, ToolDefinition};

// ── Proc-macro re-export ─────────────────────────────────────────────────

#[cfg(feature = "macros")]
pub use crate::mcp_client;
