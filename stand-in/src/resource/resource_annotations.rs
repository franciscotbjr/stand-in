//! Resource annotations — optional metadata for resources.

use serde::{Deserialize, Serialize};

/// Annotations for a resource, providing hints to clients.
///
/// Per the MCP spec, this is optional metadata attached to `Resource`
/// definitions. Both fields are optional.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceAnnotations {
    /// Intended audience(s) for this resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audience: Option<Vec<String>>,

    /// Priority on a scale from 0.0 (lowest) to 1.0 (highest).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_annotations_serialize() {
        let ann = ResourceAnnotations {
            audience: None,
            priority: None,
        };
        let json = serde_json::to_value(&ann).unwrap();
        assert!(json.get("audience").is_none());
        assert!(json.get("priority").is_none());
    }

    #[test]
    fn test_full_annotations_serialize() {
        let ann = ResourceAnnotations {
            audience: Some(vec!["user".to_string(), "assistant".to_string()]),
            priority: Some(0.8),
        };
        let json = serde_json::to_value(&ann).unwrap();
        assert_eq!(json["audience"], serde_json::json!(["user", "assistant"]));
        assert_eq!(json["priority"], 0.8);
    }

    #[test]
    fn test_deserialize() {
        let json = serde_json::json!({"audience": ["user"], "priority": 0.5});
        let ann: ResourceAnnotations = serde_json::from_value(json).unwrap();
        assert_eq!(ann.audience.unwrap(), vec!["user"]);
        assert_eq!(ann.priority.unwrap(), 0.5);
    }
}
