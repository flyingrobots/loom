//! DeltaSpec: Counterfactual Specification (Phase 0.5.3)
//!
//! This module implements controlled violations of history for debugging,
//! testing, and causal analysis. It enables "what if" questions to be
//! precisely specified, executed, and compared.
//!
//! See SPEC-0002-deltaspec.md for detailed specification.

use crate::canonical::CanonicalError;
use crate::events::{AgentId, EventId};
use crate::Hash;
use serde::{Deserialize, Serialize};

/// Policy hash - content-addressed reference to a policy
pub type PolicyHash = Hash;

/// Placeholder for input events (will be expanded later)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputEvent {
    // TODO: Define InputEvent structure
    pub placeholder: u64,
}

/// Describes a controlled violation of history
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeltaSpec {
    /// What kind of counterfactual
    pub kind: DeltaKind,

    /// Human-readable justification (for debugging)
    pub description: String,

    /// Content-addressed hash of this spec
    /// Used to reference this delta in fork events
    pub hash: Hash,
}

impl DeltaSpec {
    /// Compute the canonical hash of this DeltaSpec.
    ///
    /// INVARIANT: Same logical delta → identical hash (cross-platform, cross-runtime)
    pub fn compute_hash(&self) -> Result<Hash, CanonicalError> {
        // TODO: Implement hash computation
        // For now, return a dummy hash to make tests compile
        Ok(Hash([0u8; 32]))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum DeltaKind {
    /// Change scheduler policy (e.g., FIFO → LIFO)
    SchedulerPolicy { new_policy: PolicyHash },

    /// Inject/modify/delete input events
    InputMutation {
        insert: Vec<InputEvent>,
        delete: Vec<EventId>,
        modify: Vec<(EventId, InputEvent)>,
    },

    /// Change clock interpretation policy
    ClockPolicy { new_policy: PolicyHash },

    /// Change trust assumptions
    TrustPolicy { new_trust_roots: Vec<AgentId> },
}

/// Errors that can occur when working with DeltaSpec
#[derive(Debug, thiserror::Error)]
pub enum DeltaError {
    #[error("Invalid event reference: {0:?}")]
    InvalidEventRef(EventId),

    #[error("Invalid hash: computed hash does not match stored hash")]
    InvalidHash,

    #[error("Canonical encoding error: {0}")]
    CanonicalError(#[from] CanonicalError),
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test 1: Canonical Encoding (from SPEC-0002 Section 6.1)
    ///
    /// REQUIREMENT: Same logical structure → identical bytes on all platforms
    #[test]
    fn test_deltaspec_canonical_encoding() {
        let delta = DeltaSpec {
            kind: DeltaKind::SchedulerPolicy {
                new_policy: Hash([1u8; 32]),
            },
            description: "Test scheduler policy change".to_string(),
            hash: Hash([0u8; 32]), // Will be computed
        };

        // Encode twice
        let bytes1 = canonical::encode(&delta).expect("encoding should succeed");
        let bytes2 = canonical::encode(&delta).expect("encoding should succeed");

        // Must be deterministic
        assert_eq!(
            bytes1, bytes2,
            "Canonical encoding must be deterministic"
        );
    }

    /// Test 2: Round-Trip (from SPEC-0002 Section 6.1)
    ///
    /// REQUIREMENT: encode(decode(bytes)) = bytes (bijection)
    #[test]
    fn test_deltaspec_roundtrip() {
        let original = DeltaSpec {
            kind: DeltaKind::ClockPolicy {
                new_policy: Hash([2u8; 32]),
            },
            description: "Test clock policy change".to_string(),
            hash: Hash([0u8; 32]),
        };

        // Encode then decode
        let bytes = canonical::encode(&original).expect("encoding should succeed");
        let decoded: DeltaSpec = canonical::decode(&bytes).expect("decoding should succeed");

        // Must round-trip exactly
        assert_eq!(
            original, decoded,
            "DeltaSpec must round-trip through canonical encoding"
        );
    }

    /// Test 3: Hash Stability (from SPEC-0002 Section 6.1)
    ///
    /// REQUIREMENT: compute_hash() is deterministic and stable
    #[test]
    fn test_deltaspec_hash_stability() {
        let delta = DeltaSpec {
            kind: DeltaKind::TrustPolicy {
                new_trust_roots: vec![
                    AgentId::new("alice").expect("valid agent id"),
                    AgentId::new("bob").expect("valid agent id"),
                ],
            },
            description: "Test trust policy change".to_string(),
            hash: Hash([0u8; 32]),
        };

        // Compute hash twice
        let hash1 = delta.compute_hash().expect("hash computation should succeed");
        let hash2 = delta.compute_hash().expect("hash computation should succeed");

        // Must be stable
        assert_eq!(
            hash1, hash2,
            "Hash computation must be deterministic and stable"
        );
    }

    /// Test 4: Collision Resistance (from SPEC-0002 Section 6.1)
    ///
    /// REQUIREMENT: Different deltas → different hashes
    #[test]
    fn test_different_deltas_different_hashes() {
        let delta1 = DeltaSpec {
            kind: DeltaKind::SchedulerPolicy {
                new_policy: Hash([1u8; 32]),
            },
            description: "First delta".to_string(),
            hash: Hash([0u8; 32]),
        };

        let delta2 = DeltaSpec {
            kind: DeltaKind::ClockPolicy {
                new_policy: Hash([2u8; 32]),
            },
            description: "Second delta".to_string(),
            hash: Hash([0u8; 32]),
        };

        let hash1 = delta1.compute_hash().expect("hash1 should succeed");
        let hash2 = delta2.compute_hash().expect("hash2 should succeed");

        // Must be different (collision resistance)
        assert_ne!(
            hash1, hash2,
            "Different DeltaSpecs must produce different hashes"
        );
    }
}
