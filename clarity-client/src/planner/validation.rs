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
/// # Arguments
/// * `tasks` - All tasks in the graph
///
/// # Returns
/// Vector of cycle information with full paths
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
  let mut rec_stack: Vec<Uuid> = Vec::new();
  let mut in_stack: HashSet<Uuid> = HashSet::new();

  for &task_id in task_map.keys() {
    if !visited.contains(&task_id) {
      if let Some(cycle) = dfs_detect_cycle(
        task_id,
        &adj,
        &task_map,
        &mut visited,
        &mut rec_stack,
        &mut in_stack,
      ) {
        cycles.push(cycle);
      }
    }
  }

  cycles
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
