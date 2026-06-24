//! Pure functions for prompt argument building, validation, and role labels.
//!
//! Zero gpui context, testable without a Window. Prompt arguments are always
//! strings — simpler than tools (no JSON Schema type inference).

use stand_in::prompt::PromptArgument;
use stand_in_client::prelude::{GetPromptResult, PromptRole};

/// Prompt execution lifecycle state.
#[derive(Debug, Clone, Default)]
pub enum PromptRun {
    #[default]
    Idle,
    Building,
    Messages(Box<GetPromptResult>),
    Error(String),
}

/// Build a `serde_json::Value` object from prompt arguments and their text
/// values. All arguments are strings. Empty non-required fields are omitted.
pub fn build_prompt_args(params: &[(PromptArgument, String)]) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for (arg, text) in params {
        let trimmed = text.trim();
        if trimmed.is_empty() && !arg.required.unwrap_or(false) {
            continue;
        }
        map.insert(
            arg.name.clone(),
            serde_json::Value::String(trimmed.to_string()),
        );
    }
    serde_json::Value::Object(map)
}

/// Returns names of required arguments whose text value is empty.
pub fn missing_required(params: &[(PromptArgument, String)]) -> Vec<&str> {
    params
        .iter()
        .filter(|(arg, text)| arg.required.unwrap_or(false) && text.trim().is_empty())
        .map(|(arg, _)| arg.name.as_str())
        .collect()
}

/// Map a `PromptRole` to a monospace display label.
pub fn role_label(role: &PromptRole) -> &'static str {
    match role {
        PromptRole::User => "user",
        PromptRole::Assistant => "assistant",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arg(name: &str, required: bool) -> PromptArgument {
        PromptArgument {
            name: name.into(),
            description: None,
            required: Some(required),
        }
    }

    fn opt_arg(name: &str) -> PromptArgument {
        PromptArgument {
            name: name.into(),
            description: None,
            required: None,
        }
    }

    #[test]
    fn test_build_prompt_args_single() {
        let v = build_prompt_args(&[(arg("name", true), "stand-in".into())]);
        assert_eq!(v["name"], "stand-in");
    }

    #[test]
    fn test_build_prompt_args_omits_empty_optional() {
        let v = build_prompt_args(&[(opt_arg("style"), "".into())]);
        let obj = v.as_object().unwrap();
        assert!(obj.is_empty());
    }

    #[test]
    fn test_build_prompt_args_empty_required_still_included() {
        let v = build_prompt_args(&[(arg("name", true), "".into())]);
        assert_eq!(v["name"], "");
    }

    #[test]
    fn test_build_prompt_args_multiple() {
        let v = build_prompt_args(&[
            (arg("name", true), "stand-in".into()),
            (opt_arg("style"), "friendly".into()),
        ]);
        assert_eq!(v["name"], "stand-in");
        assert_eq!(v["style"], "friendly");
    }

    #[test]
    fn test_missing_required_none() {
        let params = [(arg("name", true), "stand-in".into())];
        assert!(missing_required(&params).is_empty());
    }

    #[test]
    fn test_missing_required_some() {
        let params = [
            (arg("name", true), "stand-in".into()),
            (arg("one", true), "".into()),
        ];
        assert_eq!(missing_required(&params), vec!["one"]);
    }

    #[test]
    fn test_missing_required_optional_ignored() {
        let params = [(opt_arg("style"), "".into())];
        assert!(missing_required(&params).is_empty());
    }

    #[test]
    fn test_role_label_user() {
        assert_eq!(role_label(&PromptRole::User), "user");
    }

    #[test]
    fn test_role_label_assistant() {
        assert_eq!(role_label(&PromptRole::Assistant), "assistant");
    }
}
