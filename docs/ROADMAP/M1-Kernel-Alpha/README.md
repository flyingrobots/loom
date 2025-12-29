# Milestone 1: Kernel Genesis (Alpha)

**Status:** Planned
**Target Date:** TBD
**Owner:** James Ross
**Context:** Foundational implementation of the JITOS Kernel (ARCH-0001).

---

## 1. Executive Summary
This milestone establishes the **bootable kernel** of JITOS. It delivers the core Rust crates (`jitos-warp-core`, `jitos-kernel`, `jitos-net`) and a running `jitosd` daemon that can accept GraphQL queries, maintain an in-memory System WARP graph, and support basic SWS (Shadow Working Set) lifecycle operations.

**Goal:** A running process (`jitosd`) that I can talk to via GraphQL (`jitos-net`) which holds a valid, hash-linked causal graph (`jitos-warp-core`) and can branch it into overlays (`jitos-kernel` SWS).

---

## 2. User Stories

### US-1: The Living Graph
> As a developer, I want to start `jitosd` and query its "System Graph" via GraphQL so that I can verify the kernel is running and holding state.

### US-2: Speculative Workspace
> As a tool author, I want to create a new SWS (Shadow Working Set) via API so that I can have an isolated scratchpad for my changes.

### US-3: The Immutable Log
> As an operator, I want to see a log of "Rewrites" (events) that have occurred in the system so that I can audit what `jitosd` has done since boot.

### US-4: Deterministic Identity
> As a kernel engineer, I want to verify that two identical nodes created in different SWSs have the exact same Hash ID so that content-addressing is preserved.

---

## 3. Requirements

### Functional
1.  **Daemon:** `jitosd` binary must compile and run.
2.  **API:** GraphQL endpoint at `http://127.0.0.1:8080/graphql`.
3.  **Graph:** In-memory graph structure supporting Nodes and Edges.
4.  **Identity:** BLAKE3-based canonical content addressing for all nodes.
5.  **SWS:** Ability to `create`, `list`, and `discard` SWS contexts.
6.  **Rewrites:** Ability to apply a basic `AddNode` rewrite to an SWS.

### Non-Functional
1.  **Rust:** 100% Safe Rust (where possible).
2.  **Async:** Tokio-based runtime.
3.  **Performance:** Graph operations should be < 1ms for toy sizes (< 10k nodes).
4.  **Determinism:** Replaying the same sequence of API calls must result in the exact same Graph Hash.

### Constraints
*   No database (yet). In-memory only for Alpha.
*   No persistent WAL (yet). State is lost on restart.
*   No real "workers" (yet). Only kernel-internal operations.

---

## 4. Architecture & Design

### 4.1. Crate Hierarchy
```
crates/
├── jitos-core       # (Existing) Hashes, Slap types
├── jitos-warp-core  # (New) The Graph Engine. Nodes, Edges, Store trait.
├── jitos-kernel     # (New) The OS Logic. SystemWARP, SWS Manager, Process Table.
└── jitos-net        # (New) Axum + Async-GraphQL adapter.
bins/
└── jitosd           # (New) The executable entry point.
```

### 4.2. Data Model (Core)

**Node:**
```rust
struct Node {
    id: Hash,
    kind: String,
    payload: Vec<u8>, // Canonical CBOR
}
```

**SWS:**
```rust
struct Sws {
    id: Uuid,
    base_snapshot: Hash, // Points to SystemWARP
    overlay: HashMap<Hash, Node>, // Copy-on-write layer
}
```

**Kernel State:**
```rust
struct Kernel {
    system_graph: Graph,
    sws_registry: HashMap<Uuid, Sws>,
}
```

---

## 5. Implementation Plan

### 5.1. Module: `jitos-warp-core`
*   Define `Node`, `Edge`, `Graph` structs.
*   Implement `CanonicalSerialize` for Node.
*   Implement `Store` trait (in-memory `HashMap` backend).

### 5.2. Module: `jitos-kernel`
*   Define `Kernel` struct wrapping `warp-core`.
*   Implement `SwsManager` to handle `create_sws`, `discard_sws`.
*   Implement `apply_rewrite` (basic insert/update).

### 5.3. Module: `jitos-net`
*   Define GraphQL Schema (SPEC-NET-0001 implementation).
*   Setup Axum server.
*   Wire `Mutation.createSws` -> `Kernel.create_sws`.

---

## 6. Testing Strategy

### Unit Tests
*   **Warp Core:** Verify hash stability. `Node A` on Mac must hash same as `Node A` on Linux.
*   **SWS:** Verify isolation. Adding node to SWS must NOT show up in System Graph.

### Integration Tests
*   **API:** Spin up `jitos-net` mock, send GraphQL query, assert JSON response.

---

## 7. Deliverables
1.  `jitosd` binary.
2.  `cargo test` passing suite.
3.  Milestone Report (PDF).
