//! Commit: the only thing that mutates world state. Appended to the ledger.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::Result;
use crate::hash::content_hash;
use crate::ids::{CommitId, ProposalId, TrajectoryId, WritId};

/// An observation captured from a tool execution. Persisted verbatim.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Observation {
    pub tool: String,
    pub output: Value,
    pub latency_ms: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Commit {
    pub id: CommitId,
    pub body: CommitBody,
}

impl Commit {
    pub fn new(body: CommitBody) -> Result<Self> {
        let id = CommitId(content_hash(&body)?);
        Ok(Commit { id, body })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommitBody {
    /// Empty for the root commit of a trajectory; one for linear extension;
    /// multiple for merges.
    pub parent: Vec<CommitId>,
    pub trajectory_id: TrajectoryId,
    pub proposal_id: ProposalId,
    pub writ_id: WritId,
    /// Logical Lamport-style clock, monotonically increasing within a trajectory.
    pub seq: u64,
    pub delta: crate::delta::StructuredDelta,
    pub observations: Vec<Observation>,
    pub compiler_version: String,
    /// Budget cost incurred by this commit (tool_calls, tokens, wall_clock_ms, usd).
    pub budget_cost: crate::writ::BudgetCost,
    /// Reserved for ed25519 signature over canonical_json(body_without_signature).
    pub signature: Option<String>,
}
