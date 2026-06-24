//! History buffer — records tool calls and prompt generations as accordion
//! entries for the History tab (M14).
//!
//! A `pending_call` is set before dispatching a `CallTool` or `GetPrompt`
//! command. When the response arrives (at drain time), a `HistoryEntry` is
//! created via the pure `make_history_entry` and prepended to the buffer.
//!
//! The buffer grows in chronological dispatch order; the render reverses it
//! (newest-first). A cap of 200 entries prevents unbounded growth.
//!
//! ## Two-plane error handling
//!
//! Errors are **recorded** with status = error so the user can inspect what
//! went wrong — both execution errors (`isError`) and transport/protocol
//! errors (`ToolError`/`PromptError`) produce history entries.

use serde_json::Value;
use std::time::Instant;

const HISTORY_CAP: usize = 200;

/// What kind of call produced this history entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistKind {
    Tool,
    Prompt,
}

/// A recorded request→response pair, accordion-ready.
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub kind: HistKind,
    pub name: String,
    pub request: Value,
    pub response: Value,
    pub time: String,
    pub timing_ms: Option<u64>,
    pub has_error: bool,
    pub expanded: bool,
}

/// In-flight call — set before dispatching, cleared on response.
pub struct PendingCall {
    pub kind: HistKind,
    pub name: String,
    pub request: Value,
    pub started: Instant,
}

/// Build a `HistoryEntry` from a pending call and a response value.
///
/// Pure — no window, no context. The caller strings the time and serializes
/// the response.
pub fn make_history_entry(
    pending: PendingCall,
    response: Value,
    time: String,
    has_error: bool,
) -> HistoryEntry {
    let timing_ms = Some(pending.started.elapsed().as_millis() as u64);
    HistoryEntry {
        kind: pending.kind,
        name: pending.name,
        request: pending.request,
        response,
        time,
        timing_ms,
        has_error,
        expanded: false,
    }
}

/// Push a history entry, respecting the capacity cap.
pub fn push_history(history: &mut Vec<HistoryEntry>, entry: HistoryEntry) {
    history.insert(0, entry);
    history.truncate(HISTORY_CAP);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_history_entry_tool() {
        let pending = PendingCall {
            kind: HistKind::Tool,
            name: "greet".into(),
            request: serde_json::json!({"name": "World"}),
            started: Instant::now(),
        };
        let response = serde_json::json!({"greeting": "Hello, World!"});
        let entry = make_history_entry(pending, response.clone(), "12:34:56.789".into(), false);
        assert_eq!(entry.kind, HistKind::Tool);
        assert_eq!(entry.name, "greet");
        assert_eq!(entry.request, serde_json::json!({"name": "World"}));
        assert_eq!(entry.response, response);
        assert_eq!(entry.time, "12:34:56.789");
        assert!(entry.timing_ms.is_some());
        assert!(!entry.has_error);
        assert!(!entry.expanded);
    }

    #[test]
    fn test_make_history_entry_prompt() {
        let pending = PendingCall {
            kind: HistKind::Prompt,
            name: "greeting".into(),
            request: serde_json::json!({"style": "formal"}),
            started: Instant::now(),
        };
        let response = serde_json::json!({"messages": []});
        let entry = make_history_entry(pending, response.clone(), "14:02:31.457".into(), false);
        assert_eq!(entry.kind, HistKind::Prompt);
        assert_eq!(entry.name, "greeting");
    }

    #[test]
    fn test_make_history_entry_error() {
        let pending = PendingCall {
            kind: HistKind::Tool,
            name: "bad".into(),
            request: serde_json::json!({}),
            started: Instant::now(),
        };
        let response = serde_json::json!({"error": "timeout"});
        let entry = make_history_entry(pending, response.clone(), "15:00:00.000".into(), true);
        assert!(entry.has_error);
    }

    #[test]
    fn test_push_history_newest_first() {
        let mut history: Vec<HistoryEntry> = Vec::new();
        let e1 = HistoryEntry {
            kind: HistKind::Tool,
            name: "first".into(),
            request: Value::Null,
            response: Value::Null,
            time: "10:00:00.000".into(),
            timing_ms: Some(10),
            has_error: false,
            expanded: false,
        };
        let e2 = HistoryEntry {
            kind: HistKind::Tool,
            name: "second".into(),
            request: Value::Null,
            response: Value::Null,
            time: "10:00:01.000".into(),
            timing_ms: Some(20),
            has_error: false,
            expanded: false,
        };
        push_history(&mut history, e1);
        push_history(&mut history, e2);
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].name, "second"); // newest first
        assert_eq!(history[1].name, "first");
    }

    #[test]
    fn test_push_history_cap() {
        let mut history: Vec<HistoryEntry> = Vec::new();
        for i in 0..HISTORY_CAP + 10 {
            push_history(
                &mut history,
                HistoryEntry {
                    kind: HistKind::Tool,
                    name: format!("call-{}", i),
                    request: Value::Null,
                    response: Value::Null,
                    time: "12:00:00.000".into(),
                    timing_ms: Some(i as u64),
                    has_error: false,
                    expanded: false,
                },
            );
        }
        assert_eq!(history.len(), HISTORY_CAP);
        // Newest first: last pushed = call-(N+9)
        assert_eq!(history[0].name, format!("call-{}", HISTORY_CAP + 9));
        assert_eq!(history[HISTORY_CAP - 1].name, format!("call-{}", 10));
    }

    #[test]
    fn test_hist_kind_equality() {
        assert_eq!(HistKind::Tool, HistKind::Tool);
        assert_eq!(HistKind::Prompt, HistKind::Prompt);
        assert_ne!(HistKind::Tool, HistKind::Prompt);
    }
}
