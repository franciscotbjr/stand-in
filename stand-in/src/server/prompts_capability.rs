//! Prompts capability advertisement for an MCP server.

use serde::{Deserialize, Serialize};

/// Advertises that the server supports MCP prompts.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PromptsCapability {
    /// Whether the server can notify the client when the prompt list changes.
    #[serde(rename = "listChanged", skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompts_capability_default_empty() {
        let caps = PromptsCapability::default();
        let json = serde_json::to_value(&caps).unwrap();
        assert!(json.get("listChanged").is_none());
    }

    #[test]
    fn test_prompts_capability_list_changed_serializes() {
        let caps = PromptsCapability {
            list_changed: Some(true),
        };
        let json = serde_json::to_value(&caps).unwrap();
        assert_eq!(json["listChanged"], true);
    }
}
