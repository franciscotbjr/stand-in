//! The `McpResource` trait implemented by all `#[mcp_resource]` functions.

use async_trait::async_trait;

use crate::error::Result;

use super::{ReadResourceResult, Resource, ResourceTemplate};

/// Implemented by every `#[mcp_resource]`-annotated function.
///
/// The `#[mcp_resource]` macro generates a struct and this implementation automatically.
/// Resources may be concrete (fixed URI) or template-based (URI with `{param}` placeholders).
#[async_trait]
pub trait McpResource: Send + Sync + std::fmt::Debug {
    /// URI of the resource, or URI template if this is a template resource.
    fn uri(&self) -> &str;

    /// Human-readable name.
    fn name(&self) -> &str;

    /// Human-readable description.
    fn description(&self) -> Option<&str> {
        None
    }

    /// Optional MIME type of the resource content.
    fn mime_type(&self) -> Option<&str> {
        None
    }

    /// Returns `true` if this is a template resource (URI contains `{param}` placeholders).
    fn is_template(&self) -> bool {
        false
    }

    /// Read the resource contents for the given URI.
    ///
    /// For concrete resources, `uri` is the resource's own URI.
    /// For template resources, `uri` is the concrete URI with template parameters
    /// filled in (e.g., `project://my-project/readme` for template `project://{project_id}/readme`).
    async fn read(&self, uri: &str) -> Result<ReadResourceResult>;

    /// Return the `Resource` definition for `resources/list`, or `None` if this
    /// is a template resource.
    fn to_resource(&self) -> Option<Resource> {
        None
    }

    /// Return the `ResourceTemplate` definition for `resources/templates/list`,
    /// or `None` if this is a concrete resource.
    fn to_template(&self) -> Option<ResourceTemplate> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    #[derive(Debug)]
    struct ConcreteResource;

    #[async_trait]
    impl McpResource for ConcreteResource {
        fn uri(&self) -> &str {
            "file:///readme.md"
        }

        fn name(&self) -> &str {
            "README"
        }

        fn is_template(&self) -> bool {
            false
        }

        async fn read(&self, _uri: &str) -> Result<ReadResourceResult> {
            Ok(ReadResourceResult::text("file:///readme.md", "# Hello"))
        }

        fn to_resource(&self) -> Option<Resource> {
            Some(Resource {
                uri: self.uri().to_string(),
                name: self.name().to_string(),
                description: self.description().map(|s| s.to_string()),
                mime_type: self.mime_type().map(|s| s.to_string()),
                size: None,
                annotations: None,
            })
        }
    }

    #[derive(Debug)]
    struct TemplateResource;

    #[async_trait]
    impl McpResource for TemplateResource {
        fn uri(&self) -> &str {
            "docs://{topic}/readme"
        }

        fn name(&self) -> &str {
            "Documentation"
        }

        fn is_template(&self) -> bool {
            true
        }

        async fn read(&self, _uri: &str) -> Result<ReadResourceResult> {
            Ok(ReadResourceResult::text("docs://rust/readme", "# Rust"))
        }

        fn to_template(&self) -> Option<ResourceTemplate> {
            Some(ResourceTemplate {
                uri_template: self.uri().to_string(),
                name: self.name().to_string(),
                description: self.description().map(|s| s.to_string()),
                mime_type: self.mime_type().map(|s| s.to_string()),
            })
        }
    }

    #[test]
    fn test_default_description_is_none() {
        let resource = ConcreteResource;
        assert!(resource.description().is_none());
    }

    #[test]
    fn test_default_mime_type_is_none() {
        let resource = ConcreteResource;
        assert!(resource.mime_type().is_none());
    }

    #[test]
    fn test_default_is_template_is_false() {
        let resource = ConcreteResource;
        assert!(!resource.is_template());
    }

    #[test]
    fn test_concrete_to_template_returns_none() {
        let resource = ConcreteResource;
        assert!(resource.to_template().is_none());
    }

    #[test]
    fn test_template_to_resource_returns_none() {
        let resource = TemplateResource;
        assert!(resource.to_resource().is_none());
    }

    #[tokio::test]
    async fn test_concrete_read() {
        let resource = ConcreteResource;
        let result = resource.read("file:///readme.md").await.unwrap();
        assert_eq!(result.contents.len(), 1);
        assert_eq!(result.contents[0].uri(), "file:///readme.md");
    }

    #[tokio::test]
    async fn test_template_read() {
        let resource = TemplateResource;
        let result = resource.read("docs://rust/readme").await.unwrap();
        assert_eq!(result.contents[0].uri(), "docs://rust/readme");
    }
}
