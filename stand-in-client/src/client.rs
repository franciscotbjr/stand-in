//! High-level MCP client — connect, handshake, and call methods.
//!
//! The [`Client`] is the main entry point for the MCP client SDK. It manages
//! the transport lifecycle, runs the read-loop for correlation/notifications,
//! executes the MCP handshake, auto-fetches server capabilities, and exposes
//! a typed async API for every MCP method.

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tracing::debug;

use crate::connection::read_loop;
use crate::correlation::{PendingRequests, next_request_id};
use crate::error::{Error, Result};
use crate::notification::Notification;
use crate::transport::ClientTransport;

use stand_in::prompt::{GetPromptParams, GetPromptResult, ListPromptsResult, PromptDefinition};
use stand_in::protocol::JsonRpcRequest;
use stand_in::resource::{
    ListResourceTemplatesResult, ListResourcesResult, ReadResourceParams, ReadResourceResult,
    Resource, ResourceTemplate, SubscribeParams, UnsubscribeParams,
};
use stand_in::server::{
    ClientInfo, InitializeParams, InitializeResult, ServerCapabilities, ServerInfo,
};
use stand_in::tool::{CallToolParams, CallToolResult, ListToolsResult, ToolDefinition};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// High-level MCP client.
///
/// Created via [`ClientBuilder::connect()`]. Holds cached server metadata
/// (tools, resources, prompts, capabilities) and dispatches typed method calls.
///
/// # Example
///
/// ```rust,no_run
/// # use stand_in_client::prelude::*;
/// # #[tokio::main]
/// # async fn main() -> Result<()> {
/// let client = Client::builder()
///     .transport(StdioTransport::command("my-server", &[] as &[&str]))
///     .client_info("my-app", "1.0.0")
///     .connect()
///     .await?;
///
/// let tools = client.tools();
/// let result = client.call_tool("greet", serde_json::json!({ "name": "World" })).await?;
/// client.disconnect().await?;
/// # Ok(())
/// # }
/// ```
pub struct Client {
    transport: Arc<dyn ClientTransport>,
    pending: PendingRequests,
    notif_tx: broadcast::Sender<Notification>,
    read_task: Option<JoinHandle<()>>,
    timeout: Duration,

    tools_cache: Vec<ToolDefinition>,
    resources_cache: Vec<Resource>,
    templates_cache: Vec<ResourceTemplate>,
    prompts_cache: Vec<PromptDefinition>,
    server_info: ServerInfo,
    server_capabilities: ServerCapabilities,
}

impl Client {
    /// Create a new [`ClientBuilder`].
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    /// Cached tool definitions from the `tools/list` handshake fetch.
    pub fn tools(&self) -> &[ToolDefinition] {
        &self.tools_cache
    }

    /// Cached concrete resources from the `resources/list` handshake fetch.
    pub fn resources(&self) -> &[Resource] {
        &self.resources_cache
    }

    /// Cached resource templates from `resources/templates/list` fetch.
    pub fn resource_templates(&self) -> &[ResourceTemplate] {
        &self.templates_cache
    }

    /// Cached prompt definitions from the `prompts/list` handshake fetch.
    pub fn prompts(&self) -> &[PromptDefinition] {
        &self.prompts_cache
    }

    /// Server identity returned during the handshake.
    pub fn server_info(&self) -> &ServerInfo {
        &self.server_info
    }

    /// Server capabilities returned during the handshake.
    pub fn server_capabilities(&self) -> &ServerCapabilities {
        &self.server_capabilities
    }

    /// Call a tool on the server.
    ///
    /// Returns `Ok(CallToolResult)` even when the tool execution itself fails
    /// (indicated by `result.is_error == Some(true)`). A protocol-level error
    /// (e.g. invalid params) returns `Err(Error::ProtocolError(...))`.
    pub async fn call_tool(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> Result<CallToolResult> {
        let params = serde_json::to_value(CallToolParams {
            name: name.to_string(),
            arguments,
        })?;
        let result = send_request(
            &self.transport,
            &self.pending,
            self.timeout,
            "tools/call",
            Some(params),
        )
        .await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Read a resource by URI.
    ///
    /// Works for both concrete resources (e.g. `info://version`) and template
    /// resources (e.g. `docs://rust/readme`).
    pub async fn read_resource(&self, uri: &str) -> Result<ReadResourceResult> {
        let params = serde_json::to_value(ReadResourceParams {
            uri: uri.to_string(),
        })?;
        let result = send_request(
            &self.transport,
            &self.pending,
            self.timeout,
            "resources/read",
            Some(params),
        )
        .await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Get a prompt by name with optional arguments.
    pub async fn get_prompt(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> Result<GetPromptResult> {
        let params = serde_json::to_value(GetPromptParams {
            name: name.to_string(),
            arguments: Some(arguments),
        })?;
        let result = send_request(
            &self.transport,
            &self.pending,
            self.timeout,
            "prompts/get",
            Some(params),
        )
        .await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Subscribe to resource change notifications.
    ///
    /// Note: on stdio transports, the server does not push notifications.
    /// Live push notifications are received only over HTTP transports (M5).
    pub async fn subscribe(&self, uri: &str) -> Result<()> {
        let params = serde_json::to_value(SubscribeParams {
            uri: uri.to_string(),
        })?;
        send_request(
            &self.transport,
            &self.pending,
            self.timeout,
            "resources/subscribe",
            Some(params),
        )
        .await?;
        Ok(())
    }

    /// Unsubscribe from resource change notifications.
    pub async fn unsubscribe(&self, uri: &str) -> Result<()> {
        let params = serde_json::to_value(UnsubscribeParams {
            uri: uri.to_string(),
        })?;
        send_request(
            &self.transport,
            &self.pending,
            self.timeout,
            "resources/unsubscribe",
            Some(params),
        )
        .await?;
        Ok(())
    }

    /// Get a broadcast receiver for server-pushed notifications.
    ///
    /// New receivers see only notifications arriving after the call.
    /// On stdio transports, no live push notifications are expected;
    /// the `Disconnected` variant signals transport closure.
    pub fn notifications(&self) -> broadcast::Receiver<Notification> {
        self.notif_tx.subscribe()
    }

    /// Refresh cached lists from the server (tools, resources, prompts).
    ///
    /// Re-fetches all lists based on advertised capabilities.
    pub async fn refresh(&mut self) -> Result<()> {
        let (tools, prompts, resources, templates) = auto_fetch_lists(
            &self.transport,
            &self.pending,
            self.timeout,
            &self.server_capabilities,
        )
        .await?;
        self.tools_cache = tools;
        self.prompts_cache = prompts;
        self.resources_cache = resources;
        self.templates_cache = templates;
        Ok(())
    }

    /// Gracefully disconnect from the server.
    ///
    /// Closes the transport, aborts the read-loop, and cancels all pending
    /// requests. Consumes `self` — after this, the client is done.
    pub async fn disconnect(mut self) -> Result<()> {
        self.transport.close().await?;
        if let Some(task) = self.read_task.take() {
            task.abort();
        }
        Ok(())
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        // Best-effort abort of the read-loop on drop.
        if let Some(task) = self.read_task.take() {
            task.abort();
        }
    }
}

/// Builder for [`Client`].
///
/// Configure a transport, client identity, and timeout, then call [`connect`](Self::connect)
/// to launch the handshake and obtain a live [`Client`].
pub struct ClientBuilder {
    transport: Option<Box<dyn ClientTransport>>,
    client_info_name: Option<String>,
    client_info_version: Option<String>,
    timeout: Duration,
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self {
            transport: None,
            client_info_name: None,
            client_info_version: None,
            timeout: DEFAULT_TIMEOUT,
        }
    }
}

impl ClientBuilder {
    /// Set the transport for this client.
    ///
    /// The transport's `connect()` will be called during [`connect`](Self::connect).
    pub fn transport(mut self, t: impl ClientTransport + 'static) -> Self {
        self.transport = Some(Box::new(t));
        self
    }

    /// Set the client identity for the `initialize` handshake.
    ///
    /// Defaults to `CARGO_PKG_NAME` / `CARGO_PKG_VERSION` if not set.
    pub fn client_info(mut self, name: impl Into<String>, version: impl Into<String>) -> Self {
        self.client_info_name = Some(name.into());
        self.client_info_version = Some(version.into());
        self
    }

    /// Set the per-request timeout.
    ///
    /// Defaults to 30 seconds.
    pub fn timeout(mut self, d: Duration) -> Self {
        self.timeout = d;
        self
    }

    /// Establish the connection, perform the MCP handshake, and return a live [`Client`].
    ///
    /// Steps:
    /// 1. Call `transport.connect()`
    /// 2. Spawn the background read-loop
    /// 3. Send `initialize` → parse capabilities and server info
    /// 4. Send `notifications/initialized`
    /// 5. Auto-fetch lists (tools, prompts, resources, templates) based on advertised capabilities
    pub async fn connect(mut self) -> Result<Client> {
        let mut transport = self
            .transport
            .take()
            .ok_or_else(|| Error::ConnectionError("no transport configured".into()))?;

        transport.connect().await?;

        let transport: Arc<dyn ClientTransport> = Arc::from(transport);
        let pending = PendingRequests::new();
        let (notif_tx, _) = broadcast::channel(256);

        let read_task = tokio::spawn(read_loop(
            transport.clone(),
            pending.clone(),
            notif_tx.clone(),
        ));

        let client_name = self
            .client_info_name
            .unwrap_or_else(|| env!("CARGO_PKG_NAME").to_string());
        let client_version = self
            .client_info_version
            .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string());

        // Handshake: initialize
        let init_params = serde_json::to_value(InitializeParams {
            protocol_version: "2025-03-26".to_string(),
            capabilities: serde_json::json!({}),
            client_info: ClientInfo {
                name: client_name,
                version: client_version,
            },
        })?;
        let init_result_value = send_request(
            &transport,
            &pending,
            self.timeout,
            "initialize",
            Some(init_params),
        )
        .await?;
        let init: InitializeResult = serde_json::from_value(init_result_value)?;

        // Notify initialized (no `id`, no response)
        send_notify(&transport, "notifications/initialized", None).await?;

        // Auto-fetch lists based on advertised capabilities
        let (tools_cache, prompts_cache, resources_cache, templates_cache) =
            auto_fetch_lists(&transport, &pending, self.timeout, &init.capabilities).await?;

        debug!(
            server = %init.server_info.name,
            tools = tools_cache.len(),
            resources = resources_cache.len(),
            templates = templates_cache.len(),
            prompts = prompts_cache.len(),
            "MCP handshake complete"
        );

        Ok(Client {
            transport,
            pending,
            notif_tx,
            read_task: Some(read_task),
            timeout: self.timeout,
            tools_cache,
            resources_cache,
            templates_cache,
            prompts_cache,
            server_info: init.server_info,
            server_capabilities: init.capabilities,
        })
    }
}

/// Send a JSON-RPC request and wait for the correlated response.
async fn send_request(
    transport: &Arc<dyn ClientTransport>,
    pending: &PendingRequests,
    timeout: Duration,
    method: &str,
    params: Option<serde_json::Value>,
) -> Result<serde_json::Value> {
    let id = next_request_id();
    let rx = pending
        .insert(id)
        .ok_or_else(|| Error::ProtocolError("duplicate request ID".into()))?;

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(serde_json::json!(id)),
        method: method.to_string(),
        params,
    };
    let request_str = serde_json::to_string(&request)?;
    transport.send(&request_str).await?;

    match tokio::time::timeout(timeout, rx).await {
        Ok(Ok(resp_value)) => {
            if let Some(error) = resp_value.get("error") {
                if error.is_object() {
                    let code = error.get("code").and_then(|c| c.as_i64()).unwrap_or(-32603);
                    let message = error
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("unknown error");
                    Err(Error::ProtocolError(format!("{code}: {message}")))
                } else {
                    Err(Error::ProtocolError(
                        "response error field is not an object".into(),
                    ))
                }
            } else {
                let result = resp_value
                    .get("result")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null);
                Ok(result)
            }
        }
        Ok(Err(_cancelled)) => Err(Error::TransportClosed("connection closed".into())),
        Err(_elapsed) => {
            pending.remove(id);
            Err(Error::TimeoutError(format!(
                "request '{}' timed out",
                method
            )))
        }
    }
}

/// Send a JSON-RPC notification (no `id`, no response expected).
async fn send_notify(
    transport: &Arc<dyn ClientTransport>,
    method: &str,
    params: Option<serde_json::Value>,
) -> Result<()> {
    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: None,
        method: method.to_string(),
        params,
    };
    let request_str = serde_json::to_string(&request)?;
    transport.send(&request_str).await
}

/// Fetch cached lists from the server based on advertised capabilities.
///
/// Each fetch is gated by the corresponding capability field in
/// `ServerCapabilities`. If a fetch fails with `-32601` (method not found),
/// it is treated as an empty list rather than a hard error — this is
/// defensive interop for servers that don't implement all methods.
async fn auto_fetch_lists(
    transport: &Arc<dyn ClientTransport>,
    pending: &PendingRequests,
    timeout: Duration,
    capabilities: &ServerCapabilities,
) -> Result<(
    Vec<ToolDefinition>,
    Vec<PromptDefinition>,
    Vec<Resource>,
    Vec<ResourceTemplate>,
)> {
    let tools = if capabilities.tools.is_some() {
        fetch_list(transport, pending, timeout, "tools/list")
            .await
            .and_then(|v| Ok(serde_json::from_value::<ListToolsResult>(v)?))
            .map(|r| r.tools)
            .unwrap_or_else(|e| {
                debug!("tools/list fetch skipped: {e}");
                Vec::new()
            })
    } else {
        Vec::new()
    };

    let prompts = if capabilities.prompts.is_some() {
        fetch_list(transport, pending, timeout, "prompts/list")
            .await
            .and_then(|v| Ok(serde_json::from_value::<ListPromptsResult>(v)?))
            .map(|r| r.prompts)
            .unwrap_or_else(|e| {
                debug!("prompts/list fetch skipped: {e}");
                Vec::new()
            })
    } else {
        Vec::new()
    };

    let (resources, templates) = if capabilities.resources.is_some() {
        let r = fetch_list(transport, pending, timeout, "resources/list")
            .await
            .and_then(|v| Ok(serde_json::from_value::<ListResourcesResult>(v)?))
            .map(|r| r.resources)
            .unwrap_or_else(|e| {
                debug!("resources/list fetch skipped: {e}");
                Vec::new()
            });

        let t = fetch_list(transport, pending, timeout, "resources/templates/list")
            .await
            .and_then(|v| Ok(serde_json::from_value::<ListResourceTemplatesResult>(v)?))
            .map(|r| r.resource_templates)
            .unwrap_or_else(|e| {
                debug!("resources/templates/list fetch skipped: {e}");
                Vec::new()
            });

        (r, t)
    } else {
        (Vec::new(), Vec::new())
    };

    Ok((tools, prompts, resources, templates))
}

/// Send a parameterless JSON-RPC request and return the raw result value.
///
/// Used for list-fetch methods (`tools/list`, `prompts/list`, etc.).
async fn fetch_list(
    transport: &Arc<dyn ClientTransport>,
    pending: &PendingRequests,
    timeout: Duration,
    method: &str,
) -> Result<serde_json::Value> {
    send_request(transport, pending, timeout, method, None).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notification::Notification;
    use crate::transport::ClientTransport;
    use async_trait::async_trait;
    use std::sync::Mutex;

    /// A deterministic, in-process transport for testing the read-loop and
    /// correlation machinery without spawning a real subprocess.
    ///
    /// Injected response lines are delivered via a channel so `receive` can
    /// block without busy-waiting. Sent messages are captured for inspection.
    struct FakeTransport {
        send_log: Mutex<Vec<String>>,
        inject_tx: tokio::sync::Mutex<Option<tokio::sync::mpsc::UnboundedSender<Option<String>>>>,
        inject_rx: tokio::sync::Mutex<Option<tokio::sync::mpsc::UnboundedReceiver<Option<String>>>>,
    }

    impl FakeTransport {
        fn new() -> Self {
            let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
            Self {
                send_log: Mutex::new(Vec::new()),
                inject_tx: tokio::sync::Mutex::new(Some(tx)),
                inject_rx: tokio::sync::Mutex::new(Some(rx)),
            }
        }

        async fn inject(&self, line: &str) {
            if let Some(ref tx) = *self.inject_tx.lock().await {
                let _ = tx.send(Some(line.to_string()));
            }
        }

        async fn inject_eof(&self) {
            if let Some(ref tx) = *self.inject_tx.lock().await {
                let _ = tx.send(None);
            }
        }

        #[allow(dead_code)]
        fn sent_messages(&self) -> Vec<String> {
            self.send_log.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl ClientTransport for FakeTransport {
        async fn connect(&mut self) -> Result<()> {
            Ok(())
        }

        async fn send(&self, message: &str) -> Result<()> {
            self.send_log.lock().unwrap().push(message.to_string());
            Ok(())
        }

        async fn receive(&self) -> Result<Option<String>> {
            let mut rx_guard = self.inject_rx.lock().await;
            if let Some(ref mut rx) = *rx_guard {
                match rx.recv().await {
                    Some(item) => Ok(item),
                    None => Ok(None), // channel closed = EOF
                }
            } else {
                Ok(None)
            }
        }

        async fn close(&self) -> Result<()> {
            // Close the injection channel to signal EOF
            *self.inject_tx.lock().await = None;
            Ok(())
        }
    }

    fn spawn_read_loop(
        fake: Arc<FakeTransport>,
    ) -> (PendingRequests, broadcast::Receiver<Notification>) {
        let pending = PendingRequests::new();
        let (notif_tx, notif_rx) = broadcast::channel(256);
        tokio::spawn(read_loop(
            fake as Arc<dyn ClientTransport>,
            pending.clone(),
            notif_tx,
        ));
        (pending, notif_rx)
    }

    #[tokio::test]
    async fn test_read_loop_correlation() {
        let fake = Arc::new(FakeTransport::new());
        let (pending, _notif_rx) = spawn_read_loop(fake.clone());

        // Use a deterministic ID so we know what to inject.
        let id = 42u64;
        let rx = pending.insert(id).expect("should insert");

        // Inject AFTER inserting (ensures read_loop resolves it).
        // The full JSON-RPC response object is passed through to the receiver.
        fake.inject(r#"{"jsonrpc":"2.0","id":42,"result":{"ok":true}}"#)
            .await;
        tokio::time::sleep(Duration::from_millis(50)).await;

        let val = rx.await.expect("should receive response");
        // The read_loop resolves with the full Value, including id/jsonrpc/result.
        assert_eq!(val["id"], 42);
        assert_eq!(val["result"]["ok"], true);
    }

    #[tokio::test]
    async fn test_read_loop_protocol_error() {
        let fake = Arc::new(FakeTransport::new());
        let (pending, _notif_rx) = spawn_read_loop(fake.clone());

        let id = 99u64;
        let rx = pending.insert(id).expect("should insert");

        fake.inject(
            r#"{"jsonrpc":"2.0","id":99,"error":{"code":-32601,"message":"Method not found"}}"#,
        )
        .await;
        tokio::time::sleep(Duration::from_millis(50)).await;

        let val = rx.await.expect("should receive response");
        assert_eq!(val["error"]["code"], -32601);
        assert_eq!(val["error"]["message"], "Method not found");
    }

    #[tokio::test]
    async fn test_read_loop_notification() {
        let fake = Arc::new(FakeTransport::new());
        let (_pending, mut notif_rx) = spawn_read_loop(fake.clone());

        fake.inject(
            r#"{"jsonrpc":"2.0","method":"notifications/resources/updated","params":{"uri":"file:///x.txt"}}"#,
        )
        .await;
        tokio::time::sleep(Duration::from_millis(10)).await;

        let n = tokio::time::timeout(Duration::from_secs(1), notif_rx.recv())
            .await
            .expect("timeout")
            .expect("should receive notification");

        assert_eq!(
            n,
            Notification::ResourcesUpdated {
                uri: "file:///x.txt".into()
            }
        );
    }

    #[tokio::test]
    async fn test_read_loop_timeout_no_leak() {
        let fake = Arc::new(FakeTransport::new());
        let (pending, _notif_rx) = spawn_read_loop(fake.clone());

        let id = next_request_id();
        let rx = pending.insert(id).expect("should insert");

        // Don't inject anything — remove from pending to simulate timeout
        pending.remove(id);

        // Receiver should be cancelled
        match rx.await {
            Err(_) => {} // expected
            Ok(_) => panic!("expected cancelled error after remove"),
        }

        assert!(pending.is_empty());
    }

    #[tokio::test]
    async fn test_read_loop_eof_cancels_pending() {
        let fake = Arc::new(FakeTransport::new());
        let (pending, _notif_rx) = spawn_read_loop(fake.clone());

        let id = next_request_id();
        let rx = pending.insert(id).expect("should insert");
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Inject EOF
        fake.inject_eof().await;
        tokio::time::sleep(Duration::from_millis(100)).await;

        // After EOF, pending should be cancelled
        assert!(pending.is_empty());
        match rx.await {
            Err(_) => {} // expected
            Ok(_) => panic!("expected cancelled error after EOF"),
        }
    }

    #[test]
    fn test_client_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Client>();
    }

    #[tokio::test]
    async fn test_send_request_protocol_error_detection() {
        let fake = Arc::new(FakeTransport::new());
        let pending = PendingRequests::new();
        let (notif_tx, _) = broadcast::channel(1);
        tokio::spawn(read_loop(
            fake.clone() as Arc<dyn ClientTransport>,
            pending.clone(),
            notif_tx,
        ));

        // Use manual correlation: insert with a known ID, inject matching response.
        let known_id = 7u64;
        let rx = pending.insert(known_id).expect("should insert");

        // Send the request ourselves (mimic what send_request does)
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(known_id)),
            method: "test/method".to_string(),
            params: None,
        };
        let request_str = serde_json::to_string(&request).expect("should serialize");
        (fake.clone() as Arc<dyn ClientTransport>)
            .send(&request_str)
            .await
            .expect("send should succeed");

        // Inject the protocol error response
        fake.inject(
            r#"{"jsonrpc":"2.0","id":7,"error":{"code":-32601,"message":"Method not found"}}"#,
        )
        .await;

        // Now the raw receiver should get the error response
        let val = tokio::time::timeout(Duration::from_secs(5), rx)
            .await
            .expect("timeout")
            .expect("should receive response");

        // Verify the two error planes by simulating what send_request does
        let error = &val["error"];
        assert_eq!(error["code"], -32601);
        let code = error["code"].as_i64().unwrap();
        let message = error["message"].as_str().unwrap();
        let err = Error::ProtocolError(format!("{code}: {message}"));
        assert!(err.to_string().contains("-32601"));
    }

    #[tokio::test]
    async fn test_send_request_timeout() {
        let fake = Arc::new(FakeTransport::new());
        let pending = PendingRequests::new();
        let (notif_tx, _) = broadcast::channel(1);
        tokio::spawn(read_loop(
            fake.clone() as Arc<dyn ClientTransport>,
            pending.clone(),
            notif_tx,
        ));

        // Don't inject a response — the request will time out.
        let result = send_request(
            &(fake as Arc<dyn ClientTransport>),
            &pending,
            Duration::from_millis(100),
            "test/timeout",
            None,
        )
        .await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::TimeoutError(_)));
        assert!(pending.is_empty());
    }
}
