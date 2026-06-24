//! MCP tool registry — manages the collection of available tools.

use crate::error::{Error, Result};

use super::McpTool;
use super::{CallToolResult, ToolDefinition};

/// Registry of all available MCP tools.
#[derive(Debug, Default)]
pub struct ToolRegistry {
    tools: Vec<Box<dyn McpTool>>,
}

impl ToolRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self { tools: Vec::new() }
    }

    /// Register a tool.
    pub fn register(&mut self, tool: Box<dyn McpTool>) {
        self.tools.push(tool);
    }

    /// Number of registered tools.
    pub fn len(&self) -> usize {
        self.tools.len()
    }

    /// Returns `true` if no tools are registered.
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    /// List all registered tool definitions.
    pub fn list_definitions(&self) -> Vec<ToolDefinition> {
        self.tools.iter().map(|t| t.to_definition()).collect()
    }

    /// Find and execute a tool by name.
    pub async fn call_tool(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> Result<CallToolResult> {
        let tool = self
            .tools
            .iter()
            .find(|t| t.name() == name)
            .ok_or_else(|| Error::ToolError(format!("Unknown tool: {name}")))?;

        tool.execute(arguments).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    #[derive(Debug)]
    struct FakeTool {
        tool_name: String,
    }

    impl FakeTool {
        fn new(name: &str) -> Self {
            Self {
                tool_name: name.to_string(),
            }
        }
    }

    #[async_trait]
    impl McpTool for FakeTool {
        fn name(&self) -> &str {
            &self.tool_name
        }

        fn description(&self) -> &str {
            "A fake tool"
        }

        fn input_schema(&self) -> serde_json::Value {
            serde_json::json!({"type": "object"})
        }

        async fn execute(&self, _arguments: serde_json::Value) -> Result<CallToolResult> {
            Ok(CallToolResult::text(format!("executed {}", self.tool_name)))
        }
    }

    #[test]
    fn test_empty_registry() {
        let registry = ToolRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
        assert!(registry.list_definitions().is_empty());
    }

    #[test]
    fn test_register_and_list() {
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FakeTool::new("alpha")));
        registry.register(Box::new(FakeTool::new("beta")));
        assert_eq!(registry.len(), 2);
        assert!(!registry.is_empty());
        let defs = registry.list_definitions();
        assert_eq!(defs[0].name, "alpha");
        assert_eq!(defs[1].name, "beta");
    }

    #[tokio::test]
    async fn test_call_existing_tool() {
        let mut registry = ToolRegistry::new();
        registry.register(Box::new(FakeTool::new("greet")));
        let result = registry
            .call_tool("greet", serde_json::json!({}))
            .await
            .unwrap();
        match &result.content[0] {
            super::super::Content::Text { text } => assert_eq!(text, "executed greet"),
        }
    }

    #[tokio::test]
    async fn test_call_unknown_tool_error() {
        let registry = ToolRegistry::new();
        let result = registry
            .call_tool("nonexistent", serde_json::json!({}))
            .await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Unknown tool: nonexistent"));
    }
}
