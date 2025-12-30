# Milestone 6: TASKS / SLAPS / Workers (Beta-4)

**Status:** Planned (Approval-Ready)  
**Target Date:** TBD  
**Owner:** James Ross  
**Primary Artifact:** real intent submission + task state machine + worker invocations + receipts/events  
**Architecture Anchor:** [ARCH-0001](../../ARCH/ARCH-0001-universal-job-fabric.md) (“Monolith with Seams”)

Milestone 6 makes JITOS do actual work: intent submission becomes real, tasks have a deterministic lifecycle, and workers can be invoked with auditable receipts.

---

## 1. Executive Summary

Milestone 6 activates the “job fabric” described in ARCH-0001:

- `submitIntent` becomes real (even if minimal)
- task lifecycle exists and is observable (`taskEvents`)
- worker invocation/result events exist (even if workers are in-process)

---

## 2. User Stories

### US-1: Submit Intent
As a tool author, I want to submit an intent and receive a task/process/SWS context so the kernel can plan/execute in a shadow world.

### US-2: Observable Task Lifecycle
As an operator, I want deterministic task state transitions and a live event stream for debugging and audit.

### US-3: Worker Receipts
As a kernel engineer, I want worker invocations/results recorded as deterministic events with receipts so replay and audit remain possible.

---

## 3. Requirements

### Functional
1. **submitIntent:** returns `taskId`, `processId`, `swsId` and creates corresponding graph objects.
2. **Task state machine:** `queued → running → done/failed` with deterministic transitions.
3. **Worker interface:** minimal worker trait/interface + demo worker (in-process).
4. **Events:** `TASK_STATE`, `WORKER_INVOCATION`, `WORKER_RESULT` events exist and are queryable/subscribable.
5. **Subscriptions:** `taskEvents(processId, taskId)` works.

### Non-Functional
1. **Determinism:** given same inputs and policy pins, task execution yields identical event history (within defined constraints).
2. **Audit:** every worker invocation has a receipt and is linked to the task/process context.

### Constraints / Non-goals (Beta-4)
- No distributed/remote workers yet.

---

## 4. Determinism Invariants (Hard Law)

### LAW-1: Task state is derived from events
Task “status” is not mutable cosplay; it is derived from the event/rewrite history.

### LAW-2: Worker boundary is explicit
Any non-deterministic I/O must cross explicit adapters/ports and be recordable/replayable.

---

## 5. Architecture & Design

### 5.1 Task objects
Define minimal types/IDs and how they map into the graph.

### 5.2 Worker capability boundary
Even if localhost-only, define capability checks for invoking workers (foundation for auth later).

---

## 6. API surface

Required:
- `submitIntent(input)` implemented
- `taskEvents(processId, taskId)` subscription implemented

Optional:
- typed intent schema (still JSON in v0 is acceptable)

---

## 7. Testing Strategy

### Unit Tests
- Task state transitions deterministic given same events.
- Worker invocation produces expected event sequence.

### Integration Tests
- Submit intent, observe task events, verify deterministic receipts.
- Replay test across restart (post-M4) yields identical task event history for the same script.

---

## 8. Deliverables
1. Real `submitIntent` path.
2. Task state machine + events/subscriptions.
3. Minimal worker interface + demo worker.
4. Determinism and replay tests.

---

## 9. Definition of Done (Milestone Gate)

Milestone 6 is **DONE** when:

- `submitIntent` creates task/process/SWS context and returns IDs
- task lifecycle produces deterministic `TASK_STATE` events
- worker invocation/result events are recorded with receipts
- taskEvents subscription works end-to-end

---

## 10. Task Checklist (Inline)

### Phase 0 — Freeze task model
- [ ] Define task/process identifiers and their encoding
- [ ] Define state machine and event types
- [ ] Define worker interface and receipt fields

### Phase 1 — Kernel implementation
- [ ] Implement submitIntent and task/process creation
- [ ] Implement task lifecycle transitions
- [ ] Implement worker invocation pipeline (in-process)

### Phase 2 — API wiring
- [ ] GraphQL resolvers for submitIntent and taskEvents subscription
- [ ] Error codes and deterministic output ordering

### Phase 3 — Tests
- [ ] Unit tests for state machine + worker events
- [ ] Integration tests for end-to-end task run + replay

---

## 11. Explicit Non-Goals
- distributed worker pools
- remote execution
