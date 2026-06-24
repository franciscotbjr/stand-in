//! Integration tests for credential header injection.
//!
//! Spawns an in-process axum "recorder" server that captures the
//! `Authorization` header per verb (POST, GET, DELETE), then drives
//! the `HttpTransport` directly against it. This proves that the
//! credential is injected on all three HTTP verbs and that `NoAuth`
//! sends no header.

#![cfg(feature = "http")]

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::{Router, extract::State, http::Request, response::Response, routing};
use stand_in_client::auth::Credential;
use stand_in_client::transport::ClientTransport;
use stand_in_client::transport::HttpTransport;

// ---------------------------------------------------------------------------
// Test server — records the Authorization header per verb
// ---------------------------------------------------------------------------

#[derive(Debug, Default, Clone)]
struct Recorder {
    post_auth: Arc<Mutex<Option<String>>>,
    get_auth: Arc<Mutex<Option<String>>>,
    delete_auth: Arc<Mutex<Option<String>>>,
}

impl Recorder {
    fn new() -> Self {
        Self {
            post_auth: Arc::new(Mutex::new(None)),
            get_auth: Arc::new(Mutex::new(None)),
            delete_auth: Arc::new(Mutex::new(None)),
        }
    }
}

async fn handle_post(State(rec): State<Recorder>, req: Request<axum::body::Body>) -> Response {
    // Record the Authorization header if present.
    let auth = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    *rec.post_auth.lock().unwrap() = auth;

    // Return a minimal JSON-RPC response with Mcp-Session-Id so the
    // transport spawns the GET-SSE task.
    axum::http::Response::builder()
        .status(200)
        .header("Content-Type", "application/json")
        .header("Mcp-Session-Id", "test-session")
        .body(axum::body::Body::from(
            r#"{"jsonrpc":"2.0","id":1,"result":{}}"#,
        ))
        .unwrap()
}

async fn handle_get(State(rec): State<Recorder>, req: Request<axum::body::Body>) -> Response {
    let auth = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    *rec.get_auth.lock().unwrap() = auth;

    // Return a minimal SSE stream that closes immediately.
    axum::http::Response::builder()
        .status(200)
        .header("Content-Type", "text/event-stream")
        .body(axum::body::Body::from(":ok\n\n"))
        .unwrap()
}

async fn handle_delete(State(rec): State<Recorder>, req: Request<axum::body::Body>) -> Response {
    let auth = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    *rec.delete_auth.lock().unwrap() = auth;

    axum::http::Response::builder()
        .status(200)
        .body(axum::body::Body::empty())
        .unwrap()
}

fn free_addr() -> SocketAddr {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap()
}

async fn spawn_server(recorder: Recorder) -> String {
    let addr = free_addr();
    let app = Router::new()
        .route(
            "/mcp",
            routing::post(handle_post)
                .get(handle_get)
                .delete(handle_delete),
        )
        .with_state(recorder);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let url = format!("http://{addr}/mcp");
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    url
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_noauth_sends_no_header() {
    let recorder = Recorder::new();
    let url = spawn_server(recorder.clone()).await;

    let mut transport = HttpTransport::new(&url);
    transport.connect().await.unwrap();
    transport
        .send(r#"{"jsonrpc":"2.0","id":1,"method":"test"}"#)
        .await
        .unwrap();
    // Wait for the async GET-SSE task to run.
    tokio::time::sleep(Duration::from_millis(400)).await;
    transport.close().await.unwrap();

    assert_eq!(
        *recorder.post_auth.lock().unwrap(),
        None,
        "NoAuth should not send Authorization on POST"
    );
    assert_eq!(
        *recorder.get_auth.lock().unwrap(),
        None,
        "NoAuth should not send Authorization on GET-SSE"
    );
    assert_eq!(
        *recorder.delete_auth.lock().unwrap(),
        None,
        "NoAuth should not send Authorization on DELETE"
    );
}

#[tokio::test]
async fn test_basic_sends_correct_header() {
    let recorder = Recorder::new();
    let url = spawn_server(recorder.clone()).await;

    let credential = Credential::basic("myuser".into(), "mypass".into());
    let expected_auth = credential.authorization_header().unwrap();

    let mut transport = HttpTransport::new(&url).with_credential(credential);
    transport.connect().await.unwrap();
    transport
        .send(r#"{"jsonrpc":"2.0","id":1,"method":"test"}"#)
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(400)).await;
    transport.close().await.unwrap();

    assert_eq!(
        *recorder.post_auth.lock().unwrap(),
        Some(expected_auth.clone()),
        "Basic credential should send Authorization on POST"
    );
    assert_eq!(
        *recorder.get_auth.lock().unwrap(),
        Some(expected_auth.clone()),
        "Basic credential should send Authorization on GET-SSE"
    );
    assert_eq!(
        *recorder.delete_auth.lock().unwrap(),
        Some(expected_auth),
        "Basic credential should send Authorization on DELETE"
    );
}

#[tokio::test]
async fn test_bearer_sends_correct_header() {
    let recorder = Recorder::new();
    let url = spawn_server(recorder.clone()).await;

    let credential = Credential::bearer("s3cr3t-t0k3n".into());
    let expected_auth = credential.authorization_header().unwrap();

    let mut transport = HttpTransport::new(&url).with_credential(credential);
    transport.connect().await.unwrap();
    transport
        .send(r#"{"jsonrpc":"2.0","id":1,"method":"test"}"#)
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(400)).await;
    transport.close().await.unwrap();

    assert_eq!(
        *recorder.post_auth.lock().unwrap(),
        Some(expected_auth.clone()),
        "Bearer credential should send Authorization on POST"
    );
    assert_eq!(
        *recorder.get_auth.lock().unwrap(),
        Some(expected_auth.clone()),
        "Bearer credential should send Authorization on GET-SSE"
    );
    assert_eq!(
        *recorder.delete_auth.lock().unwrap(),
        Some(expected_auth),
        "Bearer credential should send Authorization on DELETE"
    );
}

/// Verify the constructors are accessible from a downstream crate (the
/// design-gate r1 finding). This is an integration test in a separate
/// crate, so it proves the constructors are truly public.
#[test]
fn test_constructors_accessible_from_downstream() {
    let _ = Credential::basic("user".into(), "pass".into());
    let _ = Credential::bearer("tok".into());
    let _ = Credential::default(); // NoAuth
}
