//! Typed, newtype-wrapped identifiers.
//!
//! `CommitId`, `IntentId`, `ProposalId` are content-addressed (ContentHash).
//! `TrajectoryId`, `WritId` are random 32-byte opaque ids (generated at mint).

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::hash::ContentHash;

macro_rules! content_id {
    ($name:ident, $tag:literal) => {
        #[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub ContentHash);

        impl $name {
            pub const ZERO: Self = $name(ContentHash::ZERO);
            pub fn inner(&self) -> &ContentHash { &self.0 }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({})", $tag, self.0.short())
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}:{}", $tag, self.0)
            }
        }
    };
}

content_id!(CommitId, "commit");
content_id!(IntentId, "intent");
content_id!(ProposalId, "proposal");

/// Random 32-byte opaque id.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TrajectoryId(pub ContentHash);

impl TrajectoryId {
    pub fn new_from_seed(seed: &[u8]) -> Self {
        TrajectoryId(ContentHash(*blake3::hash(seed).as_bytes()))
    }
}

impl fmt::Debug for TrajectoryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "traj({})", self.0.short())
    }
}
impl fmt::Display for TrajectoryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "traj:{}", self.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WritId(pub ContentHash);

impl WritId {
    pub fn new_from_seed(seed: &[u8]) -> Self {
        WritId(ContentHash(*blake3::hash(seed).as_bytes()))
    }
}

impl fmt::Debug for WritId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "writ({})", self.0.short())
    }
}
impl fmt::Display for WritId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "writ:{}", self.0)
    }
}
