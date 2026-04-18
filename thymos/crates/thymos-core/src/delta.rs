//! Structured delta: the vocabulary of state mutation.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct StructuredDelta(pub Vec<DeltaOp>);

impl StructuredDelta {
    pub fn single(op: DeltaOp) -> Self {
        StructuredDelta(vec![op])
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "op")]
pub enum DeltaOp {
    /// New resource. Rejected if `(kind, id)` already exists.
    Create {
        kind: String,
        id: String,
        value: Value,
    },
    /// Replace value if current `version == expected_version`. Monotonically
    /// increments the stored version.
    Replace {
        kind: String,
        id: String,
        expected_version: u64,
        value: Value,
    },
    /// Mark a resource retracted. Subsequent reads return `RetractedSince(commit)`.
    Retract {
        kind: String,
        id: String,
        expected_version: u64,
        reason: String,
    },
}
