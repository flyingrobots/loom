# JITOS Next Moves: Execution Plan

**Author:** Claude (Sonnet 4.5)
**Date:** 2025-12-26
**Status:** Execution Roadmap
**Context:** Post-absorption of AION Papers, Echo, flyingrobots.dev, ninelives, and all supporting projects

---

## Executive Summary

We are consolidating **three fragmented implementations** (echo Rust kernel, flyingrobots.dev JS prototype, legacy Go planner) into **one rigorous Rust monorepo** that compiles to WASM and powers a lightweight JS shell. This plan addresses the "Two-Kernel Problem," implements the missing Inversion Engine, integrates ninelives as a deterministic resilience layer, and delivers a stakeholder-ready system.

---

## Phase 0: Documentation & Cleanup (IMMEDIATE)

### 0.1 Clean Up Documentation Cruft
**Why:** Stakeholders need a clear view of the project without wading through legacy drafts.

**Tasks:**
- [ ] Generate `JITOS-CLEAN-UP-LIST.md` with justifications for each removal
- [ ] Archive or delete files identified in cleanup list
- [ ] Consolidate redundant ADRs into canonical ARCH documents
- [ ] Move UNORGANIZED/ content to proper homes or archive

**Deliverable:** Clean `docs/` directory with clear navigation.

---

### 0.2 Finalize SOTU-2025 for Stakeholders
**Why:** This document explains project delays and the path forward.

**Current Issues:**
- Missing prose in Section 4 (just TikZ diagrams with no explanation)
- Inversion Engine not discussed
- No clear migration map from old → new

**Tasks:**
- [ ] Rewrite Section 4 with full narrative explaining the "Bionic Architecture"
- [ ] Add Section 5: "The Inversion Engine" (SWS collapse/merge logic)
- [ ] Add Section 6: "Migration Map" (table showing Source → Destination → Deprecated)
- [ ] Add Section 7: "Tooling Evolution" (Time Travel Debugger, RMG Viewer fate)
- [ ] Verify all three LaTeX versions compile (display, dark, print)
- [ ] Generate PDFs and commit

**Deliverable:** Professional-grade report suitable for stakeholder review.

---

## Phase 1: Core Infrastructure Migration (WEEKS 1-2)

### 1.1 Port Echo Scheduler to jitos-scheduler
**Why:** This is the mathematical heart of determinism—the O(n) Radix scheduler with footprint-based independence checking.

**Source:** `~/git/echo/crates/rmg-core/src/scheduler.rs`
**Target:** `~/git/jitos/crates/jitos-scheduler/src/`

**Tasks:**
- [ ] Copy `RadixScheduler`, `ActiveFootprints`, `GenSet` logic
- [ ] Update imports to use `jitos-core::*` types
- [ ] Add tests from echo (determinism, independence, ordering)
- [ ] Document the Radix sort algorithm in rustdoc
- [ ] Implement `normalize()` for antichain reordering (Slice Theorem requirement)

**Acceptance Criteria:**
- All echo scheduler tests pass
- New test: "antichain swap produces identical normalized hash"

---

### 1.2 Port Echo Graph to jitos-graph
**Why:** The WARP graph is the state container. It must support recursive attachments and content addressing.

**Source:** `~/git/echo/crates/rmg-core/src/graph.rs`
**Target:** `~/git/jitos/crates/jitos-graph/src/`

**Tasks:**
- [ ] Copy `GraphStore`, `NodeRecord`, `EdgeRecord` structures
- [ ] Replace `BTreeMap` with `slotmap` for O(1) access
- [ ] Implement `hash()` method using BLAKE3 (not SHA-256)
- [ ] Implement `diff()` for SWS overlay tracking
- [ ] Add recursive attachment loading (lazy or eager—document policy)

**Acceptance Criteria:**
- Graph can store 100k+ nodes in memory
- Hash computation is deterministic and fast (<10ms for 10k nodes)

---

### 1.3 Implement jitos-inversion (NEW CRATE)
**Why:** This is the missing piece—the deterministic SWS collapse logic that Gemini omitted.

**Conceptual Source:** ARCH-0010 (Slice Theorem), echo conflict policies
**Target:** `~/git/jitos/crates/jitos-inversion/src/`

**Tasks:**
- [ ] Create crate with `InversionEngine` struct
- [ ] Implement `rebase(sws: &GraphOverlay, truth: &WarpGraph) -> Result<Commit, ConflictSet>`
- [ ] Implement conflict detection using footprints
- [ ] Implement conflict policies: `Abort`, `Retry`, `Join`
- [ ] Add `normalize()` integration for deterministic merge order
- [ ] Write extensive tests for conflict scenarios

**Acceptance Criteria:**
- Two independent SWSs can merge without conflicts
- Conflicting SWSs fail deterministically with clear error
- Replay of merge produces identical result

---

### 1.4 Implement jitos-provenance (WAL/Ledger)
**Why:** The WAL is the source of truth. Boot is resurrection.

**Conceptual Source:** RFC-0001, RFC-0022, shiplog patterns
**Target:** `~/git/jitos/crates/jitos-provenance/src/`

**Tasks:**
- [ ] Define `Receipt` format (tick, inputs, rules, state hash, signature)
- [ ] Define `Event` enum (Input, Decision, Claim, Anchor) per ARCH-0009
- [ ] Implement `WalAdapter` with IndexedDB backend (browser) and file backend (daemon)
- [ ] Implement `readRange(start, end)` for optimized replay
- [ ] Implement `verify()` using BLAKE3 hash chains
- [ ] Port `Compliance` logic from libgitledger (policy verification)

**Acceptance Criteria:**
- WAL can persist 1M+ ticks without corruption
- Replay from WAL produces byte-identical state hashes
- Compliance checks reject unsigned or policy-violating entries

---

## Phase 2: Resilience & Policy Layers (WEEKS 3-4)

### 2.1 Port ninelives to jitos-resilience
**Why:** We need mature I/O patterns, but they must be deterministic.

**Source:** `~/git/ninelives/src/`
**Target:** `~/git/jitos/crates/jitos-resilience/src/`

**Critical Refactors:**
- [ ] Replace `std::time::Instant` with `JitosTime` trait (injected dependency)
- [ ] Replace `rand::thread_rng()` with `DeterministicRng` (seeded from ledger)
- [ ] Move circuit breaker state from `Mutex<State>` to WARP graph nodes
- [ ] Make rate limiting tick-based, not wall-clock-based
- [ ] Document all breaking changes for determinism

**Acceptance Criteria:**
- Retry logic produces identical results when replayed with same seed
- Circuit breaker state transitions are graph rewrites (logged)

---

### 2.2 Implement jitos-policy (Rhai Host)
**Why:** User-defined behavior (rules, agents) must be sandboxed and deterministic.

**Conceptual Source:** gatos policy engine, echo scripting vision
**Target:** `~/git/jitos/crates/jitos-policy/src/`

**Tasks:**
- [ ] Embed Rhai engine
- [ ] Implement instruction metering (prevent infinite loops)
- [ ] Expose `Graph` API to scripts (read-only initially)
- [ ] Implement `PolicyRegistry` (load scripts by hash)
- [ ] Add versioning for script compatibility
- [ ] Write example scripts (physics rules, camera damping)

**Acceptance Criteria:**
- Script execution is deterministic (same input → same output)
- Scripts cannot access IO, network, or wall-clock time
- Scripts can read graph, propose mutations (not execute directly)

---

### 2.3 Implement jitos-io (Port Adapters)
**Why:** The boundary between Pure (kernel) and Messy (world).

**Conceptual Source:** ninelives adapters, echo networking
**Target:** `~/git/jitos/crates/jitos-io/src/`

**Tasks:**
- [ ] Define `Port` trait (async API for external systems)
- [ ] Implement `FilePort` (read/write with event logging)
- [ ] Implement `NetworkPort` (send/recv with deterministic ordering)
- [ ] Implement `ClockPort` (time samples per ARCH-0009)
- [ ] All ports must emit `Input` events to WAL

**Acceptance Criteria:**
- File reads are logged as `fs.device_read` events
- Network messages are logged as `net.recv` events
- Replay never touches real filesystem or network

---

## Phase 3: Planner & WASM Bridge (WEEKS 5-6)

### 3.1 Implement jitos-planner (HTN Logic)
**Why:** High-level intent → low-level graph rewrites.

**Conceptual Source:** Legacy Go planner, wesley directives
**Target:** `~/git/jitos/crates/jitos-planner/src/`

**Tasks:**
- [ ] Define `HtnMethod` struct (goal, preconditions, decomposition)
- [ ] Implement `decompose(slap: Slap) -> Vec<Task>`
- [ ] Implement `TaskExecutor` (maps tasks to graph rewrites)
- [ ] Write example methods (FixBug, DeployService, etc.)
- [ ] Integrate with jitos-kernel (SLAPS → HTN → Scheduler)

**Acceptance Criteria:**
- `SLAP::MoveNode(A, B)` decomposes into atomic rewrites
- Planner produces deterministic task DAGs

---

### 3.2 Implement jitos-wasm (Browser Bridge)
**Why:** Connect Rust kernel to JS shell.

**Target:** `~/git/jitos/crates/jitos-wasm/src/`

**Tasks:**
- [ ] Use `wasm-bindgen` to expose `Kernel::step()`, `Kernel::snapshot()`
- [ ] Expose RPC layer compatible with existing `JitBridge.js`
- [ ] Serialize state updates as binary (not JSON) for performance
- [ ] Add TypeScript type generation via `ts-rs`
- [ ] Build WASM blob and copy to `shell/public/jitd.wasm`

**Acceptance Criteria:**
- JS shell can call `kernel.step()` and receive state delta
- WASM blob is <2MB compressed
- Kernel runs at 60 TPS in Chrome

---

### 3.3 Migrate flyingrobots.dev to Shell
**Why:** The JS code becomes a pure View layer.

**Source:** `~/git/flyingrobots.dev/src/`
**Target:** `~/git/jitos/shell/src/`

**Tasks:**
- [ ] Move React components to `shell/`
- [ ] Replace `WarpKernel.js` with WASM import
- [ ] Update `JitBridge.js` to call WASM kernel
- [ ] Keep `EngineRoomController` (UI orchestration)
- [ ] Keep `ThreeGraphicsPort` (renderer)
- [ ] Delete JS `Ledger.js`, `Scheduler.js`, `WarpKernel.js` (replaced by WASM)

**Acceptance Criteria:**
- UI runs against WASM kernel
- Time Travel Debugger still works (reading from WASM-generated WAL)

---

## Phase 4: Storage & Performance (WEEKS 7-8)

### 4.1 Implement CAS Layer (Content-Addressed Store)
**Why:** O(1) time travel instead of O(N) replay.

**Conceptual Source:** ARCH-0009, gatos opaque pointers
**Target:** `~/git/jitos/crates/jitos-provenance/src/cas.rs`

**Tasks:**
- [ ] Implement `CasAdapter` with IndexedDB backend
- [ ] Store `Hash → CompressedGraph` mappings
- [ ] Implement `TickIndex` (Tick → Hash lookup)
- [ ] Implement snapshot policy (every N ticks, exponential decay)
- [ ] Integrate with WAL replay (load snapshot + replay tail)

**Acceptance Criteria:**
- Boot time <1s for 100k tick history
- Time travel to arbitrary tick is <100ms

---

### 4.2 Implement Storage Tiering
**Why:** Browser storage is finite; long histories need archival.

**Target:** `~/git/jitos/crates/jitos-provenance/src/tiers.rs`

**Tasks:**
- [ ] Tier 0 (Instant): JS heap, last 100 ticks
- [ ] Tier 1 (Hot): IndexedDB, full session WAL + snapshots
- [ ] Tier 2 (Warm): Reserved for future (server relay)
- [ ] Tier 3 (Cold): Export BTR files (download/upload)
- [ ] Implement `TierManager` (eviction policy)

**Acceptance Criteria:**
- Hot tier usage <500MB per session
- BTR export/import round-trips successfully

---

## Phase 5: Debugging & Tooling (WEEKS 9-10)

### 5.1 Enhance Time Travel Debugger
**Why:** The killer feature.

**Target:** `~/git/jitos/shell/src/debugger/`

**Tasks:**
- [ ] Implement "Tick View" (chronological scrubbing)
- [ ] Implement "Causality View" (DAG visualization per Slice Theorem)
- [ ] Implement "State View" (hash-addressed state inspection)
- [ ] Implement "Verify" (ledger hash chain validation)
- [ ] Add search: "Find tick where property X changed"

**Acceptance Criteria:**
- Can scrub through 10k tick history smoothly
- Can bisect to find bug introduction point

---

### 5.2 Deprecate Echo RMG Viewer
**Why:** Redundant with web-based debugger.

**Tasks:**
- [ ] Document migration path (viewer → web debugger)
- [ ] Archive echo viewer code
- [ ] Update docs to reference web debugger only

---

## Phase 6: Compliance & Audit (WEEKS 11-12)

### 6.1 Implement Compliance Layer
**Why:** Enforce policies on ledger entries (per libgitledger).

**Target:** `~/git/jitos/crates/jitos-provenance/src/compliance.rs`

**Tasks:**
- [ ] Define `Policy` trait (verify entry)
- [ ] Implement policies: `MustBeSigned`, `NoForcePush`, `RequireApproval`
- [ ] Implement `AuditReport` (violations + remediation)
- [ ] Write extensive tests for policy enforcement

**Acceptance Criteria:**
- Unsigned entries are rejected
- Policy violations produce actionable reports

---

### 6.2 Implement Genealogy & Attribution
**Why:** Every decision must be attributable (Paper V ethics).

**Target:** `~/git/jitos/crates/jitos-core/src/attribution.rs`

**Tasks:**
- [ ] Add `agent_id` to every `Receipt`
- [ ] Add `signature` field (Ed25519)
- [ ] Implement `AgentWallet` (key management)
- [ ] Add attribution to graph edges (who linked what)

**Acceptance Criteria:**
- Every ledger entry has a verified signer
- Disputed links are flagged in UI

---

## Critical Success Metrics

### Determinism
- [ ] Golden test: Same ledger replayed 1000x → identical hashes every time
- [ ] Fuzz test: Random inputs + random suspend/resume → replay equality
- [ ] Cross-platform: WASM in Chrome/Firefox/Safari produce identical hashes

### Performance
- [ ] Kernel sustains 60 TPS in browser
- [ ] Boot time <1s for 100k tick history (with snapshots)
- [ ] Suspended CPU usage <0.1%
- [ ] Wake-to-first-tick latency <16ms

### Usability
- [ ] Time travel scrubbing feels instant
- [ ] UI never blocks on kernel computation
- [ ] Error messages are actionable

---

## Dependencies & Risks

### Dependencies
- `wasm-bindgen` (WASM bridge)
- `rhai` (scripting)
- `slotmap` (graph storage)
- `serde` (serialization)
- `blake3` (hashing)
- `ed25519-dalek` (signing)

### Risks
1. **WASM Performance:** If kernel is too slow in browser, may need multi-threading or SharedArrayBuffer.
2. **Determinism Drift:** Floating-point operations may diverge across platforms. May need fixed-point or explicit rounding.
3. **Storage Limits:** IndexedDB quotas vary by browser. May need aggressive compression.

### Mitigations
1. Profile early, optimize hot paths, consider SIMD.
2. Document platform policy, add explicit rounding at commit points.
3. Implement tiering and eviction aggressively.

---

## Stakeholder Deliverables

### Week 2
- [ ] Clean documentation
- [ ] Finalized SOTU-2025 report
- [ ] Core infrastructure (scheduler, graph, inversion) functional

### Week 6
- [ ] WASM kernel running in browser
- [ ] JS shell migrated
- [ ] Basic time travel working

### Week 12
- [ ] Full feature parity with JS prototype
- [ ] Compliance and audit features
- [ ] Production-ready system

---

## Open Questions for Resolution

These decisions require stakeholder input as they affect scope, performance, and user experience:

1. **Camera Policy:** Deterministic (Policy A) or Presentation-only (Policy B)?
   - **Impact:** Policy A enables "visual replay" but increases ledger size ~30%. Policy B is simpler and faster.
   - **Recommendation:** Policy B unless visual replay is a core product promise.

2. **Floating-Point Policy:** Document supported platforms or use fixed-point?
   - **Impact:** Platform restriction (x86-64, ARM64 only) simplifies determinism guarantees but limits exotic hardware support.
   - **Recommendation:** Document platform policy to avoid cross-platform floating-point divergence.

3. **Snapshot Frequency:** Every 60 ticks? Every 600? Tunable?
   - **Impact:** More frequent snapshots = faster time travel but larger storage footprint.
   - **Recommendation:** Tunable with sensible default (100 ticks), exposing as advanced setting.

4. **BTR Format:** JSON (human-readable), CBOR (efficient), or custom binary?
   - **Impact:** JSON aids debugging but bloats archives. CBOR balances readability and size.
   - **Recommendation:** CBOR with optional JSON export tool for auditing.

5. **Total Engineering Effort:** Estimated ~500 hours over 12 weeks (1 FTE). Acceptable?
   - **Impact:** This assumes focused execution without major scope creep or blocking dependencies.
   - **Recommendation:** Frontload Phase 0-1 to validate architecture before committing to full schedule.

---

## Conclusion

This plan consolidates three fragmented implementations into one rigorous system. It prioritizes **determinism** (replay equality), **performance** (60 TPS, <1s boot), and **auditability** (every decision logged). The Rust monorepo strategy eliminates the "Two-Kernel Problem" and positions JITOS as a production-grade causal operating system.

**Next Immediate Action:** Execute Phase 0 (Documentation Cleanup) and finalize SOTU-2025 for stakeholder review.
