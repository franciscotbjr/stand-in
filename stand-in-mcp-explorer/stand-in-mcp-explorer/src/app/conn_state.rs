//! Connection state machine and its pure reducer.
//!
//! `ConnState` drives the app shell rendering (M3) and every region (M5+).
//! The `reduce` function is a **pure** state transition — no I/O, no gpui
//! context — and is unit-testable without a window or subprocess.

use crate::app::events::{EngineEvent, ServerSnapshot};

#[derive(Debug, Clone)]
pub enum ConnState {
    Disconnected,
    Connecting,
    Connected(Box<ServerSnapshot>),
    Error(String),
}

pub fn reduce(_state: ConnState, event: EngineEvent) -> ConnState {
    match event {
        EngineEvent::Connecting => ConnState::Connecting,
        EngineEvent::Connected(s) => ConnState::Connected(s),
        EngineEvent::ConnectionError(e) => ConnState::Error(e),
        EngineEvent::Disconnected => ConnState::Disconnected,
        // Tool and resource events do not change connection state
        EngineEvent::ToolResult(_)
        | EngineEvent::ToolError(_)
        | EngineEvent::ResourceResult(_)
        | EngineEvent::ResourceError(_)
        | EngineEvent::Subscribed(_)
        | EngineEvent::Unsubscribed(_)
        | EngineEvent::PromptMessages(_)
        | EngineEvent::PromptError(_)
        | EngineEvent::Notification(_)
        | EngineEvent::Authorized(_)
        | EngineEvent::AuthorizationError(_) => _state,
    }
}

pub fn capture_fixture_state(state: &str) -> ConnState {
    match state {
        "connected" | "longtext" | "tools" | "list" | "selected" | "search" | "empty"
        | "guided" | "resources" | "text" | "json" | "binary" | "subscribed" | "loading"
        | "prompts" | "messages-1" | "messages-2" | "result-friendly" | "result-raw"
        | "result-error" | "running" => ConnState::Connected(Box::new(fixture_snapshot())),
        #[cfg(feature = "perf")]
        "perf" => ConnState::Connected(Box::new(perf_snapshot())),
        "connecting" => ConnState::Connecting,
        "error" => ConnState::Error("fixture error".into()),
        _ => ConnState::Disconnected,
    }
}

/// Synthetic snapshot for perf measurement (037 / O-024): `MCPX_PERF_N` tools
/// with varied names/descriptions. Resources/templates/prompts are empty — the
/// perf focus is the Tools list (the `uniform_list`/fixed-height family).
#[cfg(feature = "perf")]
fn perf_snapshot() -> ServerSnapshot {
    use stand_in_client::prelude::{InputSchema, ToolDefinition};
    let n = crate::app::perf::perf_n();
    let tools = (0..n)
        .map(|i| ToolDefinition {
            name: format!("tool_{i:06}"),
            description: format!(
                "Synthetic tool #{i} for performance measurement — {}",
                "lorem ipsum dolor ".repeat(1 + (i % 4))
            ),
            input_schema: InputSchema::object().with_properties(serde_json::json!({
                "arg": {"type": "string", "description": "synthetic argument"}
            })),
        })
        .collect();
    ServerSnapshot {
        server_info: stand_in_client::prelude::ServerInfo {
            name: "perf-fixture".into(),
            version: "0.0.0".into(),
        },
        capabilities: stand_in_client::prelude::ServerCapabilities::new(),
        tools,
        resources: vec![],
        templates: vec![],
        prompts: vec![],
        latency_ms: 0,
    }
}

fn fixture_snapshot() -> ServerSnapshot {
    use stand_in_client::prelude::{InputSchema, ToolDefinition};
    ServerSnapshot {
        server_info: stand_in_client::prelude::ServerInfo {
            name: "stand-in-reference".into(),
            version: "0.0.4".into(),
        },
        capabilities: stand_in_client::prelude::ServerCapabilities::new(),
        tools: vec![
            ToolDefinition {
                name: "greet".into(),
                description: "Says hello to a person by name.".into(),
                input_schema: InputSchema::object()
                    .with_properties(serde_json::json!({
                        "name": {"type": "string", "description": "The person's name"},
                        "language": {"type": "string", "description": "Language code (pt, en, es)"}
                    }))
                    .with_required(vec!["name".into()]),
            },
            ToolDefinition {
                name: "weather".into(),
                description: "Gets weather for a city.".into(),
                input_schema: InputSchema::object().with_properties(serde_json::json!({
                    "city": {"type": "string", "description": "City name"},
                    "units": {"type": "string", "description": "Units (celsius or fahrenheit)"}
                })),
            },
            ToolDefinition {
                name: "search".into(),
                description: "Searches documentation by keyword.".into(),
                input_schema: InputSchema::object().with_properties(serde_json::json!({
                    "query": {"type": "string", "description": "Search term"},
                    "limit": {"type": "integer", "description": "Max results (default 10)"}
                })),
            },
            ToolDefinition {
                name: "echo".into(),
                description: "Returns the input unchanged, with no parameters.".into(),
                input_schema: InputSchema::object(),
            },
            ToolDefinition {
                name: "add".into(),
                description: "Adds two numbers together.".into(),
                input_schema: InputSchema::object()
                    .with_properties(serde_json::json!({
                        "a": {"type": "number", "description": "First number"},
                        "b": {"type": "number", "description": "Second number"}
                    }))
                    .with_required(vec!["a".into(), "b".into()]),
            },
        ],
        resources: vec![
            stand_in_client::prelude::Resource {
                uri: "file:///readme".into(),
                name: "readme".into(),
                description: Some("README file".into()),
                mime_type: Some("text/markdown".into()),
                annotations: None,
                size: None,
            },
            stand_in_client::prelude::Resource {
                uri: "file:///config".into(),
                name: "config".into(),
                description: Some("Config file".into()),
                mime_type: Some("application/json".into()),
                annotations: None,
                size: None,
            },
            stand_in_client::prelude::Resource {
                uri: "file:///data".into(),
                name: "data".into(),
                description: Some("Data file".into()),
                mime_type: Some("application/octet-stream".into()),
                annotations: None,
                size: None,
            },
        ],
        templates: vec![stand_in_client::prelude::ResourceTemplate {
            uri_template: "file:///{path}".into(),
            name: "files".into(),
            description: Some("Any file".into()),
            mime_type: Some("text/plain".into()),
        }],
        prompts: vec![
            stand_in_client::prelude::PromptDefinition {
                name: "greeting".into(),
                description: "Generate a greeting for someone".into(),
                arguments: Some(vec![stand_in::prompt::PromptArgument {
                    name: "name".into(),
                    description: Some("The person's name".into()),
                    required: Some(true),
                }]),
            },
            stand_in_client::prelude::PromptDefinition {
                name: "summarize".into(),
                description: "Summarizes content".into(),
                arguments: None,
            },
            stand_in_client::prelude::PromptDefinition {
                name: "review".into(),
                description: "Code review".into(),
                arguments: None,
            },
        ],
        latency_ms: 57,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot() -> ServerSnapshot {
        ServerSnapshot {
            server_info: stand_in_client::prelude::ServerInfo {
                name: "test".into(),
                version: "1.0".into(),
            },
            capabilities: stand_in_client::prelude::ServerCapabilities::new(),
            tools: vec![],
            resources: vec![],
            templates: vec![],
            prompts: vec![],
            latency_ms: 0,
        }
    }

    #[test]
    fn test_reduce_connecting() {
        let s = reduce(ConnState::Disconnected, EngineEvent::Connecting);
        assert!(matches!(s, ConnState::Connecting));
    }

    #[test]
    fn test_reduce_connected() {
        let snap = snapshot();
        let s = reduce(
            ConnState::Connecting,
            EngineEvent::Connected(Box::new(snap)),
        );
        assert!(matches!(s, ConnState::Connected(_)));
    }

    #[test]
    fn test_reduce_connection_error() {
        let s = reduce(
            ConnState::Connecting,
            EngineEvent::ConnectionError("fail".into()),
        );
        match s {
            ConnState::Error(msg) => assert!(msg.contains("fail")),
            _ => panic!("expected Error"),
        }
    }

    #[test]
    fn test_reduce_disconnected_from_connected() {
        let snap = snapshot();
        let s = ConnState::Connected(Box::new(snap));
        let s = reduce(s, EngineEvent::Disconnected);
        assert!(matches!(s, ConnState::Disconnected));
    }

    #[test]
    fn test_reduce_error_can_retry() {
        let s = ConnState::Error("fail".into());
        let s = reduce(s, EngineEvent::Connecting);
        assert!(matches!(s, ConnState::Connecting));
    }
}
