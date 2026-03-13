//! Result of the `tools/list` method.

use serde::{Deserialize, Serialize};

use super::ToolDefinition;

/// Result of `tools/list`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListToolsResult {
    /// Available tool definitions.
    pub tools: Vec<ToolDefinition>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::InputSchema;

    #[test]
    fn test_list_result_serialize() {
        let result = ListToolsResult {
            tools: vec![ToolDefinition::new(
                "greet",
                "Greet someone",
                InputSchema::object(),
            )],
        };
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["tools"][0]["name"], "greet");
    }
}
