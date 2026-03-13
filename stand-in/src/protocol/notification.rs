//! JSON-RPC 2.0 notification (no id field).

use serde::{Deserialize, Serialize};

/// JSON-RPC 2.0 notification (no id field).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    /// Protocol version, always "2.0".
    pub jsonrpc: String,

    /// Method name.
    pub method: String,

    /// Method parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_serialize() {
        let notif = JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: "notifications/initialized".to_string(),
            params: None,
        };
        let json = serde_json::to_string(&notif).unwrap();
        assert!(json.contains("\"method\":\"notifications/initialized\""));
        assert!(!json.contains("\"id\""));
        assert!(!json.contains("\"params\""));
    }

    #[test]
    fn test_notification_no_id() {
        let json = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
        let notif: JsonRpcNotification = serde_json::from_str(json).unwrap();
        assert_eq!(notif.jsonrpc, "2.0");
        assert_eq!(notif.method, "notifications/initialized");
        assert_eq!(notif.params, None);
    }
}
