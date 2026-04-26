//! JSON-RPC method dispatch for MCP servers.

use tracing::{debug, error, info, warn};

use crate::prompt::{GetPromptParams, PromptRegistry};
use crate::protocol::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use crate::resource::{ReadResourceParams, ResourceRegistry, SubscribeParams, UnsubscribeParams};
use crate::tool::{CallToolParams, ToolRegistry};

use super::{
    InitializeResult, PromptsCapability, ResourcesCapability, ServerCapabilities, ServerInfo,
    ToolsCapability,
};

/// Dispatches incoming JSON-RPC requests to the appropriate handler.
#[derive(Debug)]
pub struct RequestHandler {
    registry: ToolRegistry,
    prompt_registry: PromptRegistry,
    resource_registry: ResourceRegistry,
    server_info: ServerInfo,
}

impl RequestHandler {
    /// Create a new request handler.
    pub fn new(
        registry: ToolRegistry,
        prompt_registry: PromptRegistry,
        resource_registry: ResourceRegistry,
        server_info: ServerInfo,
    ) -> Self {
        Self {
            registry,
            prompt_registry,
            resource_registry,
            server_info,
        }
    }

    /// Dispatch a parsed JSON-RPC request and return a response.
    pub async fn handle(&self, request: &JsonRpcRequest) -> JsonRpcResponse {
        let id = request.id.clone().unwrap_or(serde_json::Value::Null);

        info!(method = %request.method, "Received JSON-RPC request");

        match request.method.as_str() {
            "initialize" => self.handle_initialize(id),
            "notifications/initialized" => {
                info!("Client initialized notification received");
                JsonRpcResponse::success(id, serde_json::json!({}))
            }
            "tools/list" => self.handle_tools_list(id),
            "tools/call" => self.handle_tools_call(id, &request.params).await,
            "prompts/list" => self.handle_prompts_list(id),
            "prompts/get" => self.handle_prompts_get(id, &request.params).await,
            "resources/list" => self.handle_resources_list(id),
            "resources/templates/list" => self.handle_resources_templates_list(id),
            "resources/read" => self.handle_resources_read(id, &request.params).await,
            "resources/subscribe" => self.handle_resources_subscribe(id, &request.params),
            "resources/unsubscribe" => self.handle_resources_unsubscribe(id, &request.params),
            method => {
                info!(method, "Unknown method requested");
                JsonRpcResponse::error(id, JsonRpcError::method_not_found(method))
            }
        }
    }

    /// Wire a subscription sender to a resource.
    ///
    /// Called by the transport layer after `resources/subscribe` succeeds,
    /// so the resource registry can send SSE notifications to the subscriber.
    pub async fn wire_resource_subscription(
        &self,
        uri: &str,
        session_id: &str,
        sender: tokio::sync::broadcast::Sender<String>,
    ) {
        self.resource_registry
            .subscribe(uri, session_id.to_string(), sender)
            .await;
    }

    /// Remove a subscription for a resource.
    pub async fn wire_resource_unsubscribe(&self, uri: &str, session_id: &str) {
        self.resource_registry.unsubscribe(uri, session_id).await;
    }

    /// Return a reference to the resource registry.
    pub fn resource_registry(&self) -> &ResourceRegistry {
        &self.resource_registry
    }

    fn handle_initialize(&self, id: serde_json::Value) -> JsonRpcResponse {
        info!(
            server = %self.server_info.name,
            version = %self.server_info.version,
            "Handling initialize — protocol 2025-03-26"
        );
        let result = InitializeResult {
            protocol_version: "2025-03-26".to_string(),
            capabilities: ServerCapabilities::new()
                .with_tools(ToolsCapability::default())
                .with_prompts(PromptsCapability::default())
                .with_resources(ResourcesCapability {
                    subscribe: Some(true),
                    list_changed: Some(true),
                }),
            server_info: self.server_info.clone(),
        };

        JsonRpcResponse::success(id, serde_json::to_value(result).unwrap())
    }

    fn handle_tools_list(&self, id: serde_json::Value) -> JsonRpcResponse {
        let definitions = self.registry.list_definitions();
        info!(tool_count = definitions.len(), "Handling tools/list");
        let result = crate::tool::ListToolsResult { tools: definitions };
        JsonRpcResponse::success(id, serde_json::to_value(result).unwrap())
    }

    async fn handle_tools_call(
        &self,
        id: serde_json::Value,
        params: &Option<serde_json::Value>,
    ) -> JsonRpcResponse {
        let params = match params {
            Some(p) => p,
            None => {
                warn!("tools/call missing params");
                return JsonRpcResponse::error(
                    id,
                    JsonRpcError::invalid_params("Missing params for tools/call"),
                );
            }
        };

        let call_params: CallToolParams = match serde_json::from_value(params.clone()) {
            Ok(p) => p,
            Err(e) => {
                error!(error = %e, "tools/call param deserialization failed");
                return JsonRpcResponse::error(id, JsonRpcError::invalid_params(e.to_string()));
            }
        };

        info!(tool = %call_params.name, "Handling tools/call");

        match self
            .registry
            .call_tool(&call_params.name, call_params.arguments)
            .await
        {
            Ok(result) => {
                debug!(tool = %call_params.name, "tools/call succeeded");
                JsonRpcResponse::success(id, serde_json::to_value(result).unwrap())
            }
            Err(e) => {
                warn!(tool = %call_params.name, error = %e, "tools/call tool execution error");
                let error_result = crate::tool::CallToolResult::error(e.to_string());
                JsonRpcResponse::success(id, serde_json::to_value(error_result).unwrap())
            }
        }
    }

    fn handle_prompts_list(&self, id: serde_json::Value) -> JsonRpcResponse {
        let definitions = self.prompt_registry.list_definitions();
        info!(prompt_count = definitions.len(), "Handling prompts/list");
        let result = crate::prompt::ListPromptsResult {
            prompts: definitions,
        };
        JsonRpcResponse::success(id, serde_json::to_value(result).unwrap())
    }

    async fn handle_prompts_get(
        &self,
        id: serde_json::Value,
        params: &Option<serde_json::Value>,
    ) -> JsonRpcResponse {
        let params = match params {
            Some(p) => p,
            None => {
                warn!("prompts/get missing params");
                return JsonRpcResponse::error(
                    id,
                    JsonRpcError::invalid_params("Missing params for prompts/get"),
                );
            }
        };

        let get_params: GetPromptParams = match serde_json::from_value(params.clone()) {
            Ok(p) => p,
            Err(e) => {
                error!(error = %e, "prompts/get param deserialization failed");
                return JsonRpcResponse::error(id, JsonRpcError::invalid_params(e.to_string()));
            }
        };

        info!(prompt = %get_params.name, "Handling prompts/get");

        let arguments = get_params
            .arguments
            .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

        match self
            .prompt_registry
            .get_prompt(&get_params.name, arguments)
            .await
        {
            Ok(result) => {
                debug!(prompt = %get_params.name, "prompts/get succeeded");
                JsonRpcResponse::success(id, serde_json::to_value(result).unwrap())
            }
            Err(e) => {
                warn!(prompt = %get_params.name, error = %e, "prompts/get error");
                JsonRpcResponse::error(id, JsonRpcError::method_not_found(&get_params.name))
            }
        }
    }

    fn handle_resources_list(&self, id: serde_json::Value) -> JsonRpcResponse {
        let resources = self.resource_registry.list_resources();
        info!(resource_count = resources.len(), "Handling resources/list");
        let result = crate::resource::ListResourcesResult {
            resources,
            next_cursor: None,
        };
        JsonRpcResponse::success(id, serde_json::to_value(result).unwrap())
    }

    fn handle_resources_templates_list(&self, id: serde_json::Value) -> JsonRpcResponse {
        let templates = self.resource_registry.list_templates();
        info!(
            template_count = templates.len(),
            "Handling resources/templates/list"
        );
        let result = crate::resource::ListResourceTemplatesResult {
            resource_templates: templates,
            next_cursor: None,
        };
        JsonRpcResponse::success(id, serde_json::to_value(result).unwrap())
    }

    async fn handle_resources_read(
        &self,
        id: serde_json::Value,
        params: &Option<serde_json::Value>,
    ) -> JsonRpcResponse {
        let params = match params {
            Some(p) => p,
            None => {
                warn!("resources/read missing params");
                return JsonRpcResponse::error(
                    id,
                    JsonRpcError::invalid_params("Missing params for resources/read"),
                );
            }
        };

        let read_params: ReadResourceParams = match serde_json::from_value(params.clone()) {
            Ok(p) => p,
            Err(e) => {
                error!(error = %e, "resources/read param deserialization failed");
                return JsonRpcResponse::error(id, JsonRpcError::invalid_params(e.to_string()));
            }
        };

        info!(uri = %read_params.uri, "Handling resources/read");

        match self.resource_registry.read_resource(&read_params.uri).await {
            Ok(result) => {
                debug!(uri = %read_params.uri, "resources/read succeeded");
                JsonRpcResponse::success(id, serde_json::to_value(result).unwrap())
            }
            Err(e) => {
                warn!(uri = %read_params.uri, error = %e, "resources/read error");
                // Reuse -32601 here: the MCP spec doesn't define a "resource not found"
                // code, so we treat an unknown URI similarly to an unknown method.
                JsonRpcResponse::error(id, JsonRpcError::method_not_found(&read_params.uri))
            }
        }
    }

    fn handle_resources_subscribe(
        &self,
        id: serde_json::Value,
        params: &Option<serde_json::Value>,
    ) -> JsonRpcResponse {
        let params = match params {
            Some(p) => p,
            None => {
                warn!("resources/subscribe missing params");
                return JsonRpcResponse::error(
                    id,
                    JsonRpcError::invalid_params("Missing params for resources/subscribe"),
                );
            }
        };

        let subscribe_params: SubscribeParams = match serde_json::from_value(params.clone()) {
            Ok(p) => p,
            Err(e) => {
                error!(
                    error = %e,
                    "resources/subscribe param deserialization failed"
                );
                return JsonRpcResponse::error(id, JsonRpcError::invalid_params(e.to_string()));
            }
        };

        info!(uri = %subscribe_params.uri, "Handling resources/subscribe");
        // Subscription sender is wired by the transport layer after this response.
        JsonRpcResponse::success(id, serde_json::json!({}))
    }

    fn handle_resources_unsubscribe(
        &self,
        id: serde_json::Value,
        params: &Option<serde_json::Value>,
    ) -> JsonRpcResponse {
        let params = match params {
            Some(p) => p,
            None => {
                warn!("resources/unsubscribe missing params");
                return JsonRpcResponse::error(
                    id,
                    JsonRpcError::invalid_params("Missing params for resources/unsubscribe"),
                );
            }
        };

        let unsubscribe_params: UnsubscribeParams = match serde_json::from_value(params.clone()) {
            Ok(p) => p,
            Err(e) => {
                error!(
                    error = %e,
                    "resources/unsubscribe param deserialization failed"
                );
                return JsonRpcResponse::error(id, JsonRpcError::invalid_params(e.to_string()));
            }
        };

        info!(
            uri = %unsubscribe_params.uri,
            "Handling resources/unsubscribe"
        );
        // Unsubscription is handled by the transport layer after this response.
        JsonRpcResponse::success(id, serde_json::json!({}))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Result;
    use crate::prompt::{McpPrompt, Prompt, PromptArgument};
    use crate::resource::{McpResource, ReadResourceResult, Resource};
    use crate::tool::{CallToolResult, McpTool};
    use async_trait::async_trait;

    #[derive(Debug)]
    struct EchoTool;

    #[async_trait]
    impl McpTool for EchoTool {
        fn name(&self) -> &str {
            "echo"
        }

        fn description(&self) -> &str {
            "Echo back the input"
        }

        fn input_schema(&self) -> serde_json::Value {
            serde_json::json!({
                "type": "object",
                "properties": {"message": {"type": "string"}},
                "required": ["message"]
            })
        }

        async fn execute(&self, arguments: serde_json::Value) -> Result<CallToolResult> {
            let msg = arguments["message"].as_str().unwrap_or("no message");
            Ok(CallToolResult::text(msg))
        }
    }

    #[derive(Debug)]
    struct HelloPrompt;

    #[async_trait]
    impl McpPrompt for HelloPrompt {
        fn name(&self) -> &str {
            "hello"
        }

        fn description(&self) -> &str {
            "Say hello"
        }

        fn arguments(&self) -> Vec<PromptArgument> {
            vec![]
        }

        async fn execute(&self, _arguments: serde_json::Value) -> Result<Prompt> {
            Ok(Prompt::user("Hello!"))
        }
    }

    #[derive(Debug)]
    struct ReadmeResource;

    #[async_trait]
    impl McpResource for ReadmeResource {
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
                description: None,
                mime_type: None,
                size: None,
                annotations: None,
            })
        }
    }

    fn make_handler() -> RequestHandler {
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(EchoTool));
        let mut prompt_registry = PromptRegistry::new();
        prompt_registry.register(Box::new(HelloPrompt));
        let mut resource_registry = ResourceRegistry::new();
        resource_registry.register(Box::new(ReadmeResource));
        let server_info = ServerInfo {
            name: "test-server".to_string(),
            version: "0.1.0".to_string(),
        };
        RequestHandler::new(registry, prompt_registry, resource_registry, server_info)
    }

    fn make_request(
        method: &str,
        id: serde_json::Value,
        params: Option<serde_json::Value>,
    ) -> JsonRpcRequest {
        JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(id),
            method: method.to_string(),
            params,
        }
    }

    #[tokio::test]
    async fn test_handle_initialize() {
        let handler = make_handler();
        let req = make_request(
            "initialize",
            serde_json::json!(1),
            Some(serde_json::json!({
                "protocolVersion": "2025-03-26",
                "capabilities": {},
                "clientInfo": {"name": "test", "version": "1.0.0"}
            })),
        );
        let resp = handler.handle(&req).await;
        assert!(resp.error.is_none());
        let result = resp.result.unwrap();
        assert_eq!(result["protocolVersion"], "2025-03-26");
        assert_eq!(result["serverInfo"]["name"], "test-server");
        assert!(result["capabilities"]["prompts"].is_object());
        assert!(result["capabilities"]["resources"].is_object());
    }

    #[tokio::test]
    async fn test_handle_tools_list() {
        let handler = make_handler();
        let req = make_request("tools/list", serde_json::json!(2), None);
        let resp = handler.handle(&req).await;
        assert!(resp.error.is_none());
        let result = resp.result.unwrap();
        assert_eq!(result["tools"][0]["name"], "echo");
    }

    #[tokio::test]
    async fn test_handle_tools_call() {
        let handler = make_handler();
        let req = make_request(
            "tools/call",
            serde_json::json!(3),
            Some(serde_json::json!({
                "name": "echo",
                "arguments": {"message": "hello"}
            })),
        );
        let resp = handler.handle(&req).await;
        assert!(resp.error.is_none());
        let result = resp.result.unwrap();
        assert_eq!(result["content"][0]["text"], "hello");
    }

    #[tokio::test]
    async fn test_handle_unknown_method() {
        let handler = make_handler();
        let req = make_request("foo/bar", serde_json::json!(4), None);
        let resp = handler.handle(&req).await;
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, -32601);
    }

    #[tokio::test]
    async fn test_handle_missing_params() {
        let handler = make_handler();
        let req = make_request("tools/call", serde_json::json!(5), None);
        let resp = handler.handle(&req).await;
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, -32602);
    }

    #[tokio::test]
    async fn test_handle_prompts_list() {
        let handler = make_handler();
        let req = make_request("prompts/list", serde_json::json!(6), None);
        let resp = handler.handle(&req).await;
        assert!(resp.error.is_none());
        let result = resp.result.unwrap();
        assert_eq!(result["prompts"][0]["name"], "hello");
    }

    #[tokio::test]
    async fn test_handle_prompts_get() {
        let handler = make_handler();
        let req = make_request(
            "prompts/get",
            serde_json::json!(7),
            Some(serde_json::json!({"name": "hello"})),
        );
        let resp = handler.handle(&req).await;
        assert!(resp.error.is_none());
        let result = resp.result.unwrap();
        assert_eq!(result["messages"][0]["content"]["text"], "Hello!");
        assert_eq!(result["description"], "Say hello");
    }

    #[tokio::test]
    async fn test_handle_prompts_get_unknown() {
        let handler = make_handler();
        let req = make_request(
            "prompts/get",
            serde_json::json!(8),
            Some(serde_json::json!({"name": "nonexistent"})),
        );
        let resp = handler.handle(&req).await;
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, -32601);
    }

    #[tokio::test]
    async fn test_handle_prompts_get_missing_params() {
        let handler = make_handler();
        let req = make_request("prompts/get", serde_json::json!(9), None);
        let resp = handler.handle(&req).await;
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, -32602);
    }

    #[tokio::test]
    async fn test_handle_resources_list() {
        let handler = make_handler();
        let req = make_request("resources/list", serde_json::json!(10), None);
        let resp = handler.handle(&req).await;
        assert!(resp.error.is_none());
        let result = resp.result.unwrap();
        assert_eq!(result["resources"][0]["uri"], "file:///readme.md");
        assert_eq!(result["resources"][0]["name"], "README");
    }

    #[tokio::test]
    async fn test_handle_resources_list_empty() {
        let handler = RequestHandler::new(
            ToolRegistry::new(),
            PromptRegistry::new(),
            ResourceRegistry::new(),
            ServerInfo {
                name: "test".to_string(),
                version: "0.1.0".to_string(),
            },
        );
        let req = make_request("resources/list", serde_json::json!(11), None);
        let resp = handler.handle(&req).await;
        assert!(resp.error.is_none());
        let result = resp.result.unwrap();
        assert_eq!(result["resources"].as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_handle_resources_templates_list() {
        let handler = make_handler();
        let req = make_request("resources/templates/list", serde_json::json!(12), None);
        let resp = handler.handle(&req).await;
        assert!(resp.error.is_none());
        // README is concrete, not a template, so templates list should be empty
        let result = resp.result.unwrap();
        assert_eq!(result["resourceTemplates"].as_array().unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_handle_resources_read() {
        let handler = make_handler();
        let req = make_request(
            "resources/read",
            serde_json::json!(13),
            Some(serde_json::json!({"uri": "file:///readme.md"})),
        );
        let resp = handler.handle(&req).await;
        assert!(resp.error.is_none());
        let result = resp.result.unwrap();
        assert_eq!(result["contents"][0]["uri"], "file:///readme.md");
        assert_eq!(result["contents"][0]["text"], "# Hello");
    }

    #[tokio::test]
    async fn test_handle_resources_read_unknown() {
        let handler = make_handler();
        let req = make_request(
            "resources/read",
            serde_json::json!(14),
            Some(serde_json::json!({"uri": "file:///nonexistent"})),
        );
        let resp = handler.handle(&req).await;
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, -32601);
    }

    #[tokio::test]
    async fn test_handle_resources_subscribe() {
        let handler = make_handler();
        let req = make_request(
            "resources/subscribe",
            serde_json::json!(15),
            Some(serde_json::json!({"uri": "file:///readme.md"})),
        );
        let resp = handler.handle(&req).await;
        assert!(resp.error.is_none());
    }

    #[tokio::test]
    async fn test_handle_resources_unsubscribe() {
        let handler = make_handler();
        let req = make_request(
            "resources/unsubscribe",
            serde_json::json!(16),
            Some(serde_json::json!({"uri": "file:///readme.md"})),
        );
        let resp = handler.handle(&req).await;
        assert!(resp.error.is_none());
    }

    #[tokio::test]
    async fn test_handle_resources_subscribe_missing_params() {
        let handler = make_handler();
        let req = make_request("resources/subscribe", serde_json::json!(17), None);
        let resp = handler.handle(&req).await;
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, -32602);
    }

    #[tokio::test]
    async fn test_handle_resources_read_missing_params() {
        let handler = make_handler();
        let req = make_request("resources/read", serde_json::json!(18), None);
        let resp = handler.handle(&req).await;
        assert!(resp.error.is_some());
        assert_eq!(resp.error.unwrap().code, -32602);
    }
}
