//! Server identity metadata.

use serde::{Deserialize, Serialize};

/// Server identity metadata sent during initialization.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ServerInfo {
    /// Server name.
    pub name: String,

    /// Server version.
    pub version: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_info_serialize() {
        let info = ServerInfo {
            name: "my-server".to_string(),
            version: "0.1.0".to_string(),
        };
        let json = serde_json::to_value(&info).unwrap();
        assert_eq!(json["name"], "my-server");
        assert_eq!(json["version"], "0.1.0");
    }
}
