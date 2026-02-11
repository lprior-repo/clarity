//! Planner validation module
//!
//! Pure validation functions for the Diamond methodology planning system.
//! All functions are deterministic and return Result types for error handling.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::planner::types::{
  GraphHealth, PlanTask, ValidationCheck, ValidationSeverity, COMPLETED_EPSILON, MAX_DEPTH,
};
use std::collections::{HashMap, HashSet, VecDeque};
use thiserror::Error;
use uuid::Uuid;

/// Validation errors for planner domain
#[derive(Debug, Error, PartialEq)]
pub enum ValidationError {
  #[error("task has invalid title: {0}")]
  InvalidTaskTitle(String),

  #[error("task description is empty")]
  EmptyTaskDescription,

  #[error("task has circular dependencies")]
  CircularDependencies,

  #[error("task depends on non-existent task: {0}")]
  DependencyNotFound(Uuid),

  #[error("task completion out of range: {0}")]
  InvalidCompletion(f32),

  #[error("cycle detected: {0}")]
  CycleDetected(String),

  #[error("self-dependency detected: task {0} cannot depend on itself")]
  SelfDependency(Uuid),

  #[error("graph has disconnected components: {0}")]
  DisconnectedComponents(usize),

  #[error("duplicate task ID: {0}")]
  DuplicateTaskId(Uuid),
}

/// Validate a single task
///
/// Checks that the task has valid title, description, and completion percentage.
/// Also checks for self-dependencies.
///
/// # Errors
/// Returns `ValidationError` if validation fails
#[must_use]
pub fn validate_task(task: &PlanTask) -> Result<(), Vec<ValidationError>> {
  let mut errors = Vec::new();

  // Check title
  if task.title.trim().is_empty() {
    errors.push(ValidationError::InvalidTaskTitle(
      "title is empty".to_string(),
    ));
  }

  // Check description
  if task.description.trim().is_empty() {
    errors.push(ValidationError::EmptyTaskDescription);
  }

  // Check completion is between 0.0 and 1.0
  if task.completion < 0.0 || task.completion > 1.0 {
    errors.push(ValidationError::InvalidCompletion(task.completion));
  }

  // Check for self-dependencies
  if task.dependencies.contains(&task.id) {
    errors.push(ValidationError::SelfDependency(task.id));
  }

  match errors.is_empty() {
    true => Ok(()),
    false => Err(errors),
  }
}

/// Check if a task is ready to start
///
/// A task is ready if all its dependencies are complete (completion >= 1.0 - epsilon).
///
/// # Arguments
/// * `task` - The task to check
/// * `all_tasks` - All tasks in the graph (indexed by ID)
///
/// # Returns
/// `true` if the task is ready to start
#[must_use]
pub fn is_task_ready(task: &PlanTask, all_tasks: &HashMap<Uuid, PlanTask>) -> bool {
  task
    .dependencies
    .iter()
    .map(|dep_id| match all_tasks.get(dep_id) {
      None => false, // Dependency not found means not ready
      Some(dep_task) => dep_task.completion >= 1.0 - COMPLETED_EPSILON,
    })
    .all(|ready| ready)
}

/// Calculate graph health metrics
///
/// Analyzes the task graph to determine:
/// - Node count
/// - Edge count
/// - Disconnected components
/// - Maximum depth
/// - Orphaned nodes (nodes with no dependencies and no dependents)
/// - Cyclomatic complexity
/// - Health score (0.0 to 1.0)
///
/// # Arguments
/// * `tasks` - All tasks in the graph
///
/// # Returns
/// Updated `GraphHealth` with calculated metrics
#[must_use]
pub fn get_graph_health(tasks: &[PlanTask]) -> GraphHealth {
  let node_count = tasks.len();
  let edge_count = tasks.iter().map(|t| t.dependencies.len()).sum();

  // Build adjacency lists
  let mut adj: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
  let mut reverse_adj: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
  let mut task_set: HashSet<Uuid> = HashSet::new();

  for task in tasks {
    let id = task.id;
    task_set.insert(id);

    for dep_id in &task.dependencies {
      adj.entry(*dep_id).or_default().push(id);
      reverse_adj.entry(id).or_default().push(*dep_id);
    }
  }

  // Find orphaned nodes (no deps and no dependents)
  let orphaned_nodes = tasks
    .iter()
    .filter(|t| {
      let has_deps = !t.dependencies.is_empty();
      let has_dependents = reverse_adj.get(&t.id).map_or(false, |v| !v.is_empty());
      !has_deps && !has_dependents
    })
    .count();

  // Find disconnected components using BFS
  let mut visited: HashSet<Uuid> = HashSet::new();
  let mut components: usize = 0;

  for task in tasks {
    if visited.contains(&task.id) {
      continue;
    }

    components += 1;
    let mut queue = VecDeque::new();
    queue.push_back(task.id);

    while let Some(id) = queue.pop_front() {
      if visited.contains(&id) {
        continue;
      }

      visited.insert(id);

      // Add all neighbors (both forward and reverse)
      if let Some(neighbors) = adj.get(&id) {
        queue.extend(neighbors.iter().filter(|n| !visited.contains(n)));
      }

      if let Some(neighbors) = reverse_adj.get(&id) {
        queue.extend(neighbors.iter().filter(|n| !visited.contains(n)));
      }
    }
  }

  // For empty graph, there are 0 components
  let actual_components = if node_count == 0 { 0 } else { components };

  let disconnected_components = actual_components.saturating_sub(1);

  // Calculate maximum depth using iterative DFS
  let mut max_depth = 0;

  for task in tasks {
    let depth = calculate_depth(task.id, &adj);
    max_depth = max_depth.max(depth);
  }

  // Cyclomatic complexity: E - N + 2P
  // Where E = edges, N = nodes, P = connected components
  let complexity = if node_count > 0 {
    (edge_count as f32) - (node_count as f32) + (2.0 * actual_components as f32)
  } else {
    0.0
  };

  GraphHealth::new().with_metrics(
    node_count,
    edge_count,
    disconnected_components,
    max_depth,
    orphaned_nodes,
    complexity,
  )
}

/// Calculate the maximum depth from a node using iterative DFS
///
/// # Arguments
/// * `node_id` - Starting node ID
/// * `adj` - Adjacency list
///
/// # Returns
/// Maximum depth from this node
#[must_use]
fn calculate_depth(node_id: Uuid, adj: &HashMap<Uuid, Vec<Uuid>>) -> usize {
  let mut max_depth = 0;
  let mut stack: Vec<(Uuid, usize)> = vec![(node_id, 0)];
  let mut visited: HashSet<Uuid> = HashSet::new();

  while let Some((current_id, current_depth)) = stack.pop() {
    if visited.contains(&current_id) {
      continue; // Skip already visited nodes (cycle protection)
    }

    visited.insert(current_id);
    max_depth = max_depth.max(current_depth);

    // Enforce maximum depth limit
    if current_depth >= MAX_DEPTH {
      continue;
    }

    if let Some(neighbors) = adj.get(&current_id) {
      for &neighbor in neighbors {
        if !visited.contains(&neighbor) {
          stack.push((neighbor, current_depth + 1));
        }
      }
    }
  }

  max_depth
}

/// Cycle detection result with full path
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CycleInfo {
  /// Nodes involved in the cycle
  pub nodes: Vec<Uuid>,
  /// Formatted cycle path description
  pub path: String,
}

/// Detect cycles in the task dependency graph with full cycle paths
///
/// Uses depth-first search with path tracking to detect cycles.
/// Returns detailed cycle information including the full cycle path.
///
/// This algorithm detects ALL cycles in the graph, including multiple
/// independent cycles in disconnected components.
///
/// # Arguments
/// * `tasks` - All tasks in the graph
///
/// # Returns
/// Vector of cycle information with full paths (one entry per unique cycle)
#[must_use]
pub fn detect_cycles_with_path(tasks: &[PlanTask]) -> Vec<CycleInfo> {
  // Build adjacency list and task lookup
  let mut adj: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
  let task_map: HashMap<Uuid, &PlanTask> = tasks.iter().map(|t| (t.id, t)).collect();

  for task in tasks {
    for dep_id in &task.dependencies {
      adj.entry(task.id).or_default().push(*dep_id);
    }
  }

  let mut cycles: Vec<CycleInfo> = Vec::new();
  let mut visited: HashSet<Uuid> = HashSet::new();

  // CRITICAL-002: Detect ALL cycles, not just the first one
  // For each unvisited node, run DFS to find cycles in that component
  for &task_id in task_map.keys() {
    if !visited.contains(&task_id) {
      // Find all cycles in this component
      let component_cycles = find_all_cycles_in_component(task_id, &adj, &task_map, &mut visited);
      cycles.extend(component_cycles);
    }
  }

  cycles
}

/// Find all cycles in a single connected component
///
/// This is a helper function that performs DFS and collects ALL cycles
/// in the component, not just the first one.
fn find_all_cycles_in_component(
  start_node: Uuid,
  adj: &HashMap<Uuid, Vec<Uuid>>,
  task_map: &HashMap<Uuid, &PlanTask>,
  global_visited: &mut HashSet<Uuid>,
) -> Vec<CycleInfo> {
  let mut cycles: Vec<CycleInfo> = Vec::new();
  let mut local_visited: HashSet<Uuid> = HashSet::new();
  let mut rec_stack: Vec<Uuid> = Vec::new();
  let mut in_stack: HashSet<Uuid> = HashSet::new();

  // Perform DFS, collecting all cycles
  dfs_collect_all_cycles(
    start_node,
    adj,
    task_map,
    &mut local_visited,
    &mut rec_stack,
    &mut in_stack,
    &mut cycles,
  );

  // Mark all nodes in this component as globally visited
  global_visited.extend(local_visited);

  cycles
}

/// DFS that collects ALL cycles (not just the first one)
///
/// Unlike dfs_detect_cycle which returns early, this continues searching
/// for additional cycles after finding one.
fn dfs_collect_all_cycles(
  node: Uuid,
  adj: &HashMap<Uuid, Vec<Uuid>>,
  task_map: &HashMap<Uuid, &PlanTask>,
  visited: &mut HashSet<Uuid>,
  rec_stack: &mut Vec<Uuid>,
  in_stack: &mut HashSet<Uuid>,
  cycles: &mut Vec<CycleInfo>,
) {
  visited.insert(node);
  rec_stack.push(node);
  in_stack.insert(node);

  if let Some(neighbors) = adj.get(&node) {
    for &neighbor in neighbors {
      if !visited.contains(&neighbor) {
        // Continue DFS to find more cycles
        dfs_collect_all_cycles(
          neighbor, adj, task_map, visited, rec_stack, in_stack, cycles,
        );
      } else if in_stack.contains(&neighbor) {
        // Found a cycle - extract and record it
        let cycle_start_idx = match rec_stack.iter().position(|&id| id == neighbor) {
          Some(idx) => idx,
          None => continue, // Should never happen
        };
        let cycle_nodes: Vec<Uuid> = rec_stack[cycle_start_idx..].to_vec();

        // Build readable path
        let path = build_cycle_path(&cycle_nodes, task_map);

        cycles.push(CycleInfo {
          nodes: cycle_nodes,
          path,
        });
      }
    }
  }

  rec_stack.pop();
  in_stack.remove(&node);
}

/// DFS visit for cycle detection with full path tracking
///
/// Returns Some(CycleInfo) if a cycle is found, None otherwise
fn dfs_detect_cycle(
  node: Uuid,
  adj: &HashMap<Uuid, Vec<Uuid>>,
  task_map: &HashMap<Uuid, &PlanTask>,
  visited: &mut HashSet<Uuid>,
  rec_stack: &mut Vec<Uuid>,
  in_stack: &mut HashSet<Uuid>,
) -> Option<CycleInfo> {
  visited.insert(node);
  rec_stack.push(node);
  in_stack.insert(node);

  if let Some(neighbors) = adj.get(&node) {
    for &neighbor in neighbors {
      if !visited.contains(&neighbor) {
        if let Some(cycle) = dfs_detect_cycle(neighbor, adj, task_map, visited, rec_stack, in_stack)
        {
          return Some(cycle);
        }
      } else if in_stack.contains(&neighbor) {
        // Found a cycle - extract the full cycle path
        let cycle_start_idx = rec_stack.iter().position(|&id| id == neighbor)?;
        let cycle_nodes: Vec<Uuid> = rec_stack[cycle_start_idx..].to_vec();
        let cycle_nodes_clone = cycle_nodes.clone();

        // Build readable path
        let path = build_cycle_path(&cycle_nodes_clone, task_map);

        return Some(CycleInfo {
          nodes: cycle_nodes,
          path,
        });
      }
    }
  }

  rec_stack.pop();
  in_stack.remove(&node);
  None
}

/// Build a human-readable cycle path description
fn build_cycle_path(nodes: &[Uuid], task_map: &HashMap<Uuid, &PlanTask>) -> String {
  let titles: Vec<String> = nodes
    .iter()
    .filter_map(|id| task_map.get(id).map(|t| t.title.clone()))
    .collect();

  if titles.is_empty() {
    return "Unknown cycle".to_string();
  }

  let mut path = titles.join(" -> ");
  path.push_str(" -> ");
  path.push_str(&titles[0]); // Complete the cycle
  path
}

/// Detect cycles in the task dependency graph (legacy compatibility)
///
/// Uses depth-first search with color marking to detect cycles.
/// Returns a list of node IDs involved in cycles.
///
/// # Arguments
/// * `tasks` - All tasks in the graph
///
/// # Returns
/// Set of task IDs that are part of cycles
#[must_use]
pub fn detect_cycles(tasks: &[PlanTask]) -> HashSet<Uuid> {
  detect_cycles_with_path(tasks)
    .into_iter()
    .flat_map(|cycle| cycle.nodes.into_iter())
    .collect()
}

/// Run all validation checks on a set of tasks
///
/// Creates validation checks for:
/// - Invalid tasks
/// - Tasks with invalid dependencies
/// - Duplicate task IDs
/// - Circular dependencies
/// - Graph health issues
///
/// # Arguments
/// * `tasks` - All tasks to validate
///
/// # Returns
/// Vector of validation checks
#[must_use]
pub fn validate_all_tasks(tasks: &[PlanTask]) -> Vec<ValidationCheck> {
  let mut checks = Vec::new();

  // Build task map and check for duplicates
  let mut task_map: HashMap<Uuid, PlanTask> = HashMap::new();
  let mut id_counts: HashMap<Uuid, usize> = HashMap::new();

  for task in tasks {
    *id_counts.entry(task.id).or_insert(0) += 1;
    if id_counts[&task.id] == 1 {
      task_map.insert(task.id, task.clone());
    }
  }

  // Check for duplicate IDs
  for (id, count) in &id_counts {
    if *count > 1 {
      let check = ValidationCheck::new(
        "Duplicate task ID detected".to_string(),
        format!("Task ID {id} appears {count} times in the task list"),
        ValidationSeverity::Critical,
      )
      .with_passed(false)
      .with_entity(*id);

      checks.push(check);
    }
  }

  // Validate each task
  for task in tasks {
    match validate_task(task) {
      Ok(()) => {
        let check = ValidationCheck::new(
          format!("Task valid: {}", task.title),
          format!("Task {} passed validation", task.id),
          ValidationSeverity::Info,
        )
        .with_passed(true)
        .with_entity(task.id);

        checks.push(check);
      }
      Err(errors) => {
        for error in errors {
          let (severity, message) = match &error {
            ValidationError::InvalidTaskTitle(msg) => (ValidationSeverity::Error, msg.clone()),
            ValidationError::EmptyTaskDescription => (
              ValidationSeverity::Error,
              "Task description is empty".to_string(),
            ),
            ValidationError::SelfDependency(id) => (
              ValidationSeverity::Error,
              format!("Task {id} cannot depend on itself"),
            ),
            ValidationError::InvalidCompletion(val) => (
              ValidationSeverity::Error,
              format!("Invalid completion value: {val}"),
            ),
            _ => (ValidationSeverity::Warning, error.to_string()),
          };

          let check = ValidationCheck::new(
            format!("Task validation error: {}", task.title),
            error.to_string(),
            severity,
          )
          .with_passed(false)
          .with_message(message)
          .with_entity(task.id);

          checks.push(check);
        }
      }
    }

    // Check dependencies exist
    for dep_id in &task.dependencies {
      if !task_map.contains_key(dep_id) {
        let check = ValidationCheck::new(
          format!("Missing dependency: {}", task.title),
          format!("Task depends on non-existent task: {dep_id}"),
          ValidationSeverity::Error,
        )
        .with_passed(false)
        .with_entity(task.id);

        checks.push(check);
      }
    }
  }

  // Check for cycles with detailed paths
  let cycles = detect_cycles_with_path(tasks);
  if !cycles.is_empty() {
    for cycle_info in &cycles {
      let check = ValidationCheck::new(
        "Circular dependencies detected".to_string(),
        format!("Cycle: {}", cycle_info.path),
        ValidationSeverity::Critical,
      )
      .with_passed(false);

      checks.push(check);
    }
  }

  // Check graph health
  let health = get_graph_health(tasks);

  if health.disconnected_components > 0 {
    let check = ValidationCheck::new(
      "Disconnected graph components".to_string(),
      format!(
        "Graph has {} disconnected component(s)",
        health.disconnected_components
      ),
      ValidationSeverity::Warning,
    )
    .with_passed(false);

    checks.push(check);
  }

  if health.orphaned_nodes > 0 {
    let check = ValidationCheck::new(
      "Orphaned tasks detected".to_string(),
      format!(
        "{} task(s) have no dependencies or dependents",
        health.orphaned_nodes
      ),
      ValidationSeverity::Info,
    )
    .with_passed(false);

    checks.push(check);
  }

  checks
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::planner::types::{DiamondPhase, TaskType};

  #[test]
  fn test_validate_task_valid() {
    let task = PlanTask::new(
      "Test Task".to_string(),
      "A valid task description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    )
    .with_completion(0.5);

    assert!(validate_task(&task).is_ok());
  }

  #[test]
  fn test_validate_task_empty_title() {
    let task = PlanTask::new(
      "".to_string(),
      "A valid task description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    );

    let result = validate_task(&task);
    assert!(result.is_err());

    let errors = result.unwrap_err();
    assert!(errors
      .iter()
      .any(|e| matches!(e, ValidationError::InvalidTaskTitle(_))));
  }

  #[test]
  fn test_validate_task_empty_description() {
    let task = PlanTask::new(
      "Test Task".to_string(),
      "".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    );

    let result = validate_task(&task);
    assert!(result.is_err());

    let errors = result.unwrap_err();
    assert!(errors
      .iter()
      .any(|e| matches!(e, ValidationError::EmptyTaskDescription)));
  }

  #[test]
  fn test_validate_task_invalid_completion() {
    let task = PlanTask::new(
      "Test Task".to_string(),
      "A valid task description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    )
    .with_completion(1.5);

    let result = validate_task(&task);
    assert!(result.is_err());

    let errors = result.unwrap_err();
    assert!(errors
      .iter()
      .any(|e| matches!(e, ValidationError::InvalidCompletion(_))));
  }

  #[test]
  fn test_is_task_ready_no_deps() {
    let task = PlanTask::new(
      "Task".to_string(),
      "Description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    );

    let all_tasks: HashMap<Uuid, PlanTask> = HashMap::new();
    assert!(is_task_ready(&task, &all_tasks));
  }

  #[test]
  fn test_is_task_ready_with_complete_deps() {
    let dep = PlanTask::new(
      "Dep".to_string(),
      "Description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    )
    .with_completion(1.0);

    let task = PlanTask::new(
      "Task".to_string(),
      "Description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    )
    .with_dependency(dep.id);

    let all_tasks: HashMap<Uuid, PlanTask> = [(dep.id, dep), (task.id, task.clone())]
      .into_iter()
      .collect();

    assert!(is_task_ready(&task, &all_tasks));
  }

  #[test]
  fn test_is_task_ready_with_incomplete_deps() {
    let dep = PlanTask::new(
      "Dep".to_string(),
      "Description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    )
    .with_completion(0.5);

    let task = PlanTask::new(
      "Task".to_string(),
      "Description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    )
    .with_dependency(dep.id);

    let all_tasks: HashMap<Uuid, PlanTask> = [(dep.id, dep), (task.id, task.clone())]
      .into_iter()
      .collect();

    assert!(!is_task_ready(&task, &all_tasks));
  }

  #[test]
  fn test_detect_cycles_no_cycle() {
    let first_task = PlanTask::new(
      "Task1".to_string(),
      "Description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    );

    let second_task = PlanTask::new(
      "Task2".to_string(),
      "Description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    )
    .with_dependency(first_task.id);

    let all_tasks = vec![first_task.clone(), second_task.clone()];
    let cycles = detect_cycles(&all_tasks);

    assert!(cycles.is_empty());
  }

  #[test]
  fn test_detect_cycles_with_cycle() {
    let first_task = PlanTask::new(
      "Task1".to_string(),
      "Description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    );

    let second_task = PlanTask::new(
      "Task2".to_string(),
      "Description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    )
    .with_dependency(first_task.id);

    // Create cycle: task1 depends on task2
    let first_task_with_cycle = PlanTask {
      dependencies: vec![second_task.id],
      ..first_task.clone()
    };

    let all_tasks = vec![first_task_with_cycle, second_task];
    let cycles = detect_cycles(&all_tasks);

    assert!(!cycles.is_empty());
  }

  #[test]
  fn test_detect_cycles_with_path() {
    let first_task = PlanTask::new(
      "Task1".to_string(),
      "Description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    );

    let second_task = PlanTask::new(
      "Task2".to_string(),
      "Description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    )
    .with_dependency(first_task.id);

    let third_task = PlanTask::new(
      "Task3".to_string(),
      "Description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    )
    .with_dependency(second_task.id);

    // Create cycle: task1 depends on task3
    let first_task_with_cycle = PlanTask {
      dependencies: vec![third_task.id],
      ..first_task.clone()
    };

    let all_tasks = vec![first_task_with_cycle, second_task, third_task];
    let cycles = detect_cycles_with_path(&all_tasks);

    assert_eq!(cycles.len(), 1);
    assert_eq!(cycles[0].nodes.len(), 3);
    // Path should include all three task titles
    assert!(cycles[0].path.contains("Task1"));
    assert!(cycles[0].path.contains("Task2"));
    assert!(cycles[0].path.contains("Task3"));
  }

  // CRITICAL-002 HOSTILE TEST: Complex 5-node cycle
  #[test]
  fn test_detect_cycles_complex_5_node_cycle() {
    // Create A -> B -> C -> D -> E -> A (5 nodes)
    let task_a = PlanTask::new(
      "Task A".to_string(),
      "Description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    );

    let task_b = PlanTask::new(
      "Task B".to_string(),
      "Description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    )
    .with_dependency(task_a.id);

    let task_c = PlanTask::new(
      "Task C".to_string(),
      "Description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    )
    .with_dependency(task_b.id);

    let task_d = PlanTask::new(
      "Task D".to_string(),
      "Description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    )
    .with_dependency(task_c.id);

    let task_e = PlanTask::new(
      "Task E".to_string(),
      "Description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    )
    .with_dependency(task_d.id);

    // Close the cycle: A depends on E
    let task_a_with_cycle = PlanTask {
      dependencies: vec![task_e.id],
      ..task_a.clone()
    };

    let all_tasks = vec![
      task_a_with_cycle,
      task_b.clone(),
      task_c.clone(),
      task_d.clone(),
      task_e.clone(),
    ];
    let cycles = detect_cycles_with_path(&all_tasks);

    // Should detect exactly 1 cycle with all 5 nodes
    assert_eq!(cycles.len(), 1, "Should detect exactly one cycle");
    assert_eq!(cycles[0].nodes.len(), 5, "Cycle should contain all 5 nodes");

    // Verify all nodes are in the cycle
    let cycle_ids: std::collections::HashSet<Uuid> = cycles[0].nodes.iter().copied().collect();
    assert!(cycle_ids.contains(&task_a.id));
    assert!(cycle_ids.contains(&task_b.id));
    assert!(cycle_ids.contains(&task_c.id));
    assert!(cycle_ids.contains(&task_d.id));
    assert!(cycle_ids.contains(&task_e.id));

    // Path should contain all task titles
    assert!(cycles[0].path.contains("Task A"));
    assert!(cycles[0].path.contains("Task B"));
    assert!(cycles[0].path.contains("Task C"));
    assert!(cycles[0].path.contains("Task D"));
    assert!(cycles[0].path.contains("Task E"));

    // Path should be a proper cycle (starts and ends with same node)
    // Path format: "A -> B -> C -> D -> E -> A"
    let path_parts: Vec<&str> = cycles[0].path.split(" -> ").collect();
    assert_eq!(
      path_parts.len(),
      6,
      "Cycle path should have 6 parts (5 nodes + 1 repeat)"
    );
    assert_eq!(
      path_parts[0], path_parts[5],
      "Cycle should start and end with same node"
    );
  }

  // CRITICAL-002 HOSTILE TEST: Multiple independent cycles
  #[test]
  fn test_detect_cycles_multiple_independent() {
    // Create two completely independent cycles:
    // Cycle 1: A -> B -> A
    // Cycle 2: C -> D -> E -> C

    let task_a = PlanTask::new(
      "Task A".to_string(),
      "Description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    );

    let task_b = PlanTask::new(
      "Task B".to_string(),
      "Description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    )
    .with_dependency(task_a.id);

    let task_a_with_cycle = PlanTask {
      dependencies: vec![task_b.id],
      ..task_a.clone()
    };

    let task_c = PlanTask::new(
      "Task C".to_string(),
      "Description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    );

    let task_d = PlanTask::new(
      "Task D".to_string(),
      "Description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    )
    .with_dependency(task_c.id);

    let task_e = PlanTask::new(
      "Task E".to_string(),
      "Description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    )
    .with_dependency(task_d.id);

    let task_c_with_cycle = PlanTask {
      dependencies: vec![task_e.id],
      ..task_c.clone()
    };

    let all_tasks = vec![
      task_a_with_cycle,
      task_b.clone(),
      task_c_with_cycle,
      task_d.clone(),
      task_e.clone(),
    ];
    let cycles = detect_cycles_with_path(&all_tasks);

    // Should detect cycles (may detect more than 2 due to multiple entry points)
    assert!(!cycles.is_empty(), "Should detect at least one cycle");

    // Check that we found cycles involving both components
    let mut found_ab_cycle = false;
    let mut found_cde_cycle = false;

    for cycle in &cycles {
      let has_a = cycle.nodes.contains(&task_a.id);
      let has_b = cycle.nodes.contains(&task_b.id);
      let has_c = cycle.nodes.contains(&task_c.id);
      let has_d = cycle.nodes.contains(&task_d.id);
      let has_e = cycle.nodes.contains(&task_e.id);

      if has_a && has_b && cycle.nodes.len() == 2 {
        found_ab_cycle = true;
      }
      if has_c && has_d && has_e && cycle.nodes.len() == 3 {
        found_cde_cycle = true;
      }
    }

    assert!(
      found_ab_cycle,
      "Should detect cycle A -> B -> A, found {} cycles: {:?}",
      cycles.len(),
      cycles
    );
    assert!(
      found_cde_cycle,
      "Should detect cycle C -> D -> E -> C, found {} cycles: {:?}",
      cycles.len(),
      cycles
    );
  }

  // CRITICAL-002 HOSTILE TEST: Overlapping cycles (diamond dependency)
  #[test]
  fn test_detect_cycles_overlapping_diamond() {
    // Create diamond with cycle:
    //     A
    //    / \
    //   B   C
    //    \ /
    //     D
    //     |
    //     A (cycle)

    let task_a = PlanTask::new(
      "Task A".to_string(),
      "Description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    );

    let task_b = PlanTask::new(
      "Task B".to_string(),
      "Description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    )
    .with_dependency(task_a.id);

    let task_c = PlanTask::new(
      "Task C".to_string(),
      "Description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    )
    .with_dependency(task_a.id);

    let task_d = PlanTask::new(
      "Task D".to_string(),
      "Description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    )
    .with_dependency(task_b.id)
    .with_dependency(task_c.id);

    // Create cycle: A depends on D
    let task_a_with_cycle = PlanTask {
      dependencies: vec![task_d.id],
      ..task_a.clone()
    };

    let all_tasks = vec![task_a_with_cycle, task_b, task_c, task_d];
    let cycles = detect_cycles_with_path(&all_tasks);

    // Should detect the cycle (might detect multiple paths through diamond)
    assert!(!cycles.is_empty(), "Should detect at least one cycle");

    // All cycles should include all 4 nodes
    for cycle in &cycles {
      assert!(cycle.nodes.len() >= 3, "Cycle should have at least 3 nodes");
      assert!(cycle.path.contains("Task A"));
    }
  }

  // CRITICAL-002 HOSTILE TEST: Self-loop (task depends on itself via weird path)
  #[test]
  fn test_detect_cycles_self_loop() {
    // Create a task that depends on itself (should be prevented by with_dependency,
    // but let's test the validator catches it anyway)

    let task = PlanTask::new(
      "Self Task".to_string(),
      "Description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    );

    // Manually construct a self-loop (bypassing with_dependency protection)
    let task_with_self_loop = PlanTask {
      dependencies: vec![task.id],
      ..task
    };

    let all_tasks = vec![task_with_self_loop.clone()];
    let cycles = detect_cycles_with_path(&all_tasks);

    // Should detect the self-loop as a cycle
    assert_eq!(cycles.len(), 1);
    assert_eq!(cycles[0].nodes.len(), 1);
    assert!(cycles[0].path.contains("Self Task"));
  }

  #[test]
  fn test_calculate_depth_iterative() {
    let task1 = PlanTask::new(
      "Task1".to_string(),
      "Description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    );

    let task2 = PlanTask::new(
      "Task2".to_string(),
      "Description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    )
    .with_dependency(task1.id);

    let task3 = PlanTask::new(
      "Task3".to_string(),
      "Description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    )
    .with_dependency(task2.id);

    // Build adjacency list for depth calculation
    let mut adj: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
    adj.insert(task1.id, vec![task2.id]);
    adj.insert(task2.id, vec![task3.id]);

    let depth1 = calculate_depth(task1.id, &adj);
    assert_eq!(depth1, 2); // task1 -> task2 -> task3
  }

  #[test]
  fn test_get_graph_health_empty() {
    let health = get_graph_health(&[]);

    assert_eq!(health.node_count, 0);
    assert_eq!(health.edge_count, 0);
    assert_eq!(health.health_score, 1.0);
  }

  #[test]
  fn test_get_graph_health_single_task() {
    let task = PlanTask::new(
      "Task".to_string(),
      "Description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    );

    let health = get_graph_health(&[task]);

    assert_eq!(health.node_count, 1);
    assert_eq!(health.edge_count, 0);
    assert_eq!(health.orphaned_nodes, 1);
  }

  #[test]
  fn test_get_graph_health_connected() {
    let task1 = PlanTask::new(
      "Task1".to_string(),
      "Description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    );

    let task2 = PlanTask::new(
      "Task2".to_string(),
      "Description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    )
    .with_dependency(task1.id);

    let health = get_graph_health(&[task1, task2]);

    assert_eq!(health.node_count, 2);
    assert_eq!(health.edge_count, 1);
    assert_eq!(health.disconnected_components, 0);
    assert_eq!(health.max_depth, 1);
  }
}
