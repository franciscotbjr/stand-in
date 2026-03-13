//! MCP session — represents a single client connection with session state.

use tokio::sync::broadcast;

/// Represents a single MCP client session.
///
/// Created during the `initialize` handshake and tracked via the
/// `Mcp-Session-Id` HTTP header for the lifetime of the connection.
#[derive(Debug)]
pub struct Session {
    /// Unique session identifier (UUID v4).
    id: String,

    /// Broadcast sender for server-initiated notifications (SSE).
    notification_tx: broadcast::Sender<String>,
}

impl Session {
    /// Create a new session with the given ID.
    pub fn new(id: String) -> Self {
        let (notification_tx, _) = broadcast::channel(64);
        Self {
            id,
            notification_tx,
        }
    }

    /// Return the session ID.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Return a clone of the notification broadcast sender.
    pub fn notification_tx(&self) -> broadcast::Sender<String> {
        self.notification_tx.clone()
    }

    /// Subscribe to server-initiated notifications.
    pub fn subscribe_notifications(&self) -> broadcast::Receiver<String> {
        self.notification_tx.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_new() {
        let session = Session::new("test-id".to_string());
        assert_eq!(session.id(), "test-id");
    }

    #[test]
    fn test_session_notification_channel() {
        let session = Session::new("test-id".to_string());
        let mut rx = session.subscribe_notifications();
        let tx = session.notification_tx();

        tx.send("hello".to_string()).unwrap();
        let msg = rx.try_recv().unwrap();
        assert_eq!(msg, "hello");
    }
}
