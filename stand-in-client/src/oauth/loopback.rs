//! Loopback redirect capture — accepts one TCP connection and captures
//! the authorization `code` and `state` from a single redirect request.

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use crate::error::{Error, Result};

/// Parameters captured from the redirect URI query string.
#[derive(Debug)]
pub(crate) struct RedirectParams {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
}

/// Accept exactly one connection from an already-bound `TcpListener`,
/// parse the query string from the request line, respond with a minimal HTML
/// page, and return the captured parameters.
///
/// The caller must have bound the listener before calling this function.
/// This ensures the port is open before the browser redirect is triggered.
pub(crate) async fn accept_one(listener: TcpListener) -> Result<RedirectParams> {
    let (mut stream, _peer) = listener
        .accept()
        .await
        .map_err(|e| Error::OAuthError(format!("loopback accept failed: {e}")))?;

    let mut buf = vec![0u8; 4096];
    let n = stream
        .read(&mut buf)
        .await
        .map_err(|e| Error::OAuthError(format!("loopback read failed: {e}")))?;

    let request = std::str::from_utf8(&buf[..n])
        .map_err(|_| Error::OAuthError("loopback received non-UTF-8 request".into()))?;

    let first_line = request.lines().next().unwrap_or("");
    let params = parse_redirect_params(first_line);

    let html = "<!DOCTYPE html><html><head><meta charset=\"utf-8\"><title>Authentication Complete</title></head><body><h1>Authentication complete</h1><p>You may close this tab.</p></body></html>";
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{html}",
        html.len()
    );

    let _ = stream.write_all(response.as_bytes()).await;
    let _ = stream.shutdown().await;

    Ok(params)
}

/// Parse query parameters from an HTTP request line.
///
/// Expects `GET /callback?key1=val1&key2=val2 HTTP/1.1` and extracts
/// `code`, `state`, and `error` parameters with minimal URL decoding.
fn parse_redirect_params(request_line: &str) -> RedirectParams {
    let query = request_line
        .split_whitespace()
        .nth(1)
        .and_then(|path| path.split('?').nth(1))
        .unwrap_or("");

    let mut code = None;
    let mut state = None;
    let mut error = None;

    for pair in query.split('&') {
        let (key, val) = match pair.split_once('=') {
            Some((k, v)) => (k, url_decode(v)),
            None => (pair, String::new()),
        };
        match key {
            "code" => code = Some(val),
            "state" => state = Some(val),
            "error" => error = Some(val),
            _ => {}
        }
    }

    RedirectParams { code, state, error }
}

/// Minimal URL percent-decode.
///
/// Decodes `%xx` hex sequences and `+` → space. Unrecognized sequences
/// are passed through as-is.
fn url_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'%' if i + 2 < bytes.len() => {
                if let (Some(hi), Some(lo)) = (hex_val(bytes[i + 1]), hex_val(bytes[i + 2])) {
                    result.push((hi << 4 | lo) as char);
                    i += 3;
                } else {
                    result.push('%');
                    i += 1;
                }
            }
            b'+' => {
                result.push(' ');
                i += 1;
            }
            b => {
                result.push(b as char);
                i += 1;
            }
        }
    }
    result
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'A'..=b'F' => Some(b - b'A' + 10),
        b'a'..=b'f' => Some(b - b'a' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_redirect_params_happy() {
        let line = "GET /callback?code=abc123&state=xyz789 HTTP/1.1";
        let params = parse_redirect_params(line);
        assert_eq!(params.code.as_deref(), Some("abc123"));
        assert_eq!(params.state.as_deref(), Some("xyz789"));
        assert!(params.error.is_none());
    }

    #[test]
    fn test_parse_redirect_params_with_error() {
        let line = "GET /callback?error=access_denied&state=st HTTP/1.1";
        let params = parse_redirect_params(line);
        assert_eq!(params.error.as_deref(), Some("access_denied"));
        assert_eq!(params.state.as_deref(), Some("st"));
        assert!(params.code.is_none());
    }

    #[test]
    fn test_parse_redirect_params_url_encoded() {
        let line = "GET /callback?code=c%2Bo%20de&state=st%21 HTTP/1.1";
        let params = parse_redirect_params(line);
        assert_eq!(params.code.as_deref(), Some("c+o de"));
        assert_eq!(params.state.as_deref(), Some("st!"));
    }

    #[test]
    fn test_parse_redirect_params_no_query() {
        let line = "GET /callback HTTP/1.1";
        let params = parse_redirect_params(line);
        assert!(params.code.is_none());
        assert!(params.state.is_none());
        assert!(params.error.is_none());
    }

    #[test]
    fn test_url_decode_basic() {
        assert_eq!(url_decode("hello%20world"), "hello world");
        assert_eq!(url_decode("a+b"), "a b");
        assert_eq!(
            url_decode("%21%2A%27%28%29%3B%3A%40%26%3D%2B%24%2C%2F%3F%23%5B%5D"),
            "!*'();:@&=+$,/?#[]"
        );
    }

    #[test]
    fn test_url_decode_truncated_percent() {
        assert_eq!(url_decode("ab%2"), "ab%2");
        assert_eq!(url_decode("ab%"), "ab%");
    }

    #[test]
    fn test_url_decode_invalid_hex() {
        assert_eq!(url_decode("%GG"), "%GG");
        assert_eq!(url_decode("%0G"), "%0G");
    }
}
