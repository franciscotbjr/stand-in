//! Active tab routing — pure `Tab` enum, `selected_ix()`, `tab_count_visible()`,
//! and `work_mode()` selector. Unit-testable without a Window.

use crate::app::conn_state::ConnState;
use crate::app::events::ServerSnapshot;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Tools,
    Resources,
    Prompts,
    Notifications,
    History,
}

impl Tab {
    pub fn selected_ix(self) -> usize {
        match self {
            Tab::Tools => 0,
            Tab::Resources => 1,
            Tab::Prompts => 2,
            Tab::Notifications => 3,
            Tab::History => 4,
        }
    }

    /// Stable lowercase label for perf instrumentation (037).
    #[cfg(feature = "perf")]
    pub fn perf_name(self) -> &'static str {
        match self {
            Tab::Tools => "tools",
            Tab::Resources => "resources",
            Tab::Prompts => "prompts",
            Tab::Notifications => "notifications",
            Tab::History => "history",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkMode {
    Disconnected,
    Error,
    Split,
    Single,
}

pub struct TabCounts {
    pub tools: usize,
    pub resources: usize,
    pub prompts: usize,
    pub notifications: usize,
    pub history: usize,
}

impl TabCounts {
    pub fn from_snapshot(snap: &ServerSnapshot) -> Self {
        Self {
            tools: snap.tools.len(),
            resources: snap.resources.len() + snap.templates.len(),
            prompts: snap.prompts.len(),
            notifications: 0,
            history: 0,
        }
    }

    pub const ZERO: Self = Self {
        tools: 0,
        resources: 0,
        prompts: 0,
        notifications: 0,
        history: 0,
    };
}

pub fn tab_count_visible(tab: Tab, counts: &TabCounts) -> Option<usize> {
    match tab {
        Tab::Tools => Some(counts.tools),
        Tab::Resources => Some(counts.resources),
        Tab::Prompts => Some(counts.prompts),
        Tab::Notifications if counts.notifications > 0 => Some(counts.notifications),
        Tab::History if counts.history > 0 => Some(counts.history),
        _ => None,
    }
}

pub fn work_mode(state: &ConnState, active_tab: Tab) -> WorkMode {
    match state {
        ConnState::Disconnected => WorkMode::Disconnected,
        ConnState::Connecting => WorkMode::Split,
        ConnState::Connected(_) => match active_tab {
            Tab::Tools | Tab::Resources | Tab::Prompts => WorkMode::Split,
            Tab::Notifications | Tab::History => WorkMode::Single,
        },
        ConnState::Error(_) => WorkMode::Error,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stand_in_client::prelude::{
        InputSchema, PromptDefinition, Resource, ResourceTemplate, ServerCapabilities, ServerInfo,
        ToolDefinition,
    };

    fn snapshot() -> ServerSnapshot {
        ServerSnapshot {
            server_info: ServerInfo {
                name: "test".into(),
                version: "1.0".into(),
            },
            capabilities: ServerCapabilities::new(),
            tools: vec![
                ToolDefinition {
                    name: "greet".into(),
                    description: "hello".into(),
                    input_schema: InputSchema::object(),
                },
                ToolDefinition {
                    name: "weather".into(),
                    description: "weather".into(),
                    input_schema: InputSchema::object(),
                },
            ],
            resources: vec![Resource {
                uri: "file:///a".into(),
                name: "a".into(),
                description: None,
                mime_type: None,
                annotations: None,
                size: None,
            }],
            templates: vec![ResourceTemplate {
                uri_template: "file:///{p}".into(),
                name: "files".into(),
                description: None,
                mime_type: None,
            }],
            prompts: vec![PromptDefinition {
                name: "summarize".into(),
                description: String::new(),
                arguments: None,
            }],
            latency_ms: 57,
        }
    }

    #[test]
    fn test_selected_ix() {
        assert_eq!(Tab::Tools.selected_ix(), 0);
        assert_eq!(Tab::Resources.selected_ix(), 1);
        assert_eq!(Tab::Prompts.selected_ix(), 2);
        assert_eq!(Tab::Notifications.selected_ix(), 3);
        assert_eq!(Tab::History.selected_ix(), 4);
    }

    #[test]
    fn test_tab_count_from_snapshot() {
        let counts = TabCounts::from_snapshot(&snapshot());
        assert_eq!(counts.tools, 2);
        assert_eq!(counts.resources, 2); // 1 resource + 1 template
        assert_eq!(counts.prompts, 1);
        assert_eq!(counts.notifications, 0);
        assert_eq!(counts.history, 0);
    }

    #[test]
    fn test_tab_count_visible_always_for_tools_res_prompts() {
        let counts = TabCounts::from_snapshot(&snapshot());
        assert_eq!(tab_count_visible(Tab::Tools, &counts), Some(2));
        assert_eq!(tab_count_visible(Tab::Resources, &counts), Some(2));
        assert_eq!(tab_count_visible(Tab::Prompts, &counts), Some(1));
    }

    #[test]
    fn test_tab_count_hidden_when_zero() {
        assert_eq!(
            tab_count_visible(Tab::Notifications, &TabCounts::ZERO),
            None
        );
        assert_eq!(tab_count_visible(Tab::History, &TabCounts::ZERO), None);
    }

    #[test]
    fn test_tab_count_visible_when_nonzero() {
        let counts = TabCounts {
            notifications: 3,
            history: 1,
            ..TabCounts::ZERO
        };
        assert_eq!(tab_count_visible(Tab::Notifications, &counts), Some(3));
        assert_eq!(tab_count_visible(Tab::History, &counts), Some(1));
    }

    #[test]
    fn test_work_mode_disconnected() {
        assert_eq!(
            work_mode(&ConnState::Disconnected, Tab::Tools),
            WorkMode::Disconnected
        );
    }

    #[test]
    fn test_work_mode_error() {
        assert_eq!(
            work_mode(&ConnState::Error("fail".into()), Tab::Tools),
            WorkMode::Error
        );
    }

    #[test]
    fn test_work_mode_connecting_split() {
        // Connecting shows split scaffold (tabbar is not rendered but work area uses split placeholder)
        assert_eq!(
            work_mode(&ConnState::Connecting, Tab::Tools),
            WorkMode::Split
        );
    }

    #[test]
    fn test_work_mode_connected_split() {
        let snap = snapshot();
        let state = ConnState::Connected(Box::new(snap));
        assert_eq!(work_mode(&state, Tab::Tools), WorkMode::Split);
        assert_eq!(work_mode(&state, Tab::Resources), WorkMode::Split);
        assert_eq!(work_mode(&state, Tab::Prompts), WorkMode::Split);
    }

    #[test]
    fn test_work_mode_connected_single() {
        let snap = snapshot();
        let state = ConnState::Connected(Box::new(snap));
        assert_eq!(work_mode(&state, Tab::Notifications), WorkMode::Single);
        assert_eq!(work_mode(&state, Tab::History), WorkMode::Single);
    }
}
