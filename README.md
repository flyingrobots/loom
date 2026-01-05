<div align="center">
  <img alt="Continuum" src="https://github.com/user-attachments/assets/89d13603-2e42-4096-ba0c-b44d80add132" />
  <h3><i>The Causal OS</i></h3>
</div>

> [!WARNING]
> **Status: Nascent / Experimental**
> This project is currently in active R&D. Expect raw edges, rapid evolution, and breaking changes as we define the AIΩN specifications.

## The End of Computational Amnesia

For seventy years, computing has followed a blueprint from 1945: a solitary desk worker shuffling papers. Files, folders, mutable RAM—all engineered before the internet, distributed systems, or AI existed.

This paradigm relies on a fundamental flaw: **Destruction.** To write a new value to memory, the machine must destroy the old one. We built our digital world on a foundation of forgetting. This is the breeding ground for concurrency nightmares, race conditions, and "Heisenbugs"—where the state that caused a crash is obliterated the moment it happens.

**Continuum** is the inverse. It is a system based on **perfect memory**.

### The AIΩN Architecture

> [!TIP]
> Continuum is the reference implementation of the [AIΩN Computing](https://github.com/flyingrobots/aion) standard. Visit the AIΩN repo to read the Foundation Series papers.

Continuum is the **Causal OS** engineered for the high-stakes future of autonomous agents and deterministic computing. It replaces "black box" execution with a "glass box" environment where causality is observable, immutable, and provable.

It utilizes **WARP Graphs** (Worldline Algebra for Recursive Provenance) to treat time as the fundamental geometry of the computer. It doesn't write logs, it appends computational holograms to the causal graph. This immutable, append-only ledger _is_ the computation itself, stored. In Continuum, you do not overwrite data; you weave it into the history.

## The Causal Promise

* **Absolute Determinism:** `Input + History = Output`. Always.
* **Perfect Recall:** Traverse the machine's entire state history as effortlessly as reading a variable.
* **Zero Hallucinations:** The system cannot diverge from reality because it is mathematically tethered to its own causal chain.
* **Multiverse-Native:** Time travel is built-in. Rewind execution to any point, fork the timeline, and explore a new worldline without losing the original context.

## System Architecture

Continuum coalesces a suite of deterministic tools into a unified kernel. It is not just a collection of libraries; it is a closed-loop environment managed by the `loom` daemon.

| Component | Role | Description |
| :--- | :--- | :--- |
| **WARP Graphs** | Substrate | The immutable, content-addressed causal graph (History as Truth). |
| **Loom** | Daemon | The kernel process that "weaves" intents into the graph. |
| **Echo Engine** | Runtime | The deterministic rewrite execution loop. |
| **TASKS/SLAPS** | Planning | The intent planning and agentic layer. |
| **SWS** | Isolation | **Shadow Working Sets** for speculative process branching. |
| **Wesley** | Compiler | The `GraphQL -> Everything` bridge. |
| **Nine Lives** | Resilience | Fault tolerance and supervision library. |

## 🗺️ Roadmap

> [!NOTE]
> **Context:** Continuum is being developed alongside **Echo**.
> * **Echo:** A high-performance deterministic game engine (driving feature velocity).
> * **Continuum:** The pedantic, canonical reference implementation of the OS (driving architectural correctness).
>
> Track progress on the [AIΩN Project Board](https://github.com/users/flyingrobots/projects/13).

We are currently executing **Phase 0: Kernel Skeleton**.

- [ ] **Phase 0: The Loom** (`loom` daemon + in-memory WARP graph)
- [ ] **Phase 1: Observation** (Live visualization via GraphQL subscriptions)
- [ ] **Phase 2: Multiverse** (SWS overlays, branching, and merging)
- [ ] **Phase 3: Agency** (Deterministic Planner & Intent Scheduling)
- [ ] **Phase 4: Execution** (End-to-End `submit_intent` → Worker)
- [ ] **Phase 5: Realization** (Shell/LLM Integration)

## License

Continuum © 2026 by James Ross. Continuum is licensed under the [Apache License](./LICENSE.md), Version 2.0 OR [MIND-UCAL](https://github.com/universalcharter/mind-ucal).

> [!note]
> **In short:** you may freely use the theory, papers, and documentation without adopting MIND-UCAL; MIND-UCAL applies only to derivative ethical commitments, not technical use.

---

> *"Things are only impossible until they're not."*
> — Jean-Luc Picard
