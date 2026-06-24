//! Integration tests for the async bridge (M3).
//!
//! Tests the full path: spawn engine → send Connect → drain events →
//! assert Connected snapshot matches the `stand-in-reference` server.

use stand_in_client::prelude::Credential;
use stand_in_mcp_explorer::app::conn_state::{ConnState, reduce};
use stand_in_mcp_explorer::app::engine_loop::spawn_engine;
use stand_in_mcp_explorer::app::events::{ConnConfig, Transport, UiCommand};
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .expect("parent of app crate dir")
        .parent()
        .expect("repo root")
        .to_path_buf()
}

fn reference_binary() -> PathBuf {
    let mut bin = repo_root()
        .join("target")
        .join("debug")
        .join("stand-in-reference");
    #[cfg(windows)]
    bin.set_extension("exe");
    if !bin.exists() {
        let status = std::process::Command::new("cargo")
            .args([
                "build",
                "--manifest-path",
                &repo_root().join("Cargo.toml").to_string_lossy(),
                "-p",
                "stand-in-reference",
            ])
            .status()
            .expect("failed to build stand-in-reference");
        assert!(status.success(), "cargo build -p stand-in-reference failed");
    }
    bin
}

#[test]
fn test_bridge_connect_lists_reference() {
    let bin = reference_binary();
    let (cmd_tx, mut evt_rx) = spawn_engine();

    cmd_tx
        .send(UiCommand::Connect {
            config: ConnConfig {
                transport: Transport::Stdio,
                command: bin.to_string_lossy().into_owned(),
                args: vec![],
                url: String::new(),
                env: Vec::new(),
            },
            credential: Box::new(Credential::default()),
        })
        .expect("send Connect");

    let mut state = ConnState::Disconnected;
    let mut connected = false;
    let mut tools_count = 0;
    let mut resources_count = 0;
    let mut templates_count = 0;
    let mut prompts_count = 0;

    while let Some(event) = evt_rx.blocking_recv() {
        state = reduce(state, event);
        match &state {
            ConnState::Connected(snap) => {
                tools_count = snap.tools.len();
                resources_count = snap.resources.len();
                templates_count = snap.templates.len();
                prompts_count = snap.prompts.len();
                connected = true;
                break;
            }
            ConnState::Error(e) => {
                panic!("unexpected connection error: {e}");
            }
            _ => {}
        }
    }

    assert!(connected, "should reach Connected state");
    assert_eq!(tools_count, 3, "stand-in-reference serves 3 tools");
    assert_eq!(
        resources_count, 2,
        "stand-in-reference serves 2 concrete resources"
    );
    assert_eq!(
        templates_count, 1,
        "stand-in-reference serves 1 resource template"
    );
    assert_eq!(prompts_count, 1, "stand-in-reference serves 1 prompt");

    cmd_tx.send(UiCommand::Disconnect).expect("send Disconnect");

    while let Some(event) = evt_rx.blocking_recv() {
        state = reduce(state, event);
        if matches!(state, ConnState::Disconnected) {
            break;
        }
    }
    assert!(matches!(state, ConnState::Disconnected));
}

#[test]
fn test_bridge_connect_failure_is_transport_error() {
    let (cmd_tx, mut evt_rx) = spawn_engine();

    cmd_tx
        .send(UiCommand::Connect {
            config: ConnConfig {
                transport: Transport::Stdio,
                command: "definitely-not-a-real-binary-xyz".into(),
                args: vec![],
                url: String::new(),
                env: Vec::new(),
            },
            credential: Box::new(Credential::default()),
        })
        .expect("send Connect");

    let mut state = ConnState::Disconnected;
    let mut got_error = false;

    while let Some(event) = evt_rx.blocking_recv() {
        state = reduce(state, event);
        match &state {
            ConnState::Error(e) => {
                eprintln!("error: {e}");
                got_error = true;
                break;
            }
            ConnState::Connected(_) => {
                panic!("should not connect to a nonexistent binary");
            }
            _ => {}
        }
    }

    assert!(got_error, "should reach Error state for nonexistent binary");
}

// --- M10: call_tool run-loop integration tests ---

fn connect_to_reference() -> (
    tokio::sync::mpsc::UnboundedSender<UiCommand>,
    tokio::sync::mpsc::UnboundedReceiver<stand_in_mcp_explorer::app::events::EngineEvent>,
    ConnState,
) {
    let bin = reference_binary();
    let (cmd_tx, mut evt_rx) = spawn_engine();

    cmd_tx
        .send(UiCommand::Connect {
            config: ConnConfig {
                transport: Transport::Stdio,
                command: bin.to_string_lossy().into_owned(),
                args: vec!["--stdio".into()],
                url: String::new(),
                env: Vec::new(),
            },
            credential: Box::new(Credential::default()),
        })
        .expect("send Connect");

    let mut state = ConnState::Disconnected;

    while let Some(event) = evt_rx.blocking_recv() {
        state = reduce(state, event);
        match &state {
            ConnState::Connected(_) => break,
            ConnState::Error(e) => panic!("unexpected connection error: {e}"),
            _ => {}
        }
    }

    assert!(
        matches!(state, ConnState::Connected(_)),
        "should reach Connected state"
    );

    (cmd_tx, evt_rx, state)
}

#[test]
fn test_call_tool_greet() {
    let (cmd_tx, mut evt_rx, _state) = connect_to_reference();

    cmd_tx
        .send(UiCommand::CallTool {
            name: "greet".into(),
            arguments: serde_json::json!({"name": "stand-in"}),
        })
        .expect("send CallTool");

    let mut got_result = false;
    while let Some(event) = evt_rx.blocking_recv() {
        match event {
            stand_in_mcp_explorer::app::events::EngineEvent::ToolResult(r) => {
                let text: String = r
                    .content
                    .iter()
                    .map(|c| match c {
                        stand_in_client::prelude::Content::Text { text } => text.as_str(),
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                assert!(
                    text.contains("Hello, stand-in!"),
                    "greet should say hello, got: {text}"
                );
                assert!(r.is_error.is_none(), "greet should not be an error");
                got_result = true;
                break;
            }
            stand_in_mcp_explorer::app::events::EngineEvent::ToolError(e) => {
                panic!("unexpected tool error from greet: {e}");
            }
            _ => {}
        }
    }

    assert!(got_result, "should receive ToolResult from greet");

    cmd_tx.send(UiCommand::Disconnect).expect("send Disconnect");
}

#[test]
fn test_call_tool_add() {
    let (cmd_tx, mut evt_rx, _state) = connect_to_reference();

    cmd_tx
        .send(UiCommand::CallTool {
            name: "add".into(),
            arguments: serde_json::json!({"a": 10, "b": 32}),
        })
        .expect("send CallTool");

    let mut got_result = false;
    while let Some(event) = evt_rx.blocking_recv() {
        match event {
            stand_in_mcp_explorer::app::events::EngineEvent::ToolResult(r) => {
                let text: String = r
                    .content
                    .iter()
                    .map(|c| match c {
                        stand_in_client::prelude::Content::Text { text } => text.as_str(),
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                assert!(text.contains("42"), "add 10 + 32 should be 42, got: {text}");
                assert!(r.is_error.is_none(), "add should not be an error");
                got_result = true;
                break;
            }
            stand_in_mcp_explorer::app::events::EngineEvent::ToolError(e) => {
                panic!("unexpected tool error from add: {e}");
            }
            _ => {}
        }
    }

    assert!(got_result, "should receive ToolResult from add");

    cmd_tx.send(UiCommand::Disconnect).expect("send Disconnect");
}

#[test]
fn test_call_tool_unknown_tool_gives_tool_error() {
    let (cmd_tx, mut evt_rx, _state) = connect_to_reference();

    cmd_tx
        .send(UiCommand::CallTool {
            name: "nope".into(),
            arguments: serde_json::json!({}),
        })
        .expect("send CallTool");

    let mut got_tool_error = false;
    while let Some(event) = evt_rx.blocking_recv() {
        match event {
            stand_in_mcp_explorer::app::events::EngineEvent::ToolResult(r)
                if r.is_error == Some(true) =>
            {
                got_tool_error = true;
                break;
            }
            stand_in_mcp_explorer::app::events::EngineEvent::ToolError(e) => {
                eprintln!("tool error (expected): {e}");
                got_tool_error = true;
                break;
            }
            _ => {}
        }
    }

    assert!(
        got_tool_error,
        "should receive ToolError or is_error from unknown tool"
    );

    cmd_tx.send(UiCommand::Disconnect).expect("send Disconnect");
}

#[test]
fn test_call_tool_not_connected_gives_error() {
    let (cmd_tx, mut evt_rx, _state) = connect_to_reference();

    // Disconnect first
    cmd_tx.send(UiCommand::Disconnect).expect("send Disconnect");
    let mut disconnected = false;
    while let Some(event) = evt_rx.blocking_recv() {
        if matches!(
            event,
            stand_in_mcp_explorer::app::events::EngineEvent::Disconnected
        ) {
            disconnected = true;
            break;
        }
    }
    assert!(disconnected, "should disconnect");

    // Now try call_tool while disconnected
    cmd_tx
        .send(UiCommand::CallTool {
            name: "greet".into(),
            arguments: serde_json::json!({"name": "test"}),
        })
        .expect("send CallTool");

    let mut got_error = false;
    while let Some(event) = evt_rx.blocking_recv() {
        if let stand_in_mcp_explorer::app::events::EngineEvent::ToolError(e) = event {
            eprintln!("tool error when disconnected (expected): {e}");
            got_error = true;
            break;
        }
    }

    assert!(
        got_error,
        "should receive ToolError when calling tool while disconnected"
    );
}

// --- M11: read_resource + subscribe/unsubscribe integration tests ---

#[test]
fn test_read_resource_version() {
    let (cmd_tx, mut evt_rx, _state) = connect_to_reference();

    cmd_tx
        .send(UiCommand::ReadResource {
            uri: "info://version".into(),
        })
        .expect("send ReadResource");

    let mut got_result = false;
    while let Some(event) = evt_rx.blocking_recv() {
        match event {
            stand_in_mcp_explorer::app::events::EngineEvent::ResourceResult(r) => {
                let text: String = r
                    .contents
                    .iter()
                    .filter_map(|c| match c {
                        stand_in_client::prelude::ResourceContents::Text { text, .. } => {
                            Some(text.as_str())
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                assert!(
                    text.contains("version"),
                    "info://version should contain 'version', got: {text}"
                );
                got_result = true;
                break;
            }
            stand_in_mcp_explorer::app::events::EngineEvent::ResourceError(e) => {
                panic!("unexpected resource error from info://version: {e}");
            }
            _ => {}
        }
    }

    assert!(
        got_result,
        "should receive ResourceResult from reading info://version"
    );

    cmd_tx.send(UiCommand::Disconnect).expect("send Disconnect");
}

#[test]
fn test_subscribe_unsubscribe_ok() {
    let (cmd_tx, mut evt_rx, _state) = connect_to_reference();

    cmd_tx
        .send(UiCommand::Subscribe {
            uri: "info://version".into(),
        })
        .expect("send Subscribe");

    let mut got_subscribed = false;
    while let Some(event) = evt_rx.blocking_recv() {
        match event {
            stand_in_mcp_explorer::app::events::EngineEvent::Subscribed(uri) => {
                assert_eq!(uri, "info://version");
                got_subscribed = true;
                break;
            }
            stand_in_mcp_explorer::app::events::EngineEvent::ResourceError(e) => {
                panic!("unexpected resource error from subscribe: {e}");
            }
            _ => {}
        }
    }
    assert!(got_subscribed, "should receive Subscribed event");

    cmd_tx
        .send(UiCommand::Unsubscribe {
            uri: "info://version".into(),
        })
        .expect("send Unsubscribe");

    let mut got_unsubscribed = false;
    while let Some(event) = evt_rx.blocking_recv() {
        match event {
            stand_in_mcp_explorer::app::events::EngineEvent::Unsubscribed(uri) => {
                assert_eq!(uri, "info://version");
                got_unsubscribed = true;
                break;
            }
            stand_in_mcp_explorer::app::events::EngineEvent::ResourceError(e) => {
                panic!("unexpected resource error from unsubscribe: {e}");
            }
            _ => {}
        }
    }
    assert!(got_unsubscribed, "should receive Unsubscribed event");

    cmd_tx.send(UiCommand::Disconnect).expect("send Disconnect");
}

#[test]
fn test_read_resource_not_connected_gives_error() {
    let (cmd_tx, mut evt_rx, _state) = connect_to_reference();

    // Disconnect first
    cmd_tx.send(UiCommand::Disconnect).expect("send Disconnect");
    let mut disconnected = false;
    while let Some(event) = evt_rx.blocking_recv() {
        if matches!(
            event,
            stand_in_mcp_explorer::app::events::EngineEvent::Disconnected
        ) {
            disconnected = true;
            break;
        }
    }
    assert!(disconnected, "should disconnect");

    cmd_tx
        .send(UiCommand::ReadResource {
            uri: "info://version".into(),
        })
        .expect("send ReadResource");

    let mut got_error = false;
    while let Some(event) = evt_rx.blocking_recv() {
        if let stand_in_mcp_explorer::app::events::EngineEvent::ResourceError(e) = event {
            eprintln!("resource error when disconnected (expected): {e}");
            got_error = true;
            break;
        }
    }

    assert!(
        got_error,
        "should receive ResourceError when reading resource while disconnected"
    );
}

// --- M12: get_prompt integration tests ---

#[test]
fn test_get_prompt_greeting() {
    let (cmd_tx, mut evt_rx, _state) = connect_to_reference();

    cmd_tx
        .send(UiCommand::GetPrompt {
            name: "greeting".into(),
            arguments: serde_json::json!({"name": "stand-in"}),
        })
        .expect("send GetPrompt");

    let mut got_result = false;
    while let Some(event) = evt_rx.blocking_recv() {
        match event {
            stand_in_mcp_explorer::app::events::EngineEvent::PromptMessages(r) => {
                assert!(
                    !r.messages.is_empty(),
                    "greeting prompt should return at least one message"
                );
                // The message should contain "greeting" or "stand-in"
                let text: String = r
                    .messages
                    .iter()
                    .map(|m| match &m.content {
                        stand_in_client::prelude::PromptContent::Text { text } => text.as_str(),
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                assert!(
                    text.contains("greeting") || text.contains("stand-in"),
                    "message should contain 'greeting' or 'stand-in', got: {text}"
                );
                got_result = true;
                break;
            }
            stand_in_mcp_explorer::app::events::EngineEvent::PromptError(e) => {
                panic!("unexpected prompt error from greeting: {e}");
            }
            _ => {}
        }
    }

    assert!(got_result, "should receive PromptMessages from greeting");

    cmd_tx.send(UiCommand::Disconnect).expect("send Disconnect");
}

#[test]
fn test_get_prompt_unknown_gives_error() {
    let (cmd_tx, mut evt_rx, _state) = connect_to_reference();

    cmd_tx
        .send(UiCommand::GetPrompt {
            name: "nonexistent-xyz".into(),
            arguments: serde_json::json!({}),
        })
        .expect("send GetPrompt");

    let mut got_error = false;
    while let Some(event) = evt_rx.blocking_recv() {
        match event {
            stand_in_mcp_explorer::app::events::EngineEvent::PromptError(e) => {
                eprintln!("prompt error (expected): {e}");
                got_error = true;
                break;
            }
            stand_in_mcp_explorer::app::events::EngineEvent::PromptMessages(_) => {
                panic!("should not receive messages for unknown prompt");
            }
            _ => {}
        }
    }

    assert!(got_error, "should receive PromptError for unknown prompt");

    cmd_tx.send(UiCommand::Disconnect).expect("send Disconnect");
}

#[test]
fn test_get_prompt_not_connected_gives_error() {
    let (cmd_tx, mut evt_rx, _state) = connect_to_reference();

    // Disconnect first
    cmd_tx.send(UiCommand::Disconnect).expect("send Disconnect");
    let mut disconnected = false;
    while let Some(event) = evt_rx.blocking_recv() {
        if matches!(
            event,
            stand_in_mcp_explorer::app::events::EngineEvent::Disconnected
        ) {
            disconnected = true;
            break;
        }
    }
    assert!(disconnected, "should disconnect");

    cmd_tx
        .send(UiCommand::GetPrompt {
            name: "greeting".into(),
            arguments: serde_json::json!({"name": "test"}),
        })
        .expect("send GetPrompt");

    let mut got_error = false;
    while let Some(event) = evt_rx.blocking_recv() {
        if let stand_in_mcp_explorer::app::events::EngineEvent::PromptError(e) = event {
            eprintln!("prompt error when disconnected (expected): {e}");
            got_error = true;
            break;
        }
    }

    assert!(
        got_error,
        "should receive PromptError when calling get_prompt while disconnected"
    );
}
