# ARCH-0002: Common Architectural Invariants

**Status:** Draft (but treat as law)
**Date:** 2025-12-29
**Purpose:** The “constitution.” If you violate these, you’re not extending JITOS — you’re building a different system.

---

## A. History is the system

1.  **History-first truth**
    The authoritative record is events/rewrites, not mutable state.
2.  **State is a view**
    “Current state” is always derived from history under a policy/observer.
3.  **No silent mutation**
    If a change matters, it must appear as a rewrite/event with provenance.

## B. Determinism is non-negotiable

4.  **Replay equality is a feature requirement**
    Same inputs + same policies ⇒ same history ⇒ same derived views.
5.  **No ambient nondeterminism**
    Wall clock, RNG, network timing, thread scheduling are not allowed to leak into canonical history without explicit modeling.
6.  **Policy is explicit**
    Any interpretation choice (clock, scheduler, trust, merge) is represented as a policy identifier / rule-pack pin.

## C. Speculation is default

7.  **Risky work happens in overlays**
    Agent-driven, user-driven, or uncertain operations run in SWS by default.
8.  **Collapse is transactional**
    Changes become “real” only via an explicit collapse/commit step.
9.  **Abort is still information**
    Failed attempts are recorded as history (even if summarized/compacted later). No “nothing happened.”

## D. Intent, plan, execution are separate

10. **Intent ≠ Plan ≠ Execution**
    *   **TASK:** what we want
    *   **SLAP:** how we might do it
    *   **Worker steps:** what we actually did
11. **Plans are branchable artifacts**
    Multiple SLAPs may exist; selection is policy-driven, observable, and reversible.

## E. Policy over mechanism

12. **Kernel enforces policy; workers execute mechanism**
    Workers do not define truth. They propose results. The kernel records and validates.
13. **Capabilities are explicit**
    Authority is granted, not implied. No ambient “because it’s in-process.”

## F. Observability is first-class

14. **Everything important is queryable**
    If it affects outcomes, it must be inspectable: why it ran, why it didn’t, what blocked it.
15. **Viewer isn’t a toy**
    The viewer is a primary interface to truth. If the viewer can’t explain it, the system is lying.

## G. Composability with hard seams

16. **“Monolith with seams” is the default strategy**
    Start one daemon for speed. Keep boundaries so components can split later without rewriting the universe.
17. **Stable contracts at boundaries**
    *   kernel API: commands + views
    *   worker API: invocation + receipts
    *   planner API: intent → DAG

## H. Security posture: assume hostile environments (eventually)

18. **API calls are bounded**
    Query depth/cost, rate limits, and safety defaults exist before “public exposure.”
19. **Auditability is not optional**
    External-facing actions must be attributable and recorded.

## I. Anti-invariants (things we refuse)

20. **No kanban-in-the-kernel**
    Status fields are derived views, not primary truth.
21. **No generic “set field” API**
    Commands only. Domain verbs only.
22. **No “just add a job queue”**
    Queues hide causality and erase near-misses. JITOS keeps the causal fabric.

---

## Optional add-on: “Invariant tests” as CI gates

For each invariant we care about, we eventually add:
*   property tests (determinism/replay)
*   golden history vectors
*   schema validation (node/edge kinds)
*   “no nondeterminism” linting (clock/RNG/network calls)
