//! JSON-RPC 2.0 error object.

use serde::{Deserialize, Serialize};

/// JSON-RPC 2.0 error object.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Error code.
    pub code: i32,

    /// Error message.
    pub message: String,

    /// Optional additional data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl JsonRpcError {
    /// Standard JSON-RPC error: Parse error (-32700).
    pub fn parse_error(detail: impl Into<String>) -> Self {
        Self {
            code: -32700,
            message: "Parse error".to_string(),
            data: Some(serde_json::Value::String(detail.into())),
        }
    }

    /// Standard JSON-RPC error: Invalid request (-32600).
    pub fn invalid_request(detail: impl Into<String>) -> Self {
        Self {
            code: -32600,
            message: "Invalid Request".to_string(),
            data: Some(serde_json::Value::String(detail.into())),
        }
    }

    /// Standard JSON-RPC error: Method not found (-32601).
    pub fn method_not_found(method: impl Into<String>) -> Self {
        Self {
            code: -32601,
            message: "Method not found".to_string(),
            data: Some(serde_json::Value::String(method.into())),
        }
    }

    /// Standard JSON-RPC error: Invalid params (-32602).
    pub fn invalid_params(detail: impl Into<String>) -> Self {
        Self {
            code: -32602,
            message: "Invalid params".to_string(),
            data: Some(serde_json::Value::String(detail.into())),
        }
    }

    /// Standard JSON-RPC error: Internal error (-32603).
    pub fn internal_error(detail: impl Into<String>) -> Self {
        Self {
            code: -32603,
            message: "Internal error".to_string(),
            data: Some(serde_json::Value::String(detail.into())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error_code() {
        let err = JsonRpcError::parse_error("bad json");
        assert_eq!(err.code, -32700);
        assert_eq!(err.message, "Parse error");
        assert_eq!(err.data, Some(serde_json::json!("bad json")));
    }

    #[test]
    fn test_invalid_request_code() {
        let err = JsonRpcError::invalid_request("missing jsonrpc");
        assert_eq!(err.code, -32600);
        assert_eq!(err.message, "Invalid Request");
    }

    #[test]
    fn test_method_not_found_code() {
        let err = JsonRpcError::method_not_found("foo/bar");
        assert_eq!(err.code, -32601);
        assert_eq!(err.message, "Method not found");
        assert_eq!(err.data, Some(serde_json::json!("foo/bar")));
    }

    #[test]
    fn test_invalid_params_code() {
        let err = JsonRpcError::invalid_params("missing name");
        assert_eq!(err.code, -32602);
        assert_eq!(err.message, "Invalid params");
    }

    #[test]
    fn test_internal_error_code() {
        let err = JsonRpcError::internal_error("something broke");
        assert_eq!(err.code, -32603);
        assert_eq!(err.message, "Internal error");
    }
}
