#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

mod core;
mod profile;

use crate::intent::interview::types::{InterviewSession, Profile};
use crate::intent::plan::types::{ExecutionPlan, PlanBead, PlanError};
use std::collections::HashSet;

/// Emission mode controlling whether beads are persisted or just simulated.
///
/// This enum replaces the `dry_run: bool` parameter, following Scott Wlaschin's DDD
/// principle of making modes explicit rather than using cryptic true/false values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EmissionMode {
  /// Actually persist beads to the execution plan.
  #[default]
  Persist,
  /// Simulate emission without modifying the plan (dry run).
  Simulate,
}

impl EmissionMode {
  /// Check if this mode will persist changes.
  #[must_use]
  pub const fn should_persist(&self) -> bool {
    matches!(self, Self::Persist)
  }

  /// Check if this mode is a simulation (dry run).
  #[must_use]
  pub const fn is_simulation(&self) -> bool {
    matches!(self, Self::Simulate)
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmissionResult {
  pub emitted: usize,
  pub skipped: usize,
  pub errors: Vec<String>,
}

impl Default for EmissionResult {
  fn default() -> Self {
    Self::new()
  }
}

impl EmissionResult {
  #[must_use]
  pub const fn new() -> Self {
    Self {
      emitted: 0,
      skipped: 0,
      errors: Vec::new(),
    }
  }

  pub const fn add_skipped(&mut self, count: usize) {
    self.skipped += count;
  }

  pub fn add_error(&mut self, error: String) {
    self.errors.push(error);
  }

  #[must_use]
  pub const fn is_success(&self) -> bool {
    self.errors.is_empty()
  }

  #[must_use]
  pub const fn total_processed(&self) -> usize {
    self.emitted + self.skipped
  }
}

/// Emits plan beads from interview artifacts.
///
/// # Errors
/// Returns `PlanError` when session validation fails or emitted beads cannot be added to the plan.
pub fn emit_beads(
  session: &InterviewSession,
  plan: &mut ExecutionPlan,
  mode: EmissionMode,
) -> Result<(Vec<PlanBead>, EmissionResult), PlanError> {
  core::emit_beads(session, plan, mode)
}

#[must_use]
pub fn check_existing_beads(titles: &[String], existing: &[String]) -> Vec<String> {
  let existing_set: HashSet<&str> = existing.iter().map(String::as_str).collect();
  titles
    .iter()
    .filter(|title| !existing_set.contains(title.as_str()))
    .cloned()
    .collect()
}

#[must_use]
pub fn generate_profile_beads(profile: Profile, phase: u32) -> Vec<PlanBead> {
  profile::generate_profile_beads(profile, phase)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn emission_mode_default_is_persist() {
    let mode = EmissionMode::default();
    assert!(mode.should_persist());
    assert!(!mode.is_simulation());
  }

  #[test]
  fn emission_mode_predicates_are_exhaustive() {
    for mode in [EmissionMode::Persist, EmissionMode::Simulate] {
      let should_persist = mode.should_persist();
      let is_simulation = mode.is_simulation();

      // Exactly one predicate should be true
      assert_eq!(
        usize::from(should_persist) + usize::from(is_simulation),
        1,
        "Mode {mode:?} should match exactly one predicate"
      );
    }
  }

  #[test]
  fn emission_mode_persist_properties() {
    let mode = EmissionMode::Persist;
    assert!(mode.should_persist());
    assert!(!mode.is_simulation());
  }

  #[test]
  fn emission_mode_simulate_properties() {
    let mode = EmissionMode::Simulate;
    assert!(!mode.should_persist());
    assert!(mode.is_simulation());
  }
}
