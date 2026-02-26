#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Phase components for the Progressive Discover flow.
//!
//! This module contains the individual phase components that make up
//! the Progressive Discover flow. Each phase is a distinct step in
//! guiding users from initial idea to locked plan.

pub mod preview_phase;

pub use preview_phase::{
    PreviewPhase, PreviewPhaseProps, SummaryField, SummaryFieldProps, TranscriptSummary,
    TranscriptSummaryProps,
};
