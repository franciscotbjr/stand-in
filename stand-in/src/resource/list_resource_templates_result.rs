//! Result of a `resources/templates/list` JSON-RPC call.

use serde::{Deserialize, Serialize};

use super::ResourceTemplate;

/// The result returned by `resources/templates/list`.
///
/// Contains the list of registered resource templates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResourceTemplatesResult {
    /// Resource templates known to the server.
    #[serde(rename = "resourceTemplates")]
    pub resource_templates: Vec<ResourceTemplate>,

    /// Pagination cursor, if there are more results.
    #[serde(rename = "nextCursor", skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_list_serialize() {
        let result = ListResourceTemplatesResult {
            resource_templates: vec![],
            next_cursor: None,
        };
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["resourceTemplates"].as_array().unwrap().len(), 0);
        assert!(json.get("nextCursor").is_none());
    }

    #[test]
    fn test_populated_list_serialize() {
        let result = ListResourceTemplatesResult {
            resource_templates: vec![ResourceTemplate {
                uri_template: "docs://{topic}/readme".to_string(),
                name: "Docs".to_string(),
                description: None,
                mime_type: None,
            }],
            next_cursor: Some("cursor2".to_string()),
        };
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(
            json["resourceTemplates"][0]["uriTemplate"],
            "docs://{topic}/readme"
        );
        assert_eq!(json["nextCursor"], "cursor2");
    }
}
