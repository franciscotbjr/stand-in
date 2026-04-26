//! `Resource` definition — returned by `resources/list`.

use serde::{Deserialize, Serialize};

use super::ResourceAnnotations;

/// A concrete resource definition, as returned in `resources/list`.
///
/// Represents a resource with a fixed URI. Clients can call `resources/read`
/// with this URI to retrieve the resource contents.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Resource {
    /// Unique URI of the resource.
    pub uri: String,

    /// Human-readable name.
    pub name: String,

    /// Human-readable description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// MIME type of the resource content.
    #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,

    /// Size of the resource in bytes, if known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,

    /// Optional annotations for the resource.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub annotations: Option<ResourceAnnotations>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_serialize_minimal() {
        let resource = Resource {
            uri: "file:///readme.md".to_string(),
            name: "README".to_string(),
            description: None,
            mime_type: None,
            size: None,
            annotations: None,
        };
        let json = serde_json::to_value(&resource).unwrap();
        assert_eq!(json["uri"], "file:///readme.md");
        assert_eq!(json["name"], "README");
        assert!(json.get("description").is_none());
        assert!(json.get("mimeType").is_none());
        assert!(json.get("size").is_none());
        assert!(json.get("annotations").is_none());
    }

    #[test]
    fn test_resource_serialize_full() {
        let resource = Resource {
            uri: "file:///readme.md".to_string(),
            name: "README".to_string(),
            description: Some("Project readme".to_string()),
            mime_type: Some("text/markdown".to_string()),
            size: Some(1024),
            annotations: Some(ResourceAnnotations {
                audience: Some(vec!["user".to_string()]),
                priority: Some(0.5),
            }),
        };
        let json = serde_json::to_value(&resource).unwrap();
        assert_eq!(json["description"], "Project readme");
        assert_eq!(json["mimeType"], "text/markdown");
        assert_eq!(json["size"], 1024);
        assert_eq!(json["annotations"]["priority"], 0.5);
    }

    #[test]
    fn test_resource_deserialize() {
        let json = serde_json::json!({
            "uri": "file:///readme.md",
            "name": "README",
            "description": "Project readme",
            "mimeType": "text/markdown"
        });
        let resource: Resource = serde_json::from_value(json).unwrap();
        assert_eq!(resource.uri, "file:///readme.md");
        assert_eq!(resource.name, "README");
        assert_eq!(resource.description.unwrap(), "Project readme");
        assert_eq!(resource.mime_type.unwrap(), "text/markdown");
        assert!(resource.size.is_none());
    }
}
