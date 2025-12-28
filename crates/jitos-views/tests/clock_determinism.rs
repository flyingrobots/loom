// Copyright 2025 James Ross
// SPDX-License-Identifier: Apache-2.0

//! Clock View Determinism Tests (AC2, AC3)
//!
//! These tests verify that ClockView is a pure, deterministic function over
//! event sequences - no syscalls, identical results on replay.

use jitos_core::events::{CanonicalBytes, EventEnvelope};
use jitos_views::{ClockPolicyId, ClockSample, ClockSource, ClockView};

/// Helper: Create a clock sample observation event
fn make_clock_event(source: ClockSource, value_ns: u64, uncertainty_ns: u64) -> EventEnvelope {
    let sample = ClockSample {
        source,
        value_ns,
        uncertainty_ns,
    };

    EventEnvelope::new_observation(
        CanonicalBytes::from_value(&sample).expect("encode sample"),
        vec![],  // no parents for test
        None,    // agent_id
        None,    // signature
    )
    .expect("create observation event")
}

// ============================================================================
// T1: Pure Query Behavior (AC2)
// ============================================================================

#[test]
fn t1_now_is_deterministic() {
    // Scenario: now() is deterministic
    // Given: ClockView with 5 monotonic samples applied
    let mut view = ClockView::new(ClockPolicyId::TrustMonotonicLatest);

    for i in 0..5 {
        let event = make_clock_event(
            ClockSource::Monotonic,
            1_000_000_000 + i * 1_000_000,  // +1ms each
            100_000,  // ±100μs
        );
        view.apply_event(&event).expect("apply event");
    }

    // When: Call now() 1000 times
    let mut results = Vec::new();
    for _ in 0..1000 {
        results.push(view.now().clone());
    }

    // Then: All 1000 calls return identical Time value
    let first = &results[0];
    for result in &results {
        assert_eq!(result.ns, first.ns, "ns must be identical");
        assert_eq!(result.uncertainty_ns, first.uncertainty_ns, "uncertainty must be identical");
        assert_eq!(result.domain, first.domain, "domain must be identical");
        assert_eq!(result.provenance, first.provenance, "provenance must be identical");
    }
}

// ============================================================================
// T2: Replay Determinism (AC3)
// ============================================================================

#[test]
fn t2_replay_produces_identical_time() {
    // Scenario: Identical event sequences produce identical time
    // Given: Event sequence with 10 clock sample events
    let events: Vec<EventEnvelope> = (0..10)
        .map(|i| {
            make_clock_event(
                ClockSource::Monotonic,
                1_000_000_000 + i * 10_000_000,  // +10ms each
                50_000,  // ±50μs
            )
        })
        .collect();

    // When: Replay sequence 100 times in separate ClockView instances
    let mut results = Vec::new();
    for _ in 0..100 {
        let mut view = ClockView::new(ClockPolicyId::TrustMonotonicLatest);
        for event in &events {
            view.apply_event(event).expect("apply event");
        }
        results.push(view.now().clone());
    }

    // Then: All 100 instances report identical now() values
    let first = &results[0];
    for (i, result) in results.iter().enumerate() {
        assert_eq!(
            result, first,
            "iteration {} produced different time: {:?} vs {:?}",
            i, result, first
        );
    }
}

// ============================================================================
// T5: Historical Queries (AC2)
// ============================================================================

#[test]
fn t5_now_at_cut_queries_historical_time() {
    // Scenario: now_at_cut() allows querying time at any worldline position
    // Given: Event sequence of length 20
    let events: Vec<EventEnvelope> = (0..20)
        .map(|i| {
            make_clock_event(
                ClockSource::Monotonic,
                1_000_000_000 + i * 5_000_000,  // +5ms each
                100_000,  // ±100μs
            )
        })
        .collect();

    // When: Query now_at_cut(events, 10, policy)
    let time_at_10 = ClockView::now_at_cut(
        &events,
        10,
        ClockPolicyId::TrustMonotonicLatest,
    )
    .expect("now_at_cut succeeded");

    // Then: Returns time belief as-of event 10 (not current cut)
    // At cut=10, latest monotonic sample is event[9] with value 1_000_000_000 + 9*5_000_000
    assert_eq!(time_at_10.ns, 1_000_000_000 + 9 * 5_000_000);

    // Also verify cut=20 gives different result (latest sample)
    let time_at_20 = ClockView::now_at_cut(
        &events,
        20,
        ClockPolicyId::TrustMonotonicLatest,
    )
    .expect("now_at_cut succeeded");

    assert_eq!(time_at_20.ns, 1_000_000_000 + 19 * 5_000_000);
    assert_ne!(time_at_10.ns, time_at_20.ns, "different cuts should yield different times");
}

// ============================================================================
// T7: Unknown State Initialization (AC1)
// ============================================================================

#[test]
fn t7_unknown_state_initialization() {
    // Scenario: ClockView starts with sensible defaults
    // Given: ClockView::new(policy)
    let view = ClockView::new(ClockPolicyId::TrustMonotonicLatest);

    // When: Call now() before any events applied
    let time = view.now();

    // Then: Returns Time::unknown() with uncertainty_ns = u64::MAX and empty provenance
    assert_eq!(time.ns, 0, "unknown time has ns=0");
    assert_eq!(time.uncertainty_ns, u64::MAX, "unknown time has max uncertainty");
    assert_eq!(time.provenance.len(), 0, "unknown time has empty provenance");
    assert_eq!(time.domain, jitos_views::TimeDomain::Unknown, "unknown time has Unknown domain");
}
