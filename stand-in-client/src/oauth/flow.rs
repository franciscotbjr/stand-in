//! OAuth 2.0 Authorization Code + PKCE flow executor.

use std::time::{Duration, SystemTime};

use reqwest::Client;
use serde::Deserialize;
use tracing::debug;

use crate::error::{Error, Result};

use super::config::OAuthConfig;
use super::loopback::{self, RedirectParams};
use super::pkce::{self, Pkce};
use super::tokens::OAuthTokens;

/// Internal deserialization target for the token endpoint response.
#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: Option<String>,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
    token_type: Option<String>,
    #[serde(rename = "error")]
    error_field: Option<String>,
}

/// OAuth 2.0 flow executor (Authorization Code + PKCE + refresh).
///
/// Created with an [`OAuthConfig`], the flow provides two operations:
///
/// - [`authorize`] — runs the full Authorization Code + PKCE flow with
///   loopback redirect capture. The `open_url` callback is UI-agnostic
///   (D2): the app provides the browser-opening logic.
/// - [`refresh`] — renews an access token using a refresh token.
///
/// Both methods communicate with the token endpoint via `POST` with
/// `application/x-www-form-urlencoded` body.
///
/// [`authorize`]: OAuthFlow::authorize
/// [`refresh`]: OAuthFlow::refresh
pub struct OAuthFlow {
    config: OAuthConfig,
    http: Client,
}

impl OAuthFlow {
    /// Create a new flow executor with the given configuration.
    pub fn new(config: OAuthConfig) -> Self {
        Self {
            config,
            http: Client::new(),
        }
    }

    /// Run the full Authorization Code + PKCE flow.
    ///
    /// 1. Generates PKCE (code_verifier + code_challenge S256) and an anti-CSRF `state`.
    /// 2. Builds the authorization URL.
    /// 3. Binds a loopback listener on `127.0.0.1:{loopback_port}` **before** calling `open_url`.
    /// 4. Calls `open_url(&auth_url)` — in production this opens a browser; in tests
    ///    it simulates the redirect by connecting to the loopback.
    /// 5. Captures the `code` and `state` from the redirect.
    /// 6. Validates that the returned `state` matches the generated one.
    /// 7. Exchanges the `code` for tokens at the token endpoint.
    /// 8. Returns [`OAuthTokens`].
    ///
    /// # Errors
    ///
    /// Returns `Err(Error::OAuthError(_))` on: loopback bind failure, timeout,
    /// state mismatch, authorization server error, token exchange failure, or
    /// missing authorization code.
    pub async fn authorize<F>(&self, open_url: F) -> Result<OAuthTokens>
    where
        F: FnOnce(&str),
    {
        let pkce = Pkce::generate();
        let state = pkce::generate_state();
        let auth_url = self.build_authorization_url(&pkce, &state);

        debug!(%auth_url, "starting OAuth authorization code flow");

        // Bind listener BEFORE calling open_url so the loopback is ready.
        let port = self.config.loopback_port;
        let listener_addr: std::net::SocketAddr = ([127, 0, 0, 1], port).into();

        let listener = tokio::net::TcpListener::bind(listener_addr)
            .await
            .map_err(|e| {
                Error::OAuthError(format!(
                    "failed to bind loopback listener on port {port}: {e}"
                ))
            })?;

        let capture_task = {
            let (tx, rx) = tokio::sync::oneshot::channel();
            tokio::spawn(async move {
                let result = loopback::accept_one(listener).await;
                let _ = tx.send(result);
            });
            rx
        };

        // Call the UI-agnostic opener.
        open_url(&auth_url);

        // Await the captured redirect parameters with a 120-second timeout.
        let params = tokio::time::timeout(Duration::from_secs(120), capture_task)
            .await
            .map_err(|_| Error::OAuthError("authorization timed out after 120s".into()))?
            .map_err(|e| Error::OAuthError(format!("loopback task panicked: {e}")))??;

        self.validate_and_exchange(params, &state, &pkce).await
    }

    /// Renew an access token using a refresh token.
    ///
    /// Sends `grant_type=refresh_token` to the token endpoint. If the response
    /// omits a `refresh_token`, the existing one is preserved (common behaviour
    /// in authorization servers).
    pub async fn refresh(&self, refresh_token: &str) -> Result<OAuthTokens> {
        debug!("refreshing OAuth access token");

        let params: Vec<(&str, &str)> = vec![
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", &self.config.client_id),
        ];

        let body = build_form_body(&params);

        let resp = self
            .http
            .post(&self.config.token_url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .map_err(|e| Error::OAuthError(format!("token endpoint request failed: {e}")))?;

        let status = resp.status();
        let body = resp
            .text()
            .await
            .map_err(|e| Error::OAuthError(format!("failed to read token response: {e}")))?;

        if !status.is_success() {
            return Err(Error::OAuthError(format!(
                "token endpoint returned {status}: {body}"
            )));
        }

        let tr: TokenResponse = serde_json::from_str(&body)
            .map_err(|e| Error::OAuthError(format!("invalid token response JSON: {e}")))?;

        self.build_tokens(tr, Some(refresh_token.to_string()))
    }

    // ── private helpers ──────────────────────────────────────────────────

    async fn validate_and_exchange(
        &self,
        params: RedirectParams,
        state: &str,
        pkce: &Pkce,
    ) -> Result<OAuthTokens> {
        // Validate state (anti-CSRF).
        if params.state.as_deref() != Some(state) {
            return Err(Error::OAuthError(format!(
                "state mismatch: expected '{state}', got '{:?}'",
                params.state
            )));
        }

        // Check for authorization server error in the redirect.
        if let Some(error) = params.error {
            return Err(Error::OAuthError(format!(
                "authorization server returned error in redirect: {error}"
            )));
        }

        let code = params.code.ok_or_else(|| {
            Error::OAuthError("no authorization code received in redirect".into())
        })?;

        self.exchange_code(&code, &pkce.verifier).await
    }

    async fn exchange_code(&self, code: &str, code_verifier: &str) -> Result<OAuthTokens> {
        let redirect_uri = self.config.redirect_uri();
        let params: Vec<(&str, &str)> = vec![
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", &redirect_uri),
            ("client_id", &self.config.client_id),
            ("code_verifier", code_verifier),
        ];

        let body = build_form_body(&params);

        let resp = self
            .http
            .post(&self.config.token_url)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .map_err(|e| Error::OAuthError(format!("token endpoint request failed: {e}")))?;

        let status = resp.status();
        let body = resp
            .text()
            .await
            .map_err(|e| Error::OAuthError(format!("failed to read token response: {e}")))?;

        if !status.is_success() {
            return Err(Error::OAuthError(format!(
                "token endpoint returned {status}: {body}"
            )));
        }

        let tr: TokenResponse = serde_json::from_str(&body)
            .map_err(|e| Error::OAuthError(format!("invalid token response JSON: {e}")))?;

        if let Some(error) = tr.error_field {
            return Err(Error::OAuthError(format!(
                "token endpoint returned error: {error}"
            )));
        }

        self.build_tokens(tr, None)
    }

    fn build_tokens(
        &self,
        tr: TokenResponse,
        fallback_refresh: Option<String>,
    ) -> Result<OAuthTokens> {
        let access_token = tr
            .access_token
            .ok_or_else(|| Error::OAuthError("token response missing access_token".into()))?;

        let expires_at = tr.expires_in.map(|secs| {
            SystemTime::now()
                .checked_add(Duration::from_secs(secs))
                .unwrap_or(SystemTime::now() + Duration::from_secs(secs))
        });

        Ok(OAuthTokens {
            access_token,
            refresh_token: tr.refresh_token.or(fallback_refresh),
            expires_at,
            token_type: tr.token_type.unwrap_or_else(|| "Bearer".into()),
        })
    }

    fn build_authorization_url(&self, pkce: &Pkce, state: &str) -> String {
        let scope = self.config.scopes.join(" ");
        let scope_encoded = url_encode(&scope);
        let redirect_uri_encoded = url_encode(&self.config.redirect_uri());

        format!(
            "{}?response_type=code&client_id={}&redirect_uri={redirect_uri_encoded}&scope={scope_encoded}&state={state}&code_challenge={}&code_challenge_method=S256",
            self.config.authorization_url, self.config.client_id, pkce.challenge,
        )
    }
}

/// Minimal URL encoding for query parameter values.
///
/// Encodes characters that are not unreserved per RFC 3986.
fn url_encode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                result.push(byte as char);
            }
            b' ' => result.push_str("%20"),
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}

/// Build an `application/x-www-form-urlencoded` body from key-value pairs.
fn build_form_body(params: &[(&str, &str)]) -> String {
    params
        .iter()
        .map(|(k, v)| format!("{}={}", url_encode(k), url_encode(v)))
        .collect::<Vec<_>>()
        .join("&")
}
