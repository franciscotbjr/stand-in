//! Thread-safe session store for managing MCP client sessions.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

use super::session::Session;

/// Thread-safe store for active MCP sessions.
///
/// Manages session lifecycle: creation, lookup, validation, and removal.
/// Shared across all axum handlers via `Arc`.
#[derive(Debug, Clone)]
pub struct SessionStore {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
}

impl SessionStore {
    /// Create an empty session store.
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new session and return its ID.
    pub async fn create(&self) -> String {
        let id = Uuid::new_v4().to_string();
        let session = Session::new(id.clone());
        self.sessions.write().await.insert(id.clone(), session);
        info!(session_id = %id, "Session stored");
        id
    }

    /// Check whether a session with the given ID exists.
    pub async fn validate(&self, id: &str) -> bool {
        let exists = self.sessions.read().await.contains_key(id);
        debug!(session_id = %id, valid = exists, "Session validated");
        exists
    }

    /// Remove a session by ID. Returns `true` if the session existed.
    pub async fn remove(&self, id: &str) -> bool {
        let removed = self.sessions.write().await.remove(id).is_some();
        if removed {
            info!(session_id = %id, "Session removed");
        } else {
            debug!(session_id = %id, "Session not found on remove");
        }
        removed
    }

    /// Run a closure with a read reference to the session, if it exists.
    pub async fn with_session<F, R>(&self, id: &str, f: F) -> Option<R>
    where
        F: FnOnce(&Session) -> R,
    {
        let sessions = self.sessions.read().await;
        sessions.get(id).map(f)
    }
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_session() {
        let store = SessionStore::new();
        let id = store.create().await;
        assert!(!id.is_empty());
        assert!(store.validate(&id).await);
    }

    #[tokio::test]
    async fn test_validate_unknown_session() {
        let store = SessionStore::new();
        assert!(!store.validate("nonexistent").await);
    }

    #[tokio::test]
    async fn test_remove_session() {
        let store = SessionStore::new();
        let id = store.create().await;
        assert!(store.remove(&id).await);
        assert!(!store.validate(&id).await);
    }

    #[tokio::test]
    async fn test_remove_unknown_session() {
        let store = SessionStore::new();
        assert!(!store.remove("nonexistent").await);
    }

    #[tokio::test]
    async fn test_with_session() {
        let store = SessionStore::new();
        let id = store.create().await;
        let session_id = store.with_session(&id, |s| s.id().to_string()).await;
        assert_eq!(session_id, Some(id));
    }

    #[tokio::test]
    async fn test_with_session_unknown() {
        let store = SessionStore::new();
        let result = store
            .with_session("nonexistent", |s| s.id().to_string())
            .await;
        assert!(result.is_none());
    }
}
