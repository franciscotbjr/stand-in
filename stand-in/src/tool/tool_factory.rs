//! Factory function type for creating tool instances via inventory.

use super::McpTool;

/// A factory function that creates a boxed `McpTool` instance.
///
/// Used by `inventory` to collect tools registered with `#[mcp_tool]`.
pub struct ToolFactory(pub fn() -> Box<dyn McpTool>);

inventory::collect!(ToolFactory);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_factory_collects() {
        // Verify inventory collection doesn't panic
        let _count = inventory::iter::<ToolFactory>.into_iter().count();
    }
}
