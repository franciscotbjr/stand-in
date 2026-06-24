//! Messages that flow across the async bridge between the UI (gpui) and
//! the engine (tokio thread hosting the `stand-in-client` SDK).
//!
//! ## Error taxonomy (three planes, never conflate)
//!
//! | Plane | Origin (SDK) | App manifestation | First used |
//! |-------|-------------|-------------------|------------|
//! | **Transport** | `Err(ConnectionError/TransportClosed/TimeoutError)` | `EngineEvent::ConnectionError` | M3 |
//! | **Protocol** | `Err(ProtocolError("code: msg"))` | `EngineEvent::ProtocolError` | M9+ |
//! | **Data** | `Ok(CallToolResult{is_error:Some(true)})` | rendered as error data in tool result | M10 |

use serde::{Deserialize, Serialize};
use stand_in_client::prelude::{
    CallToolResult, Credential, Notification, OAuthConfig, OAuthTokens, PromptDefinition,
    ReadResourceResult, Resource, ResourceTemplate, ServerCapabilities, ServerInfo, ToolDefinition,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Transport {
    Stdio,
    Http,
}

impl Transport {
    pub fn selected_ix(self) -> usize {
        // HTTP first in the transport selector (036 addendum).
        match self {
            Self::Http => 0,
            Self::Stdio => 1,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Stdio => "STDIO",
            Self::Http => "HTTP",
        }
    }

    pub fn is_remote(self) -> bool {
        !matches!(self, Self::Stdio)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnConfig {
    pub transport: Transport,
    pub command: String,
    pub args: Vec<String>,
    pub url: String,
    pub env: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub struct ServerSnapshot {
    pub server_info: ServerInfo,
    pub capabilities: ServerCapabilities,
    pub tools: Vec<ToolDefinition>,
    pub resources: Vec<Resource>,
    pub templates: Vec<ResourceTemplate>,
    pub prompts: Vec<PromptDefinition>,
    pub latency_ms: u64,
}

#[derive(Debug)]
pub enum UiCommand {
    Connect {
        config: ConnConfig,
        credential: Box<Credential>,
    },
    Disconnect,
    CallTool {
        name: String,
        arguments: serde_json::Value,
    },
    ReadResource {
        uri: String,
    },
    Subscribe {
        uri: String,
    },
    Unsubscribe {
        uri: String,
    },
    GetPrompt {
        name: String,
        arguments: serde_json::Value,
    },
    Authorize {
        config: Box<OAuthConfig>,
    },
    RefreshAuth {
        config: Box<OAuthConfig>,
        refresh_token: String,
    },
}

#[derive(Debug)]
pub enum EngineEvent {
    Connecting,
    Connected(Box<ServerSnapshot>),
    ConnectionError(String),
    Disconnected,
    ToolResult(Box<CallToolResult>),
    ToolError(String),
    ResourceResult(Box<ReadResourceResult>),
    ResourceError(String),
    Subscribed(String),
    Unsubscribed(String),
    PromptMessages(Box<stand_in_client::prelude::GetPromptResult>),
    PromptError(String),
    Notification(Notification),
    Authorized(Box<OAuthTokens>),
    AuthorizationError(String),
}
