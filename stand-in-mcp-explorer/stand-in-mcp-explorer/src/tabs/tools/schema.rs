//! Schema parsing — pure, testable conversion from `InputSchema` (JSON Schema)
//! to a flat list of `ParamField`. Zero gpui context, zero panic on malformed input.

use stand_in_client::prelude::{CallToolResult, InputSchema};

/// A single tool parameter parsed from the schema.
#[derive(Debug, Clone, PartialEq)]
pub struct ParamField {
    pub name: String,
    pub type_str: String,
    pub description: String,
    pub required: bool,
}

/// Tool execution lifecycle state.
#[derive(Debug, Clone, Default)]
pub enum ToolRun {
    #[default]
    Idle,
    Running,
    Result(Box<CallToolResult>),
    Error(String),
}

/// Pure reducer for `ToolRun` — zero gpui context.
pub fn reduce_tool_run(state: ToolRun, event: &crate::app::events::EngineEvent) -> ToolRun {
    match event {
        crate::app::events::EngineEvent::ToolResult(r) => ToolRun::Result(r.clone()),
        crate::app::events::EngineEvent::ToolError(e) => ToolRun::Error(e.clone()),
        crate::app::events::EngineEvent::Connected(_)
        | crate::app::events::EngineEvent::Disconnected => ToolRun::Idle,
        _ => state,
    }
}

/// Parse an `InputSchema` into an ordered list of `ParamField`s.
///
/// Reads `properties` as a JSON object, extracts `type` and `description` per
/// property, and marks fields listed in `required`. Returns an empty vec on
/// missing or malformed schemas — never panics.
///
/// Order: follows the iteration order of the JSON object (serde_json preserves
/// insertion order by default). Falls back to alphabetical if the property
/// value is not an object.
pub fn parse_params(schema: &InputSchema) -> Vec<ParamField> {
    let Some(ref properties) = schema.properties else {
        return vec![];
    };

    let obj = match properties.as_object() {
        Some(o) => o,
        None => return vec![],
    };

    let required: Vec<&str> = schema
        .required
        .as_ref()
        .map(|r| r.iter().map(String::as_str).collect())
        .unwrap_or_default();

    obj.iter()
        .map(|(name, prop)| {
            let type_str = prop
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("string")
                .to_string();
            let description = prop
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let required = required.contains(&name.as_str());
            ParamField {
                name: name.clone(),
                type_str,
                description,
                required,
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Pure argument builders — testable without gpui
// ---------------------------------------------------------------------------

/// Build a `serde_json::Value` object from parameter fields and their current
/// text values. Converts by type: `string` → Value::String; `number`/`integer`
/// → parsed number (invalid → blocked, returns None); `boolean` → `true`/`false`
/// (anything else is omitted). Empty non-required fields are omitted.
pub fn build_arguments(params: &[(ParamField, String)]) -> Option<serde_json::Value> {
    let mut map = serde_json::Map::new();

    for (field, text) in params {
        let trimmed = text.trim();
        if trimmed.is_empty() && !field.required {
            continue;
        }

        let value = match field.type_str.as_str() {
            "number" | "integer" => match trimmed.parse::<serde_json::Number>() {
                Ok(n) => serde_json::Value::Number(n),
                Err(_) => return None,
            },
            "boolean" => match trimmed.to_lowercase().as_str() {
                "true" | "1" | "yes" => serde_json::Value::Bool(true),
                "false" | "0" | "no" => serde_json::Value::Bool(false),
                _ => continue,
            },
            _ => serde_json::Value::String(trimmed.to_string()),
        };

        map.insert(field.name.clone(), value);
    }

    Some(serde_json::Value::Object(map))
}

/// Returns names of required parameters whose text value is empty.
pub fn missing_required(params: &[(ParamField, String)]) -> Vec<&str> {
    params
        .iter()
        .filter(|(f, t)| f.required && t.trim().is_empty())
        .map(|(f, _)| f.name.as_str())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn schema_with(properties: serde_json::Value, required: Option<Vec<String>>) -> InputSchema {
        let mut s = InputSchema::object().with_properties(properties);
        if let Some(r) = required {
            s = s.with_required(r);
        }
        s
    }

    #[test]
    fn test_empty_schema() {
        let s = InputSchema::object();
        assert_eq!(parse_params(&s), vec![]);
    }

    #[test]
    fn test_properties_none() {
        let s = InputSchema::object();
        assert_eq!(parse_params(&s), vec![]);
    }

    #[test]
    fn test_two_props_one_required() {
        let s = schema_with(
            serde_json::json!({
                "path": {"type": "string", "description": "File path"},
                "count": {"type": "integer", "description": "How many"}
            }),
            Some(vec!["path".into()]),
        );

        let fields = parse_params(&s);
        assert_eq!(fields.len(), 2);

        assert_eq!(fields[0].name, "path");
        assert_eq!(fields[0].type_str, "string");
        assert_eq!(fields[0].description, "File path");
        assert!(fields[0].required);

        assert_eq!(fields[1].name, "count");
        assert_eq!(fields[1].type_str, "integer");
        assert_eq!(fields[1].description, "How many");
        assert!(!fields[1].required);
    }

    #[test]
    fn test_type_string() {
        let s = schema_with(serde_json::json!({"name": {"type": "string"}}), None);
        let fields = parse_params(&s);
        assert_eq!(fields[0].type_str, "string");
    }

    #[test]
    fn test_type_number() {
        let s = schema_with(serde_json::json!({"age": {"type": "number"}}), None);
        assert_eq!(parse_params(&s)[0].type_str, "number");
    }

    #[test]
    fn test_type_boolean() {
        let s = schema_with(serde_json::json!({"flag": {"type": "boolean"}}), None);
        assert_eq!(parse_params(&s)[0].type_str, "boolean");
    }

    #[test]
    fn test_missing_type_defaults_to_string() {
        let s = schema_with(
            serde_json::json!({"data": {"description": "Some data"}}),
            None,
        );
        assert_eq!(parse_params(&s)[0].type_str, "string");
    }

    #[test]
    fn test_missing_description_defaults_to_empty() {
        let s = schema_with(serde_json::json!({"x": {"type": "integer"}}), None);
        assert_eq!(parse_params(&s)[0].description, "");
    }

    #[test]
    fn test_empty_required() {
        let s = schema_with(serde_json::json!({"a": {"type": "string"}}), Some(vec![]));
        assert!(!parse_params(&s)[0].required);
    }

    #[test]
    fn test_properties_null_returns_empty() {
        let s = InputSchema::object().with_properties(serde_json::Value::Null);
        assert_eq!(parse_params(&s), vec![]);
    }

    #[test]
    fn test_properties_string_value_not_object() {
        // If properties is a string (non-object), as_object() returns None
        let s = InputSchema::object().with_properties(serde_json::Value::String("bad".into()));
        assert_eq!(parse_params(&s), vec![]);
    }

    // -- Tests for build_arguments / missing_required / reduce_tool_run --

    #[test]
    fn test_build_arguments_string() {
        let args = build_arguments(&[(param("name", "string", false), "stand-in".into())]);
        let v = args.unwrap();
        assert_eq!(v["name"], "stand-in");
    }

    #[test]
    fn test_build_arguments_number_valid() {
        let args = build_arguments(&[(param("a", "number", false), "42".into())]);
        let v = args.unwrap();
        assert_eq!(
            v["a"],
            serde_json::Value::Number(serde_json::Number::from(42))
        );
    }

    #[test]
    fn test_build_arguments_number_invalid_returns_none() {
        let args = build_arguments(&[(param("a", "number", false), "x".into())]);
        assert!(args.is_none());
    }

    #[test]
    fn test_build_arguments_integer_valid() {
        let args = build_arguments(&[(param("count", "integer", false), "7".into())]);
        let v = args.unwrap();
        assert_eq!(
            v["count"],
            serde_json::Value::Number(serde_json::Number::from(7))
        );
    }

    #[test]
    fn test_build_arguments_boolean_true() {
        let args = build_arguments(&[(param("flag", "boolean", false), "true".into())]);
        let v = args.unwrap();
        assert_eq!(v["flag"], serde_json::Value::Bool(true));
    }

    #[test]
    fn test_build_arguments_boolean_false() {
        let args = build_arguments(&[(param("flag", "boolean", false), "false".into())]);
        let v = args.unwrap();
        assert_eq!(v["flag"], serde_json::Value::Bool(false));
    }

    #[test]
    fn test_build_arguments_boolean_unrecognized_omitted() {
        let args = build_arguments(&[(param("flag", "boolean", false), "maybe".into())]);
        let v = args.unwrap();
        assert!(v.as_object().unwrap().is_empty());
    }

    #[test]
    fn test_build_arguments_empty_non_required_omitted() {
        let args = build_arguments(&[(param("opt", "string", false), "".into())]);
        let v = args.unwrap();
        assert!(v.as_object().unwrap().is_empty());
    }

    #[test]
    fn test_build_arguments_multiple_fields() {
        let args = build_arguments(&[
            (param("name", "string", true), "stand-in".into()),
            (param("count", "integer", true), "3".into()),
            (param("opt", "string", false), "".into()),
        ]);
        let v = args.unwrap();
        assert_eq!(v["name"], "stand-in");
        assert_eq!(
            v["count"],
            serde_json::Value::Number(serde_json::Number::from(3))
        );
        assert!(!v.as_object().unwrap().contains_key("opt"));
    }

    #[test]
    fn test_missing_required_none() {
        let params = [(param("a", "string", true), "hello".into())];
        let missing = missing_required(&params);
        assert!(missing.is_empty());
    }

    #[test]
    fn test_missing_required_some() {
        let params = [
            (param("a", "string", true), "hello".into()),
            (param("b", "string", true), "".into()),
        ];
        let missing = missing_required(&params);
        assert_eq!(missing, vec!["b"]);
    }

    #[test]
    fn test_missing_required_whitespace_only() {
        let params = [(param("a", "string", true), "   ".into())];
        let missing = missing_required(&params);
        assert_eq!(missing, vec!["a"]);
    }

    #[test]
    fn test_missing_required_optional_ignored() {
        let params = [(param("opt", "string", false), "".into())];
        let missing = missing_required(&params);
        assert!(missing.is_empty());
    }

    #[test]
    fn test_reduce_tool_run_result() {
        use crate::app::events::EngineEvent;
        use stand_in_client::prelude::CallToolResult;
        let result = CallToolResult::text("hello");
        let event = EngineEvent::ToolResult(Box::new(result));
        let state = reduce_tool_run(ToolRun::Running, &event);
        match state {
            ToolRun::Result(r) => {
                assert!(r.is_error.is_none());
            }
            _ => panic!("expected Result"),
        }
    }

    #[test]
    fn test_reduce_tool_run_error() {
        use crate::app::events::EngineEvent;
        let event = EngineEvent::ToolError("protocol error".into());
        let state = reduce_tool_run(ToolRun::Running, &event);
        match state {
            ToolRun::Error(e) => assert!(e.contains("protocol")),
            _ => panic!("expected Error"),
        }
    }

    #[test]
    fn test_reduce_tool_run_reset_on_connected() {
        use crate::app::events::EngineEvent;
        let event = EngineEvent::Disconnected;
        let state = reduce_tool_run(
            ToolRun::Result(Box::new(stand_in_client::prelude::CallToolResult::text(
                "ok",
            ))),
            &event,
        );
        assert!(matches!(state, ToolRun::Idle));
    }

    fn param(name: &str, type_str: &str, required: bool) -> ParamField {
        ParamField {
            name: name.into(),
            type_str: type_str.into(),
            description: String::new(),
            required,
        }
    }
}
