//! Tools-specific capability advertisement.

use serde::{Deserialize, Serialize};

/// Tools-specific capability advertisement.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ToolsCapability {
    /// Whether the tool list may change during the session.
    #[serde(rename = "listChanged", skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tools_capability_serialize() {
        let cap = ToolsCapability::default();
        let json = serde_json::to_value(&cap).unwrap();
        assert!(json.get("listChanged").is_none());

        let cap_with = ToolsCapability {
            list_changed: Some(true),
        };
        let json = serde_json::to_value(&cap_with).unwrap();
        assert_eq!(json["listChanged"], true);
    }
}
