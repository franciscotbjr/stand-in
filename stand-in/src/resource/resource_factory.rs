//! Inventory-based factory for registering `#[mcp_resource]` implementations.

use super::McpResource;

/// A factory function that constructs a boxed [`McpResource`].
///
/// Every `#[mcp_resource]` macro expansion submits one of these via `inventory::submit!`.
pub struct ResourceFactory(pub fn() -> Box<dyn McpResource>);

inventory::collect!(ResourceFactory);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_factory_collects() {
        // inventory::iter returns all submitted factories; in a test binary there
        // are none unless a #[mcp_resource] function is in scope — this just verifies
        // the collect! call compiles and the iterator is accessible.
        let _count = inventory::iter::<ResourceFactory>.into_iter().count();
    }
}
