//! Event log buffer — every `EngineEvent` produces a `LogEntry` at drain time,
//! consumed by the Notifications tab (M13). Pure helpers (`log_for_event`,
//! `filter_logs`) are testable without a window or subprocess.
//!
//! The buffer grows in chronological order; the render reverses it
//! (newest-first). A cap of 500 entries prevents unbounded growth.

use crate::app::events::EngineEvent;
use stand_in_client::prelude::Notification;
use stand_in_mcp_explorer_ds::data::LogLevel;

const LOG_CAP: usize = 500;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    pub time: String,
    pub level: LogLevel,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogFilter {
    #[default]
    All,
    Level(LogLevel),
}

impl LogFilter {
    pub fn selected_ix(self) -> usize {
        match self {
            Self::All => 0,
            Self::Level(LogLevel::Info) => 1,
            Self::Level(LogLevel::Ok) => 2,
            Self::Level(LogLevel::Warn) => 3,
            Self::Level(LogLevel::Error) => 4,
            Self::Level(LogLevel::Debug) => 0,
        }
    }

    pub fn from_ix(ix: usize) -> Self {
        match ix {
            1 => Self::Level(LogLevel::Info),
            2 => Self::Level(LogLevel::Ok),
            3 => Self::Level(LogLevel::Warn),
            4 => Self::Level(LogLevel::Error),
            _ => Self::All,
        }
    }
}

pub fn push_log(logs: &mut Vec<LogEntry>, entry: LogEntry) {
    logs.push(entry);
    if logs.len() > LOG_CAP {
        logs.remove(0);
    }
}

pub fn filter_logs<'a>(logs: &'a [LogEntry], filter: &LogFilter) -> Vec<&'a LogEntry> {
    match filter {
        LogFilter::All => logs.iter().collect(),
        LogFilter::Level(lvl) => logs.iter().filter(|e| e.level == *lvl).collect(),
    }
}

pub fn log_for_event(event: &EngineEvent, time: String) -> Option<LogEntry> {
    let (level, message) = match event {
        EngineEvent::Connecting => (LogLevel::Info, "conectando…".into()),
        EngineEvent::Connected(snap) => (
            LogLevel::Ok,
            format!(
                "conectado a {} ({}ms)",
                snap.server_info.name, snap.latency_ms
            ),
        ),
        EngineEvent::ConnectionError(e) => (LogLevel::Error, e.clone()),
        EngineEvent::Disconnected => (LogLevel::Info, "conexao encerrada".into()),
        EngineEvent::ToolResult(r) => {
            if r.is_error == Some(true) {
                (
                    LogLevel::Error,
                    format!("tool call failed ({:?})", r.content),
                )
            } else {
                (LogLevel::Ok, "tool call ok".into())
            }
        }
        EngineEvent::ToolError(e) => (LogLevel::Error, format!("tool call error: {}", e)),
        EngineEvent::ResourceResult(_r) => (LogLevel::Ok, "resource read".into()),
        EngineEvent::ResourceError(e) => (LogLevel::Error, format!("resource error: {}", e)),
        EngineEvent::Subscribed(uri) => (LogLevel::Info, format!("inscrito {}", uri)),
        EngineEvent::Unsubscribed(_uri) => (LogLevel::Info, "unsubscribed".into()),
        EngineEvent::PromptMessages(_r) => (LogLevel::Ok, "preview generated".into()),
        EngineEvent::PromptError(e) => (LogLevel::Error, format!("prompt error: {}", e)),
        EngineEvent::Notification(n) => match n {
            Notification::ResourcesUpdated { uri } => {
                (LogLevel::Info, format!("resource updated: {}", uri))
            }
            Notification::ResourcesListChanged => (LogLevel::Info, "resources list changed".into()),
            Notification::ToolsListChanged => (LogLevel::Info, "tools list changed".into()),
            Notification::Disconnected => (LogLevel::Warn, "transport closed".into()),
            Notification::Other { method, .. } => {
                (LogLevel::Debug, format!("notification: {}", method))
            }
            _ => (LogLevel::Debug, "unknown notification".into()),
        },
        EngineEvent::Authorized(_) => (LogLevel::Ok, "oauth authorized".into()),
        EngineEvent::AuthorizationError(e) => (LogLevel::Error, format!("oauth error: {}", e)),
    };
    Some(LogEntry {
        time,
        level,
        message,
    })
}

pub fn now_time() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let total_secs = dur.as_secs();
    let millis = dur.subsec_millis();
    let hours = ((total_secs / 3600) % 24) as u32;
    let mins = ((total_secs / 60) % 60) as u32;
    let secs = (total_secs % 60) as u32;
    format!("{:02}:{:02}:{:02}.{:03}", hours, mins, secs, millis)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::events::ServerSnapshot;
    use stand_in_client::prelude::ServerInfo;

    fn snap() -> ServerSnapshot {
        ServerSnapshot {
            server_info: ServerInfo {
                name: "test-server".into(),
                version: "1.0".into(),
            },
            capabilities: stand_in_client::prelude::ServerCapabilities::new(),
            tools: vec![],
            resources: vec![],
            templates: vec![],
            prompts: vec![],
            latency_ms: 42,
        }
    }

    fn time() -> String {
        "12:34:56.789".into()
    }

    #[test]
    fn test_log_for_connecting() {
        let e = log_for_event(&EngineEvent::Connecting, time()).unwrap();
        assert_eq!(e.level, LogLevel::Info);
        assert_eq!(e.message, "conectando…");
    }

    #[test]
    fn test_log_for_connected() {
        let e = log_for_event(&EngineEvent::Connected(Box::new(snap())), time()).unwrap();
        assert_eq!(e.level, LogLevel::Ok);
        assert_eq!(e.message, "conectado a test-server (42ms)");
    }

    #[test]
    fn test_log_for_connection_error() {
        let e = log_for_event(&EngineEvent::ConnectionError("timeout".into()), time()).unwrap();
        assert_eq!(e.level, LogLevel::Error);
        assert_eq!(e.message, "timeout");
    }

    #[test]
    fn test_log_for_disconnected() {
        let e = log_for_event(&EngineEvent::Disconnected, time()).unwrap();
        assert_eq!(e.level, LogLevel::Info);
        assert_eq!(e.message, "conexao encerrada");
    }

    #[test]
    fn test_log_for_tool_result_error() {
        use stand_in_client::prelude::CallToolResult;
        let result = CallToolResult {
            content: vec![stand_in_client::prelude::Content::text("fail")],
            is_error: Some(true),
        };
        let e = log_for_event(&EngineEvent::ToolResult(Box::new(result)), time()).unwrap();
        assert_eq!(e.level, LogLevel::Error);
    }

    #[test]
    fn test_log_for_tool_result_ok() {
        use stand_in_client::prelude::CallToolResult;
        let result = CallToolResult {
            content: vec![],
            is_error: None,
        };
        let e = log_for_event(&EngineEvent::ToolResult(Box::new(result)), time()).unwrap();
        assert_eq!(e.level, LogLevel::Ok);
    }

    #[test]
    fn test_log_for_notification_resources_updated() {
        let n = Notification::ResourcesUpdated {
            uri: "file:///data".into(),
        };
        let e = log_for_event(&EngineEvent::Notification(n), time()).unwrap();
        assert_eq!(e.level, LogLevel::Info);
        assert!(e.message.contains("resource updated"));
    }

    #[test]
    fn test_log_for_notification_disconnected() {
        let n = Notification::Disconnected;
        let e = log_for_event(&EngineEvent::Notification(n), time()).unwrap();
        assert_eq!(e.level, LogLevel::Warn);
        assert_eq!(e.message, "transport closed");
    }

    #[test]
    fn test_log_for_subscribed() {
        let e = log_for_event(&EngineEvent::Subscribed("file:///x".into()), time()).unwrap();
        assert_eq!(e.level, LogLevel::Info);
        assert_eq!(e.message, "inscrito file:///x");
    }

    #[test]
    fn test_filter_logs_all() {
        let logs = vec![
            LogEntry {
                time: time(),
                level: LogLevel::Info,
                message: "a".into(),
            },
            LogEntry {
                time: time(),
                level: LogLevel::Error,
                message: "b".into(),
            },
        ];
        let result = filter_logs(&logs, &LogFilter::All);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_filter_logs_error_only() {
        let logs = vec![
            LogEntry {
                time: time(),
                level: LogLevel::Info,
                message: "a".into(),
            },
            LogEntry {
                time: time(),
                level: LogLevel::Error,
                message: "b".into(),
            },
            LogEntry {
                time: time(),
                level: LogLevel::Info,
                message: "c".into(),
            },
        ];
        let result = filter_logs(&logs, &LogFilter::Level(LogLevel::Error));
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].message, "b");
    }

    #[test]
    fn test_filter_logs_preserves_order() {
        let logs: Vec<LogEntry> = (0..5)
            .map(|i| LogEntry {
                time: format!("{:02}:00:00.000", i),
                level: if i % 2 == 0 {
                    LogLevel::Info
                } else {
                    LogLevel::Ok
                },
                message: format!("msg {}", i),
            })
            .collect();
        let result = filter_logs(&logs, &LogFilter::All);
        assert_eq!(result.len(), 5);
        for (i, e) in result.iter().enumerate() {
            assert_eq!(e.message, format!("msg {}", i));
        }
    }

    #[test]
    fn test_log_filter_selected_ix() {
        assert_eq!(LogFilter::All.selected_ix(), 0);
        assert_eq!(LogFilter::Level(LogLevel::Info).selected_ix(), 1);
        assert_eq!(LogFilter::Level(LogLevel::Ok).selected_ix(), 2);
        assert_eq!(LogFilter::Level(LogLevel::Warn).selected_ix(), 3);
        assert_eq!(LogFilter::Level(LogLevel::Error).selected_ix(), 4);
    }

    #[test]
    fn test_log_filter_from_ix() {
        assert_eq!(LogFilter::from_ix(0), LogFilter::All);
        assert_eq!(LogFilter::from_ix(1), LogFilter::Level(LogLevel::Info));
        assert_eq!(LogFilter::from_ix(2), LogFilter::Level(LogLevel::Ok));
        assert_eq!(LogFilter::from_ix(3), LogFilter::Level(LogLevel::Warn));
        assert_eq!(LogFilter::from_ix(4), LogFilter::Level(LogLevel::Error));
        assert_eq!(LogFilter::from_ix(99), LogFilter::All);
    }

    #[test]
    fn test_push_log_with_cap() {
        let mut logs: Vec<LogEntry> = Vec::new();
        for i in 0..LOG_CAP + 10 {
            push_log(
                &mut logs,
                LogEntry {
                    time: time(),
                    level: LogLevel::Info,
                    message: format!("msg {}", i),
                },
            );
        }
        assert_eq!(logs.len(), LOG_CAP);
        assert_eq!(logs.first().unwrap().message, "msg 10");
        assert_eq!(logs.last().unwrap().message, format!("msg {}", LOG_CAP + 9));
    }

    #[test]
    fn test_log_level_entry_fields() {
        let entry = LogEntry {
            time: "14:02:31.457".into(),
            level: LogLevel::Warn,
            message: "timeout".into(),
        };
        assert_eq!(entry.time, "14:02:31.457");
        assert_eq!(entry.level, LogLevel::Warn);
        assert_eq!(entry.message, "timeout");
    }
}
