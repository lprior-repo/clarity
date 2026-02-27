use crate::intent::plan::resolver::resolve_dependencies;
use crate::intent::plan::types::{ExecutionPlan, PlanBead, PlanError};
use std::collections::HashSet;

pub fn validate_plan_dependencies(plan: &ExecutionPlan) -> Result<(), PlanError> {
  let bead_ids: HashSet<&str> = plan.beads.iter().map(|bead| bead.id.as_str()).collect();
  match plan.beads.iter().find_map(|bead| {
    bead
      .dependencies
      .iter()
      .find(|dependency| !bead_ids.contains(dependency.as_str()))
      .map(|dependency| PlanError::InvalidDependency {
        bead_id: bead.id.clone(),
        dependency: dependency.clone(),
      })
  }) {
    Some(error) => Err(error),
    None => Ok(()),
  }
}

#[must_use]
pub fn get_dependents(beads: &[PlanBead], bead_id: &str) -> Vec<String> {
  beads
    .iter()
    .filter(|bead| {
      bead
        .dependencies
        .iter()
        .any(|dependency| dependency == bead_id)
    })
    .map(|bead| bead.id.clone())
    .collect()
}

#[must_use]
pub fn get_dependencies(beads: &[PlanBead], bead_id: &str) -> Vec<String> {
  beads
    .iter()
    .find(|bead| bead.id == bead_id)
    .map_or_else(Vec::new, |bead| bead.dependencies.clone())
}

pub fn apply_resolution_to_plan(plan: &mut ExecutionPlan) -> Result<(), PlanError> {
  resolve_dependencies(&plan.beads).and_then(|resolved| {
    if resolved.is_valid() {
      plan.execution_order = resolved.sorted;
      plan.validated = true;
      Ok(())
    } else {
      Err(PlanError::CircularDependency)
    }
  })
}
