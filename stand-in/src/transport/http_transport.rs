//! Streamable HTTP transport — serves MCP over HTTP with SSE support.

use std::net::SocketAddr;
use std::sync::Arc;

use async_trait::async_trait;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::{debug, info, warn};

use crate::error::Result;
use crate::protocol::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use crate::server::RequestHandler;

use super::Transport;
use super::session_store::SessionStore;
use super::sse;

/// Shared state for axum route handlers.
#[derive(Debug, Clone)]
struct AppState {
    handler: Arc<RequestHandler>,
    sessions: SessionStore,
}

/// Streamable HTTP transport for MCP servers.
///
/// Implements the MCP 2025-03-26 Streamable HTTP specification:
/// - `POST /mcp` — JSON-RPC request/response (creates session on `initialize`)
/// - `GET /mcp` — SSE stream for server-initiated notifications
/// - `DELETE /mcp` — Session termination
///
/// # Examples
///
/// ```rust,ignore
/// use stand_in::prelude::*;
///
/// // Default: 127.0.0.1:3000
/// MyServer::serve(HttpTransport::default()).await?;
///
/// // Custom address
/// MyServer::serve(HttpTransport::new(([0, 0, 0, 0], 8080))).await?;
/// ```
#[derive(Debug, Clone)]
pub struct HttpTransport {
    addr: SocketAddr,
}

impl HttpTransport {
    /// Create an HTTP transport bound to the given address.
    pub fn new(addr: impl Into<SocketAddr>) -> Self {
        Self { addr: addr.into() }
    }

    /// Return the bind address.
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }
}

impl Default for HttpTransport {
    fn default() -> Self {
        Self {
            addr: SocketAddr::from(([127, 0, 0, 1], 3000)),
        }
    }
}

#[async_trait]
impl Transport for HttpTransport {
    async fn run(&self, handler: RequestHandler) -> Result<()> {
        let state = AppState {
            handler: Arc::new(handler),
            sessions: SessionStore::new(),
        };

        let app = Router::new()
            .route("/mcp", post(handle_post))
            .route("/mcp", get(handle_get))
            .route("/mcp", delete(handle_delete))
            .layer(CorsLayer::permissive())
            .with_state(state);

        let listener = TcpListener::bind(self.addr).await?;
        print_banner(self.addr);
        info!(addr = %self.addr, "HttpTransport listening");
        info!("Endpoint: POST|GET|DELETE /mcp");

        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await?;

        info!("HttpTransport shut down");
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Route handlers
// ---------------------------------------------------------------------------

const SESSION_HEADER: &str = "mcp-session-id";

/// POST /mcp — JSON-RPC request dispatch.
async fn handle_post(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<JsonRpcRequest>,
) -> impl IntoResponse {
    let session_id = headers
        .get(SESSION_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let is_initialize = request.method == "initialize";
    let id = request.id.clone().unwrap_or(serde_json::Value::Null);

    debug!(
        method = %request.method,
        session_id = ?session_id,
        "POST /mcp"
    );

    // Non-initialize requests require a valid session
    if !is_initialize {
        match &session_id {
            None => {
                warn!(method = %request.method, "POST /mcp rejected: missing Mcp-Session-Id header");
                return (
                    StatusCode::BAD_REQUEST,
                    HeaderMap::new(),
                    Json(JsonRpcResponse::error(
                        id,
                        JsonRpcError::invalid_request("Missing Mcp-Session-Id header"),
                    )),
                );
            }
            Some(sid) => {
                if !state.sessions.validate(sid).await {
                    warn!(session_id = %sid, "POST /mcp rejected: unknown session");
                    return (
                        StatusCode::NOT_FOUND,
                        HeaderMap::new(),
                        Json(JsonRpcResponse::error(
                            id,
                            JsonRpcError::invalid_request("Unknown session"),
                        )),
                    );
                }
            }
        }
    }

    // Dispatch the request
    let response = state.handler.handle(&request).await;

    // On successful initialize, create a session
    let mut response_headers = HeaderMap::new();
    if is_initialize && response.error.is_none() {
        let new_session_id = state.sessions.create().await;
        info!(session_id = %new_session_id, "Session created on initialize");
        if let Ok(val) = new_session_id.parse() {
            response_headers.insert(SESSION_HEADER, val);
        }
    } else if let Some(sid) = &session_id
        && let Ok(val) = sid.parse()
    {
        response_headers.insert(SESSION_HEADER, val);
    }

    debug!(method = %request.method, status = 200, "POST /mcp response sent");
    (StatusCode::OK, response_headers, Json(response))
}

/// GET /mcp — SSE notification stream.
async fn handle_get(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> std::result::Result<impl IntoResponse, StatusCode> {
    let session_id = headers.get(SESSION_HEADER).and_then(|v| v.to_str().ok());

    let session_id = match session_id {
        Some(sid) => sid,
        None => {
            warn!("GET /mcp rejected: missing Mcp-Session-Id header");
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    let rx = state
        .sessions
        .with_session(session_id, |s| s.subscribe_notifications())
        .await;

    let rx = match rx {
        Some(rx) => rx,
        None => {
            warn!(session_id = %session_id, "GET /mcp rejected: unknown session");
            return Err(StatusCode::NOT_FOUND);
        }
    };

    info!(session_id = %session_id, "SSE notification stream opened");
    Ok(sse::notification_stream(rx, Some(session_id.to_string())))
}

/// DELETE /mcp — session termination.
async fn handle_delete(State(state): State<AppState>, headers: HeaderMap) -> StatusCode {
    let session_id = match headers.get(SESSION_HEADER).and_then(|v| v.to_str().ok()) {
        Some(id) => id,
        None => {
            warn!("DELETE /mcp rejected: missing Mcp-Session-Id header");
            return StatusCode::BAD_REQUEST;
        }
    };

    if state.sessions.remove(session_id).await {
        info!(session_id = %session_id, "Session terminated");
        StatusCode::OK
    } else {
        warn!(session_id = %session_id, "DELETE /mcp rejected: unknown session");
        StatusCode::NOT_FOUND
    }
}

// ---------------------------------------------------------------------------
// Banner
// ---------------------------------------------------------------------------

fn print_banner(addr: SocketAddr) {
    let version = env!("CARGO_PKG_VERSION");
    println!(
        r"
 ███████ ████████  █████  ███    ██ ██████          ██ ███    ██
 ██         ██    ██   ██ ████   ██ ██   ██         ██ ████   ██
 ███████    ██    ███████ ██ ██  ██ ██   ██ ██████  ██ ██ ██  ██
      ██    ██    ██   ██ ██  ██ ██ ██   ██         ██ ██  ██ ██
 ███████    ██    ██   ██ ██   ████ ██████          ██ ██   ████

  v{version} | MCP 2025-03-26 | Streamable HTTP
  Listening on http://{addr}
"
    );
}

// ---------------------------------------------------------------------------
// Graceful shutdown
// ---------------------------------------------------------------------------

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }

    info!("Shutdown signal received");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_transport_default() {
        let transport = HttpTransport::default();
        assert_eq!(transport.addr(), SocketAddr::from(([127, 0, 0, 1], 3000)));
    }

    #[test]
    fn test_http_transport_custom_addr() {
        let transport = HttpTransport::new(([0, 0, 0, 0], 8080));
        assert_eq!(transport.addr(), SocketAddr::from(([0, 0, 0, 0], 8080)));
    }

    #[test]
    fn test_http_transport_debug() {
        let transport = HttpTransport::default();
        let debug = format!("{transport:?}");
        assert!(debug.contains("HttpTransport"));
    }
}
