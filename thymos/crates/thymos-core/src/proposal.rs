//! Proposal: the compiler's output. The only thing the scheduler executes.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::Result;
use crate::hash::content_hash;
use crate::ids::{IntentId, ProposalId, WritId};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Proposal {
    pub id: ProposalId,
    pub body: ProposalBody,
}

impl Proposal {
    pub fn new(body: ProposalBody) -> Result<Self> {
        let id = ProposalId(content_hash(&body)?);
        Ok(Proposal { id, body })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProposalBody {
    pub intent_id: IntentId,
    pub writ_id: WritId,
    pub plan: ExecutionPlan,
    pub policy_trace: PolicyTrace,
    pub status: ProposalStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub tool: String,
    /// Validated input to the tool contract (already schema-checked by the compiler).
    pub args: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PolicyTrace {
    pub rules_evaluated: Vec<String>,
    pub decision: PolicyDecision,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "kind", content = "detail")]
pub enum PolicyDecision {
    Permit,
    Deny(String),
    RequireApproval { channel: String, reason: String },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProposalStatus {
    Staged,
    Rejected,
    SuspendedForApproval,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "detail")]
pub enum RejectionReason {
    AuthorityVoid(String),
    PolicyDenied(String),
    BudgetExhausted(String),
    PreconditionFailed(String),
    UnknownTool(String),
    TypeMismatch { tool: String, detail: String },
}
