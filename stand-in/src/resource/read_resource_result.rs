//! Result of a `resources/read` JSON-RPC call.

use serde::{Deserialize, Serialize};

use super::ResourceContents;

/// The result returned by `resources/read`.
///
/// Contains the resource contents, which may be one or more content
/// blocks (text or blob).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadResourceResult {
    /// Resource contents. Typically a single entry, but may return
    /// multiple content blocks for composite resources.
    pub contents: Vec<ResourceContents>,
}

impl ReadResourceResult {
    /// Create a result containing a single text content block.
    pub fn text(uri: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            contents: vec![ResourceContents::text(uri, text)],
        }
    }

    /// Create a text result with a MIME type.
    pub fn text_with_mime(
        uri: impl Into<String>,
        text: impl Into<String>,
        mime_type: impl Into<String>,
    ) -> Self {
        Self {
            contents: vec![ResourceContents::text(uri, text).with_mime_type(mime_type)],
        }
    }

    /// Create a result containing a single blob content block.
    pub fn blob(uri: impl Into<String>, blob: impl Into<String>) -> Self {
        Self {
            contents: vec![ResourceContents::blob(uri, blob)],
        }
    }

    /// Create a result from raw bytes, base64-encoding them as blob content.
    pub fn from_blob(uri: impl Into<String>, data: Vec<u8>) -> Self {
        use base64::Engine;
        let encoded = base64::engine::general_purpose::STANDARD.encode(&data);
        Self {
            contents: vec![ResourceContents::blob(uri, encoded)],
        }
    }

    /// Create a result with multiple content blocks.
    pub fn with_contents(contents: Vec<ResourceContents>) -> Self {
        Self { contents }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_result_serialize() {
        let result = ReadResourceResult::text("file:///readme.md", "# Hello");
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["contents"][0]["type"], "text");
        assert_eq!(json["contents"][0]["uri"], "file:///readme.md");
        assert_eq!(json["contents"][0]["text"], "# Hello");
    }

    #[test]
    fn test_multi_contents_result_serialize() {
        let result = ReadResourceResult::with_contents(vec![
            ResourceContents::text("file:///a.txt", "A"),
            ResourceContents::text("file:///b.txt", "B"),
        ]);
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["contents"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_blob_result_serialize() {
        let result = ReadResourceResult::blob("file:///img.png", "abcd");
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["contents"][0]["type"], "blob");
        assert_eq!(json["contents"][0]["blob"], "abcd");
    }
}
