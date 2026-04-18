//! Thymos core types.
//!
//! Invariants enforced at the type level where possible:
//!   * `Commit.id == blake3(canonical_json(commit_without_id_or_sig))`.
//!   * A commit's `parent` is either the trajectory root marker or a prior commit id.
//!   * An `Intent` is inert: it never mutates state; only a `Commit` appended to
//!     the ledger mutates state (via its `StructuredDelta`).
//!   * A `Proposal` is the only thing the scheduler may execute.
//!   * Every mutation is authorized by exactly one `Writ`.

pub mod crypto;
pub mod hash;
pub mod ids;
pub mod writ;
pub mod intent;
pub mod proposal;
pub mod commit;
pub mod delta;
pub mod world;
pub mod error;

pub use error::{Error, Result};
pub use hash::{canonical_json_bytes, content_hash, ContentHash};
pub use ids::{CommitId, IntentId, ProposalId, TrajectoryId, WritId};
pub use writ::{Budget, EffectCeiling, ToolPattern, Writ, WritBody};
pub use intent::{Intent, IntentBody, IntentKind};
pub use proposal::{
    ExecutionPlan, PolicyTrace, Proposal, ProposalBody, ProposalStatus, RejectionReason,
};
pub use commit::{Commit, CommitBody};
pub use delta::{DeltaOp, StructuredDelta};
pub use world::{ResourceKey, World};

/// The compiler / protocol version stamped into every commit for replay fidelity.
pub const COMPILER_VERSION: &str = "thymos-compiler/0.0.1";
