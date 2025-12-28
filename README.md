# Loom (formerly JITOS)

> **Status:** Early Stage - Foundational Phase
> **Current Progress:** 3/6 core foundations complete

**Loom** is a history-native, deterministic computational system where execution is woven into an immutable fabric. This is the foundation layer for a new kind of computing where history is structural, not incidental.

```
THE REVΩLUTION WILL BE DETERMINISTIC
```

---

## Project Status

We're building the **axles** before the **wheels**. The current focus is foundational semantics that make determinism possible.

### Phase 0.5: Foundational Semantics (In Progress)

| Component | Status | Details |
|-----------|--------|---------|
| **Canonical Encoding** | ✅ Shipped | CBOR-based deterministic serialization (PR #7) |
| **Event Envelope DAG** | ✅ Shipped | Content-addressed event graph with 4 event types (PR #8) |
| **DeltaSpec** | ✅ Shipped | Counterfactual specification with hash validation (PR #9) |
| **Clock View** | 🔨 Next | Time as materialized view over events |
| **Timer Semantics** | 🔜 Planned | Deterministic timers (depends on Clock View) |
| **Deterministic IDs** | 🔜 Planned | Content-addressed node allocation |

**Progress:** 3/6 foundational commits complete (50%)

---

## What Works Right Now

### 1. Canonical Encoding (SPEC-0001)

Deterministic, cross-platform serialization:

```rust
use jitos_core::canonical;

let data = vec![1, 2, 3];
let bytes = canonical::encode(&data)?;
// Same logical structure → identical bytes (always)
```

- **28 test vectors** covering edge cases
- **CBOR-based** with strict ordering guarantees
- **Cross-platform determinism** (x86-64, ARM64, WASM)

### 2. Event Envelope DAG (SPEC-0001-events)

Content-addressed event graph with structural invariants:

```rust
use jitos_core::events::*;

// Create policy context (immutable rulebook)
let policy = EventEnvelope::new_policy_context(
    b"clock_policy=trust_ntp".to_vec(),
    vec![], // no parents (genesis)
    None,
)?;

// Create decision referencing that policy
let decision = EventEnvelope::new_decision(
    b"fire_timer_123".to_vec(),
    vec![policy.event_id()], // policy as parent
    None,
)?;
```

**Features:**
- 4 event types: Observation, PolicyContext, Decision, Commit
- Parent canonicalization (sorted, deduplicated)
- Custom Deserialize validation
- 60 unit tests + 12 integration tests

### 3. DeltaSpec (SPEC-0002)

Formal counterfactual specification:

```rust
use jitos_core::delta::*;

// "What if we used LIFO instead of FIFO?"
let delta = DeltaSpec::new_scheduler_policy(
    policy_hash,
    "Test race bug with reversed task order".to_string(),
)?;

// Hash is content-addressed and validated on deserialize
assert_eq!(delta.hash(), delta.compute_hash()?);
```

**Features:**
- 4 delta kinds: SchedulerPolicy, InputMutation, ClockPolicy, TrustPolicy
- Custom Deserialize with hash integrity validation
- Prevents spoofed delta references
- 11 tests including tampered hash rejection

---

## Architecture Vision

Loom is being built on **WARP Graphs (Worldline Algebra for Recursive Provenance)** with **Double-Pushout (DPO)** rewriting semantics.

### Core Concepts (Planned)

**The Loom (Fabric)**
The realized, immutable history of execution. Append-only event DAG.

**The Stylus (Commit)**
The mechanism that performs irreversible writes. Does not calculate; finalizes.

**The Scheduler (Constraint)**
Governs when the Stylus may act. Determines admissible trajectories.

**The Umbra (Shadow Index)**
Structured archive of unrealized possibilities. Valid alternatives are queryable.

---

## Getting Started

```bash
# Clone the repository
git clone https://github.com/flyingrobots/loom.git
cd loom

# Build (requires Rust 1.75+)
cargo build

# Run tests
cargo test

# Current test results:
# - 72 passing tests (60 unit + 12 integration)
# - 0 warnings
```

---

## Current Codebase Structure

```
crates/
├── jitos-core/          ✅ Foundations (shipped)
│   ├── canonical.rs     ✅ Deterministic CBOR encoding
│   ├── events.rs        ✅ Event DAG with 4 event types
│   └── delta.rs         ✅ Counterfactual specifications
├── jitos-graph/         🔨 Placeholder (basic structure only)
└── jitos-scheduler/     🔨 Placeholder (basic structure only)
```

---

## What's Coming Next

**Immediate Next Steps (Phase 0.5.4-0.5.6):**

1. **Clock View** - Time as pure function over events (no syscalls)
2. **Timer Semantics** - Deterministic timer request/fire mechanism
3. **Deterministic IDs** - Content-addressed node allocation tied to schedule

See [NEXT-MOVES.md](./NEXT-MOVES.md) for detailed roadmap.

---

## Design Principles

**Why these foundations matter:**

1. **Determinism requires canonical encoding** - Same structure must produce identical bytes across all platforms/runtimes
2. **Branching requires event DAG** - Linear hash chains can't express counterfactual divergence
3. **Replay requires time-as-view** - Can't use syscalls; time must be materialized from events
4. **Content-addressing requires hash validation** - Must prevent spoofed references

These aren't features. **These are the axles that everything else rotates on.**

---

## History

This project was previously known as **JITOS**. The name changed when the architecture converged on its true form: a machine that orders threads to weave the fabric of reality.

JITOS served its purpose. **Loom** is the name that fits.

---

## Part of AIΩN

Loom is the execution model for [AIΩN](https://github.com/flyingrobots/aion)—a unified, deterministic computational framework where history is the primary artifact and state is merely a derived view.

---

## License

[Apache 2.0](./LICENSE) — Use it, fork it, ship it.

---

## Contributing

We're in foundational phase. The best way to contribute right now is to:

1. Review the specs in `docs/SPECS/`
2. Run the tests and report issues
3. Wait for Phase 1 (core infrastructure) before major feature work

---

<p align="center">
  <strong>THE REVOLUTION WILL BE DETERMINISTIC</strong>
</p>
