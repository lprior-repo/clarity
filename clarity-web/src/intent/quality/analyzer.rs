//! Quality Analyzer (WP28) - Quality scoring and analysis for specs

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

mod domain;
mod formatting;
mod issue_collection;
mod scoring;
#[cfg(test)]
mod tests;

pub use domain::{QualityIssue, QualityReport};
pub use formatting::format_report;
pub use scoring::{
    analyze_spec, calculate_ai_readiness_score, calculate_clarity_score, calculate_coverage_score,
    calculate_overall_score, calculate_testability_score,
};
