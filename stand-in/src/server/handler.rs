//! JSON-RPC method dispatch for MCP servers.

use tracing::{debug, error, info, warn};

use crate::prompt::{GetPromptParams, PromptRegistry};
use crate::protocol::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use crate::tool::{CallToolParams, ToolRegistry};

use super::{InitializeResult, PromptsCapability, ServerCapabilities, ServerInfo, ToolsCapability};

/// Dispatches incoming JSON-RPC requests to the appropriate handler.
#[derive(Debug)]
pub struct RequestHandler {
    registry: ToolRegistry,
    prompt_registry: PromptRegistry,
    server_info: ServerInfo,
}

impl RequestHandler {
    /// Create a new request handler.
    pub fn new(
        registry: ToolRegistry,
        prompt_registry: PromptRegistry,
        server_info: ServerInfo,
    ) -> Self {
        Self {
            registry,
            prompt_registry,
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
            method => {
                info!(method, "Unknown method requested");
                JsonRpcResponse::error(id, JsonRpcError::method_not_found(method))
            }
        }
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
                .with_prompts(PromptsCapability::default()),
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Result;
    use crate::prompt::{McpPrompt, Prompt, PromptArgument};
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

    fn make_handler() -> RequestHandler {
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(EchoTool));
        let mut prompt_registry = PromptRegistry::new();
        prompt_registry.register(Box::new(HelloPrompt));
        let server_info = ServerInfo {
            name: "test-server".to_string(),
            version: "0.1.0".to_string(),
        };
        RequestHandler::new(registry, prompt_registry, server_info)
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
}
