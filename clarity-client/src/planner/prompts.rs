//! Stub module for prompts
//!
//! TODO: Implement proper prompts

use crate::planner::types::DiamondPhase;
use crate::planner::types_coach::CoachStep;

/// Get steps for a phase - stub implementation
#[must_use]
pub fn get_steps_for_phase(phase: &DiamondPhase) -> Vec<CoachStep> {
    vec![]
}

/// Get steps for phase string - stub implementation
#[must_use]
pub fn get_steps_for_phase_string(_phase: &str) -> Vec<CoachStep> {
    vec![]
}
