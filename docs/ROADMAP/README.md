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

### M2 — Kernel Reality Layer (Beta-0)
- **Doc:** `docs/ROADMAP/M2-Kernel-Reality-Layer/README.md`
- **Focus:** viewer-stable snapshots, stable `snapshotId`, real `first+after` cursoring, rewrite allowlist/registry
- **Key outcomes:** paging that doesn’t lie; snapshot identity semantics locked

### M3 — Collapse & Commit (Beta-1)
- **Doc:** `docs/ROADMAP/M3-Collapse-Commit/README.md`
- **Focus:** deterministic collapse semantics (SWS → System) and explicit conflict policy
- **Key outcomes:** System mutability only via deterministic collapse; commit boundary becomes first-class

### M4 — Persistence & Replay (Beta-2)
- **Doc:** `docs/ROADMAP/M4-Persistence-Replay/README.md`
- **Focus:** durable append-only WAL + deterministic boot/replay + restart verification
- **Key outcomes:** restart-safe kernel; WAL schema/versioning becomes irreversible contract

### M5 — Time & Scheduling (Beta-3)
- **Doc:** `docs/ROADMAP/M5-Time-Scheduling/README.md`
- **Focus:** deterministic time (“Clock View”), tick loop, timer primitive, policy pins in receipts/events
- **Key outcomes:** `now()` derived from history; ticks are observable and replay-stable

### M6 — TASKS / SLAPS / Workers (Beta-4)
- **Doc:** `docs/ROADMAP/M6-Tasks-Slaps-Workers/README.md`
- **Focus:** make work real: `submitIntent`, deterministic task lifecycle, worker invocation/results with receipts
- **Key outcomes:** job fabric becomes operational; worker boundary and capability checks become real

### M7 — Typed Domain API v1 + Wesley Mode (1.0 runway)
- **Doc:** `docs/ROADMAP/M7-Typed-Domain-API-v1/README.md`
- **Focus:** stop hiding behind JSON; typed schema + stable kind registries + generator skeleton
- **Key outcomes:** schema/validators/registries converge; deprecation path for JSON ops exists

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

