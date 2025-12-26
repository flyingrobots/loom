# ARCH-0010: The Slice Theorem for WARP Graphs

## 1. Summary

This document formalizes the unification of the JITOS subsystems (Scheduler, Replay, Ledger) as distinct **geometric cross-sections** of a single underlying WARP Graph object. By proving correctness on the graph, we inherit guarantees for all subsystems.

## 2. The Core Concept

*   **WARP Graph (\(\\mathcal{W}\\\):** The single source of truth. Nodes are states; typed edges are double-pushout (DPO) rewrites.
*   **Worldline:** A path through \(\\mathcal{W}\\) with a fixed witness/schedule.
*   **Slice:** A functor that holds certain coordinates fixed and observes the rest.

## 3. The Three Slices

### 3.1 The Scheduling Slice (\(\\pi_{\\text{sched}}\\\)
*   **Domain:** The raw derivation.
*   **Codomain:** A **Partial Order (DAG)** of causality.
*   **Mechanism:** Independence is defined by the non-overlap of DPO pushout complements (Footprints).
*   **JITOS Component:** **Echo Scheduler**.
*   **Guarantee:** Any linear extension of this DAG is a valid execution. No race-induced divergence.

### 3.2 The Replay Slice (\(\\pi_{\\text{replay}}\\\)
*   **Domain:** The derivation + Boundary Data \(B = (U_0, P)\\\).
*   **Codomain:** A **Linear Path** (Worldline) \(U_0 \Rightarrow \dots \Rightarrow U_n\\\).
*   **Mechanism:** Fixes the schedule choice \(\Sigma\\\).
*   **JITOS Component:** **WarpKernel Replay**.
*   **Guarantee:** Byte-identical state reconstruction.

### 3.3 The Ledger Slice (\(\\pi_{\\text{ledger}}\\\)
*   **Domain:** The derivation.
*   **Codomain:** A **Quotient Path** of tamper-evident checkpoints (BTRs).
*   **Mechanism:** Collapses ticks by a hash/signature congruence. Uses `normalize()` to canonicalize independent events.
*   **JITOS Component:** **Shiplog (WAL)**.
*   **Guarantee:** Tamper-evidence, provenance-completeness, light-client verification.

## 4. The JITOS Slice Theorem

For any derivation \(d\\) in \(\\mathcal{W}\\\), the triple (\(\\pi_{\\text{sched}}(d)\\\), \(\\pi_{\\text{replay}}(d)\\\), \(\\pi_{\\text{ledger}}(d)\\\)) forms a compatible fibered cone over \(d\\\).

**Implications:**
1.  **Concurrency Commutes with Replay:** Unique linear extension inside each antichain.
2.  **Ledger Integrity:** Ledger quotients respect both the partial-order structure and the chosen linearization.
3.  **Holography:** \(d\\) is uniquely determined by any two slices plus boundary data \(B\\\).

## 5. Engineering Spec

### Invariants
1.  **Determinism:** For fixed \(B\\) and \(\Sigma\\\), worldline is unique.
2.  **Commutation on Antichains:** If \(\mu_i\\) and \(\mu_j\\) are independent, swapping them preserves state.
3.  **Ledger Congruence:** BTR chain hashes are equal under normalization of antichains.

### APIs
*   `plan(U0, R, Sigma) -> POSET`: Returns causality DAG.
*   `replay(U0, P) -> U_n`: Verifies patches against DAG constraints.
*   `checkpoint(U_i, mu_i) -> BTR_i`: Emits `hash(sig(prev), mu_i, auth, time)`.
*   `normalize(BTR*) -> BTR*`: Canonicalizes antichain reorderings.

## 6. Implementation Strategy

*   **Echo (Kernel):** Implements `plan()` via Radix Sort.
*   **Shiplog (Provenance):** Implements `checkpoint()` and `normalize()` via BLAKE3.
*   **Debugger (Shell):** Visualizes the projection of \(\\pi_{\\text{sched}}\\) (DAG) onto \(\\pi_{\\text{replay}}\\) (Timeline).
