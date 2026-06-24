//! OAuth client configuration — endpoints, scopes, and loopback settings.

/// OAuth 2.0 client configuration.
///
/// The `client_id` is a **public** OAuth identifier (not a secret) and is
/// therefore visible in `Debug`. All secrets (tokens, verifier) are held in
/// [`OAuthTokens`](super::OAuthTokens) and `Pkce` with
/// redacted `Debug`.
#[derive(Clone, Debug)]
pub struct OAuthConfig {
    /// OAuth client identifier (public, not a secret).
    pub client_id: String,
    /// Authorization endpoint URL (user-provided per D4).
    pub authorization_url: String,
    /// Token endpoint URL (user-provided per D4).
    pub token_url: String,
    /// Space-separated list of requested scopes.
    pub scopes: Vec<String>,
    /// Fixed loopback port for the redirect URI (default 8765, D7).
    pub loopback_port: u16,
}

impl OAuthConfig {
    /// Read-only redirect URI derived from the loopback port (D7).
    ///
    /// Returns `http://127.0.0.1:{port}/callback`.
    pub fn redirect_uri(&self) -> String {
        format!("http://127.0.0.1:{}/callback", self.loopback_port)
    }

    /// Create a new `OAuthConfig` with default loopback port 8765.
    pub fn new(
        client_id: impl Into<String>,
        authorization_url: impl Into<String>,
        token_url: impl Into<String>,
        scopes: Vec<String>,
    ) -> Self {
        Self {
            client_id: client_id.into(),
            authorization_url: authorization_url.into(),
            token_url: token_url.into(),
            scopes,
            loopback_port: 8765,
        }
    }

    /// Override the default loopback port.
    pub fn with_loopback_port(mut self, port: u16) -> Self {
        self.loopback_port = port;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redirect_uri_default_port() {
        let config = OAuthConfig::new(
            "id",
            "https://example.com/auth",
            "https://example.com/token",
            vec![],
        );
        assert_eq!(config.redirect_uri(), "http://127.0.0.1:8765/callback");
    }

    #[test]
    fn test_redirect_uri_custom_port() {
        let config = OAuthConfig::new(
            "id",
            "https://example.com/auth",
            "https://example.com/token",
            vec![],
        )
        .with_loopback_port(9999);
        assert_eq!(config.redirect_uri(), "http://127.0.0.1:9999/callback");
    }

    #[test]
    fn test_debug_includes_client_id() {
        let config = OAuthConfig::new(
            "my-public-id",
            "https://example.com/auth",
            "https://example.com/token",
            vec!["read".into()],
        );
        let debug = format!("{:?}", config);
        assert!(debug.contains("my-public-id"));
        assert!(!debug.contains("***"));
    }
}
