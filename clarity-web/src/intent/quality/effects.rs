//! Effects Analyzer (WP29) - Second-order effect detection

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(clippy::missing_errors_doc)]

mod analysis;
mod patterns;
mod rules;
#[cfg(test)]
mod tests;
mod types;

pub use analysis::{
  analyze_behavior, analyze_behavior_report, analyze_feature, analyze_spec,
  behaviors_with_effect_type, count_effects_by_type, has_critical_effects,
  has_high_severity_effects, max_effect_severity,
};
pub use types::{
  Effect, EffectSeverity, EffectType, EffectsError, EffectsReport, EffectsResult, EffectsSummary,
  SpecEffectsReport,
};
