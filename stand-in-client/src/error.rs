//! Centralized error types for stand-in-client.

use thiserror::Error;

/// Result type alias using the stand-in-client Error.
pub type Result<T> = std::result::Result<T, Error>;

/// Centralized error enum for the MCP client SDK.
///
/// Errors fall on two planes:
/// - **Transport errors** (this enum) — connection failures, I/O, timeouts.
/// - **Server errors** (`CallToolResult { isError: true }`) — returned as *data*, never as `Err`.
///
/// This enum is `#[non_exhaustive]`: new variants may be added in SemVer-compatible
/// releases. Match with a wildcard `_` arm for forward compatiblity.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// I/O error.
    #[error("I/O error: {0}")]
    IoError(String),

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    JsonError(String),

    /// Connection error (server unreachable, process launch failed).
    #[error("Connection error: {0}")]
    ConnectionError(String),

    /// Protocol error (unexpected response, handshake failure, bad wire format).
    #[error("Protocol error: {0}")]
    ProtocolError(String),

    /// Request timed out.
    #[error("Timeout: {0}")]
    TimeoutError(String),

    /// Transport is closed or unavailable.
    #[error("Transport closed: {0}")]
    TransportClosed(String),

    /// Tool execution error returned by the server (`isError: true`).
    ///
    /// Only used by the typed `#[mcp_client]` layer, which collapses the two
    /// error planes into a single `Result`. The dynamic `call_tool()` returns
    /// tool errors as `Ok(CallToolResult { isError: Some(true) })` — data,
    /// not `Err`.
    #[error("{0}")]
    ToolError(String),

    /// OAuth 2.0 authentication error.
    ///
    /// Covers: loopback bind/accept failures, state mismatch, authorization
    /// server errors, token exchange/refresh failures, and missing
    /// authorization codes.
    #[error("OAuth error: {0}")]
    OAuthError(String),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_error_display() {
        let err = Error::IoError("broken pipe".into());
        assert_eq!(err.to_string(), "I/O error: broken pipe");
    }

    #[test]
    fn test_json_error_display() {
        let err = Error::JsonError("unexpected token".into());
        assert_eq!(err.to_string(), "JSON error: unexpected token");
    }

    #[test]
    fn test_connection_error_display() {
        let err = Error::ConnectionError("refused".into());
        assert_eq!(err.to_string(), "Connection error: refused");
    }

    #[test]
    fn test_protocol_error_display() {
        let err = Error::ProtocolError("missing id in response".into());
        assert_eq!(err.to_string(), "Protocol error: missing id in response");
    }

    #[test]
    fn test_timeout_error_display() {
        let err = Error::TimeoutError("request exceeded 30s".into());
        assert_eq!(err.to_string(), "Timeout: request exceeded 30s");
    }

    #[test]
    fn test_transport_closed_display() {
        let err = Error::TransportClosed("session expired".into());
        assert_eq!(err.to_string(), "Transport closed: session expired");
    }

    #[test]
    fn test_tool_error_display() {
        let err = Error::ToolError("division by zero".into());
        assert_eq!(err.to_string(), "division by zero");
    }

    #[test]
    fn test_oauth_error_display() {
        let err = Error::OAuthError("state mismatch".into());
        assert_eq!(err.to_string(), "OAuth error: state mismatch");
    }

    #[test]
    fn test_io_error_from_trait() {
        let io = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: Error = io.into();
        assert!(matches!(err, Error::IoError(_)));
    }

    #[test]
    fn test_json_error_from_trait() {
        let json_err = serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
        let err: Error = json_err.into();
        assert!(matches!(err, Error::JsonError(_)));
    }

    #[test]
    fn test_result_alias() {
        fn ok_fn() -> Result<u32> {
            Ok(42)
        }
        assert_eq!(ok_fn().unwrap(), 42);

        fn err_fn() -> Result<u32> {
            Err(Error::ProtocolError("bad handshake".into()))
        }
        assert!(err_fn().is_err());
    }

    #[test]
    fn test_error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Error>();
    }
}
