//! MCP tool abstraction — trait, registry, and tool-related types.

mod call_tool_params;
mod call_tool_result;
mod content;
mod input_schema;
mod list_tools_result;
mod tool_definition;
mod tool_registry;
mod tool_trait;

pub use call_tool_params::CallToolParams;
pub use call_tool_result::CallToolResult;
pub use content::Content;
pub use input_schema::InputSchema;
pub use list_tools_result::ListToolsResult;
pub use tool_definition::ToolDefinition;
pub use tool_registry::ToolRegistry;
pub use tool_trait::McpTool;
