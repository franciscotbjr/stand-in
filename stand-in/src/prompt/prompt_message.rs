//! Message types returned as part of a prompt result.

use serde::{Deserialize, Serialize};

/// Role of a message participant.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PromptRole {
    /// A message from the user.
    User,
    /// A message from the assistant.
    Assistant,
}

/// Content of a prompt message.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PromptContent {
    /// Plain text content.
    #[serde(rename = "text")]
    Text {
        /// The text value.
        text: String,
    },
}

/// A single message in a prompt result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PromptMessage {
    /// Who authored this message.
    pub role: PromptRole,
    /// The message content.
    pub content: PromptContent,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_serializes_lowercase() {
        assert_eq!(
            serde_json::to_value(PromptRole::User).unwrap(),
            "user"
        );
        assert_eq!(
            serde_json::to_value(PromptRole::Assistant).unwrap(),
            "assistant"
        );
    }

    #[test]
    fn test_content_serializes_with_type_tag() {
        let content = PromptContent::Text {
            text: "Hello".to_string(),
        };
        let json = serde_json::to_value(&content).unwrap();
        assert_eq!(json["type"], "text");
        assert_eq!(json["text"], "Hello");
    }

    #[test]
    fn test_message_serialize() {
        let msg = PromptMessage {
            role: PromptRole::User,
            content: PromptContent::Text {
                text: "What is Rust?".to_string(),
            },
        };
        let json = serde_json::to_value(&msg).unwrap();
        assert_eq!(json["role"], "user");
        assert_eq!(json["content"]["type"], "text");
        assert_eq!(json["content"]["text"], "What is Rust?");
    }
}
