//! Result of the `initialize` method.

use serde::{Deserialize, Serialize};

use super::{ServerCapabilities, ServerInfo};

/// Result of the `initialize` method (server → client).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InitializeResult {
    /// Protocol version the server supports.
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,

    /// Server capabilities.
    pub capabilities: ServerCapabilities,

    /// Server identity.
    #[serde(rename = "serverInfo")]
    pub server_info: ServerInfo,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::ToolsCapability;

    #[test]
    fn test_initialize_result_serialize() {
        let result = InitializeResult {
            protocol_version: "2025-03-26".to_string(),
            capabilities: ServerCapabilities::new().with_tools(ToolsCapability::default()),
            server_info: ServerInfo {
                name: "test-server".to_string(),
                version: "0.1.0".to_string(),
            },
        };
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["protocolVersion"], "2025-03-26");
        assert_eq!(json["serverInfo"]["name"], "test-server");
        assert!(json["capabilities"]["tools"].is_object());
    }
}
