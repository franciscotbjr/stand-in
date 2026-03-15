//! Inventory-based factory for registering `#[mcp_prompt]` implementations.

use super::McpPrompt;

/// A factory function that constructs a boxed [`McpPrompt`].
///
/// Every `#[mcp_prompt]` macro expansion submits one of these via `inventory::submit!`.
pub struct PromptFactory(pub fn() -> Box<dyn McpPrompt>);

inventory::collect!(PromptFactory);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_factory_collects() {
        // inventory::iter returns all submitted factories; in a test binary there
        // are none unless a #[mcp_prompt] function is in scope — this just verifies
        // the collect! call compiles and the iterator is accessible.
        let _count = inventory::iter::<PromptFactory>.into_iter().count();
    }
}
