# Milestone 7: Typed Domain API v1 + Wesley Mode (1.0 runway)

**Status:** Planned (Approval-Ready)  
**Target Date:** TBD  
**Owner:** James Ross  
**Primary Artifact:** typed GraphQL v1 schema + stable kind registries + generator skeleton (“Wesley mode”)  
**Architecture Anchor:** [ARCH-0001](../../ARCH/ARCH-0001-universal-job-fabric.md) (“Monolith with Seams”)

Milestone 7 stops hiding behind JSON and turns the control plane into a stable, typed domain API, with the beginnings of schema-driven generation.

---

## 1. Executive Summary

Milestone 7 introduces:

- stable NodeKind/EdgeKind registries (canonical list)
- typed domain objects (Task, Slap, Primitive, Policy, etc.)
- a generator pipeline skeleton (“Wesley mode”): SDL → Rust types/validators/registries

**Goal:** deprecate v0 JSON escape hatches with a safe migration path, while keeping determinism and replay guarantees intact.

---

## 2. User Stories

### US-1: Typed Schema
As a client author, I want typed domain objects so tooling doesn’t depend on untyped JSON blobs.

### US-2: Generator-Driven Correctness
As a kernel engineer, I want schema-driven generation so the control plane, validators, and registries can’t drift apart.

---

## 3. Requirements

### Functional
1. **Typed schema:** GraphQL v1 schema with typed domain objects.
2. **Kind enums:** NodeKind/EdgeKind defined and frozen (with versioning strategy).
3. **Generator skeleton:** a crate/tool that generates Rust enums + validators from SDL/kind registry.
4. **Deprecation path:** JSON ops remain supported but deprecated with explicit replacement.

### Non-Functional
1. **Compatibility:** stable migration path from v0 → v1.
2. **Determinism:** typed changes must not introduce nondeterminism in hashing/encoding.

### Constraints / Non-goals (1.0 runway)
- Federation/plugins optional (can come after v1 stabilization).

---

## 4. Determinism Invariants (Hard Law)

### LAW-1: Kind registries are versioned and canonical
Kind lists become part of identity semantics; changes require explicit versioning and compatibility review.

### LAW-2: Generated validators are the reference behavior
Validation logic must be generated or derived from a single canonical schema, not handwritten in multiple places.

---

## 5. Architecture & Design

### 5.1 Wesley pipeline
Start minimal:
- generate Rust enums for NodeKind/EdgeKind
- generate validators for rewrite ops and typed inputs

---

## 6. API surface

Required:
- GraphQL v1 typed schema
- deprecation annotations for v0 JSON ops

---

## 7. Testing Strategy

### Unit Tests
- Golden vectors for kind registry hashing/serialization.
- Generated validator correctness.

### Integration Tests
- v0 client continues to work (deprecated paths).
- v1 client can perform equivalent operations without JSON.

---

## 8. Deliverables
1. Typed GraphQL v1 schema.
2. Stable kind registries.
3. Generator skeleton (Wesley mode).
4. Deprecation/migration documentation.

---

## 9. Definition of Done (Milestone Gate)

Milestone 7 is **DONE** when:

- v1 typed schema exists and is stable
- generator can produce core enums/validators
- v0→v1 migration path is documented and tested

---

## 10. Task Checklist (Inline)

### Phase 0 — Freeze kind registries
- [ ] Define NodeKind/EdgeKind canonical list and versioning policy
- [ ] Decide deprecation policy for JSON ops

### Phase 1 — Schema and types
- [ ] Author GraphQL v1 schema
- [ ] Introduce typed Task/Slap/Primitive/Policy objects

### Phase 2 — Generator skeleton
- [ ] Create generator crate
- [ ] Generate Rust enums + validators from schema

### Phase 3 — Migration and tests
- [ ] Deprecate v0 JSON surfaces with clear replacements
- [ ] Add integration tests for v0 compatibility + v1 equivalence

---

## 11. Sequenced Task DAG (Dependencies)

This DAG is the execution ordering for Milestone 7. It is phase-based so status can be derived from checkboxes.

```mermaid
flowchart TD
  %% Status is auto-updated by scripts/update_roadmap_dags.py
  P0[Phase 0: Freeze kind registries<br/>versioning + deprecation policy] --> P1[Phase 1: Schema and types<br/>GraphQL v1 typed API]
  P1 --> P2[Phase 2: Generator skeleton<br/>enums + validators]
  P2 --> P3[Phase 3: Migration and tests<br/>v0 compat + v1 equivalence]
  P3 --> Gate[Milestone Gate (DoD)]

  classDef done fill:#dcfce7,stroke:#166534,color:#052e16,stroke-width:2px;
  classDef inprogress fill:#dbeafe,stroke:#1d4ed8,color:#1e3a8a,stroke-width:2px;
  classDef blocked fill:#fee2e2,stroke:#b91c1c,color:#7f1d1d,stroke-width:2px;

  class P0 blocked;
  class P1 blocked;
  class P2 blocked;
  class P3 blocked;
  class Gate blocked;
```

---

## 12. Explicit Non-Goals
- federation/plugins (unless explicitly pulled forward)
