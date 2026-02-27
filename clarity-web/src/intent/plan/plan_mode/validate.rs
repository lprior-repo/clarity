use super::types::{ExecutionPlan, PlanBead, PlanError};
use std::collections::{HashMap, HashSet};

pub fn validate_plan_dependencies(plan: &ExecutionPlan) -> Result<(), PlanError> {
  let all_beads: HashMap<&str, &PlanBead> = plan
    .phases
    .iter()
    .flat_map(|phase| phase.beads.iter())
    .map(|bead| (bead.id.as_str(), bead))
    .collect();

  all_beads
    .keys()
    .try_fold(HashSet::new(), |visited, bead_id| {
      if visited.contains(*bead_id) {
        Ok(visited)
      } else {
        validate_bead_dependencies(&all_beads, bead_id, HashSet::new(), visited)
      }
    })
    .map(|_| ())
}

fn validate_bead_dependencies<'a>(
  all_beads: &HashMap<&'a str, &'a PlanBead>,
  bead_id: &'a str,
  visiting: HashSet<&'a str>,
  visited: HashSet<&'a str>,
) -> Result<HashSet<&'a str>, PlanError> {
  if visited.contains(bead_id) {
    return Ok(visited);
  }
  if visiting.contains(bead_id) {
    return Err(PlanError::CircularDependency(
      bead_id.to_string(),
      bead_id.to_string(),
    ));
  }

  let visiting_next = visiting
    .into_iter()
    .chain(std::iter::once(bead_id))
    .collect::<HashSet<_>>();

  let visited_after_dependencies = all_beads.get(bead_id).map_or(Ok(visited.clone()), |bead| {
    bead
      .depends_on
      .iter()
      .try_fold(visited, |visited_state, dependency| {
        if visiting_next.contains(dependency.as_str()) {
          return Err(PlanError::CircularDependency(
            bead_id.to_string(),
            dependency.clone(),
          ));
        }
        validate_bead_dependencies(
          all_beads,
          dependency.as_str(),
          visiting_next.clone(),
          visited_state,
        )
      })
  })?;

  Ok(
    visited_after_dependencies
      .into_iter()
      .chain(std::iter::once(bead_id))
      .collect(),
  )
}
