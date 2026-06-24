//! Pure JSON tokenizer — walks a JSON string once, producing colored spans.
//!
//! Tolerant: malformed JSON does not panic; unrecognized characters fall back
//! to `Punc` or raw text. The tokenizer is a pure function with no dependencies
//! — ideal for exhaustive unit testing.

use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonToken {
    Key,
    Str,
    Num,
    Bool,
    Punc,
}

pub fn tokenize(json: &str) -> Vec<(Range<usize>, JsonToken)> {
    let mut tokens: Vec<(Range<usize>, JsonToken)> = Vec::new();
    let bytes = json.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        let ch = bytes[i];

        match ch {
            b'{' | b'}' | b'[' | b']' | b',' | b':' => {
                tokens.push((i..i + 1, JsonToken::Punc));
                i += 1;
            }
            b'"' => {
                let start = i;
                i = consume_string(bytes, i);
                let range = start..i;
                let kind = resolve_string_kind(&tokens, json, i, len);
                tokens.push((range, kind));
            }
            b't' | b'f' | b'n' if try_eat_literal(bytes, &mut i) => {
                let end = i;
                tokens.push((end - word_len(bytes, end)..end, JsonToken::Bool));
            }
            b'-' | b'0'..=b'9' => {
                let start = i;
                i = consume_number(bytes, i);
                tokens.push((start..i, JsonToken::Num));
            }
            b' ' | b'\t' | b'\n' | b'\r' => {
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }

    tokens
}

fn consume_string(bytes: &[u8], start: usize) -> usize {
    let len = bytes.len();
    let mut i = start + 1;
    while i < len {
        match bytes[i] {
            b'\\' => {
                if i + 1 < len {
                    i += 2;
                } else {
                    return i + 1;
                }
            }
            b'"' => return i + 1,
            _ => i += 1,
        }
    }
    i
}

fn resolve_string_kind(
    tokens: &[(Range<usize>, JsonToken)],
    json: &str,
    pos: usize,
    len: usize,
) -> JsonToken {
    let mut skip = pos;
    while skip < len {
        match json.as_bytes()[skip] {
            b' ' | b'\t' | b'\n' | b'\r' => skip += 1,
            b':' => return JsonToken::Key,
            _ => break,
        }
    }

    for (range, kind) in tokens.iter().rev() {
        if *kind == JsonToken::Punc {
            if &json[range.start..range.end] == ":"
                && tokens.last().is_some_and(|(r, _)| r.end == range.start)
            {
                return JsonToken::Str;
            }
            break;
        }
    }

    JsonToken::Str
}

fn try_eat_literal(bytes: &[u8], i: &mut usize) -> bool {
    let remaining = &bytes[*i..];
    if remaining.len() >= 4 && &remaining[..4] == b"true" {
        *i += 4;
        true
    } else if remaining.len() >= 5 && &remaining[..5] == b"false" {
        *i += 5;
        true
    } else if remaining.len() >= 4 && &remaining[..4] == b"null" {
        *i += 4;
        true
    } else {
        false
    }
}

fn word_len(bytes: &[u8], end: usize) -> usize {
    if end >= 4 && (&bytes[end - 4..end] == b"true" || &bytes[end - 4..end] == b"null") {
        4
    } else if end >= 5 && &bytes[end - 5..end] == b"false" {
        5
    } else {
        0
    }
}

fn consume_number(bytes: &[u8], start: usize) -> usize {
    let len = bytes.len();
    let mut i = start;

    if i < len && bytes[i] == b'-' {
        i += 1;
    }

    while i < len && bytes[i].is_ascii_digit() {
        i += 1;
    }

    if i < len && bytes[i] == b'.' {
        i += 1;
        while i < len && bytes[i].is_ascii_digit() {
            i += 1;
        }
    }

    if i < len && (bytes[i] == b'e' || bytes[i] == b'E') {
        i += 1;
        if i < len && (bytes[i] == b'+' || bytes[i] == b'-') {
            i += 1;
        }
        while i < len && bytes[i].is_ascii_digit() {
            i += 1;
        }
    }

    i
}

#[cfg(test)]
mod tests {
    use super::*;

    fn collect(json: &str) -> Vec<(String, JsonToken)> {
        tokenize(json)
            .into_iter()
            .map(|(range, kind)| (json[range].to_string(), kind))
            .collect()
    }

    #[test]
    fn test_simple_object() {
        let t = collect(r#"{"name": "John", "age": 30}"#);
        assert!(t.contains(&("{".into(), JsonToken::Punc)));
        assert!(t.contains(&("\"name\"".into(), JsonToken::Key)));
        assert!(t.contains(&("\"John\"".into(), JsonToken::Str)));
        assert!(t.contains(&("\"age\"".into(), JsonToken::Key)));
        assert!(t.contains(&("30".into(), JsonToken::Num)));
    }

    #[test]
    fn test_nested_object() {
        let t = collect(r#"{"user": {"id": 1}"#);
        assert!(t.contains(&("\"user\"".into(), JsonToken::Key)));
        assert!(t.contains(&("\"id\"".into(), JsonToken::Key)));
        assert!(t.contains(&("{".into(), JsonToken::Punc)));
    }

    #[test]
    fn test_array() {
        let t = collect(r#"["a", 1, true]"#);
        assert!(t.contains(&("[".into(), JsonToken::Punc)));
        assert!(t.contains(&("\"a\"".into(), JsonToken::Str)));
        assert!(t.contains(&("1".into(), JsonToken::Num)));
        assert!(t.contains(&("true".into(), JsonToken::Bool)));
    }

    #[test]
    fn test_string_escapes() {
        let t = collect(r#"{"key": "val\"ue\\esc"}"#);
        assert!(t.contains(&("\"key\"".into(), JsonToken::Key)));
        assert!(t.contains(&("\"val\\\"ue\\\\esc\"".into(), JsonToken::Str)));
    }

    #[test]
    fn test_numbers_variety() {
        let t = collect(r#"[0, -5, 3.14, 1e10, -2.5E-3]"#);
        let nums: Vec<_> = t
            .iter()
            .filter(|(_, k)| *k == JsonToken::Num)
            .map(|(s, _)| s.clone())
            .collect();
        assert!(nums.contains(&"0".into()));
        assert!(nums.contains(&"-5".into()));
        assert!(nums.contains(&"3.14".into()));
        assert!(nums.contains(&"1e10".into()));
        assert!(nums.contains(&"-2.5E-3".into()));
    }

    #[test]
    fn test_bool_and_null() {
        let t = collect(r#"{"a": true, "b": false, "c": null}"#);
        let bools: Vec<_> = t
            .iter()
            .filter(|(_, k)| *k == JsonToken::Bool)
            .map(|(s, _)| s.clone())
            .collect();
        assert!(bools.contains(&"true".into()));
        assert!(bools.contains(&"false".into()));
        assert!(bools.contains(&"null".into()));
    }

    #[test]
    fn test_key_vs_value_string() {
        let t = collect(r#"{"key": "value"}"#);
        assert!(t.contains(&("\"key\"".into(), JsonToken::Key)));
        assert!(t.contains(&("\"value\"".into(), JsonToken::Str)));
    }

    #[test]
    fn test_keys_different_positions() {
        let t = collect(r#"{"a": 1, "b": 2}"#);
        assert!(t.contains(&("\"a\"".into(), JsonToken::Key)));
        assert!(t.contains(&("\"b\"".into(), JsonToken::Key)));
    }

    #[test]
    fn test_malformed_no_panic() {
        let t = collect(r#"{broken"#);
        assert!(!t.is_empty());
    }

    #[test]
    fn test_empty_json() {
        let t = collect("");
        assert!(t.is_empty());
    }

    #[test]
    fn test_unicode_in_string() {
        let t = collect(r#"{"greeting": "ol\u00e1"}"#);
        assert!(t.contains(&("\"greeting\"".into(), JsonToken::Key)));
        assert!(t.contains(&("\"ol\\u00e1\"".into(), JsonToken::Str)));
    }

    #[test]
    fn test_deep_nesting() {
        let t = collect(r#"{"a":{"b":{"c":{"d":42}}}}"#);
        assert!(t.contains(&("\"d\"".into(), JsonToken::Key)));
        assert!(t.contains(&("42".into(), JsonToken::Num)));
    }

    #[test]
    fn test_punct_spans() {
        let t = collect(r#"{"x":[1,2]}"#);
        let punc: Vec<_> = t
            .iter()
            .filter(|(_, k)| *k == JsonToken::Punc)
            .map(|(s, _)| s.clone())
            .collect();
        assert!(punc.contains(&"{".into()));
        assert!(punc.contains(&"}".into()));
        assert!(punc.contains(&"[".into()));
        assert!(punc.contains(&"]".into()));
        assert!(punc.contains(&",".into()));
        assert!(punc.contains(&":".into()));
    }

    #[test]
    fn test_tokenize_is_pure() {
        let a = collect(r#"{"x": 1}"#);
        let b = collect(r#"{"x": 1}"#);
        assert_eq!(a, b);
    }
}
