# SPEC-0002: DeltaSpec - Counterfactual Specification

**Status:** Draft
**Author:** Claude (Sonnet 4.5)
**Date:** 2025-12-28
**Depends On:** SPEC-0001 (Canonical Encoding), ARCH-0009 (Time & Counterfactuals)

## 1. Abstract

DeltaSpec provides a formal, content-addressed way to express controlled violations of history for debugging, testing, and causal analysis. It enables "what if" questions to be precisely specified, executed, and compared.

## 2. Motivation

### 2.1 The Problem

Debugging non-deterministic systems requires answering counterfactual questions:

- "What if the packet arrived 10ms later?"
- "What if the scheduler used LIFO instead of FIFO?"
- "What if we trusted NTP less?"

Without formal specification, these questions are:

- **Vague**: "Later" by whose clock? Which packet?
- **Unreproducible**: Can't replay the exact scenario
- **Uncomparable**: Can't diff two counterfactuals precisely

### 2.2 The Solution

DeltaSpec is a **content-addressed, canonical-encodable** struct that:

1. Precisely describes a controlled violation of history
2. Can be hashed and referenced in event DAG (for fork points)
3. Enables deterministic replay of counterfactual branches
4. Allows causal diff: "which decision mattered?"

## 3. Design

### 3.1 Core Types

```rust
/// Describes a controlled violation of history
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DeltaSpec {
    /// What kind of counterfactual
    pub kind: DeltaKind,

    /// Human-readable justification (for debugging)
    pub description: String,

    /// Content-addressed hash of this spec
    /// Used to reference this delta in fork events
    pub hash: Hash,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum DeltaKind {
    /// Change scheduler policy (e.g., FIFO → LIFO)
    SchedulerPolicy {
        new_policy: PolicyHash,
    },

    /// Inject/modify/delete input events
    InputMutation {
        insert: Vec<InputEvent>,
        delete: Vec<EventId>,
        modify: Vec<(EventId, InputEvent)>,
    },

    /// Change clock interpretation policy
    ClockPolicy {
        new_policy: PolicyHash,
    },

    /// Change trust assumptions
    TrustPolicy {
        new_trust_roots: Vec<AgentId>,
    },
}
```

### 3.2 Content-Addressing

DeltaSpec MUST be canonical-encodable (per SPEC-0001):

```rust
delta_hash = BLAKE3(canonical_cbor(DeltaSpec))
```

**Invariants:**

1. Same logical delta → identical hash (cross-platform, cross-runtime)
2. Different deltas → different hashes (collision resistance)
3. Hash is computed deterministically from fields

### 3.3 Usage in Event DAG

DeltaSpec creates fork points in the event DAG:

```text
E1 ──► E2 ──► E3 ──► E4 (baseline worldline)
          ╲
           ╲──► E3′ ──► E4′ (counterfactual with delta_hash)
```

Fork event structure:

```rust
Fork {
    base_cut: EventId,      // Where to diverge (E2)
    delta_spec: DeltaSpec,  // What to change
    delta_hash: Hash, // For referencing
}
```

## 4. Requirements

### 4.1 Functional Requirements

### FR-1: Scheduler Policy Changes

MUST be able to express: "same inputs, different schedule"

Example:

```rust
DeltaSpec {
    kind: SchedulerPolicy {
        new_policy: hash("lifo_scheduler.rhai"),
    },
    description: "Test race bug with reversed task order".into(),
    hash: computed_hash,
}
```

### FR-2: Input Mutations

MUST be able to express: "same schedule, different inputs"

Example:

```rust
DeltaSpec {
    kind: InputMutation {
        insert: vec![
            InputEvent::NetworkPacket {
                delay_ns: 10_000_000, // 10ms
                payload: [...],
            }
        ],
        delete: vec![original_packet_id],
        modify: vec![],
    },
    description: "Delay packet by 10ms".into(),
    hash: computed_hash,
}
```

### FR-3: Clock Policy Changes

MUST be able to express: "same inputs, different clock interpretation"

Example:

```rust
DeltaSpec {
    kind: ClockPolicy {
        new_policy: hash("ntp_skeptical.rhai"),
    },
    description: "Test timeout with skeptical NTP".into(),
    hash: computed_hash,
}
```

### FR-4: Trust Policy Changes

MUST be able to express: "same inputs, different trust assumptions"

Example:

```rust
DeltaSpec {
    kind: TrustPolicy {
        new_trust_roots: vec![
            AgentId("alice".into()),
            // removed "bob" - what if we didn't trust bob?
        ],
    },
    description: "Test with bob untrusted".into(),
    hash: computed_hash,
}
```

### 4.2 Non-Functional Requirements

### NFR-1: Determinism

Given the same DeltaSpec, replaying from the same base_cut MUST produce identical results.

### NFR-2: Content-Addressing

DeltaSpec hash MUST be deterministic across platforms (x86-64, ARM64, WASM).

### NFR-3: Canonical Encoding

DeltaSpec MUST serialize to canonical CBOR (per SPEC-0001).

### NFR-4: No Side Effects

Constructing a DeltaSpec MUST NOT execute the delta. Execution happens only when explicitly applied during replay.

## 5. Acceptance Criteria

### AC-1: Expressiveness

- [ ] Can express "same inputs, different schedule"
- [ ] Can express "same schedule, different inputs"
- [ ] Can express "same inputs, different clock policy"
- [ ] Can express "same inputs, different trust policy"

### AC-2: Content-Addressing

- [ ] DeltaSpec is canonical-encodable
- [ ] Same delta → identical hash
- [ ] Different deltas → different hashes
- [ ] Hash is stable across platforms

### AC-3: Type Safety

- [ ] Cannot construct invalid DeltaSpec (enforced by types)
- [ ] PolicyHash references existing policies
- [ ] EventId references existing events
- [ ] AgentId is non-empty

## 6. Test Plan

### 6.1 Unit Tests

#### Test 1: Canonical Encoding

```rust
#[test]
fn test_deltaspec_canonical_encoding() {
    let delta = DeltaSpec { ... };
    let bytes1 = canonical::encode(&delta)?;
    let bytes2 = canonical::encode(&delta)?;
    assert_eq!(bytes1, bytes2); // deterministic
}
```

#### Test 2: Round-Trip

```rust
#[test]
fn test_deltaspec_roundtrip() {
    let original = DeltaSpec { ... };
    let bytes = canonical::encode(&original)?;
    let decoded: DeltaSpec = canonical::decode(&bytes)?;
    assert_eq!(original, decoded);
}
```

#### Test 3: Hash Stability

```rust
#[test]
fn test_deltaspec_hash_stability() {
    let delta = DeltaSpec { ... };
    let hash1 = delta.compute_hash();
    let hash2 = delta.compute_hash();
    assert_eq!(hash1, hash2);
}
```

#### Test 4: Collision Resistance

```rust
#[test]
fn test_different_deltas_different_hashes() {
    let delta1 = DeltaSpec { kind: SchedulerPolicy { ... }, ... };
    let delta2 = DeltaSpec { kind: ClockPolicy { ... }, ... };
    assert_ne!(delta1.hash, delta2.hash);
}
```

### 6.2 Integration Tests

#### Test 5: Scheduler Policy Fork

```rust
#[test]
fn test_scheduler_policy_fork() {
    let base = build_baseline_worldline();
    let delta = DeltaSpec {
        kind: SchedulerPolicy { new_policy: lifo_hash },
        ...
    };

    let fork = Fork::new(base.last_event_id(), delta);
    let counterfactual = replay_from_fork(fork);

    assert_ne!(base.final_hash(), counterfactual.final_hash());
    // But both should be deterministic on re-replay
}
```

#### Test 6: Input Mutation Fork

```rust
#[test]
fn test_input_mutation_fork() {
    let base = build_baseline_worldline();
    let delta = DeltaSpec {
        kind: InputMutation {
            insert: vec![delayed_packet],
            ...
        },
        ...
    };

    let fork = Fork::new(base.cut_at(100), delta);
    let counterfactual = replay_from_fork(fork);

    // Counterfactual should have extra event
    assert_eq!(counterfactual.events.len(), base.events.len() + 1);
}
```

### 6.3 Property Tests

#### Property 1: Replay Determinism

```rust
#[quickcheck]
fn prop_replay_determinism(delta: DeltaSpec, cut: EventId) {
    let fork1 = replay_from_delta(cut, delta.clone());
    let fork2 = replay_from_delta(cut, delta.clone());
    assert_eq!(fork1.hash(), fork2.hash());
}
```

#### Property 2: Content-Addressing Correctness

```rust
#[quickcheck]
fn prop_content_addressing(delta: DeltaSpec) {
    // Recompute hash and verify it matches stored hash
    let computed = delta.compute_hash();
    assert_eq!(computed, delta.hash, "Stored hash must match computed hash");
}
```

**Note:** Hash collision resistance cannot be tested as an absolute property (collisions are
theoretically possible). Instead, we rely on BLAKE3's cryptographic guarantees and validate
that identical content produces identical hashes (determinism).

## 7. Implementation Notes

### 7.1 Hash Computation

```rust
impl DeltaSpec {
    pub fn compute_hash(&self) -> Hash {
        let bytes = canonical::encode(&(&self.kind, &self.description))
            .expect("DeltaSpec must be canonically encodable");
        Hash(*blake3::hash(&bytes).as_bytes())
    }
}
```

### 7.2 Constructor Pattern

```rust
impl DeltaSpec {
    pub fn new_scheduler_policy(
        new_policy: PolicyHash,
        description: String,
    ) -> Self {
        let spec = Self {
            kind: DeltaKind::SchedulerPolicy { new_policy },
            description,
            hash: Hash([0u8; 32]), // temp
        };

        let hash = spec.compute_hash();
        Self { hash, ..spec }
    }
}
```

### 7.3 Validation (Future Work)

**Note:** The validation function below is specified but NOT implemented in Phase 0.5.3.
`DeltaError::InvalidEventRef` and `DeltaError::InvalidHash` are defined but not yet used.
This will be implemented in a future phase when event store integration is added.

```rust
// FUTURE: Not implemented in Phase 0.5.3
pub fn validate_deltaspec(delta: &DeltaSpec, store: &EventStore) -> Result<(), DeltaError> {
    match &delta.kind {
        DeltaKind::InputMutation { delete, modify, ... } => {
            // Ensure deleted/modified events exist
            for id in delete {
                if store.get(id).is_none() {
                    return Err(DeltaError::InvalidEventRef(*id));
                }
            }
            ...
        }
        ...
    }

    // Verify hash matches
    let computed = delta.compute_hash();
    if computed != delta.hash {
        return Err(DeltaError::InvalidHash);
    }

    Ok(())
}
```

## 8. Future Extensions

### Extension 1: Composite Deltas

Allow multiple deltas to be combined:

```rust
DeltaKind::Composite {
    deltas: Vec<DeltaSpec>,
}
```

### Extension 2: Probabilistic Deltas

For fuzzing and Monte Carlo analysis:

```rust
DeltaKind::Probabilistic {
    distribution: Distribution,
    seed: u64,
}
```

### Extension 3: Temporal Constraints

Deltas that apply only within time windows:

```rust
DeltaSpec {
    valid_from: Time,
    valid_until: Time,
    ...
}
```

## 9. References

- SPEC-0001: Canonical Encoding Standard
- ARCH-0009: Materialized Time & Counterfactuals
- RFC 8949: Concise Binary Object Representation (CBOR)
- BLAKE3 Specification

## 10. Open Questions

1. **Q:** Should DeltaSpec include provenance (who created it, when)?
   **A:** Not in v1. Provenance is tracked at the Fork event level, not in DeltaSpec itself.

2. **Q:** How to handle deltas that reference non-existent policies?
   **A:** Validation step MUST catch this. PolicyHash must resolve in PolicyStore.

3. **Q:** Can deltas be nested/recursive?
   **A:** Not in v1. Extension 1 (Composite Deltas) addresses this if needed.

4. **Q:** How to merge counterfactual branches?
   **A:** Out of scope for DeltaSpec. Merge is an event type that references multiple parent worldlines.
