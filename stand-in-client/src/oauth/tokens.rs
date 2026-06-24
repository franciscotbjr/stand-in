//! OAuth 2.0 token pair: access token, optional refresh token, expiry.

use std::time::{Duration, SystemTime};

use crate::auth::Credential;

/// OAuth 2.0 token response.
///
/// `Debug` is redacted — the `access_token` and `refresh_token` are
/// never logged. The `Credential` bridge (M1) is [`to_credential`].
///
/// [`to_credential`]: OAuthTokens::to_credential
#[derive(Clone)]
pub struct OAuthTokens {
    /// The access token for API calls.
    pub access_token: String,
    /// Refresh token used to obtain a new access token (may be absent).
    pub refresh_token: Option<String>,
    /// When the access token expires. `None` means the token never expires.
    pub expires_at: Option<SystemTime>,
    /// Token type, usually `"Bearer"`.
    pub token_type: String,
}

impl std::fmt::Debug for OAuthTokens {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OAuthTokens")
            .field("access_token", &"***")
            .field("refresh_token", &self.refresh_token.as_ref().map(|_| "***"))
            .field("expires_at", &self.expires_at)
            .field("token_type", &self.token_type)
            .finish()
    }
}

impl OAuthTokens {
    /// Returns `true` if the access token is expired (or within 30 seconds of expiring).
    ///
    /// When `expires_at` is `None` (the authorization server did not provide
    /// `expires_in`), returns `false` — the token is treated as non-expiring.
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            None => false,
            Some(exp) => exp <= SystemTime::now() + Duration::from_secs(30),
        }
    }

    /// Convert to a [`Credential::Bearer`] for injection into
    /// [`HttpTransport`](crate::transport::HttpTransport) (M1 bridge).
    pub fn to_credential(&self) -> Credential {
        Credential::bearer(self.access_token.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_expired_no_expires_at() {
        let tokens = OAuthTokens {
            access_token: "tok".into(),
            refresh_token: None,
            expires_at: None,
            token_type: "Bearer".into(),
        };
        assert!(!tokens.is_expired());
    }

    #[test]
    fn test_is_expired_future() {
        let tokens = OAuthTokens {
            access_token: "tok".into(),
            refresh_token: None,
            expires_at: Some(SystemTime::now() + Duration::from_secs(3600)),
            token_type: "Bearer".into(),
        };
        assert!(!tokens.is_expired());
    }

    #[test]
    fn test_is_expired_within_grace_period() {
        // 10 seconds from now — within the 30s grace window → expired.
        let tokens = OAuthTokens {
            access_token: "tok".into(),
            refresh_token: None,
            expires_at: Some(SystemTime::now() + Duration::from_secs(10)),
            token_type: "Bearer".into(),
        };
        assert!(tokens.is_expired());
    }

    #[test]
    fn test_is_expired_past() {
        let tokens = OAuthTokens {
            access_token: "tok".into(),
            refresh_token: None,
            expires_at: Some(SystemTime::now() - Duration::from_secs(60)),
            token_type: "Bearer".into(),
        };
        assert!(tokens.is_expired());
    }

    #[test]
    fn test_to_credential_returns_bearer() {
        let tokens = OAuthTokens {
            access_token: "my-access".into(),
            refresh_token: Some("my-refresh".into()),
            expires_at: None,
            token_type: "Bearer".into(),
        };
        let cred = tokens.to_credential();
        let header = cred.authorization_header().unwrap();
        assert_eq!(header, "Bearer my-access");
    }

    #[test]
    fn test_debug_redacted() {
        let tokens = OAuthTokens {
            access_token: "secret-access".into(),
            refresh_token: Some("secret-refresh".into()),
            expires_at: None,
            token_type: "Bearer".into(),
        };
        let debug = format!("{:?}", tokens);
        assert!(debug.contains("***"));
        assert!(!debug.contains("secret-access"));
        assert!(!debug.contains("secret-refresh"));
    }

    #[test]
    fn test_debug_no_refresh_token() {
        let tokens = OAuthTokens {
            access_token: "secret-access".into(),
            refresh_token: None,
            expires_at: None,
            token_type: "Bearer".into(),
        };
        let debug = format!("{:?}", tokens);
        assert!(debug.contains("***"));
        assert!(!debug.contains("secret-access"));
    }

    #[test]
    fn test_tokens_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<OAuthTokens>();
    }
}
