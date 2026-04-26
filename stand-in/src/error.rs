//! Centralized error types for stand-in.

use thiserror::Error;

/// Result type alias using the stand-in Error.
pub type Result<T> = std::result::Result<T, Error>;

/// Centralized error enum for stand-in.
#[derive(Debug, Error)]
pub enum Error {
    /// I/O error.
    #[error("I/O error: {0}")]
    IoError(String),

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    JsonError(String),

    /// Tool execution error.
    #[error("Tool error: {0}")]
    ToolError(String),

    /// Protocol error (JSON-RPC, MCP).
    #[error("Protocol error: {0}")]
    ProtocolError(String),

    /// Transport error (stdio, HTTP).
    #[error("Transport error: {0}")]
    TransportError(String),

    /// Session error.
    #[error("Session error: {0}")]
    SessionError(String),

    /// Prompt execution error.
    #[error("Prompt error: {0}")]
    PromptError(String),

    /// Resource read/execution error.
    #[error("Resource error: {0}")]
    ResourceError(String),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::JsonError(err.to_string())
    }
}
