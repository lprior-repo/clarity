#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(clippy::suspicious_else_formatting)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
// These re-exports are part of the public API, used by consumers via explicit import
#![allow(unused_imports)]

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
