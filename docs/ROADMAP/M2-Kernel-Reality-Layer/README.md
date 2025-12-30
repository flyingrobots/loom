# Milestone 2: Kernel Reality Layer (Beta-0)

**Status:** Planned (Approval-Ready)  
**Target Date:** TBD  
**Owner:** James Ross  
**Primary Artifact:** stable view semantics (snapshots + paging) + validated rewrite model + operator-grade log queries  
**Architecture Anchor:** [ARCH-0001](../../ARCH/ARCH-0001-universal-job-fabric.md) (“Monolith with Seams”)

Milestone 2 turns the Milestone 1 daemon into a *useful* kernel: stable snapshots, deterministic cursoring, and a rewrite/event model the viewer can depend on, without introducing collapse semantics or persistence yet.

---

## 1. Executive Summary

Milestone 2 upgrades “toy-but-deterministic” into “viewer-stable and audit-stable”:

- `graph(view)` returns a stable snapshot with a stable `snapshotId`
- deterministic pagination for graph snapshots and rewrite logs
- rewrite validation is no longer “whatever JSON”; ops are checked against a kernel allowlist/registry

**Goal:** A viewer can render `graph(view)` repeatedly without drift, page deterministically, and consume `rewrites(view, page)` reliably, all while preserving determinism invariants from Milestone 1.

---

## 2. User Stories

### US-1: Stable Snapshot
As a viewer developer, I want `graph(view)` to return a stable `snapshotId` so I can cache, diff, and page without races or “same state, different response” bugs.

### US-2: Deterministic Paging
As an operator, I want deterministic `first+after` paging for nodes/edges and rewrites so I can page large views without missing/duplicating items.

### US-3: Validated Rewrite Surface
As a kernel engineer, I want `applyRewrite` to validate ops against a registry so clients can’t smuggle in future formats and cause audit/spec drift.

---

## 3. Requirements

### Functional
1. **Snapshots:** `graph(view)` returns a `GraphSnapshot` including `snapshotId` and `digest`.
2. **Snapshot identity:** `snapshotId` has stable semantics (see “Frozen Contract Choices”).
3. **Paging:** Graph snapshot nodes/edges support deterministic `first+after` cursoring.
4. **Rewrites query:** `rewrites(view, page)` supports `after` cursor properly, ordered by `idx` ascending.
5. **Rewrite validation:** `applyRewrite` validates ops against a kernel allowlist/registry (even if the registry is tiny).
6. **Ordering:** Snapshot output remains deterministic (nodes/edges sorted by id ascending).

### Non-Functional
1. **Determinism:** Replaying the same ordered mutation sequence yields identical digests and identical rewrite log contents.
2. **Compatibility:** No breaking changes to Milestone 1 frozen encodings without explicit spec versioning.

### Constraints / Non-goals (Beta-0)
- No collapse/merge yet.
- No persistence/WAL yet.
- No worker execution yet.

---

## 4. Determinism Invariants (Hard Law)

Milestone 2 extends Milestone 1 laws and adds snapshot/cursor determinism.

### LAW-1: Single-writer kernel ordering remains authoritative
All mutations flow through a single-writer command loop. No concurrent writers.

### LAW-2: Stable ordering for all list surfaces
Any list surface that feeds a canonical digest or a cursor must have deterministic ordering:

- `graph(view).nodes` sorted by `GraphNode.id` ascending (lexicographic bytes)
- `graph(view).edges` sorted by `GraphEdge.id` ascending
- `rewrites(view)` sorted by `RewriteEvent.idx` ascending

### LAW-3: Cursor meaning is stable and derived from ordering keys
`after` cursors must be derived from the deterministic ordering key:

- nodes/edges: cursor is the last returned `id`
- rewrites: cursor is the last returned `idx`

---

## 5. Architecture & Design

### 5.1 Snapshot IDs

Milestone 2 introduces stable snapshot identity. See contract choice below (digest-as-snapshotId is recommended).

### 5.2 Rewrite model upgrade

Milestone 1 accepted only `AddNode` with strict JSON. Milestone 2 keeps JSON ops but introduces a **registry/allowlist**:

- allowed ops are explicitly listed (even if only `AddNode` exists)
- op JSON must match the schema for that op (deny unknown fields)
- unknown op → `NOT_IMPLEMENTED`

---

## 6. API (GraphQL v0 → v0.1)

Milestone 2 ships additional *viewer-stability* guarantees on top of SPEC-NET-0001.

### Required in Milestone 2
- `graph(view)` includes:
  - `digest: Hash!`
  - `snapshotId: ID!` (stable semantics)
  - `nodesPage` / `edgesPage` support deterministic `first+after`
- `rewrites(view, page)` supports `after` deterministically by `idx`

### Deferred (not required in Milestone 2)
- GraphDelta subscription (optional; only if cheap)
- persistence-related query surfaces

---

## 7. Testing Strategy

### Unit Tests
- SnapshotId stability: same `GraphSnapshot.digest` → same `snapshotId` (if using digest-as-id).
- Paging determinism: permuting insertion order does not change snapshot paging order.
- Rewrite registry: unknown op → `NOT_IMPLEMENTED`; extra fields → `INVALID_INPUT`.

### Integration Tests (HTTP, end-to-end)
1. Start `jitosd` on ephemeral port.
2. Create SWS, apply AddNode, capture `graph(view).snapshotId` + `digest`.
3. Page nodes with `first=1` and `after=<id>`; assert:
   - no duplication
   - deterministic order
4. Query `rewrites(view, page)` with `after=<idx>`; assert correct continuation.

---

## 8. Deliverables
1. Stable snapshots + snapshot IDs.
2. Deterministic paging for graph snapshots and rewrites.
3. Rewrite validation registry (allowlist) hooked into `applyRewrite`.
4. Passing tests (`cargo test` + end-to-end integration tests).

---

## 9. Definition of Done (Milestone Gate)

Milestone 2 is **DONE** when all are true:

- `graph(view)` returns `GraphSnapshot { digest, snapshotId }` with stable semantics.
- `graph(view)` supports deterministic `first+after` paging for nodes/edges on sorted IDs.
- `rewrites(view, page)` supports `after` cursor properly (by idx ascending).
- `applyRewrite` validates ops against an explicit allowlist/registry; unknown op → `NOT_IMPLEMENTED`.
- Integration tests prove paging correctness and stable snapshot identity.

---

## 10. Task Checklist (Inline)

Primary docs (must not contradict):
- [ARCH-0001](../../ARCH/ARCH-0001-universal-job-fabric.md)
- [SPEC-0001](../../SPECS/SPEC-0001-canonical-encoding.md)
- [SPEC-0005](../../SPECS/SPEC-0005-deterministic-ids.md)
- [SPEC-NET-0001](../../SPECS/SPEC-NET-0001-graphql-sdl-v0.md)

### Phase 0 — Freeze snapshot + cursor semantics (mandatory)
- [ ] Choose `snapshotId` meaning (see frozen contract choices)
- [ ] Specify cursor formats (IDs/idx as strings, derived from deterministic ordering)
- [ ] Update SPEC-NET-0001 “Milestone 2 subset” section (addendum)

### Phase 1 — Kernel snapshot identity
- [ ] Implement snapshotId generation and exposure in kernel snapshot responses
- [ ] Ensure snapshotId is stable under repeated queries with identical digest

### Phase 2 — Deterministic paging
- [ ] Implement `first+after` paging on sorted node/edge IDs
- [ ] Implement `after` paging for rewrites by idx

### Phase 3 — Rewrite registry
- [ ] Create a small allowlist registry for supported ops
- [ ] Validate ops strictly (deny unknown fields)
- [ ] Return error codes per SPEC-NET-0001 (INVALID_INPUT vs NOT_IMPLEMENTED)

### Phase 4 — Tests
- [ ] Unit tests for snapshotId/paging/registry rules
- [ ] HTTP integration test for paging and rewrite continuation

---

## 11. Frozen Contract Choices (Milestone 2)

These choices are frozen in Milestone 2:

- **Hash encoding:** lowercase hex, 64 chars, 32 bytes (same as M1).
- **Snapshot identity (pick one):**
  - Recommended: `snapshotId == digest` (string form of `GraphSnapshot.digest`).
  - Alternate: minted `snapshotId` (requires stable minting rules; more complexity).
- **Cursors:**
  - nodes/edges: `after` is the last returned `id`
  - rewrites: `after` is the last returned `idx`

---

## 12. Explicit Non-Goals
- collapse/commit
- persistence/WAL
- worker execution
