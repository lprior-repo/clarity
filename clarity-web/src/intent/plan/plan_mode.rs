#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

mod compute;
mod flow;
mod format;
mod types;
mod validate;

use crate::intent::interview::types::InterviewSession;

pub use types::{Action, BeadStatus, ExecutionPlan, Phase, PhaseStatus, PlanBead, PlanError};

pub use format::{
  bead_status_to_string, effort_to_label, format_plan_ai, format_plan_human, format_plan_json,
  phase_status_to_string, risk_to_string,
};

/// Computes an execution plan from the current interview session.
///
/// # Errors
/// Returns `PlanError` when the session does not satisfy plan preconditions.
pub fn compute_plan(session: &InterviewSession) -> Result<ExecutionPlan, PlanError> {
  compute::compute_plan(session)
}

#[must_use]
pub fn apply_phase_gating(plan: &ExecutionPlan, session: &InterviewSession) -> ExecutionPlan {
  flow::apply_phase_gating(plan, session)
}

#[must_use]
pub fn get_actionable_beads(plan: &ExecutionPlan) -> Vec<&PlanBead> {
  flow::get_actionable_beads(plan)
}

/// Check if a specific phase can be executed based on session completion state.
///
/// Phase 1 can always be executed. Other phases require all prior phases
/// (1 to phase_number-1) to be completed.
#[must_use]
pub fn can_execute_phase(session: &InterviewSession, phase_number: u32) -> bool {
  flow::can_execute_phase(session, phase_number)
}

/// Validates that all bead dependencies reference existing beads.
///
/// # Errors
/// Returns `PlanError::InvalidDependency` when any dependency points to a missing bead.
pub fn validate_plan_dependencies(plan: &ExecutionPlan) -> Result<(), PlanError> {
  validate::validate_plan_dependencies(plan)
}
