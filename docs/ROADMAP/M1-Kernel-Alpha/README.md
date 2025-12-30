# Milestone 1: Kernel Genesis (Alpha)

**Status:** Planned (Approval-Ready)  
**Target Date:** TBD  
**Owner:** James Ross  
**Primary Artifact:** `jitosd` daemon + deterministic in-memory kernel + GraphQL control plane  
**Architecture Anchor:** [ARCH-0001](../../ARCH/ARCH-0001-universal-job-fabric.md) (“Monolith with Seams”)

This milestone is the minimum bootable foundation: a deterministic kernel daemon that hosts a System WARP graph, supports SWS overlays, and exposes an operator-grade control plane to query state and apply deterministic mutations.

---

## 1. Executive Summary

Milestone 1 delivers a bootable kernel daemon (`jitosd`) that hosts an in-memory **System WARP** graph and exposes a GraphQL endpoint for:

- querying graph state and deterministic digests, and
- performing deterministic mutations through a single-writer kernel loop.

It introduces **SWS (Shadow Working Set; a.k.a. “Schrödinger Workspace”)** as isolated overlays with **read-through semantics** (overlay reads first, then system) and **overlay-only writes** (for Alpha).

**Goal:** Run `jitosd`, create SWS overlays via GraphQL, apply deterministic rewrites, query graph hashes, and inspect an append-only rewrite log.

---

## 2. User Stories

### US-1: The Living Graph
As a developer, I want to start `jitosd` and query the system graph so I can verify the kernel is running and holding state.

### US-2: Speculative Workspace
As a tool author, I want to create an SWS via API so I can have an isolated scratchpad for changes.

### US-3: The Immutable Log
As an operator, I want to query a deterministic, append-only rewrite log so I can audit what `jitosd` has done since boot.

### US-4: Deterministic Identity
As a kernel engineer, I want identical nodes created in different SWSs to have the exact same content-addressed `NodeId`.

---

## 3. Requirements

### Functional
1. **Daemon:** `jitosd` compiles and runs.
2. **API:** GraphQL endpoint at `http://127.0.0.1:8080/graphql`.
3. **Graph:** In-memory graph supporting Nodes + Edges.
4. **Identity:** Deterministic content addressing (BLAKE3) for nodes and edges.
5. **SWS:** Ability to create, list, and discard SWS contexts.
6. **Rewrites:** Apply at least `AddNode` into an SWS overlay.
7. **Hashes:** Query deterministic graph digests for System and SWS views (see API section).
8. **Rewrite Log:** Query an append-only rewrite log since boot (see API section).

### Non-Functional
1. **Rust:** 100% safe Rust where possible (`unsafe` requires explicit justification).
2. **Async Runtime:** Tokio.
3. **Performance (best effort):** < 1ms per op at toy sizes (< 10k nodes).
4. **Determinism:** Replaying the same *ordered* mutation sequence yields identical graph digests and identical rewrite log contents.

### Constraints (Alpha)
- No database / persistence / WAL.
- State lost on restart.
- No external workers; only kernel-internal operations.

---

## 4. Determinism Invariants (Hard Law)

These are not “nice-to-have.” They are the Milestone 1 contract.

### LAW-1: Kernel mutation order is total and deterministic
All mutations flow through a single-writer command loop (queue). No concurrent writes. This prevents reordering due to Tokio scheduling.

### LAW-2: SWS identity is deterministic
No UUIDs participate in kernel semantics for Alpha.  
`SwsId = u64` assigned by the kernel in deterministic sequence order (`0, 1, 2, ...`).

### LAW-3: `HashMap` iteration never affects canonical output
`HashMap` is allowed for storage/lookups, but any canonical digest must fold data in a **sorted order** (e.g., sort by `NodeId` / `EdgeId` bytes). Alternatively, use `BTreeMap` / `BTreeSet` for digest-participating collections.

### LAW-4: `NodeId` does not include `NodeId`
Node IDs are derived from canonical content, not self-referential fields.

---

## 5. Architecture & Design

### 5.1. Crate Hierarchy

This milestone uses the ARCH-0001 “Monolith with Seams” stance: start monolithic enough to ship, but keep boundaries crisp so we can split later.

```
crates/
├── jitos-core       # (Existing) hash types + canonical helpers (or re-export)
├── jitos-warp-core  # (New) WARP graph engine: Node/Edge/Graph + deterministic digests
├── jitos-kernel     # (New) kernel: system graph + SWS overlays + rewrite log + cmd loop
└── jitos-net        # (New) Axum + async-graphql adapter (thin control plane)
bins/
└── jitosd           # (New) daemon entry point
```

### 5.2. Core Data Model

#### Node (content-addressed)

```rust
struct Node {
    kind: String,
    payload: Vec<u8>, // canonical bytes; kernel does not reinterpret
}

type NodeId = Hash; // blake3(canonical(kind, payload))
```

Notes:
- “Canonical bytes” means the kernel treats the payload as an opaque byte vector that is already canonical.
- **Milestone 1 payload rule:** the payload is provided to the kernel as bytes (API: base64). The kernel hashes the bytes **as-is** (no decoding, normalization, or “helpful” interpretation).
- Avoid “helpful” decoding/normalization inside the kernel for v0; that is a determinism trap.
- If we later introduce structured encodings (e.g., canonical CBOR), that must be explicitly specified via [SPEC-0001](../../SPECS/SPEC-0001-canonical-encoding.md) and validated by tests (future milestone).

#### Edge (content-addressed)

```rust
struct Edge {
    from: NodeId,
    to: NodeId,
    kind: String,
}

type EdgeId = Hash; // blake3(canonical(from, to, kind))
```

#### Graph (in-memory + deterministic digest)

```rust
struct Graph {
    nodes: HashMap<NodeId, Node>,
    edges: HashMap<EdgeId, Edge>,
}

type GraphDigest = Hash; // blake3(canonical(fold(nodes_sorted), fold(edges_sorted)))
```

Digest law:
- fold nodes in sorted `NodeId` order
- fold edges in sorted `EdgeId` order

### 5.3. SWS Overlay Semantics

SWS is a read-through overlay on top of the system graph, with a base digest captured at creation time.

```rust
type SwsId = u64;

struct Sws {
    id: SwsId,
    base_digest: GraphDigest,                 // captured at creation
    overlay_nodes: HashMap<NodeId, Node>,     // overlay-only writes (Alpha)
    overlay_edges: HashMap<EdgeId, Edge>,
}
```

Read path:
- check overlay first, then system graph

Write path (Milestone 1):
- writes only to overlay

**Alpha semantic note (Milestone 1):** the System graph is immutable (no System writes). Therefore an SWS base behaves as “System at creation time.” `base_digest` is captured for audit and future collapse semantics, and reads are defined as overlay-first, then System-as-of-creation.

### 5.4. Kernel Mutation Model (Single Writer)

Kernel runs as a task that owns mutable state and processes commands sequentially:

- `CreateSws`
- `DiscardSws`
- `ApplyRewrite`
- `Query*` (Alpha: simplest is to serve reads through the same loop for strict ordering)

This guarantees stable ordering, stable SWS IDs, and stable rewrite indices.

### 5.5. Rewrite Log (Append-only, Deterministic)

```rust
struct RewriteEvent {
    idx: u64,            // deterministic sequence number
    view: ViewRef,       // System | Sws(SwsId)
    ops: Vec<JSON>,      // validated inbound ops (audit truth in M1)
    meta: JSON,          // optional metadata (deterministic)
    receipt: Receipt,    // optional, deterministic
}
```

`idx` increments once per applied mutation (through the single-writer loop).

---

## 6. API (GraphQL v0)

Milestone 1 should implement the minimal API needed to prove the kernel is alive, deterministic, and inspectable.

This is intended to be compatible with [SPEC-NET-0001](../../SPECS/SPEC-NET-0001-graphql-sdl-v0.md), but *Milestone 1 only requires a strict subset*.

Milestone-1 constraint: avoid ambiguous routing inputs. There must be a single source of truth for which view a mutation targets (prefer `ViewRefInput` in `applyRewrite(view, rewrite)`).

### Queries
Milestone 1 adopts a single canonical digest surface for external determinism validation:

- `graph(view: ViewRefInput!): GraphSnapshot!` where `GraphSnapshot` includes `digest: Hash`.

Additional required queries:
- `listSws(...)` (or equivalent) to enumerate SWS contexts.
- `rewrites(...)` to retrieve an append-only rewrite log since boot, ordered deterministically by `idx` ascending.

Deferred/optional (not required for Milestone 1):
- dedicated `systemHash` / `swsHash(id)` fields (can be derived from `graph(view).digest` later)
- `graphHash(view)` convenience field

### Mutations
- `createSws(...)`
- `discardSws(...)`
- `applyRewrite(...)` with at least `AddNode` supported (into an SWS overlay).

### M1 Frozen API Contract (implementation must match)

These choices are frozen for Milestone 1 to prevent “interpretation drift” between docs, clients, and kernel.

- **Hash encoding:** `Hash` strings are lowercase hex, length 64, representing 32-byte BLAKE3 digests. No `0x` prefix.
- **Rewrite ops:** `RewriteInput.ops: [JSON!]!` contains JSON objects. Milestone 1 supports exactly one op:

  ```json
  {
    "op": "AddNode",
    "data": {
      "kind": "demo",
      "payload_b64": "aGVsbG8="
    }
  }
  ```

  Notes:
  - `payload_b64` decodes to bytes and is hashed as-is (no canonicalization step inside the kernel in M1).
  - `kind` is case-sensitive.
  - extra fields are rejected (see error policy).
- **Receipt v0:** `applyRewrite` returns a deterministic receipt containing:
  - `rewriteIdx: U64` (global monotone sequence since boot)
  - `view: ViewRef`
  - `viewDigest: Hash` (digest after applying the rewrite to that view)
- **Pagination:** implement `first` only; `after` returns `NOT_IMPLEMENTED` in M1.
- **Timestamps:** all `Timestamp` fields return `null` in M1.
- **Errors:** GraphQL errors include `extensions.code` with one of:
  - `INVALID_INPUT` (bad ID format, schema mismatch, bad base64, missing required fields)
  - `NOT_FOUND` (SWS id doesn’t exist)
  - `NOT_IMPLEMENTED` (unsupported op variant, `after` cursor, `collapseSws`, `submitIntent`, etc.)
  - `INTERNAL` (kernel loop down, invariant violated, unexpected errors)

---

## 7. Testing Strategy

### Unit Tests (Warp Core)
- Stable `NodeId`: same `(kind, payload)` produces identical `NodeId` across runs/platforms.
- Stable `GraphDigest`: inserting nodes/edges in different orders produces the same `GraphDigest`.

### Unit Tests (Kernel)
- SWS isolation: overlay changes are not visible in system graph queries.
- Deterministic `SwsId`: SWS IDs assigned sequentially and reproducibly.
- Deterministic rewrite log: `idx` increments exactly once per mutation in correct order.

### Integration Test (End-to-End)
1. Start `jitosd` in a test harness.
2. Query the System digest surface (capture `H0`).
3. `createSws` → get `swsId = 0`.
4. `applyRewrite(AddNode)` to SWS `0`.
5. Query the SWS digest surface for `0` and assert it differs from `H0`.
6. Query the System digest surface again and assert it still equals `H0`.
7. Query `rewrites` shows deterministic ordered events.
8. Repeat the exact same mutation sequence in a fresh process; assert identical outputs/hashes/log.

---

## 8. Deliverables
1. `jitosd` binary (runs locally).
2. GraphQL endpoint with the minimal schema above.
3. Deterministic hashing & replay invariants implemented (laws satisfied).
4. Passing tests: `cargo test` green (unit + integration).
5. Milestone report (PDF): scope, invariants, API summary, tests, known gaps.

---

## 9. Definition of Done (Milestone Gate)

Milestone 1 is **DONE** when all are true:

- `cargo run -p jitosd` starts successfully and serves `/graphql`.
- GraphQL supports:
  - SWS lifecycle: `createSws`, `listSws`, `discardSws`
  - rewriting: `applyRewrite` (AddNode minimum)
  - digest surface: system digest + SWS digest via `graph(view).digest`
  - rewrite log query: `rewrites(...)`
- Determinism proofs exist and pass:
  - `GraphDigest` is stable under insertion order changes (unit test).
  - kernel mutation order is single-writer (architecture/code structure).
  - re-running the same scripted mutation sequence yields identical system digest output, SWS digest output, and rewrite log output.
  - SWS overlay isolation confirmed by integration test.
- No UUIDs participate in kernel semantic identity (Alpha).

---

## 10. Task Checklist (Inline)

This section is the execution checklist for the milestone, sequenced to protect determinism (laws → hashes → kernel → network → e2e proof).

Primary docs (must not contradict):
- [ARCH-0001](../../ARCH/ARCH-0001-universal-job-fabric.md)
- [SPEC-0001](../../SPECS/SPEC-0001-canonical-encoding.md)
- [SPEC-0005](../../SPECS/SPEC-0005-deterministic-ids.md)
- [SPEC-NET-0001](../../SPECS/SPEC-NET-0001-graphql-sdl-v0.md)

### Phase 0 — Lock the Laws (mandatory, fast)

Goal: decide and document determinism-critical choices *before* implementing networking.

**M1 Frozen Contract Choices (Codex must implement exactly):**
- Hash encoding: lowercase hex, 64 chars, 32 bytes. No prefix.
- Rewrite ops: `RewriteInput.ops` contains JSON objects. M1 supports only:
  - `{"op":"AddNode","data":{"kind":<string>,"payload_b64":<base64>}}`
- Receipt v0: `applyRewrite` returns a deterministic receipt containing:
  - `rewriteIdx: U64`, `view: ViewRef`, `viewDigest: Hash`
- Pagination: `first` supported; `after` returns `NOT_IMPLEMENTED` in M1.
- Timestamps: all `Timestamp` fields return `null` in M1.
- Errors: `extensions.code` is one of `INVALID_INPUT`, `NOT_FOUND`, `NOT_IMPLEMENTED`, `INTERNAL`.

- [ ] Confirm canonical-bytes rule for payloads (30–60m)
  - [ ] Kernel treats node/edge payload as opaque canonical bytes (no helpful decoding).
  - [ ] Milestone 1 payload rule: node payload is provided as base64 bytes and is hashed as-is.
  - [ ] If using a structured encoding (e.g., canonical CBOR), define *exact* canonicalization (spec + tests) (future milestone).
- [ ] Finalize deterministic identity derivations (30–60m)
  - [ ] `NodeId = blake3(canonical(kind, payload))` (no self-inclusion).
  - [ ] `EdgeId = blake3(canonical(from, to, kind))`.
  - [ ] Choose hash string encoding for API (`hex` vs `base32`) and do not change it later.
- [ ] Finalize graph digest law (30–60m)
  - [ ] `GraphDigest` folds nodes by sorted `NodeId` bytes and edges by sorted `EdgeId` bytes.
  - [ ] Explicitly outlaw `HashMap` iteration affecting digests (sort or use `BTree*`).
- [ ] Finalize kernel ordering law (30–60m)
  - [ ] All mutations go through a single-writer command loop (no concurrent writers).
  - [ ] `SwsId: u64` allocated deterministically (`0, 1, 2, ...`).
  - [ ] `RewriteEvent.idx: u64` increments once per applied mutation.
- [ ] Spec alignment note / addendum (30–60m)
  - [ ] Either add a short “graph digests” addendum to existing specs, or introduce a new spec doc and link it here.

### Phase 1 — Warp Core (`jitos-warp-core`): make hashes real first

Goal: deterministic IDs + deterministic digests with behavioral tests.

- [ ] Implement content-addressed Node/Edge IDs (60–120m)
  - [ ] `Node { kind, payload }` (payload = canonical bytes).
  - [ ] `Edge { from, to, kind }` (minimal edge model for M1).
  - [ ] `NodeId` / `EdgeId` derivation functions.
- [ ] Implement in-memory `Graph` + deterministic `digest()` (60–180m)
  - [ ] Store nodes/edges (internal collections may be `HashMap`).
  - [ ] `digest()` must fold in sorted ID order (stable across insertion order).
  - [ ] Provide sorted snapshot iterators for API output (`nodes_sorted()`, `edges_sorted()`).
- [ ] Warp-core unit tests (behavioral) (60–120m)
  - [ ] stable `NodeId` across runs given identical `(kind, payload)`.
  - [ ] stable `GraphDigest` under permuted insertion order for same set of nodes/edges.

### Phase 2 — Kernel (`jitos-kernel`): single-writer + SWS overlays + rewrite log

Goal: kernel owns semantics; ordering and logging are deterministic by construction.

- [ ] Single-writer kernel loop (60–120m)
  - [ ] Define command surface (`KernelCmd`) for queries + mutations.
  - [ ] Implement a single task that owns all mutable kernel state and processes commands sequentially.
- [ ] SWS lifecycle with deterministic IDs (60–120m)
  - [ ] create/list/discard with `SwsId: u64` allocator.
  - [ ] capture `base_digest` at SWS creation.
- [ ] SWS overlay semantics (read-through, overlay-only writes) (120–180m)
  - [ ] reads check overlay first, then system.
  - [ ] writes go to overlay only (Alpha).
- [ ] Rewrite application + append-only rewrite log (120–180m)
  - [ ] implement `AddNode` into SWS overlay (minimum).
  - [ ] append `RewriteEvent { idx, view, ops, meta, receipt }` deterministically.
  - [ ] expose a query to retrieve rewrites by `(view, after, limit)` or equivalent cursor.
- [ ] Kernel unit tests (behavioral) (60–120m)
  - [ ] deterministic `SwsId` allocation.
  - [ ] SWS isolation: overlay changes not visible in System view.
  - [ ] rewrite log deterministic ordering and monotone indexing.

### Phase 3 — Net/API (`jitos-net`): thin GraphQL adapter

Goal: GraphQL is a control plane; kernel is the semantics engine.

- [ ] Scaffold Axum + async-graphql (30–90m)
  - [ ] `/graphql` endpoint and GraphiQL.
  - [ ] optional `/ws` for subscriptions (nice-to-have; not required for M1 gate).
- [ ] Implement Milestone-1 subset of SPEC-NET-0001 (120–180m)
  - [ ] SWS lifecycle queries/mutations (`createSws`, `listSws`, `discardSws`).
  - [ ] `applyRewrite` targeting a view (SWS minimum).
  - [ ] rewrite log query (`rewrites(...)`) for operator-grade audit since boot.
  - [ ] `graph(view)` returns `GraphSnapshot.digest` (canonical digest surface for M1).
- [ ] Determinism hygiene in API output (60–120m)
  - [ ] graph snapshots return nodes/edges in deterministic ordering (sorted by ID bytes).
  - [ ] timestamps are `null` for Milestone 1 (avoid wall-clock nondeterminism).
  - [ ] avoid “two sources of truth” routing (prefer `ViewRefInput` as the only routing input).
  - [ ] M1 pagination: implement `first` only; `after` may return `NOT_IMPLEMENTED`/`INVALID_INPUT`. Ordering is always deterministic (sorted by ID/idx).

### Phase 4 — `jitosd` + integration tests + milestone report

Goal: prove the system boots, is inspectable, and replay-deterministic end-to-end.

- [ ] `bins/jitosd` daemon (60–120m)
  - [ ] `--host`, `--port` flags (default `127.0.0.1:8080`).
  - [ ] spawn kernel loop; mount GraphQL router.
- [ ] End-to-end determinism integration test (120–180m)
  - [ ] start daemon in test harness.
  - [ ] run scripted mutation sequence:
    - [ ] capture System digest `H0`.
    - [ ] create SWS (expect deterministic `swsId = 0`).
    - [ ] apply `AddNode` into SWS 0.
    - [ ] assert SWS digest differs, System digest unchanged.
    - [ ] query rewrite log and assert deterministic event ordering.
  - [ ] repeat script in a fresh process; assert identical outputs (digests + rewrite log).
- [ ] Milestone report PDF (30–90m)
  - [ ] build with `make milestone-1` (produces `MILESTONE-1.pdf`).

---

## 11. Explicit Non-Goals (to prevent scope creep)

These are out of scope for Milestone 1 unless explicitly pulled into the gate:

- persistence / WAL / database-backed stores
- collapse semantics / merge into System truth
- worker execution / schedulers / tasks beyond kernel-internal operations
- wall-clock timestamps (return `null`)
