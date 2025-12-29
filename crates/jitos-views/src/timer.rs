// Copyright 2025 James Ross
// SPDX-License-Identifier: Apache-2.0

//! Timer View - Deterministic Timer Semantics
//!
//! SPEC-0004: Timers as materialized view over timer request/fire events.
//! No hidden wall-clock timers.

use jitos_core::{events::EventEnvelope, Hash};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::Time;

/// Observation type tag for timer request events
pub const OBS_TIMER_REQUEST_V0: &str = "OBS_TIMER_REQUEST_V0";

/// Decision type tag for timer fire events
pub const DEC_TIMER_FIRE_V0: &str = "DEC_TIMER_FIRE_V0";

/// Timer view - deterministic materialized view over timer events
#[derive(Debug, Clone)]
pub struct TimerView {
    requests: Vec<TimerRequestRecord>,
    fired: Vec<TimerFireRecord>,
}

impl TimerView {
    /// Create new timer view
    pub fn new() -> Self {
        Self {
            requests: Vec::new(),
            fired: Vec::new(),
        }
    }

    /// Apply one event in canonical worldline order
    ///
    /// # Errors
    ///
    /// Currently never returns an error. Events that are not timer-related
    /// are silently ignored.
    pub fn apply_event(&mut self, event: &EventEnvelope) -> Result<(), TimerError> {
        // Process timer request observations
        if matches!(event.kind(), jitos_core::events::EventKind::Observation) {
            if event.observation_type() == Some(OBS_TIMER_REQUEST_V0) {
                // Decode timer request payload
                let request: TimerRequest = match event.payload().to_value() {
                    Ok(r) => r,
                    Err(_) => return Ok(()), // Ignore malformed requests
                };

                // Create request record with provenance
                let record = TimerRequestRecord {
                    event_id: event.event_id(),
                    request,
                };

                // Track the request
                self.requests.push(record);
            }
        }

        // Process timer fire decisions
        // NOTE: We try to decode any Decision event as TimerFire
        // In the future, Decision events may have decision_type tags like Observations
        if matches!(event.kind(), jitos_core::events::EventKind::Decision) {
            // Attempt to decode as timer fire
            if let Ok(fire) = event.payload().to_value::<TimerFire>() {
                // Create fire record with provenance
                let record = TimerFireRecord {
                    event_id: event.event_id(),
                    fire,
                };

                // Track the fire event
                self.fired.push(record);
            }
            // Silently ignore decisions that aren't timer fires
        }

        Ok(())
    }

    /// Get timers that should fire at current_time but haven't yet
    pub fn pending_timers(&self, current_time: &Time) -> Vec<TimerRequest> {
        let mut pending = Vec::new();

        for record in &self.requests {
            // Check if already fired
            let already_fired = self
                .fired
                .iter()
                .any(|f| f.fire.request_id == record.request.request_id);

            if already_fired {
                continue;
            }

            // Calculate fire time: requested_at + duration
            let fire_time_ns = record.request.requested_at_ns + record.request.duration_ns;

            // Check if current time >= fire time
            if current_time.ns() >= fire_time_ns {
                pending.push(record.request.clone());
            }
        }

        pending
    }
}

impl Default for TimerView {
    fn default() -> Self {
        Self::new()
    }
}

/// Timer request with provenance
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimerRequestRecord {
    pub event_id: Hash,
    pub request: TimerRequest,
}

/// Timer request from observation event
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimerRequest {
    pub request_id: Hash,
    pub duration_ns: u64,
    pub requested_at_ns: u64, // Nanosecond timestamp when request was made
}

/// Timer fire record with provenance
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimerFireRecord {
    pub event_id: Hash,
    pub fire: TimerFire,
}

/// Timer fire from decision event
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimerFire {
    pub request_id: Hash,
    pub fired_at_ns: u64,
}

/// Timer view errors
#[derive(Debug, Error, PartialEq, Eq)]
pub enum TimerError {
    #[error("placeholder error")]
    Placeholder,
}
