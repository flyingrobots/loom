# ARCH-0000: Integrated Architecture Direction (Non-Normative)

**Status:** Draft (Directional / forward-looking)  
**Derived From:** ARCH-0001, ARCH-0002, ADRs, SPECS, ROADMAP  
**Scope:** A consolidated *directional* overview of where JITOS is going. This document is intended to reduce fragmentation and keep the “big picture” coherent.

**Important:** This document is **not** an implementation contract and must not be treated as a second “source of truth.”

If this document conflicts with:
- `docs/ARCH/ARCH-0002-architectural-invariants.md` (invariants), or
- `docs/SPECS/*` (contracts), or
- `docs/ROADMAP/*` (milestone gates),

then this document is wrong and must be updated.

For doc hierarchy, see: `docs/DOCS-GOVERNANCE.md`.

---

## 1. Core Thesis: The Causal Operating System

JITOS rejects the traditional OS model of "mutable state + logs." Instead, it asserts:
1.  **History is the System:** The authoritative record is the sequence of events (Rewrites), not the current memory image.
2.  **State is a View:** Any "current state" is a deterministic projection of history under a specific Policy.
3.  **Speculation is Default:** All risky work happens in **Shadow Working Sets (SWS)**—copy-on-write overlays—never on the live system graph.
4.  **Intent $\neq$ Execution:** We separate **Task** (what), **Slap** (plan), and **Worker** (primitive execution).

---

## 2. The Hierarchy of Reality (RMG Stacks)

JITOS is not a single monolithic graph. It is a stratified universe of scopes, mirroring the physics of local fields and global spacetime.

### 2.1. The Root OS RMG (Kernel Space)
The "God Graph." Contains:
*   Global authority and policies.
*   The System Device Tree.
*   The Global Causality Timeline (Chronos).
*   Process directory and security contexts.
*   **Invariant:** Only the Kernel can mutate the Root RMG directly (via "Syscalls").

### 2.2. Process RMG (Address Space)
When a process starts, it gets a fresh RMG.
*   Represents internal state/memory.
*   Isolated from the Root (mapped via capability edges).
*   Forks here are cheap (like `git branch` or `fork()`).
*   **Analogy:** A POSIX process address space or a WASM sandbox.

### 2.3. Shadow Working Set (SWS)
The ephemeral, speculative overlay where work actually happens.
*   **Read:** `merge(SystemSnapshot, OverlayDeltas)`
*   **Write:** Applies deltas to the Overlay only.
*   **Collapse:** A transactional commit that merges Overlay deltas into the Parent RMG.
*   **Discard:** A transactional abort (rollback).

**The Physical Analogy:**
*   Root RMG = Spacetime.
*   Process RMG = Local Field.
*   SWS = Observer's Causal Cone.

---

## 3. Execution Model: The Job Fabric

JITOS does not use "job queues." It models work as a causal graph of Intent and Plans.

### 3.1. The HTN Cycle
1.  **TASK (Intent):** A node declaring a goal (e.g., "Fix Auth Bug"). It is declarative, not imperative.
2.  **SLAP (Plan):** A proposed decomposition of a Task into primitive steps. A Task may have multiple competing Slaps.
3.  **WORKER (Mechanism):** An executor (LLM, Script, Human) that runs a primitive step.
    *   Workers **do not** mutate state.
    *   Workers **propose Rewrites** to an SWS.
    *   The Kernel validates and records the Rewrites.

### 3.2. Scheduler Loop
The Scheduler is a generic engine that:
1.  Observes the Graph State.
2.  Selects runnable Primitives (deps satisfied, caps OK).
3.  Emits a `Dispatch` rewrite.
4.  Worker executes and returns a `Result` rewrite.

---

## 4. Operational Semantics: Rewrites & Merging

To support distributed, offline, and speculative work, JITOS implements **Semantic Merging** rather than simple "Last-Write-Wins."

### 4.1. Semantic Operations
Every Rewrite carries a semantic intent, not just a raw value:
*   `Set(Value)`
*   `Increment(n)` / `Decrement(n)`
*   `Push(Item)` / `Remove(Item)`
*   `Connect(Node)` / `Disconnect(Node)`

### 4.2. Merge Strategies
Every field in the RMG declares a Merge Strategy used during **Collapse**:
*   **CRDT:** Merge algebraically (GCounter, OR-Set). Never conflicts.
*   **LWW:** Last-Write-Wins (based on Causal Time).
*   **ThreeWay:** Git-style diff/merge. Can conflict.
*   **Manual:** Always raises a Conflict Task for human resolution.

### 4.3. The Collapse Algorithm
When an SWS collapses into the System:
1.  **Diff:** Compute `Delta = SWS - BaseSnapshot`.
2.  **Rebase:** Check if System has moved since `BaseSnapshot`.
3.  **Resolve:** For every conflict, apply the Field's `MergeStrategy` using the Rewrite's `SemanticOp`.
4.  **Commit:** Append the resolved Rewrites to the System WAL.

---

## 5. The Interface: GraphQL as Universal Control Plane

JITOS uses GraphQL as the typed schema for the entire OS.

*   **Query:** "Show me the Graph (System or SWS)."
*   **Mutation:** Domain Commands (Intent), not raw edits.
    *   `submitIntent`, `createSws`, `collapseSws`, `applyRewrite`
*   **Subscription:** The live pulse of the OS.
    *   `rewrites(view)`: Stream graph deltas for the Viewer.
    *   `ticks`: Stream scheduler heartbeats.

### 5.1. The "Wesley" Pattern
We use Schema-First Design:
1.  The GraphQL SDL defines the Types (Task, Slap, Process).
2.  We codegen the Rust structs, the Database Schemas, and the Validation Logic from the SDL.
3.  This prevents drift between the Kernel Reality and the API Contract.

---

## 6. Implementation Plan (Phased)

1.  **Phase 0 (Kernel Skeleton):** `jitosd` daemon, System WARP, basic `submit_rewrite`.
2.  **Phase 1 (Viewer):** GraphQL Subscriptions + Live Graph Visualization.
3.  **Phase 2 (SWS):** Overlay logic, `create/collapse/discard`.
4.  **Phase 3 (Tasks):** `echo-tasks` crate, Slap structs, HTN planning.
5.  **Phase 4 (Execution):** Scheduler runs Workers -> Rewrites -> SWS.

---

## 7. Summary of Invariants

| Invariant | Description |
| :--- | :--- |
| **History First** | State is a derived view of the Event Log. |
| **Speculation** | All work defaults to SWS overlays. |
| **Policy > Mechanism** | Kernel enforces rules; Workers propose changes. |
| **No "Queues"** | Tasks are nodes in the graph, not rows in a DB. |
| **Observability** | If the Viewer can't explain it, the System is lying. |
