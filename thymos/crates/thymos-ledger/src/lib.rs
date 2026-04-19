//! Thymos Ledger: append-only, content-addressed, parent-chained storage.
//!
//! Supports two backends:
//!   - **SQLite** (default, feature `sqlite`) — single-process, zero-config
//!   - **Postgres** (feature `postgres`) — multi-node, production-grade
//!
//! Both backends share the same entry/payload types and integrity guarantees:
//!   * Append-only — rows are never updated
//!   * Content-addressed — `id = blake3(canonical_json(payload))`
//!   * Parent-chained — every non-root entry references its parent
//!   * Typed kinds: Root, Commit, Rejection, PendingApproval, Delegation, Branch

use serde::{Deserialize, Serialize};

use thymos_core::{
    commit::Commit,
    content_hash,
    ids::IntentId,
    proposal::{Proposal, RejectionReason},
    CommitId, ContentHash, Error, Result, TrajectoryId,
};

// Backend modules.
#[cfg(feature = "postgres")]
pub mod postgres;
#[cfg(feature = "sqlite")]
pub mod sqlite;

// Re-export the default backend as `Ledger`.
#[cfg(feature = "sqlite")]
pub use sqlite::SqliteLedger as Ledger;

/// A typed ledger entry.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entry {
    pub id: ContentHash,
    pub trajectory_id: TrajectoryId,
    pub parent: Option<ContentHash>,
    pub seq: u64,
    pub kind: EntryKind,
    pub payload: EntryPayload,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryKind {
    Root,
    Commit,
    Rejection,
    PendingApproval,
    Delegation,
    Branch,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EntryPayload {
    Root {
        note: String,
    },
    Commit(Commit),
    Rejection {
        intent_id: IntentId,
        reason: RejectionReason,
    },
    PendingApproval {
        proposal: Proposal,
        channel: String,
        reason: String,
    },
    Delegation {
        child_trajectory_id: TrajectoryId,
        task: String,
        final_answer: Option<String>,
    },
    Branch {
        source_trajectory_id: TrajectoryId,
        source_commit_id: CommitId,
        note: String,
    },
}

// ---- Shared helpers used by both backends ----

pub(crate) fn build_entry(
    trajectory_id: TrajectoryId,
    parent: Option<ContentHash>,
    seq: u64,
    kind: EntryKind,
    payload: EntryPayload,
) -> Result<Entry> {
    let id = content_hash(&payload)?;
    Ok(Entry {
        id,
        trajectory_id,
        parent,
        seq,
        kind,
        payload,
    })
}

pub(crate) fn kind_to_str(kind: EntryKind) -> &'static str {
    match kind {
        EntryKind::Root => "root",
        EntryKind::Commit => "commit",
        EntryKind::Rejection => "rejection",
        EntryKind::PendingApproval => "pending_approval",
        EntryKind::Delegation => "delegation",
        EntryKind::Branch => "branch",
    }
}

pub(crate) fn str_to_kind(s: &str) -> Result<EntryKind> {
    match s {
        "root" => Ok(EntryKind::Root),
        "commit" => Ok(EntryKind::Commit),
        "rejection" => Ok(EntryKind::Rejection),
        "pending_approval" => Ok(EntryKind::PendingApproval),
        "delegation" => Ok(EntryKind::Delegation),
        "branch" => Ok(EntryKind::Branch),
        other => Err(Error::Ledger(format!("unknown entry kind: {other}"))),
    }
}

/// Verify integrity of a sequence of entries (used by both backends).
pub(crate) fn verify_integrity_entries(entries: &[Entry]) -> Result<()> {
    let mut prev_seq: Option<u64> = None;
    let mut prev_id: Option<ContentHash> = None;
    for e in entries {
        let recomputed = content_hash(&e.payload)?;
        if e.id != recomputed {
            return Err(Error::Invariant(format!(
                "hash mismatch at seq {}: claimed {} vs recomputed {}",
                e.seq, e.id, recomputed
            )));
        }
        if let (Some(ps), Some(pid)) = (prev_seq, prev_id) {
            if e.seq != ps + 1 {
                return Err(Error::Invariant(format!(
                    "non-contiguous seq: {} after {}",
                    e.seq, ps
                )));
            }
            if e.parent != Some(pid) {
                return Err(Error::Invariant("parent mismatch".into()));
            }
        }
        prev_seq = Some(e.seq);
        prev_id = Some(e.id);
    }
    Ok(())
}

/// Extension helper: pull every Commit from a trajectory for projection.
pub fn project_commits(entries: &[Entry]) -> Vec<&Commit> {
    entries
        .iter()
        .filter_map(|e| match &e.payload {
            EntryPayload::Commit(c) => Some(c),
            _ => None,
        })
        .collect()
}

/// A flattened audit-friendly entry with hex IDs and a timestamp.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub trajectory_id: String,
    pub seq: u64,
    pub kind: String,
    pub payload: EntryPayload,
    pub created_at: u64,
}

impl Entry {
    pub fn commit_id(&self) -> Option<CommitId> {
        match &self.payload {
            EntryPayload::Commit(c) => Some(c.id),
            _ => None,
        }
    }
}

#[cfg(all(test, feature = "sqlite"))]
mod tests {
    use super::*;
    use thymos_core::{
        commit::{Commit, CommitBody, Observation},
        delta::{DeltaOp, StructuredDelta},
        ids::{ProposalId, WritId},
        COMPILER_VERSION,
    };

    fn trivial_commit(traj: TrajectoryId, parent: Option<CommitId>, seq: u64) -> Commit {
        let body = CommitBody {
            parent: parent.into_iter().collect(),
            trajectory_id: traj,
            proposal_id: ProposalId::ZERO,
            writ_id: WritId(ContentHash::ZERO),
            seq,
            delta: StructuredDelta::single(DeltaOp::Create {
                kind: "kv".into(),
                id: "foo".into(),
                value: serde_json::json!("bar"),
            }),
            observations: vec![Observation {
                tool: "kv_set".into(),
                output: serde_json::json!(null),
                latency_ms: 1,
            }],
            compiler_version: COMPILER_VERSION.into(),
            budget_cost: thymos_core::writ::BudgetCost::default(),
            signature: None,
        };
        Commit::new(body).unwrap()
    }

    #[test]
    fn root_and_commit_append() {
        let l = Ledger::open_in_memory().unwrap();
        let traj = TrajectoryId::new_from_seed(b"t1");
        let root = l.append_root(traj, "hello").unwrap();
        assert_eq!(root.seq, 0);

        let c1 = trivial_commit(traj, Some(CommitId(root.id)), 1);
        let e = l.append_commit(c1).unwrap();
        assert_eq!(e.seq, 1);

        l.verify_integrity(traj).unwrap();
    }

    #[test]
    fn determinism_same_inputs_same_id() {
        let l1 = Ledger::open_in_memory().unwrap();
        let l2 = Ledger::open_in_memory().unwrap();
        let traj = TrajectoryId::new_from_seed(b"det");
        let r1 = l1.append_root(traj, "x").unwrap();
        let r2 = l2.append_root(traj, "x").unwrap();
        assert_eq!(r1.id, r2.id);
        let c1 = trivial_commit(traj, Some(CommitId(r1.id)), 1);
        let c2 = trivial_commit(traj, Some(CommitId(r2.id)), 1);
        assert_eq!(c1.id, c2.id);
    }

    #[test]
    fn rejects_out_of_order_seq() {
        let l = Ledger::open_in_memory().unwrap();
        let traj = TrajectoryId::new_from_seed(b"t2");
        let root = l.append_root(traj, "hello").unwrap();
        let c = trivial_commit(traj, Some(CommitId(root.id)), 5);
        assert!(l.append_commit(c).is_err());
    }
}
