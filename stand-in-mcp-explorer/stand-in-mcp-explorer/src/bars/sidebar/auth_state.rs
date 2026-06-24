//! Authorization state model — pure enum + draft struct with InputState entities,
//! persistence types, OAuth runtime status, and credential derivation helpers.
//!
//! One file per sidebar concept (same pattern as `sidebar_state.rs`).
//! `AuthMethod` is pure + unit-testable; `AuthDraft` holds per-method
//! input entities. The caller (`StudioApp`) creates the `InputState` entities
//! with its own context.
//!
//! ## Redirect URI (D7)
//!
//! Loopback port 8765 mirrors the default of `OAuthConfig` in `stand-in-client`
//! (M2). In M3 this is display-only; M4 wires it to the real OAuth flow.
//!
//! ## Persistence (D3)
//!
//! Non-secret config (`AuthConfig`) lives in `servers.json` on the `ServerEntry`.
//! Secrets (`AuthSecrets`) live in the OS keychain via `keyring` (`secrets.rs`).
//! Neither `AuthStatus` nor `AuthDraft.oauth_tokens` are persisted directly.

use gpui::Entity;
use gpui_component::input::InputState;
use serde::{Deserialize, Serialize};
use stand_in_client::prelude::{Credential, OAuthTokens};
use std::time::SystemTime;

/// Fixed loopback port for the OAuth redirect URI (D7).
/// Mirrors the default `OAuthConfig::default_port` in `stand-in-client`.
pub const LOOPBACK_PORT: u16 = 8765;

/// The fixed, read-only redirect URI shown in the OAuth panel (D7).
pub fn redirect_uri() -> String {
    format!("http://127.0.0.1:{LOOPBACK_PORT}/callback")
}

// ---------------------------------------------------------------------------
// AuthMethod
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AuthMethod {
    #[default]
    NoAuth,
    Basic,
    Bearer,
    OAuth,
}

impl AuthMethod {
    pub const ALL: [AuthMethod; 4] = [Self::NoAuth, Self::Basic, Self::Bearer, Self::OAuth];

    pub fn selected_ix(self) -> usize {
        match self {
            Self::NoAuth => 0,
            Self::Basic => 1,
            Self::Bearer => 2,
            Self::OAuth => 3,
        }
    }

    pub fn from_ix(ix: usize) -> Self {
        match ix {
            1 => Self::Basic,
            2 => Self::Bearer,
            3 => Self::OAuth,
            _ => Self::NoAuth,
        }
    }

    /// i18n key for the short label used in the trigger and Select.
    pub fn label_key(self) -> &'static str {
        match self {
            Self::NoAuth => "auth.none",
            Self::Basic => "auth.basic",
            Self::Bearer => "auth.bearer",
            Self::OAuth => "auth.oauth",
        }
    }
}

// ---------------------------------------------------------------------------
// AuthConfig (non-secret, persisted in servers.json — D3)
// ---------------------------------------------------------------------------

/// Configuration NÃO-secreta de autorização, persistida na `ServerEntry`
/// dentro de `servers.json`. Nunca contém senhas, tokens nem segredos.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthConfig {
    pub method: AuthMethod,
    pub basic_username: String,
    pub oauth_client_id: String,
    pub oauth_auth_url: String,
    pub oauth_token_url: String,
    pub oauth_scopes: String,
}

// ---------------------------------------------------------------------------
// AuthSecrets (segredos — NUNCA no servers.json; vivem no keychain / D3)
// ---------------------------------------------------------------------------

/// Segredos de autorização — **nunca** serializados em `servers.json`.
/// Persistidos no keychain do SO via `keyring` (`secrets.rs`).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuthSecrets {
    pub basic_password: String,
    pub bearer_token: String,
    pub oauth_refresh_token: Option<String>,
    pub oauth_access_token: Option<String>,
    pub oauth_expires_at_unix: Option<u64>,
}

// ---------------------------------------------------------------------------
// AuthStatus (runtime — NÃO persistido)
// ---------------------------------------------------------------------------

/// Status runtime do fluxo OAuth (não é persistido).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum AuthStatus {
    #[default]
    Idle,
    Authorizing,
    Authorized,
    Failed,
}

// ---------------------------------------------------------------------------
// credential_from — helper puro
// ---------------------------------------------------------------------------

/// Constrói o `Credential` a partir do método de auth e dos valores textuais
/// já lidos das `InputState` entities pelo caller. É puro — não depende de
/// gpui context. OAuth sem access token vigente → `Credential::default()` (NoAuth).
pub fn credential_from(
    method: AuthMethod,
    basic_user: &str,
    basic_pw: &str,
    bearer: &str,
    oauth_access: Option<&str>,
) -> Credential {
    match method {
        AuthMethod::NoAuth => Credential::default(),
        AuthMethod::Basic => {
            if basic_user.is_empty() && basic_pw.is_empty() {
                Credential::default()
            } else {
                Credential::basic(basic_user.to_string(), basic_pw.to_string())
            }
        }
        AuthMethod::Bearer => {
            if bearer.is_empty() {
                Credential::default()
            } else {
                Credential::bearer(bearer.to_string())
            }
        }
        AuthMethod::OAuth => match oauth_access {
            Some(tok) if !tok.is_empty() => Credential::bearer(tok.to_string()),
            _ => Credential::default(),
        },
    }
}

/// Formata a validade do token OAuth para exibição no painel.
/// "expira em X min" / "sem expiração" — usa `OAuthTokens::is_expired()`.
pub fn format_oauth_expiry(tokens: &Option<OAuthTokens>, lang: crate::app::i18n::Lang) -> String {
    let Some(t) = tokens else {
        return String::new();
    };
    if t.is_expired() {
        return crate::app::i18n::tr("auth.expired", lang).to_string();
    }
    let Some(exp) = t.expires_at else {
        return crate::app::i18n::tr("auth.noExpiry", lang).to_string();
    };
    let now = SystemTime::now();
    let remaining = exp.duration_since(now).unwrap_or_default();
    let secs = remaining.as_secs();
    let prefix = crate::app::i18n::tr("auth.expiresIn", lang);
    if secs < 60 {
        format!("{}{}s", prefix, secs)
    } else if secs < 3600 {
        format!("{}{} min", prefix, secs / 60)
    } else if secs < 86400 {
        format!("{}{}h {}min", prefix, secs / 3600, (secs % 3600) / 60)
    } else {
        format!("{}{}d", prefix, secs / 86400)
    }
}

// ---------------------------------------------------------------------------
// AuthDraft
// ---------------------------------------------------------------------------

/// In-memory authorization form state (per-connection-in-edit).
///
/// The caller (`StudioApp`) creates the `InputState` entities with its own
/// window + cx and stores them here — mirrors `SidebarState`.
/// Runtime OAuth fields are populated by the engine bridge after
/// `Authorize` / `RefreshAuth` completes (M4).
pub struct AuthDraft {
    pub method: AuthMethod,
    pub basic_username: Entity<InputState>,
    pub basic_password: Entity<InputState>,
    pub bearer_token: Entity<InputState>,
    pub oauth_client_id: Entity<InputState>,
    pub oauth_auth_url: Entity<InputState>,
    pub oauth_token_url: Entity<InputState>,
    pub oauth_scopes: Entity<InputState>,
    /// Runtime OAuth flow status — set by the UI (Idle→Authorizing) and
    /// updated by the engine bridge (Authorized / Failed).
    pub oauth_status: AuthStatus,
    /// Tokens recebidos do fluxo OAuth (None até Authorized).
    pub oauth_tokens: Option<OAuthTokens>,
    /// Mensagem do último erro OAuth (apenas quando `oauth_status == Failed`).
    pub oauth_error: Option<String>,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loopback_port_matches_oauth_config_default() {
        assert_eq!(LOOPBACK_PORT, 8765);
    }

    #[test]
    fn test_redirect_uri_format() {
        assert_eq!(redirect_uri(), "http://127.0.0.1:8765/callback");
    }

    #[test]
    fn test_auth_method_default_is_no_auth() {
        assert_eq!(AuthMethod::default(), AuthMethod::NoAuth);
    }

    #[test]
    fn test_selected_ix_round_trip() {
        for m in AuthMethod::ALL {
            assert_eq!(AuthMethod::from_ix(m.selected_ix()), m);
        }
    }

    #[test]
    fn test_from_ix_oob_is_no_auth() {
        assert_eq!(AuthMethod::from_ix(99), AuthMethod::NoAuth);
        assert_eq!(AuthMethod::from_ix(4), AuthMethod::NoAuth);
    }

    #[test]
    fn test_label_keys() {
        assert_eq!(AuthMethod::NoAuth.label_key(), "auth.none");
        assert_eq!(AuthMethod::Basic.label_key(), "auth.basic");
        assert_eq!(AuthMethod::Bearer.label_key(), "auth.bearer");
        assert_eq!(AuthMethod::OAuth.label_key(), "auth.oauth");
    }

    // -----------------------------------------------------------------------
    // AuthConfig / AuthMethod serde round-trip (M4)
    // -----------------------------------------------------------------------

    #[test]
    fn test_auth_method_serde_round_trip() {
        for method in AuthMethod::ALL {
            let json = serde_json::to_string(&method).unwrap();
            let back: AuthMethod = serde_json::from_str(&json).unwrap();
            assert_eq!(back, method);
        }
    }

    #[test]
    fn test_auth_method_serde_known_values() {
        let no_auth: AuthMethod = serde_json::from_str(r#""NoAuth""#).unwrap();
        assert_eq!(no_auth, AuthMethod::NoAuth);
        let basic: AuthMethod = serde_json::from_str(r#""Basic""#).unwrap();
        assert_eq!(basic, AuthMethod::Basic);
        let bearer: AuthMethod = serde_json::from_str(r#""Bearer""#).unwrap();
        assert_eq!(bearer, AuthMethod::Bearer);
        let oauth: AuthMethod = serde_json::from_str(r#""OAuth""#).unwrap();
        assert_eq!(oauth, AuthMethod::OAuth);
    }

    #[test]
    fn test_auth_config_serde_round_trip() {
        let cfg = AuthConfig {
            method: AuthMethod::Bearer,
            basic_username: String::new(),
            oauth_client_id: String::new(),
            oauth_auth_url: String::new(),
            oauth_token_url: String::new(),
            oauth_scopes: String::new(),
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let back: AuthConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back, cfg);
    }

    #[test]
    fn test_auth_config_default() {
        let cfg = AuthConfig::default();
        assert_eq!(cfg.method, AuthMethod::NoAuth);
    }

    // -----------------------------------------------------------------------
    // credential_from (M4)
    // -----------------------------------------------------------------------

    #[test]
    fn test_credential_from_no_auth() {
        let cred = credential_from(AuthMethod::NoAuth, "", "", "", None);
        assert!(cred.authorization_header().is_none());
    }

    #[test]
    fn test_credential_from_basic() {
        let cred = credential_from(AuthMethod::Basic, "user", "pass", "", None);
        let hdr = cred.authorization_header().unwrap();
        assert!(hdr.starts_with("Basic "));
    }

    #[test]
    fn test_credential_from_basic_empty_returns_no_auth() {
        let cred = credential_from(AuthMethod::Basic, "", "", "", None);
        assert!(cred.authorization_header().is_none());
    }

    #[test]
    fn test_credential_from_bearer() {
        let cred = credential_from(AuthMethod::Bearer, "", "", "tok", None);
        let hdr = cred.authorization_header().unwrap();
        assert_eq!(hdr, "Bearer tok");
    }

    #[test]
    fn test_credential_from_bearer_empty_returns_no_auth() {
        let cred = credential_from(AuthMethod::Bearer, "", "", "", None);
        assert!(cred.authorization_header().is_none());
    }

    #[test]
    fn test_credential_from_oauth_with_token() {
        let cred = credential_from(AuthMethod::OAuth, "", "", "", Some("access123"));
        let hdr = cred.authorization_header().unwrap();
        assert_eq!(hdr, "Bearer access123");
    }

    #[test]
    fn test_credential_from_oauth_without_token_returns_no_auth() {
        let cred = credential_from(AuthMethod::OAuth, "", "", "", None);
        assert!(cred.authorization_header().is_none());
    }

    #[test]
    fn test_credential_from_oauth_empty_token_returns_no_auth() {
        let cred = credential_from(AuthMethod::OAuth, "", "", "", Some(""));
        assert!(cred.authorization_header().is_none());
    }
}
