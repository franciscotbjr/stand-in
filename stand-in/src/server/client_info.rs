//! Client identity metadata.

use serde::{Deserialize, Serialize};

/// Client identity metadata received during initialization.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClientInfo {
    /// Client name.
    pub name: String,

    /// Client version.
    pub version: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_info_deserialize() {
        let json = r#"{"name":"claude-desktop","version":"1.0.0"}"#;
        let info: ClientInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.name, "claude-desktop");
        assert_eq!(info.version, "1.0.0");
    }
}
