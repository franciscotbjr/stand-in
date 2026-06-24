//! Integration tests for the OAuth 2.0 Authorization Code + PKCE flow.
//!
//! Spawns a fake authorization server (token endpoint) via axum and drives
//! the full OAuth flow without a real browser. The `open_url` callback
//! simulates the browser redirect by connecting to the loopback listener
//! with a raw TCP stream.

#![cfg(feature = "oauth")]

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};

use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::{Router, routing};
use serde::Deserialize;
use serde_json::json;
use stand_in_client::error::Error;
use stand_in_client::oauth::{OAuthConfig, OAuthFlow};

// ---------------------------------------------------------------------------
// Fake authorization server state
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
struct FakeServer {
    /// When `true`, the token endpoint returns an error response.
    error_mode: Arc<AtomicBool>,
    /// When `true`, refresh returns a *new* refresh token (vs omitting it).
    rotate_refresh: Arc<AtomicBool>,
    /// Track whether the code_verifier was present in the exchange request.
    saw_code_verifier: Arc<AtomicBool>,
    /// Track whether the refresh_token was present in the refresh request.
    saw_refresh_token: Arc<AtomicBool>,
}

#[derive(Debug, Deserialize)]
struct TokenForm {
    grant_type: String,
    #[allow(dead_code)]
    code: Option<String>,
    #[allow(dead_code)]
    redirect_uri: Option<String>,
    #[allow(dead_code)]
    client_id: Option<String>,
    #[allow(dead_code)]
    code_verifier: Option<String>,
    #[allow(dead_code)]
    refresh_token: Option<String>,
}

async fn handle_token(
    State(state): State<FakeServer>,
    axum::extract::Form(form): axum::extract::Form<TokenForm>,
) -> Response {
    if state.error_mode.load(Ordering::SeqCst) {
        return axum::response::Json(json!({"error": "invalid_grant"})).into_response();
    }

    match form.grant_type.as_str() {
        "authorization_code" => {
            if form.code_verifier.is_some() {
                state.saw_code_verifier.store(true, Ordering::SeqCst);
            }
            axum::response::Json(json!({
                "access_token": "test-access-token",
                "refresh_token": "test-refresh-token",
                "expires_in": 3600,
                "token_type": "Bearer",
            }))
            .into_response()
        }
        "refresh_token" => {
            if form.refresh_token.is_some() {
                state.saw_refresh_token.store(true, Ordering::SeqCst);
            }
            let mut body = json!({
                "access_token": "new-access-token",
                "expires_in": 7200,
                "token_type": "Bearer",
            });
            if state.rotate_refresh.load(Ordering::SeqCst) {
                body["refresh_token"] = json!("new-refresh-token");
            }
            axum::response::Json(body).into_response()
        }
        _ => axum::response::Json(json!({"error": "unsupported_grant_type"})).into_response(),
    }
}

fn free_addr() -> SocketAddr {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap()
}

async fn spawn_fake_server() -> (String, FakeServer) {
    let addr = free_addr();
    let state = FakeServer::default();

    let app = Router::new()
        .route("/token", routing::post(handle_token))
        .with_state(state.clone());

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let url = format!("http://{addr}/token");
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (url, state)
}

/// Return a unique loopback port for each test to avoid port collisions
/// when tests run in parallel with `--test-threads > 1`.
fn test_port() -> u16 {
    static NEXT: AtomicU16 = AtomicU16::new(19876);
    let port = NEXT.fetch_add(1, Ordering::SeqCst);
    // Wrap around if we somehow exhaust the range.
    if port > 30000 {
        NEXT.store(19876, Ordering::SeqCst);
        return NEXT.fetch_add(1, Ordering::SeqCst);
    }
    port
}

/// Extract the `state` query parameter value from an authorization URL.
fn extract_state(url: &str) -> String {
    url.split("state=")
        .nth(1)
        .and_then(|s| s.split('&').next())
        .map(|s| s.to_string())
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread")]
async fn test_full_authorization_flow() {
    let (token_url, server) = spawn_fake_server().await;
    let port = test_port();

    let config = OAuthConfig::new(
        "test-client-id",
        "https://example.com/authorize",
        token_url,
        vec!["read".into(), "write".into()],
    )
    .with_loopback_port(port);

    let flow = OAuthFlow::new(config);

    let tokens = flow
        .authorize(|auth_url: &str| {
            let state = extract_state(auth_url);

            // Simulate the browser redirect by making a raw TCP GET to the loopback.
            let mut conn = TcpStream::connect(format!("127.0.0.1:{port}"))
                .expect("should connect to loopback");
            let request = format!(
                "GET /callback?code=test-auth-code&state={state} HTTP/1.1\r\n\
                 Host: 127.0.0.1\r\n\
                 Connection: close\r\n\
                 \r\n"
            );
            conn.write_all(request.as_bytes())
                .expect("should write to loopback");

            // Read the response to ensure the loopback processed it.
            let mut response = String::new();
            let _ = conn.read_to_string(&mut response);
        })
        .await
        .expect("authorize should succeed");

    assert_eq!(tokens.access_token, "test-access-token");
    assert_eq!(tokens.refresh_token.as_deref(), Some("test-refresh-token"));
    assert_eq!(tokens.token_type, "Bearer");
    assert!(tokens.expires_at.is_some(), "should have expires_at");

    // Token should not be expired yet.
    assert!(!tokens.is_expired());

    // Verify the fake server saw the code_verifier.
    assert!(
        server.saw_code_verifier.load(Ordering::SeqCst),
        "token endpoint should have received code_verifier"
    );

    // Verify to_credential bridge (M1).
    let cred = tokens.to_credential();
    assert_eq!(
        cred.authorization_header().unwrap(),
        "Bearer test-access-token"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_state_mismatch_returns_error() {
    let (token_url, _server) = spawn_fake_server().await;
    let port = test_port();

    let config = OAuthConfig::new(
        "test-client-id",
        "https://example.com/authorize",
        token_url,
        vec![],
    )
    .with_loopback_port(port);

    let flow = OAuthFlow::new(config);

    let result = flow
        .authorize(|_auth_url: &str| {
            // Send a redirect with a *wrong* state.
            let mut conn = TcpStream::connect(format!("127.0.0.1:{port}"))
                .expect("should connect to loopback");
            let request = "GET /callback?code=test-code&state=wrong-state HTTP/1.1\r\n\
                 Host: 127.0.0.1\r\n\
                 Connection: close\r\n\
                 \r\n";
            conn.write_all(request.as_bytes())
                .expect("should write to loopback");
            let mut response = String::new();
            let _ = conn.read_to_string(&mut response);
        })
        .await;

    assert!(result.is_err(), "should fail on state mismatch");
    let err = result.unwrap_err();
    assert!(
        matches!(err, Error::OAuthError(_)),
        "expected OAuthError, got {err:?}"
    );
    assert!(
        err.to_string().contains("state mismatch"),
        "error should mention state mismatch: {err}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_as_error_redirect_returns_error() {
    let (token_url, _server) = spawn_fake_server().await;
    let port = test_port();

    let config = OAuthConfig::new(
        "test-client-id",
        "https://example.com/authorize",
        token_url,
        vec![],
    )
    .with_loopback_port(port);

    let flow = OAuthFlow::new(config);

    let result = flow
        .authorize(|auth_url: &str| {
            let state = extract_state(auth_url);
            // Simulate AS returning an error via redirect.
            let mut conn = TcpStream::connect(format!("127.0.0.1:{port}"))
                .expect("should connect to loopback");
            let request = format!(
                "GET /callback?error=access_denied&state={state} HTTP/1.1\r\n\
                 Host: 127.0.0.1\r\n\
                 Connection: close\r\n\
                 \r\n"
            );
            conn.write_all(request.as_bytes())
                .expect("should write to loopback");
            let mut response = String::new();
            let _ = conn.read_to_string(&mut response);
        })
        .await;

    assert!(result.is_err(), "should fail on AS error in redirect");
    let err = result.unwrap_err();
    assert!(matches!(err, Error::OAuthError(_)));
    assert!(
        err.to_string().contains("access_denied"),
        "error should contain AS error: {err}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_token_endpoint_error_returns_oauth_error() {
    let (token_url, server) = spawn_fake_server().await;
    // Enable error mode on the fake server.
    server.error_mode.store(true, Ordering::SeqCst);
    let port = test_port();

    let config = OAuthConfig::new(
        "test-client-id",
        "https://example.com/authorize",
        token_url,
        vec![],
    )
    .with_loopback_port(port);

    let flow = OAuthFlow::new(config);

    let result = flow
        .authorize(|auth_url: &str| {
            let state = extract_state(auth_url);
            let mut conn = TcpStream::connect(format!("127.0.0.1:{port}"))
                .expect("should connect to loopback");
            let request = format!(
                "GET /callback?code=test-code&state={state} HTTP/1.1\r\n\
                 Host: 127.0.0.1\r\n\
                 Connection: close\r\n\
                 \r\n"
            );
            conn.write_all(request.as_bytes())
                .expect("should write to loopback");
            let mut response = String::new();
            let _ = conn.read_to_string(&mut response);
        })
        .await;

    assert!(
        result.is_err(),
        "should fail when token endpoint returns error"
    );
    let err = result.unwrap_err();
    assert!(matches!(err, Error::OAuthError(_)));
    assert!(
        err.to_string().contains("invalid_grant"),
        "error should contain token error: {err}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_refresh_returns_new_access_token() {
    let (token_url, server) = spawn_fake_server().await;

    let config = OAuthConfig::new(
        "test-client-id",
        "https://example.com/authorize",
        token_url,
        vec![],
    );

    let flow = OAuthFlow::new(config);

    let new_tokens = flow
        .refresh("existing-refresh-token")
        .await
        .expect("refresh should succeed");

    assert_eq!(new_tokens.access_token, "new-access-token");
    assert!(new_tokens.expires_at.is_some());
    assert_eq!(new_tokens.token_type, "Bearer");

    // When rotate_refresh is false, the original refresh token is preserved.
    assert_eq!(
        new_tokens.refresh_token.as_deref(),
        Some("existing-refresh-token"),
        "should preserve original refresh token when not rotated"
    );

    assert!(
        server.saw_refresh_token.load(Ordering::SeqCst),
        "token endpoint should have received refresh_token"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_refresh_with_rotation_preserves_new_refresh_token() {
    let (token_url, server) = spawn_fake_server().await;
    server.rotate_refresh.store(true, Ordering::SeqCst);

    let config = OAuthConfig::new(
        "test-client-id",
        "https://example.com/authorize",
        token_url,
        vec![],
    );

    let flow = OAuthFlow::new(config);

    let new_tokens = flow
        .refresh("old-refresh-token")
        .await
        .expect("refresh should succeed");

    assert_eq!(new_tokens.access_token, "new-access-token");
    assert_eq!(
        new_tokens.refresh_token.as_deref(),
        Some("new-refresh-token"),
        "should use the new refresh token when the server rotates it"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_refresh_token_error() {
    let (token_url, server) = spawn_fake_server().await;
    server.error_mode.store(true, Ordering::SeqCst);

    let config = OAuthConfig::new(
        "test-client-id",
        "https://example.com/authorize",
        token_url,
        vec![],
    );

    let flow = OAuthFlow::new(config);

    let result = flow.refresh("expired-refresh-token").await;
    assert!(
        result.is_err(),
        "refresh should fail on token endpoint error"
    );
    let err = result.unwrap_err();
    assert!(matches!(err, Error::OAuthError(_)));
}

#[tokio::test(flavor = "multi_thread")]
async fn test_authorization_url_contains_required_params() {
    let (token_url, _server) = spawn_fake_server().await;
    let port = test_port();

    let config = OAuthConfig::new(
        "my-client",
        "https://example.com/authorize",
        token_url,
        vec!["profile".into(), "email".into()],
    )
    .with_loopback_port(port);

    let flow = OAuthFlow::new(config);

    let result = flow
        .authorize(|auth_url: &str| {
            // Validate the URL structure before simulating the redirect.
            assert!(auth_url.starts_with("https://example.com/authorize?"));
            assert!(auth_url.contains("response_type=code"));
            assert!(auth_url.contains("client_id=my-client"));
            assert!(auth_url.contains("code_challenge_method=S256"));
            assert!(auth_url.contains("code_challenge="));
            assert!(auth_url.contains("state="));
            assert!(auth_url.contains(&format!(
                "redirect_uri=http%3A%2F%2F127.0.0.1%3A{port}%2Fcallback"
            )));
            // The scope is space-separated and URL-encoded.
            assert!(auth_url.contains("scope=profile%20email"));

            let state = extract_state(auth_url);
            let mut conn = TcpStream::connect(format!("127.0.0.1:{port}"))
                .expect("should connect to loopback");
            let request = format!(
                "GET /callback?code=test-code&state={state} HTTP/1.1\r\n\
                 Host: 127.0.0.1\r\n\
                 Connection: close\r\n\
                 \r\n"
            );
            conn.write_all(request.as_bytes())
                .expect("should write to loopback");
            let mut response = String::new();
            let _ = conn.read_to_string(&mut response);
        })
        .await;

    assert!(result.is_ok(), "authorize should succeed: {result:?}");
}
