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
    pub fn len(&self) -> usize {
        self.sorted.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.sorted.is_empty()
    }
}

pub fn resolve_dependencies(beads: &[PlanBead]) -> Result<ResolutionResult, PlanError> {
    graph::resolve_dependencies(beads)
}

#[must_use]
pub fn detect_cycles(beads: &[PlanBead]) -> Vec<Vec<String>> {
    graph::detect_cycles(beads)
}

pub fn topological_sort(beads: &[PlanBead]) -> Result<Vec<String>, PlanError> {
    graph::topological_sort(beads)
}

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

pub fn apply_resolution_to_plan(plan: &mut ExecutionPlan) -> Result<(), PlanError> {
    plan::apply_resolution_to_plan(plan)
}
