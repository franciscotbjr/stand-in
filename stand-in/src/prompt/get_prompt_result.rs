//! Result of a `prompts/get` JSON-RPC call.

use serde::{Deserialize, Serialize};

use super::PromptMessage;

/// The result returned by `prompts/get`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPromptResult {
    /// Optional description of the prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The rendered prompt messages.
    pub messages: Vec<PromptMessage>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompt::prompt_message::{PromptContent, PromptRole};

    #[test]
    fn test_result_serialize_with_description() {
        let result = GetPromptResult {
            description: Some("A greeting prompt".to_string()),
            messages: vec![PromptMessage {
                role: PromptRole::User,
                content: PromptContent::Text {
                    text: "Hello!".to_string(),
                },
            }],
        };
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["description"], "A greeting prompt");
        assert_eq!(json["messages"][0]["role"], "user");
        assert_eq!(json["messages"][0]["content"]["text"], "Hello!");
    }

    #[test]
    fn test_result_omits_absent_description() {
        let result = GetPromptResult {
            description: None,
            messages: vec![],
        };
        let json = serde_json::to_value(&result).unwrap();
        assert!(json.get("description").is_none());
    }
}
