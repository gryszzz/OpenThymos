//! JWT authentication middleware.
//!
//! Verifies `Authorization: Bearer <JWT>` tokens using HMAC-SHA256 (symmetric)
//! or RS256 (asymmetric). Extracts standard claims into request extensions so
//! that downstream handlers can mint tenant-scoped writs from validated identity.
//!
//! Configuration:
//!   - `THYMOS_JWT_SECRET` — HMAC secret for HS256 (simplest setup).
//!   - `THYMOS_JWT_ISSUER` — optional expected `iss` claim.
//!   - `THYMOS_JWT_AUDIENCE` — optional expected `aud` claim.

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Json},
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Claims extracted from a verified JWT.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    /// Subject — the authenticated user ID.
    pub sub: String,
    /// Tenant / organization ID (custom claim).
    #[serde(default)]
    pub tenant_id: Option<String>,
    /// Display name (custom claim).
    #[serde(default)]
    pub name: Option<String>,
    /// Email (custom claim).
    #[serde(default)]
    pub email: Option<String>,
    /// Roles (custom claim).
    #[serde(default)]
    pub roles: Vec<String>,
    /// Standard: issuer.
    #[serde(default)]
    pub iss: Option<String>,
    /// Standard: audience (string or array; we accept string).
    #[serde(default)]
    pub aud: Option<String>,
    /// Standard: expiration (Unix timestamp).
    #[serde(default)]
    pub exp: Option<u64>,
    /// Standard: issued at.
    #[serde(default)]
    pub iat: Option<u64>,
}

/// JWT verifier configuration.
pub struct JwtConfig {
    pub decoding_key: DecodingKey,
    pub validation: Validation,
}

impl JwtConfig {
    /// Create from an HMAC secret (HS256).
    pub fn from_secret(secret: &[u8]) -> Self {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;
        // Optionally set issuer/audience from env.
        if let Ok(iss) = std::env::var("THYMOS_JWT_ISSUER") {
            validation.set_issuer(&[iss]);
        }
        if let Ok(aud) = std::env::var("THYMOS_JWT_AUDIENCE") {
            validation.set_audience(&[aud]);
        }
        JwtConfig {
            decoding_key: DecodingKey::from_secret(secret),
            validation,
        }
    }

    /// Verify a JWT and return decoded claims.
    pub fn verify(&self, token: &str) -> Result<JwtClaims, String> {
        let token_data = decode::<JwtClaims>(token, &self.decoding_key, &self.validation)
            .map_err(|e| format!("JWT verification failed: {e}"))?;
        Ok(token_data.claims)
    }
}

/// Axum middleware that verifies JWTs and injects `JwtClaims` into request
/// extensions. Requests without a valid JWT are rejected with 401.
///
/// Skips `/health` + `/ready` and allows API-key-based auth to coexist (if
/// `x-thymos-user-id` header is already set, JWT check is skipped — the
/// API gateway already authenticated the request).
pub async fn jwt_middleware(
    axum::extract::State(config): axum::extract::State<Arc<JwtConfig>>,
    mut request: Request,
    next: Next,
) -> impl IntoResponse {
    let path = request.uri().path().to_string();

    // Skip auth for health and readiness checks.
    if matches!(path.as_str(), "/health" | "/ready") {
        return next.run(request).await.into_response();
    }

    // If the API gateway already authenticated (x-thymos-user-id is set),
    // skip JWT verification.
    if request.headers().contains_key("x-thymos-user-id") {
        return next.run(request).await.into_response();
    }

    let auth_header = request
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok());

    let token = match auth_header {
        Some(h) if h.starts_with("Bearer ") => &h[7..],
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "missing or invalid Authorization header" })),
            )
                .into_response()
        }
    };

    match config.verify(token) {
        Ok(claims) => {
            // Inject claims into request extensions for downstream handlers.
            request.extensions_mut().insert(claims);
            next.run(request).await.into_response()
        }
        Err(e) => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{encode, EncodingKey, Header};

    fn test_config() -> JwtConfig {
        JwtConfig::from_secret(b"test-secret-key-at-least-32-bytes!")
    }

    fn make_token(claims: &JwtClaims) -> String {
        encode(
            &Header::default(),
            claims,
            &EncodingKey::from_secret(b"test-secret-key-at-least-32-bytes!"),
        )
        .unwrap()
    }

    #[test]
    fn valid_token_verifies() {
        let config = test_config();
        let claims = JwtClaims {
            sub: "user-123".into(),
            tenant_id: Some("tenant-abc".into()),
            name: Some("Test User".into()),
            email: None,
            roles: vec!["admin".into()],
            iss: None,
            aud: None,
            exp: Some(u64::MAX),
            iat: Some(0),
        };
        let token = make_token(&claims);
        let verified = config.verify(&token).unwrap();
        assert_eq!(verified.sub, "user-123");
        assert_eq!(verified.tenant_id, Some("tenant-abc".into()));
        assert_eq!(verified.roles, vec!["admin"]);
    }

    #[test]
    fn expired_token_rejected() {
        let config = test_config();
        let claims = JwtClaims {
            sub: "user-123".into(),
            tenant_id: None,
            name: None,
            email: None,
            roles: vec![],
            iss: None,
            aud: None,
            exp: Some(0), // Already expired.
            iat: Some(0),
        };
        let token = make_token(&claims);
        assert!(config.verify(&token).is_err());
    }

    #[test]
    fn wrong_secret_rejected() {
        let claims = JwtClaims {
            sub: "user-123".into(),
            tenant_id: None,
            name: None,
            email: None,
            roles: vec![],
            iss: None,
            aud: None,
            exp: Some(u64::MAX),
            iat: Some(0),
        };
        // Sign with a different secret.
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(b"wrong-secret-key-at-least-32-byt!"),
        )
        .unwrap();
        let config = test_config();
        assert!(config.verify(&token).is_err());
    }
}
