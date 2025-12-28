# JITOS Next Moves: Execution Plan (CORRECTED)

**Author:** Claude (Sonnet 4.5)
**Date:** 2025-12-28 (UPDATED)
**Status:** Execution Roadmap - Foundation-First Edition
**Context:** Phase 0.5.3 (DeltaSpec) complete - 3/6 foundational commits shipped

---

## Critical Course Correction

**Original Sin:** The first version of this plan treated branching, time-as-view, and canonical encoding as "later plumbing." They are not plumbing. **They are the axles.**

**The Three Holes:**
1. **Linear WAL in a Branching World:** Planned a hash chain, need a DAG
2. **Time as Port Instead of View:** Treated time as "just don't cheat," need materialized Clock View
3. **No Canonical Encoding:** Mentioned BLAKE3 without canonical serialization = hash divergence

**The Fix:** New Phase 0.5 (Foundational Semantics) that MUST complete before touching scheduler, graph, or ninelives.

---

## Phase 0: Documentation & Cleanup (UNCHANGED)

Same as before - clean docs, finalize SOTU. No changes needed here.

---

## **Phase 0.5: Foundational Semantics (NEW - CRITICAL)**

**Duration:** Week 1 (frontload before all other work)
**Why:** These are not features. These are the rules of reality. Get them wrong and everything built on top is cosmetic.

### 0.5.1 Define Canonical Encoding Standard

**The Problem:** BLAKE3(non-canonical-bytes) = hash divergence across runtimes, browsers, replay sessions.

**The Fix:**
- [x] Choose encoding: **Canonical CBOR (RFC 8949)** for ledger/events/archives
- [x] Create `jitos-core/src/canonical.rs` with strict rules:
  - [x] Map keys sorted lexicographically
  - [x] Definite-length encoding (no streaming)
  - [x] Canonical float representation (NaN → 0x7FF8_0000_0000_0000)
  - [x] No duplicate keys (rejected by strict decoder)
- [x] Generate test vectors (28 tests: 16 unit + 12 integration covering edge cases)
- [x] Add compliance test: `serialize(deserialize(bytes)) == bytes` (multiple round-trip tests)

**Acceptance Criteria:**
- [x] Same logical structure → identical bytes on all platforms (guaranteed by canonical encoding)
- [ ] Test vectors pass in Chrome, Firefox, Safari, Node, Deno (TODO: CI matrix for cross-platform testing)
- [x] Document "what breaks determinism" guide in SPEC-0001-canonical-encoding.md

---

### 0.5.2 Define Event Envelope with DAG Structure (v2 - Policy as Structure)

**The Problem (v1):** Linear hash chain can't explain counterfactual divergence cleanly.

**The DEEPER Problem (Discovered in PR #7 Review):**
- policy_hashes stored in event but not hashed → breaks content-addressing
- Nonce creates non-deterministic IDs for replay
- No structural enforcement of policy-decision relationship

**The v2 Fix (Policy as First-Class Event):**
- [x] Create `jitos-core/src/events.rs` with 4 event types (MINIMUM VIABLE):

```rust
/// Four fundamental event types - no more, no less
pub enum EventKind {
    /// Observation: Facts about the world (may be wrong/contradicted)
    /// Examples: clock samples, network messages, user inputs
    Observation,

    /// PolicyContext: How reality is interpreted (immutable rulebook)
    /// Examples: clock_policy="trust_ntp", scheduler_policy="fifo"
    PolicyContext,

    /// Decision: Interpretive choice given evidence + policy
    /// INVARIANT: Must have exactly ONE PolicyContext parent
    /// Examples: "fire timer", "apply rewrite R", "select event X"
    Decision,

    /// Commit: Irreversible effect (crossed system boundary)
    /// INVARIANT: Must have at least ONE Decision parent + signature
    /// Examples: timer fired, packet sent, disk write
    Commit,
}

/// Event ID: BORING HASHING (no nonces, no timestamps, no metadata)
/// event_id = H(kind || payload || sorted_parents)
///
/// If it affects semantics, it's a parent. If it's not a parent, it doesn't affect identity.
```

**Key Design Decisions:**
1. **Policy is structural (parent event) not metadata**: Decisions reference PolicyContext as parent
2. **No nonces**: Event uniqueness comes from causal structure (parents), not entropy
3. **Typed constructors enforce invariants**: `new_decision(evidence, policy_parent)` requires policy
4. **Private fields prevent mutation**: Can't change event_id after construction
5. **Validation pass**: `validate_event()` catches invalid structures from imports/migrations

**Acceptance Criteria:**
- [x] Event DAG can represent linear history (1 parent) - 20 tests passing
- [x] Decisions structurally depend on PolicyContext (enforced at construction)
- [x] Event DAG can represent merge (multiple parents) - parent canonicalization tested
- [x] `event_id` computation is deterministic and collision-resistant - comprehensive test
- [x] Different policy → different event_id (no hash collision) - explicit test
- [x] CanonicalBytes prevents non-canonical data - private field enforced
- [x] Validation catches invalid structures - 8 negative tests
- [x] All tests passing (69 total: 57 unit + 12 integration)

---

### 0.5.3 Define DeltaSpec for Counterfactuals

**The Problem:** "What if the packet arrived 10ms later?" needs formal expression.

**The Fix:**
- [x] Create `jitos-core/src/delta.rs`:

```rust
/// Describes a controlled violation of history
#[derive(Serialize, Deserialize, Clone)]
pub struct DeltaSpec {
    /// What kind of counterfactual
    pub kind: DeltaKind,

    /// Human-readable justification
    pub description: String,

    /// Hash of this spec (for branch.fork events)
    pub hash: Hash,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum DeltaKind {
    /// Change scheduler policy (e.g., FIFO → LIFO)
    SchedulerPolicy { new_policy: PolicyHash },

    /// Inject/modify/delete input event
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
```

**Acceptance Criteria:**
- [x] Can express "same inputs, different schedule"
- [x] Can express "same schedule, different inputs"
- [x] Can express "same inputs, different clock policy"
- [x] DeltaSpec is canonical-encodable and content-addressable

---

### 0.5.4 Implement Clock View (jitos-views)

**The Problem:** `ClockPort` isn't enough - time must be a **materialized view** not a syscall.

**The Fix:**
- [ ] Create `jitos-views/` crate
- [ ] Define Clock as a fold over events:

```rust
/// Clock is NOT a syscall. It's a pure function over events.
pub struct ClockView {
    /// Accumulated clock samples
    samples: Vec<ClockSample>,

    /// Current belief about time (updated by policy)
    current: Time,

    /// Policy that interprets samples
    policy: ClockPolicyHash,
}

impl ClockView {
    /// Pure fold - no side effects
    pub fn apply_event(&mut self, event: &EventEnvelope) {
        match event.kind {
            EventKind::Claim => {
                if let Some(sample) = parse_clock_sample(&event.payload) {
                    self.samples.push(sample);
                    self.current = self.policy.interpret(&self.samples);
                }
            }
            _ => {}
        }
    }

    /// Query time at a specific cut (worldline position)
    pub fn now_at_cut(events: &[EventEnvelope], cut: usize, policy: ClockPolicyHash) -> Time {
        let mut view = ClockView::new(policy);
        for event in &events[..cut] {
            view.apply_event(event);
        }
        view.current
    }
}

pub struct Time {
    pub ns: u64,
    pub uncertainty_ns: u64,
    pub provenance: Vec<Hash>, // which events contributed
}

pub struct ClockSample {
    pub source: ClockSource,
    pub value_ns: u64,
    pub uncertainty_ns: u64,
}

pub enum ClockSource {
    Monotonic,
    Rtc,
    Ntp,
    PeerClaim,
}
```

**Acceptance Criteria:**
- `now()` is a query, not a syscall
- Same events + same policy → same time belief
- Different policy → different time belief (with delta_spec)
- Replay never touches host clock

---

### 0.5.5 Implement Deterministic Timers

**The Problem:** `sleep(5s)` becomes "schedule a wake at logical time T" not "wait 5s of wall-clock."

**The Fix:**
- [ ] Add to `jitos-views`:

```rust
pub struct TimerView {
    requests: Vec<TimerRequest>,
    fired: Vec<TimerFired>,
}

pub struct TimerRequest {
    pub request_id: Hash,
    pub duration_ns: u64,
    pub requested_at: Time, // from ClockView
}

impl TimerView {
    pub fn check_timers(&mut self, current_time: Time, scheduler: &mut Scheduler) {
        for req in &self.requests {
            if !self.fired.contains(&req.request_id)
               && current_time.ns >= req.requested_at.ns + req.duration_ns {
                // Emit a Decision event
                scheduler.emit_event(EventKind::Decision, TimerFired {
                    request_id: req.request_id,
                    fired_at: current_time,
                });
                self.fired.push(req.request_id);
            }
        }
    }
}
```

**Acceptance Criteria:**
- `sleep(5s)` → `timer.request` event + scheduler wake when `ClockView.now() >= start + 5s`
- Replay fires timers at identical logical times
- No hidden host timers

---

### 0.5.6 Deterministic ID Allocation

**The Problem:** `slotmap` key allocation order depends on execution order → antichain swap breaks hash equality.

**The Fix:**
- [ ] Create `jitos-graph/src/ids.rs`:

```rust
/// IDs MUST be deterministic based on normalized schedule, not allocation order
pub struct DeterministicIdAllocator {
    tick_hash: Hash, // H(normalized_rewrite_set)
    counter: u64,
}

impl DeterministicIdAllocator {
    pub fn new_for_tick(rewrites: &[Rewrite]) -> Self {
        // Sort rewrites deterministically (by scope_hash, rule_id)
        let mut sorted = rewrites.to_vec();
        sorted.sort_by_key(|r| (r.scope_hash, r.rule_id));

        let tick_hash = blake3::hash(&canonical_encode(&sorted));

        Self { tick_hash, counter: 0 }
    }

    pub fn alloc_node_id(&mut self, rewrite_id: Hash) -> NodeId {
        let id_bytes = blake3::hash(&canonical_encode(&(
            self.tick_hash,
            rewrite_id,
            self.counter,
        )));
        self.counter += 1;
        NodeId::from_hash(id_bytes)
    }
}
```

**Acceptance Criteria:**
- Antichain swap (same tick, different order) → identical node IDs
- Node IDs reproducible on replay
- Test: "swap independent rewrites 1000 times → same graph hash every time"

---

### 0.5.7 The 6 Foundational Commits

**Before touching Phase 1, ship these:**

1. **[DONE]** Canonical Encoding Standard (`jitos-core/src/canonical.rs` + 28 test vectors) ✅
   - PR #7: Merged 2025-12-27
   - Status: ✅ Shipped with comprehensive test coverage

2. **[DONE]** Event Envelope v2 (`jitos-core/src/events.rs` - 4 types, policy as parent, 69 tests) ✅
   - PR #8: Merged 2025-12-28
   - Status: ✅ Shipped with validation and parent canonicalization

3. **[DONE]** DeltaSpec (`jitos-core/src/delta.rs` - counterfactual specification, 11 tests) ✅
   - PR #9: Merged 2025-12-28
   - Features: 4 constructors, custom Deserialize with hash validation, finalize() helper
   - Status: ✅ Shipped with full hash integrity enforcement

4. **[TODO]** Clock View (`jitos-views/src/clock.rs` with Time as fold)
   - Status: Not started
   - Blockers: None - ready to begin

5. **[TODO]** Timer Semantics (`jitos-views/src/timers.rs` with request/fire events)
   - Status: Not started
   - Depends on: Clock View (0.5.4)

6. **[TODO]** Deterministic IDs (`jitos-graph/src/ids.rs` tied to normalized schedule)
   - Status: Not started
   - Blockers: None - ready to begin

### Progress: 3/6 foundational commits complete (50.0%)

**Golden Test:**
```rust
#[test]
fn end_to_end_determinism() {
    let ledger = generate_random_event_dag(1000);
    let mut hashes = vec![];

    for _ in 0..1000 {
        let state = replay(&ledger);
        hashes.push(state.hash());
    }

    // ALL hashes MUST be identical
    assert!(hashes.windows(2).all(|w| w[0] == w[1]));
}
```

---

## Phase 1: Core Infrastructure (REORDERED)

**Now that foundations are locked, proceed with:**

### 1.1 Port Echo Scheduler (SAME, but with deterministic IDs)
- Use `DeterministicIdAllocator` for all node creation
- Implement `normalize()` using canonical rewrite ordering

### 1.2 Port Echo Graph (SAME, but content-addressed by canonical encoding)
- Use canonical CBOR for all serialization
- Implement `hash()` using sorted canonical IDs

### 1.3 Implement Inversion Engine (SAME, but branch-aware)
- Use event DAG for tracking SWS fork points
- Merge generates multi-parent events

### 1.4 Implement Provenance (SAME, but with new event format)
- Use `EventEnvelope` as WAL entry format
- Support DAG traversal, not just linear iteration

---

## Phase 2: Resilience & Policy (DEPENDS ON CLOCK VIEW)

### 2.1 Port ninelives to jitos-resilience

**Critical:** DO NOT START until Clock View exists.

**Refactors:**
- Replace `Instant` with `ClockView.now_at_cut()`
- Replace `rand::thread_rng()` with `DeterministicRng::from_ledger_seed()`
- Rate limiting uses `ClockView` time, not tick counts
- Backoff uses `ClockView` + `timer.request` events

### 2.2 Implement jitos-policy (SAME)

### 2.3 Implement jitos-io (DEPENDS ON CLOCK VIEW)

**All ports inject events:**
- `fs.device_read` → Input event
- `net.recv` → Input event
- `clock.sample` → Claim event (consumed by ClockView)

---

## Phases 3-6: UNCHANGED

Same as original plan, but now they're built on solid foundations.

---

## Updated Critical Success Metrics

### Determinism
- [x] Canonical encoding test vectors implemented (28 tests, all passing locally)
- [ ] Canonical encoding cross-platform validation (TODO: CI matrix for Chrome/Firefox/Safari/Node/Deno)
- [ ] Golden test: Same ledger replayed 1000x → identical hashes
- [ ] Antichain swap test: Independent rewrites reordered 1000x → identical hash
- [ ] Cross-platform: x86-64/ARM64 produce identical hashes

### Branching & Counterfactuals
- [ ] Can fork worldline with explicit DeltaSpec
- [ ] Can merge branches with multi-parent events
- [ ] Debugger can visualize causal braid (DAG, not line)

### Time Semantics
- [ ] `now()` is a pure query over events
- [ ] Timers fire at identical logical times on replay
- [ ] Clock policy changes (via DeltaSpec) produce different time beliefs

---

## Revised Open Questions

1. **Camera Policy:** Still recommend Policy B (presentation-only)
2. **Floating-Point:** Still recommend platform restriction (x86-64, ARM64) OR explicit rounding at commit
3. **Snapshot Frequency:** Adaptive (dense early, sparse later)
4. **BTR Format:** **Canonical CBOR** (locked)
5. **Engineering Effort:** Still ~500 hours, but Phase 0.5 is now 1-2 weeks of critical path

---

## Conclusion

The original roadmap had good hustle but weak foundations. This corrected version:

1. **Locks canonical encoding** before signing anything
2. **Makes branching first-class** before building debugger
3. **Treats time as view** before refactoring ninelives
4. **Fixes ID allocation** before testing determinism

**The axles are now solid. The rest is good hustle on good rails.**

**Next Immediate Action:** Execute Phase 0.5 (all 6 foundational commits) in Week 1, THEN proceed to Phase 1.
