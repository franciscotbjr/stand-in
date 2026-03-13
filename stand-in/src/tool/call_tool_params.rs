//! Parameters for the `tools/call` method.

use serde::{Deserialize, Serialize};

/// Parameters for `tools/call`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CallToolParams {
    /// Tool name to invoke.
    pub name: String,

    /// Tool arguments as a JSON object.
    #[serde(default)]
    pub arguments: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_params_deserialize() {
        let json = r#"{"name":"greet","arguments":{"name":"World"}}"#;
        let params: CallToolParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.name, "greet");
        assert_eq!(params.arguments["name"], "World");
    }

    #[test]
    fn test_params_default_arguments() {
        let json = r#"{"name":"ping"}"#;
        let params: CallToolParams = serde_json::from_str(json).unwrap();
        assert_eq!(params.name, "ping");
        assert_eq!(params.arguments, serde_json::Value::Null);
    }
}
