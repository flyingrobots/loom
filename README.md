# JITOS (Loom)

> **The Universal Job Fabric**
>
> A history-native, deterministic operating system where work is modeled as jobs over a causal graph.

![JITOS Architecture](docs/SVG/highlevel.svg)

---

## 🏗️ The New Architecture (ARCH-0001)

JITOS has pivoted from a collection of loose tools to a unified **"Monolith with Seams"** architecture. It integrates:

*   **WARP Graphs:** The deterministic causal substrate (History as Truth).
*   **Echo Engine:** The rewrite execution loop.
*   **TASKS/SLAPS:** The intent planning layer.
*   **SWS (Shadow Working Sets):** Speculative process isolation.
*   **GraphQL:** The universal control plane.

### Core Invariants (ARCH-0002)
1.  **History is First-Class:** State is just a view derived from events.
2.  **Speculation is Default:** Risky work happens in overlays (SWS).
3.  **Intent ≠ Plan ≠ Execution:** Explicit separation of concerns.

---

## 🗺️ Roadmap

We are currently executing **Phase 0: Kernel Skeleton**.

*   **Phase 0:** `jitosd` daemon with in-memory WARP graph.
*   **Phase 1:** Live visualization via GraphQL subscriptions.
*   **Phase 2:** SWS overlays (branching/merging).
*   **Phase 3:** Deterministic Planning (`jitos-planner`).
*   **Phase 4:** End-to-End Execution (`submit_intent` -> Worker).
*   **Phase 5:** Real Workers (Shell/LLM).

See [NEXT-MOVES.md](./NEXT-MOVES.md) for the daily execution plan.

---

## 📚 Documentation

*   **[Docs Tour](docs/TOUR.md)** - Where things live (Theory → Arch → Specs → Roadmap).
*   **[Milestone Roadmap (MOC)](docs/ROADMAP/README.md)** - Approval-ready milestone plans + DAGs.
*   **[ARCH-0001: Universal Job Fabric](docs/ARCH/ARCH-0001-universal-job-fabric.md)** - The Blueprint.
*   **[ARCH-0002: Architectural Invariants](docs/ARCH/ARCH-0002-architectural-invariants.md)** - The Constitution.
*   **[SPEC-NET-0001: GraphQL SDL](docs/SPECS/SPEC-NET-0001-graphql-sdl-v0.md)** - The API.
*   **[Theory (WARP/Aion)](docs/THEORY.md)** - The Math.

---

## 🧩 Modules (Crates)

*   `crates/jitos-core`: Foundational types (Hash, Slap).
*   `crates/jitos-planner`: Port of TASKS/SLAPS planning logic.
*   *(Coming Soon)* `crates/jitos-warp-core`: Deterministic graph engine.
*   *(Coming Soon)* `crates/jitos-kernel`: OS core (SWS/Process manager).
*   *(Coming Soon)* `crates/jitos-net`: GraphQL API.

---

## 🌐 The "Meta" Layer

This project powers **[flyingrobots.dev](https://flyingrobots.dev)**, which is a live, recursive instance of a WARP graph rendering itself.

> *"If an OS can’t tell you why something happened, it’s not a system — it’s a haunted house with logs."*
