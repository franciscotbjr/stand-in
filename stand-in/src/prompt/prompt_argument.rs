//! Argument descriptor for an MCP prompt template.

use serde::{Deserialize, Serialize};

/// Describes a single argument accepted by an MCP prompt template.
///
/// Serializes to `{ "name": "...", "description": "...", "required": true }` per the MCP spec.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PromptArgument {
    /// Argument name (matches the function parameter name).
    pub name: String,

    /// Human-readable description of the argument.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Whether this argument is required. Absent means unspecified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_argument_required_serializes_fields() {
        let arg = PromptArgument {
            name: "city".to_string(),
            description: Some("Target city".to_string()),
            required: Some(true),
        };
        let json = serde_json::to_value(&arg).unwrap();
        assert_eq!(json["name"], "city");
        assert_eq!(json["description"], "Target city");
        assert_eq!(json["required"], true);
    }

    #[test]
    fn test_argument_optional_omits_none_fields() {
        let arg = PromptArgument {
            name: "format".to_string(),
            description: None,
            required: Some(false),
        };
        let json = serde_json::to_value(&arg).unwrap();
        assert_eq!(json["name"], "format");
        assert!(json.get("description").is_none());
        assert_eq!(json["required"], false);
    }

    #[test]
    fn test_argument_no_required_omits_field() {
        let arg = PromptArgument {
            name: "x".to_string(),
            description: None,
            required: None,
        };
        let json = serde_json::to_value(&arg).unwrap();
        assert!(json.get("required").is_none());
    }
}
