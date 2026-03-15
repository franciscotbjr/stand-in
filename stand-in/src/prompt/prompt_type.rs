//! The `Prompt` type returned by `#[mcp_prompt]` functions.

use super::prompt_message::{PromptContent, PromptMessage, PromptRole};

/// The return type for `#[mcp_prompt]` functions.
///
/// Contains one or more messages that form the prompt template.
///
/// # Example
///
/// ```rust,no_run
/// # use stand_in::prompt::Prompt;
/// let p = Prompt::user("Summarize this document");
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Prompt {
    /// Messages that make up this prompt.
    pub messages: Vec<PromptMessage>,
}

impl Prompt {
    /// Create a prompt with a single user message.
    pub fn user(text: impl Into<String>) -> Self {
        Self {
            messages: vec![PromptMessage {
                role: PromptRole::User,
                content: PromptContent::Text { text: text.into() },
            }],
        }
    }

    /// Create a prompt with a single assistant message.
    pub fn assistant(text: impl Into<String>) -> Self {
        Self {
            messages: vec![PromptMessage {
                role: PromptRole::Assistant,
                content: PromptContent::Text { text: text.into() },
            }],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_creates_single_user_message() {
        let p = Prompt::user("Hello");
        assert_eq!(p.messages.len(), 1);
        assert_eq!(p.messages[0].role, PromptRole::User);
        assert_eq!(
            p.messages[0].content,
            PromptContent::Text {
                text: "Hello".to_string()
            }
        );
    }

    #[test]
    fn test_assistant_creates_single_assistant_message() {
        let p = Prompt::assistant("I can help");
        assert_eq!(p.messages.len(), 1);
        assert_eq!(p.messages[0].role, PromptRole::Assistant);
        assert_eq!(
            p.messages[0].content,
            PromptContent::Text {
                text: "I can help".to_string()
            }
        );
    }
}
