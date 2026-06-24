//! The result returned by a tool invocation.

use serde::{Deserialize, Serialize};

use super::Content;

/// The result returned by a tool invocation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CallToolResult {
    /// Content blocks.
    pub content: Vec<Content>,

    /// Whether this result represents an error.
    #[serde(rename = "isError", skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

impl CallToolResult {
    /// Create a successful text result.
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            content: vec![Content::text(text)],
            is_error: None,
        }
    }

    /// Create an error result.
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            content: vec![Content::text(message)],
            is_error: Some(true),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_result() {
        let result = CallToolResult::text("hello");
        assert_eq!(result.content.len(), 1);
        match &result.content[0] {
            Content::Text { text } => assert_eq!(text, "hello"),
        }
        assert!(result.is_error.is_none());
    }

    #[test]
    fn test_error_result() {
        let result = CallToolResult::error("something failed");
        match &result.content[0] {
            Content::Text { text } => assert_eq!(text, "something failed"),
        }
        assert_eq!(result.is_error, Some(true));
    }

    #[test]
    fn test_is_error_flag() {
        let ok = CallToolResult::text("ok");
        let err = CallToolResult::error("bad");
        let ok_json = serde_json::to_value(&ok).unwrap();
        let err_json = serde_json::to_value(&err).unwrap();
        assert!(ok_json.get("isError").is_none());
        assert_eq!(err_json["isError"], true);
    }
}
