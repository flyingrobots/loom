# JITOS Docs Tour

This is a lightweight map of “where to look” in the repository. It exists to reduce rediscovery time and prevent important contracts from getting lost.

If you are about to implement kernel semantics (especially anything that touches determinism, replay, collapse, or time), start with **Theory → Architecture → Specs → Roadmap** in that order.

If you’re feeling lost in doc sprawl, start with:
- `docs/INDEX.md` (map of content)
- `docs/DOCS-GOVERNANCE.md` (what is normative vs directional)

---

## 0) The non-negotiable foundations

### `docs/THEORY.md`
The theoretical substrate (Papers I–VI) that defines why certain design choices are *not optional*:
- WARP graphs as state space
- deterministic worldlines / single-writer ordering discipline
- provenance as an executable boundary (patches/receipts)
- observer geometry and translation costs
- ethics/sovereignty constraints (mind-mode)
- JITOS as an OS boundary (SWS, epochs, WAL, Echo)

If a proposed change contradicts `docs/THEORY.md`, treat it as a design-level break, not a “refactor.”

---

## 1) What we’re building next (approval-ready contracts)

### `docs/ROADMAP/README.md`
Milestone MOC (map-of-content) with:
- cross-milestone dependency DAG
- “What you can do” expectations per milestone
- derived status coloring (done / in-progress / blocked)

### `docs/ROADMAP/M*/README.md`
Each milestone is intended to be executable:
- invariants (“Hard Law”)
- scope + explicit non-goals
- Definition of Done (milestone gate)
- inline task checklist + sequenced DAG

#### Roadmap automation
Mermaid node coloring is derived from checklist progress:
- docs: `docs/ROADMAP/AUTOMATION.md`
- updater: `scripts/update_roadmap_dags.py`
- optional wrapper: `xtask/` (runs without depending on the main Rust workspace)

---

## 2) Architecture decisions (why the code is shaped this way)

### `docs/ARCH/`
System-level architectural anchors (e.g. the “Monolith with Seams” stance).

If you’re not sure where to put a boundary (crate split, API surface, etc.), check ARCH first.

---

## 3) Specs (the “you can implement this without interpretation drift” layer)

### `docs/SPECS/`
Executable contracts: canonical encodings, deterministic IDs, network/API schema, and other irreversible formats.

When a milestone says “must not contradict,” it’s usually pointing at something in `docs/SPECS/`.

Graph hashing specs:
- Canonical commitment: `docs/SPECS/SPEC-WARP-0001-graph-commit-digest.md`
- Incremental acceleration structure: `docs/SPECS/SPEC-WARP-0002-incremental-graph-hash.md`

---

## 4) Code map (“here’s the code for X”)

This project is still mid-milestone, so not every roadmap crate exists yet. This section points you at the *current* implementation locations.

### Core types + determinism primitives

- **Hash, canonical encoding, events, receipts:** `crates/jitos-core/src/lib.rs`
  - Canonical CBOR encoder/decoder/hash helper (SPEC-0001): `crates/jitos-core/src/canonical.rs`
  - Counterfactual delta spec (SPEC-0002): `crates/jitos-core/src/delta.rs`
  - Event record types: `crates/jitos-core/src/events.rs`

### WARP graph (Paper I substrate)

- **WARP graph structs (nodes/edges):** `crates/jitos-graph/src/lib.rs`
- **Deterministic NodeId allocation (SPEC-0005):** `crates/jitos-graph/src/ids.rs`

Note: `WarpGraph::compute_hash()` is currently a placeholder in `crates/jitos-graph/src/lib.rs` — roadmap M1 requires the real deterministic digest law.

### Scheduler (Paper II “Echo” direction)

- **EchoScheduler skeleton + footprints:** `crates/jitos-scheduler/src/lib.rs`

This is currently a stub (sorting/batching logic is not implemented yet).

### Views (Paper IV + time model precursors)

- **Pure, deterministic view folding:** `crates/jitos-views/src/lib.rs`
  - Clock view: `crates/jitos-views/src/clock.rs`
  - Timer view: `crates/jitos-views/src/timer.rs`

### Planner / SLAPS

- **SLAPS intent schema types:** `crates/jitos-planner/src/slaps.rs`

Note: this crate is not fully wired as a Rust target yet (it currently lacks `src/lib.rs`), so `cargo test` for the main workspace may fail until it’s completed or temporarily removed from workspace members.

### Roadmap automation + tooling glue

- Roadmap DAG updater (checkboxes → Mermaid class styling): `scripts/update_roadmap_dags.py`
- Optional Rust wrapper for scripts (standalone workspace): `xtask/src/main.rs`
- Version-controlled git hooks (optional): `.githooks/`

---

## 5) ADRs, RFCs, reports (decision trail + narrative)

### `docs/ADR/`
Architecture Decision Records (small, durable “we chose X over Y” documents).

### `docs/RFC/`
Longer-form proposals / drafts.

### `docs/REPORTS/`
Buildable reports and snapshots of status.

---

## 6) PDF / TeX pipeline

If you see a milestone gate referencing a PDF artifact:
- entrypoint: `docs/tex/build-pdf.sh`
- helper scripts: `docs/tex/scripts/`

---

## 7) “How do I…?”

### Update roadmap DAG statuses
- `python3 scripts/update_roadmap_dags.py`
- or `cargo run --manifest-path xtask/Cargo.toml -- roadmap-dags`

### Enable the ROADMAP auto-updater pre-commit hook (one-time)
- `git config core.hooksPath .githooks`
- or `cargo run --manifest-path xtask/Cargo.toml -- install-githooks`
