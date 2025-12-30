# Milestone 3: Collapse & Commit (Beta-1)

**Status:** Planned (Approval-Ready)  
**Target Date:** TBD  
**Owner:** James Ross  
**Primary Artifact:** deterministic collapse semantics (SWS → System) + tick/commit boundary + conflict policy  
**Architecture Anchor:** [ARCH-0001](../../ARCH/ARCH-0001-universal-job-fabric.md) (“Monolith with Seams”)

Milestone 3 makes SWS real: collapsing a shadow world into System deterministically, with explicit conflict handling and first-class commit events.

---

## 1. Executive Summary

Milestone 1–2 established deterministic overlays and viewer-stable snapshots. Milestone 3 introduces the first irreversible kernel mutation: **promoting SWS overlay changes into System** through a deterministic `collapseSws` operation.

This is the first milestone where System becomes mutable, but only via:
- the single-writer kernel loop, and
- a deterministic collapse/commit procedure.

**Goal:** create SWS, apply rewrites in overlay, and deterministically collapse into System producing a new System digest/snapshot plus an auditable receipt.

---

## 2. User Stories

### US-1: Deterministic Commit
As a developer, I want to collapse an SWS into System deterministically so that “commit” has a stable meaning and replay/audit remain possible.

### US-2: Conflict Clarity
As an operator, I want a deterministic conflict policy so that collapse outcomes aren’t dependent on incidental ordering or hidden merge rules.

---

## 3. Requirements

### Functional
1. **Collapse:** implement `collapseSws(input)` to merge SWS overlay changes into System.
2. **System mutation:** System graph becomes mutable only via collapse (and still only in the single-writer loop).
3. **Receipts:** collapse returns deterministic outcome fields:
   - `committed: Boolean!`
   - `systemSnapshotId` (or System digest)
   - deterministic receipt/events
4. **Events/log:** rewrite log includes `COLLAPSE` and `DISCARD` events.
5. **Conflict policy:** deterministic, explicit policy (Milestone 3 default: fail-fast).

### Non-Functional
1. **Determinism:** same System state + same SWS overlay + same policy → identical collapse outcome.
2. **Audit:** collapse produces a receipt sufficient to explain outcome (at least what changed / why it failed).

### Constraints / Non-goals (Beta-1)
- No durable storage yet.
- No external workers yet.

---

## 4. Determinism Invariants (Hard Law)

### LAW-1: Collapse is the only path to System mutation
No API surfaces permit direct System writes except through deterministic collapse.

### LAW-2: Conflict policy is explicit and deterministic
Milestone 3 default policy: **fail-fast** on conflict. A conflict is defined deterministically (e.g., competing writes to same identity / incompatible overlays).

### LAW-3: Collapse emits a commit boundary
Collapse produces a stable commit record in the rewrite/event log so the viewer and replay tooling can observe the worldline boundary.

---

## 5. Architecture & Design

### 5.1 What is a “conflict” in M3?

Define the minimal conflict set for the first version:

- If overlay introduces a node/edge whose ID collides with System but content differs → conflict.
- If overlay references missing prerequisites (e.g., edge references absent node) → conflict.

### 5.2 System digest changes

After collapse, System digest must change deterministically, and the snapshot identity semantics from Milestone 2 must remain stable.

---

## 6. API (GraphQL v0.2)

Milestone 3 makes `collapseSws` real and makes System mutable.

Required:
- `collapseSws(input: CollapseSwsInput!): CollapseSwsPayload!` implemented
- `rewrites(...)` includes `COLLAPSE`/`DISCARD` events

Deferred:
- policy plug-ins beyond selecting fail-fast policyId (optional)

---

## 7. Testing Strategy

### Unit Tests
- Collapse success: overlay changes promoted into System; System digest changes.
- Collapse isolation: post-collapse, SWS may be marked collapsed/discarded deterministically.
- Conflict fail-fast: a deterministic conflicting case returns `committed=false` with stable error code and stable receipt.

### Integration Tests (HTTP)
1. Create SWS, apply AddNode, collapse; assert System digest changes.
2. Replay test: same script produces same System digest and same collapse receipt.
3. Conflict script: create two inconsistent states and verify fail-fast determinism.

---

## 8. Deliverables
1. Deterministic collapse/commit semantics (fail-fast conflict policy).
2. System mutability via collapse only.
3. Collapse receipts/events in the rewrite log.
4. Passing tests and replay proof.

---

## 9. Definition of Done (Milestone Gate)

Milestone 3 is **DONE** when:

- `collapseSws` is implemented and deterministic.
- Conflicts are detected deterministically, and default policy is fail-fast.
- System digest/snapshot updates deterministically after successful collapse.
- Rewrite log contains collapse/discard events.
- End-to-end replay tests pass across fresh restarts (still in-memory).

---

## 10. Task Checklist (Inline)

### Phase 0 — Freeze conflict semantics
- [ ] Define conflict conditions for M3 (minimal, deterministic)
- [ ] Decide how collapse receipt encodes outcome (fields + error codes)
- [ ] Update SPEC-NET-0001 “Milestone 3 subset” section (addendum)

### Phase 1 — Kernel collapse implementation
- [ ] Implement deterministic collapse procedure (overlay → System)
- [ ] Enforce “System mutation only via collapse”
- [ ] Emit `COLLAPSE`/`DISCARD` events into rewrite log

### Phase 2 — API wiring
- [ ] Implement GraphQL `collapseSws`
- [ ] Ensure error codes match spec (`NOT_IMPLEMENTED` vs `INVALID_INPUT` vs `NOT_FOUND`)

### Phase 3 — Tests
- [ ] Unit tests for success + conflict fail-fast
- [ ] HTTP integration tests + replay proof

---

## 11. Explicit Non-Goals
- persistence/WAL (next milestone)
- worker execution
