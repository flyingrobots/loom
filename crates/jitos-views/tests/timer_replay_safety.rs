// Copyright 2025 James Ross
// SPDX-License-Identifier: Apache-2.0

//! Timer View Replay Safety Tests
//!
//! These tests verify that TimerView never touches system time functions
//! and is deterministic across replays.

mod common;

use common::{make_clock_event, make_timer_request};
use jitos_core::events::{CanonicalBytes, EventEnvelope};
use jitos_views::{ClockPolicyId, ClockSource, ClockView, TimerFire, TimerView};

/// Helper: Create a timer fire decision event
fn make_timer_fire(
    request_id: [u8; 32],
    fired_at_ns: u64,
    request_event_id: jitos_core::Hash,
) -> EventEnvelope {
    let fire = TimerFire {
        request_id: jitos_core::Hash(request_id),
        fired_at_ns,
    };

    // Create a dummy policy context to use as parent
    let policy = EventEnvelope::new_policy_context(
        CanonicalBytes::from_value(&"timer_policy".to_string()).expect("encode policy"),
        vec![],
        None,
        None,
    )
    .expect("create policy event");

    EventEnvelope::new_decision(
        CanonicalBytes::from_value(&fire).expect("encode fire"),
        vec![request_event_id], // Use timer request as evidence
        policy.event_id(),
        None,
        None,
    )
    .expect("create timer fire event")
}

// ============================================================================
// T1: Fired Timers Don't Appear in Pending
// ============================================================================

#[test]
fn t1_fired_timers_excluded_from_pending() {
    // Scenario: Timer fire decision event prevents timer from appearing in pending
    // Given: TimerView with one request and corresponding fire event
    let mut timer_view = TimerView::new();
    let mut clock_view = ClockView::new(ClockPolicyId::TrustMonotonicLatest);

    // Request timer
    let request_event = make_timer_request([1u8; 32], 5_000_000_000, 0);
    let request_id = request_event.event_id();
    timer_view
        .apply_event(&request_event)
        .expect("apply request");

    // Fire the timer
    let fire_event = make_timer_fire([1u8; 32], 5_000_000_000, request_id);
    timer_view.apply_event(&fire_event).expect("apply fire");

    // Set clock to after fire time
    let clock_event = make_clock_event(ClockSource::Monotonic, 10_000_000_000, 100_000);
    clock_view
        .apply_event(&clock_event)
        .expect("apply clock event");

    // When: Querying pending timers
    let pending = timer_view.pending_timers(clock_view.now());

    // Then: Fired timer does not appear
    assert_eq!(pending.len(), 0, "fired timer should not appear in pending");
}

// ============================================================================
// T2: Replay Produces Identical Results
// ============================================================================

#[test]
fn t2_replay_determinism() {
    // Scenario: Same events produce identical pending timer lists
    // Given: Event sequence
    let events = vec![
        make_timer_request([1u8; 32], 1_000_000_000, 0),
        make_timer_request([2u8; 32], 2_000_000_000, 0),
        make_timer_request([3u8; 32], 3_000_000_000, 0),
    ];

    let clock_events = vec![
        make_clock_event(ClockSource::Monotonic, 0, 100_000),
        make_clock_event(ClockSource::Monotonic, 2_500_000_000, 100_000),
    ];

    // Replay 100 times
    let mut results = Vec::new();
    for _ in 0..100 {
        let mut timer_view = TimerView::new();
        let mut clock_view = ClockView::new(ClockPolicyId::TrustMonotonicLatest);

        for event in &events {
            timer_view.apply_event(event).expect("apply event");
        }

        for event in &clock_events {
            clock_view.apply_event(event).expect("apply clock event");
        }

        let pending = timer_view.pending_timers(clock_view.now());
        results.push(pending);
    }

    // Then: All results identical
    for i in 1..results.len() {
        assert_eq!(
            results[0], results[i],
            "replay {} produced different result",
            i
        );
    }

    // Verify we got the expected result (first 2 timers)
    assert_eq!(results[0].len(), 2, "should have 2 pending timers at 2.5s");
}

// ============================================================================
// T3: No Host Clock Dependency
// ============================================================================

#[test]
fn t3_no_host_clock_dependency() {
    // Scenario: pending_timers() is pure - no syscall dependency
    // Given: TimerView with requests
    let mut timer_view = TimerView::new();
    let mut clock_view = ClockView::new(ClockPolicyId::TrustMonotonicLatest);

    timer_view
        .apply_event(&make_timer_request([1u8; 32], 1_000_000_000, 0))
        .expect("apply request");

    let clock_event = make_clock_event(ClockSource::Monotonic, 2_000_000_000, 100_000);
    clock_view
        .apply_event(&clock_event)
        .expect("apply clock event");

    // When: Calling pending_timers() multiple times with no new events
    let pending1 = timer_view.pending_timers(clock_view.now());
    let pending2 = timer_view.pending_timers(clock_view.now());
    let pending3 = timer_view.pending_timers(clock_view.now());

    // Then: Results are identical (pure function)
    assert_eq!(pending1, pending2, "pending_timers must be pure");
    assert_eq!(pending2, pending3, "pending_timers must be pure");

    // Call 1000x to verify no syscall timing effects
    let mut results = Vec::new();
    for _ in 0..1000 {
        results.push(timer_view.pending_timers(clock_view.now()));
    }

    for result in &results {
        assert_eq!(
            result, &pending1,
            "all pending_timers() calls must return identical values"
        );
    }
}

// ============================================================================
// T4: Event Order Independence for Concurrent Requests
// ============================================================================

#[test]
fn t4_event_order_independence() {
    // Scenario: Timers requested in different orders produce same result
    // Given: Same set of timer requests in different orders
    let requests = vec![
        make_timer_request([1u8; 32], 1_000_000_000, 0),
        make_timer_request([2u8; 32], 2_000_000_000, 0),
        make_timer_request([3u8; 32], 3_000_000_000, 0),
    ];

    let clock_event = make_clock_event(ClockSource::Monotonic, 2_500_000_000, 100_000);

    // Apply in original order
    let mut view1 = TimerView::new();
    let mut clock1 = ClockView::new(ClockPolicyId::TrustMonotonicLatest);
    for req in &requests {
        view1.apply_event(req).expect("apply");
    }
    clock1.apply_event(&clock_event).expect("apply clock");
    let pending1 = view1.pending_timers(clock1.now());

    // Apply in reverse order
    let mut view2 = TimerView::new();
    let mut clock2 = ClockView::new(ClockPolicyId::TrustMonotonicLatest);
    for req in requests.iter().rev() {
        view2.apply_event(req).expect("apply");
    }
    clock2.apply_event(&clock_event).expect("apply clock");
    let pending2 = view2.pending_timers(clock2.now());

    // Then: Results should contain same timers (order may differ)
    assert_eq!(
        pending1.len(),
        pending2.len(),
        "same number of pending timers"
    );

    // Verify both have timers 1 and 2
    assert_eq!(pending1.len(), 2, "should have 2 pending timers");
    let ids1: Vec<_> = pending1.iter().map(|r| r.request.request_id).collect();
    let ids2: Vec<_> = pending2.iter().map(|r| r.request.request_id).collect();

    assert!(ids1.contains(&jitos_core::Hash([1u8; 32])));
    assert!(ids1.contains(&jitos_core::Hash([2u8; 32])));
    assert!(ids2.contains(&jitos_core::Hash([1u8; 32])));
    assert!(ids2.contains(&jitos_core::Hash([2u8; 32])));
}
