#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

mod graph;
mod metrics;
mod plan;

use crate::intent::plan::types::{ExecutionPlan, PlanBead, PlanError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolutionResult {
  pub sorted: Vec<String>,
  pub cycles: Vec<Vec<String>>,
}

impl Default for ResolutionResult {
  fn default() -> Self {
    Self::new()
  }
}

impl ResolutionResult {
  #[must_use]
  pub const fn new() -> Self {
    Self {
      sorted: Vec::new(),
      cycles: Vec::new(),
    }
  }

  #[must_use]
  pub const fn is_valid(&self) -> bool {
    self.cycles.is_empty()
  }

  #[must_use]
  pub const fn len(&self) -> usize {
    self.sorted.len()
  }

  #[must_use]
  pub const fn is_empty(&self) -> bool {
    self.sorted.is_empty()
  }
}

/// Resolves bead dependencies and computes an ordered execution plan.
///
/// # Errors
/// Returns `PlanError` when dependencies are invalid or cycles are present.
pub fn resolve_dependencies(beads: &[PlanBead]) -> Result<ResolutionResult, PlanError> {
  graph::resolve_dependencies(beads)
}

#[must_use]
pub fn detect_cycles(beads: &[PlanBead]) -> Vec<Vec<String>> {
  graph::detect_cycles(beads)
}

/// Produces a topologically sorted bead execution order.
///
/// # Errors
/// Returns `PlanError` when dependency cycles are detected.
pub fn topological_sort(beads: &[PlanBead]) -> Result<Vec<String>, PlanError> {
  graph::topological_sort(beads)
}

/// Validates that plan dependencies refer to known bead ids.
///
/// # Errors
/// Returns `PlanError::InvalidDependency` if any dependency cannot be resolved.
pub fn validate_plan_dependencies(plan: &ExecutionPlan) -> Result<(), PlanError> {
  plan::validate_plan_dependencies(plan)
}

#[must_use]
pub fn get_dependents(beads: &[PlanBead], bead_id: &str) -> Vec<String> {
  plan::get_dependents(beads, bead_id)
}

#[must_use]
pub fn get_dependencies(beads: &[PlanBead], bead_id: &str) -> Vec<String> {
  plan::get_dependencies(beads, bead_id)
}

#[must_use]
pub fn compute_critical_path(beads: &[PlanBead]) -> Vec<String> {
  metrics::compute_critical_path(beads)
}

#[must_use]
pub fn compute_parallelism(beads: &[PlanBead]) -> usize {
  metrics::compute_parallelism(beads)
}

/// Applies dependency resolution results to a mutable execution plan.
///
/// # Errors
/// Returns `PlanError` when dependencies are invalid or contain cycles.
pub fn apply_resolution_to_plan(plan: &mut ExecutionPlan) -> Result<(), PlanError> {
  plan::apply_resolution_to_plan(plan)
}
