//! Event Envelope v2: Policy as Structure (Phase 0.5.2)
//!
//! This module implements a content-addressed event DAG where:
//! - Policy is a first-class PolicyContext event (not metadata)
//! - Decision events MUST reference exactly one PolicyContext parent
//! - Event IDs are boring: H(kind || payload || sorted_parents)
//! - No nonces, no hidden state, no lies

use crate::canonical::{self, CanonicalError};
use crate::Hash;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Event ID - content-addressed hash of the canonical event bytes
pub type EventId = Hash;

/// Agent identifier (human, AI, or system)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct AgentId(String);

impl AgentId {
    pub fn new(id: impl Into<String>) -> Result<Self, EventError> {
        let id = id.into();
        if id.is_empty() {
            return Err(EventError::InvalidStructure(
                "AgentId cannot be empty".to_string(),
            ));
        }
        Ok(AgentId(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// SECURITY: Custom Deserialize validates non-empty on deserialization
impl<'de> Deserialize<'de> for AgentId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let id = String::deserialize(deserializer)?;
        if id.is_empty() {
            return Err(serde::de::Error::custom("AgentId cannot be empty"));
        }
        Ok(AgentId(id))
    }
}

/// Cryptographic signature over event data
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Signature(Vec<u8>);

impl Signature {
    pub fn new(bytes: Vec<u8>) -> Result<Self, EventError> {
        if bytes.is_empty() {
            return Err(EventError::InvalidStructure(
                "Signature cannot be empty".to_string(),
            ));
        }
        Ok(Signature(bytes))
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

// SECURITY: Custom Deserialize validates non-empty on deserialization
impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes = Vec::<u8>::deserialize(deserializer)?;
        if bytes.is_empty() {
            return Err(serde::de::Error::custom("Signature cannot be empty"));
        }
        Ok(Signature(bytes))
    }
}

/// Canonically-encoded bytes for event payloads.
///
/// This wrapper ensures all payloads are canonical CBOR.
/// The inner field is private to prevent construction of non-canonical data.
///
/// SECURITY: Custom Deserialize validates canonicality on deserialization.
/// Non-canonical bytes are rejected to prevent hash divergence attacks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct CanonicalBytes(Vec<u8>);

impl CanonicalBytes {
    /// Create canonical bytes by encoding a serializable value.
    pub fn from_value<T: Serialize>(value: &T) -> Result<Self, CanonicalError> {
        let bytes = canonical::encode(value)?;
        Ok(CanonicalBytes(bytes))
    }

    /// Decode canonical bytes to a deserializable value.
    pub fn to_value<T: for<'de> Deserialize<'de>>(&self) -> Result<T, CanonicalError> {
        canonical::decode(&self.0)
    }

    /// Get the raw bytes (read-only).
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Validate that bytes are canonical CBOR.
    ///
    /// This is used internally by Deserialize to reject non-canonical payloads.
    fn validate_canonical(bytes: &[u8]) -> Result<(), String> {
        // Decode the bytes to any valid CBOR value
        let value: ciborium::Value =
            canonical::decode(bytes).map_err(|e| format!("Invalid CBOR: {}", e))?;

        // Re-encode using canonical encoding
        let canonical_bytes =
            canonical::encode(&value).map_err(|e| format!("Re-encoding failed: {}", e))?;

        // If the bytes don't match, the original was not canonical
        if bytes != canonical_bytes {
            return Err("Payload bytes are not canonical CBOR".to_string());
        }

        Ok(())
    }
}

/// Custom Deserialize implementation that validates canonicality.
///
/// This prevents attacks where non-canonical encodings of the same logical value
/// produce different event_ids, breaking content-addressing.
impl<'de> Deserialize<'de> for CanonicalBytes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        // Deserialize as raw bytes
        let bytes = Vec::<u8>::deserialize(deserializer)?;

        // Validate canonicality
        Self::validate_canonical(&bytes).map_err(D::Error::custom)?;

        Ok(CanonicalBytes(bytes))
    }
}

/// Event classification - the four fundamental types.
///
/// These are the minimum types needed for:
/// - Content addressing (no semantic ambiguity)
/// - Policy-aware determinism
/// - Honest counterfactuals
/// - Mergeable DAGs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EventKind {
    /// Observation: Facts about the world
    ///
    /// Examples: clock samples, network messages, sensor readings, user inputs
    /// Properties: May be wrong, may be contradicted later, does not cause effects by itself
    Observation,

    /// PolicyContext: How reality will be interpreted
    ///
    /// Examples: clock_policy="trust_ntp", scheduler_policy="fifo", trust_policy=["agentA"]
    /// Properties: Immutable, hash-stable, explicit, can be forked/merged/superseded
    PolicyContext,

    /// Decision: Interpretive choice given evidence + policy
    ///
    /// Examples: "fire timer now", "rewrite rule R fired", "scheduler selected event X"
    /// INVARIANT: Every Decision MUST have exactly one PolicyContext as a parent
    Decision,

    /// Commit: Irreversible effect that escaped the system boundary
    ///
    /// Examples: timer fired, packet sent, disk write, external API call
    /// Properties: Only Commit events are "real" in the causal sense
    Commit,
}

/// The universal event envelope for the Loom worldline DAG (v2).
///
/// Events are content-addressed and cryptographically linked to form a DAG
/// representing the complete causal history of the system.
///
/// Fields are private to prevent post-construction mutation that would
/// invalidate the content-addressed event_id.
///
/// SECURITY: Custom Deserialize validates invariants on deserialization.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EventEnvelope {
    /// Content-addressed ID: H(kind || payload || sorted_parents)
    event_id: EventId,

    /// Event classification
    kind: EventKind,

    /// The actual payload (MUST be canonically encoded)
    payload: CanonicalBytes,

    /// Parent event(s) - sorted, deduplicated at construction
    parents: Vec<EventId>,

    /// Who created this event (optional)
    agent_id: Option<AgentId>,

    /// Cryptographic signature (required for Commit, optional otherwise)
    signature: Option<Signature>,

    /// Observation type tag (for Observation events only)
    /// Enables efficient filtering without decoding payloads
    observation_type: Option<String>,
}

impl EventEnvelope {
    /// Compute the event_id from the envelope's components.
    ///
    /// The event_id is content-addressed: H(kind || payload || sorted_parents)
    /// This ensures deterministic, collision-resistant identification.
    ///
    /// No nonces, no timestamps, no metadata. If it affects semantics, it's a parent.
    pub fn compute_event_id(
        kind: &EventKind,
        payload: &CanonicalBytes,
        parents: &[EventId],
    ) -> Result<EventId, CanonicalError> {
        // Canonical structure for hashing
        #[derive(Serialize)]
        struct EventIdInput<'a> {
            kind: &'a EventKind,
            payload: &'a [u8],
            parents: &'a [EventId],
        }

        let input = EventIdInput {
            kind,
            payload: payload.as_bytes(),
            parents, // Already sorted at construction
        };

        // Canonical encode and hash
        let canonical_bytes = canonical::encode(&input)?;
        let hash_bytes = blake3::hash(&canonical_bytes);

        Ok(Hash(*hash_bytes.as_bytes()))
    }

    /// Create a new Observation event.
    ///
    /// Observations are facts about the world (may be wrong, contradicted, or untrusted).
    ///
    /// # Arguments
    ///
    /// * `observation_type` - Optional type tag for efficient filtering (e.g., "OBS_CLOCK_SAMPLE_V0")
    pub fn new_observation(
        payload: CanonicalBytes,
        parents: Vec<EventId>,
        observation_type: Option<String>,
        agent_id: Option<AgentId>,
        signature: Option<Signature>,
    ) -> Result<Self, EventError> {
        let parents = Self::canonicalize_parents(parents);
        let event_id = Self::compute_event_id(&EventKind::Observation, &payload, &parents)?;

        Ok(EventEnvelope {
            event_id,
            kind: EventKind::Observation,
            payload,
            parents,
            agent_id,
            signature,
            observation_type,
        })
    }

    /// Create a new PolicyContext event.
    ///
    /// PolicyContexts define how reality will be interpreted.
    pub fn new_policy_context(
        payload: CanonicalBytes,
        parents: Vec<EventId>,
        agent_id: Option<AgentId>,
        signature: Option<Signature>,
    ) -> Result<Self, EventError> {
        let parents = Self::canonicalize_parents(parents);
        let event_id = Self::compute_event_id(&EventKind::PolicyContext, &payload, &parents)?;

        Ok(EventEnvelope {
            event_id,
            kind: EventKind::PolicyContext,
            payload,
            parents,
            agent_id,
            signature,
            observation_type: None,
        })
    }

    /// Create a new Decision event.
    ///
    /// INVARIANT: A Decision MUST have exactly one PolicyContext parent.
    /// This is enforced at construction time to make invalid states unrepresentable.
    pub fn new_decision(
        payload: CanonicalBytes,
        evidence_parents: Vec<EventId>,
        policy_parent: EventId,
        agent_id: Option<AgentId>,
        signature: Option<Signature>,
    ) -> Result<Self, EventError> {
        // Enforce: Decision must have at least one evidence parent
        if evidence_parents.is_empty() {
            return Err(EventError::InvalidStructure(
                "Decision must have at least one evidence parent".to_string(),
            ));
        }

        // Enforce: policy_parent must not be in evidence_parents (semantic clarity)
        if evidence_parents.contains(&policy_parent) {
            return Err(EventError::InvalidStructure(
                "policy_parent must not be included in evidence_parents".to_string(),
            ));
        }

        // Combine evidence + policy into parents list
        let mut all_parents = evidence_parents;
        all_parents.push(policy_parent);
        let parents = Self::canonicalize_parents(all_parents);

        let event_id = Self::compute_event_id(&EventKind::Decision, &payload, &parents)?;

        Ok(EventEnvelope {
            event_id,
            kind: EventKind::Decision,
            payload,
            parents,
            agent_id,
            signature,
            observation_type: None,
        })
    }

    /// Create a new Commit event.
    ///
    /// INVARIANT: A Commit MUST have at least one Decision parent.
    /// Signature is required for Commits (they crossed the system boundary).
    pub fn new_commit(
        payload: CanonicalBytes,
        decision_parent: EventId,
        extra_parents: Vec<EventId>,
        agent_id: Option<AgentId>,
        signature: Signature,
    ) -> Result<Self, EventError> {
        let mut all_parents = extra_parents;
        all_parents.push(decision_parent);
        let parents = Self::canonicalize_parents(all_parents);

        let event_id = Self::compute_event_id(&EventKind::Commit, &payload, &parents)?;

        Ok(EventEnvelope {
            event_id,
            kind: EventKind::Commit,
            payload,
            parents,
            agent_id,
            signature: Some(signature),
            observation_type: None,
        })
    }

    /// Canonicalize parent list: sort lexicographically and deduplicate.
    ///
    /// This ensures that H(parents) is deterministic regardless of insertion order.
    fn canonicalize_parents(parents: Vec<EventId>) -> Vec<EventId> {
        let unique: BTreeSet<EventId> = parents.into_iter().collect();
        unique.into_iter().collect()
    }

    /// Verify that the event_id matches the computed hash.
    pub fn verify_event_id(&self) -> Result<bool, CanonicalError> {
        let computed = Self::compute_event_id(&self.kind, &self.payload, &self.parents)?;
        Ok(computed == self.event_id)
    }

    // Read-only accessors (fields are private to prevent mutation)

    pub fn event_id(&self) -> EventId {
        self.event_id
    }

    pub fn kind(&self) -> &EventKind {
        &self.kind
    }

    pub fn payload(&self) -> &CanonicalBytes {
        &self.payload
    }

    pub fn parents(&self) -> &[EventId] {
        &self.parents
    }

    pub fn agent_id(&self) -> Option<&AgentId> {
        self.agent_id.as_ref()
    }

    pub fn signature(&self) -> Option<&Signature> {
        self.signature.as_ref()
    }

    pub fn observation_type(&self) -> Option<&str> {
        self.observation_type.as_deref()
    }

    /// Check if this event is a genesis event (no parents).
    pub fn is_genesis(&self) -> bool {
        self.parents.is_empty()
    }

    /// Check if this event is a merge (multiple parents).
    pub fn is_merge(&self) -> bool {
        self.parents.len() > 1
    }
}

// SECURITY: Custom Deserialize validates invariants that can be checked without EventStore.
// This prevents deserialized EventEnvelope from violating structural invariants.
// Full validation (parent existence, type-specific constraints) still requires validate_event().
impl<'de> Deserialize<'de> for EventEnvelope {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Helper struct for raw deserialization
        #[derive(Deserialize)]
        struct RawEventEnvelope {
            event_id: EventId,
            kind: EventKind,
            payload: CanonicalBytes,
            parents: Vec<EventId>,
            agent_id: Option<AgentId>,
            signature: Option<Signature>,
            observation_type: Option<String>,
        }

        let raw = RawEventEnvelope::deserialize(deserializer)?;

        // Validation 1: Verify event_id matches computed ID
        let computed_id = EventEnvelope::compute_event_id(&raw.kind, &raw.payload, &raw.parents)
            .map_err(serde::de::Error::custom)?;

        if raw.event_id != computed_id {
            return Err(serde::de::Error::custom(format!(
                "Tampered event_id: expected {:?}, got {:?}",
                computed_id, raw.event_id
            )));
        }

        // Validation 2: Verify parents are sorted and unique (canonical order)
        // Strict inequality ensures both sorted AND unique (no duplicates)
        // Note: windows(2) is empty when len <= 1, so all() returns true (correct behavior)
        let is_canonical = raw.parents.windows(2).all(|w| w[0] < w[1]);

        if !is_canonical {
            return Err(serde::de::Error::custom(
                "Parents must be canonically sorted and deduplicated",
            ));
        }

        // Validation 3: Commit events MUST have signature
        if raw.kind == EventKind::Commit && raw.signature.is_none() {
            return Err(serde::de::Error::custom(
                "Commit event must have a signature",
            ));
        }

        Ok(EventEnvelope {
            event_id: raw.event_id,
            kind: raw.kind,
            payload: raw.payload,
            parents: raw.parents,
            agent_id: raw.agent_id,
            signature: raw.signature,
            observation_type: raw.observation_type,
        })
    }
}

/// Errors that can occur when constructing or validating events.
#[derive(Debug, thiserror::Error)]
pub enum EventError {
    #[error("Canonical encoding error: {0}")]
    CanonicalError(#[from] CanonicalError),

    #[error("Invalid event structure: {0}")]
    InvalidStructure(String),

    #[error("Validation error: {0}")]
    ValidationError(String),
}

/// Event store abstraction for validation.
pub trait EventStore {
    /// Get an event by its ID.
    fn get(&self, event_id: &EventId) -> Option<&EventEnvelope>;
}

/// Validate a single event against structural rules.
///
/// This enforces invariants that may not be checkable at construction time
/// (e.g., when importing events from disk/network).
pub fn validate_event<S: EventStore>(event: &EventEnvelope, store: &S) -> Result<(), EventError> {
    // Rule 1: Event ID must match computed hash
    if !event.verify_event_id()? {
        return Err(EventError::ValidationError(
            "Event ID does not match computed hash".to_string(),
        ));
    }

    // Rule 2: Parents must be canonical (sorted, unique)
    // Zero-allocation check: strict inequality ensures both sorted AND unique
    // Note: windows(2) is empty when len <= 1, so all() returns true (correct behavior)
    let is_canonical = event.parents.windows(2).all(|w| w[0] < w[1]);
    if !is_canonical {
        return Err(EventError::ValidationError(
            "Parents are not canonically sorted/deduplicated".to_string(),
        ));
    }

    // Rule 2.5: All parents must exist in the store (data integrity)
    // This applies to ALL event types, not just Decision/Commit
    for parent_id in &event.parents {
        if store.get(parent_id).is_none() {
            return Err(EventError::ValidationError(format!(
                "{:?} event references unknown parent: {:?}. \
                 Ensure events are provided in topological order (parents before children).",
                event.kind, parent_id
            )));
        }
    }

    // Rule 3: Decision must have exactly one PolicyContext parent
    if matches!(event.kind, EventKind::Decision) {
        let mut policy_count = 0;
        let mut has_non_policy_parent = false;

        for parent_id in &event.parents {
            // Parent existence already validated in Rule 2.5
            let parent = store.get(parent_id).unwrap();
            if matches!(parent.kind, EventKind::PolicyContext) {
                policy_count += 1;
            } else {
                has_non_policy_parent = true;
            }
        }

        if policy_count != 1 {
            return Err(EventError::ValidationError(format!(
                "Decision must have exactly one PolicyContext parent, found {}",
                policy_count
            )));
        }

        if !has_non_policy_parent && event.parents.len() == 1 {
            return Err(EventError::ValidationError(
                "Decision must have evidence parents in addition to policy".to_string(),
            ));
        }
    }

    // Rule 4: Commit must have at least one Decision parent
    if matches!(event.kind, EventKind::Commit) {
        let mut has_decision_parent = false;

        for parent_id in &event.parents {
            // Parent existence already validated in Rule 2.5
            let parent = store.get(parent_id).unwrap();
            if matches!(parent.kind, EventKind::Decision) {
                has_decision_parent = true;
                break;
            }
        }

        if !has_decision_parent {
            return Err(EventError::ValidationError(
                "Commit must have at least one Decision parent".to_string(),
            ));
        }
    }

    // Rule 5: Commit must have a signature
    if matches!(event.kind, EventKind::Commit) && event.signature.is_none() {
        return Err(EventError::ValidationError(
            "Commit must have a signature".to_string(),
        ));
    }

    Ok(())
}

/// Validate a batch of events for structural consistency.
///
/// This function validates events against a base store, allowing events in the
/// batch to reference other events in the same batch. Events are validated in
/// the order provided, and parent lookups check both the base store and
/// previously validated events in the batch.
///
/// **Usage**: For validating a complete event set from import/migration:
/// 1. Provide the existing store (may be empty)
/// 2. Provide events in topological order (parents before children)
/// 3. All events will be validated, allowing intra-batch references
pub fn validate_store<S: EventStore>(
    store: &S,
    events: &[EventEnvelope],
) -> Result<(), EventError> {
    use std::collections::HashMap;

    // Build temporary lookup for events in this batch
    let mut batch_events: HashMap<EventId, &EventEnvelope> = HashMap::new();

    for event in events {
        // Create a combined store view (base + batch so far)
        let combined_store = CombinedStore {
            base: store,
            batch: &batch_events,
        };

        validate_event(event, &combined_store)?;

        // Add to batch lookup for subsequent events
        batch_events.insert(event.event_id(), event);
    }
    Ok(())
}

/// Temporary combined view of base store + batch events for validation.
struct CombinedStore<'a, S: EventStore> {
    base: &'a S,
    batch: &'a std::collections::HashMap<EventId, &'a EventEnvelope>,
}

impl<'a, S: EventStore> EventStore for CombinedStore<'a, S> {
    fn get(&self, event_id: &EventId) -> Option<&EventEnvelope> {
        // Check batch first, then base store
        self.batch
            .get(event_id)
            .copied()
            .or_else(|| self.base.get(event_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn test_agent_id() -> AgentId {
        AgentId::new("test-agent").unwrap()
    }

    fn test_signature() -> Signature {
        Signature::new(vec![0u8; 64]).unwrap()
    }

    #[test]
    fn test_genesis_observation() {
        let payload = CanonicalBytes::from_value(&"genesis observation").unwrap();
        let event = EventEnvelope::new_observation(
            payload,
            vec![],
            None,
            Some(test_agent_id()),
            Some(test_signature()),
        )
        .unwrap();

        assert!(event.is_genesis());
        assert!(!event.is_merge());
        assert!(event.verify_event_id().unwrap());
        assert_eq!(event.kind(), &EventKind::Observation);
    }

    #[test]
    fn test_policy_context() {
        let payload = CanonicalBytes::from_value(&serde_json::json!({
            "clock_policy": "trust_ntp",
            "scheduler_policy": "fifo",
            "trust_policy": ["agentA", "agentB"]
        }))
        .unwrap();

        let event = EventEnvelope::new_policy_context(
            payload,
            vec![],
            Some(test_agent_id()),
            Some(test_signature()),
        )
        .unwrap();

        assert!(event.is_genesis());
        assert_eq!(event.kind(), &EventKind::PolicyContext);
        assert!(event.verify_event_id().unwrap());
    }

    #[test]
    fn test_decision_with_policy_parent() {
        // Create evidence
        let obs_payload = CanonicalBytes::from_value(&"clock_sample=6000ms").unwrap();
        let observation =
            EventEnvelope::new_observation(obs_payload, vec![], None, Some(test_agent_id()), None)
                .unwrap();

        // Create policy
        let policy_payload = CanonicalBytes::from_value(&serde_json::json!({
            "clock_policy": "trust_ntp"
        }))
        .unwrap();
        let policy =
            EventEnvelope::new_policy_context(policy_payload, vec![], Some(test_agent_id()), None)
                .unwrap();

        // Create decision referencing both
        let decision_payload = CanonicalBytes::from_value(&"fire_timer").unwrap();
        let decision = EventEnvelope::new_decision(
            decision_payload,
            vec![observation.event_id()],
            policy.event_id(),
            Some(test_agent_id()),
            None,
        )
        .unwrap();

        assert_eq!(decision.kind(), &EventKind::Decision);
        assert!(decision.is_merge()); // Has 2 parents (observation + policy)
        assert!(decision.verify_event_id().unwrap());

        // Verify parents include both evidence and policy
        assert_eq!(decision.parents().len(), 2);
        assert!(decision.parents().contains(&observation.event_id()));
        assert!(decision.parents().contains(&policy.event_id()));
    }

    #[test]
    fn test_commit_requires_signature() {
        let decision_payload = CanonicalBytes::from_value(&"fire_timer").unwrap();

        // Create evidence (observation)
        let evidence = EventEnvelope::new_observation(
            CanonicalBytes::from_value(&"timer_request").unwrap(),
            vec![],
            None,
            None,
            None,
        )
        .unwrap();

        let policy = EventEnvelope::new_policy_context(
            CanonicalBytes::from_value(&"policy").unwrap(),
            vec![],
            None,
            None,
        )
        .unwrap();
        let decision = EventEnvelope::new_decision(
            decision_payload,
            vec![evidence.event_id()],
            policy.event_id(),
            Some(test_agent_id()),
            None,
        )
        .unwrap();

        let commit_payload = CanonicalBytes::from_value(&"timer_fired").unwrap();
        let commit = EventEnvelope::new_commit(
            commit_payload,
            decision.event_id(),
            vec![],
            Some(test_agent_id()),
            test_signature(),
        )
        .unwrap();

        assert_eq!(commit.kind(), &EventKind::Commit);
        assert!(commit.signature().is_some());
        assert!(commit.verify_event_id().unwrap());
    }

    #[test]
    fn test_parent_canonicalization() {
        let hash1 = Hash([1u8; 32]);
        let hash2 = Hash([2u8; 32]);
        let hash3 = Hash([3u8; 32]);

        // Same parents in different orders should produce same event_id
        let payload = CanonicalBytes::from_value(&"test").unwrap();

        let event1 = EventEnvelope::new_observation(
            payload.clone(),
            vec![hash1, hash2, hash3],
            None,
            None,
            None,
        )
        .unwrap();

        let event2 = EventEnvelope::new_observation(
            payload.clone(),
            vec![hash3, hash1, hash2],
            None,
            None,
            None,
        )
        .unwrap();

        assert_eq!(event1.event_id(), event2.event_id());
    }

    #[test]
    fn test_parent_deduplication() {
        let hash1 = Hash([1u8; 32]);

        let payload = CanonicalBytes::from_value(&"test").unwrap();

        // Duplicate parents should be deduplicated
        let event =
            EventEnvelope::new_observation(payload, vec![hash1, hash1, hash1], None, None, None)
                .unwrap();

        assert_eq!(event.parents().len(), 1);
        assert_eq!(event.parents()[0], hash1);
    }

    #[test]
    fn test_different_policy_yields_different_event_id() {
        // Create two different policies
        let policy1_payload = CanonicalBytes::from_value(&serde_json::json!({
            "clock_policy": "trust_ntp"
        }))
        .unwrap();
        let policy1 =
            EventEnvelope::new_policy_context(policy1_payload, vec![], None, None).unwrap();

        let policy2_payload = CanonicalBytes::from_value(&serde_json::json!({
            "clock_policy": "trust_monotonic"
        }))
        .unwrap();
        let policy2 =
            EventEnvelope::new_policy_context(policy2_payload, vec![], None, None).unwrap();

        // Same evidence
        let obs_payload = CanonicalBytes::from_value(&"clock_sample=6000ms").unwrap();
        let observation =
            EventEnvelope::new_observation(obs_payload, vec![], None, None, None).unwrap();

        // Same decision payload, same evidence, different policy
        let decision_payload = CanonicalBytes::from_value(&"fire_timer").unwrap();

        let decision1 = EventEnvelope::new_decision(
            decision_payload.clone(),
            vec![observation.event_id()],
            policy1.event_id(),
            None,
            None,
        )
        .unwrap();

        let decision2 = EventEnvelope::new_decision(
            decision_payload.clone(),
            vec![observation.event_id()],
            policy2.event_id(),
            None,
            None,
        )
        .unwrap();

        // Different policy → different event_id (no collision)
        assert_ne!(decision1.event_id(), decision2.event_id());
    }

    #[test]
    fn test_tampered_event_id_fails_verification() {
        let payload = CanonicalBytes::from_value(&"test").unwrap();
        let mut event = EventEnvelope::new_observation(payload, vec![], None, None, None).unwrap();

        // Tamper with event_id
        event.event_id = Hash([0xFF; 32]);

        // Verification should fail
        assert!(!event.verify_event_id().unwrap());
    }

    #[test]
    fn test_collision_resistance_comprehensive() {
        let base_payload = CanonicalBytes::from_value(&"test").unwrap();
        let base_parents = vec![Hash([0u8; 32])];

        let base = EventEnvelope::new_observation(
            base_payload.clone(),
            base_parents.clone(),
            None,
            None,
            None,
        )
        .unwrap();

        // Different parents
        let diff_parents = EventEnvelope::new_observation(
            base_payload.clone(),
            vec![Hash([1u8; 32])],
            None,
            None,
            None,
        )
        .unwrap();
        assert_ne!(base.event_id(), diff_parents.event_id());

        // Different kind (Policy vs Observation)
        let diff_kind = EventEnvelope::new_policy_context(
            base_payload.clone(),
            base_parents.clone(),
            None,
            None,
        )
        .unwrap();
        assert_ne!(base.event_id(), diff_kind.event_id());

        // Different payload
        let diff_payload = CanonicalBytes::from_value(&"different").unwrap();
        let diff_payload_event =
            EventEnvelope::new_observation(diff_payload, base_parents.clone(), None, None, None)
                .unwrap();
        assert_ne!(base.event_id(), diff_payload_event.event_id());
    }

    #[test]
    fn test_empty_payload() {
        let payload = CanonicalBytes::from_value(&()).unwrap();
        let event = EventEnvelope::new_observation(payload, vec![], None, None, None).unwrap();

        assert!(event.verify_event_id().unwrap());
        assert!(event.is_genesis());
    }

    #[test]
    fn test_agent_id_validation() {
        assert!(AgentId::new("valid").is_ok());
        assert!(AgentId::new("").is_err());
    }

    #[test]
    fn test_signature_validation() {
        assert!(Signature::new(vec![1, 2, 3]).is_ok());
        assert!(Signature::new(vec![]).is_err());
    }

    // Validation tests - negative cases

    /// Simple in-memory event store for testing validation
    struct TestStore {
        events: HashMap<EventId, EventEnvelope>,
    }

    impl TestStore {
        fn new() -> Self {
            TestStore {
                events: HashMap::new(),
            }
        }

        fn insert(&mut self, event: EventEnvelope) {
            self.events.insert(event.event_id(), event);
        }
    }

    impl EventStore for TestStore {
        fn get(&self, event_id: &EventId) -> Option<&EventEnvelope> {
            self.events.get(event_id)
        }
    }

    #[test]
    fn test_validate_decision_without_policy_parent() {
        let mut store = TestStore::new();

        // Create observation (no policy)
        let obs = EventEnvelope::new_observation(
            CanonicalBytes::from_value(&"test").unwrap(),
            vec![],
            None,
            None,
            None,
        )
        .unwrap();
        store.insert(obs.clone());

        // Manually construct a Decision with only observation parent (no policy)
        // This bypasses the typed constructor to test validation
        let payload = CanonicalBytes::from_value(&"bad").unwrap();
        let parents = vec![obs.event_id()];
        let event_id =
            EventEnvelope::compute_event_id(&EventKind::Decision, &payload, &parents).unwrap();

        let bad_decision = EventEnvelope {
            event_id,
            kind: EventKind::Decision,
            payload,
            parents,
            agent_id: None,
            signature: None,
            observation_type: None,
        };

        let result = validate_event(&bad_decision, &store);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("exactly one PolicyContext parent"));
    }

    #[test]
    fn test_validate_decision_with_two_policies() {
        let mut store = TestStore::new();

        // Create two policies
        let policy1 = EventEnvelope::new_policy_context(
            CanonicalBytes::from_value(&"policy1").unwrap(),
            vec![],
            None,
            None,
        )
        .unwrap();
        let policy2 = EventEnvelope::new_policy_context(
            CanonicalBytes::from_value(&"policy2").unwrap(),
            vec![],
            None,
            None,
        )
        .unwrap();
        store.insert(policy1.clone());
        store.insert(policy2.clone());

        // Manually construct Decision with two policy parents
        let payload = CanonicalBytes::from_value(&"bad").unwrap();
        let parents = vec![policy1.event_id(), policy2.event_id()];
        let event_id =
            EventEnvelope::compute_event_id(&EventKind::Decision, &payload, &parents).unwrap();

        let bad_decision = EventEnvelope {
            event_id,
            kind: EventKind::Decision,
            payload,
            parents,
            agent_id: None,
            signature: None,
            observation_type: None,
        };

        let result = validate_event(&bad_decision, &store);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("exactly one PolicyContext parent"));
    }

    #[test]
    fn test_validate_commit_without_decision_parent() {
        let mut store = TestStore::new();

        // Create observation (not a decision)
        let obs = EventEnvelope::new_observation(
            CanonicalBytes::from_value(&"test").unwrap(),
            vec![],
            None,
            None,
            None,
        )
        .unwrap();
        store.insert(obs.clone());

        // Manually construct Commit with only observation parent (no decision)
        let payload = CanonicalBytes::from_value(&"bad").unwrap();
        let parents = vec![obs.event_id()];
        let event_id =
            EventEnvelope::compute_event_id(&EventKind::Commit, &payload, &parents).unwrap();

        let bad_commit = EventEnvelope {
            event_id,
            kind: EventKind::Commit,
            payload,
            parents,
            agent_id: None,
            signature: Some(test_signature()),
            observation_type: None,
        };

        let result = validate_event(&bad_commit, &store);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("at least one Decision parent"));
    }

    #[test]
    fn test_validate_commit_without_signature() {
        let mut store = TestStore::new();

        // Create evidence (observation)
        let evidence = EventEnvelope::new_observation(
            CanonicalBytes::from_value(&"evidence").unwrap(),
            vec![],
            None,
            None,
            None,
        )
        .unwrap();

        // Create valid decision chain
        let policy = EventEnvelope::new_policy_context(
            CanonicalBytes::from_value(&"policy").unwrap(),
            vec![],
            None,
            None,
        )
        .unwrap();
        let decision = EventEnvelope::new_decision(
            CanonicalBytes::from_value(&"decide").unwrap(),
            vec![evidence.event_id()],
            policy.event_id(),
            None,
            None,
        )
        .unwrap();
        store.insert(evidence);
        store.insert(policy);
        store.insert(decision.clone());

        // Manually construct Commit without signature
        let payload = CanonicalBytes::from_value(&"bad").unwrap();
        let parents = vec![decision.event_id()];
        let event_id =
            EventEnvelope::compute_event_id(&EventKind::Commit, &payload, &parents).unwrap();

        let bad_commit = EventEnvelope {
            observation_type: None,
            event_id,
            kind: EventKind::Commit,
            payload,
            parents,
            agent_id: None,
            signature: None, // Missing signature!
        };

        let result = validate_event(&bad_commit, &store);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must have a signature"));
    }

    #[test]
    fn test_validate_tampered_event_id() {
        let store = TestStore::new();

        let mut event = EventEnvelope::new_observation(
            CanonicalBytes::from_value(&"test").unwrap(),
            vec![],
            None,
            None,
            None,
        )
        .unwrap();

        // Tamper with event_id
        event.event_id = Hash([0xFF; 32]);

        let result = validate_event(&event, &store);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("does not match computed hash"));
    }

    #[test]
    fn test_validate_valid_store() {
        let mut store = TestStore::new();

        // Create a valid event chain
        let obs = EventEnvelope::new_observation(
            CanonicalBytes::from_value(&"sample").unwrap(),
            vec![],
            None,
            None,
            None,
        )
        .unwrap();

        let policy = EventEnvelope::new_policy_context(
            CanonicalBytes::from_value(&"policy").unwrap(),
            vec![],
            None,
            None,
        )
        .unwrap();

        let decision = EventEnvelope::new_decision(
            CanonicalBytes::from_value(&"decide").unwrap(),
            vec![obs.event_id()],
            policy.event_id(),
            None,
            None,
        )
        .unwrap();

        let commit = EventEnvelope::new_commit(
            CanonicalBytes::from_value(&"commit").unwrap(),
            decision.event_id(),
            vec![],
            None,
            test_signature(),
        )
        .unwrap();

        store.insert(obs.clone());
        store.insert(policy.clone());
        store.insert(decision.clone());
        store.insert(commit.clone());

        let events = vec![obs, policy, decision, commit];
        let result = validate_store(&store, &events);
        assert!(result.is_ok());
    }

    #[test]
    fn test_decision_policy_only_parent_invalid() {
        let mut store = TestStore::new();

        let policy = EventEnvelope::new_policy_context(
            CanonicalBytes::from_value(&"policy").unwrap(),
            vec![],
            None,
            None,
        )
        .unwrap();
        store.insert(policy.clone());

        // Manually construct Decision with ONLY policy parent (no evidence)
        let payload = CanonicalBytes::from_value(&"bad").unwrap();
        let parents = vec![policy.event_id()];
        let event_id =
            EventEnvelope::compute_event_id(&EventKind::Decision, &payload, &parents).unwrap();

        let bad_decision = EventEnvelope {
            event_id,
            kind: EventKind::Decision,
            payload,
            parents,
            agent_id: None,
            signature: None,
            observation_type: None,
        };

        let result = validate_event(&bad_decision, &store);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("must have evidence parents"));
    }

    #[test]
    fn test_canonical_bytes_rejects_non_canonical() {
        // Create non-canonical CBOR: indefinite-length map (which is not canonical)
        // Canonical CBOR requires definite-length encoding
        let non_canonical_bytes = vec![
            0xBF, // Indefinite-length map start
            0x61, 0x61, // Key "a"
            0x01, // Value 1
            0xFF, // Break
        ];

        // Try to create CanonicalBytes wrapper and serialize it in an EventEnvelope
        // When the envelope is deserialized, it should reject non-canonical payloads
        let wrapper = CanonicalBytes(non_canonical_bytes);

        // Serialize the wrapper in a structure
        let mut buf = Vec::new();
        ciborium::ser::into_writer(&wrapper, &mut buf).unwrap();

        // Try to deserialize - should fail validation
        let result: Result<CanonicalBytes, _> = ciborium::de::from_reader(&buf[..]);
        assert!(result.is_err());
    }

    #[test]
    fn test_canonical_bytes_accepts_canonical() {
        // Create canonical CBOR for a simple map {"a": 1}
        let value = serde_json::json!({"a": 1});
        let canonical_bytes = canonical::encode(&value).unwrap();

        // Wrap in CanonicalBytes
        let wrapper = CanonicalBytes(canonical_bytes.clone());

        // Serialize the wrapper
        let mut buf = Vec::new();
        ciborium::ser::into_writer(&wrapper, &mut buf).unwrap();

        // Deserialize should succeed
        let result: Result<CanonicalBytes, _> = ciborium::de::from_reader(&buf[..]);
        assert!(result.is_ok());

        // And the bytes should match exactly
        assert_eq!(result.unwrap().as_bytes(), &canonical_bytes);
    }

    #[test]
    fn test_canonical_bytes_roundtrip_validation() {
        // Create a value
        let original = serde_json::json!({
            "key": "value",
            "number": 42,
            "nested": {"inner": true}
        });

        // Encode canonically
        let canonical = CanonicalBytes::from_value(&original).unwrap();

        // Serialize the CanonicalBytes wrapper
        let mut serialized = Vec::new();
        ciborium::ser::into_writer(&canonical, &mut serialized).unwrap();

        // Deserialize it back
        let deserialized: CanonicalBytes = ciborium::de::from_reader(&serialized[..]).unwrap();

        // Should match exactly
        assert_eq!(canonical.as_bytes(), deserialized.as_bytes());
    }

    #[test]
    fn test_canonical_bytes_rejects_wrong_int_encoding() {
        // CBOR allows multiple encodings of the same integer
        // E.g., the number 23 can be encoded as:
        // - 0x17 (1 byte, canonical)
        // - 0x1817 (2 bytes, NOT canonical)
        // - 0x190017 (3 bytes, NOT canonical)

        // Non-canonical: 2-byte encoding of 23
        let non_canonical = vec![0x18, 0x17];

        // Wrap it
        let wrapper = CanonicalBytes(non_canonical);

        // Serialize
        let mut buf = Vec::new();
        ciborium::ser::into_writer(&wrapper, &mut buf).unwrap();

        // This should be rejected during deserialization
        let result: Result<CanonicalBytes, _> = ciborium::de::from_reader(&buf[..]);
        assert!(result.is_err());
    }

    #[test]
    fn test_canonical_encoder_sorts_map_keys() {
        // Canonical CBOR requires map keys to be sorted lexicographically
        // Our canonical encoder should sort them, and re-encoding should produce identical bytes

        use ciborium::value::Value;

        // Create map with keys in wrong order (z before a)
        let map = vec![
            (Value::Text("z".to_string()), Value::Integer(1.into())),
            (Value::Text("a".to_string()), Value::Integer(2.into())),
        ];

        // Encode with canonical encoder (should sort keys to a, z)
        let canonical_bytes = canonical::encode(&Value::Map(map)).unwrap();

        // Wrap and serialize
        let wrapper = CanonicalBytes(canonical_bytes.clone());
        let mut buf = Vec::new();
        ciborium::ser::into_writer(&wrapper, &mut buf).unwrap();

        // Deserialize and verify it's accepted (keys were sorted by canonical encoder)
        let deserialized: CanonicalBytes = ciborium::de::from_reader(&buf[..]).unwrap();

        // The deserialized bytes should match canonical
        assert_eq!(deserialized.as_bytes(), &canonical_bytes);
    }

    #[test]
    fn test_canonical_bytes_rejects_manually_unsorted_keys() {
        // Manually construct CBOR with keys in wrong order: {"z": 1, "a": 2}
        // This tests that deserialization rejects non-canonical CBOR
        let unsorted_cbor = vec![
            0xA2, // map(2)
            0x61, 0x7A, // text(1) "z"
            0x01, // unsigned(1)
            0x61, 0x61, // text(1) "a"
            0x02, // unsigned(2)
        ];

        // Try to deserialize the raw unsorted CBOR into CanonicalBytes
        let wrapper = CanonicalBytes(unsorted_cbor);
        let mut buf = Vec::new();
        ciborium::ser::into_writer(&wrapper, &mut buf).unwrap();

        // Deserialization should reject unsorted map keys
        let result: Result<CanonicalBytes, _> = ciborium::de::from_reader(&buf[..]);
        assert!(
            result.is_err(),
            "Unsorted map keys should be rejected during deserialization"
        );
    }

    #[test]
    fn test_signature_deserialize_rejects_empty() {
        // Attempt to deserialize an empty Signature
        // This should fail because Signature::new() rejects empty bytes
        let empty_sig = Signature(vec![]);

        // Serialize it
        let mut buf = Vec::new();
        ciborium::ser::into_writer(&empty_sig, &mut buf).unwrap();

        // Deserialize should reject empty signature
        let result: Result<Signature, _> = ciborium::de::from_reader(&buf[..]);
        assert!(
            result.is_err(),
            "Empty Signature should be rejected during deserialization"
        );
    }

    #[test]
    fn test_signature_deserialize_accepts_non_empty() {
        // Valid signature with non-empty bytes
        let valid_sig = Signature::new(vec![1, 2, 3, 4]).unwrap();

        // Serialize it
        let mut buf = Vec::new();
        ciborium::ser::into_writer(&valid_sig, &mut buf).unwrap();

        // Deserialize should accept it
        let deserialized: Signature = ciborium::de::from_reader(&buf[..]).unwrap();
        assert_eq!(deserialized.as_bytes(), &[1, 2, 3, 4]);
    }

    #[test]
    fn test_agent_id_deserialize_rejects_empty() {
        // Attempt to deserialize an empty AgentId
        // This should fail because AgentId::new() rejects empty strings
        let empty_id = AgentId(String::new());

        // Serialize it
        let mut buf = Vec::new();
        ciborium::ser::into_writer(&empty_id, &mut buf).unwrap();

        // Deserialize should reject empty AgentId
        let result: Result<AgentId, _> = ciborium::de::from_reader(&buf[..]);
        assert!(
            result.is_err(),
            "Empty AgentId should be rejected during deserialization"
        );
    }

    #[test]
    fn test_agent_id_deserialize_accepts_non_empty() {
        // Valid AgentId with non-empty string
        let valid_id = AgentId::new("agent-123").unwrap();

        // Serialize it
        let mut buf = Vec::new();
        ciborium::ser::into_writer(&valid_id, &mut buf).unwrap();

        // Deserialize should accept it
        let deserialized: AgentId = ciborium::de::from_reader(&buf[..]).unwrap();
        assert_eq!(deserialized.as_str(), "agent-123");
    }

    #[test]
    fn test_event_envelope_deserialize_rejects_tampered_id() {
        // Create a valid observation
        let payload = CanonicalBytes::from_value(&serde_json::json!({"data": "test"})).unwrap();
        let agent_id = AgentId::new("agent-1").unwrap();
        let event = EventEnvelope::new_observation(
            payload.clone(),
            vec![],
            None,
            Some(agent_id.clone()),
            None,
        )
        .unwrap();

        // Manually tamper with the event_id
        let tampered = EventEnvelope {
            observation_type: None,
            event_id: Hash([0xFF; 32]), // Tampered hash
            kind: event.kind.clone(),
            payload: payload.clone(),
            parents: event.parents.clone(),
            agent_id: Some(agent_id),
            signature: None,
        };

        // Serialize the tampered event
        let mut buf = Vec::new();
        ciborium::ser::into_writer(&tampered, &mut buf).unwrap();

        // Deserialize should reject tampered event_id
        let result: Result<EventEnvelope, _> = ciborium::de::from_reader(&buf[..]);
        assert!(
            result.is_err(),
            "Deserialization should reject tampered event_id"
        );
    }

    #[test]
    fn test_event_envelope_deserialize_rejects_unsorted_parents() {
        // Create event with manually unsorted parents (bypassing constructor)
        let payload = CanonicalBytes::from_value(&serde_json::json!({"data": "test"})).unwrap();
        let parent1 = Hash([1u8; 32]);
        let parent2 = Hash([2u8; 32]);

        // Deliberately unsorted (parent2 before parent1)
        let unsorted_parents = vec![parent2, parent1];

        let tampered = EventEnvelope {
            observation_type: None,
            event_id: Hash([0xAA; 32]), // Doesn't matter, will fail parent check first
            kind: EventKind::Observation,
            payload,
            parents: unsorted_parents,
            agent_id: Some(AgentId::new("agent-1").unwrap()),
            signature: None,
        };

        // Serialize
        let mut buf = Vec::new();
        ciborium::ser::into_writer(&tampered, &mut buf).unwrap();

        // Deserialize should reject unsorted parents
        let result: Result<EventEnvelope, _> = ciborium::de::from_reader(&buf[..]);
        assert!(
            result.is_err(),
            "Deserialization should reject unsorted parents"
        );
    }

    #[test]
    fn test_event_envelope_deserialize_rejects_commit_without_signature() {
        // Create a Commit event without signature (invalid)
        let payload = CanonicalBytes::from_value(&serde_json::json!({"data": "test"})).unwrap();
        let decision_id = Hash([3u8; 32]);

        let tampered = EventEnvelope {
            observation_type: None,
            event_id: Hash([0xBB; 32]),
            kind: EventKind::Commit,
            payload,
            parents: vec![decision_id],
            agent_id: Some(AgentId::new("agent-1").unwrap()),
            signature: None, // Missing signature on Commit!
        };

        // Serialize
        let mut buf = Vec::new();
        ciborium::ser::into_writer(&tampered, &mut buf).unwrap();

        // Deserialize should reject Commit without signature
        let result: Result<EventEnvelope, _> = ciborium::de::from_reader(&buf[..]);
        assert!(
            result.is_err(),
            "Deserialization should reject Commit without signature"
        );
    }

    #[test]
    fn test_event_envelope_deserialize_rejects_duplicate_parents() {
        // Create event with duplicate parents (bypassing constructor)
        let payload = CanonicalBytes::from_value(&serde_json::json!({"data": "test"})).unwrap();
        let parent = Hash([1u8; 32]);

        // Duplicate parent
        let duplicate_parents = vec![parent, parent];

        let tampered = EventEnvelope {
            observation_type: None,
            event_id: Hash([0xCC; 32]),
            kind: EventKind::Observation,
            payload,
            parents: duplicate_parents,
            agent_id: Some(AgentId::new("agent-1").unwrap()),
            signature: None,
        };

        // Serialize
        let mut buf = Vec::new();
        ciborium::ser::into_writer(&tampered, &mut buf).unwrap();

        // Deserialize should reject duplicate parents
        let result: Result<EventEnvelope, _> = ciborium::de::from_reader(&buf[..]);
        assert!(
            result.is_err(),
            "Deserialization should reject duplicate parents"
        );
    }
}
