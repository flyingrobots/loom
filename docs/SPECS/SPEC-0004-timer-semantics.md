# SPEC-0004: Timer Semantics - Deterministic Timers as Materialized View

**Status:** Approved (v1.0)
**Related:** NEXT-MOVES.md Phase 0.5.5, SPEC-0003 Clock View
**Estimated Effort:** 2-3 hours

---

## Problem Statement

Timers in traditional systems use wall-clock time: `sleep(duration)` blocks the thread for `duration` seconds of real time. This creates nondeterminism - replay executes at different speeds, causing timers to fire at different points in the event sequence.

**Without this:** Timers fire based on wall-clock, breaking determinism and making replay unreliable.
**With this:** Timers become requests in the event log, firing when logical time (from ClockView) reaches the target - deterministic and replay-safe.

---

## User Story

**As a** JITOS scheduler developer
**I want** timers represented as logical time requests
**So that** `sleep(5s)` fires at the same logical time on every replay

---

## Requirements

### Functional Requirements

#### Core Principle: Timers are Logical, Not Real-Time

**Timers in JITOS are events + queries, not wall-clock waits.**

- **No hidden timers:** No `std::thread::sleep()`, `tokio::time::sleep()`, or OS timer APIs
- **Deterministic:** Same events + same clock policy → timers fire at identical logical times
- **Replay-safe:** Replaying event log fires timers at same logical positions
- **Event-based:** Timer requests are Observation events, timer fires are Decision events

#### TimerView Type

TimerView is a deterministic materialized view over timer request and fire events.

- `requests: Vec<TimerRequestRecord>` – all timer requests from Observation events
- `fired: Vec<TimerFireRecord>` – all timer fire events from Decision events

Where:

- `TimerRequestRecord`:
  - `event_id: Hash`
  - `request: TimerRequest`

- `TimerFireRecord`:
  - `event_id: Hash`
  - `fire: TimerFire`

#### TimerRequest Type

Timer request from an observation event.

- `request_id: Hash` - unique identifier for this timer request
- `duration_ns: u64` - duration in nanoseconds
- `requested_at_ns: u64` - logical time when request was made (from ClockView)

**Key invariant:** `requested_at_ns` MUST be captured from `ClockView.now().ns()` at the time of the request, not from any syscall.

#### TimerFire Type

Timer fire from a decision event.

- `request_id: Hash` - which timer request this corresponds to
- `fired_at_ns: u64` - logical time when timer fired (from ClockView)

#### Event Flow

**Timer Request (Observation):**
1. Application wants to wait for duration D
2. Read current logical time T from ClockView
3. Create timer request: `request_id = H(request), duration_ns = D, requested_at_ns = T`
4. Emit Observation event with payload = TimerRequest
5. Continue execution (non-blocking)

**Timer Check (Query):**
1. Scheduler calls `timer_view.pending_timers(clock_view.now())`
2. TimerView returns all timers where:
   - `current_time.ns >= requested_at_ns + duration_ns`
   - AND timer has not already fired (not in `fired` list)

**Timer Fire (Decision):**
1. Scheduler receives pending timer from TimerView
2. Creates Decision event with payload = TimerFire
3. TimerView processes Decision event and adds to `fired` list
4. Timer no longer appears in `pending_timers()` query results

#### Observation Type Tags

Timer request observations MUST be tagged:
- `OBS_TIMER_REQUEST_V0: &str = "OBS_TIMER_REQUEST_V0"`

Timer fire decisions currently use payload-based decoding (no type tag yet).

### Non-Functional Requirements

#### Performance

- O(1) per timer request application
- O(N) query for pending timers where N = number of unfired requests
- Memory: O(M) where M = total requests + fires

#### Determinism

- Same event sequence → identical pending timer lists at any point
- Replay fires timers at exact same logical times

#### Purity

- `pending_timers()` is a pure query - no side effects
- No syscalls in TimerView

---

## API

### TimerView

```rust
pub struct TimerView {
    requests: Vec<TimerRequestRecord>,
    fired: Vec<TimerFireRecord>,
}

impl TimerView {
    /// Create new timer view
    pub fn new() -> Self;

    /// Apply one event in canonical worldline order
    pub fn apply_event(&mut self, event: &EventEnvelope) -> Result<(), TimerError>;

    /// Get timers that should fire at current_time but haven't yet
    pub fn pending_timers(&self, current_time: &Time) -> Vec<TimerRequest>;
}
```

### Usage Example

```rust
use jitos_views::{ClockView, TimerView, ClockPolicyId, TimerRequest};

// Initialize views
let mut clock_view = ClockView::new(ClockPolicyId::TrustMonotonicLatest);
let mut timer_view = TimerView::new();

// Process events
for event in events {
    clock_view.apply_event(&event)?;
    timer_view.apply_event(&event)?;
}

// Check which timers should fire at current logical time
let pending = timer_view.pending_timers(clock_view.now());

for timer in pending {
    println!("Timer {} ready to fire", timer.request_id);
    // Scheduler would emit a Decision event here
}
```

---

## Test Plan

### Unit Tests

1. **T1: Timer Request Tracking**
   - Given: TimerView receives timer request observation
   - When: Event is applied
   - Then: Request appears in internal state

2. **T2: Timer Fires at Correct Time**
   - Given: Timer request with duration D at time T
   - When: ClockView reaches time T+D
   - Then: Timer appears in pending_timers() result

3. **T3: Multiple Timers**
   - Given: 3 timer requests with different durations
   - When: ClockView reaches various times
   - Then: Correct subset of timers pending at each time

### Integration Tests (Replay Safety)

4. **T4: Fired Timers Excluded**
   - Given: Timer request + fire decision
   - When: Querying pending_timers() after fire time
   - Then: Fired timer does NOT appear

5. **T5: Replay Determinism**
   - Given: Event sequence with timer requests
   - When: Replaying 100 times
   - Then: Identical pending timer lists at each point

6. **T6: No Host Clock Dependency**
   - Given: TimerView with requests
   - When: Calling pending_timers() 1000x with no new events
   - Then: All results identical (pure function)

7. **T7: Event Order Independence**
   - Given: Same timer requests in different orders
   - When: Reaching same logical time
   - Then: Same set of pending timers (order-independent)

---

## Acceptance Criteria

- [x] TimerView tracks timer requests from Observation events
- [x] `pending_timers()` returns timers ready to fire based on logical time
- [x] Fired timers (Decision events) excluded from pending results
- [x] Replay produces identical timer behavior (7/7 tests passing)
- [x] No syscalls in TimerView implementation
- [x] Pure query semantics: repeated calls with same time → same result

---

## Design Decisions

### 1. Why `requested_at_ns` instead of deriving from event timestamp?

Events don't have timestamps - they're content-addressed. We explicitly store when the request was made (from ClockView) to avoid ambiguity about "when" in logical time.

### 2. Why track fires separately instead of marking requests?

Immutability: events are append-only. Rather than mutating request records, we append fire records and filter during query. This maintains event sourcing principles.

### 3. Why O(N) query instead of priority queue?

Simplicity for Phase 0.5.5. With typically <100 concurrent timers, O(N) scan is fine. Future optimization: min-heap keyed by fire time.

### 4. Why no decision type tags yet?

Clock View established observation type tags. Decision type tags are future work (symmetry with observations). For now, decode-and-check pattern works.

---

## Implementation Notes

### Files

- `crates/jitos-views/src/timer.rs` - TimerView implementation
- `crates/jitos-views/tests/timer_determinism.rs` - Basic timer tests (3 tests)
- `crates/jitos-views/tests/timer_replay_safety.rs` - Replay safety tests (4 tests)

### Observations

- Total tests: 7 (all passing)
- Timer requests use observation type tag `OBS_TIMER_REQUEST_V0`
- Timer fires use Decision events with payload-only decoding
- No external dependencies beyond jitos-core

---

## Future Work

### Phase 2 Enhancements

1. **Priority Queue Optimization**
   - Replace O(N) scan with O(log N) min-heap
   - Benchmark: only worth it if >1000 concurrent timers

2. **Decision Type Tags**
   - Add `decision_type` field to EventEnvelope (symmetric with `observation_type`)
   - Tag timer fires with `DEC_TIMER_FIRE_V0`
   - Enables efficient filtering without payload decode

3. **Timer Cancellation**
   - Add `TimerCancel` decision type
   - Track cancellations separately from fires
   - `pending_timers()` filters both fires and cancels

4. **Timeout Policies**
   - Support alternative firing policies (earliest, latest, etc.)
   - Useful for timeout groups and deadline scheduling

---

## References

- SPEC-0003: Clock View (time as materialized view)
- NEXT-MOVES.md: Phase 0.5.5 requirements
- THEORY.md: Paper II (Deterministic Time Semantics)
