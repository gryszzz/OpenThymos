//! Thymos Client SDK — typed async HTTP client for the Thymos server API.
//!
//! # Example
//!
//! ```no_run
//! use thymos_client::ThymosClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), thymos_client::Error> {
//!     let client = ThymosClient::new("http://localhost:3001");
//!     let health = client.health().await?;
//!     println!("status: {}", health.status);
//!
//!     let run = client.create_run("Say hello", None).await?;
//!     println!("run_id: {}", run.run_id);
//!     Ok(())
//! }
//! ```

use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Client errors.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("API error ({status}): {message}")]
    Api { status: u16, message: String },
    #[error("{0}")]
    Other(String),
}

/// Typed async client for the Thymos HTTP API.
#[derive(Clone, Debug)]
pub struct ThymosClient {
    base_url: String,
    client: Client,
}

// ---- Request / Response types ----

#[derive(Debug, Serialize)]
pub struct CreateRunRequest {
    pub task: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_steps: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cognition: Option<CognitionConfig>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tool_scopes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CognitionConfig {
    pub provider: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRunResponse {
    pub run_id: String,
    pub task: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct HealthResponse {
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct RunResponse {
    pub trajectory_id: Option<String>,
    pub task: Option<String>,
    pub status: Option<String>,
    pub summary: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct WorldResponse {
    pub resources: Vec<ResourceDto>,
}

#[derive(Debug, Deserialize)]
pub struct ResourceDto {
    pub kind: String,
    pub id: String,
    pub version: u64,
    pub value: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct ApprovalRequest {
    pub approve: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proposal_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ApprovalResponse {
    pub run_id: String,
    pub channel: String,
    pub approved: bool,
}

#[derive(Debug, Deserialize)]
pub struct DelegationsResponse {
    pub delegations: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct AuditEntriesResponse {
    pub entries: Vec<serde_json::Value>,
    pub count: u64,
}

#[derive(Debug, Deserialize)]
pub struct AuditCountResponse {
    pub count: u64,
}

#[derive(Debug, Default)]
pub struct AuditQuery {
    pub run_id: Option<String>,
    pub kind: Option<String>,
    pub from: Option<u64>,
    pub to: Option<u64>,
    pub format: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct UsageResponse {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

impl ThymosClient {
    /// Create a new client pointing at the given base URL (e.g. `http://localhost:3001`).
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: Client::new(),
        }
    }

    /// Create a client with a pre-configured reqwest::Client (for custom headers, TLS, etc.).
    pub fn with_client(base_url: &str, client: Client) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client,
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    async fn check_error(&self, resp: reqwest::Response) -> Result<reqwest::Response, Error> {
        if resp.status().is_success() || resp.status().as_u16() == 202 {
            Ok(resp)
        } else {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            Err(Error::Api {
                status,
                message: body,
            })
        }
    }

    // ---- Endpoints ----

    /// GET /health
    pub async fn health(&self) -> Result<HealthResponse, Error> {
        let resp = self.client.get(self.url("/health")).send().await?;
        let resp = self.check_error(resp).await?;
        Ok(resp.json().await?)
    }

    /// POST /runs — create a new agent run.
    pub async fn create_run(
        &self,
        task: &str,
        cognition: Option<CognitionConfig>,
    ) -> Result<CreateRunResponse, Error> {
        let body = CreateRunRequest {
            task: task.to_string(),
            max_steps: None,
            cognition,
            tool_scopes: vec![],
        };
        let resp = self
            .client
            .post(self.url("/runs"))
            .json(&body)
            .send()
            .await?;
        let resp = self.check_error(resp).await?;
        Ok(resp.json().await?)
    }

    /// POST /runs with full options.
    pub async fn create_run_full(&self, req: CreateRunRequest) -> Result<CreateRunResponse, Error> {
        let resp = self
            .client
            .post(self.url("/runs"))
            .json(&req)
            .send()
            .await?;
        let resp = self.check_error(resp).await?;
        Ok(resp.json().await?)
    }

    /// GET /runs/:id
    pub async fn get_run(&self, run_id: &str) -> Result<RunResponse, Error> {
        let resp = self
            .client
            .get(self.url(&format!("/runs/{run_id}")))
            .send()
            .await?;
        let resp = self.check_error(resp).await?;
        Ok(resp.json().await?)
    }

    /// GET /runs/:id/world
    pub async fn get_world(&self, run_id: &str) -> Result<WorldResponse, Error> {
        let resp = self
            .client
            .get(self.url(&format!("/runs/{run_id}/world")))
            .send()
            .await?;
        let resp = self.check_error(resp).await?;
        Ok(resp.json().await?)
    }

    /// POST /runs/:id/approvals/:channel
    pub async fn approve(
        &self,
        run_id: &str,
        channel: &str,
        approve: bool,
    ) -> Result<ApprovalResponse, Error> {
        let body = ApprovalRequest {
            approve,
            proposal_id: None,
        };
        let resp = self
            .client
            .post(self.url(&format!("/runs/{run_id}/approvals/{channel}")))
            .json(&body)
            .send()
            .await?;
        let resp = self.check_error(resp).await?;
        Ok(resp.json().await?)
    }

    /// POST /runs/:id/resume
    pub async fn resume_run(
        &self,
        run_id: &str,
        task: &str,
        cognition: Option<CognitionConfig>,
    ) -> Result<serde_json::Value, Error> {
        let body = CreateRunRequest {
            task: task.to_string(),
            max_steps: None,
            cognition,
            tool_scopes: vec![],
        };
        let resp = self
            .client
            .post(self.url(&format!("/runs/{run_id}/resume")))
            .json(&body)
            .send()
            .await?;
        let resp = self.check_error(resp).await?;
        Ok(resp.json().await?)
    }

    /// GET /runs/:id/delegations
    pub async fn get_delegations(&self, run_id: &str) -> Result<DelegationsResponse, Error> {
        let resp = self
            .client
            .get(self.url(&format!("/runs/{run_id}/delegations")))
            .send()
            .await?;
        let resp = self.check_error(resp).await?;
        Ok(resp.json().await?)
    }

    /// GET /usage
    pub async fn usage(&self) -> Result<UsageResponse, Error> {
        let resp = self.client.get(self.url("/usage")).send().await?;
        let resp = self.check_error(resp).await?;
        Ok(resp.json().await?)
    }

    /// GET /audit/entries with optional filters.
    pub async fn audit_entries(&self, query: &AuditQuery) -> Result<AuditEntriesResponse, Error> {
        let mut params: Vec<(&str, String)> = Vec::new();
        if let Some(ref id) = query.run_id {
            params.push(("run_id", id.clone()));
        }
        if let Some(ref k) = query.kind {
            params.push(("kind", k.clone()));
        }
        if let Some(f) = query.from {
            params.push(("from", f.to_string()));
        }
        if let Some(t) = query.to {
            params.push(("to", t.to_string()));
        }
        if let Some(ref fmt) = query.format {
            params.push(("format", fmt.clone()));
        }
        if let Some(l) = query.limit {
            params.push(("limit", l.to_string()));
        }
        let resp = self
            .client
            .get(self.url("/audit/entries"))
            .query(&params)
            .send()
            .await?;
        let resp = self.check_error(resp).await?;
        Ok(resp.json().await?)
    }

    /// GET /audit/entries as raw CSV string.
    pub async fn audit_entries_csv(&self, query: &AuditQuery) -> Result<String, Error> {
        let mut params: Vec<(&str, String)> = vec![("format", "csv".into())];
        if let Some(ref id) = query.run_id {
            params.push(("run_id", id.clone()));
        }
        if let Some(ref k) = query.kind {
            params.push(("kind", k.clone()));
        }
        if let Some(f) = query.from {
            params.push(("from", f.to_string()));
        }
        if let Some(t) = query.to {
            params.push(("to", t.to_string()));
        }
        if let Some(l) = query.limit {
            params.push(("limit", l.to_string()));
        }
        let resp = self
            .client
            .get(self.url("/audit/entries"))
            .query(&params)
            .send()
            .await?;
        let resp = self.check_error(resp).await?;
        Ok(resp.text().await?)
    }

    /// GET /audit/entries/count
    pub async fn audit_count(&self, query: &AuditQuery) -> Result<AuditCountResponse, Error> {
        let mut params: Vec<(&str, String)> = Vec::new();
        if let Some(ref id) = query.run_id {
            params.push(("run_id", id.clone()));
        }
        if let Some(ref k) = query.kind {
            params.push(("kind", k.clone()));
        }
        if let Some(f) = query.from {
            params.push(("from", f.to_string()));
        }
        if let Some(t) = query.to {
            params.push(("to", t.to_string()));
        }
        let resp = self
            .client
            .get(self.url("/audit/entries/count"))
            .query(&params)
            .send()
            .await?;
        let resp = self.check_error(resp).await?;
        Ok(resp.json().await?)
    }

    /// Poll a run until it reaches a terminal status ("completed" or "failed").
    /// Returns the final RunResponse.
    pub async fn poll_run(
        &self,
        run_id: &str,
        interval_ms: u64,
        max_attempts: u32,
    ) -> Result<RunResponse, Error> {
        for _ in 0..max_attempts {
            let run = self.get_run(run_id).await?;
            match run.status.as_deref() {
                Some("completed") | Some("failed") => return Ok(run),
                _ => {
                    tokio::time::sleep(std::time::Duration::from_millis(interval_ms)).await;
                }
            }
        }
        Err(Error::Other(format!(
            "run {run_id} did not complete within {max_attempts} attempts"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_url_construction() {
        let c = ThymosClient::new("http://localhost:3001/");
        assert_eq!(c.url("/health"), "http://localhost:3001/health");
    }

    #[test]
    fn client_url_no_trailing_slash() {
        let c = ThymosClient::new("http://localhost:3001");
        assert_eq!(c.url("/runs"), "http://localhost:3001/runs");
    }
}
