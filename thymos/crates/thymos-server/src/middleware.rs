//! Middleware: API key authentication and rate limiting.
//!
//! - **API key auth**: reads `Authorization: Bearer <key>` and validates
//!   against a set of known keys (in-memory for Phase 1; Postgres in prod).
//! - **Rate limiting**: token-bucket per API key, configurable per key tier.

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Json},
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// API key record.
#[derive(Clone, Debug)]
pub struct ApiKey {
    pub key: String,
    pub tenant_id: String,
    pub name: String,
    /// Max requests per minute.
    pub rate_limit_rpm: u32,
}

/// In-memory API key store + rate limiter.
pub struct ApiGateway {
    keys: HashMap<String, ApiKey>,
    /// Tracks (key -> (count, window_start)).
    rate_state: Mutex<HashMap<String, (u32, Instant)>>,
}

impl ApiGateway {
    pub fn new() -> Self {
        ApiGateway {
            keys: HashMap::new(),
            rate_state: Mutex::new(HashMap::new()),
        }
    }

    /// Register an API key.
    pub fn add_key(&mut self, key: ApiKey) {
        self.keys.insert(key.key.clone(), key);
    }

    /// Validate a key and check rate limits. Returns the key record on success.
    pub fn authenticate(&self, bearer: &str) -> Result<&ApiKey, GatewayError> {
        let key = self
            .keys
            .get(bearer)
            .ok_or(GatewayError::InvalidKey)?;

        // Rate limit check.
        let mut state = self.rate_state.lock().unwrap();
        let entry = state
            .entry(bearer.to_string())
            .or_insert((0, Instant::now()));

        // Reset window if more than 60 seconds have passed.
        if entry.1.elapsed().as_secs() >= 60 {
            *entry = (0, Instant::now());
        }

        if entry.0 >= key.rate_limit_rpm {
            return Err(GatewayError::RateLimited);
        }

        entry.0 += 1;
        Ok(key)
    }
}

/// Per-key usage stats snapshot.
#[derive(Clone, Debug, serde::Serialize)]
pub struct KeyUsageStats {
    pub key_name: String,
    pub tenant_id: String,
    pub requests_this_window: u32,
    pub rate_limit_rpm: u32,
    pub window_started_secs_ago: u64,
}

impl ApiGateway {
    /// Return usage stats for all known keys (for a dashboard).
    pub fn usage_stats(&self) -> Vec<KeyUsageStats> {
        let state = self.rate_state.lock().unwrap();
        self.keys
            .values()
            .map(|k| {
                let (count, elapsed) = state
                    .get(&k.key)
                    .map(|(c, t)| (*c, t.elapsed().as_secs()))
                    .unwrap_or((0, 0));
                KeyUsageStats {
                    key_name: k.name.clone(),
                    tenant_id: k.tenant_id.clone(),
                    requests_this_window: count,
                    rate_limit_rpm: k.rate_limit_rpm,
                    window_started_secs_ago: elapsed,
                }
            })
            .collect()
    }
}

impl ApiGateway {
    /// Seconds until the current rate-limit window resets for a given key.
    pub fn retry_after(&self, bearer: &str) -> u64 {
        let state = self.rate_state.lock().unwrap();
        match state.get(bearer) {
            Some((_count, started)) => {
                let elapsed = started.elapsed().as_secs();
                if elapsed >= 60 { 0 } else { 60 - elapsed }
            }
            None => 0,
        }
    }
}

impl Default for ApiGateway {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub enum GatewayError {
    InvalidKey,
    RateLimited,
    MissingAuth,
}

/// Axum middleware layer for API key auth + rate limiting.
///
/// Usage in router:
/// ```ignore
/// let gateway = Arc::new(api_gateway);
/// app.layer(axum::middleware::from_fn_with_state(
///     gateway.clone(),
///     api_key_middleware,
/// ))
/// ```
pub async fn api_key_middleware(
    axum::extract::State(gateway): axum::extract::State<Arc<ApiGateway>>,
    request: Request,
    next: Next,
) -> impl IntoResponse {
    // Skip auth for health check.
    if request.uri().path() == "/health" {
        return next.run(request).await.into_response();
    }

    let auth_header = request
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok());

    let bearer = match auth_header {
        Some(h) if h.starts_with("Bearer ") => &h[7..],
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "missing or invalid Authorization header" })),
            )
                .into_response()
        }
    };

    match gateway.authenticate(bearer) {
        Ok(key) => {
            // Inject tenant_id and key name into request extensions for downstream use.
            let mut request = request;
            request.extensions_mut().insert(GatewayContext {
                tenant_id: key.tenant_id.clone(),
                key_name: key.name.clone(),
            });
            next.run(request).await.into_response()
        }
        Err(GatewayError::InvalidKey) => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "invalid API key" })),
        )
            .into_response(),
        Err(GatewayError::RateLimited) => {
            // Compute seconds remaining in the current window.
            let retry_after = gateway.retry_after(bearer);
            (
                StatusCode::TOO_MANY_REQUESTS,
                [("retry-after", retry_after.to_string())],
                Json(serde_json::json!({
                    "error": "rate limit exceeded",
                    "retry_after_secs": retry_after,
                })),
            )
                .into_response()
        }
        Err(GatewayError::MissingAuth) => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "error": "missing auth" })),
        )
            .into_response(),
    }
}

/// Context injected into request extensions after successful gateway auth.
#[derive(Clone, Debug)]
pub struct GatewayContext {
    pub tenant_id: String,
    pub key_name: String,
}
