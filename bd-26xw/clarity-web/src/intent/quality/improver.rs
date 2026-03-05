//! Quality Improver (WP30) - Spec improvement suggestions

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

mod suggestions;
#[cfg(test)]
mod tests;
mod types;
mod vague_rules;

pub use suggestions::{
  suggest_examples_improvements, suggest_improvements, suggest_missing_tests,
  suggest_vague_rules_improvements,
};
pub use types::{
  ImprovementSuggestion, ImproverError, IssueCategory, QualityIssueReport, QualityReport,
};
