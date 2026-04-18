//! ed25519 primitives + serde helpers for signed writs.
//!
//! Public keys and signatures are serialized as lowercase hex strings in
//! canonical JSON so that the digest over a `WritBody` is deterministic and
//! doesn't depend on serde_json's byte-array encoding.

use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Deserializer, Serializer};

use crate::error::{Error, Result};

pub type PublicKey = [u8; 32];
pub type SignatureBytes = [u8; 64];

/// Generate a fresh ed25519 keypair using the OS RNG.
pub fn generate_signing_key() -> SigningKey {
    SigningKey::generate(&mut OsRng)
}

/// Extract the verifying (public) key bytes from a signing key.
pub fn public_key_of(sk: &SigningKey) -> PublicKey {
    sk.verifying_key().to_bytes()
}

/// Sign `message` with `sk` and return the raw 64-byte signature.
pub fn sign(sk: &SigningKey, message: &[u8]) -> SignatureBytes {
    sk.sign(message).to_bytes()
}

/// Verify a detached ed25519 signature. Returns `AuthorityVoid` on failure.
pub fn verify(pk: &PublicKey, message: &[u8], sig: &SignatureBytes) -> Result<()> {
    let vk = VerifyingKey::from_bytes(pk)
        .map_err(|e| Error::AuthorityVoid(format!("bad pubkey: {e}")))?;
    let signature = ed25519_dalek::Signature::from_bytes(sig);
    vk.verify_strict(message, &signature)
        .map_err(|e| Error::AuthorityVoid(format!("bad signature: {e}")))
}

// ---------- serde helpers --------------------------------------------------

pub mod hex32 {
    use super::*;

    pub fn serialize<S: Serializer>(bytes: &[u8; 32], s: S) -> std::result::Result<S::Ok, S::Error> {
        s.serialize_str(&hex::encode(bytes))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        d: D,
    ) -> std::result::Result<[u8; 32], D::Error> {
        let s = String::deserialize(d)?;
        let v = hex::decode(&s).map_err(serde::de::Error::custom)?;
        v.try_into()
            .map_err(|_| serde::de::Error::custom("expected 32 bytes"))
    }
}

pub mod hex64 {
    use super::*;

    pub fn serialize<S: Serializer>(bytes: &[u8; 64], s: S) -> std::result::Result<S::Ok, S::Error> {
        s.serialize_str(&hex::encode(bytes))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        d: D,
    ) -> std::result::Result<[u8; 64], D::Error> {
        let s = String::deserialize(d)?;
        let v = hex::decode(&s).map_err(serde::de::Error::custom)?;
        v.try_into()
            .map_err(|_| serde::de::Error::custom("expected 64 bytes"))
    }
}
