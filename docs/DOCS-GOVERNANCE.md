# Docs Governance (Source of Truth + How to Lock Decisions)

This document exists because the project has *multiple* doc streams (THEORY, ARCH, ADR, SPECS, ROADMAP, RFCs), and without an explicit hierarchy you get “spec fights” and drift.

The goal is to make it obvious:
- which docs are **normative** (you implement them),
- which docs are **directional** (they describe intent/shape), and
- how a forward-looking idea becomes a locked contract.

---

## 1) Document types (what each is for)

### `docs/THEORY.md` — Foundations (axioms)
- Purpose: the non-negotiable conceptual substrate (WARPs, determinism, provenance, time model, sovereignty).
- Nature: **axiomatic** (not a wire format), but treated as “design law.”

### `docs/ARCH/` — Architecture anchors + invariants (direction + constraints)
- Purpose: stable boundaries and invariants (“Monolith with Seams”, “History is the system”, etc.).
- Nature: mostly **directional**, except explicit invariant docs (like ARCH-0002) which are constraints.

### `docs/ADR/` — Decisions (why we chose X over Y)
- Purpose: record a decision and its consequences.
- Nature: **binding at the decision level**, but typically delegates to SPECS/ROADMAP for implementable detail.

### `docs/SPECS/` — Implementation contracts (normative)
- Purpose: things that must be identical across machines/runs/languages: encodings, IDs, hashes, schemas, error codes.
- Nature: **normative**. If you can write a failing test from it, it belongs here.

### `docs/ROADMAP/` — Milestones (execution contracts)
- Purpose: “what ships” per milestone: invariants, scope, DoD gate, tasks.
- Nature: **normative for the milestone gate**, but it must not contradict ARCH invariants or SPECS.

### `docs/RFC/` — Proposals (ideas in motion)
- Purpose: brainstorming, exploration, large design drafts.
- Nature: **non-binding** until promoted into ADR/SPEC/ROADMAP.

### `docs/REPORTS/` — Snapshots (what happened)
- Purpose: generated or curated reports (status, milestone reports).
- Nature: informational.

---

## 2) Source-of-truth precedence (when docs disagree)

When two documents conflict, resolve in this order:

1. **THEORY** (`docs/THEORY.md`) and explicit invariants (`docs/ARCH/ARCH-0002-architectural-invariants.md`)
2. **SPECS** (`docs/SPECS/*`)
3. **ROADMAP** milestone gates (`docs/ROADMAP/M*/README.md`)
4. **ADRs** (`docs/ADR/*`)
5. **ARCH** (directional anchors, except invariants)
6. **RFCs / reports / other notes**

Rule of thumb:
- If it affects determinism, replay, hashing, encoding, identity, or API compatibility → it belongs in **SPECS**.
- If it’s “what we are shipping next / what counts as done” → it belongs in **ROADMAP**.
- If it’s “why we chose this path” → it belongs in **ADR**.

---

## 3) How to “lock the direction” without drifting

Forward-looking “we should build it this way” claims are useful, but they must have a promotion path:

### Step A — Directional overview (allowed to be forward-looking)
- Put the coherent narrative in an ARCH overview doc (directional).
- It MUST clearly state it is directional and MUST link to normative docs where they exist.

### Step B — Decision capture (ADR)
- When the project chooses between alternatives, write an ADR:
  - Decision
  - Alternatives considered
  - Consequences
  - Links to any affected SPECS/ROADMAP items

### Step C — Contract freeze (SPEC)
- When a decision becomes an irreversible contract (encoding/hash/schema), write a SPEC:
  - deterministic definitions
  - test requirements / golden vectors

### Step D — Execution gate (ROADMAP milestone)
- A milestone README locks what ships and what is deferred.
- The milestone MUST link to all relevant SPECS and MUST NOT contradict them.

This keeps the big picture “locked” while preventing the overview doc from becoming a second source of truth.

---

## 4) Status language (recommended)

Use these consistently in doc headers:

- **Draft:** incomplete / may change
- **Proposed:** a candidate direction/decision not yet accepted
- **Approval-Ready:** wording is stable enough to approve as a contract for implementation
- **Accepted:** chosen and should be followed
- **Deprecated:** kept for history; do not implement from it

