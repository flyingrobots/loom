// Copyright 2025 James Ross
// SPDX-License-Identifier: Apache-2.0

//! Clock View Policy Independence Tests (AC4)
//!
//! These tests verify that different policies produce different time beliefs
//! from the same event sequence - essential for counterfactual branching.

use jitos_core::events::{CanonicalBytes, EventEnvelope};
use jitos_views::{ClockPolicyId, ClockSample, ClockSource, ClockView, TimeDomain};

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
// T3: Policy Independence (AC4)
// ============================================================================

#[test]
fn t3_different_policies_produce_different_beliefs() {
    // Scenario: Different policies produce different beliefs from same observations
    // Given: Event sequence with both Monotonic and Ntp samples
    let events = vec![
        make_clock_event(ClockSource::Monotonic, 1_000_000_000, 100_000),
        make_clock_event(ClockSource::Ntp, 1_735_387_200_000_000_000, 50_000_000),  // 2024-12-28 12:00:00 UTC
        make_clock_event(ClockSource::Monotonic, 2_000_000_000, 100_000),
        make_clock_event(ClockSource::Ntp, 1_735_387_205_000_000_000, 50_000_000),  // +5s
    ];

    // When: Replay with TrustMonotonicLatest
    let mut view_mono = ClockView::new(ClockPolicyId::TrustMonotonicLatest);
    for event in &events {
        view_mono.apply_event(event).expect("apply event");
    }
    let time_mono = view_mono.now();

    // When: Replay with TrustNtpLatest
    let mut view_ntp = ClockView::new(ClockPolicyId::TrustNtpLatest);
    for event in &events {
        view_ntp.apply_event(event).expect("apply event");
    }
    let time_ntp = view_ntp.now();

    // Then: Time beliefs differ (monotonic-only vs ntp-only)
    assert_ne!(
        time_mono.ns, time_ntp.ns,
        "different policies must produce different time values"
    );

    // Verify TrustMonotonicLatest used the latest monotonic sample
    assert_eq!(
        time_mono.ns, 2_000_000_000,
        "TrustMonotonicLatest should use latest monotonic value"
    );

    // Verify TrustNtpLatest used the latest NTP sample
    assert_eq!(
        time_ntp.ns, 1_735_387_205_000_000_000,
        "TrustNtpLatest should use latest NTP value"
    );

    // Then: Domains differ (Monotonic vs Unix)
    assert_eq!(
        time_mono.domain,
        TimeDomain::Monotonic,
        "TrustMonotonicLatest produces Monotonic domain"
    );
    assert_eq!(
        time_ntp.domain,
        TimeDomain::Unix,
        "TrustNtpLatest produces Unix domain"
    );
}

// ============================================================================
// T6: Event Integration (AC1)
// ============================================================================

#[test]
fn t6_event_integration_policy_aware() {
    // Scenario: apply_event() updates current iff sample source is relevant
    // Given: ClockView with TrustMonotonicLatest policy
    let mut view = ClockView::new(ClockPolicyId::TrustMonotonicLatest);

    // Apply monotonic sample (relevant to policy)
    let mono_event = make_clock_event(ClockSource::Monotonic, 1_000_000_000, 100_000);
    view.apply_event(&mono_event).expect("apply mono event");

    let time_after_mono = view.now().clone();
    assert_eq!(
        time_after_mono.ns, 1_000_000_000,
        "current updated for relevant source"
    );

    // Apply NTP sample (NOT relevant to TrustMonotonicLatest)
    let ntp_event = make_clock_event(ClockSource::Ntp, 1_735_387_200_000_000_000, 50_000_000);
    view.apply_event(&ntp_event).expect("apply ntp event");

    let time_after_ntp = view.now().clone();
    assert_eq!(
        time_after_ntp.ns, 1_000_000_000,
        "current unchanged for irrelevant source"
    );

    // Note: The NTP sample IS recorded in samples vec and latest.ntp cache,
    // but current time belief doesn't change because policy ignores NTP.
}

#[test]
fn t6_event_integration_both_policies() {
    // Verify the inverse: TrustNtpLatest updates for NTP, not for Monotonic
    let mut view = ClockView::new(ClockPolicyId::TrustNtpLatest);

    // Apply monotonic sample (NOT relevant to TrustNtpLatest)
    let mono_event = make_clock_event(ClockSource::Monotonic, 1_000_000_000, 100_000);
    view.apply_event(&mono_event).expect("apply mono event");

    let time_after_mono = view.now().clone();
    assert_eq!(
        time_after_mono.domain,
        TimeDomain::Unknown,
        "still unknown - monotonic irrelevant to TrustNtpLatest"
    );

    // Apply NTP sample (relevant to policy)
    let ntp_event = make_clock_event(ClockSource::Ntp, 1_735_387_200_000_000_000, 50_000_000);
    view.apply_event(&ntp_event).expect("apply ntp event");

    let time_after_ntp = view.now().clone();
    assert_eq!(
        time_after_ntp.ns, 1_735_387_200_000_000_000,
        "current updated for relevant NTP source"
    );
    assert_eq!(time_after_ntp.domain, TimeDomain::Unix);
}
