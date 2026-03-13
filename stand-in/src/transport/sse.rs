//! SSE (Server-Sent Events) helpers for the Streamable HTTP transport.

use axum::response::sse::{Event, KeepAlive, Sse};
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;

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

/// Build an SSE response from a broadcast receiver.
///
/// Each received string is emitted as an SSE `message` event.
/// Lagged messages are silently skipped. Includes a keep-alive
/// that sends periodic comment frames to prevent connection timeout.
pub fn notification_stream(
    rx: tokio::sync::broadcast::Receiver<String>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, std::convert::Infallible>>> {
    let stream = BroadcastStream::new(rx).filter_map(|result| match result {
        Ok(data) => Some(Ok(Event::default().event("message").data(data))),
        Err(_) => None,
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
