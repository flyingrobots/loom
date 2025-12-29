# JITOS Next Moves: Universal Job Fabric

**Status:** Execution Roadmap
**Context:** Pivoting to "Monolith with Seams" Architecture (ARCH-0001)

---

## 🚀 The Plan: JITOS as a Universal Job Fabric

We are unifying the stack (`jitos`, `echo`, `tasks`, `ninelives`) into a single coherent operating system kernel where:
1.  **History is Truth:** The authoritative record is events/rewrites.
2.  **Speculation is Default:** Work happens in overlays (SWS).
3.  **Planning is Explicit:** Intent (TASK) -> Plan (SLAP) -> Execution (Worker).

### Phase 0: Kernel Skeleton (Current Focus)
**Goal:** Have `jitosd` running with a real WARP graph in memory and a trivial API.

- [ ] **Workspace Restructure:**
    - [x] Create `crates/jitos-planner` (Porting TASKS/SLAPS).
    - [ ] Create `crates/jitos-warp-core` (The deterministic graph engine).
    - [ ] Create `crates/jitos-sched` (The tick loop).
    - [ ] Create `crates/jitos-kernel` (The OS core owning WARP + SWS).
    - [ ] Create `crates/jitos-net` (GraphQL API).
    - [ ] Create `crates/jitos-workers` (Worker registry).
    - [ ] Create `bins/jitosd` (The daemon).

- [ ] **Core Implementation:**
    - [ ] Implement `jitos-warp-core`: Basic node/edge/graph structs + Canonical Hashing.
    - [ ] Implement `jitos-kernel`: `SystemWARP`, `SWS` structs.
    - [ ] Implement `jitos-net`: Basic GraphQL v0 schema (SPEC-NET-0001).

### Phase 1: Live Visualization
**Goal:** Connect a viewer to the running daemon.

- [ ] Implement `Subscription` in GraphQL for live rewrites.
- [ ] Build basic `jitos-viewer` (or connect existing Echo viewer).

### Phase 2: SWS Overlays
**Goal:** Speculative graph overlays with collapse/merge.

- [ ] Implement `create_sws`, `apply_rewrite`, `collapse_sws`.
- [ ] Visualize overlays in the viewer.

### Phase 3: Planning (jitos-planner)
**Goal:** Deterministic planning as a library.

- [x] Define Rust structs for SLAPS/Task/Method.
- [ ] Port HTN planning logic from Go to Rust.
- [ ] Implement `plan(task) -> DAG`.

### Phase 4: Integration
**Goal:** `submit_intent` -> Process -> SWS -> Execution.

- [ ] Connect `jitos-planner` to `jitos-kernel`.
- [ ] Implement `submit_intent` mutation.

### Phase 5: Real Workers
**Goal:** Useful automation.

- [ ] Implement `LocalScriptWorker` (runs shell commands).
- [ ] Implement `LLMWorker` (optional).

---

## 🛠️ Immediate Tasks (Next 24 Hours)

1.  **Finish `jitos-planner` Port:**
    - [x] `slaps.rs`
    - [ ] `task.rs`
    - [ ] `method.rs`
    - [ ] `dag.rs`
    - [ ] Basic serialization tests.

2.  **Scaffold `jitos-warp-core`:**
    - [ ] Create crate.
    - [ ] Port `canonical.rs` and `events.rs` from `jitos-core` into the new structure (or rename/refactor `jitos-core` to `jitos-warp-core`).

3.  **Scaffold `jitos-net`:**
    - [ ] Setup `async-graphql` + `axum`.
    - [ ] Implement the `schema.graphql` from SPEC-NET-0001.

---

## 📚 References
- [ARCH-0001: Universal Job Fabric](../docs/ARCH/ARCH-0001-universal-job-fabric.md)
- [ARCH-0002: Architectural Invariants](../docs/ARCH/ARCH-0002-architectural-invariants.md)
- [SPEC-NET-0001: GraphQL SDL](../docs/SPECS/SPEC-NET-0001-graphql-sdl-v0.md)