# Docs Index (MOC)

This is the “map of content” for JITOS docs.

If you feel lost, start here, then jump to `docs/TOUR.md` for a longer guided map.

Also see: `docs/DOCS-GOVERNANCE.md` for what is normative vs directional.

---

## Quick start (read order)

1. Foundations: `docs/THEORY.md`
2. Architectural constraints: `docs/ARCH/ARCH-0002-architectural-invariants.md`
3. Implementation contracts: `docs/SPECS/`
4. What we ship next: `docs/ROADMAP/README.md` and `docs/ROADMAP/M1-Kernel-Alpha/README.md`

---

## Milestones

- Milestone roadmap MOC: `docs/ROADMAP/README.md`
- Milestone 1 (Kernel Genesis): `docs/ROADMAP/M1-Kernel-Alpha/README.md`
- Milestone 2 (Reality Layer): `docs/ROADMAP/M2-Kernel-Reality-Layer/README.md`
- Milestone 3 (Collapse & Commit): `docs/ROADMAP/M3-Collapse-Commit/README.md`
- Milestone 4 (Persistence & Replay): `docs/ROADMAP/M4-Persistence-Replay/README.md`
- Milestone 5 (Time & Scheduling): `docs/ROADMAP/M5-Time-Scheduling/README.md`
- Milestone 6 (Tasks/Slaps/Workers): `docs/ROADMAP/M6-Tasks-Slaps-Workers/README.md`
- Milestone 7 (Typed API + Wesley): `docs/ROADMAP/M7-Typed-Domain-API-v1/README.md`

---

## Specs (contracts)

Core determinism and format contracts:

- Canonical encoding: `docs/SPECS/SPEC-0001-canonical-encoding.md`
- DeltaSpec (counterfactuals): `docs/SPECS/SPEC-0002-deltaspec.md`
- Clock view: `docs/SPECS/SPEC-0003-clock-view.md`
- Timer semantics: `docs/SPECS/SPEC-0004-timer-semantics.md`
- Deterministic IDs: `docs/SPECS/SPEC-0005-deterministic-ids.md`
- GraphQL SDL v0 (M1 subset is normative): `docs/SPECS/SPEC-NET-0001-graphql-sdl-v0.md`

Graph hashing:
- Canonical commitment: `docs/SPECS/SPEC-WARP-0001-graph-commit-digest.md`
- Incremental acceleration: `docs/SPECS/SPEC-WARP-0002-incremental-graph-hash.md`

---

## Architecture

Start here:
- ARCH-0001: `docs/ARCH/ARCH-0001-universal-job-fabric.md`
- ARCH-0002: `docs/ARCH/ARCH-0002-architectural-invariants.md`

Optional “big picture”:
- ARCH-0000 ToC: `docs/ARCH/ARCH-0000-ToC.md`
- ARCH-0000 intro: `docs/ARCH/ARCH-0000-intro.md`

---

## ADRs (decisions)

ADRs are the “why” trail:
- list directory: `docs/ADR/`

If you need a starting point, begin with:
- `docs/ADR/ADR-0001.md` (kernel framing)
- `docs/ADR/ADR-0008.md` (time/collapse scheduling/Echo integration)

---

## RFCs (proposals / drafts)

- list directory: `docs/RFC/`

---

## Reports / procedures / TeX pipeline

- reports: `docs/REPORTS/`
- procedures: `docs/procedures/`
- PDF build: `docs/tex/build-pdf.sh`

