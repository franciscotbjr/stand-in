//! Parameters for the `resources/unsubscribe` JSON-RPC method.

use serde::{Deserialize, Serialize};

/// Parameters for a `resources/unsubscribe` request.
///
/// Unsubscribes the client from update notifications for a specific resource URI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsubscribeParams {
    /// URI of the resource to unsubscribe from.
    pub uri: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize() {
        let json = serde_json::json!({"uri": "file:///readme.md"});
        let params: UnsubscribeParams = serde_json::from_value(json).unwrap();
        assert_eq!(params.uri, "file:///readme.md");
    }

    #[test]
    fn test_deserialize_missing_uri_fails() {
        let json = serde_json::json!({});
        let result = serde_json::from_value::<UnsubscribeParams>(json);
        assert!(result.is_err());
    }
}
