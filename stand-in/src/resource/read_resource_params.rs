//! Parameters for the `resources/read` JSON-RPC method.

use serde::{Deserialize, Serialize};

/// Parameters for a `resources/read` request.
///
/// Contains the URI of the resource to read. For template resources,
/// the URI is the concrete URI with template parameters filled in.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadResourceParams {
    /// URI of the resource to read.
    pub uri: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize() {
        let json = serde_json::json!({"uri": "file:///readme.md"});
        let params: ReadResourceParams = serde_json::from_value(json).unwrap();
        assert_eq!(params.uri, "file:///readme.md");
    }

    #[test]
    fn test_deserialize_missing_uri_fails() {
        let json = serde_json::json!({});
        let result = serde_json::from_value::<ReadResourceParams>(json);
        assert!(result.is_err());
    }
}
