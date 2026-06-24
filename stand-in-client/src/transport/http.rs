//! Streamable HTTP client transport — implements `ClientTransport` over
//! the MCP 2025-03-26 Streamable HTTP protocol.
//!
//! Manages `POST /mcp` for request/response, `GET /mcp` for SSE notifications
//! server→client, and `DELETE /mcp` on close. Handles `Mcp-Session-Id`
//! capture, replay, and the background SSE task.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_stream::Stream;
use tokio_stream::StreamExt;
use tracing::{debug, warn};

use crate::auth::Credential;
use crate::error::{Error, Result};

use super::ClientTransport;
use super::sse::SseParser;

/// Streamable HTTP client transport.
///
/// Communicates with an MCP server over HTTP using the Streamable HTTP
/// protocol (MCP 2025-03-26). Request/response goes over `POST /mcp`;
/// server→client notifications arrive over a persistent `GET /mcp` SSE
/// stream; `DELETE /mcp` tears down the session on close.
///
/// # Example
///
/// ```rust,no_run
/// # use stand_in_client::transport::HttpTransport;
/// # use stand_in_client::transport::ClientTransport;
/// # #[tokio::main]
/// # async fn main() -> stand_in_client::error::Result<()> {
/// let mut t = HttpTransport::new("http://127.0.0.1:3000/mcp");
/// t.connect().await?;
/// t.send(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{...}}"#).await?;
/// let response = t.receive().await?;
/// t.close().await?;
/// # Ok(())
/// # }
/// ```
///
/// # Anti-regression
///
/// - `connect()` is a **no-op** — no GET health-check (the 3rd-attempt bug).
/// - The SSE parser **emits every event** — never discards (the 2nd-cause-of-death bug).
/// - `Mcp-Session-Id` is **captured from the initialize response** and **replayed**
///   on every subsequent POST. GET SSE is only opened **after** session capture.
pub struct HttpTransport {
    client: reqwest::Client,
    url: String,
    credential: Credential,
    session_id: Mutex<Option<String>>,
    incoming_tx: Mutex<Option<mpsc::UnboundedSender<String>>>,
    incoming_rx: tokio::sync::Mutex<mpsc::UnboundedReceiver<String>>,
    sse_task: Mutex<Option<JoinHandle<()>>>,
    sse_started: AtomicBool,
}

impl HttpTransport {
    /// Create a new HTTP transport targeting the given MCP endpoint URL.
    ///
    /// The URL should point to the server's MCP endpoint, e.g.
    /// `"http://127.0.0.1:3000/mcp"`.
    pub fn new(url: impl Into<String>) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            client: reqwest::Client::new(),
            url: url.into(),
            credential: Credential::default(),
            session_id: Mutex::new(None),
            incoming_tx: Mutex::new(Some(tx)),
            incoming_rx: tokio::sync::Mutex::new(rx),
            sse_task: Mutex::new(None),
            sse_started: AtomicBool::new(false),
        }
    }

    /// Set the credential used for all HTTP requests (POST, GET-SSE, DELETE).
    ///
    /// The credential's [`authorization_header`] value is injected as the
    /// `Authorization` header. Default is [`Credential::NoAuth`] (no header).
    ///
    /// [`authorization_header`]: Credential::authorization_header
    pub fn with_credential(mut self, credential: Credential) -> Self {
        self.credential = credential;
        self
    }
}

#[async_trait]
impl ClientTransport for HttpTransport {
    /// Establish the connection — **no-op**.
    ///
    /// The first network call is the `initialize` POST, dispatched by the
    /// `Client` actor during handshake. There is **no** GET health-check
    /// (a regression that broke the 3rd attempt at the client).
    async fn connect(&mut self) -> Result<()> {
        Ok(())
    }

    /// Send a JSON-RPC message via `POST /mcp`.
    ///
    /// 1. Builds headers (`Content-Type`, `Accept`, optional `Mcp-Session-Id`).
    /// 2. POSTs the raw JSON string as the request body.
    /// 3. **Captures `Mcp-Session-Id`** from the response header on first
    ///    successful response, and spawns the background GET-SSE task.
    /// 4. Reads the response body; for `application/json` it enqueues the
    ///    body directly; for `text/event-stream` it parses all data events
    ///    and enqueues each one.
    async fn send(&self, message: &str) -> Result<()> {
        let auth = self.credential.authorization_header();
        let sid = self.session_id.lock().unwrap().clone();

        let headers = crate::auth::build_post_headers(sid.as_deref(), auth.as_deref())?;

        let resp = self
            .client
            .post(&self.url)
            .headers(headers)
            .body(message.to_string())
            .send()
            .await
            .map_err(|e| Error::ConnectionError(format!("POST failed: {e}")))?;

        if sid.is_none()
            && let Some(new_sid) = resp
                .headers()
                .get("mcp-session-id")
                .and_then(|v| v.to_str().ok())
        {
            *self.session_id.lock().unwrap() = Some(new_sid.to_string());

            // Spawn the background GET-SSE task exactly once.
            if !self.sse_started.swap(true, Ordering::AcqRel) {
                let tx = self.incoming_tx.lock().unwrap().as_ref().cloned();
                if let Some(tx) = tx {
                    let auth = auth.clone();
                    let task = tokio::spawn(run_sse_loop(
                        self.client.clone(),
                        self.url.clone(),
                        new_sid.to_string(),
                        auth,
                        tx,
                    ));
                    *self.sse_task.lock().unwrap() = Some(task);
                }
            }
        }

        let content_type = resp
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        let tx = {
            let tx_guard = self.incoming_tx.lock().unwrap();
            tx_guard.clone()
        };
        let tx = match tx {
            Some(tx) => tx,
            None => {
                return Err(Error::TransportClosed("incoming channel closed".into()));
            }
        };

        if content_type.contains("text/event-stream") {
            let body = resp.text().await.map_err(|e| {
                Error::ConnectionError(format!("failed to read SSE response body: {e}"))
            })?;
            let mut parser = SseParser::new();
            let events = parser.feed(body.as_bytes());
            for event in events {
                let _ = tx.send(event);
            }
            if let Some(last) = parser.finish() {
                let _ = tx.send(last);
            }
        } else {
            let body = resp.text().await.map_err(|e| {
                Error::ConnectionError(format!("failed to read response body: {e}"))
            })?;
            let _ = tx.send(body);
        }

        Ok(())
    }

    /// Receive the next message from the server.
    ///
    /// Drains the internal `mpsc` queue that is fed by both POST responses
    /// and the background GET-SSE task. Returns `Ok(None)` when the channel
    /// is closed (after `close()`).
    async fn receive(&self) -> Result<Option<String>> {
        let mut rx = self.incoming_rx.lock().await;
        match rx.recv().await {
            Some(line) => Ok(Some(line)),
            None => Ok(None),
        }
    }

    /// Gracefully close the transport.
    ///
    /// Sends a best-effort `DELETE /mcp` with the session ID, aborts the
    /// background SSE task, and drops the `incoming_tx` sender so that
    /// `receive()` returns `None` and the read-loop shuts down.
    async fn close(&self) -> Result<()> {
        let auth = self.credential.authorization_header();

        // Best-effort DELETE
        {
            let sid = self.session_id.lock().unwrap().clone();
            if let Some(ref sid) = sid {
                let mut req = self.client.delete(&self.url).header("Mcp-Session-Id", sid);
                if let Some(ref auth_val) = auth {
                    req = req.header(reqwest::header::AUTHORIZATION, auth_val);
                }
                match req.send().await {
                    Ok(resp) => {
                        debug!(status = %resp.status(), "DELETE /mcp sent");
                    }
                    Err(e) => {
                        debug!("DELETE /mcp failed (ignored): {e}");
                    }
                }
            }
        }

        // Abort the SSE task
        {
            let mut task_guard = self.sse_task.lock().unwrap();
            if let Some(task) = task_guard.take() {
                task.abort();
            }
        }

        // Drop the incoming channel sender — this causes receive() to return None
        {
            let mut tx_guard = self.incoming_tx.lock().unwrap();
            *tx_guard = None;
        }

        Ok(())
    }
}

/// Process SSE bytes from a stream, parsing events and sending them to `tx`.
///
/// This is the inner loop of `run_sse_loop`, factored out for independent
/// testability. Each chunk from the stream is fed through an incremental
/// `SseParser`; complete events are forwarded to the channel. When the
/// stream ends, any partial event is flushed.
pub(crate) async fn process_sse_stream<B: AsRef<[u8]>, E: std::fmt::Debug>(
    mut stream: impl Stream<Item = std::result::Result<B, E>> + Unpin,
    tx: mpsc::UnboundedSender<String>,
) {
    let mut parser = SseParser::new();
    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(bytes) => {
                let events = parser.feed(bytes.as_ref());
                for event in events {
                    if tx.send(event).is_err() {
                        debug!("SSE stream: incoming channel closed, exiting");
                        return;
                    }
                }
            }
            Err(e) => {
                debug!("SSE stream error: {e:?}");
                break;
            }
        }
    }
    if let Some(last) = parser.finish() {
        let _ = tx.send(last);
    }
}

/// Background task: connects to the GET SSE endpoint and feeds all incoming
/// notification events into the `incoming` channel so the read-loop can
/// dispatch them as `Notification` variants.
async fn run_sse_loop(
    client: reqwest::Client,
    url: String,
    session_id: String,
    auth: Option<String>,
    tx: mpsc::UnboundedSender<String>,
) {
    let mut req = client
        .get(&url)
        .header(reqwest::header::ACCEPT, "text/event-stream")
        .header("Mcp-Session-Id", &session_id);
    if let Some(ref auth_val) = auth {
        req = req.header(reqwest::header::AUTHORIZATION, auth_val);
    }
    let resp = match req.send().await {
        Ok(r) => r,
        Err(e) => {
            warn!("SSE GET connection failed: {e}");
            return;
        }
    };

    debug!(%session_id, "SSE notification stream opened");
    process_sse_stream(resp.bytes_stream(), tx).await;
    debug!("SSE notification stream ended");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_transport_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<HttpTransport>();
    }

    /// Test that `process_sse_stream` correctly feeds synthetic SSE bytes
    /// through the parser and sends all events to the channel.  This covers
    /// the glue between stream-reading, incremental SSE parsing, and the
    /// mpsc sender — the "last unproven elo" from the M5 spec §6.2.
    #[tokio::test]
    async fn test_process_sse_stream_with_fake_stream() {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let chunks: Vec<std::result::Result<Vec<u8>, String>> = vec![
            Ok(b"data: event1\n\n".to_vec()),
            Ok(b": keep-alive\n\ndata: event2\n\n".to_vec()),
            Ok(b"data: line1\ndata: line2\n\n".to_vec()),
        ];
        let stream = tokio_stream::iter(chunks);

        let handle = tokio::spawn(process_sse_stream(stream, tx));
        handle.await.unwrap();

        let mut collected = Vec::new();
        while let Some(event) = rx.recv().await {
            collected.push(event);
        }
        assert_eq!(collected, vec!["event1", "event2", "line1\nline2"]);
    }
}
