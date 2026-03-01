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

/// Formats an `EmissionResult` for human-readable display.
///
/// Returns a multi-line string with:
/// - Summary line with emitted/skipped/error counts
/// - List of errors if any exist
#[must_use]
pub fn format_result(result: &EmissionResult) -> String {
  use std::fmt::Write as _;

  let mut output = String::new();

  // Status indicator
  let status = if result.is_success() {
    "SUCCESS"
  } else {
    "FAILED"
  };

  // Summary line
  let _ = writeln!(
    output,
    "Emission Result [{}]: {} emitted, {} skipped, {} errors",
    status,
    result.emitted,
    result.skipped,
    result.errors.len()
  );

  // Total processed
  let _ = writeln!(output, "  Total processed: {}", result.total_processed());

  // Error details
  if !result.errors.is_empty() {
    output.push_str("\nErrors:\n");
    for (index, error) in result.errors.iter().enumerate() {
      let _ = writeln!(output, "  {}. {}", index + 1, error);
    }
  }

  output
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

/// Filters a list of proposed beads, returning only those that don't already exist.
///
/// This is a test helper function that checks bead uniqueness by ID.
/// It's useful for tests that need to verify emission logic without
/// duplicating existing beads.
///
/// # Arguments
/// * `proposed` - The beads to be filtered
/// * `existing` - The beads that already exist
///
/// # Returns
/// A vector containing only the proposed beads whose IDs are not in existing.
///
/// # Example
/// ```ignore
/// use clarity_web::intent::plan::filter_new_beads_for_test;
/// use clarity_web::intent::plan::types::PlanBead;
///
/// let existing = vec![
///   PlanBead::new("bead-1".to_string(), "Existing".to_string(), 1).unwrap(),
/// ];
/// let proposed = vec![
///   PlanBead::new("bead-1".to_string(), "Duplicate".to_string(), 1).unwrap(),
///   PlanBead::new("bead-2".to_string(), "New".to_string(), 1).unwrap(),
/// ];
///
/// let new_beads = filter_new_beads_for_test(&proposed, &existing);
/// assert_eq!(new_beads.len(), 1);
/// assert_eq!(new_beads[0].id, "bead-2");
/// ```
#[must_use]
pub fn filter_new_beads_for_test(proposed: &[PlanBead], existing: &[PlanBead]) -> Vec<PlanBead> {
  let existing_ids: HashSet<&str> = existing.iter().map(|bead| bead.id.as_str()).collect();
  proposed
    .iter()
    .filter(|bead| !existing_ids.contains(bead.id.as_str()))
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

  #[test]
  fn format_result_success_with_no_errors() {
    let result = EmissionResult {
      emitted: 5,
      skipped: 2,
      errors: vec![],
    };

    let formatted = format_result(&result);

    assert!(formatted.contains("SUCCESS"));
    assert!(formatted.contains("5 emitted"));
    assert!(formatted.contains("2 skipped"));
    assert!(formatted.contains("0 errors"));
    assert!(formatted.contains("Total processed: 7"));
    assert!(!formatted.contains("Errors:"));
  }

  #[test]
  fn format_result_failed_with_errors() {
    let result = EmissionResult {
      emitted: 3,
      skipped: 1,
      errors: vec![
        "Failed to add bead 'test-1': duplicate id".to_string(),
        "Failed to add bead 'test-2': invalid phase".to_string(),
      ],
    };

    let formatted = format_result(&result);

    assert!(formatted.contains("FAILED"));
    assert!(formatted.contains("3 emitted"));
    assert!(formatted.contains("1 skipped"));
    assert!(formatted.contains("2 errors"));
    assert!(formatted.contains("Errors:"));
    assert!(formatted.contains("1. Failed to add bead 'test-1': duplicate id"));
    assert!(formatted.contains("2. Failed to add bead 'test-2': invalid phase"));
  }

  #[test]
  fn format_result_empty() {
    let result = EmissionResult::new();
    let formatted = format_result(&result);

    assert!(formatted.contains("SUCCESS"));
    assert!(formatted.contains("0 emitted"));
    assert!(formatted.contains("0 skipped"));
    assert!(formatted.contains("0 errors"));
    assert!(formatted.contains("Total processed: 0"));
  }

  // ============================================
  // filter_new_beads_for_test tests
  // ============================================

  #[test]
  fn filter_new_beads_for_test_returns_empty_when_all_exist() {
    let existing = vec![
      PlanBead::new("bead-1".to_string(), "First".to_string(), 1),
      PlanBead::new("bead-2".to_string(), "Second".to_string(), 1),
    ]
    .into_iter()
    .filter_map(Result::ok)
    .collect::<Vec<_>>();

    let proposed = vec![
      PlanBead::new("bead-1".to_string(), "First Again".to_string(), 1),
      PlanBead::new("bead-2".to_string(), "Second Again".to_string(), 1),
    ]
    .into_iter()
    .filter_map(Result::ok)
    .collect::<Vec<_>>();

    let new_beads = filter_new_beads_for_test(&proposed, &existing);
    assert!(new_beads.is_empty());
  }

  #[test]
  fn filter_new_beads_for_test_returns_all_when_none_exist() {
    let existing: Vec<PlanBead> = vec![];

    let proposed = vec![
      PlanBead::new("bead-1".to_string(), "First".to_string(), 1),
      PlanBead::new("bead-2".to_string(), "Second".to_string(), 1),
    ]
    .into_iter()
    .filter_map(Result::ok)
    .collect::<Vec<_>>();

    let new_beads = filter_new_beads_for_test(&proposed, &existing);
    assert_eq!(new_beads.len(), 2);
  }

  #[test]
  fn filter_new_beads_for_test_filters_by_id_not_title() {
    let existing = vec![PlanBead::new(
      "bead-1".to_string(),
      "Original Title".to_string(),
      1,
    )]
    .into_iter()
    .filter_map(Result::ok)
    .collect::<Vec<_>>();

    // Same ID but different title - should be filtered out
    let proposed = vec![
      PlanBead::new("bead-1".to_string(), "Different Title".to_string(), 1),
      PlanBead::new("bead-2".to_string(), "New Bead".to_string(), 1),
    ]
    .into_iter()
    .filter_map(Result::ok)
    .collect::<Vec<_>>();

    let new_beads = filter_new_beads_for_test(&proposed, &existing);
    assert_eq!(new_beads.len(), 1);
    assert_eq!(new_beads[0].id, "bead-2");
  }

  #[test]
  fn filter_new_beads_for_test_via_public_api() {
    // This test demonstrates that the function is accessible via the public API
    // through the plan module re-export
    use crate::intent::plan::types::PlanBead;

    let existing = vec![PlanBead::new(
      "existing-1".to_string(),
      "Existing".to_string(),
      1,
    )]
    .into_iter()
    .filter_map(Result::ok)
    .collect::<Vec<_>>();

    let proposed = vec![
      PlanBead::new("existing-1".to_string(), "Duplicate ID".to_string(), 1),
      PlanBead::new("new-1".to_string(), "New Bead".to_string(), 1),
      PlanBead::new("new-2".to_string(), "Another New".to_string(), 2),
    ]
    .into_iter()
    .filter_map(Result::ok)
    .collect::<Vec<_>>();

    // Call via the super module (public re-export path)
    let new_beads = super::filter_new_beads_for_test(&proposed, &existing);

    assert_eq!(new_beads.len(), 2);
    assert!(new_beads.iter().any(|b| b.id == "new-1"));
    assert!(new_beads.iter().any(|b| b.id == "new-2"));
    assert!(!new_beads.iter().any(|b| b.id == "existing-1"));
  }
}
