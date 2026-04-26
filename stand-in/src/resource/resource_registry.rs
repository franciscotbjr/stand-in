//! Registry that holds all registered `#[mcp_resource]` implementations.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::{debug, warn};

use crate::error::{Error, Result};

use super::{McpResource, ReadResourceResult, Resource, ResourceTemplate};

/// Subscription map type alias to reduce type complexity.
type SubscriptionMap = HashMap<String, Vec<(String, tokio::sync::broadcast::Sender<String>)>>;

/// Holds all registered resources and dispatches `resources/list`,
/// `resources/templates/list`, `resources/read`, `resources/subscribe`,
/// and `resources/unsubscribe`.
#[derive(Debug, Default)]
pub struct ResourceRegistry {
    resources: Vec<Box<dyn McpResource>>,
    /// Subscription map: URI → Vec of (session_id, broadcast sender).
    subscriptions: Arc<RwLock<SubscriptionMap>>,
}

impl ResourceRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a resource implementation.
    pub fn register(&mut self, resource: Box<dyn McpResource>) {
        debug!(uri = %resource.uri(), "Registered resource");
        self.resources.push(resource);
    }

    /// Return all concrete resources for `resources/list`.
    pub fn list_resources(&self) -> Vec<Resource> {
        self.resources
            .iter()
            .filter(|r| !r.is_template())
            .filter_map(|r| r.to_resource())
            .collect()
    }

    /// Return all resource templates for `resources/templates/list`.
    pub fn list_templates(&self) -> Vec<ResourceTemplate> {
        self.resources
            .iter()
            .filter(|r| r.is_template())
            .filter_map(|r| r.to_template())
            .collect()
    }

    /// Read a resource by URI.
    ///
    /// First tries exact URI match on concrete resources, then falls back
    /// to template-based matching for template resources.
    pub async fn read_resource(&self, uri: &str) -> Result<ReadResourceResult> {
        // Try exact match on concrete resources first
        for resource in &self.resources {
            if !resource.is_template() && resource.uri() == uri {
                debug!(uri = %uri, "Reading concrete resource");
                return resource.read(uri).await;
            }
        }

        // Try template matching
        for resource in &self.resources {
            if resource.is_template()
                && let Some(_params) = match_template(resource.uri(), uri)
            {
                debug!(uri = %uri, template = %resource.uri(), "Matched template resource");
                return resource.read(uri).await;
            }
        }

        warn!(uri = %uri, "Unknown resource requested");
        Err(Error::ResourceError(format!("Unknown resource: {uri}")))
    }

    /// Subscribe to updates for a resource URI.
    pub async fn subscribe(
        &self,
        uri: &str,
        session_id: String,
        sender: tokio::sync::broadcast::Sender<String>,
    ) {
        let mut subs = self.subscriptions.write().await;
        subs.entry(uri.to_string())
            .or_default()
            .push((session_id, sender));
        debug!(uri = %uri, "Subscription added");
    }

    /// Unsubscribe from updates for a resource URI.
    pub async fn unsubscribe(&self, uri: &str, session_id: &str) {
        let mut subs = self.subscriptions.write().await;
        if let Some(entries) = subs.get_mut(uri) {
            entries.retain(|(sid, _)| sid != session_id);
            if entries.is_empty() {
                subs.remove(uri);
            }
        }
        debug!(uri = %uri, session_id = %session_id, "Subscription removed");
    }

    /// Notify all subscribers that a resource has been updated.
    ///
    /// Sends a `notifications/resources/updated` JSON-RPC notification
    /// to each subscribed session's SSE channel.
    pub async fn notify(&self, uri: &str) {
        let subs = self.subscriptions.read().await;
        if let Some(entries) = subs.get(uri) {
            let notification = serde_json::json!({
                "jsonrpc": "2.0",
                "method": crate::protocol::notification_methods::RESOURCES_UPDATED,
                "params": { "uri": uri }
            });
            let text = notification.to_string();
            for (_sid, sender) in entries {
                if let Err(e) = sender.send(text.clone()) {
                    debug!(
                        uri = %uri,
                        session_id = %_sid,
                        error = %e,
                        "Failed to send resource update notification"
                    );
                }
            }
        }
    }

    /// Notify all subscribers with updated resource contents.
    ///
    /// Sends a `notifications/resources/updated` notification with the
    /// new resource contents included in params.
    pub async fn notify_with_contents(&self, uri: &str, contents: &ReadResourceResult) {
        let subs = self.subscriptions.read().await;
        if let Some(entries) = subs.get(uri) {
            let notification = match serde_json::to_value(contents) {
                Ok(value) => serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": crate::protocol::notification_methods::RESOURCES_UPDATED,
                    "params": {
                        "uri": uri,
                        "contents": value["contents"]
                    }
                }),
                Err(_) => serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": crate::protocol::notification_methods::RESOURCES_UPDATED,
                    "params": { "uri": uri }
                }),
            };
            let text = notification.to_string();
            for (_sid, sender) in entries {
                let _ = sender.send(text.clone());
            }
        }
    }

    /// Return the number of registered resources (concrete + templates).
    pub fn len(&self) -> usize {
        self.resources.len()
    }

    /// Returns `true` if no resources are registered.
    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }
}

/// Match a URI template against a concrete URI, extracting parameter values.
///
/// Supports simple `{param}` placeholders separated by `/`.
/// Returns `Some(params)` if the template matches, `None` otherwise.
fn match_template(template: &str, uri: &str) -> Option<HashMap<String, String>> {
    let template_parts: Vec<&str> = template.split('/').collect();
    let uri_parts: Vec<&str> = uri.split('/').collect();

    if template_parts.len() != uri_parts.len() {
        return None;
    }

    let mut params = HashMap::new();
    for (t, u) in template_parts.iter().zip(uri_parts.iter()) {
        if let Some(param_name) = t.strip_prefix('{').and_then(|s| s.strip_suffix('}')) {
            params.insert(param_name.to_string(), (*u).to_string());
        } else if t != u {
            return None;
        }
    }
    Some(params)
}

/// Public utility: match a URI template against a concrete URI and return
/// extracted parameters as a `serde_json::Value` object.
///
/// Used both by the registry (for template matching during `resources/read`)
/// and by the `#[mcp_resource]` macro (to extract params at runtime).
pub fn match_template_params(template: &str, uri: &str) -> Option<serde_json::Value> {
    let params = match_template(template, uri)?;
    let map: serde_json::Map<String, serde_json::Value> = params
        .into_iter()
        .map(|(k, v)| (k, serde_json::Value::String(v)))
        .collect();
    Some(serde_json::Value::Object(map))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Result;
    use async_trait::async_trait;

    #[derive(Debug)]
    struct ConcrRes {
        uri: String,
        name: String,
    }

    impl Default for ConcrRes {
        fn default() -> Self {
            Self {
                uri: String::new(),
                name: String::new(),
            }
        }
    }

    #[async_trait]
    impl McpResource for ConcrRes {
        fn uri(&self) -> &str {
            &self.uri
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn is_template(&self) -> bool {
            false
        }

        async fn read(&self, _uri: &str) -> Result<ReadResourceResult> {
            Ok(ReadResourceResult::text(self.uri.clone(), "content"))
        }

        fn to_resource(&self) -> Option<Resource> {
            Some(Resource {
                uri: self.uri.clone(),
                name: self.name.clone(),
                description: None,
                mime_type: None,
                size: None,
                annotations: None,
            })
        }
    }

    #[derive(Debug)]
    struct TmplRes {
        uri_template: String,
        name: String,
    }

    #[async_trait]
    impl McpResource for TmplRes {
        fn uri(&self) -> &str {
            &self.uri_template
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn is_template(&self) -> bool {
            true
        }

        async fn read(&self, uri: &str) -> Result<ReadResourceResult> {
            Ok(ReadResourceResult::text(uri, "template content"))
        }

        fn to_template(&self) -> Option<ResourceTemplate> {
            Some(ResourceTemplate {
                uri_template: self.uri_template.clone(),
                name: self.name.clone(),
                description: None,
                mime_type: None,
            })
        }
    }

    #[test]
    fn test_match_template_simple() {
        let result = match_template("docs://{topic}/readme", "docs://rust/readme");
        assert!(result.is_some());
        let params = result.unwrap();
        assert_eq!(params.get("topic").unwrap(), "rust");
    }

    #[test]
    fn test_match_template_multiple_params() {
        let result = match_template("project://{org}/{repo}/file", "project://acme/widget/file");
        let params = result.unwrap();
        assert_eq!(params.get("org").unwrap(), "acme");
        assert_eq!(params.get("repo").unwrap(), "widget");
    }

    #[test]
    fn test_match_template_no_match_different_length() {
        let result = match_template("docs://{topic}/readme", "docs://rust/readme/extra");
        assert!(result.is_none());
    }

    #[test]
    fn test_match_template_no_match_different_literal() {
        let result = match_template("docs://{topic}/readme", "docs://rust/about");
        assert!(result.is_none());
    }

    #[test]
    fn test_match_template_no_match_different_prefix() {
        let result = match_template("docs://{topic}/readme", "files://rust/readme");
        assert!(result.is_none());
    }

    #[test]
    fn test_empty_registry() {
        let registry = ResourceRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
        assert!(registry.list_resources().is_empty());
        assert!(registry.list_templates().is_empty());
    }

    #[test]
    fn test_register_and_list_concrete() {
        let mut registry = ResourceRegistry::new();
        registry.register(Box::new(ConcrRes {
            uri: "file:///a.txt".to_string(),
            name: "A".to_string(),
            ..ConcrRes::default()
        }));
        let resources = registry.list_resources();
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0].uri, "file:///a.txt");
        assert!(registry.list_templates().is_empty());
    }

    #[test]
    fn test_register_and_list_template() {
        let mut registry = ResourceRegistry::new();
        registry.register(Box::new(TmplRes {
            uri_template: "docs://{topic}/readme".to_string(),
            name: "Docs".to_string(),
        }));
        let templates = registry.list_templates();
        assert_eq!(templates.len(), 1);
        assert_eq!(templates[0].uri_template, "docs://{topic}/readme");
        assert!(registry.list_resources().is_empty());
    }

    #[tokio::test]
    async fn test_read_concrete_resource() {
        let mut registry = ResourceRegistry::new();
        registry.register(Box::new(ConcrRes {
            uri: "file:///readme.md".to_string(),
            name: "README".to_string(),
        }));
        let result = registry.read_resource("file:///readme.md").await.unwrap();
        assert_eq!(result.contents.len(), 1);
        assert_eq!(result.contents[0].uri(), "file:///readme.md");
    }

    #[tokio::test]
    async fn test_read_template_resource() {
        let mut registry = ResourceRegistry::new();
        registry.register(Box::new(TmplRes {
            uri_template: "docs://{topic}/readme".to_string(),
            name: "Docs".to_string(),
        }));
        let result = registry.read_resource("docs://rust/readme").await.unwrap();
        assert_eq!(result.contents.len(), 1);
        assert_eq!(result.contents[0].uri(), "docs://rust/readme");
    }

    #[tokio::test]
    async fn test_read_unknown_resource_error() {
        let registry = ResourceRegistry::new();
        let result = registry.read_resource("file:///nonexistent").await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Unknown resource: file:///nonexistent")
        );
    }

    #[tokio::test]
    async fn test_subscribe_and_notify() {
        let registry = ResourceRegistry::new();
        let (tx, mut rx) = tokio::sync::broadcast::channel(16);

        registry
            .subscribe("file:///readme.md", "session-1".to_string(), tx)
            .await;

        registry.notify("file:///readme.md").await;

        let msg = rx.try_recv().unwrap();
        assert!(msg.contains("notifications/resources/updated"));
        assert!(msg.contains("file:///readme.md"));
    }

    #[tokio::test]
    async fn test_notify_with_contents_sends_correct_json() {
        let registry = ResourceRegistry::new();
        let (tx, mut rx) = tokio::sync::broadcast::channel(16);
        registry
            .subscribe("file:///readme.md", "session-1".to_string(), tx)
            .await;

        let contents = ReadResourceResult::text("file:///readme.md", "# Updated content");
        registry
            .notify_with_contents("file:///readme.md", &contents)
            .await;

        let msg = rx.try_recv().unwrap();
        let json: serde_json::Value = serde_json::from_str(&msg).unwrap();
        assert_eq!(json["method"], "notifications/resources/updated");
        assert_eq!(json["params"]["uri"], "file:///readme.md");
        let notified_contents = json["params"]["contents"].as_array().unwrap();
        assert_eq!(notified_contents.len(), 1);
        assert_eq!(notified_contents[0]["type"], "text");
        assert_eq!(notified_contents[0]["uri"], "file:///readme.md");
        assert_eq!(notified_contents[0]["text"], "# Updated content");
    }

    #[tokio::test]
    async fn test_notify_with_contents_blob() {
        let registry = ResourceRegistry::new();
        let (tx, mut rx) = tokio::sync::broadcast::channel(16);
        registry
            .subscribe("file:///data.bin", "session-1".to_string(), tx.clone())
            .await;

        let contents = ReadResourceResult::from_blob("file:///data.bin", vec![0, 1, 2, 3]);
        registry
            .notify_with_contents("file:///data.bin", &contents)
            .await;

        let msg = rx.try_recv().unwrap();
        let json: serde_json::Value = serde_json::from_str(&msg).unwrap();
        assert_eq!(json["params"]["uri"], "file:///data.bin");
        assert_eq!(json["params"]["contents"][0]["type"], "blob");
        assert_eq!(json["params"]["contents"][0]["uri"], "file:///data.bin");
        assert!(
            !json["params"]["contents"][0]["blob"]
                .as_str()
                .unwrap()
                .is_empty()
        );
    }

    #[tokio::test]
    async fn test_unsubscribe_stops_notifications() {
        let registry = ResourceRegistry::new();
        let (tx, mut rx) = tokio::sync::broadcast::channel(16);

        registry
            .subscribe("file:///readme.md", "session-1".to_string(), tx)
            .await;

        registry.unsubscribe("file:///readme.md", "session-1").await;

        registry.notify("file:///readme.md").await;

        // No message should be received after unsubscribe (try_recv returns Err)
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn test_notify_no_subscribers_does_not_panic() {
        let registry = ResourceRegistry::new();
        // Should not panic when no subscribers exist
        registry.notify("file:///readme.md").await;
    }

    #[tokio::test]
    async fn test_concrete_and_template_separation() {
        let mut registry = ResourceRegistry::new();
        registry.register(Box::new(ConcrRes {
            uri: "file:///readme.md".to_string(),
            name: "README".to_string(),
        }));
        registry.register(Box::new(TmplRes {
            uri_template: "docs://{topic}/readme".to_string(),
            name: "Docs".to_string(),
        }));

        assert_eq!(registry.list_resources().len(), 1);
        assert_eq!(registry.list_templates().len(), 1);
        assert_eq!(registry.len(), 2);
    }
}
