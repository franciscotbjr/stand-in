//! The async bridge: a dedicated tokio thread that owns the `stand-in-client`
//! SDK and talks to the gpui UI over two channels.
//!
//! ## Invariants (non-negotiable)
//!
//! - **The `Client` lives ONLY in the engine thread.** The UI never holds
//!   a `Client` — it communicates exclusively via `UiCommand`/`EngineEvent`.
//! - **Nothing from `tokio::*` runs on the gpui executor.** Timers on the UI
//!   side use `cx.background_executor().timer`, never `tokio::time::sleep`
//!   (the M38 panic class — build green, app dead).
//! - **`evt_rx.recv()` on the gpui executor is safe** because `mpsc` channels
//!   do not touch the tokio reactor — only `tokio::time`/`tokio::net` require one.

use std::time::{Duration, Instant};

use stand_in_client::prelude::{Client, Credential, HttpTransport, OAuthConfig, StdioTransport};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

use crate::app::events::{ConnConfig, EngineEvent, ServerSnapshot, Transport, UiCommand};

pub fn spawn_engine() -> (UnboundedSender<UiCommand>, UnboundedReceiver<EngineEvent>) {
    spawn_engine_with_opener(|url| {
        let _ = open::that(url);
    })
}

/// Like `spawn_engine` but accepts a custom URL opener (for testing).
pub fn spawn_engine_with_opener(
    open_url: impl Fn(&str) + Send + 'static,
) -> (UnboundedSender<UiCommand>, UnboundedReceiver<EngineEvent>) {
    let (cmd_tx, cmd_rx) = unbounded_channel::<UiCommand>();
    let (evt_tx, evt_rx) = unbounded_channel::<EngineEvent>();
    let evt_tx_owned = evt_tx;
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("failed to build tokio runtime for MCP engine");
        rt.block_on(engine_loop_with_opener(cmd_rx, evt_tx_owned, open_url));
    });
    (cmd_tx, evt_rx)
}

async fn engine_loop_with_opener(
    mut cmd_rx: UnboundedReceiver<UiCommand>,
    evt_tx: UnboundedSender<EngineEvent>,
    open_url: impl Fn(&str) + Send + 'static,
) {
    let mut client: Option<Client> = None;

    while let Some(cmd) = cmd_rx.recv().await {
        match cmd {
            UiCommand::Connect { config, credential } => {
                let _ = evt_tx.send(EngineEvent::Connecting);
                let t0 = Instant::now();
                match connect_to_server(&config, &credential).await {
                    Ok(c) => {
                        let latency_ms = t0.elapsed().as_millis() as u64;
                        let mut snap = build_snapshot(&c);
                        snap.latency_ms = latency_ms;
                        let _ = evt_tx.send(EngineEvent::Connected(Box::new(snap)));
                        // Spawn notification forwarding task on the tokio runtime
                        let notif_tx = evt_tx.clone();
                        let mut notif_rx = c.notifications();
                        tokio::spawn(async move {
                            while let Ok(n) = notif_rx.recv().await {
                                if matches!(
                                    &n,
                                    stand_in_client::prelude::Notification::Disconnected
                                ) {
                                    let _ = notif_tx.send(EngineEvent::Notification(n));
                                    break;
                                }
                                let _ = notif_tx.send(EngineEvent::Notification(n));
                            }
                        });
                        client = Some(c);
                    }
                    Err(e) => {
                        let _ = evt_tx.send(EngineEvent::ConnectionError(e.to_string()));
                    }
                }
            }
            UiCommand::Disconnect => {
                if let Some(c) = client.take() {
                    let _ = c.disconnect().await;
                }
                let _ = evt_tx.send(EngineEvent::Disconnected);
            }
            UiCommand::CallTool { name, arguments } => match &client {
                Some(c) => match c.call_tool(&name, arguments).await {
                    Ok(result) => {
                        let _ = evt_tx.send(EngineEvent::ToolResult(Box::new(result)));
                    }
                    Err(e) => {
                        let _ = evt_tx.send(EngineEvent::ToolError(e.to_string()));
                    }
                },
                None => {
                    let _ = evt_tx.send(EngineEvent::ToolError("not connected".into()));
                }
            },
            UiCommand::ReadResource { uri } => match &client {
                Some(c) => match c.read_resource(&uri).await {
                    Ok(result) => {
                        let _ = evt_tx.send(EngineEvent::ResourceResult(Box::new(result)));
                    }
                    Err(e) => {
                        let _ = evt_tx.send(EngineEvent::ResourceError(e.to_string()));
                    }
                },
                None => {
                    let _ = evt_tx.send(EngineEvent::ResourceError("not connected".into()));
                }
            },
            UiCommand::Subscribe { uri } => match &client {
                Some(c) => match c.subscribe(&uri).await {
                    Ok(()) => {
                        let _ = evt_tx.send(EngineEvent::Subscribed(uri));
                    }
                    Err(e) => {
                        let _ = evt_tx.send(EngineEvent::ResourceError(e.to_string()));
                    }
                },
                None => {
                    let _ = evt_tx.send(EngineEvent::ResourceError("not connected".into()));
                }
            },
            UiCommand::Unsubscribe { uri } => match &client {
                Some(c) => match c.unsubscribe(&uri).await {
                    Ok(()) => {
                        let _ = evt_tx.send(EngineEvent::Unsubscribed(uri));
                    }
                    Err(e) => {
                        let _ = evt_tx.send(EngineEvent::ResourceError(e.to_string()));
                    }
                },
                None => {
                    let _ = evt_tx.send(EngineEvent::ResourceError("not connected".into()));
                }
            },
            UiCommand::GetPrompt { name, arguments } => match &client {
                Some(c) => match c.get_prompt(&name, arguments).await {
                    Ok(result) => {
                        let _ = evt_tx.send(EngineEvent::PromptMessages(Box::new(result)));
                    }
                    Err(e) => {
                        let _ = evt_tx.send(EngineEvent::PromptError(e.to_string()));
                    }
                },
                None => {
                    let _ = evt_tx.send(EngineEvent::PromptError("not connected".into()));
                }
            },
            UiCommand::Authorize { config } => {
                let evt_tx_auth = evt_tx.clone();
                let url_opener = &open_url;
                run_authorize(*config, |url| url_opener(url), evt_tx_auth).await;
            }
            UiCommand::RefreshAuth {
                config,
                refresh_token,
            } => {
                let evt_tx_refresh = evt_tx.clone();
                run_refresh(*config, &refresh_token, evt_tx_refresh).await;
            }
        }
    }
}

async fn connect_to_server(
    config: &ConnConfig,
    credential: &Credential,
) -> Result<Client, stand_in_client::Error> {
    match config.transport {
        Transport::Stdio => {
            Client::builder()
                .transport(
                    StdioTransport::command(&config.command, &config.args).envs(config.env.clone()),
                )
                .client_info("mcp-explorer", env!("CARGO_PKG_VERSION"))
                .timeout(Duration::from_secs(15))
                .connect()
                .await
        }
        Transport::Http => {
            Client::builder()
                .transport(HttpTransport::new(&config.url).with_credential(credential.clone()))
                .client_info("mcp-explorer", env!("CARGO_PKG_VERSION"))
                .timeout(Duration::from_secs(15))
                .connect()
                .await
        }
    }
}

async fn run_authorize(
    config: OAuthConfig,
    open_url: impl FnOnce(&str),
    evt_tx: UnboundedSender<EngineEvent>,
) {
    use stand_in_client::prelude::OAuthFlow;
    match OAuthFlow::new(config).authorize(open_url).await {
        Ok(tokens) => {
            let _ = evt_tx.send(EngineEvent::Authorized(Box::new(tokens)));
        }
        Err(e) => {
            let _ = evt_tx.send(EngineEvent::AuthorizationError(e.to_string()));
        }
    }
}

async fn run_refresh(
    config: OAuthConfig,
    refresh_token: &str,
    evt_tx: UnboundedSender<EngineEvent>,
) {
    use stand_in_client::prelude::OAuthFlow;
    match OAuthFlow::new(config).refresh(refresh_token).await {
        Ok(tokens) => {
            let _ = evt_tx.send(EngineEvent::Authorized(Box::new(tokens)));
        }
        Err(e) => {
            let _ = evt_tx.send(EngineEvent::AuthorizationError(e.to_string()));
        }
    }
}

fn build_snapshot(client: &Client) -> ServerSnapshot {
    ServerSnapshot {
        server_info: client.server_info().clone(),
        capabilities: client.server_capabilities().clone(),
        tools: client.tools().to_vec(),
        resources: client.resources().to_vec(),
        templates: client.resource_templates().to_vec(),
        prompts: client.prompts().to_vec(),
        latency_ms: 0,
    }
}
