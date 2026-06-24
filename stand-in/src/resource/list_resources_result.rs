//! Result of a `resources/list` JSON-RPC call.

use serde::{Deserialize, Serialize};

use super::Resource;

/// The result returned by `resources/list`.
///
/// Contains the list of registered concrete resources.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResourcesResult {
    /// Concrete resources known to the server.
    pub resources: Vec<Resource>,

    /// Pagination cursor, if there are more results.
    #[serde(rename = "nextCursor", skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_list_serialize() {
        let result = ListResourcesResult {
            resources: vec![],
            next_cursor: None,
        };
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["resources"].as_array().unwrap().len(), 0);
        assert!(json.get("nextCursor").is_none());
    }

    #[test]
    fn test_populated_list_serialize() {
        let result = ListResourcesResult {
            resources: vec![Resource {
                uri: "file:///readme.md".to_string(),
                name: "README".to_string(),
                description: None,
                mime_type: None,
                size: None,
                annotations: None,
            }],
            next_cursor: Some("cursor1".to_string()),
        };
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["resources"][0]["name"], "README");
        assert_eq!(json["nextCursor"], "cursor1");
    }
}
