//! Client-side transport abstraction.
//!
//! The `ClientTransport` trait is the seam between the MCP protocol core and
//! the wire, from the client's perspective. It is designed to support both:
//!
//! - **stdio** — subprocess with stdin/stdout split naturally into write/read halves.
//! - **Streamable HTTP** — POST for request/response + GET SSE for server→client notifications.
//!
//! All methods except `connect` take `&self` — the transport is fully shareable
//! behind an `Arc`. Implementations use interior mutability (`tokio::Mutex`) for
//! the read side and child handle, so the read-loop (one task) and senders (any task)
//! can operate concurrently on a shared transport without deadlock. The write side
//! (stdin) and the read side (stdout) lock different mutexes, never contending.

pub mod stdio;

pub use stdio::StdioTransport;

#[cfg(feature = "http")]
pub mod http;
#[cfg(feature = "http")]
pub use http::HttpTransport;

#[cfg(feature = "http")]
pub(crate) mod sse;

use crate::error::Result;
use async_trait::async_trait;

/// Client-side transport for communicating with an MCP server.
///
/// A transport represents a connection to an MCP server. Messages are sent as
/// newline-delimited JSON strings. The read-loop (lives in the `Client` actor)
/// drives the correlation/notification machinery over a shared transport.
///
/// # Concurrency design
///
/// `receive` and `close` take `&self` (not `&mut self`), so a transport wrapped
/// in `Arc<dyn ClientTransport>` can be cloned and used from multiple tasks —
/// one task runs the read-loop (`receive`), others send requests (`send`).
/// Interior mutability (`tokio::Mutex`) keeps each half independently locked.
///
/// # Implementations
///
/// | Transport | Feature | Shape |
/// |-----------|---------|-------|
/// | `StdioTransport` | always available | Subprocess stdin/stdout |
/// | `HttpTransport`  | `http`           | `POST`/`GET`/`DELETE /mcp` |
///
/// # Safety
///
/// All transports must be `Send + Sync + 'static` so they can be moved
/// into background tasks and shared between the request-sending task and
/// the read-loop task.
#[async_trait]
pub trait ClientTransport: Send + Sync + 'static {
    /// Establish the connection to the server.
    ///
    /// For stdio: launches the subprocess. For HTTP: opens the initial connection.
    /// Called once before sharing the transport via `Arc`. Takes `&mut self`
    /// because no concurrent access is possible before this step completes.
    async fn connect(&mut self) -> Result<()>;

    /// Send a JSON-RPC message to the server.
    ///
    /// The message is a complete, serialized JSON-RPC request or notification
    /// as a single string. The transport is responsible for framing
    /// (e.g., appending `\n` for stdio).
    async fn send(&self, message: &str) -> Result<()>;

    /// Receive the next message from the server.
    ///
    /// Returns `Ok(Some(message))` with the raw JSON-RPC response/notification
    /// string, or `Ok(None)` when the transport is closed (EOF).
    ///
    /// Takes `&self` so the read-loop task can share the transport with senders.
    /// Implementations use interior mutability (e.g. `tokio::Mutex<BufReader>`).
    async fn receive(&self) -> Result<Option<String>>;

    /// Gracefully close the transport.
    ///
    /// For stdio: closes stdin and kills the subprocess. For HTTP: sends `DELETE`.
    /// Idempotent — safe to call more than once.
    async fn close(&self) -> Result<()>;
}
