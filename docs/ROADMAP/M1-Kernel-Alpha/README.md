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

It introduces **SWS (Shadow Working Set / Schrödinger Workspace)** as isolated overlays with **read-through semantics** (overlay reads first, then system) and **overlay-only writes** (for Alpha).

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
- “Canonical bytes” means the kernel treats the payload as an opaque byte vector that is already canonical (or is canonicalized before hashing via a single explicit, stable function).
- Avoid “helpful” decoding/normalization inside the kernel for v0; that is a determinism trap.

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
    target: Target,      // System | Sws(SwsId)
    op: RewriteOp,       // AddNode (minimum); Connect optional
    receipt: Receipt,    // minimal, deterministic
}
```

`idx` increments once per applied mutation (through the single-writer loop).

---

## 6. API (GraphQL v0)

Milestone 1 should implement the minimal API needed to prove the kernel is alive, deterministic, and inspectable.

This is intended to be compatible with [SPEC-NET-0001](../../SPECS/SPEC-NET-0001-graphql-sdl-v0.md), but *Milestone 1 only requires a strict subset*.

Milestone-1 constraint: avoid ambiguous routing inputs. There must be a single source of truth for which view a mutation targets (prefer `ViewRefInput` in `applyRewrite(view, rewrite)`).

### Queries
One of the following “digest surfaces” must exist so determinism can be validated externally:

- `systemHash: Hash` and `swsHash(id: ID!): Hash`, **or**
- `graph(view: ViewRefInput!): GraphSnapshot!` where `GraphSnapshot` includes `digest: Hash` (recommended), **or**
- `graphHash(view: ViewRefInput!): Hash!`.

Additionally:
- `listSws(...)` (or equivalent) to enumerate SWS contexts.
- `rewrites(view, page)` or `rewrites(limit, after)` to retrieve an append-only rewrite log since boot.

### Mutations
- `createSws(...)`
- `discardSws(...)`
- `applyRewrite(...)` with at least `AddNode` supported (into an SWS overlay).

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
  - digest surface: system digest + SWS digest (via `systemHash`/`swsHash` or snapshot digest)
  - rewrite log query: `rewrites(...)`
- Determinism proofs exist and pass:
  - `GraphDigest` is stable under insertion order changes (unit test).
  - kernel mutation order is single-writer (architecture/code structure).
  - re-running the same scripted mutation sequence yields identical system digest output, SWS digest output, and rewrite log output.
  - SWS overlay isolation confirmed by integration test.
- No UUIDs participate in kernel semantic identity (Alpha).

---

## 10. Milestone 1 Task Plan (Re-sequenced)

### Phase 0 — Lock the Laws (fast, mandatory)
- [SPEC-0005](../../SPECS/SPEC-0005-deterministic-ids.md): Deterministic IDs
- [SPEC-0001](../../SPECS/SPEC-0001-canonical-encoding.md): Canonical encoding
- Define (or extend with a small addendum) deterministic digests for the in-memory graph:
  - `NodeId` derivation (no self-inclusion)
  - `EdgeId` derivation
  - `GraphDigest` fold ordering rules
  - canonical-bytes rule for payload

### Phase 1 — Warp Core (make hashes real before networking)
- Create `jitos-warp-core` crate
- Implement `NodeId`, `EdgeId`
- Implement `Graph::add_node/get_node/add_edge`
- Implement `Graph::digest()` with sorted fold
- Unit tests: stable `NodeId` + `GraphDigest` under permuted insertion order

### Phase 2 — Kernel (single-writer loop + overlays + log)
- Create `jitos-kernel` crate
- Implement kernel command loop (single writer)
- Implement `SwsId: u64` allocator
- Implement SWS create/list/discard
- Implement overlay read-through + overlay-only write buffer
- Implement rewrite log with deterministic `idx`

### Phase 3 — Net/API (thin wrapper)
- Create `jitos-net` crate (Axum + async-graphql)
- Implement minimal schema v0
- Wire GraphQL mutations to kernel commands
- Expose a digest surface (System + SWS) and `rewrites`

### Phase 4 — `jitosd` + tests + report
- Create `bins/jitosd` with `clap` flags (`--host`, `--port`)
- Integration test harness: start daemon, run scripted GraphQL, assert invariants
- Milestone report PDF
