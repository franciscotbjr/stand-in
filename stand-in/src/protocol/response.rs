//! JSON-RPC 2.0 response.

use serde::{Deserialize, Serialize};

use super::JsonRpcError;

/// JSON-RPC 2.0 response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// Protocol version, always "2.0".
    pub jsonrpc: String,

    /// Request ID this response corresponds to.
    pub id: serde_json::Value,

    /// Result on success.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,

    /// Error on failure.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    /// Create a success response.
    pub fn success(id: serde_json::Value, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Create an error response.
    pub fn error(id: serde_json::Value, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_response() {
        let resp = JsonRpcResponse::success(
            serde_json::json!(1),
            serde_json::json!({"status": "ok"}),
        );
        assert_eq!(resp.jsonrpc, "2.0");
        assert_eq!(resp.id, serde_json::json!(1));
        assert!(resp.result.is_some());
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_error_response() {
        let resp = JsonRpcResponse::error(
            serde_json::json!(2),
            JsonRpcError::method_not_found("unknown"),
        );
        assert_eq!(resp.jsonrpc, "2.0");
        assert_eq!(resp.id, serde_json::json!(2));
        assert!(resp.result.is_none());
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, -32601);
    }

    #[test]
    fn test_response_round_trip() {
        let resp = JsonRpcResponse::success(
            serde_json::json!(42),
            serde_json::json!({"tools": []}),
        );
        let json = serde_json::to_string(&resp).unwrap();
        let deserialized: JsonRpcResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(resp, deserialized);
    }
}
