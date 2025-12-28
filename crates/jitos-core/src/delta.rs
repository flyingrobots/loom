//! DeltaSpec: Counterfactual Specification (Phase 0.5.3)
//!
//! This module implements controlled violations of history for debugging,
//! testing, and causal analysis. It enables "what if" questions to be
//! precisely specified, executed, and compared.
//!
//! See SPEC-0002-deltaspec.md for detailed specification.

use crate::canonical::{self, CanonicalError};
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
///
/// # Security Note
///
/// The `hash` field IS validated on deserialization. Any DeltaSpec loaded from
/// CBOR or JSON that has a mismatched hash will be rejected with
/// `DeltaError::InvalidHash`. This prevents spoofed delta references in fork events.
///
/// Callers should still prefer constructor methods (`new_scheduler_policy`, etc.)
/// which guarantee correct hashes at construction time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DeltaSpec {
    /// What kind of counterfactual
    pub kind: DeltaKind,

    /// Human-readable justification (for debugging)
    pub description: String,

    /// Content-addressed hash of this spec
    /// Used to reference this delta in fork events
    ///
    /// INVARIANT: This field is NOT included in the hash computation.
    /// Hash is computed over (kind, description) only. See `compute_hash()`.
    ///
    /// IMPORTANT: This field is not validated during deserialization.
    /// Always use constructor methods to ensure hash correctness.
    hash: Hash,
}

impl DeltaSpec {
    /// Returns the content-addressed hash of this DeltaSpec.
    pub fn hash(&self) -> Hash {
        self.hash
    }

    /// Compute the canonical hash of this DeltaSpec.
    ///
    /// INVARIANT: Same logical delta → identical hash (cross-platform, cross-runtime)
    ///
    /// NOTE: We hash (kind, description) to avoid circularity with the hash field.
    /// This is the same pattern used in EventEnvelope.
    pub fn compute_hash(&self) -> Result<Hash, CanonicalError> {
        // Hash only (kind, description), NOT the hash field (circular dependency)
        let bytes = canonical::encode(&(&self.kind, &self.description))?;
        let hash_bytes = blake3::hash(&bytes);

        // Convert blake3::Hash to our Hash type
        Ok(Hash(*hash_bytes.as_bytes()))
    }

    /// Internal: compute hash and finalize construction
    fn finalize(mut self) -> Result<Self, CanonicalError> {
        self.hash = self.compute_hash()?;
        Ok(self)
    }

    /// Create a new DeltaSpec with scheduler policy change
    pub fn new_scheduler_policy(
        new_policy: PolicyHash,
        description: String,
    ) -> Result<Self, CanonicalError> {
        Self {
            kind: DeltaKind::SchedulerPolicy { new_policy },
            description,
            hash: Hash([0u8; 32]), // temp
        }
        .finalize()
    }

    /// Create a new DeltaSpec with clock policy change
    pub fn new_clock_policy(
        new_policy: PolicyHash,
        description: String,
    ) -> Result<Self, CanonicalError> {
        Self {
            kind: DeltaKind::ClockPolicy { new_policy },
            description,
            hash: Hash([0u8; 32]), // temp
        }
        .finalize()
    }

    /// Create a new DeltaSpec with trust policy change
    ///
    /// # Errors
    ///
    /// Returns `DeltaError::InvalidStructure` if `new_trust_roots` is empty.
    /// An empty trust root set means "trust no one" which is a catastrophic
    /// policy change that should be explicitly opted into (not accidental).
    pub fn new_trust_policy(
        new_trust_roots: Vec<AgentId>,
        description: String,
    ) -> Result<Self, DeltaError> {
        if new_trust_roots.is_empty() {
            return Err(DeltaError::InvalidStructure(
                "TrustPolicy cannot have empty new_trust_roots (would mean 'trust no one')".to_string()
            ));
        }

        Self {
            kind: DeltaKind::TrustPolicy { new_trust_roots },
            description,
            hash: Hash([0u8; 32]), // temp
        }
        .finalize()
        .map_err(DeltaError::from)
    }

    /// Create a new DeltaSpec with input mutation
    pub fn new_input_mutation(
        insert: Vec<InputEvent>,
        delete: Vec<EventId>,
        modify: Vec<(EventId, InputEvent)>,
        description: String,
    ) -> Result<Self, CanonicalError> {
        Self {
            kind: DeltaKind::InputMutation { insert, delete, modify },
            description,
            hash: Hash([0u8; 32]), // temp
        }
        .finalize()
    }
}

// Custom Deserialize implementation that validates the hash
impl<'de> serde::Deserialize<'de> for DeltaSpec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        // Helper struct for deserialization (with all fields public for serde)
        #[derive(Deserialize)]
        struct DeltaSpecHelper {
            kind: DeltaKind,
            description: String,
            hash: Hash,
        }

        // Deserialize into helper
        let helper = DeltaSpecHelper::deserialize(deserializer)?;

        // Construct DeltaSpec with deserialized values
        let spec = DeltaSpec {
            kind: helper.kind,
            description: helper.description,
            hash: helper.hash,
        };

        // Validate: recompute hash and compare
        let computed_hash = spec.compute_hash().map_err(|e| {
            D::Error::custom(format!("Failed to compute hash for validation: {}", e))
        })?;

        if computed_hash != spec.hash {
            return Err(D::Error::custom(format!(
                "Invalid hash: stored hash does not match computed hash. \
                 This DeltaSpec may be corrupted or tampered. \
                 Stored: {:?}, Computed: {:?}",
                spec.hash, computed_hash
            )));
        }

        Ok(spec)
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
///
/// # Note on Unused Variants
///
/// `InvalidEventRef` is defined but not currently used in Phase 0.5.3.
/// It is reserved for future validation logic (see SPEC-0002 §7.3):
/// - `InvalidEventRef`: Will be used when validating InputMutation against EventStore
///
/// `InvalidHash` is also currently unused in direct API calls, but hash validation
/// IS enforced during deserialization (via custom Deserialize impl). The variant
/// exists for:
/// 1. Future explicit validation APIs
/// 2. Documentation of error contract
/// 3. SPEC-0002 examples compatibility
#[derive(Debug, thiserror::Error)]
pub enum DeltaError {
    /// Reserved for future validation - will be used when validating
    /// InputMutation delete/modify operations against EventStore
    #[allow(dead_code)]
    #[error("Invalid event reference: {0:?}")]
    InvalidEventRef(EventId),

    /// Hash validation error - currently used internally by Deserialize impl
    /// via serde::de::Error::custom, but this variant reserved for explicit
    /// validation APIs
    #[allow(dead_code)]
    #[error("Invalid hash: computed hash does not match stored hash")]
    InvalidHash,

    #[error("Invalid structure: {0}")]
    InvalidStructure(String),

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
        // Use constructor to get valid hash
        let original = DeltaSpec::new_clock_policy(
            Hash([2u8; 32]),
            "Test clock policy change".to_string(),
        )
        .expect("construction should succeed");

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

    /// Test 5: TrustPolicy rejects empty trust roots
    ///
    /// REQUIREMENT: Empty new_trust_roots should be rejected (means "trust no one")
    #[test]
    fn test_trust_policy_rejects_empty_roots() {
        let result = DeltaSpec::new_trust_policy(
            vec![], // Empty vector - should be rejected
            "Dangerous: trust no one".to_string(),
        );

        assert!(
            result.is_err(),
            "TrustPolicy with empty new_trust_roots should be rejected"
        );

        match result {
            Err(DeltaError::InvalidStructure(msg)) => {
                assert!(
                    msg.contains("trust no one"),
                    "Error message should explain the danger"
                );
            }
            _ => panic!("Expected InvalidStructure error"),
        }
    }

    /// Test 6: InputMutation with insert operation
    ///
    /// REQUIREMENT: Can express "same schedule, different inputs" (insert)
    #[test]
    fn test_input_mutation_insert() {
        let insert_event = InputEvent { placeholder: 123 };

        let delta = DeltaSpec::new_input_mutation(
            vec![insert_event.clone()],
            vec![],
            vec![],
            "Insert a delayed network packet".to_string(),
        )
        .expect("InputMutation with insert should succeed");

        // Verify the kind is correct
        match &delta.kind {
            DeltaKind::InputMutation { insert, delete, modify } => {
                assert_eq!(insert.len(), 1, "Should have 1 inserted event");
                assert_eq!(delete.len(), 0, "Should have 0 deleted events");
                assert_eq!(modify.len(), 0, "Should have 0 modified events");
                assert_eq!(insert[0].placeholder, 123, "Inserted event should match");
            }
            _ => panic!("Expected InputMutation kind"),
        }

        // Hash should be computed
        assert_ne!(delta.hash(), Hash([0u8; 32]), "Hash should be computed");
    }

    /// Test 7: InputMutation with delete operation
    ///
    /// REQUIREMENT: Can express "same schedule, different inputs" (delete)
    #[test]
    fn test_input_mutation_delete() {
        let event_to_delete = Hash([42u8; 32]);

        let delta = DeltaSpec::new_input_mutation(
            vec![],
            vec![event_to_delete],
            vec![],
            "Delete a network packet".to_string(),
        )
        .expect("InputMutation with delete should succeed");

        // Verify the kind is correct
        match &delta.kind {
            DeltaKind::InputMutation { insert, delete, modify } => {
                assert_eq!(insert.len(), 0, "Should have 0 inserted events");
                assert_eq!(delete.len(), 1, "Should have 1 deleted event");
                assert_eq!(modify.len(), 0, "Should have 0 modified events");
                assert_eq!(delete[0], event_to_delete, "Deleted event ID should match");
            }
            _ => panic!("Expected InputMutation kind"),
        }
    }

    /// Test 8: InputMutation with modify operation
    ///
    /// REQUIREMENT: Can express "same schedule, different inputs" (modify)
    #[test]
    fn test_input_mutation_modify() {
        let event_to_modify = Hash([99u8; 32]);
        let modified_event = InputEvent { placeholder: 456 };

        let delta = DeltaSpec::new_input_mutation(
            vec![],
            vec![],
            vec![(event_to_modify, modified_event.clone())],
            "Modify a network packet".to_string(),
        )
        .expect("InputMutation with modify should succeed");

        // Verify the kind is correct
        match &delta.kind {
            DeltaKind::InputMutation { insert, delete, modify } => {
                assert_eq!(insert.len(), 0, "Should have 0 inserted events");
                assert_eq!(delete.len(), 0, "Should have 0 deleted events");
                assert_eq!(modify.len(), 1, "Should have 1 modified event");
                assert_eq!(modify[0].0, event_to_modify, "Modified event ID should match");
                assert_eq!(modify[0].1.placeholder, 456, "Modified event should match");
            }
            _ => panic!("Expected InputMutation kind"),
        }
    }

    /// Test 9: Deserialization validates hash
    ///
    /// REQUIREMENT: Custom Deserialize must reject DeltaSpec with invalid hash
    #[test]
    fn test_deserialize_validates_hash() {
        // Create a valid DeltaSpec
        let valid_delta = DeltaSpec::new_scheduler_policy(
            Hash([1u8; 32]),
            "Test policy".to_string(),
        )
        .expect("construction should succeed");

        // Serialize it
        let bytes = canonical::encode(&valid_delta).expect("encoding should succeed");

        // Deserialize it (should succeed - hash is valid)
        let deserialized: DeltaSpec =
            canonical::decode(&bytes).expect("valid hash should deserialize");
        assert_eq!(deserialized, valid_delta);
    }

    /// Test 10: Deserialization rejects tampered hash
    ///
    /// REQUIREMENT: Deserialize must reject DeltaSpec with spoofed/tampered hash
    #[test]
    fn test_deserialize_rejects_tampered_hash() {
        // Manually construct a DeltaSpec with incorrect hash
        // (bypassing constructors which would compute correct hash)
        let tampered = DeltaSpec {
            kind: DeltaKind::SchedulerPolicy {
                new_policy: Hash([1u8; 32]),
            },
            description: "Test policy".to_string(),
            hash: Hash([0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]), // Wrong hash!
        };

        // Serialize the tampered spec
        let bytes = canonical::encode(&tampered).expect("encoding should succeed");

        // Attempt to deserialize (should FAIL - hash validation)
        let result: Result<DeltaSpec, _> = canonical::decode(&bytes);
        assert!(
            result.is_err(),
            "Deserialization should reject DeltaSpec with tampered hash"
        );

        // Verify error message mentions hash mismatch
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Invalid hash") || err_msg.contains("does not match"),
            "Error should mention hash validation failure: {}",
            err_msg
        );
    }

    /// Test 11: Constructor path produces deserializable DeltaSpec
    ///
    /// REQUIREMENT: All constructors should produce DeltaSpec that round-trips
    #[test]
    fn test_constructors_produce_valid_specs() {
        // Test all four constructors
        let scheduler = DeltaSpec::new_scheduler_policy(
            Hash([1u8; 32]),
            "Scheduler test".to_string(),
        )
        .expect("should succeed");

        let clock = DeltaSpec::new_clock_policy(
            Hash([2u8; 32]),
            "Clock test".to_string(),
        )
        .expect("should succeed");

        let trust = DeltaSpec::new_trust_policy(
            vec![AgentId::new("alice").expect("valid id")],
            "Trust test".to_string(),
        )
        .expect("should succeed");

        let mutation = DeltaSpec::new_input_mutation(
            vec![],
            vec![],
            vec![],
            "Mutation test".to_string(),
        )
        .expect("should succeed");

        // All should round-trip through serialization
        for delta in &[scheduler, clock, trust, mutation] {
            let bytes = canonical::encode(delta).expect("encoding should succeed");
            let decoded: DeltaSpec = canonical::decode(&bytes).expect("decoding should succeed");
            assert_eq!(&decoded, delta, "Round-trip should preserve DeltaSpec");
        }
    }
}
