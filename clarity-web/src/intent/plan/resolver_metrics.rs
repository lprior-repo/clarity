use crate::intent::plan::resolver::topological_sort;
use crate::intent::plan::types::PlanBead;
use std::collections::HashMap;

#[must_use]
pub fn compute_critical_path(beads: &[PlanBead]) -> Vec<String> {
  let effort_by_id: HashMap<&str, u32> = beads
    .iter()
    .map(|bead| (bead.id.as_str(), bead.effort))
    .collect();

  let order = match topological_sort(beads) {
    Ok(value) => value,
    Err(_) => Vec::new(),
  };
  let (completion, predecessor) = order.iter().fold(
    (
      HashMap::<String, u32>::new(),
      HashMap::<String, String>::new(),
    ),
    |(completion, predecessor), bead_id| {
      let bead = beads.iter().find(|candidate| candidate.id == *bead_id);
      bead.map_or((completion, predecessor), |current| {
        let best_dependency = current
          .dependencies
          .iter()
          .filter_map(|dep| {
            completion
              .get(dep)
              .copied()
              .map(|score| (dep.clone(), score))
          })
          .max_by_key(|(_, score)| *score);

        let max_score = best_dependency.as_ref().map_or(0, |(_, score)| *score);
        let completion_score = max_score
          + effort_by_id
            .get(bead_id.as_str())
            .copied()
            .map_or(0, |value| value);

        let completion_next = completion
          .into_iter()
          .chain(std::iter::once((bead_id.clone(), completion_score)))
          .collect::<HashMap<_, _>>();

        let predecessor_next = best_dependency.map_or(predecessor, |(dep, _)| {
          predecessor
            .into_iter()
            .chain(std::iter::once((bead_id.clone(), dep)))
            .collect::<HashMap<_, _>>()
        });

        (completion_next, predecessor_next)
      })
    },
  );

  completion
    .iter()
    .max_by_key(|(_, score)| *score)
    .map_or_else(Vec::new, |(end_id, _)| {
      trace_path(end_id.clone(), &predecessor)
    })
}

fn trace_path(end: String, predecessor: &HashMap<String, String>) -> Vec<String> {
  std::iter::successors(Some(end), |node| predecessor.get(node).cloned())
    .collect::<Vec<_>>()
    .into_iter()
    .rev()
    .collect()
}

#[must_use]
pub fn compute_parallelism(beads: &[PlanBead]) -> usize {
  let levels = beads
    .iter()
    .fold(HashMap::<String, usize>::new(), |levels, bead| {
      if bead.dependencies.is_empty() {
        levels
          .into_iter()
          .chain(std::iter::once((bead.id.clone(), 0_usize)))
          .collect::<HashMap<_, _>>()
      } else {
        let dep_levels = bead
          .dependencies
          .iter()
          .filter_map(|dep| levels.get(dep).copied())
          .collect::<Vec<_>>();

        if dep_levels.len() == bead.dependencies.len() {
          let next_level = dep_levels.into_iter().max().map_or(0, |level| level + 1);
          levels
            .into_iter()
            .chain(std::iter::once((bead.id.clone(), next_level)))
            .collect::<HashMap<_, _>>()
        } else {
          levels
        }
      }
    });

  levels
    .values()
    .copied()
    .fold(HashMap::<usize, usize>::new(), |counts, level| {
      let updated = counts.get(&level).copied().map_or(1, |count| count + 1);
      counts
        .into_iter()
        .filter(|(existing, _)| *existing != level)
        .chain(std::iter::once((level, updated)))
        .collect::<HashMap<_, _>>()
    })
    .values()
    .copied()
    .max()
    .map_or(0, |value| value)
}
