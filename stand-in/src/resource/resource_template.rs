//! `ResourceTemplate` definition — returned by `resources/templates/list`.

use serde::{Deserialize, Serialize};

/// A resource template definition, as returned in `resources/templates/list`.
///
/// Resource templates use URI templates with `{param}` placeholders
/// (e.g., `project://{id}/readme`) and are resolved to concrete URIs
/// when calling `resources/read`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceTemplate {
    /// RFC 6570 URI template (e.g., `project://{project_id}/readme`).
    #[serde(rename = "uriTemplate")]
    pub uri_template: String,

    /// Human-readable name.
    pub name: String,

    /// Human-readable description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// MIME type of the resource content.
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_template_serialize() {
        let template = ResourceTemplate {
            uri_template: "project://{id}/readme".to_string(),
            name: "Project README".to_string(),
            description: Some("Readme for a project".to_string()),
            mime_type: Some("text/markdown".to_string()),
        };
        let json = serde_json::to_value(&template).unwrap();
        assert_eq!(json["uriTemplate"], "project://{id}/readme");
        assert_eq!(json["name"], "Project README");
        assert_eq!(json["description"], "Readme for a project");
        assert_eq!(json["mimeType"], "text/markdown");
    }

    #[test]
    fn test_template_serialize_minimal() {
        let template = ResourceTemplate {
            uri_template: "docs://{topic}/readme".to_string(),
            name: "Docs".to_string(),
            description: None,
            mime_type: None,
        };
        let json = serde_json::to_value(&template).unwrap();
        assert_eq!(json["uriTemplate"], "docs://{topic}/readme");
        assert!(json.get("description").is_none());
        assert!(json.get("mimeType").is_none());
    }

    #[test]
    fn test_template_deserialize() {
        let json = serde_json::json!({
            "uriTemplate": "project://{id}/readme",
            "name": "Project README",
            "mimeType": "text/markdown"
        });
        let template: ResourceTemplate = serde_json::from_value(json).unwrap();
        assert_eq!(template.uri_template, "project://{id}/readme");
        assert_eq!(template.name, "Project README");
        assert_eq!(template.mime_type.unwrap(), "text/markdown");
        assert!(template.description.is_none());
    }
}
