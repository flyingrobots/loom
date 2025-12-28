// Copyright 2025 James Ross
// SPDX-License-Identifier: Apache-2.0

//! Common test utilities for jitos-views tests

use jitos_core::events::{CanonicalBytes, EventEnvelope};
use jitos_views::{ClockSample, ClockSource, OBS_CLOCK_SAMPLE_V0};

/// Helper: Create a clock sample observation event
pub fn make_clock_event(source: ClockSource, value_ns: u64, uncertainty_ns: u64) -> EventEnvelope {
    let sample = ClockSample {
        source,
        value_ns,
        uncertainty_ns,
    };

    EventEnvelope::new_observation(
        CanonicalBytes::from_value(&sample).expect("encode sample"),
        vec![],                                // no parents for test
        Some(OBS_CLOCK_SAMPLE_V0.to_string()), // observation type tag
        None,                                  // agent_id
        None,                                  // signature
    )
    .expect("create observation event")
}
