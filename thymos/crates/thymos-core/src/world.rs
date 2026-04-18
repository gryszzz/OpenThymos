//! In-memory world projection.
//!
//! The world is not authoritative; the ledger is. This is a fold of the ledger
//! up to a given head. For Phase 1 we recompute eagerly; later phases will
//! cache projections and snapshot.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

use crate::delta::{DeltaOp, StructuredDelta};
use crate::error::{Error, Result};
use crate::ids::CommitId;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ResourceKey {
    pub kind: String,
    pub id: String,
}

impl ResourceKey {
    pub fn new(kind: impl Into<String>, id: impl Into<String>) -> Self {
        ResourceKey {
            kind: kind.into(),
            id: id.into(),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct World {
    pub resources: BTreeMap<ResourceKey, ResourceState>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceState {
    pub version: u64,
    pub value: Value,
    pub retracted: bool,
    pub last_commit: CommitId,
}

impl World {
    pub fn apply(&mut self, delta: &StructuredDelta, commit: CommitId) -> Result<()> {
        for op in &delta.0 {
            match op {
                DeltaOp::Create { kind, id, value } => {
                    let key = ResourceKey::new(kind.clone(), id.clone());
                    if self.resources.contains_key(&key) {
                        return Err(Error::Invariant(format!(
                            "create collided on existing resource {}:{}",
                            kind, id
                        )));
                    }
                    self.resources.insert(
                        key,
                        ResourceState {
                            version: 1,
                            value: value.clone(),
                            retracted: false,
                            last_commit: commit,
                        },
                    );
                }
                DeltaOp::Replace {
                    kind,
                    id,
                    expected_version,
                    value,
                } => {
                    let key = ResourceKey::new(kind.clone(), id.clone());
                    let state = self.resources.get_mut(&key).ok_or_else(|| {
                        Error::Invariant(format!("replace on missing {}:{}", kind, id))
                    })?;
                    if state.version != *expected_version {
                        return Err(Error::CasConflict {
                            kind: kind.clone(),
                            id: id.clone(),
                            expected: *expected_version,
                            found: state.version,
                        });
                    }
                    state.version += 1;
                    state.value = value.clone();
                    state.last_commit = commit;
                }
                DeltaOp::Retract {
                    kind,
                    id,
                    expected_version,
                    reason: _,
                } => {
                    let key = ResourceKey::new(kind.clone(), id.clone());
                    let state = self.resources.get_mut(&key).ok_or_else(|| {
                        Error::Invariant(format!("retract on missing {}:{}", kind, id))
                    })?;
                    if state.version != *expected_version {
                        return Err(Error::CasConflict {
                            kind: kind.clone(),
                            id: id.clone(),
                            expected: *expected_version,
                            found: state.version,
                        });
                    }
                    state.version += 1;
                    state.retracted = true;
                    state.last_commit = commit;
                }
            }
        }
        Ok(())
    }

    pub fn get(&self, key: &ResourceKey) -> Option<&ResourceState> {
        self.resources.get(key).filter(|s| !s.retracted)
    }
}
