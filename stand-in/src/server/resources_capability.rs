//! Resources capability advertisement for an MCP server.

use serde::{Deserialize, Serialize};

/// Advertises that the server supports MCP resources.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ResourcesCapability {
    /// Whether the server supports resource subscriptions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscribe: Option<bool>,

    /// Whether the server can notify the client when the resource list changes.
    #[serde(rename = "listChanged", skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resources_capability_default_empty() {
        let caps = ResourcesCapability::default();
        let json = serde_json::to_value(&caps).unwrap();
        assert!(json.get("subscribe").is_none());
        assert!(json.get("listChanged").is_none());
    }

    #[test]
    fn test_resources_capability_full() {
        let caps = ResourcesCapability {
            subscribe: Some(true),
            list_changed: Some(true),
        };
        let json = serde_json::to_value(&caps).unwrap();
        assert_eq!(json["subscribe"], true);
        assert_eq!(json["listChanged"], true);
    }

    #[test]
    fn test_resources_capability_subscribe_only() {
        let caps = ResourcesCapability {
            subscribe: Some(true),
            list_changed: None,
        };
        let json = serde_json::to_value(&caps).unwrap();
        assert_eq!(json["subscribe"], true);
        assert!(json.get("listChanged").is_none());
    }
}
