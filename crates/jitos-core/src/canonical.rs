//! Canonical encoding for deterministic serialization.
//!
//! All ledger events, receipts, snapshots, deltas, and archives MUST use canonical encoding.
//! Non-canonical encoders (including serde_json) are FORBIDDEN in jitos-provenance.
//!
//! ## Canonical Rules (RFC 8949)
//!
//! 1. Map entries sorted by lexicographic ordering of canonical CBOR encoding of keys
//! 2. Definite-length encoding only (no streaming/indefinite lengths)
//! 3. Float canonicalization:
//!    - NaN → IEEE-754 quiet NaN with payload=0 (bit pattern: 0x7FF8_0000_0000_0000)
//!    - ±0 normalized to +0
//!    - ±∞ preserved as-is
//! 4. No duplicate map keys
//!
//! ## Safety
//!
//! Use `hash_canonical()` instead of manual hashing to prevent accidental non-canonical hashing.

use serde::{Deserialize, Serialize};

/// Error type for canonical encoding operations.
#[derive(thiserror::Error, Debug)]
pub enum CanonicalError {
    #[error("Serialization failed: {0}")]
    SerializationFailed(#[from] ciborium::ser::Error<std::io::Error>),

    #[error("Deserialization failed: {0}")]
    DeserializationFailed(#[from] ciborium::de::Error<std::io::Error>),

    #[error("Duplicate map key detected")]
    DuplicateKey,
}

/// Encode a value using canonical CBOR.
///
/// This is the ONLY valid way to serialize data for the ledger.
/// All other serialization methods are forbidden for determinism-critical data.
pub fn encode<T: Serialize>(_value: &T) -> Result<Vec<u8>, CanonicalError> {
    // TODO: Implement canonical CBOR encoding
    unimplemented!("canonical::encode not yet implemented")
}

/// Decode a value from canonical CBOR.
pub fn decode<T: for<'de> Deserialize<'de>>(_bytes: &[u8]) -> Result<T, CanonicalError> {
    // TODO: Implement canonical CBOR decoding with duplicate key detection
    unimplemented!("canonical::decode not yet implemented")
}

/// Hash a value using canonical encoding.
///
/// This is the ONLY valid way to hash data for determinism.
/// Never call `blake3::hash()` directly on serialized data.
pub fn hash_canonical<T: Serialize>(_value: &T) -> Result<crate::Hash, CanonicalError> {
    // TODO: Implement canonical hashing
    unimplemented!("canonical::hash_canonical not yet implemented")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        let value = 42u64;
        let bytes = encode(&value).unwrap();
        let decoded: u64 = decode(&bytes).unwrap();
        assert_eq!(decoded, value);
    }
}
