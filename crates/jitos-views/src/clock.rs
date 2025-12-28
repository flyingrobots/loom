// Copyright 2025 James Ross
// SPDX-License-Identifier: Apache-2.0

//! Clock View - Time as Materialized View
//!
//! SPEC-0003: Clock View provides deterministic time beliefs as a pure fold
//! over observation events. Time never comes from syscalls.

use jitos_core::{events::EventEnvelope, Hash};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Clock view - deterministic materialized view over clock observation events
#[derive(Debug, Clone)]
pub struct ClockView {
    samples: Vec<ClockSampleRecord>,
    latest: LatestSamples,
    current: Time,
    policy: ClockPolicyId,
}

impl ClockView {
    /// Create new clock view with given policy
    pub fn new(policy: ClockPolicyId) -> Self {
        todo!("Implement ClockView::new")
    }

    /// Apply one event in canonical worldline order
    pub fn apply_event(&mut self, _event: &EventEnvelope) -> Result<(), ClockError> {
        todo!("Implement ClockView::apply_event")
    }

    /// Pure fold over a prefix of a canonical worldline
    pub fn now_at_cut(
        _events: &[EventEnvelope],
        _cut: usize,
        _policy: ClockPolicyId,
    ) -> Result<Time, ClockError> {
        todo!("Implement ClockView::now_at_cut")
    }

    /// Current belief as-of the last applied event
    pub fn now(&self) -> &Time {
        &self.current
    }
}

/// Time is a belief, not a fact
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Time {
    pub ns: u64,
    pub uncertainty_ns: u64,
    pub domain: TimeDomain,
    pub provenance: Vec<Hash>,
}

impl Time {
    /// Unknown time (no observations yet)
    pub fn unknown() -> Self {
        Self {
            ns: 0,
            uncertainty_ns: u64::MAX,
            domain: TimeDomain::Unknown,
            provenance: vec![],
        }
    }
}

/// Time domain (semantic context for time values)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeDomain {
    Monotonic,  // Monotonic time (relative, no wall-clock meaning)
    Unix,       // Unix epoch time (1970-01-01 00:00:00 UTC)
    Unknown,    // No time information available
}

/// Clock sample with provenance
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClockSampleRecord {
    pub event_id: Hash,
    pub sample: ClockSample,
}

/// Clock sample from an observation event
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClockSample {
    pub source: ClockSource,
    pub value_ns: u64,
    pub uncertainty_ns: u64,
}

/// Clock source type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClockSource {
    Monotonic,  // Monotonic clock (safe, no jumps)
    Rtc,        // Real-time clock (can jump)
    Ntp,        // Network time protocol
    PeerClaim,  // Time claim from another agent
}

/// Latest samples by source (O(1) cache)
#[derive(Debug, Clone, Default)]
pub struct LatestSamples {
    pub monotonic: Option<ClockSampleRecord>,
    pub ntp: Option<ClockSampleRecord>,
    pub rtc: Option<ClockSampleRecord>,
    pub peer: Option<ClockSampleRecord>,
}

/// Clock policy selector
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockPolicyId {
    TrustMonotonicLatest,  // Use latest monotonic sample only
    TrustNtpLatest,        // Use latest NTP sample only
}

/// Clock view errors
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ClockError {
    #[error("canonical decoding failed for tagged clock sample")]
    DecodingError,

    #[error("cut {cut} exceeds event sequence length {len}")]
    CutOutOfBounds { cut: usize, len: usize },
}
