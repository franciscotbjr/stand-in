//! Content block inside a tool result.

use serde::{Deserialize, Serialize};

/// Content block inside a tool result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Content {
    /// Text content.
    #[serde(rename = "text")]
    Text {
        /// The text value.
        text: String,
    },
}

impl Content {
    /// Create a text content block.
    pub fn text(text: impl Into<String>) -> Self {
        Content::Text { text: text.into() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_content() {
        let content = Content::text("hello");
        match &content {
            Content::Text { text } => assert_eq!(text, "hello"),
        }
    }

    #[test]
    fn test_text_content_serialize() {
        let content = Content::text("hello");
        let json = serde_json::to_value(&content).unwrap();
        assert_eq!(json["type"], "text");
        assert_eq!(json["text"], "hello");
    }
}
