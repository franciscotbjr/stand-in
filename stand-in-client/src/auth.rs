//! Credential types for HTTP authentication.
//!
//! Gated behind the `http` feature — auth applies only to the HTTP transport path (D0).

use base64::Engine;

/// Credential for authenticating with an MCP server over HTTP.
///
/// Injected as the `Authorization` header on every request (POST `/mcp`,
/// GET-SSE `/mcp`, DELETE `/mcp`). Only meaningful with [`HttpTransport`];
/// [`StdioTransport`] ignores authentication by construction (subprocess-local).
///
/// `#[non_exhaustive]` preserves SemVer extension space for OAuth 2.0 (M2)
/// without breaking existing consumers.
///
/// [`HttpTransport`]: crate::transport::HttpTransport
/// [`StdioTransport`]: crate::transport::StdioTransport
#[non_exhaustive]
#[derive(Clone, Default)]
pub enum Credential {
    /// No `Authorization` header is sent.
    #[default]
    NoAuth,
    /// HTTP Basic authentication — `Authorization: Basic base64(username:password)`.
    Basic {
        /// The username portion of the credential.
        username: String,
        /// The password portion of the credential (masked in `Debug`).
        password: String,
    },
    /// Bearer token — `Authorization: Bearer <token>`.
    Bearer {
        /// The bearer token value (masked in `Debug`).
        token: String,
    },
}

impl std::fmt::Debug for Credential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoAuth => write!(f, "NoAuth"),
            Self::Basic { username, .. } => f
                .debug_struct("Basic")
                .field("username", username)
                .field("password", &"***")
                .finish(),
            Self::Bearer { .. } => f.debug_struct("Bearer").field("token", &"***").finish(),
        }
    }
}

impl Credential {
    /// Construct a `Basic` credential.
    ///
    /// **Required:** `#[non_exhaustive]` blocks struct-variant construction from
    /// downstream crates, so this constructor (and [`bearer`]) are the only
    /// way to obtain a non-`NoAuth` credential from outside `stand-in-client`.
    ///
    /// [`bearer`]: Credential::bearer
    pub fn basic(username: String, password: String) -> Self {
        Self::Basic { username, password }
    }

    /// Construct a `Bearer` credential.
    pub fn bearer(token: String) -> Self {
        Self::Bearer { token }
    }

    /// Compute the `Authorization` header value for this credential.
    ///
    /// Returns `None` for `NoAuth`, otherwise the complete header value
    /// (e.g. `"Basic QWxhZGRpbjpvcGVuIHNlc2FtZQ=="` or `"Bearer my-token"`).
    pub fn authorization_header(&self) -> Option<String> {
        match self {
            Self::NoAuth => None,
            Self::Basic { username, password } => {
                let raw = format!("{username}:{password}");
                let encoded = base64::engine::general_purpose::STANDARD.encode(raw.as_bytes());
                Some(format!("Basic {encoded}"))
            }
            Self::Bearer { token } => Some(format!("Bearer {token}")),
        }
    }
}

/// Build the fixed set of headers for a `POST /mcp` request.
///
/// Always includes `Content-Type: application/json` and
/// `Accept: application/json, text/event-stream`. Optionally includes
/// `Mcp-Session-Id` (when the session has been established) and
/// `Authorization` (when a credential is configured).
///
/// Returns `Err(ConnectionError)` if the `auth` value contains bytes
/// that are invalid for an HTTP header (user-controlled input — must
/// never panic).
pub(crate) fn build_post_headers(
    session_id: Option<&str>,
    auth: Option<&str>,
) -> crate::error::Result<reqwest::header::HeaderMap> {
    let mut headers = reqwest::header::HeaderMap::new();

    headers.insert(
        reqwest::header::CONTENT_TYPE,
        "application/json".parse().unwrap(),
    );
    headers.insert(
        reqwest::header::ACCEPT,
        "application/json, text/event-stream".parse().unwrap(),
    );

    // Mcp-Session-Id is server-generated — lenient (swallow parse failure).
    if let Some(sid) = session_id
        && let Ok(val) = sid.parse()
    {
        headers.insert("Mcp-Session-Id", val);
    }

    // Authorization is user-controlled — must not panic on malformed values.
    if let Some(auth_val) = auth {
        let header_val = reqwest::header::HeaderValue::from_str(auth_val).map_err(|e| {
            crate::error::Error::ConnectionError(format!("invalid credential header: {e}"))
        })?;
        headers.insert(reqwest::header::AUTHORIZATION, header_val);
    }

    Ok(headers)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── authorization_header ────────────────────────────────────────────

    #[test]
    fn test_noauth_returns_none() {
        let cred = Credential::NoAuth;
        assert_eq!(cred.authorization_header(), None);
    }

    #[test]
    fn test_basic_known_vector() {
        let cred = Credential::basic("Aladdin".into(), "open sesame".into());
        let header = cred.authorization_header().unwrap();
        // Verified against RFC 7617 §2 example.
        assert_eq!(header, "Basic QWxhZGRpbjpvcGVuIHNlc2FtZQ==");
    }

    #[test]
    fn test_bearer_token() {
        let cred = Credential::bearer("secret-token".into());
        let header = cred.authorization_header().unwrap();
        assert_eq!(header, "Bearer secret-token");
    }

    #[test]
    fn test_basic_empty_password() {
        let cred = Credential::basic("user".into(), String::new());
        let header = cred.authorization_header().unwrap();
        assert!(header.starts_with("Basic "));
        let encoded = &header[6..];
        let decoded = String::from_utf8(
            base64::engine::general_purpose::STANDARD
                .decode(encoded)
                .unwrap(),
        )
        .unwrap();
        assert_eq!(decoded, "user:");
    }

    #[test]
    fn test_basic_password_with_colon() {
        let cred = Credential::basic("user".into(), "pass:word".into());
        let header = cred.authorization_header().unwrap();
        assert!(header.starts_with("Basic "));
        let encoded = &header[6..];
        let decoded = String::from_utf8(
            base64::engine::general_purpose::STANDARD
                .decode(encoded)
                .unwrap(),
        )
        .unwrap();
        assert_eq!(decoded, "user:pass:word");
    }

    #[test]
    fn test_bearer_long_token() {
        let long = "a".repeat(1000);
        let cred = Credential::bearer(long.clone());
        let header = cred.authorization_header().unwrap();
        assert_eq!(header, format!("Bearer {long}"));
    }

    // ── Debug redaction ─────────────────────────────────────────────────

    #[test]
    fn test_debug_noauth() {
        assert_eq!(format!("{:?}", Credential::NoAuth), "NoAuth");
    }

    #[test]
    fn test_debug_basic_redacted() {
        let cred = Credential::basic("admin".into(), "s3cret".into());
        let debug = format!("{:?}", cred);
        assert!(debug.contains("admin"));
        assert!(debug.contains("***"));
        assert!(!debug.contains("s3cret"));
    }

    #[test]
    fn test_debug_bearer_redacted() {
        let cred = Credential::bearer("abc123".into());
        let debug = format!("{:?}", cred);
        assert!(debug.contains("***"));
        assert!(!debug.contains("abc123"));
    }

    // ── Trait impls ─────────────────────────────────────────────────────

    #[test]
    fn test_clone() {
        let a = Credential::basic("u".into(), "p".into());
        let b = a.clone();
        assert_eq!(a.authorization_header(), b.authorization_header());
    }

    #[test]
    fn test_default_is_noauth() {
        let cred = Credential::default();
        assert!(matches!(cred, Credential::NoAuth));
    }

    #[test]
    fn test_credential_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Credential>();
    }

    // ── build_post_headers ──────────────────────────────────────────────

    #[test]
    fn test_build_post_headers_no_session_no_auth() {
        let headers = super::build_post_headers(None, None).unwrap();
        assert_eq!(headers.get("content-type").unwrap(), "application/json");
        assert_eq!(
            headers.get("accept").unwrap(),
            "application/json, text/event-stream"
        );
        assert!(headers.get("mcp-session-id").is_none());
        assert!(headers.get("authorization").is_none());
    }

    #[test]
    fn test_build_post_headers_with_session_id() {
        let headers = super::build_post_headers(Some("abc-123"), None).unwrap();
        assert_eq!(headers.get("mcp-session-id").unwrap(), "abc-123");
        assert!(headers.get("authorization").is_none());
    }

    #[test]
    fn test_build_post_headers_with_auth() {
        let headers = super::build_post_headers(None, Some("Bearer tok")).unwrap();
        assert!(headers.get("mcp-session-id").is_none());
        assert_eq!(headers.get("authorization").unwrap(), "Bearer tok");
    }

    #[test]
    fn test_build_post_headers_both() {
        let headers =
            super::build_post_headers(Some("sid"), Some("Basic QWxhZGRpbjpvcGVuIHNlc2FtZQ=="))
                .unwrap();
        assert_eq!(headers.get("mcp-session-id").unwrap(), "sid");
        assert_eq!(
            headers.get("authorization").unwrap(),
            "Basic QWxhZGRpbjpvcGVuIHNlc2FtZQ=="
        );
    }

    #[test]
    fn test_build_post_headers_malformed_auth_rejected() {
        // HeaderValue rejects control characters like \n.
        let result = super::build_post_headers(None, Some("Bearer ab\nc"));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, crate::error::Error::ConnectionError(_)),
            "expected ConnectionError, got {:?}",
            err
        );
        assert!(
            err.to_string().contains("invalid credential header"),
            "error message should mention credential header: {err}",
        );
    }
}
