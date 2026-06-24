//! SSE (Server-Sent Events) helpers for the Streamable HTTP transport.

use axum::response::sse::{Event, KeepAlive, Sse};
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;
use tracing::{debug, info, trace};

use crate::protocol::JsonRpcResponse;

/// Convert a [`JsonRpcResponse`] into an SSE [`Event`].
///
/// The event data is the JSON-serialized response. The event type
/// is set to `"message"` per the MCP specification.
#[allow(dead_code)]
pub fn response_to_event(response: &JsonRpcResponse) -> Result<Event, serde_json::Error> {
    let json = serde_json::to_string(response)?;
    Ok(Event::default().event("message").data(json))
}

/// Guard that logs when the SSE stream is dropped (client disconnect or shutdown).
struct StreamDropGuard {
    session_id: String,
}

impl Drop for StreamDropGuard {
    fn drop(&mut self) {
        info!(session_id = %self.session_id, "SSE notification stream closed");
    }
}

/// Build an SSE response from a broadcast receiver.
///
/// Each received string is emitted as an SSE `message` event.
/// Lagged messages are silently skipped. Includes a keep-alive
/// that sends periodic comment frames to prevent connection timeout.
///
/// If `session_id` is provided, an info-level log is emitted when the
/// stream is dropped (client disconnect or server shutdown).
pub fn notification_stream(
    rx: tokio::sync::broadcast::Receiver<String>,
    session_id: Option<String>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, std::convert::Infallible>>> {
    let guard = session_id.map(|id| StreamDropGuard { session_id: id });

    let stream = BroadcastStream::new(rx).filter_map(move |result| {
        let _ = &guard;
        match result {
            Ok(data) => {
                trace!("SSE event emitted");
                Some(Ok(Event::default().event("message").data(data)))
            }
            Err(_) => {
                debug!("SSE lagged message skipped");
                None
            }
        }
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_to_event_success() {
        let response =
            JsonRpcResponse::success(serde_json::json!(1), serde_json::json!({"status": "ok"}));
        let event = response_to_event(&response);
        assert!(event.is_ok());
    }

    #[test]
    fn test_response_to_event_error() {
        let response = JsonRpcResponse::error(
            serde_json::json!(2),
            crate::protocol::JsonRpcError::internal_error("boom"),
        );
        let event = response_to_event(&response);
        assert!(event.is_ok());
    }
}
