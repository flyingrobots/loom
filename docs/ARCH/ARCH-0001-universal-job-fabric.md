# ARCH-0001: JITOS as a Universal Job Fabric

**Status:** Draft
**Date:** 2025-12-29
**Owner:** James
**Related:** Echo / WARP / Scheduler notes; TASKS/SLAPS; SWS worker pool

## Thesis

Traditional PM tools and traditional OS primitives fail for the same reason: they lie about work. They treat progress as a linear list of “states” instead of a causal history of decisions, attempts, constraints, and commits.

JITOS models all work as jobs over a causal graph:
*   Intent is declared (TASK)
*   A plan is proposed (SLAP)
*   Execution happens in speculative overlays (SWS)
*   The system produces an immutable provenance history (rewrites/events)
*   Only then do outcomes become “real” via collapse/commit

If an OS can’t tell you why something happened, it’s not a system — it’s a haunted house with logs.

## Why this architecture

This design intentionally rhymes with a few well-known ideas:
*   **Event sourcing:** store all changes as a sequence of events so you can reconstruct and replay state. That is the “history-first” backbone here.
*   **Overlay / copy-on-write layers:** speculative changes live in an upper layer that can be merged or discarded. SWS is “OverlayFS, but for causal state.”
*   **HTN planning:** decompose high-level goals into primitive executable steps with ordering constraints. That’s the TASKS/SLAPS planning model.
*   **Microkernel instinct (eventually):** keep the kernel core minimal and push “drivers/workers” out. Start monolithic for speed; keep boundaries crisp so you can split later.

(We are not cargo-culting these patterns. We’re stealing the good parts and refusing the rest.)

---

## Definitions

**WARP**
Rewrite MultiGraph (name TBD): the canonical graph structure representing state and its transformation history via rewrites.

**Rewrite**
An atomic, append-only state transition applied to an WARP. A rewrite is the unit of provenance.

**System WARP**
The canonical persistent “base reality” graph owned by the kernel.

**SWS (Schrödinger Workspace)**
A speculative, copy-on-write overlay over a snapshot of the system WARP. It is where risky work happens.

**TASK**
A declaration of intent (“make X true”), not an instruction for how.

**SLAP**
A proposed plan (possibly one of many) for satisfying a TASK. SLAPs are branchable, revisable, and auditable.

**Worker**
An executor of primitive operations: scripts, LLMs, tool adapters, humans-in-the-loop, etc.

**Collapse**
Transactional merge of an SWS overlay into the system WARP (commit). Discard is the inverse (abort).

---

## Non-negotiable invariants

These are laws. If a change violates one, it’s not a “refactor,” it’s a fork of the project.

1.  **History is first-class.**
    State is derived from rewrites/events; we do not treat “current state” as authoritative without provenance.
2.  **Speculation is default.**
    Untrusted / risky / agent-driven work happens in SWS overlays, not directly in the system WARP.
3.  **Abort still produces knowledge.**
    A failed attempt is not “nothing happened.” It is an event in the system’s history. (We can choose how much to persist, but we don’t pretend it didn’t occur.)
4.  **Intent ≠ Plan ≠ Execution.**
    TASK declares what. SLAP proposes how. Workers perform primitive steps.
5.  **The kernel enforces policy; workers perform mechanism.**
    We start monolithic for velocity, but the architecture is intentionally separable (kernel vs worker execution boundary).
6.  **No “task-state cosplay.”**
    We do not build a kanban board and call it a kernel. “Status” is a view computed from the graph.

---

## Component architecture

### Logical layers

1.  **Kernel** (`echo-kernel` + `echo-sched` + `echo-WARP-core`)
    *   Owns system WARP
    *   Manages SWS lifecycle
    *   Runs scheduler ticks
    *   Enforces policy + permissions
    *   Exposes APIs: `submit_intent` / `submit_rewrite` / `query_state`
2.  **Workers** (`echo-workers`)
    *   Pluggable executors (LLMs, shell, adapters, humans)
    *   In-process for v0; out-of-process later
3.  **Clients** (`echo-net` + `jitos-cli` + viewer)
    *   CLI/TUI/GUI + visualization
    *   Communicate via RPC/socket

### Physical deployment (v0)
*   `jitosd`: single daemon process linking kernel + workers + net
*   Separate processes for CLI and viewer, talking to `jitosd`

This is the “monolith with seams” strategy: ship now, split later.

---

## Rust workspace layout

```
echo/
  Cargo.toml          # workspace
  crates/
    echo-WARP-core/   # WARP data structures + rewrite engine
    echo-sched/       # generic scheduler (ticks + rewrites)
    echo-kernel/      # JITOS kernel core (owns WARPs, SWS, processes)
    echo-tasks/       # TASKS + SLAPS + HTN planning -> DAG/job specs
    echo-workers/     # worker registry + invocation abstractions
    echo-net/         # RPC / protocol (gRPC, HTTP, unix socket)
    echo-viewer/      # WARP inspector / debugging UI
  bins/
    jitosd/           # daemon: kernel + net + workers
    jitos-cli/        # CLI client: talks to jitosd via echo-net
```

---

## Core data model

### Kernel ownership model
*   One canonical system WARP
*   Many SWS overlays (copy-on-write deltas) per process/job/agent attempt

**Suggested structs:**

```rust
struct Kernel {
    system_WARP: WARPInstance,                  // base reality
    sws_pool: HashMap<SwsId, SwsInstance>,    // overlays
    processes: HashMap<ProcessId, Process>,   // runtime handles
}

struct Process {
    id: ProcessId,
    sws_id: SwsId,
    caps: Capabilities,
    // metadata: owner, quota, TTL, etc
}

struct SwsInstance {
    parent_snapshot: WARPSnapshotId, // points at system snapshot
    overlay_WARP: WARPInstance,       // deltas only
}
```

This is conceptually identical to overlay/copy-up systems: reads see merged view; writes go to upper layer; merge commits deltas.

### SWS read/write semantics
*   **Read:** `view = merge(system_snapshot, overlay)`
*   **Write:** rewrite applies to overlay only
*   **Collapse:** compute/apply rewrite diff from overlay into system, transactionally
*   **Discard:** drop overlay (optionally keep audit trail)

### Conflict semantics (initial stance)
For v0:
*   Collapse is “best-effort transactional”
*   Conflicts are explicit failures requiring rebase/replan (i.e., generate a new SLAP or re-run primitives)

We can later add:
*   conflict-free merge rules for certain edge types
*   CRDT-like behavior for specific graph substructures (only if it pays rent)

---

## TASKS/SLAPS planning model

### Why HTN-ish decomposition
We need a planner that can take “Fix auth bug” and produce a structured, inspectable execution DAG without requiring an LLM.

That is literally what HTN planning is about: decompose compound tasks into primitive tasks with ordering constraints.

### Contract
*   **TASK** is an intent object written into the system graph
*   **SLAP** is a plan candidate (possibly multiple per TASK)
*   **Planner output** is a DAG of primitive tasks with:
    *   dependency edges
    *   required capabilities
    *   expected artifacts
    *   suggested workers

### Minimal API
*   `plan(task: Task, methods: MethodLibrary) -> Vec<SlapCandidate>`
*   `compile(slap: Slap) -> JobDag`

### Method library
*   Stored as data (YAML/JSON) + compiled to Rust structs
*   Deterministic planner first; allow “nondeterministic suggestions” later (LLM can propose methods, but the kernel should not depend on that)

---

## Execution model

### Scheduler loop (echo-sched)
The scheduler is a generic “tick & apply rewrites” engine:
1.  Observe graph state (system + relevant overlays)
2.  Select runnable primitive nodes (deps satisfied, caps ok, quotas ok)
3.  Emit rewrite(s) representing “dispatch”
4.  Worker executes
5.  Worker returns result as rewrite(s) into overlay
6.  Repeat

### Worker invocation
Workers are not trusted as truth. They are:
*   mechanisms that produce proposals/results
*   that must be recorded as rewrites
*   and may require validation gates before collapse

**Idempotence rule (strongly preferred):**
Primitive tasks should be written so retries are safe, or have explicit “already-done” detection.

---

## Policy and security stance

Even in v0, we treat “who/what can rewrite what” as core.

**Recommended direction:**
*   **Capability-style permissions:** processes carry explicit rights, not ambient authority (least privilege).
*   **Workers run with bounded capabilities** (filesystem, network, tool APIs)
*   **SWS boundaries are safety rails:** “do dumb stuff in the overlay, then prove it’s good”

(You can ship without the full capability model; you cannot ship without the architecture that allows it.)

---

## GraphQL Surface

*   `echo-net` implements GraphQL over HTTP for query/mutation following emerging “GraphQL over HTTP” guidance.
*   Subscriptions power the viewer and live tooling; prefer `graphql-transport-ws` for WebSocket transport.
*   Mutations are commands, never raw state edits.
*   Authorization & safety: depth limits, cost limits, persisted queries in non-dev modes.

---

## Build plan (fast dopamine, minimal regret)

### Phase 0 — Kernel skeleton
*   workspace + crates
*   system WARP + `submit_rewrite`
*   `jitosd` starts and exposes minimal API (HTTP/unix socket)

**Demo:** mutate and inspect a live system graph.

### Phase 1 — Viewer attaches to daemon
*   snapshot/streaming endpoint
*   live WARP visualization

**Demo:** “OS graph animating in real time.”

### Phase 2 — SWS overlays
*   `create_sws` / `apply_rewrite_sws` / `collapse_sws` / `discard_sws`
*   visualize overlays + diffs

**Demo:** parallel speculative workspaces like branches.

### Phase 3 — echo-tasks
*   SLAPS structs + validation
*   HTN-ish method library + deterministic planner
*   compile SLAP -> DAG

**Demo:** “intent in, DAG out.”

### Phase 4 — Integrate intent -> SWS -> execution
*   `submit_intent` -> ProcessId
*   write DAG into SWS graph
*   scheduler dispatches primitives

**Demo:** tasks appear, run, collapse.

### Phase 5 — Real workers
*   LocalScriptWorker
*   LLMWorker (optional)
*   stage code changes in SWS, test, collapse on green

**Demo:** “holy shit it fixed a trivial bug.”

---

## Explicit anti-patterns

*   “Just add a task table.” **No.** Tasks are nodes in the causal graph, not rows in a database.
*   “Status fields are the truth.” **No.** Status is derived, never authoritative.
*   “Workers mutate the world and we hope.” **No.** Workers propose rewrites; the kernel records and validates.
*   “Speculation is optional.” **No.** Speculation is the default safety model.

---

## Open questions

1.  What is the minimal rewrite schema that keeps history useful but doesn’t explode storage?
2.  How do we represent “confidence” and “validation gates” in the graph?
3.  What merge policy do we want for common artifact types (files, configs, structured nodes)?
4.  What’s the GC/compaction story for old overlays and old rewrite chains?
5.  How do we make “human-in-the-loop” a first-class worker type without turning into Jira?

---

## Appendix: Why this is an ARCH, not an ADR

ADRs are great for recording discrete decisions in a standard structure (title/status/context/decision/consequences). This document is not one decision. It’s a foundational thesis + invariants that future ADRs must not violate.
