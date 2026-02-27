#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![forbid(unsafe_code)]

mod plan_mode_compute;
mod plan_mode_flow;
mod plan_mode_types;
mod plan_mode_validate;

use crate::intent::interview::types::InterviewSession;

pub use plan_mode_types::{
    BeadStatus, ExecutionPlan, Phase, PhaseStatus, PlanBead, PlanError,
};

pub fn compute_plan(session: &InterviewSession) -> Result<ExecutionPlan, PlanError> {
    plan_mode_compute::compute_plan(session)
}

#[must_use]
pub fn apply_phase_gating(plan: &ExecutionPlan, session: &InterviewSession) -> ExecutionPlan {
    plan_mode_flow::apply_phase_gating(plan, session)
}

#[must_use]
pub fn get_actionable_beads(plan: &ExecutionPlan) -> Vec<&PlanBead> {
    plan_mode_flow::get_actionable_beads(plan)
}

pub fn validate_plan_dependencies(plan: &ExecutionPlan) -> Result<(), PlanError> {
    plan_mode_validate::validate_plan_dependencies(plan)
}
