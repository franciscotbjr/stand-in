//! Top-level capability advertisement for an MCP server.

use serde::{Deserialize, Serialize};

use super::{PromptsCapability, ToolsCapability};

/// Top-level capability advertisement for an MCP server.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerCapabilities {
    /// Tools capability.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsCapability>,

    /// Prompts capability.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<PromptsCapability>,
}

impl ServerCapabilities {
    /// Create empty capabilities.
    pub fn new() -> Self {
        Self {
            tools: None,
            prompts: None,
        }
    }

    /// Advertise tool support.
    pub fn with_tools(mut self, tools: ToolsCapability) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Advertise prompt support.
    pub fn with_prompts(mut self, prompts: PromptsCapability) -> Self {
        self.prompts = Some(prompts);
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

    #[test]
    fn test_capabilities_with_prompts() {
        let caps = ServerCapabilities::new().with_prompts(PromptsCapability::default());
        let json = serde_json::to_value(&caps).unwrap();
        assert!(json.get("prompts").is_some());
        assert!(json.get("tools").is_none());
    }

    #[test]
    fn test_capabilities_with_tools_and_prompts() {
        let caps = ServerCapabilities::new()
            .with_tools(ToolsCapability::default())
            .with_prompts(PromptsCapability::default());
        let json = serde_json::to_value(&caps).unwrap();
        assert!(json.get("tools").is_some());
        assert!(json.get("prompts").is_some());
    }
}
