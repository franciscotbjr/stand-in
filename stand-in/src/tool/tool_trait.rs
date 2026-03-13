//! MCP tool trait — defines the interface each tool must implement.

use async_trait::async_trait;

use super::{CallToolResult, InputSchema, ToolDefinition};
use crate::error::Result;

/// Trait that each MCP tool must implement.
#[async_trait]
pub trait McpTool: Send + Sync + std::fmt::Debug {
    /// The tool name as exposed to MCP clients.
    fn name(&self) -> &str;

    /// Human-readable description of what the tool does.
    fn description(&self) -> &str;

    /// JSON Schema describing the tool's expected input.
    fn input_schema(&self) -> serde_json::Value;

    /// Execute the tool with the given arguments.
    async fn execute(&self, arguments: serde_json::Value) -> Result<CallToolResult>;

    /// Build a `ToolDefinition` from this tool's metadata.
    fn to_definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: self.name().to_string(),
            description: self.description().to_string(),
            input_schema: serde_json::from_value(self.input_schema()).unwrap_or_else(|_| {
                InputSchema::object()
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct DummyTool;

    #[async_trait]
    impl McpTool for DummyTool {
        fn name(&self) -> &str {
            "dummy"
        }

        fn description(&self) -> &str {
            "A dummy tool"
        }

        fn input_schema(&self) -> serde_json::Value {
            serde_json::json!({
                "type": "object",
                "properties": {
                    "msg": {"type": "string"}
                },
                "required": ["msg"]
            })
        }

        async fn execute(&self, _arguments: serde_json::Value) -> Result<CallToolResult> {
            Ok(CallToolResult::text("ok"))
        }
    }

    #[test]
    fn test_to_definition_default_impl() {
        let tool = DummyTool;
        let def = tool.to_definition();
        assert_eq!(def.name, "dummy");
        assert_eq!(def.description, "A dummy tool");
        assert_eq!(def.input_schema.schema_type, "object");
        assert!(def.input_schema.properties.is_some());
        assert_eq!(
            def.input_schema.required,
            Some(vec!["msg".to_string()])
        );
    }
}
