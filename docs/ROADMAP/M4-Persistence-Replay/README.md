# Milestone 4: Persistence & Replay (Beta-2)

**Status:** Planned (Approval-Ready)  
**Target Date:** TBD  
**Owner:** James Ross  
**Primary Artifact:** versioned WAL + restart-safe deterministic replay + replay verification tests  
**Architecture Anchor:** [ARCH-0001](../../ARCH/ARCH-0001-universal-job-fabric.md) (“Monolith with Seams”)

Milestone 4 makes JITOS survive a restart and proves deterministic replay for real: durable append-only WAL, deterministic boot, and replay verification.

---

## 1. Executive Summary

Milestone 4 introduces durability:

- a persistent append-only WAL/event log
- deterministic boot that replays WAL to reconstruct System (and optionally SWS metadata)
- integration tests that prove “run script → restart → outputs identical”

This milestone is **irreversible**: WAL record formats and hashing rules become long-lived compatibility constraints.

---

## 2. User Stories

### US-1: Restart Survival
As an operator, I want `jitosd` to survive a restart and reconstruct System deterministically from durable records.

### US-2: Replay Verification
As a kernel engineer, I want a replay verification test that fails loudly if determinism regresses.

---

## 3. Requirements

### Functional
1. **Data directory:** `jitosd --data-dir <path>` persists kernel state.
2. **WAL append:** all committed events are appended to WAL durably.
3. **Boot replay:** on startup, kernel loads and replays WAL deterministically.
4. **Versioning:** WAL format is versioned.
5. **Pins:** WAL entries include `rulePackId`/`policyId` pins (even if trivial in Beta-2).

### Non-Functional
1. **Determinism:** given same WAL, restart yields identical digests and identical rewrite log outputs.
2. **Integrity:** WAL records are checksummed/hashes are verified on load.

### Constraints / Non-goals (Beta-2)
- No replication yet.
- No advanced GC/compaction beyond basics.

---

## 4. Determinism Invariants (Hard Law)

### LAW-1: WAL defines Chronos order
The WAL order is the authoritative total order for kernel events.

### LAW-2: Replay is sufficient and deterministic
WAL records must contain enough information to deterministically reconstruct the same digests on replay (no hidden dependence on ambient state).

### LAW-3: Format changes are versioned
Any WAL schema evolution must be explicit and versioned; decoding must be deterministic.

---

## 5. Architecture & Design

### 5.1 WAL record format (versioned)

Milestone 4 locks:
- record framing (magic/version/length/checksum)
- canonical encoding for record payload
- record hashing rules

### 5.2 Checkpoints (optional)

Optional compaction: periodic checkpoints that store a snapshot digest + state materialization to reduce replay cost.

---

## 6. API surface

Milestone 4 is mostly internal, but may add:
- `kernelInfo` fields showing WAL version / data dir state
- `replayStatus` (optional) for operator diagnostics

---

## 7. Testing Strategy

### Unit Tests
- WAL encode/decode round-trip determinism.
- WAL hashing/checksum verification.

### Integration Tests (required)
1. Start daemon with `--data-dir tempdir`.
2. Run deterministic mutation script that changes System (post-M3).
3. Stop daemon cleanly.
4. Restart daemon pointing at same data dir.
5. Assert System digest and rewrites log are identical to pre-restart results.

---

## 8. Deliverables
1. Durable WAL and `--data-dir`.
2. Deterministic replay on startup.
3. Versioned WAL format and pins (rulePackId/policyId).
4. Passing replay integration tests.

---

## 9. Definition of Done (Milestone Gate)

Milestone 4 is **DONE** when:

- restarting `jitosd` yields identical digests given the same WAL
- WAL format is versioned and verified
- replay test exists and passes in CI

---

## 10. Task Checklist (Inline)

### Phase 0 — Freeze WAL format (irreversible)
- [ ] Choose WAL record framing + versioning
- [ ] Choose canonical encoding for WAL payload (must be deterministic)
- [ ] Define which events are persisted (minimum: rewrite + collapse + discard)

### Phase 1 — WAL implementation
- [ ] Implement append-only WAL writer (fsync policy explicit)
- [ ] Implement WAL reader + verifier
- [ ] Implement deterministic replay to reconstruct System

### Phase 2 — Daemon wiring
- [ ] Add `--data-dir`
- [ ] Boot path: load + replay + serve

### Phase 3 — Tests
- [ ] Round-trip unit tests for WAL encoding
- [ ] End-to-end restart/replay integration test

---

## 11. Explicit Non-Goals
- replication/federation
- advanced compaction/GC (beyond minimal checkpointing if needed)
