//! Parameters for the `initialize` method.

use serde::{Deserialize, Serialize};

use super::ClientInfo;

/// Parameters for the `initialize` method (client → server).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InitializeParams {
    /// Protocol version the client supports.
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,

    /// Client capabilities (currently unused, reserved for future).
    #[serde(default)]
    pub capabilities: serde_json::Value,

    /// Client identity.
    #[serde(rename = "clientInfo")]
    pub client_info: ClientInfo,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize_params_deserialize() {
        let json = r#"{
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": {"name": "test-client", "version": "1.0.0"}
        }"#;
        let params: InitializeParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.protocol_version, "2025-03-26");
        assert_eq!(params.client_info.name, "test-client");
        assert_eq!(params.client_info.version, "1.0.0");
    }
}
