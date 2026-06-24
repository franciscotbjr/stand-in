//! PKCE (Proof Key for Code Exchange) — RFC 7636 §4.

use base64::Engine;
use rand::Rng;
use sha2::{Digest, Sha256};

/// PKCE parameters: a random code verifier and its S256 challenge.
///
/// `Debug` is redacted — the verifier is a secret.
pub(crate) struct Pkce {
    pub verifier: String,
    pub challenge: String,
}

impl std::fmt::Debug for Pkce {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pkce")
            .field("verifier", &"***")
            .field("challenge", &self.challenge)
            .finish()
    }
}

impl Pkce {
    /// Generate a new PKCE pair using S256.
    ///
    /// - `verifier` = base64url-nopad of 32 CSPRNG bytes (43 characters, RFC 7636 §4.1).
    /// - `challenge` = base64url-nopad of SHA-256(verifier) (RFC 7636 §4.2).
    pub fn generate() -> Self {
        let verifier_bytes: [u8; 32] = rand::rng().random();
        let verifier = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(verifier_bytes);

        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        let hash = hasher.finalize();
        let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hash);

        Self {
            verifier,
            challenge,
        }
    }
}

/// Generate a random anti-CSRF `state` parameter.
///
/// Returns 16 CSPRNG bytes encoded as base64url-nopad (22 characters).
pub(crate) fn generate_state() -> String {
    let bytes: [u8; 16] = rand::rng().random();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify PKCE against the RFC 7636 Appendix B test vector.
    ///
    /// Appendix B provides a known code_verifier → code_challenge pair:
    ///   verifier:  dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk
    ///   challenge: E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM
    #[test]
    fn test_pkce_rfc7636_appendix_b_vector() {
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        let hash = hasher.finalize();
        let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hash);
        assert_eq!(challenge, "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM");
    }

    #[test]
    fn test_verifier_is_43_chars() {
        let pkce = Pkce::generate();
        assert_eq!(
            pkce.verifier.len(),
            43,
            "verifier should be 43 chars (32 bytes base64url-nopad)"
        );
    }

    #[test]
    fn test_verifier_only_url_safe_chars() {
        let pkce = Pkce::generate();
        for ch in pkce.verifier.chars() {
            assert!(
                ch.is_ascii_alphanumeric() || ch == '-' || ch == '_',
                "verifier character '{ch}' is not URL-safe base64"
            );
        }
    }

    #[test]
    fn test_challenge_is_43_chars() {
        let pkce = Pkce::generate();
        assert_eq!(
            pkce.challenge.len(),
            43,
            "challenge should be 43 chars (SHA-256 base64url-nopad)"
        );
    }

    #[test]
    fn test_challenge_is_sha256_of_verifier() {
        let pkce = Pkce::generate();
        let mut hasher = Sha256::new();
        hasher.update(pkce.verifier.as_bytes());
        let hash = hasher.finalize();
        let expected = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hash);
        assert_eq!(pkce.challenge, expected);
    }

    #[test]
    fn test_state_is_22_chars() {
        let state = generate_state();
        assert_eq!(
            state.len(),
            22,
            "state should be 22 chars (16 bytes base64url-nopad)"
        );
    }

    #[test]
    fn test_generate_is_deterministic_in_shape() {
        let a = Pkce::generate();
        let b = Pkce::generate();
        // Two generations should produce different values (probabilistic).
        assert_ne!(a.verifier, b.verifier);
        assert_ne!(a.challenge, b.challenge);
        // But each verifier→challenge mapping must be correct.
        let mut hasher = Sha256::new();
        hasher.update(a.verifier.as_bytes());
        let a_expected = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hasher.finalize());
        assert_eq!(a.challenge, a_expected);
    }

    #[test]
    fn test_pkce_debug_redacted() {
        let pkce = Pkce::generate();
        let debug = format!("{:?}", pkce);
        assert!(debug.contains("***"));
        assert!(!debug.contains(&pkce.verifier));
        // challenge should be visible (it is public).
        assert!(debug.contains(&pkce.challenge));
    }
}
