#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Phase components for the Progressive Discover flow.
//!
//! This module contains the individual phase components that make up
//! the Progressive Discover flow. Each phase is a distinct step in
//! guiding users from initial idea to locked plan.

pub mod extracting_phase;
pub mod preview_phase;

pub use extracting_phase::{ExtractingPhase, ExtractingPhaseProps};
pub use preview_phase::{
  PreviewPhase, PreviewPhaseProps, SummaryField, SummaryFieldProps, TranscriptSummary,
  TranscriptSummaryProps,
};
