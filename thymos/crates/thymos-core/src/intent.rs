//! Intent: the only thing cognition emits. Never executed directly.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::Result;
use crate::hash::content_hash;
use crate::ids::{CommitId, IntentId};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Intent {
    pub id: IntentId,
    pub body: IntentBody,
}

impl Intent {
    pub fn new(body: IntentBody) -> Result<Self> {
        let id = IntentId(content_hash(&body)?);
        Ok(Intent { id, body })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IntentBody {
    pub parent_commit: Option<CommitId>,
    pub author: String,
    pub kind: IntentKind,
    /// Tool name for `Act`, resource key for `Query`, target agent for `Delegate`.
    pub target: String,
    pub args: Value,
    pub rationale: String,
    /// 16 random bytes to make otherwise-identical intents distinct by content.
    pub nonce: [u8; 16],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IntentKind {
    Plan,
    Act,
    Query,
    Delegate,
    MemoryPromote,
    Reconcile,
}
