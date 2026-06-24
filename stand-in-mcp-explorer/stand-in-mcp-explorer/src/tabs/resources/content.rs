//! Pure functions for resource content classification and template substitution.
//!
//! Zero gpui context, zero panic on malformed input. Testable without a Window.

use stand_in_client::prelude::{ReadResourceResult, ResourceContents};

/// Outcome of classifying a `ReadResourceResult`.
#[derive(Debug, Clone, PartialEq)]
pub enum ContentKind {
    /// Text content where the mime type indicates JSON or the text looks like
    /// JSON (starts with `{` or `[`).
    Json { text: String },
    /// Plain text content (non-JSON).
    Text { text: String },
    /// Binary content — only the byte count (decoded from base64 blob) is
    /// stored; no actual preview.
    Binary { bytes: usize },
}

/// Classify a `ReadResourceResult` into its content kind.
///
/// Walks `result.contents`, picking the first non-empty entry. Non-JSON text
/// mime types yield `Text`. `application/json` or text that starts with `{`
/// or `[` yields `Json`. Blob contents yield `Binary` with the decoded byte
/// count (defensive: invalid base64 → 0 bytes).
pub fn classify_content(result: &ReadResourceResult) -> Option<ContentKind> {
    for contents in &result.contents {
        match contents {
            ResourceContents::Text {
                text, mime_type, ..
            } if !text.is_empty() => {
                let is_json = mime_type.as_deref().is_some_and(|m| m.contains("json"))
                    || text.trim_start().starts_with('{')
                    || text.trim_start().starts_with('[');
                return Some(if is_json {
                    ContentKind::Json { text: text.clone() }
                } else {
                    ContentKind::Text { text: text.clone() }
                });
            }
            ResourceContents::Blob { blob, .. } if !blob.is_empty() => {
                let bytes = estimated_bytes(blob);
                return Some(ContentKind::Binary { bytes });
            }
            _ => continue,
        }
    }
    None
}

fn estimated_bytes(blob: &str) -> usize {
    // base64 encodes 3 bytes → 4 chars; rough estimate: len * 3 / 4
    // Defensive: fall back to blob string length if decoding not available.
    (blob.len() * 3).div_ceil(4)
}

/// Substitute `{param}` placeholders in a URI template with provided values.
///
/// Returns the substituted URI string. Parameters without a matching key are
/// left as-is (unsubstituted).
pub fn substitute_template(uri_template: &str, params: &[(&str, &str)]) -> String {
    let mut result = uri_template.to_string();
    for (key, value) in params {
        let placeholder = format!("{{{}}}", key);
        result = result.replace(&placeholder, value);
    }
    result
}

/// Extract `{param}` names from a URI template string.
pub fn template_params(uri_template: &str) -> Vec<String> {
    let mut params = Vec::new();
    let chars: Vec<char> = uri_template.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '{' {
            let start = i + 1;
            while i < chars.len() && chars[i] != '}' {
                i += 1;
            }
            if i < chars.len() && start < i {
                let param: String = chars[start..i].iter().collect();
                if !param.is_empty() && !params.contains(&param) {
                    params.push(param);
                }
            }
        }
        i += 1;
    }
    params
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- classify_content -------------------------------------------------

    #[test]
    fn test_classify_text_json_mime() {
        let result =
            ReadResourceResult::text_with_mime("file:///a", r#"{"key":"val"}"#, "application/json");
        let kind = classify_content(&result).unwrap();
        assert!(matches!(kind, ContentKind::Json { .. }));
        if let ContentKind::Json { text } = kind {
            assert!(text.contains("key"));
        }
    }

    #[test]
    fn test_classify_text_plain() {
        let result = ReadResourceResult::text("file:///a", "hello world");
        let kind = classify_content(&result).unwrap();
        assert!(matches!(kind, ContentKind::Text { .. }));
        if let ContentKind::Text { text } = kind {
            assert_eq!(text, "hello world");
        }
    }

    #[test]
    fn test_classify_text_json_heuristic_brace() {
        let result = ReadResourceResult::text("file:///a", r#"{"name":"test"}"#);
        let kind = classify_content(&result).unwrap();
        assert!(matches!(kind, ContentKind::Json { .. }));
    }

    #[test]
    fn test_classify_text_json_heuristic_bracket() {
        let result = ReadResourceResult::text("file:///a", r#"[1,2,3]"#);
        let kind = classify_content(&result).unwrap();
        assert!(matches!(kind, ContentKind::Json { .. }));
    }

    #[test]
    fn test_classify_blob() {
        // "aGVsbG8gYmluYXJ5" = base64("hello binary") = 12 bytes
        let b64 = "aGVsbG8gYmluYXJ5";
        let result = ReadResourceResult::blob("file:///b", b64);
        let kind = classify_content(&result).unwrap();
        // estimated: 16 * 3 / 4 = 12
        assert!(matches!(kind, ContentKind::Binary { bytes: 12 }));
    }

    #[test]
    fn test_classify_empty_result() {
        let result = ReadResourceResult { contents: vec![] };
        assert!(classify_content(&result).is_none());
    }

    #[test]
    fn test_classify_empty_text() {
        let result = ReadResourceResult::text("file:///a", "");
        assert!(classify_content(&result).is_none());
    }

    // --- substitute_template -----------------------------------------------

    #[test]
    fn test_substitute_simple() {
        let result = substitute_template("docs://{topic}/readme", &[("topic", "rust")]);
        assert_eq!(result, "docs://rust/readme");
    }

    #[test]
    fn test_substitute_multiple() {
        let result = substitute_template(
            "file:///{org}/{repo}/{path}",
            &[("org", "foo"), ("repo", "bar"), ("path", "README.md")],
        );
        assert_eq!(result, "file:///foo/bar/README.md");
    }

    #[test]
    fn test_substitute_no_params() {
        let result = substitute_template("docs:///readme", &[]);
        assert_eq!(result, "docs:///readme");
    }

    #[test]
    fn test_substitute_missing_param() {
        let result = substitute_template("docs://{topic}/readme", &[("other", "val")]);
        assert_eq!(result, "docs://{topic}/readme");
    }

    // --- template_params ---------------------------------------------------

    #[test]
    fn test_template_params_single() {
        let params = template_params("docs://{topic}/readme");
        assert_eq!(params, vec!["topic"]);
    }

    #[test]
    fn test_template_params_multiple() {
        let params = template_params("file:///{org}/{repo}/{path}");
        assert_eq!(params, vec!["org", "repo", "path"]);
    }

    #[test]
    fn test_template_params_none() {
        let params = template_params("file:///readme");
        assert!(params.is_empty());
    }

    #[test]
    fn test_template_params_dedup() {
        let params = template_params("a://{x}/b/{x}");
        assert_eq!(params, vec!["x"]);
    }
}
