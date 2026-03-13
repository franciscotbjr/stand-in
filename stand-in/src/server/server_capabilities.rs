//! Top-level capability advertisement for an MCP server.

use serde::{Deserialize, Serialize};

use super::ToolsCapability;

/// Top-level capability advertisement for an MCP server.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerCapabilities {
    /// Tools capability.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsCapability>,
}

impl ServerCapabilities {
    /// Create empty capabilities.
    pub fn new() -> Self {
        Self { tools: None }
    }

    /// Advertise tool support.
    pub fn with_tools(mut self, tools: ToolsCapability) -> Self {
        self.tools = Some(tools);
        self
    }
}

impl Default for ServerCapabilities {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capabilities_serialize() {
        let caps = ServerCapabilities::new();
        let json = serde_json::to_value(&caps).unwrap();
        assert!(json.get("tools").is_none());
    }

    #[test]
    fn test_capabilities_with_tools() {
        let caps = ServerCapabilities::new().with_tools(ToolsCapability::default());
        let json = serde_json::to_value(&caps).unwrap();
        assert!(json.get("tools").is_some());
    }
}
