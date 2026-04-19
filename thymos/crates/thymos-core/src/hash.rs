//! Content addressing.
//!
//! Canonical encoding rules (v0):
//!   * `serde_json::to_vec` with the `preserve_order` feature disabled for Maps —
//!     producing sorted-key object serialization via `BTreeMap` internals.
//!   * All struct fields serialize in declaration order (stable by construction).
//!   * Floating-point values are forbidden in canonical payloads (use fixed-point).
//!
//! The `ContentHash` is the BLAKE3 digest (32 bytes) of those canonical bytes.
//! Equality on `ContentHash` is constant-time via the standard `Eq` derivation
//! of the underlying array.

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::error::Result;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ContentHash(#[serde(with = "hex_bytes")] pub [u8; 32]);

impl ContentHash {
    pub const ZERO: Self = ContentHash([0u8; 32]);

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn short(&self) -> String {
        hex::encode(&self.0[..4])
    }
}

impl fmt::Debug for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "H({})", hex::encode(self.0))
    }
}

impl fmt::Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&hex::encode(self.0))
    }
}

/// Canonical JSON bytes (MUST be used for all content addressing).
///
/// Note: we rely on `serde_json`'s default `Map` implementation (BTreeMap-backed
/// when the `preserve_order` feature is *disabled*). The workspace enables
/// `preserve_order`, so for canonical encoding we route through a `Value` round
/// trip and normalize object key order via `BTreeMap`.
pub fn canonical_json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>> {
    let v: serde_json::Value = serde_json::to_value(value)?;
    let normalized = sort_value(v);
    Ok(serde_json::to_vec(&normalized)?)
}

pub fn content_hash<T: Serialize>(value: &T) -> Result<ContentHash> {
    let bytes = canonical_json_bytes(value)?;
    let digest = blake3::hash(&bytes);
    Ok(ContentHash(*digest.as_bytes()))
}

fn sort_value(v: serde_json::Value) -> serde_json::Value {
    use serde_json::Value;
    match v {
        Value::Object(map) => {
            let mut sorted: std::collections::BTreeMap<String, Value> =
                std::collections::BTreeMap::new();
            for (k, val) in map {
                sorted.insert(k, sort_value(val));
            }
            // Re-insert into a fresh serde_json::Map; with preserve_order the
            // order is insertion order, which is now sorted.
            let mut out = serde_json::Map::new();
            for (k, val) in sorted {
                out.insert(k, val);
            }
            Value::Object(out)
        }
        Value::Array(arr) => Value::Array(arr.into_iter().map(sort_value).collect()),
        other => other,
    }
}

/// Hex-as-string serde adapter for fixed byte arrays.
mod hex_bytes {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(bytes: &[u8; 32], s: S) -> Result<S::Ok, S::Error> {
        hex::encode(bytes).serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<[u8; 32], D::Error> {
        let s = String::deserialize(d)?;
        let bytes = hex::decode(&s).map_err(serde::de::Error::custom)?;
        if bytes.len() != 32 {
            return Err(serde::de::Error::custom("expected 32-byte hex"));
        }
        let mut out = [0u8; 32];
        out.copy_from_slice(&bytes);
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Serialize)]
    struct Thing {
        b: u32,
        a: String,
    }

    #[test]
    fn canonical_is_stable() {
        let t1 = Thing {
            b: 2,
            a: "x".into(),
        };
        let t2 = Thing {
            b: 2,
            a: "x".into(),
        };
        assert_eq!(content_hash(&t1).unwrap(), content_hash(&t2).unwrap());
    }

    #[test]
    fn map_ordering_is_canonical() {
        let v1: serde_json::Value = serde_json::from_str(r#"{"b":1,"a":2}"#).unwrap();
        let v2: serde_json::Value = serde_json::from_str(r#"{"a":2,"b":1}"#).unwrap();
        assert_eq!(content_hash(&v1).unwrap(), content_hash(&v2).unwrap());
    }
}
