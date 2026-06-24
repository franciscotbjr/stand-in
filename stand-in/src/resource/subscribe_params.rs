//! Parameters for the `resources/subscribe` JSON-RPC method.

use serde::{Deserialize, Serialize};

/// Parameters for a `resources/subscribe` request.
///
/// Subscribes the client to update notifications for a specific resource URI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeParams {
    /// URI of the resource to subscribe to.
    pub uri: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize() {
        let json = serde_json::json!({"uri": "file:///readme.md"});
        let params: SubscribeParams = serde_json::from_value(json).unwrap();
        assert_eq!(params.uri, "file:///readme.md");
    }

    #[test]
    fn test_deserialize_missing_uri_fails() {
        let json = serde_json::json!({});
        let result = serde_json::from_value::<SubscribeParams>(json);
        assert!(result.is_err());
    }
}
