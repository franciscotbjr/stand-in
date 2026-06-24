//! Result of a `prompts/list` JSON-RPC call.

use serde::{Deserialize, Serialize};

use super::PromptDefinition;

/// The result returned by `prompts/list`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPromptsResult {
    /// All registered prompts.
    pub prompts: Vec<PromptDefinition>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_result_serialize() {
        let result = ListPromptsResult {
            prompts: vec![PromptDefinition {
                name: "hello".to_string(),
                description: "Say hello".to_string(),
                arguments: None,
            }],
        };
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["prompts"][0]["name"], "hello");
        assert_eq!(json["prompts"][0]["description"], "Say hello");
    }

    #[test]
    fn test_list_result_empty() {
        let result = ListPromptsResult { prompts: vec![] };
        let json = serde_json::to_value(&result).unwrap();
        assert!(json["prompts"].as_array().unwrap().is_empty());
    }
}
