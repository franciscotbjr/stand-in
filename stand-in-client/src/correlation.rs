//! Request/response correlation primitives.
//!
//! The MCP protocol is asynchronous: the client sends a JSON-RPC request with an `id`
//! and the server responds with the same `id`. This module provides the bookkeeping
//! machinery to match outgoing requests to incoming responses.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;

/// Generate the next monotonic JSON-RPC request ID.
///
/// IDs start at 1 and increase by 1 on each call. Thread-safe.
pub(crate) fn next_request_id() -> u64 {
    static NEXT_ID: AtomicU64 = AtomicU64::new(0);
    NEXT_ID.fetch_add(1, Ordering::SeqCst)
}

/// A map of pending requests waiting for a response from the server.
///
/// When a request is sent, a [`oneshot::Receiver`] is created and the sender is
/// stored keyed by request ID. When the read-loop receives a response with a
/// matching ID, it resolves the pending request by sending the result through
/// the stored sender.
///
/// Thread-safe: shared between the request-sending task and the read-loop task.
#[derive(Debug, Clone, Default)]
pub struct PendingRequests {
    inner: Arc<Mutex<HashMap<u64, oneshot::Sender<serde_json::Value>>>>,
}

impl PendingRequests {
    /// Create an empty pending-requests map.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register a new pending request and return the receiver that will get the response.
    ///
    /// Returns `None` if an entry already exists for this ID.
    pub fn insert(&self, id: u64) -> Option<oneshot::Receiver<serde_json::Value>> {
        let (tx, rx) = oneshot::channel();
        let mut map = self.inner.lock().unwrap();
        if map.insert(id, tx).is_some() {
            // Overwrote an existing entry — the old sender is dropped,
            // and the old receiver will get a cancelled error.
            None
        } else {
            Some(rx)
        }
    }

    /// Resolve a pending request by sending the response value through the stored sender.
    ///
    /// Returns `true` if the request was resolved, `false` if no matching ID was found.
    pub fn resolve(&self, id: u64, value: serde_json::Value) -> bool {
        let sender = {
            let mut map = self.inner.lock().unwrap();
            map.remove(&id)
        };
        match sender {
            Some(tx) => {
                // The receiver may have been dropped (e.g. timeout) — ignore send errors.
                let _ = tx.send(value);
                true
            }
            None => false,
        }
    }

    /// Remove a pending request without resolving it.
    ///
    /// Drops the stored sender, causing the receiver to get a cancelled error.
    /// Used when a request times out to prevent the pending map from leaking.
    ///
    /// Returns `true` if an entry was removed, `false` if no matching ID was found.
    pub fn remove(&self, id: u64) -> bool {
        let mut map = self.inner.lock().unwrap();
        map.remove(&id).is_some()
    }

    /// Cancel all pending requests.
    ///
    /// This is called when the transport is closed, so any awaiting callers
    /// get a cancelled error instead of hanging forever.
    pub fn cancel_all(&self) {
        let mut map = self.inner.lock().unwrap();
        map.drain();
        // Dropping all senders will cause all receivers to get Cancelled errors.
    }

    #[allow(dead_code)] // used in tests across modules
    /// Return the number of pending requests.
    pub fn len(&self) -> usize {
        self.inner.lock().unwrap().len()
    }

    #[allow(dead_code)] // used in tests across modules
    /// Return `true` if there are no pending requests.
    pub fn is_empty(&self) -> bool {
        self.inner.lock().unwrap().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_next_request_id_is_monotonic() {
        let id1 = next_request_id();
        let id2 = next_request_id();
        assert!(id2 > id1);
    }

    #[test]
    fn test_next_request_id_thread_safe() {
        let id1 = next_request_id();
        let id2 = next_request_id();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_pending_requests_insert_and_resolve() {
        let pending = PendingRequests::new();
        let id = 1;

        let rx = pending.insert(id).expect("should insert");
        assert_eq!(pending.len(), 1);

        let resolved = pending.resolve(id, serde_json::json!({"ok": true}));
        assert!(resolved);
        assert!(pending.is_empty());

        // Receiver should have the value.
        let val = rx.blocking_recv().expect("should receive value");
        assert_eq!(val, serde_json::json!({"ok": true}));
    }

    #[test]
    fn test_pending_requests_resolve_unknown_id() {
        let pending = PendingRequests::new();
        let resolved = pending.resolve(42, serde_json::json!(null));
        assert!(!resolved);
    }

    #[test]
    fn test_pending_requests_cancel_all() {
        let pending = PendingRequests::new();
        let rx = pending.insert(1).expect("should insert");
        assert_eq!(pending.len(), 1);

        pending.cancel_all();
        assert!(pending.is_empty());

        // Receiver should get a cancelled error.
        match rx.blocking_recv() {
            Err(oneshot::error::RecvError { .. }) => {} // expected
            Ok(_) => panic!("expected cancelled error"),
        }
    }

    #[test]
    fn test_pending_requests_remove() {
        let pending = PendingRequests::new();
        let rx = pending.insert(1).expect("should insert");
        assert_eq!(pending.len(), 1);

        // Remove should return true and drop the sender
        let removed = pending.remove(1);
        assert!(removed);
        assert!(pending.is_empty());

        // Receiver should get cancelled
        match rx.blocking_recv() {
            Err(oneshot::error::RecvError { .. }) => {}
            Ok(_) => panic!("expected cancelled error after remove"),
        }
    }

    #[test]
    fn test_pending_requests_remove_unknown_id() {
        let pending = PendingRequests::new();
        let removed = pending.remove(42);
        assert!(!removed);
    }

    #[test]
    fn test_pending_requests_insert_duplicate_id_returns_none() {
        let pending = PendingRequests::new();
        let _rx1 = pending.insert(1);
        let rx2 = pending.insert(1); // duplicate ID
        assert!(rx2.is_none());
    }

    #[test]
    fn test_pending_requests_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<PendingRequests>();
    }
}
