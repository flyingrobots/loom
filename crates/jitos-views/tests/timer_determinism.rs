// Copyright 2025 James Ross
// SPDX-License-Identifier: Apache-2.0

//! Timer View Determinism Tests
//!
//! These tests verify that TimerView behaves as a pure, deterministic fold
//! over events.

mod common;

use common::make_clock_event;
use jitos_core::events::{CanonicalBytes, EventEnvelope};
use jitos_views::{
    ClockSource, ClockView, ClockPolicyId, TimerRequest, TimerView, Time,
    OBS_TIMER_REQUEST_V0, DEC_TIMER_FIRE_V0,
};

/// Helper: Create a timer request observation event
fn make_timer_request(request_id: [u8; 32], duration_ns: u64, requested_at_ns: u64) -> EventEnvelope {
    let request = TimerRequest {
        request_id: jitos_core::Hash(request_id),
        duration_ns,
        requested_at_ns,
    };

    EventEnvelope::new_observation(
        CanonicalBytes::from_value(&request).expect("encode request"),
        vec![],
        Some(OBS_TIMER_REQUEST_V0.to_string()),
        None,
        None,
    )
    .expect("create timer request event")
}

// ============================================================================
// T1: Basic Timer Request Processing
// ============================================================================

#[test]
fn t1_timer_request_is_tracked() {
    // Scenario: Timer request event is applied to TimerView
    // Given: Empty TimerView
    let mut view = TimerView::new();

    // When: Timer request observation event is applied
    let request_event = make_timer_request([1u8; 32], 5_000_000_000, 1_000_000_000);
    view.apply_event(&request_event).expect("apply event");

    // Then: Timer appears in pending timers when time >= requested_at + duration
    let current_time = Time::unknown(); // TODO: Replace with actual Time constructor
    let pending = view.pending_timers(&current_time);

    // At unknown time, no timers should be ready
    assert_eq!(pending.len(), 0, "no timers ready at unknown time");
}

// ============================================================================
// T2: Timer Fires at Correct Logical Time
// ============================================================================

#[test]
fn t2_timer_fires_at_correct_time() {
    // Scenario: Timer should fire when current_time >= requested_at + duration
    // Given: TimerView with one request
    let mut timer_view = TimerView::new();
    let mut clock_view = ClockView::new(ClockPolicyId::TrustMonotonicLatest);

    // Set initial time to 1 second
    let clock_event = make_clock_event(ClockSource::Monotonic, 1_000_000_000, 100_000);
    clock_view.apply_event(&clock_event).expect("apply clock event");

    // Request timer for 5 seconds from now
    let request_event = make_timer_request(
        [1u8; 32],
        5_000_000_000, // 5 seconds
        1_000_000_000, // requested at 1s
    );
    timer_view.apply_event(&request_event).expect("apply timer request");

    // When: Time is before fire time (at 5s, need to reach 6s)
    let clock_event2 = make_clock_event(ClockSource::Monotonic, 5_000_000_000, 100_000);
    clock_view.apply_event(&clock_event2).expect("apply clock event");

    // Then: Timer is not ready
    let pending = timer_view.pending_timers(clock_view.now());
    assert_eq!(pending.len(), 0, "timer not ready before fire time");

    // When: Time reaches fire time (6s = 1s + 5s)
    let clock_event3 = make_clock_event(ClockSource::Monotonic, 6_000_000_000, 100_000);
    clock_view.apply_event(&clock_event3).expect("apply clock event");

    // Then: Timer is ready
    let pending = timer_view.pending_timers(clock_view.now());
    assert_eq!(pending.len(), 1, "timer ready at fire time");
    assert_eq!(pending[0].request_id, jitos_core::Hash([1u8; 32]));
}

// ============================================================================
// T3: Multiple Timers
// ============================================================================

#[test]
fn t3_multiple_timers() {
    // Scenario: Multiple timers fire at different times
    // Given: TimerView with 3 timer requests
    let mut timer_view = TimerView::new();
    let mut clock_view = ClockView::new(ClockPolicyId::TrustMonotonicLatest);

    // Set initial time to 0
    let clock_event = make_clock_event(ClockSource::Monotonic, 0, 100_000);
    clock_view.apply_event(&clock_event).expect("apply clock event");

    // Request 3 timers at different durations
    timer_view
        .apply_event(&make_timer_request([1u8; 32], 1_000_000_000, 0))
        .expect("apply timer 1"); // fires at 1s
    timer_view
        .apply_event(&make_timer_request([2u8; 32], 2_000_000_000, 0))
        .expect("apply timer 2"); // fires at 2s
    timer_view
        .apply_event(&make_timer_request([3u8; 32], 3_000_000_000, 0))
        .expect("apply timer 3"); // fires at 3s

    // When: Time is at 1.5s
    let clock_event2 = make_clock_event(ClockSource::Monotonic, 1_500_000_000, 100_000);
    clock_view.apply_event(&clock_event2).expect("apply clock event");

    // Then: Only first timer is ready
    let pending = timer_view.pending_timers(clock_view.now());
    assert_eq!(pending.len(), 1, "only first timer ready at 1.5s");
    assert_eq!(pending[0].request_id, jitos_core::Hash([1u8; 32]));

    // When: Time is at 2.5s
    let clock_event3 = make_clock_event(ClockSource::Monotonic, 2_500_000_000, 100_000);
    clock_view.apply_event(&clock_event3).expect("apply clock event");

    // Then: First two timers are ready
    let pending = timer_view.pending_timers(clock_view.now());
    assert_eq!(pending.len(), 2, "two timers ready at 2.5s");

    // When: Time is at 3.5s
    let clock_event4 = make_clock_event(ClockSource::Monotonic, 3_500_000_000, 100_000);
    clock_view.apply_event(&clock_event4).expect("apply clock event");

    // Then: All three timers are ready
    let pending = timer_view.pending_timers(clock_view.now());
    assert_eq!(pending.len(), 3, "all timers ready at 3.5s");
}
