---
Status: Draft
Author: James Ross
Contributors: JIT Community
Start Date: 2025-12-25
Target Spec: JITOS v0.x
License: TBD
---

# JIT RFC-0022

## Materialized Time & I/O Views for Deterministic Replay and Counterfactual Execution

### 0. Status

Draft

### 1. Purpose

Define how JITOS represents time, I/O, and other "externalities" as materialized, observer-relative views over an append-only worldline, and specify how this model supports:
*   Time-travel debugging (replay, inspect, rewind, bisect)
*   Counterfactual execution (fork a worldline, try alternate inputs/schedules, compare outcomes)
*   Deterministic execution guarantees (same worldline + same observer config ⇒ same results)

### 2. Core Principles

1.  **Worldline primacy:** The worldline is the only authoritative record.
2.  **No privileged side effects:** "Time", "I/O", "randomness", etc. are derived from events, not performed as opaque mutations.
3.  **Observer relativity:** "Now", "filesystem state", and "network state" are projections defined by an observer.
4.  **Determinism by construction:** Replays must not consult ambient OS state (host wall clock, real network, etc.) except through explicitly logged events.

### 3. Terminology

*   **Worldline:** Append-only causal DAG of events and derived states.
*   **Event:** Immutable record appended to the worldline (inputs, decisions, observations).
*   **Materialized View:** A cached fold over a subset of events, producing a queryable state.
*   **Observer:** A configuration that defines view policies, trust assumptions, and resource bounds.
*   **Branch (Counterfactual Worldline):** A new worldline rooted at some prior point with altered inputs/policies/schedule.
*   **Schedule:** The deterministic rule that selects the next reducible rewrite / task / continuation.

### 4. Data Model

#### 4.1 Event Classes

All externality-facing behavior is represented as one of:

**A) Input events (from outside JITOS)**
*   `net.recv`, `fs.device_read`, `clock.sample`, `entropy.sample`, `user.input`, etc.

**B) Decision events (chosen by JITOS)**
*   `sched.choose { runnable_set_hash, chosen_id, reason }`
*   `retry.decide { op_id, attempt, backoff_ns }`

**C) Claim events (untrusted assertions)**
*   `clock.claim { source, claimed_time, confidence }`
*   `net.claim_peer_time { … }`

**D) Commit/anchor events (trust boundaries)**
*   `anchor.commit { hash, signature, upstream_ref }` (optional but recommended)

#### 4.2 View State Records

Views materialize state as immutable snapshots in the worldline (optional caching) or computed on-demand:
*   `view.clock.state`
*   `view.fs.state`
*   `view.net.state`
*   `view.entropy.state`

A view state must be reproducible solely from:
*   worldline events up to some cut
*   observer policy/config
*   deterministic computation

### 5. The Clock View

#### 5.1 Clock Is Not "Ticking"

There is no ambient ticking. "Time" advances only through clock-related events and scheduler steps.

Two common modes:

**Mode 1: Logical time**
*   time advances by deterministic increments tied to execution steps and logged decisions.
*   Great for pure replay/counterfactual.

**Mode 2: Sampled physical time**
*   Real-world time enters only via logged samples:
*   `clock.sample { source: monotonic|rtc|ntp, value_ns, uncertainty_ns }`

#### 5.2 Clock Query Semantics

`now(observer, cut)` returns a value produced by:
1.  Gather clock events ≤ cut
2.  Apply `observer.clock_policy` (Rhai)
3.  Return `Time { ns, uncertainty, provenance }`

**Important:** `now()` is a query, not a syscall.

#### 5.3 Clock Policy (Rhai)

Clock policy is a pure function:
*   Inputs: recent clock events, drift model params, last state
*   Output: next clock state

Policy must be deterministic and side-effect free.

Example responsibilities:
*   smoothing / monotonic enforcement
*   NTP step vs slew rules
*   confidence/uncertainty propagation

### 6. I/O Views

#### 6.1 Filesystem View

Filesystem is a fold over:
*   `fs.write`, `fs.mkdir`, `fs.unlink`, `fs.rename`, etc.
plus optional device constraints recorded as events.

Reads are resolved as:
*   `fs.read(path, cut, observer) ⇒ bytes + provenance`
No host filesystem reads unless they were captured into worldline events.

#### 6.2 Network View

Network is modeled as streams of `net.recv` input events and `net.send` intent events.
*   `net.send` is an event describing an attempted emission.
*   Delivery is not assumed; delivery only exists if a corresponding `net.recv` occurs (possibly in another worldline).

This separation is what makes counterfactual networking sane.

#### 6.3 Randomness / Entropy View

Randomness enters as `entropy.sample` events (or a deterministic PRNG seeded by an event).

For counterfactuals:
*   Either re-use the same entropy samples (to isolate schedule differences)
*   Or fork new entropy stream intentionally (to explore stochasticity)

This MUST be explicit.

### 7. Time-Travel Debugging

#### 7.1 Cuts

A **cut** is a reference to a point in the worldline:
*   event id
*   hash
*   logical step count
*   (optional) clock-resolved time via the clock view

All debugging operations are "as-of cut" queries.

#### 7.2 Core Debug Operations
*   `inspect(cut, query)` — query any view as-of cut
*   `step(cut)` — advance by one deterministic reduction (produces new events)
*   `rewind(cut, n)` — choose an earlier cut
*   `bisect(predicate)` — deterministic search over cuts
*   `trace(var|addr|node)` — provenance walk in the worldline graph

#### 7.3 Deterministic Replay Requirement

Replay is valid iff:
*   all inputs are present as events
*   schedule decisions are present OR reproducible from policy + runnable set hash
*   observer policies are identical (or deliberately changed and recorded)

If any query consults host state not captured as events, replay is invalid.

### 8. Counterfactual Execution (Branching)

#### 8.1 Branch Definition

A branch is:
*   `branch_id`
*   `base_cut`
*   `delta set` (what changes)

Delta may include:
*   different schedule policy
*   modified/inserted input events
*   altered trust policies (e.g., "treat NTP as untrusted")
*   alternate device constraints

#### 8.2 Counterfactual Rule

A counterfactual worldline must preserve:
*   all base events ≤ base_cut
*   then diverge only via explicitly declared deltas and subsequent derived events

No silent divergence. Every "what if" must be attributable to a delta.

#### 8.3 Three Counterfactual Modes (pick intentionally)

**Mode A: Same inputs, different schedule**
*   isolates race/ordering bugs

**Mode B: Same schedule, different inputs**
*   explores input sensitivity

**Mode C: Same inputs, different observer policies**
*   explores "what if we trusted/didn't trust X", or different clock smoothing, etc.

#### 8.4 Merging / Comparing Branches

Merging is not "git merge." It's analysis.

Provide comparison primitives:
*   divergence point (first differing event hash)
*   output diffs (view outputs, state hashes)
*   causality explanation (minimal delta explanation)

A "merge" that creates a new worldline is allowed only if it is represented as:
*   `analysis.merge { base_a, base_b, justification, chosen_path }`
and the chosen path is itself deterministic.

### 9. Interaction Between Time View and Counterfactuals

#### 9.1 "Now" Is Branch-Relative

Because clock is a view, the same base cut can yield different `now()` values if:
*   clock policy differs
*   clock inputs differ
*   schedule decisions differ (logical time depends on step counts)

That's a feature: you can replay time and reshape time without lying.

#### 9.2 Time Travel With External Time Inputs

If your base worldline includes `clock.sample` from real hardware:
*   Replays reproduce those samples exactly.
*   Counterfactual branches may:
    *   keep them (to compare schedule changes under same "time")
    *   replace them (to explore different time environments)

Both must be encoded as explicit deltas.

#### 9.3 Time-Based APIs

Any API that would normally block on time must be modeled as:
*   a scheduled wait decision + a clock view query

Example:
*   `sleep(5s)` becomes:
    *   `timer.request { duration_ns }`
    *   scheduler steps until `clock_view(now) >= start + duration`
    *   every decision is logged, so replay is identical

No hidden host timers.

### 10. Required Interfaces

#### 10.1 Kernel/Internal APIs
*   `worldline.append(event) -> EventId`
*   `worldline.cut(ref) -> Cut`
*   `view.eval(view_name, observer, cut) -> ViewState`
*   `schedule.next(observer, cut, runnable_set) -> Choice (logged)`

#### 10.2 User-Space APIs (Conceptual)
*   `jitos_now() -> Time`
*   `jitos_fs_read(path) -> Bytes`
*   `jitos_net_send(pkt) -> SendReceipt`
*   `jitos_branch(base_cut, delta_spec) -> branch_id`
*   `jitos_replay(branch_id, to_cut|steps)`

### 11. Policy Scripting (Rhai)

#### 11.1 Constraints

Policies must be:
*   deterministic
*   pure (no I/O)
*   total (must return something for any valid input)
*   versioned and hash-addressed

#### 11.2 Versioning & Provenance

Observer configuration must include:
*   policy script hashes
*   policy params
*   allowed capabilities

Store these in:
*   `observer.define { id, policy_hashes, params_hash, limits }`

So every replay knows exactly "what mind was interpreting reality."

### 12. Security & Trust
*   Untrusted inputs remain untrusted unless transformed by policy and anchored.
*   Trust transitions must be explicit events.
*   Remote/host integration must happen via capture adapters that emit input events, never by letting policies read host state.

### 13. Non-Goals
*   Perfect modeling of physical time (we model evidence of time, not metaphysics)
*   Transparent "live networking" without capture (that's just nondeterminism wearing a trench coat)

---

### The critical "don't screw this up" note

> If any part of the system is allowed to call host `clock_gettime()`, read host `/dev/random`, or do raw socket I/O without emitting events, you've created a nondeterministic wormhole that will eat debugging, reproducibility, and rights-bearing provenance. Don't do it.
