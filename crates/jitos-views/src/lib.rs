// Copyright 2025 James Ross
// SPDX-License-Identifier: Apache-2.0

//! # jitos-views
//!
//! Materialized views over JITOS event streams.
//!
//! This crate provides deterministic, pure views that fold over event history
//! without side effects. Views never touch syscalls - they are pure functions
//! of their input events.

pub mod clock;
pub mod timer;

pub use clock::{
    ClockError, ClockPolicyId, ClockSample, ClockSampleRecord, ClockSource, ClockView,
    LatestSamples, Time, TimeDomain, OBS_CLOCK_SAMPLE_V0,
};
pub use timer::{
    TimerError, TimerFire, TimerFireRecord, TimerRequest, TimerRequestRecord, TimerView,
    OBS_TIMER_REQUEST_V0,
};
