//! Connection management — the read-loop that drives request/response
//! correlation and notification delivery over a shared transport.
//!
//! The read-loop is spawned as a background task by `Client::connect()`.
//! It is the single reader of the transport; all request senders share
//! the same transport via `Arc`.

use std::sync::Arc;

use tokio::sync::broadcast;
use tracing::{debug, info, warn};

use crate::correlation::PendingRequests;
use crate::notification::Notification;
use crate::transport::ClientTransport;

/// Run the read-loop on a shared transport.
///
/// This function is spawned as a background task. It:
/// - Reads JSON lines from the transport via `receive()`
/// - Routes responses (with `id`) to the matching `pending` entry
/// - Routes notifications (with `method`, no `id`) to the `notif_tx` broadcast
/// - On EOF (`Ok(None)`) or transport error, cancels all pending requests
///   and emits `Disconnected`
///
/// This function does not return — when the loop exits, the transport is done.
pub(crate) async fn read_loop(
    transport: Arc<dyn ClientTransport>,
    pending: PendingRequests,
    notif_tx: broadcast::Sender<Notification>,
) {
    loop {
        match transport.receive().await {
            Ok(Some(line)) => {
                let value: serde_json::Value = match serde_json::from_str(&line) {
                    Ok(v) => v,
                    Err(e) => {
                        warn!("read-loop: failed to parse JSON: {e}");
                        continue;
                    }
                };

                if let Some(id) = value.get("id").and_then(|v| v.as_u64()) {
                    if !pending.resolve(id, value) {
                        debug!("read-loop: unmatched response id={id}");
                    }
                } else if value.get("id").is_some() {
                    // id exists but is not a u64 (e.g. null from the server
                    // echoing notifications/initialized). Ignore.
                    debug!("read-loop: response with non-u64 id ignored");
                } else if value.get("method").is_some() {
                    if let Some(n) = Notification::from_value(&value) {
                        let _ = notif_tx.send(n);
                    } else {
                        debug!("read-loop: failed to parse notification");
                    }
                } else {
                    debug!("read-loop: frame without id or method — ignored");
                }
            }
            Ok(None) => {
                info!("read-loop: transport closed (EOF)");
                break;
            }
            Err(e) => {
                warn!("read-loop: transport error: {e}");
                break;
            }
        }
    }

    pending.cancel_all();
    let _ = notif_tx.send(Notification::disconnected());
}
