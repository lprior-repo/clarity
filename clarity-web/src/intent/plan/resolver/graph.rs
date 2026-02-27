use super::ResolutionResult;
use crate::intent::plan::types::{PlanBead, PlanError};
use std::collections::{HashMap, HashSet};

pub fn resolve_dependencies(beads: &[PlanBead]) -> Result<ResolutionResult, PlanError> {
  if beads.is_empty() {
    return Err(PlanError::NoBeads);
  }

  let bead_ids: HashSet<&str> = beads.iter().map(|bead| bead.id.as_str()).collect();
  let invalid = beads.iter().find_map(|bead| {
    bead
      .dependencies
      .iter()
      .find(|dep| !bead_ids.contains(dep.as_str()))
      .map(|dep| PlanError::InvalidDependency {
        bead_id: bead.id.clone(),
        dependency: dep.clone(),
      })
  });

  if let Some(error) = invalid {
    return Err(error);
  }

  let cycles = detect_cycles(beads);
  if !cycles.is_empty() {
    return Ok(ResolutionResult {
      sorted: Vec::new(),
      cycles,
    });
  }

  topological_sort(beads).map(|sorted| ResolutionResult {
    sorted,
    cycles: Vec::new(),
  })
}

#[must_use]
pub fn detect_cycles(beads: &[PlanBead]) -> Vec<Vec<String>> {
  let adjacency: HashMap<String, Vec<String>> = beads
    .iter()
    .map(|bead| (bead.id.clone(), bead.dependencies.clone()))
    .collect();

  beads
    .iter()
    .fold(
      (HashSet::new(), HashSet::new(), Vec::new()),
      |state, bead| {
        let (visited, stack, cycles) = state;
        if visited.contains(&bead.id) {
          (visited, stack, cycles)
        } else {
          dfs_collect_cycles(&bead.id, &adjacency, visited, stack, Vec::new(), cycles)
        }
      },
    )
    .2
}

fn dfs_collect_cycles(
  node: &str,
  adjacency: &HashMap<String, Vec<String>>,
  mut visited: HashSet<String>,
  mut stack: HashSet<String>,
  path: Vec<String>,
  cycles: Vec<Vec<String>>,
) -> (HashSet<String>, HashSet<String>, Vec<Vec<String>>) {
  visited.insert(node.to_string());
  stack.insert(node.to_string());

  let path_now = path
    .into_iter()
    .chain(std::iter::once(node.to_string()))
    .collect::<Vec<_>>();

  let cycled = if let Some(neighbors) = adjacency.get(node) {
    neighbors
      .iter()
      .fold((visited, stack, cycles), |state, neighbor| {
        let (visited_next, stack_next, cycles_list) = state;
        if !visited_next.contains(neighbor) {
          dfs_collect_cycles(
            neighbor,
            adjacency,
            visited_next,
            stack_next,
            path_now.clone(),
            cycles_list,
          )
        } else if stack_next.contains(neighbor) {
          let cycle = path_now
            .iter()
            .position(|item| item == neighbor)
            .map_or_else(Vec::new, |index| path_now[index..].to_vec());
          let cycles_added = (!cycle.is_empty())
            .then_some(cycle)
            .into_iter()
            .chain(cycles_list)
            .collect::<Vec<_>>();
          (visited_next, stack_next, cycles_added)
        } else {
          (visited_next, stack_next, cycles_list)
        }
      })
  } else {
    (visited, stack, cycles)
  };

  let (visited_final, stack_final, cycles_final) = cycled;
  let stack_without = stack_final
    .into_iter()
    .filter(|candidate| candidate != node)
    .collect();
  (visited_final, stack_without, cycles_final)
}

pub fn topological_sort(beads: &[PlanBead]) -> Result<Vec<String>, PlanError> {
  let adjacency: HashMap<String, Vec<String>> = beads
    .iter()
    .map(|bead| (bead.id.clone(), bead.dependencies.clone()))
    .collect();

  let sorted = beads.iter().fold(Vec::new(), |order, bead| {
    if order.contains(&bead.id) {
      order
    } else {
      visit_node(&bead.id, &adjacency, order)
    }
  });

  if sorted.len() == beads.len() {
    Ok(sorted)
  } else {
    Err(PlanError::CircularDependency)
  }
}

fn visit_node(
  node: &str,
  adjacency: &HashMap<String, Vec<String>>,
  order: Vec<String>,
) -> Vec<String> {
  let with_dependencies = if let Some(dependencies) = adjacency.get(node) {
    dependencies.iter().fold(order, |current, dep| {
      if current.contains(dep) {
        current
      } else {
        visit_node(dep, adjacency, current)
      }
    })
  } else {
    order
  };

  if with_dependencies.iter().any(|existing| existing == node) {
    with_dependencies
  } else {
    with_dependencies
      .into_iter()
      .chain(std::iter::once(node.to_string()))
      .collect()
  }
}
