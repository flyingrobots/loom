// Copyright 2025 James Ross
// SPDX-License-Identifier: Apache-2.0

//! Clock View Replay Safety Tests (AC5)
//!
//! These tests verify that ClockView NEVER touches system time functions.
//! Time is a pure view over events - no syscalls, no side effects.

use jitos_core::events::{CanonicalBytes, EventEnvelope};
use jitos_views::{ClockError, ClockPolicyId, ClockSample, ClockSource, ClockView};

/// Helper: Create a clock sample observation event
fn make_clock_event(source: ClockSource, value_ns: u64, uncertainty_ns: u64) -> EventEnvelope {
    let sample = ClockSample {
        source,
        value_ns,
        uncertainty_ns,
    };

    EventEnvelope::new_observation(
        CanonicalBytes::from_value(&sample).expect("encode sample"),
        vec![],
        None,  // agent_id
        None,  // signature
    )
    .expect("create observation event")
}

// ============================================================================
// T4: No Host Clock Dependency (AC5)
// ============================================================================

#[test]
fn t4_no_host_clock_dependency() {
    // Scenario: Implementation never touches system clock
    // Given: ClockView implementation
    // When: Code audit performed
    // Then: Zero matches for Instant::, SystemTime::, clock_gettime
    //
    // NOTE: This test is primarily enforced at compile-time via:
    // 1. Code review (no std::time imports in clock.rs)
    // 2. Clippy disallowed-methods (if configured)
    // 3. This runtime test verifies behavioral purity
    //
    // If ClockView were touching syscalls, time would vary across runs.
    // We verify purity by checking that repeated calls with no new events
    // produce identical results (already covered by T1, but we add
    // an explicit behavioral test here).

    let mut view = ClockView::new(ClockPolicyId::TrustMonotonicLatest);

    // Apply one sample
    let event = make_clock_event(ClockSource::Monotonic, 1_000_000_000, 100_000);
    view.apply_event(&event).expect("apply event");

    let time1 = view.now().clone();

    // Sleep would reveal syscall dependency (time would change)
    // But we don't sleep - we just call now() again immediately
    let time2 = view.now().clone();

    // If implementation were pure, these MUST be identical
    assert_eq!(time1, time2, "now() must be pure - no syscall dependency");

    // Additional check: calling now() 1000x with no intervening events
    // should produce identical results
    let mut results = Vec::new();
    for _ in 0..1000 {
        results.push(view.now().clone());
    }

    for result in &results {
        assert_eq!(result, &time1, "all now() calls must return identical values");
    }
}

// ============================================================================
// Cut Bounds Checking
// ============================================================================

#[test]
fn test_now_at_cut_bounds_checking() {
    // Scenario: now_at_cut() validates cut bounds
    // Given: Event sequence of length 5
    let events: Vec<EventEnvelope> = (0..5)
        .map(|i| make_clock_event(ClockSource::Monotonic, 1_000_000_000 + i * 1_000_000, 100_000))
        .collect();

    // When: cut > events.len()
    let result = ClockView::now_at_cut(&events, 10, ClockPolicyId::TrustMonotonicLatest);

    // Then: Returns Err(CutOutOfBounds)
    match result {
        Err(ClockError::CutOutOfBounds { cut, len }) => {
            assert_eq!(cut, 10);
            assert_eq!(len, 5);
        }
        _ => panic!("expected CutOutOfBounds error, got {:?}", result),
    }

    // Verify valid cut succeeds
    let result = ClockView::now_at_cut(&events, 5, ClockPolicyId::TrustMonotonicLatest);
    assert!(result.is_ok(), "cut == len should succeed");

    let result = ClockView::now_at_cut(&events, 3, ClockPolicyId::TrustMonotonicLatest);
    assert!(result.is_ok(), "cut < len should succeed");
}

// ============================================================================
// Observation Type Filtering
// ============================================================================

#[test]
fn test_malformed_observation_handling() {
    // Scenario: apply_event() handles malformed observations gracefully
    // Given: ClockView
    let mut view = ClockView::new(ClockPolicyId::TrustMonotonicLatest);

    // Create an observation event with invalid payload (not a ClockSample)
    let malformed_event = EventEnvelope::new_observation(
        CanonicalBytes::from_value(&vec![1, 2, 3, 4]).expect("encode bytes"),
        vec![],
        None,  // agent_id
        None,  // signature
    )
    .expect("create malformed event");

    // When: apply_event() called on malformed observation
    let result = view.apply_event(&malformed_event);

    // Then: Returns Ok(()) - ignored (decode attempt fails but doesn't error)
    // OR returns Err(DecodingError) depending on implementation choice
    // For Phase 0.5.4, we'll accept either - just verify it doesn't panic
    match result {
        Ok(()) => {
            // Silently ignored - acceptable
            assert_eq!(view.now().domain, jitos_views::TimeDomain::Unknown);
        }
        Err(ClockError::DecodingError) => {
            // Explicit error - also acceptable
        }
        Err(e) => {
            panic!("unexpected error: {:?}", e);
        }
    }
}
