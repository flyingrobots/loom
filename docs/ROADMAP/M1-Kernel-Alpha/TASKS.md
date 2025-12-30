# Milestone 1: Kernel Genesis (Alpha) — Tasks (Aligned)

**Context:** Execute the “Monolith with Seams” strategy from [ARCH-0001](../../ARCH/ARCH-0001-universal-job-fabric.md).  
**Milestone Plan:** [README](README.md)  
**Goal:** A running `jitosd` daemon with a deterministic in-memory kernel (System WARP + SWS overlays) and a minimal GraphQL control plane.

**Primary Specs (must not contradict):**
- [SPEC-0001](../../SPECS/SPEC-0001-canonical-encoding.md) — Canonical encoding
- [SPEC-0005](../../SPECS/SPEC-0005-deterministic-ids.md) — Deterministic IDs
- [SPEC-NET-0001](../../SPECS/SPEC-NET-0001-graphql-sdl-v0.md) — GraphQL SDL v0

---

## Milestone Gate (Definition of Done)

Milestone 1 is **DONE** when all are true (see [README](README.md) §9):

- [ ] `cargo run -p jitosd` starts and serves `/graphql` locally.
- [ ] SWS lifecycle is usable from GraphQL: create, list, discard.
- [ ] A deterministic digest surface exists for System and SWS views (either dedicated `systemHash/swsHash` or `GraphSnapshot.digest`).
- [ ] Rewrite log is queryable (append-only since boot) and ordered deterministically.
- [ ] Determinism proofs pass:
  - [ ] `GraphDigest` stable under insertion order changes (unit test).
  - [ ] kernel mutation order enforced by a single-writer loop (architecture).
  - [ ] replaying the same ordered mutation script yields identical digests and identical rewrite log output.
- [ ] No UUIDs participate in kernel semantic identity (Alpha). (`SwsId` is deterministic `u64`.)

---

## Phase 0 — Lock the Laws (mandatory, fast)

Goal: decide and document the determinism-critical choices *before* implementing networking.

- [ ] **Confirm canonical bytes rule for payloads** (30–60m)
  - [ ] Kernel treats node/edge payload as opaque canonical bytes (no helpful decoding).
  - [ ] If we use a structured encoding (e.g., CBOR), define *exact* canonicalization (spec + tests).
- [ ] **Finalize deterministic identity derivations** (30–60m)
  - [ ] `NodeId = blake3(canonical(kind, payload))` (no self-inclusion).
  - [ ] `EdgeId = blake3(canonical(from, to, kind))`.
  - [ ] Pick and document hash string encoding for API (`hex` vs `base32`) and do not change it later.
- [ ] **Finalize graph digest law** (30–60m)
  - [ ] `GraphDigest` folds nodes by sorted `NodeId` bytes and edges by sorted `EdgeId` bytes.
  - [ ] Explicitly outlaw `HashMap` iteration affecting digests (sort or use `BTree*`).
- [ ] **Finalize kernel ordering law** (30–60m)
  - [ ] All mutations go through a single-writer command loop (no concurrent writers).
  - [ ] `SwsId: u64` allocated deterministically (`0,1,2,...`).
  - [ ] `RewriteEvent.idx: u64` increments once per applied mutation.
- [ ] **Spec alignment note / addendum** (30–60m)
  - [ ] Either add a short “graph digests” addendum to existing specs (preferred), or introduce a new spec doc for deterministic digests and link it from the milestone README.

---

## Phase 1 — Warp Core (`jitos-warp-core`): make hashes real first

Goal: deterministic IDs + deterministic digests with behavioral tests.

- [ ] **Implement content-addressed Node/Edge IDs** (60–120m)
  - [ ] `Node { kind, payload }` (payload = canonical bytes).
  - [ ] `Edge { from, to, kind }` (minimal edge model for M1).
  - [ ] `NodeId` / `EdgeId` derivation functions.
- [ ] **Implement in-memory Graph + deterministic digest** (60–180m)
  - [ ] Store nodes/edges (internal collections may be `HashMap`).
  - [ ] `digest()` must fold in sorted ID order (stable across insertion order).
  - [ ] Provide sorted snapshot iterators for API output (`nodes_sorted()`, `edges_sorted()`).
- [ ] **Warp-core unit tests (behavioral)** (60–120m)
  - [ ] stable `NodeId` across runs given identical `(kind, payload)`.
  - [ ] stable `GraphDigest` under permuted insertion order for same set of nodes/edges.

---

## Phase 2 — Kernel (`jitos-kernel`): single-writer + SWS overlays + rewrite log

Goal: the kernel owns all semantics; ordering and logging are deterministic by construction.

- [ ] **Single-writer kernel loop** (60–120m)
  - [ ] Define command surface (`KernelCmd`) for queries + mutations.
  - [ ] Implement a single task that owns all mutable kernel state and processes commands sequentially.
- [ ] **SWS lifecycle with deterministic IDs** (60–120m)
  - [ ] create/list/discard with `SwsId: u64` allocator.
  - [ ] capture `base_digest` at SWS creation.
- [ ] **SWS overlay semantics (read-through, overlay-only writes)** (120–180m)
  - [ ] reads check overlay first, then system.
  - [ ] writes go to overlay only (Alpha).
- [ ] **Rewrite application + append-only rewrite log** (120–180m)
  - [ ] implement `AddNode` into SWS overlay (minimum).
  - [ ] append `RewriteEvent { idx, view/target, op, receipt/meta }` deterministically.
  - [ ] expose a query to retrieve rewrites by `(view, after, limit)` or equivalent cursor.
- [ ] **Kernel unit tests (behavioral)** (60–120m)
  - [ ] deterministic `SwsId` allocation.
  - [ ] SWS isolation: overlay changes not visible in System view.
  - [ ] rewrite log deterministic ordering and monotone indexing.

---

## Phase 3 — Net/API (`jitos-net`): thin GraphQL adapter

Goal: GraphQL is a control plane; kernel is the semantics engine.

- [ ] **Scaffold Axum + async-graphql** (30–90m)
  - [ ] `/graphql` endpoint and GraphiQL.
  - [ ] optional `/ws` for subscriptions (nice-to-have; not required for M1 gate).
- [ ] **Implement the Milestone-1 subset of SPEC-NET-0001** (120–180m)
  - [ ] SWS lifecycle queries/mutations (`createSws`, `listSws`, `discardSws`).
  - [ ] `applyRewrite` targeting a view (SWS minimum).
  - [ ] rewrite log query (`rewrites(...)`) for operator-grade audit since boot.
  - [ ] digest surface exists (either `systemHash/swsHash` or snapshot digest field).
- [ ] **Determinism hygiene in API output** (60–120m)
  - [ ] graph snapshots return nodes/edges in deterministic ordering (sorted by ID bytes).
  - [ ] timestamps are `null` for Milestone 1 (avoid wall-clock nondeterminism).
  - [ ] avoid “two sources of truth” routing (prefer `ViewRefInput` as the only routing input).

---

## Phase 4 — `jitosd` + integration tests + milestone report

Goal: prove the system boots, is inspectable, and replay-deterministic end-to-end.

- [ ] **`bins/jitosd` daemon** (60–120m)
  - [ ] `--host`, `--port` flags (default `127.0.0.1:8080`).
  - [ ] spawn kernel loop; mount GraphQL router.
- [ ] **End-to-end determinism integration test** (120–180m)
  - [ ] start daemon in test harness.
  - [ ] run scripted mutation sequence:
    - [ ] capture System digest `H0`.
    - [ ] create SWS (expect deterministic `swsId = 0`).
    - [ ] apply `AddNode` into SWS 0.
    - [ ] assert SWS digest differs, System digest unchanged.
    - [ ] query rewrite log and assert deterministic event ordering.
  - [ ] repeat script in a fresh process; assert identical outputs (digests + rewrite log).
- [ ] **Milestone report PDF** (30–90m)
  - [ ] build with `make milestone-1` (produces `MILESTONE-1.pdf`).

---

## Explicit Non-Goals (to prevent scope creep)

These are out of scope for Milestone 1 unless explicitly pulled into the gate:

- persistence / WAL / database-backed stores
- collapse semantics / merge into System truth
- worker execution / schedulers / tasks beyond kernel-internal operations
- wall-clock timestamps (return `null`)
