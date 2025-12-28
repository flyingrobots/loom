// Copyright 2025 James Ross
// SPDX-License-Identifier: Apache-2.0

//! Clock View - Time as Materialized View
//!
//! SPEC-0003: Clock View provides deterministic time beliefs as a pure fold
//! over observation events. Time never comes from syscalls.

use jitos_core::{events::EventEnvelope, Hash};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Observation type tag for clock sample events (Phase 0.5.4)
pub const OBS_CLOCK_SAMPLE_V0: &str = "OBS_CLOCK_SAMPLE_V0";

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
        Self {
            samples: Vec::new(),
            latest: LatestSamples::default(),
            current: Time::unknown(),
            policy,
        }
    }

    /// Apply one event in canonical worldline order
    ///
    /// # Errors
    ///
    /// Currently never returns an error. Events that are not clock observations
    /// are silently ignored.
    pub fn apply_event(&mut self, event: &EventEnvelope) -> Result<(), ClockError> {
        // Only process Observation events
        if !matches!(event.kind(), jitos_core::events::EventKind::Observation) {
            return Ok(()); // Ignore non-observation events
        }

        // SPEC-0003 (lines 127-130): Only decode observations tagged OBS_CLOCK_SAMPLE_V0
        // Strict enforcement: untagged or mismatched observations are ignored
        if event.observation_type() != Some(OBS_CLOCK_SAMPLE_V0) {
            return Ok(()); // Ignore observations without correct type tag
        }

        // Decode payload as ClockSample (type tag already verified)
        let sample: ClockSample = match event.payload().to_value() {
            Ok(s) => s,
            Err(_) => {
                // Decoding failed even with correct tag - ignore silently
                return Ok(());
            }
        };

        // Create sample record with provenance
        let record = ClockSampleRecord {
            event_id: event.event_id(),
            sample,
        };

        // Update latest cache (O(1) per source)
        match record.sample.source {
            ClockSource::Monotonic => self.latest.monotonic = Some(record.clone()),
            ClockSource::Ntp => self.latest.ntp = Some(record.clone()),
            ClockSource::Rtc => self.latest.rtc = Some(record.clone()),
            ClockSource::PeerClaim => self.latest.peer = Some(record.clone()),
        }

        // Append to full sample history
        self.samples.push(record);

        // Recompute current time based on policy
        self.current = self.compute_current_time();

        Ok(())
    }

    /// Pure fold over a prefix of a canonical worldline
    ///
    /// # Errors
    ///
    /// Returns [`ClockError::CutOutOfBounds`] if `cut > events.len()`.
    pub fn now_at_cut(
        events: &[EventEnvelope],
        cut: usize,
        policy: ClockPolicyId,
    ) -> Result<Time, ClockError> {
        // Bounds check
        if cut > events.len() {
            return Err(ClockError::CutOutOfBounds {
                cut,
                len: events.len(),
            });
        }

        // Fold over events[0..cut]
        let mut view = Self::new(policy);
        for event in &events[..cut] {
            view.apply_event(event)?;
        }

        Ok(view.current)
    }

    /// Current belief as-of the last applied event
    pub fn now(&self) -> &Time {
        &self.current
    }

    /// Compute current time based on active policy and latest samples
    fn compute_current_time(&self) -> Time {
        match self.policy {
            ClockPolicyId::TrustMonotonicLatest => {
                if let Some(ref record) = self.latest.monotonic {
                    Time {
                        ns: record.sample.value_ns,
                        uncertainty_ns: record.sample.uncertainty_ns,
                        domain: TimeDomain::Monotonic,
                        provenance: vec![record.event_id],
                    }
                } else {
                    Time::unknown()
                }
            }
            ClockPolicyId::TrustNtpLatest => {
                if let Some(ref record) = self.latest.ntp {
                    Time {
                        ns: record.sample.value_ns,
                        uncertainty_ns: record.sample.uncertainty_ns,
                        domain: TimeDomain::Unix,
                        provenance: vec![record.event_id],
                    }
                } else {
                    Time::unknown()
                }
            }
        }
    }
}

/// Time is a belief, not a fact
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Time {
    ns: u64,
    uncertainty_ns: u64,
    domain: TimeDomain,
    provenance: Vec<Hash>,
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

    /// Time value in nanoseconds
    pub fn ns(&self) -> u64 {
        self.ns
    }

    /// Uncertainty in nanoseconds
    pub fn uncertainty_ns(&self) -> u64 {
        self.uncertainty_ns
    }

    /// Time domain
    pub fn domain(&self) -> TimeDomain {
        self.domain
    }

    /// Event IDs contributing to this belief
    pub fn provenance(&self) -> &[Hash] {
        &self.provenance
    }
}

/// Time domain (semantic context for time values)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeDomain {
    Monotonic, // Monotonic time (relative, no wall-clock meaning)
    Unix,      // Unix epoch time (1970-01-01 00:00:00 UTC)
    Unknown,   // No time information available
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
    Monotonic, // Monotonic clock (safe, no jumps)
    Rtc,       // Real-time clock (can jump)
    Ntp,       // Network time protocol
    PeerClaim, // Time claim from another agent
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
    TrustMonotonicLatest, // Use latest monotonic sample only
    TrustNtpLatest,       // Use latest NTP sample only
}

/// Clock view errors
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ClockError {
    #[error("cut {cut} exceeds event sequence length {len}")]
    CutOutOfBounds { cut: usize, len: usize },
}
