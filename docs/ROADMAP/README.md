# JITOS Milestone Roadmap (MOC)

This directory contains the approval-ready milestone plans for JITOS/JITOSD. Each milestone README is intended to be executable: it defines invariants, scope, a Definition of Done, and an inline task checklist.

**How to use this roadmap**
- Treat each milestone doc as the canonical “what we are building” contract for that phase.
- When starting a milestone, freeze its remaining “contract choices” (hash/cursor formats, record schemas, etc.) *before* implementation.
- Prefer shipping irreversibles in a controlled order: snapshot semantics → collapse semantics → WAL format → time model → workers/task model → typed APIs.

---

## Milestones

### M1 — Kernel Genesis (Alpha)
- **Doc:** `docs/ROADMAP/M1-Kernel-Alpha/README.md`
- **Focus:** deterministic in-memory kernel + SWS overlays (overlay-only writes) + GraphQL control plane
- **Key outcomes:** single-writer mutation ordering, deterministic digests, strict op validation, operator-grade rewrite log
- **What you can do:**
  - run `jitosd` locally and query `graph(view) { digest }`
  - create/list/discard SWS overlays
  - apply exactly one rewrite op (`AddNode`) into an SWS overlay and receive a deterministic receipt (`rewriteIdx`, `viewDigest`)
  - query an append-only rewrite log and verify deterministic ordering
  - repeat the same mutation script in a fresh process and get identical digests/log outputs (in-memory; no persistence)

### M2 — Kernel Reality Layer (Beta-0)
- **Doc:** `docs/ROADMAP/M2-Kernel-Reality-Layer/README.md`
- **Focus:** viewer-stable snapshots, stable `snapshotId`, real `first+after` cursoring, rewrite allowlist/registry
- **Key outcomes:** paging that doesn’t lie; snapshot identity semantics locked
- **What you can do:**
  - treat `graph(view)` as a stable snapshot: get `snapshotId` and page nodes/edges deterministically with `first+after`
  - page `rewrites(view, page)` deterministically using `after` by `idx`
  - rely on a kernel-side rewrite registry (allowlist) so clients can’t drift the op format silently
  - build a viewer that can cache, diff, and paginate without “same state, different response” bugs

### M3 — Collapse & Commit (Beta-1)
- **Doc:** `docs/ROADMAP/M3-Collapse-Commit/README.md`
- **Focus:** deterministic collapse semantics (SWS → System) and explicit conflict policy
- **Key outcomes:** System mutability only via deterministic collapse; commit boundary becomes first-class
- **What you can do:**
  - collapse an SWS into System deterministically (`collapseSws`) and observe a new System digest/snapshot
  - get deterministic conflict handling (initially fail-fast) instead of implicit/accidental merge behavior
  - observe collapse/discard events in the rewrite log (commit boundary becomes visible)
  - treat System as mutable *only* through deterministic commit semantics

### M4 — Persistence & Replay (Beta-2)
- **Doc:** `docs/ROADMAP/M4-Persistence-Replay/README.md`
- **Focus:** durable append-only WAL + deterministic boot/replay + restart verification
- **Key outcomes:** restart-safe kernel; WAL schema/versioning becomes irreversible contract
- **What you can do:**
  - run `jitosd --data-dir …`, perform mutations/commits, stop and restart
  - verify restart/replay reconstructs identical System digests from the same WAL
  - use the WAL as the authoritative Chronos order (auditable, replayable history)
  - run an integration test that proves “script → restart → digests identical”

### M5 — Time & Scheduling (Beta-3)
- **Doc:** `docs/ROADMAP/M5-Time-Scheduling/README.md`
- **Focus:** deterministic time (“Clock View”), tick loop, timer primitive, policy pins in receipts/events
- **Key outcomes:** `now()` derived from history; ticks are observable and replay-stable
- **What you can do:**
  - query deterministic “now” (derived from history, not wall clock)
  - subscribe to global tick events and observe replay-stable scheduling behavior
  - run a minimal timer demo that fires deterministically and records its firing as events
  - see `policyId`/`rulePackId` pins show up consistently in receipts/events (policy boundary becomes real)

### M6 — TASKS / SLAPS / Workers (Beta-4)
- **Doc:** `docs/ROADMAP/M6-Tasks-Slaps-Workers/README.md`
- **Focus:** make work real: `submitIntent`, deterministic task lifecycle, worker invocation/results with receipts
- **Key outcomes:** job fabric becomes operational; worker boundary and capability checks become real
- **What you can do:**
  - call `submitIntent` to create a task/process/SWS context and get IDs back
  - observe deterministic task state transitions (queued → running → done/failed) via `taskEvents`
  - invoke an in-process demo worker and get auditable invocation/result events with receipts
  - start enforcing a capability boundary for worker invocation (even if localhost-only initially)

### M7 — Typed Domain API v1 + Wesley Mode (1.0 runway)
- **Doc:** `docs/ROADMAP/M7-Typed-Domain-API-v1/README.md`
- **Focus:** stop hiding behind JSON; typed schema + stable kind registries + generator skeleton
- **Key outcomes:** schema/validators/registries converge; deprecation path for JSON ops exists
- **What you can do:**
  - interact with typed Task/Slap/Primitive/Policy objects via a stable GraphQL v1 schema
  - rely on a canonical NodeKind/EdgeKind registry (versioned) instead of free-form strings everywhere
  - use a generator skeleton (“Wesley mode”) to produce Rust enums/validators from schema/registries
  - migrate away from JSON ops via an explicit deprecation path (v0 remains supported but discouraged)

---

## Continuous tracks (every milestone)

### Spec hardening
Every milestone should end with:
- what became irreversible
- spec updates/versioning for affected contracts
- golden-vector tests for hashing/digest/WAL serialization

### Tooling
A small CLI pays dividends early:
- `jitosctl graph --view system --digest`
- `jitosctl rewrites --view sws:0`
- `jitosctl replay --verify`
