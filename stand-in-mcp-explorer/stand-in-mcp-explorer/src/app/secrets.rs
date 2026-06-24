//! OS keychain facade for persisting authorization secrets (D3).
//!
//! Secrets (passwords, bearer tokens, OAuth tokens) are stored in the OS
//! keychain via the `keyring` crate — **never** in `servers.json`.
//! Non-secret config (method, Client ID, URLs) lives in `ServerEntry.auth`.
//!
//! ## Tolerance (D3)
//!
//! Keychain backend may be absent (CI, headless Linux). Both `load` and `save`
//! degrade gracefully — they log a warning and return `Default`/no-op; they
//! **never** panic or propagate the error to the caller.
//!
//! ## Storage
//!
//! A single JSON blob per server per http-url, stored as the keychain password:
//! - Service: `"mcp-explorer"`
//! - Account: the HTTP URL (`config.url`)
//! - Password: JSON-serialized `AuthSecrets`

use crate::bars::sidebar::auth_state::AuthSecrets;

const SERVICE: &str = "mcp-explorer";

/// Load secrets for a server identified by its HTTP URL.
/// Returns `Default` (empty) if no secrets exist or the backend is unavailable.
pub fn load_secrets(account: &str) -> AuthSecrets {
    match keyring::Entry::new(SERVICE, account) {
        Ok(entry) => match entry.get_password() {
            Ok(json) => match serde_json::from_str::<AuthSecrets>(&json) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("WARN: failed to parse keychain secrets for {account}: {e}");
                    AuthSecrets::default()
                }
            },
            Err(e) => {
                if !matches!(e, keyring::Error::NoEntry) {
                    eprintln!("WARN: failed to read keychain secrets for {account}: {e}");
                }
                AuthSecrets::default()
            }
        },
        Err(e) => {
            eprintln!("WARN: keychain backend unavailable for {account}: {e}");
            AuthSecrets::default()
        }
    }
}

/// Save secrets for a server identified by its HTTP URL.
/// Silently no-ops if the keychain backend is unavailable.
pub fn save_secrets(account: &str, secrets: &AuthSecrets) {
    match serde_json::to_string(secrets) {
        Ok(json) => match keyring::Entry::new(SERVICE, account) {
            Ok(entry) => {
                if let Err(e) = entry.set_password(&json) {
                    eprintln!("WARN: failed to save keychain secrets for {account}: {e}");
                }
            }
            Err(e) => {
                eprintln!("WARN: keychain backend unavailable for {account}: {e}");
            }
        },
        Err(e) => {
            eprintln!("WARN: failed to serialize secrets for {account}: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_secrets_json_round_trip() {
        let original = AuthSecrets {
            basic_password: "p4ss".into(),
            bearer_token: "b3ar3r".into(),
            oauth_refresh_token: Some("r3fr3sh".into()),
            oauth_access_token: Some("4cc3ss".into()),
            oauth_expires_at_unix: Some(1719000000),
        };
        let json = serde_json::to_string(&original).expect("serialize");
        let back: AuthSecrets = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.basic_password, "p4ss");
        assert_eq!(back.bearer_token, "b3ar3r");
        assert_eq!(back.oauth_refresh_token.as_deref(), Some("r3fr3sh"));
        assert_eq!(back.oauth_access_token.as_deref(), Some("4cc3ss"));
        assert_eq!(back.oauth_expires_at_unix, Some(1719000000));
    }

    #[test]
    fn auth_secrets_default_is_empty() {
        let s = AuthSecrets::default();
        assert!(s.basic_password.is_empty());
        assert!(s.bearer_token.is_empty());
        assert!(s.oauth_refresh_token.is_none());
        assert!(s.oauth_access_token.is_none());
        assert!(s.oauth_expires_at_unix.is_none());
    }

    #[test]
    fn load_secrets_impossible_account_returns_default() {
        // A deliberately malformed/invalid account name that keyring can't
        // resolve — returns Default without panicking.
        let s = load_secrets("");
        assert!(s.basic_password.is_empty());
        assert!(s.bearer_token.is_empty());
    }
}
