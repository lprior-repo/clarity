//! WP25: Resolver - Dependency resolution and topological sorting
//!
//! This module provides dependency resolution for plan beads using
//! topological sorting with cycle detection.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::intent::plan::types::{ExecutionPlan, PlanBead, PlanError};
use std::collections::{HashMap, HashSet};

/// Result of dependency resolution
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolutionResult {
    /// Bead IDs in topological order
    pub sorted: Vec<String>,
    /// Detected cycles (each cycle is a list of bead IDs forming the cycle)
    pub cycles: Vec<Vec<String>>,
}

impl Default for ResolutionResult {
    fn default() -> Self {
        Self::new()
    }
}

impl ResolutionResult {
    /// Create a new empty resolution result
    #[must_use]
    pub const fn new() -> Self {
        Self {
            sorted: Vec::new(),
            cycles: Vec::new(),
        }
    }

    /// Check if resolution was successful (no cycles)
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.cycles.is_empty()
    }

    /// Get the number of resolved beads
    #[must_use]
    pub const fn len(&self) -> usize {
        self.sorted.len()
    }

    /// Check if there are no resolved beads
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.sorted.is_empty()
    }
}

/// Resolve dependencies using topological sort
///
/// Builds a dependency graph from beads, detects cycles, and performs
/// topological sorting to determine execution order.
///
/// # Arguments
/// * `beads` - Slice of plan beads to resolve
///
/// # Returns
/// Resolution result with sorted bead IDs and any detected cycles
///
/// # Errors
/// Returns `PlanError::NoBeads` if beads slice is empty
pub fn resolve_dependencies(beads: &[PlanBead]) -> Result<ResolutionResult, PlanError> {
    if beads.is_empty() {
        return Err(PlanError::NoBeads);
    }

    // Build bead ID set for validation
    let bead_ids: HashSet<&str> = beads.iter().map(|b| b.id.as_str()).collect();

    // Validate dependencies reference existing beads
    for bead in beads {
        for dep in &bead.dependencies {
            if !bead_ids.contains(dep.as_str()) {
                return Err(PlanError::InvalidDependency {
                    bead_id: bead.id.clone(),
                    dependency: dep.clone(),
                });
            }
        }
    }

    // Detect cycles first
    let cycles = detect_cycles(beads);

    // If cycles exist, return partial result with cycles
    if !cycles.is_empty() {
        return Ok(ResolutionResult {
            sorted: Vec::new(),
            cycles,
        });
    }

    // Perform topological sort
    let sorted = topological_sort(beads)?;

    Ok(ResolutionResult {
        sorted,
        cycles: Vec::new(),
    })
}

/// Detect cycles in the dependency graph using DFS
///
/// # Arguments
/// * `beads` - Slice of plan beads to analyze
///
/// # Returns
/// Vector of cycles found (each cycle is a list of bead IDs)
#[must_use]
pub fn detect_cycles(beads: &[PlanBead]) -> Vec<Vec<String>> {
    let mut cycles = Vec::new();
    let mut visited: HashSet<String> = HashSet::new();
    let mut rec_stack: HashSet<String> = HashSet::new();
    let mut path = Vec::new();

    // Build adjacency map (owned strings to avoid lifetime issues)
    let adj: HashMap<String, Vec<String>> = beads
        .iter()
        .map(|b| (b.id.clone(), b.dependencies.clone()))
        .collect();

    for bead in beads {
        if !visited.contains(&bead.id) {
            dfs_detect_cycles_owned(
                &bead.id,
                &adj,
                &mut visited,
                &mut rec_stack,
                &mut path,
                &mut cycles,
            );
        }
    }

    cycles
}

/// DFS helper for cycle detection (owned version to avoid lifetime complexity)
fn dfs_detect_cycles_owned(
    node: &str,
    adj: &HashMap<String, Vec<String>>,
    visited: &mut HashSet<String>,
    rec_stack: &mut HashSet<String>,
    path: &mut Vec<String>,
    cycles: &mut Vec<Vec<String>>,
) {
    visited.insert(node.to_string());
    rec_stack.insert(node.to_string());
    path.push(node.to_string());

    if let Some(neighbors) = adj.get(node) {
        for neighbor in neighbors {
            if !visited.contains(neighbor) {
                dfs_detect_cycles_owned(neighbor, adj, visited, rec_stack, path, cycles);
            } else if rec_stack.contains(neighbor) {
                // Found a cycle - extract it from path
                if let Some(cycle_start) = path.iter().position(|p| p == neighbor) {
                    let cycle: Vec<String> = path[cycle_start..].to_vec();
                    cycles.push(cycle);
                }
            }
        }
    }

    path.pop();
    rec_stack.remove(node);
}

/// Perform topological sort on beads
///
/// Uses Kahn's algorithm for efficient topological sorting.
///
/// # Arguments
/// * `beads` - Slice of plan beads to sort
///
/// # Returns
/// Vector of bead IDs in topological order
///
/// # Errors
/// Returns `PlanError::CircularDependency` if a cycle is detected
pub fn topological_sort(beads: &[PlanBead]) -> Result<Vec<String>, PlanError> {
    if beads.is_empty() {
        return Ok(Vec::new());
    }

    // Build adjacency list and in-degree count
    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();

    // Initialize all beads with 0 in-degree
    for bead in beads {
        in_degree.insert(bead.id.as_str(), 0);
        adj.insert(bead.id.as_str(), Vec::new());
    }

    // Build graph: for each dependency, add edge from dep -> bead
    for bead in beads {
        for dep in &bead.dependencies {
            if let Some(neighbors) = adj.get_mut(dep.as_str()) {
                neighbors.push(bead.id.as_str());
            }
            if let Some(deg) = in_degree.get_mut(bead.id.as_str()) {
                *deg += 1;
            }
        }
    }

    // Find all nodes with 0 in-degree
    let mut queue: Vec<&str> = in_degree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(&id, _)| id)
        .collect();

    // Sort queue for deterministic ordering (by phase, then priority)
    queue.sort_by(|&a, &b| {
        let bead_a = beads.iter().find(|bead| bead.id == a);
        let bead_b = beads.iter().find(|bead| bead.id == b);
        match (bead_a, bead_b) {
            (Some(ba), Some(bb)) => (ba.phase, ba.priority).cmp(&(bb.phase, bb.priority)),
            _ => std::cmp::Ordering::Equal,
        }
    });

    let mut result = Vec::new();

    while !queue.is_empty() {
        // Get node with 0 in-degree
        let node = queue.remove(0);
        result.push(node.to_string());

        // Reduce in-degree of neighbors
        if let Some(neighbors) = adj.get(node) {
            let mut new_zero_degree = Vec::new();

            for &neighbor in neighbors {
                if let Some(deg) = in_degree.get_mut(neighbor) {
                    *deg = deg.saturating_sub(1);
                    if *deg == 0 {
                        new_zero_degree.push(neighbor);
                    }
                }
            }

            // Sort new zero-degree nodes
            new_zero_degree.sort_by(|&a, &b| {
                let bead_a = beads.iter().find(|bead| bead.id == a);
                let bead_b = beads.iter().find(|bead| bead.id == b);
                match (bead_a, bead_b) {
                    (Some(ba), Some(bb)) => (ba.phase, ba.priority).cmp(&(bb.phase, bb.priority)),
                    _ => std::cmp::Ordering::Equal,
                }
            });

            queue.extend(new_zero_degree);
        }
    }

    // If result doesn't contain all nodes, there's a cycle
    if result.len() != beads.len() {
        return Err(PlanError::CircularDependency);
    }

    Ok(result)
}

/// Validate dependencies in an execution plan
///
/// # Arguments
/// * `plan` - The execution plan to validate
///
/// # Returns
/// Ok(()) if all dependencies are valid
///
/// # Errors
/// Returns `PlanError::InvalidDependency` if a dependency references a non-existent bead
pub fn validate_plan_dependencies(plan: &ExecutionPlan) -> Result<(), PlanError> {
    let bead_ids: HashSet<&str> = plan.beads.iter().map(|b| b.id.as_str()).collect();

    for bead in &plan.beads {
        for dep in &bead.dependencies {
            if !bead_ids.contains(dep.as_str()) {
                return Err(PlanError::InvalidDependency {
                    bead_id: bead.id.clone(),
                    dependency: dep.clone(),
                });
            }
        }
    }

    Ok(())
}

/// Get beads that depend on a given bead
///
/// # Arguments
/// * `beads` - Slice of plan beads
/// * `bead_id` - ID of the bead to find dependents for
///
/// # Returns
/// Vector of bead IDs that depend on the given bead
#[must_use]
pub fn get_dependents(beads: &[PlanBead], bead_id: &str) -> Vec<String> {
    beads
        .iter()
        .filter(|b| b.dependencies.iter().any(|d| d == bead_id))
        .map(|b| b.id.clone())
        .collect()
}

/// Get dependencies of a given bead
///
/// # Arguments
/// * `beads` - Slice of plan beads
/// * `bead_id` - ID of the bead to get dependencies for
///
/// # Returns
/// Vector of dependency bead IDs, or empty if bead not found
#[must_use]
pub fn get_dependencies(beads: &[PlanBead], bead_id: &str) -> Vec<String> {
    beads
        .iter()
        .find(|b| b.id == bead_id)
        .map(|b| b.dependencies.clone())
        .unwrap_or_default()
}

/// Compute the critical path through the dependency graph
///
/// The critical path is the longest path through the graph,
/// representing the minimum time to complete all beads if
/// dependencies must be satisfied sequentially.
///
/// # Arguments
/// * `beads` - Slice of plan beads with effort estimates
///
/// # Returns
/// Vector of bead IDs forming the critical path
#[must_use]
pub fn compute_critical_path(beads: &[PlanBead]) -> Vec<String> {
    if beads.is_empty() {
        return Vec::new();
    }

    // Build adjacency list (reverse direction: from dependency to dependent)
    let mut dependents: HashMap<&str, Vec<&str>> = HashMap::new();
    let mut effort: HashMap<&str, u32> = HashMap::new();

    for bead in beads {
        effort.insert(bead.id.as_str(), bead.effort);
        for dep in &bead.dependencies {
            dependents.entry(dep.as_str()).or_default().push(bead.id.as_str());
        }
    }

    // Find beads with no dependencies (starting points)
    let _starting_beads: Vec<&str> = beads
        .iter()
        .filter(|b| b.dependencies.is_empty())
        .map(|b| b.id.as_str())
        .collect();

    // Compute earliest completion time for each bead using dynamic programming
    let mut earliest_completion: HashMap<&str, u32> = HashMap::new();
    let mut predecessor: HashMap<&str, &str> = HashMap::new();

    // Process in topological order
    if let Ok(order) = topological_sort(beads) {
        for bead_id in &order {
            let bead = beads.iter().find(|b| &b.id == bead_id);
            if let Some(bead) = bead {
                let max_dep_completion = bead
                    .dependencies
                    .iter()
                    .filter_map(|dep| earliest_completion.get(dep.as_str()).copied())
                    .max()
                    .unwrap_or(0);

                let completion = max_dep_completion + bead.effort;
                earliest_completion.insert(bead_id.as_str(), completion);

                // Find which dependency led to this completion time
                for dep in &bead.dependencies {
                    if let Some(&dep_completion) = earliest_completion.get(dep.as_str()) {
                        if dep_completion == max_dep_completion {
                            predecessor.insert(bead_id.as_str(), dep.as_str());
                        }
                    }
                }
            }
        }

        // Find bead with maximum completion time (end of critical path)
        let critical_end = earliest_completion
            .iter()
            .max_by_key(|(_, &completion)| completion)
            .map(|(&id, _)| id);

        // Trace back to find critical path
        let mut path = Vec::new();
        if let Some(mut current) = critical_end {
            path.push(current.to_string());
            while let Some(&pred) = predecessor.get(current) {
                path.push(pred.to_string());
                current = pred;
            }
            path.reverse();
        }

        path
    } else {
        Vec::new()
    }
}

/// Compute parallelism factor - maximum number of beads that can run concurrently
///
/// # Arguments
/// * `beads` - Slice of plan beads
///
/// # Returns
/// Maximum number of beads that can be executed in parallel
#[must_use]
pub fn compute_parallelism(beads: &[PlanBead]) -> usize {
    if beads.is_empty() {
        return 0;
    }

    // Compute for each "level" how many beads can run
    let mut levels: HashMap<&str, usize> = HashMap::new();

    // Level 0: beads with no dependencies
    for bead in beads {
        if bead.dependencies.is_empty() {
            levels.insert(bead.id.as_str(), 0);
        }
    }

    // Compute levels iteratively
    let mut changed = true;
    while changed {
        changed = false;
        for bead in beads {
            if !levels.contains_key(bead.id.as_str()) {
                // Check if all dependencies have levels
                let dep_levels: Vec<usize> = bead
                    .dependencies
                    .iter()
                    .filter_map(|dep| levels.get(dep.as_str()).copied())
                    .collect();

                if dep_levels.len() == bead.dependencies.len() {
                    let level = dep_levels.into_iter().max().map_or(0, |max| max + 1);
                    levels.insert(bead.id.as_str(), level);
                    changed = true;
                }
            }
        }
    }

    // Count beads at each level
    let mut level_counts: HashMap<usize, usize> = HashMap::new();
    for &level in levels.values() {
        *level_counts.entry(level).or_insert(0) += 1;
    }

    // Return max count at any level
    level_counts.values().copied().max().unwrap_or(0)
}

/// Apply topological order to an execution plan
///
/// Updates the plan's `execution_order` field with the resolved order.
///
/// # Arguments
/// * `plan` - The execution plan to update
///
/// # Returns
/// Ok(()) if successful
///
/// # Errors
/// Returns `PlanError` if resolution fails
pub fn apply_resolution_to_plan(plan: &mut ExecutionPlan) -> Result<(), PlanError> {
    let result = resolve_dependencies(&plan.beads)?;

    if !result.is_valid() {
        return Err(PlanError::CircularDependency);
    }

    plan.execution_order = result.sorted;
    plan.validated = true;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_bead(id: &str, phase: u32, dependencies: Vec<&str>) -> PlanBead {
        PlanBead::new(id.to_string(), format!("Bead {}", id), phase)
            .expect("valid bead")
            .with_dependencies(
                dependencies
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            )
    }

    impl PlanBead {
        fn with_dependencies(mut self, deps: Vec<String>) -> Self {
            self.dependencies = deps;
            self
        }
    }

    #[test]
    fn test_resolution_result_new() {
        let result = ResolutionResult::new();
        assert!(result.is_empty());
        assert!(result.is_valid());
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_resolution_result_with_cycles() {
        let result = ResolutionResult {
            sorted: vec!["a".to_string()],
            cycles: vec![vec!["b".to_string(), "c".to_string()]],
        };

        assert!(!result.is_valid());
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_resolve_dependencies_empty() {
        let beads: Vec<PlanBead> = Vec::new();
        let result = resolve_dependencies(&beads);
        assert!(matches!(result, Err(PlanError::NoBeads)));
    }

    #[test]
    fn test_resolve_dependencies_single_bead() {
        let beads = vec![create_test_bead("a", 1, vec![])];
        let result = resolve_dependencies(&beads);

        assert!(result.is_ok());
        let res = result.expect("ok");
        assert!(res.is_valid());
        assert_eq!(res.sorted, vec!["a"]);
    }

    #[test]
    fn test_resolve_dependencies_simple_chain() {
        let beads = vec![
            create_test_bead("c", 1, vec!["b"]),
            create_test_bead("b", 1, vec!["a"]),
            create_test_bead("a", 1, vec![]),
        ];

        let result = resolve_dependencies(&beads);
        assert!(result.is_ok());
        let res = result.expect("ok");

        assert!(res.is_valid());
        assert_eq!(res.sorted, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_resolve_dependencies_diamond() {
        let beads = vec![
            create_test_bead("d", 1, vec!["b", "c"]),
            create_test_bead("c", 1, vec!["a"]),
            create_test_bead("b", 1, vec!["a"]),
            create_test_bead("a", 1, vec![]),
        ];

        let result = resolve_dependencies(&beads);
        assert!(result.is_ok());
        let res = result.expect("ok");

        assert!(res.is_valid());
        // a must come first
        assert_eq!(res.sorted[0], "a");
        // d must come last
        assert_eq!(res.sorted[3], "d");
        // b and c can be in any order between a and d
        assert!(res.sorted.contains(&"b".to_string()));
        assert!(res.sorted.contains(&"c".to_string()));
    }

    #[test]
    fn test_resolve_dependencies_invalid_dependency() {
        let beads = vec![create_test_bead("a", 1, vec!["nonexistent"])];

        let result = resolve_dependencies(&beads);
        assert!(matches!(
            result,
            Err(PlanError::InvalidDependency { .. })
        ));
    }

    #[test]
    fn test_detect_cycles_no_cycle() {
        let beads = vec![
            create_test_bead("a", 1, vec![]),
            create_test_bead("b", 1, vec!["a"]),
            create_test_bead("c", 1, vec!["b"]),
        ];

        let cycles = detect_cycles(&beads);
        assert!(cycles.is_empty());
    }

    #[test]
    fn test_detect_cycles_simple_cycle() {
        let beads = vec![
            create_test_bead("a", 1, vec!["c"]),
            create_test_bead("b", 1, vec!["a"]),
            create_test_bead("c", 1, vec!["b"]),
        ];

        let cycles = detect_cycles(&beads);
        assert!(!cycles.is_empty());
    }

    #[test]
    fn test_detect_cycles_self_cycle() {
        let beads = vec![create_test_bead("a", 1, vec!["a"])];

        let cycles = detect_cycles(&beads);
        assert!(!cycles.is_empty());
    }

    #[test]
    fn test_detect_cycles_disconnected() {
        let beads = vec![
            create_test_bead("a", 1, vec![]),
            create_test_bead("b", 1, vec![]),
            create_test_bead("c", 1, vec!["c"]), // Self-cycle
        ];

        let cycles = detect_cycles(&beads);
        assert!(!cycles.is_empty());
    }

    #[test]
    fn test_topological_sort_empty() {
        let beads: Vec<PlanBead> = Vec::new();
        let result = topological_sort(&beads);
        assert!(result.is_ok());
        assert!(result.expect("ok").is_empty());
    }

    #[test]
    fn test_topological_sort_single() {
        let beads = vec![create_test_bead("a", 1, vec![])];
        let result = topological_sort(&beads);

        assert!(result.is_ok());
        assert_eq!(result.expect("ok"), vec!["a"]);
    }

    #[test]
    fn test_topological_sort_with_cycle() {
        let beads = vec![
            create_test_bead("a", 1, vec!["b"]),
            create_test_bead("b", 1, vec!["a"]),
        ];

        let result = topological_sort(&beads);
        assert!(matches!(result, Err(PlanError::CircularDependency)));
    }

    #[test]
    fn test_topological_sort_respects_phases() {
        let beads = vec![
            create_test_bead("phase2-a", 2, vec![]),
            create_test_bead("phase1-a", 1, vec![]),
            create_test_bead("phase2-b", 2, vec![]),
            create_test_bead("phase1-b", 1, vec![]),
        ];

        let result = topological_sort(&beads);
        assert!(result.is_ok());
        let sorted = result.expect("ok");

        // Phase 1 beads should come before phase 2 beads
        let phase1_pos_a = sorted.iter().position(|id| id == "phase1-a").expect("found");
        let phase1_pos_b = sorted.iter().position(|id| id == "phase1-b").expect("found");
        let phase2_pos_a = sorted.iter().position(|id| id == "phase2-a").expect("found");
        let phase2_pos_b = sorted.iter().position(|id| id == "phase2-b").expect("found");

        assert!(phase1_pos_a < phase2_pos_a);
        assert!(phase1_pos_b < phase2_pos_b);
    }

    #[test]
    fn test_get_dependents() {
        let beads = vec![
            create_test_bead("a", 1, vec![]),
            create_test_bead("b", 1, vec!["a"]),
            create_test_bead("c", 1, vec!["a"]),
            create_test_bead("d", 1, vec!["b"]),
        ];

        let dependents = get_dependents(&beads, "a");
        assert_eq!(dependents.len(), 2);
        assert!(dependents.contains(&"b".to_string()));
        assert!(dependents.contains(&"c".to_string()));

        let dependents_d = get_dependents(&beads, "d");
        assert!(dependents_d.is_empty());
    }

    #[test]
    fn test_get_dependencies() {
        let beads = vec![
            create_test_bead("a", 1, vec![]),
            create_test_bead("b", 1, vec!["a", "c"]),
        ];

        let deps = get_dependencies(&beads, "b");
        assert_eq!(deps.len(), 2);
        assert!(deps.contains(&"a".to_string()));
        assert!(deps.contains(&"c".to_string()));

        let deps_a = get_dependencies(&beads, "a");
        assert!(deps_a.is_empty());

        let deps_nonexistent = get_dependencies(&beads, "nonexistent");
        assert!(deps_nonexistent.is_empty());
    }

    #[test]
    fn test_validate_plan_dependencies_valid() {
        let mut plan = ExecutionPlan::new("test".to_string());
        plan.add_bead(create_test_bead("a", 1, vec![])).expect("add");
        plan.add_bead(create_test_bead("b", 1, vec!["a"])).expect("add");

        let result = validate_plan_dependencies(&plan);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_plan_dependencies_invalid() {
        let mut plan = ExecutionPlan::new("test".to_string());
        plan.add_bead(create_test_bead("a", 1, vec!["nonexistent"]))
            .expect("add");

        let result = validate_plan_dependencies(&plan);
        assert!(matches!(
            result,
            Err(PlanError::InvalidDependency { .. })
        ));
    }

    #[test]
    fn test_compute_critical_path_empty() {
        let beads: Vec<PlanBead> = Vec::new();
        let path = compute_critical_path(&beads);
        assert!(path.is_empty());
    }

    #[test]
    fn test_compute_critical_path_single() {
        let beads = vec![PlanBead::new("a".to_string(), "A".to_string(), 1)
            .expect("valid")
            .with_effort(5)];

        let path = compute_critical_path(&beads);
        assert_eq!(path, vec!["a"]);
    }

    #[test]
    fn test_compute_critical_path_chain() {
        let mut beads = vec![
            create_test_bead("a", 1, vec![]),
            create_test_bead("b", 1, vec!["a"]),
            create_test_bead("c", 1, vec!["b"]),
        ];

        beads[0].effort = 2;
        beads[1].effort = 5;
        beads[2].effort = 3;

        let path = compute_critical_path(&beads);
        // Critical path should include all beads in the chain
        assert_eq!(path, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_compute_critical_path_with_branch() {
        let mut beads = vec![
            create_test_bead("start", 1, vec![]),
            create_test_bead("path-a", 1, vec!["start"]),
            create_test_bead("path-b", 1, vec!["start"]),
            create_test_bead("end", 1, vec!["path-a", "path-b"]),
        ];

        beads[0].effort = 1;
        beads[1].effort = 10; // Critical path through path-a
        beads[2].effort = 2;
        beads[3].effort = 1;

        let path = compute_critical_path(&beads);
        // Critical path should go through path-a (higher effort)
        assert!(path.contains(&"start".to_string()));
        assert!(path.contains(&"path-a".to_string()));
        assert!(path.contains(&"end".to_string()));
    }

    #[test]
    fn test_compute_parallelism_empty() {
        let beads: Vec<PlanBead> = Vec::new();
        let parallelism = compute_parallelism(&beads);
        assert_eq!(parallelism, 0);
    }

    #[test]
    fn test_compute_parallelism_single() {
        let beads = vec![create_test_bead("a", 1, vec![])];
        let parallelism = compute_parallelism(&beads);
        assert_eq!(parallelism, 1);
    }

    #[test]
    fn test_compute_parallelism_chain() {
        let beads = vec![
            create_test_bead("a", 1, vec![]),
            create_test_bead("b", 1, vec!["a"]),
            create_test_bead("c", 1, vec!["b"]),
        ];

        let parallelism = compute_parallelism(&beads);
        assert_eq!(parallelism, 1); // Chain has no parallelism
    }

    #[test]
    fn test_compute_parallelism_parallel() {
        let beads = vec![
            create_test_bead("a", 1, vec![]),
            create_test_bead("b", 1, vec![]),
            create_test_bead("c", 1, vec![]),
            create_test_bead("d", 1, vec!["a", "b", "c"]),
        ];

        let parallelism = compute_parallelism(&beads);
        assert_eq!(parallelism, 3); // a, b, c can run in parallel
    }

    #[test]
    fn test_apply_resolution_to_plan() {
        let mut plan = ExecutionPlan::new("test".to_string());
        plan.add_bead(create_test_bead("b", 1, vec!["a"])).expect("add");
        plan.add_bead(create_test_bead("a", 1, vec![])).expect("add");

        let result = apply_resolution_to_plan(&mut plan);
        assert!(result.is_ok());
        assert!(plan.validated);
        assert_eq!(plan.execution_order, vec!["a", "b"]);
    }

    #[test]
    fn test_apply_resolution_to_plan_with_cycle() {
        let mut plan = ExecutionPlan::new("test".to_string());
        plan.add_bead(create_test_bead("a", 1, vec!["b"])).expect("add");
        plan.add_bead(create_test_bead("b", 1, vec!["a"])).expect("add");

        let result = apply_resolution_to_plan(&mut plan);
        assert!(matches!(result, Err(PlanError::CircularDependency)));
        assert!(!plan.validated);
    }

    #[test]
    fn test_resolve_dependencies_multiple_phases() {
        let beads = vec![
            create_test_bead("p1-a", 1, vec![]),
            create_test_bead("p1-b", 1, vec![]),
            create_test_bead("p2-a", 2, vec!["p1-a"]),
            create_test_bead("p2-b", 2, vec!["p1-b"]),
            create_test_bead("p3-a", 3, vec!["p2-a", "p2-b"]),
        ];

        let result = resolve_dependencies(&beads);
        assert!(result.is_ok());
        let res = result.expect("ok");

        assert!(res.is_valid());
        assert_eq!(res.sorted.len(), 5);

        // Verify order constraints
        let pos = |id: &str| res.sorted.iter().position(|x| x == id).expect("found");

        assert!(pos("p1-a") < pos("p2-a"));
        assert!(pos("p1-b") < pos("p2-b"));
        assert!(pos("p2-a") < pos("p3-a"));
        assert!(pos("p2-b") < pos("p3-a"));
    }
}
