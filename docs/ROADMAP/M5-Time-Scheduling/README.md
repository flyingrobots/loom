# Milestone 5: Time & Scheduling (Beta-3)

**Status:** Planned (Approval-Ready)  
**Target Date:** TBD  
**Owner:** James Ross  
**Primary Artifact:** deterministic time model (Clock View) + tick loop + timer primitive + policy pins in receipts  
**Architecture Anchor:** [ARCH-0001](../../ARCH/ARCH-0001-universal-job-fabric.md) (“Monolith with Seams”)

Milestone 5 introduces deterministic time and the beginnings of Echo-style scheduling: tick events, timer semantics, and policy boundaries that show up in receipts.

---

## 1. Executive Summary

Milestone 5 introduces a deterministic time model consistent with the Loom foundations:

- “now” is a query over events (not wall-clock)
- the kernel emits deterministic `TICK` events in a tick loop
- a minimal timer primitive demonstrates deterministic scheduling

---

## 2. User Stories

### US-1: Deterministic Now
As an operator, I want `now()` to be deterministic and derived from event history so replay produces the same time behavior.

### US-2: Tick Observability
As a viewer developer, I want a `ticks` subscription so the UI can animate progress without polling.

### US-3: Timer Demo
As a kernel engineer, I want a minimal timer that fires deterministically and records its firing in the event log.

---

## 3. Requirements

### Functional
1. **Clock View:** implement deterministic time querying aligned with [SPEC-0003](../../SPECS/SPEC-0003-clock-view.md).
2. **Tick loop:** introduce a tick loop that emits `TICK` events.
3. **Timer primitive:** implement a minimal timer/sleep primitive that schedules a deterministic “fire” event.
4. **Policy pins:** `policyId`/`rulePackId` show up in receipts/events consistently.

### Non-Functional
1. **No wall-clock in semantics:** wall clock is an adapter; core kernel time is derived/deterministic.
2. **Replay stability:** replay produces identical ticks/timer firings given the same event history.

### Constraints / Non-goals (Beta-3)
- full tasks/workers may still be stubbed unless combined intentionally

---

## 4. Determinism Invariants (Hard Law)

### LAW-1: Time is event-derived
“Now” is derived from Chronos order (WAL/event index), not from OS wall-clock.

### LAW-2: Tick loop is deterministic under replay
Tick emission is driven by deterministic inputs and is captured in the event log.

---

## 5. Architecture & Design

### 5.1 Time representation
Decide and lock:
- monotonic logical time (u64)
- mapping from tick index to time unit
- timer scheduling policy (derived from logical time)

### 5.2 Scheduling boundary
Introduce `jitos-sched` as a seam: policy selection must be explicit and visible in receipts.

---

## 6. API surface

Required:
- `ticks` subscription emits deterministic tick events (global)
- `kernelInfo` exposes policy pins

Optional:
- query for clock view (e.g., `clockNow`)

---

## 7. Testing Strategy

### Unit Tests
- Clock view derivation is stable under replay.
- Timer fires at deterministic tick/time.

### Integration Tests
- Start daemon, schedule timer, observe deterministic fire event.
- Restart/replay yields identical tick/timer behavior.

---

## 8. Deliverables
1. Deterministic clock view and tick loop.
2. Working ticks subscription.
3. Timer demo with deterministic firing events.
4. Policy pins in receipts/events.

---

## 9. Definition of Done (Milestone Gate)

Milestone 5 is **DONE** when:

- `now()` is deterministic and not wall-clock based
- ticks stream works and is replay-stable
- timer demo produces deterministic event record

---

## 10. Task Checklist (Inline)

### Phase 0 — Freeze time semantics
- [ ] Decide logical time representation and derivation
- [ ] Document time sampling policy (derived, not claimed)
- [ ] Update SPEC-0003 if needed

### Phase 1 — Tick loop
- [ ] Implement kernel tick loop and event emission
- [ ] Persist tick events in WAL/event log

### Phase 2 — Timer primitive
- [ ] Implement timer scheduling and fire event
- [ ] Add GraphQL/subscription hooks as needed

### Phase 3 — Tests
- [ ] Unit tests for clock view and timers
- [ ] Integration tests for replay stability

---

## 11. Sequenced Task DAG (Dependencies)

This DAG is the execution ordering for Milestone 5. It is phase-based so status can be derived from checkboxes.

```mermaid
flowchart TD
  %% Status is auto-updated by scripts/update_roadmap_dags.py
  P0[Phase 0: Freeze time semantics<br/>representation + policy] --> P1[Phase 1: Tick loop<br/>emit + persist TICK]
  P1 --> P2[Phase 2: Timer primitive<br/>schedule + fire event]
  P2 --> P3[Phase 3: Tests<br/>replay-stable time]
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
- full distributed scheduling
- remote time sources
