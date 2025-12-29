# Milestone 1: Kernel Genesis (Alpha) - Tasks

**Context:** Execute the "Monolith with Seams" strategy (ARCH-0001).
**Goal:** A running `jitosd` daemon with in-memory graph and GraphQL API.

---

## 1. Core Scaffolding & Types
- [ ] **Create `jitos-warp-core` crate** (30m)
    - [ ] Initialize crate with `Cargo.toml`.
    - [ ] Port `canonical.rs` and `Hash` types from `jitos-core` (or re-export).
    - [ ] Define `Node`, `Edge`, `Graph` structs (in-memory backend).
    - [ ] Implement `Store` trait for in-memory graph.
- [ ] **Create `jitos-kernel` crate** (30m)
    - [ ] Initialize crate.
    - [ ] Define `Kernel` struct holding `SystemWARP`.
    - [ ] Define `Sws` struct (Overlay graph).
    - [ ] Implement `Kernel::new()`.

## 2. Graph Engine Implementation (`jitos-warp-core`)
- [ ] **Implement Node Identity** (1h)
    - [ ] Implement `CanonicalSerialize` for `Node`.
    - [ ] Verify `NodeId` generation matches `blake3(canonical(node))`.
- [ ] **Implement Graph Storage** (2h)
    - [ ] `Graph::add_node(node) -> Result<NodeId>`.
    - [ ] `Graph::get_node(id) -> Option<Node>`.
    - [ ] `Graph::add_edge(from, to, kind) -> Result<EdgeId>`.
- [ ] **Implement Basic Rewrite Op** (2h)
    - [ ] Define `RewriteOp` enum (`AddNode`, `DeleteNode`, `Connect`).
    - [ ] Implement `apply_rewrite(graph, op) -> Result<Receipt>`.

## 3. Kernel Logic (`jitos-kernel`)
- [ ] **SWS Manager** (2h)
    - [ ] `create_sws(base_snapshot) -> SwsId`.
    - [ ] `get_sws(id) -> &Sws`.
    - [ ] `discard_sws(id)`.
- [ ] **Overlay Logic** (3h)
    - [ ] Implement "Read-Through" logic: `Sws::get_node` checks overlay then base.
    - [ ] Implement "Write-Buffer" logic: `Sws::apply_rewrite` writes to overlay only.

## 4. Network & API (`jitos-net`)
- [ ] **Scaffold Axum + Async-GraphQL** (1h)
    - [ ] Setup `jitos-net` crate with dependencies.
    - [ ] Create basic `axum` router.
- [ ] **Implement GraphQL Schema v0** (3h)
    - [ ] Port `SPEC-NET-0001` SDL to `async-graphql` macros.
    - [ ] Implement `Query.graph(view)`.
    - [ ] Implement `Mutation.createSws`.
    - [ ] Implement `Mutation.applyRewrite`.
- [ ] **Wire Kernel to API** (2h)
    - [ ] Inject `Arc<Kernel>` into GraphQL context.
    - [ ] Map GraphQL inputs to Kernel method calls.

## 5. Daemon Entry Point (`bins/jitosd`)
- [ ] **Create `bins/jitosd`** (30m)
    - [ ] Setup `main.rs` with `tokio` runtime.
    - [ ] Initialize `Kernel`.
    - [ ] Start `jitos-net` server.
- [ ] **CLI Argument Parsing** (30m)
    - [ ] Add `--port`, `--host` flags using `clap`.

## 6. Verification & Documentation
- [ ] **Unit Tests: Warp Core** (1h)
    - [ ] Test hash stability across 2 different struct instantiations.
- [ ] **Integration Test: End-to-End** (2h)
    - [ ] Start daemon (test harness).
    - [ ] Create SWS via GraphQL.
    - [ ] Add node to SWS.
    - [ ] Query SWS graph.
    - [ ] Verify node exists in SWS but NOT in System graph.
- [ ] **Milestone Report** (1h)
    - [ ] Compile LaTeX report (`make milestone-1`).
