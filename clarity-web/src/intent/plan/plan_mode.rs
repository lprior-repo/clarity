#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

mod compute;
mod flow;
mod types;
mod validate;

use crate::intent::interview::types::InterviewSession;

pub use types::{
    BeadStatus, ExecutionPlan, Phase, PhaseStatus, PlanBead, PlanError,
};

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

pub fn validate_plan_dependencies(plan: &ExecutionPlan) -> Result<(), PlanError> {
    validate::validate_plan_dependencies(plan)
}
