//! Resource content types — `TextResourceContents` and `BlobResourceContents`.

use serde::{Deserialize, Serialize};

/// Contents of a resource, as returned by `resources/read`.
///
/// Two variants per the MCP spec:
/// - `Text` — plain text content (`TextResourceContents`).
/// - `Blob` — binary data as base64 (`BlobResourceContents`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ResourceContents {
    /// Text-based resource content.
    #[serde(rename = "text")]
    Text {
        /// URI of the resource.
        uri: String,

        /// MIME type of the content.
        #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
        mime_type: Option<String>,

        /// The text content.
        text: String,
    },

    /// Binary resource content (base64-encoded).
    #[serde(rename = "blob")]
    Blob {
        /// URI of the resource.
        uri: String,

        /// MIME type of the content.
        #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
        mime_type: Option<String>,

        /// Base64-encoded binary data.
        blob: String,
    },
}

impl ResourceContents {
    /// Create text resource contents for the given URI.
    pub fn text(uri: impl Into<String>, text: impl Into<String>) -> Self {
        Self::Text {
            uri: uri.into(),
            mime_type: None,
            text: text.into(),
        }
    }

    /// Create blob resource contents for the given URI.
    pub fn blob(uri: impl Into<String>, blob: impl Into<String>) -> Self {
        Self::Blob {
            uri: uri.into(),
            mime_type: None,
            blob: blob.into(),
        }
    }

    /// Attach a MIME type to the resource contents.
    pub fn with_mime_type(mut self, mime_type: impl Into<String>) -> Self {
        let mime = Some(mime_type.into());
        match &mut self {
            Self::Text { mime_type: mt, .. } | Self::Blob { mime_type: mt, .. } => *mt = mime,
        }
        self
    }

    /// Return the URI of the resource contents.
    pub fn uri(&self) -> &str {
        match self {
            Self::Text { uri, .. } | Self::Blob { uri, .. } => uri,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_contents_serialize() {
        let contents = ResourceContents::text("file:///readme.md", "# Hello");
        let json = serde_json::to_value(&contents).unwrap();
        assert_eq!(json["type"], "text");
        assert_eq!(json["uri"], "file:///readme.md");
        assert_eq!(json["text"], "# Hello");
        assert!(json.get("mimeType").is_none());
    }

    #[test]
    fn test_text_contents_with_mime() {
        let contents =
            ResourceContents::text("file:///readme.md", "# Hello").with_mime_type("text/markdown");
        let json = serde_json::to_value(&contents).unwrap();
        assert_eq!(json["mimeType"], "text/markdown");
    }

    #[test]
    fn test_blob_contents_serialize() {
        let contents = ResourceContents::blob("file:///image.png", "iVBORw0KGgo=");
        let json = serde_json::to_value(&contents).unwrap();
        assert_eq!(json["type"], "blob");
        assert_eq!(json["uri"], "file:///image.png");
        assert_eq!(json["blob"], "iVBORw0KGgo=");
    }

    #[test]
    fn test_blob_contents_with_mime() {
        let contents =
            ResourceContents::blob("file:///image.png", "iVBORw0KGgo=").with_mime_type("image/png");
        let json = serde_json::to_value(&contents).unwrap();
        assert_eq!(json["mimeType"], "image/png");
    }

    #[test]
    fn test_text_contents_deserialize() {
        let json = serde_json::json!({
            "type": "text",
            "uri": "file:///readme.md",
            "mimeType": "text/markdown",
            "text": "# Hello"
        });
        let contents: ResourceContents = serde_json::from_value(json).unwrap();
        match contents {
            ResourceContents::Text {
                uri,
                mime_type,
                text,
            } => {
                assert_eq!(uri, "file:///readme.md");
                assert_eq!(mime_type.unwrap(), "text/markdown");
                assert_eq!(text, "# Hello");
            }
            _ => panic!("expected Text variant"),
        }
    }

    #[test]
    fn test_blob_contents_deserialize() {
        let json = serde_json::json!({
            "type": "blob",
            "uri": "file:///image.png",
            "mimeType": "image/png",
            "blob": "iVBORw0KGgo="
        });
        let contents: ResourceContents = serde_json::from_value(json).unwrap();
        match contents {
            ResourceContents::Blob {
                uri,
                mime_type,
                blob,
            } => {
                assert_eq!(uri, "file:///image.png");
                assert_eq!(mime_type.unwrap(), "image/png");
                assert_eq!(blob, "iVBORw0KGgo=");
            }
            _ => panic!("expected Blob variant"),
        }
    }

    #[test]
    fn test_uri_accessor() {
        let text = ResourceContents::text("file:///a.txt", "hello");
        assert_eq!(text.uri(), "file:///a.txt");

        let blob = ResourceContents::blob("file:///b.png", "abc");
        assert_eq!(blob.uri(), "file:///b.png");
    }
}
