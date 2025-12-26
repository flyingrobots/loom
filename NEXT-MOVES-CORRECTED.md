# JITOS Next Moves: Execution Plan (CORRECTED)

**Author:** Claude (Sonnet 4.5)
**Date:** 2025-12-26 (REVISED)
**Status:** Execution Roadmap - Foundation-First Edition
**Context:** Post-stakeholder feedback - fixing the three foundational holes

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
- [ ] Choose encoding: **Canonical CBOR (RFC 8949)** for ledger/events/archives
- [ ] Create `jitos-core/src/canonical.rs` with strict rules:
  - Map keys sorted lexicographically
  - Definite-length encoding (no streaming)
  - Canonical float representation (NaN → specific bit pattern)
  - No duplicate keys
- [ ] Generate test vectors (100+ edge cases)
- [ ] Add compliance test: `serialize(deserialize(bytes)) == bytes`

**Acceptance Criteria:**
- Same logical structure → identical bytes on all platforms
- Test vectors pass in Chrome, Firefox, Safari, Node, Deno
- Document "what breaks determinism" guide (e.g., "never use HashMap.iter()")

---

### 0.5.2 Define Event Envelope with DAG Structure

**The Problem:** Linear hash chain can't explain counterfactual divergence cleanly.

**The Fix:**
- [ ] Create `jitos-core/src/events.rs` with DAG-aware envelope:

```rust
/// The universal event envelope for the JITOS worldline DAG
#[derive(Serialize, Deserialize, Clone)]
pub struct EventEnvelope {
    /// Content-addressed ID: H(parents || canonical_payload || nonce)
    pub event_id: Hash,

    /// Parent event(s) - normally 1, >1 for merge/anchor
    pub parents: Vec<Hash>,

    /// Optional branch identifier (can be derived from parents but useful)
    pub branch_id: Option<BranchId>,

    /// Event classification
    pub kind: EventKind,

    /// The actual payload (MUST be canonically encoded)
    pub payload: CanonicalBytes,

    /// Who created this event
    pub agent_id: AgentId,

    /// Cryptographic signature over (event_id || payload)
    pub signature: Signature,

    /// Which policy/observer interpreted reality (for now() queries)
    pub policy_hashes: Vec<Hash>,

    /// Nonce for uniqueness (prevents duplicate event_id collisions)
    pub nonce: u64,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum EventKind {
    /// External input (user, device, network)
    Input,

    /// Kernel decision (schedule choice, timer fire)
    Decision,

    /// Untrusted assertion (clock sample, peer claim)
    Claim,

    /// Derived computation (rule application result)
    Derivation,

    /// Trust boundary crossing (signature verification, BTR anchor)
    Anchor,

    /// Branch fork
    BranchFork {
        base_event: Hash,
        delta_spec_hash: Hash,
    },

    /// Branch merge (multiple parents)
    BranchMerge {
        strategy: MergeStrategy,
    },
}
```

**Acceptance Criteria:**
- Event DAG can represent linear history (1 parent)
- Event DAG can represent fork (new branch_id, delta_spec)
- Event DAG can represent merge (multiple parents)
- `event_id` computation is deterministic and collision-resistant

---

### 0.5.3 Define DeltaSpec for Counterfactuals

**The Problem:** "What if the packet arrived 10ms later?" needs formal expression.

**The Fix:**
- [ ] Create `jitos-core/src/delta.rs`:

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
    SchedulerPolicy { new_policy: SchedulerPolicyHash },

    /// Inject/modify/delete input event
    InputMutation {
        insert: Vec<InputEvent>,
        delete: Vec<Hash>,
        modify: Vec<(Hash, InputEvent)>,
    },

    /// Change clock interpretation policy
    ClockPolicy { new_policy: ClockPolicyHash },

    /// Change trust assumptions
    TrustPolicy { new_trust_roots: Vec<AgentId> },
}
```

**Acceptance Criteria:**
- Can express "same inputs, different schedule"
- Can express "same schedule, different inputs"
- Can express "same inputs, different clock policy"
- DeltaSpec is canonical-encodable and content-addressable

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

1. **Canonical Encoding Standard** (`jitos-core/src/canonical.rs` + test vectors)
2. **Event Envelope** (`jitos-core/src/events.rs` with DAG structure)
3. **DeltaSpec** (`jitos-core/src/delta.rs` for counterfactuals)
4. **Clock View** (`jitos-views/src/clock.rs` with Time as fold)
5. **Timer Semantics** (`jitos-views/src/timers.rs` with request/fire events)
6. **Deterministic IDs** (`jitos-graph/src/ids.rs` tied to normalized schedule)

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
- [x] Canonical encoding test vectors pass on all platforms
- [ ] Golden test: Same ledger replayed 1000x → identical hashes
- [ ] Antichain swap test: Independent rewrites reordered 1000x → identical hash
- [ ] Cross-browser: Chrome/Firefox/Safari produce identical hashes
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
