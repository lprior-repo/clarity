#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

//! Planning types for project management and task tracking
//!
//! Provides comprehensive types for managing plans, tasks, and dependencies.
//! All construction returns Result<T, PlanningError> - no unwraps, no panics.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;

/// Error types for planning operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlanningError {
  /// Tasks form a circular dependency
  #[serde(rename = "cyclic_dependency")]
  CyclicDependency { cycle: Vec<String> },

  /// Invalid task status transition
  #[serde(rename = "invalid_transition")]
  InvalidTransition {
    from: TaskStatus,
    to: TaskStatus,
    valid_transitions: Vec<TaskStatus>,
  },

  /// Duplicate task ID in plan
  #[serde(rename = "duplicate_id")]
  DuplicateId { id: String },

  /// Task depends on non-existent task
  #[serde(rename = "missing_dependency")]
  MissingDependency {
    task_id: String,
    dependency_id: String,
  },

  /// Validation error
  #[serde(rename = "validation")]
  Validation { field: String, reason: String },

  /// Self-dependency (task depends on itself)
  #[serde(rename = "self_dependency")]
  SelfDependency { task_id: String },
}

impl fmt::Display for PlanningError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::CyclicDependency { cycle } => {
        write!(f, "Cyclic dependency detected: {}", cycle.join(" -> "))
      }
      Self::InvalidTransition {
        from,
        to,
        valid_transitions,
      } => {
        write!(
          f,
          "Invalid transition from {:?} to {:?}. Valid transitions: {:?}",
          from, to, valid_transitions
        )
      }
      Self::DuplicateId { id } => write!(f, "Duplicate task ID: {id}"),
      Self::MissingDependency {
        task_id,
        dependency_id,
      } => write!(
        f,
        "Task {task_id} depends on non-existent task {dependency_id}"
      ),
      Self::Validation { field, reason } => {
        write!(f, "Validation failed for field '{field}': {reason}")
      }
      Self::SelfDependency { task_id } => {
        write!(f, "Task {task_id} cannot depend on itself")
      }
    }
  }
}

impl std::error::Error for PlanningError {}

/// Task status representing the state of a task
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
  /// Task is not yet started
  Todo,
  /// Task is currently being worked on
  InProgress,
  /// Task is completed
  Done,
  /// Task is blocked by dependencies
  Blocked,
}

impl TaskStatus {
  /// Get valid next states for this status
  #[must_use]
  pub const fn valid_transitions(&self) -> &[TaskStatus] {
    match self {
      Self::Todo => &[Self::InProgress, Self::Blocked],
      Self::InProgress => &[Self::Done, Self::Blocked],
      Self::Blocked => &[Self::Todo, Self::InProgress],
      Self::Done => &[], // No transitions from Done
    }
  }

  /// Check if a transition to another status is valid
  #[must_use]
  pub fn can_transition_to(&self, to: &TaskStatus) -> bool {
    self.valid_transitions().contains(to)
  }
}

/// Task priority levels (ordered from highest to lowest)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Priority {
  /// Critical priority (highest)
  P0,
  /// High priority
  P1,
  /// Medium priority
  P2,
  /// Low priority (lowest)
  P3,
}

impl fmt::Display for Priority {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::P0 => write!(f, "P0"),
      Self::P1 => write!(f, "P1"),
      Self::P2 => write!(f, "P2"),
      Self::P3 => write!(f, "P3"),
    }
  }
}

/// Represents a dependency between tasks
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskDependency {
  /// ID of the task that has the dependency
  #[serde(rename = "task_id")]
  pub task_id: String,

  /// ID of the task this task depends on
  #[serde(rename = "depends_on")]
  pub depends_on: String,
}

impl TaskDependency {
  /// Create a new task dependency
  ///
  /// # Errors
  /// Returns `PlanningError::SelfDependency` if task_id == depends_on
  pub fn new(task_id: String, depends_on: String) -> Result<Self, PlanningError> {
    if task_id == depends_on {
      return Err(PlanningError::SelfDependency { task_id });
    }

    Ok(Self {
      task_id,
      depends_on,
    })
  }
}

/// Represents a single task within a plan
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Task {
  /// Unique identifier for the task
  #[serde(rename = "id")]
  pub id: String,

  /// Task title
  #[serde(rename = "title")]
  pub title: String,

  /// Detailed description
  #[serde(rename = "description")]
  pub description: String,

  /// Current status
  #[serde(rename = "status")]
  pub status: TaskStatus,

  /// Priority level
  #[serde(rename = "priority")]
  pub priority: Priority,

  /// Due date in ISO 8601 format (optional)
  #[serde(rename = "due_date")]
  pub due_date: Option<String>,

  /// Time estimate in hours (optional)
  #[serde(rename = "estimate_hours")]
  pub estimate_hours: Option<f64>,

  /// Tags/labels for categorization
  #[serde(rename = "tags")]
  pub tags: Vec<String>,
}

impl Task {
  /// Create a new task with validation
  ///
  /// # Errors
  /// - Returns `PlanningError::Validation` if title is empty
  /// - Returns `PlanningError::Validation` if estimate is negative
  pub fn new(
    id: String,
    title: String,
    description: String,
    status: TaskStatus,
    priority: Priority,
    due_date: Option<String>,
    estimate_hours: Option<f64>,
    tags: Vec<String>,
  ) -> Result<Self, PlanningError> {
    let trimmed_title = title.trim();
    if trimmed_title.is_empty() {
      return Err(PlanningError::Validation {
        field: "title".to_string(),
        reason: "title cannot be empty".to_string(),
      });
    }

    if let Some(estimate) = estimate_hours {
      if estimate < 0.0 {
        return Err(PlanningError::Validation {
          field: "estimate_hours".to_string(),
          reason: "estimate cannot be negative".to_string(),
        });
      }
    }

    Ok(Self {
      id,
      title: trimmed_title.to_string(),
      description,
      status,
      priority,
      due_date,
      estimate_hours,
      tags,
    })
  }

  /// Transition to a new status
  ///
  /// # Errors
  /// Returns `PlanningError::InvalidTransition` if transition is invalid
  pub fn transition_to(&mut self, new_status: TaskStatus) -> Result<(), PlanningError> {
    if !self.status.can_transition_to(&new_status) {
      return Err(PlanningError::InvalidTransition {
        from: self.status,
        to: new_status,
        valid_transitions: self.status.valid_transitions().to_vec(),
      });
    }

    self.status = new_status;
    Ok(())
  }

  /// Check if this task is overdue (not Done and due_date is in the past)
  #[must_use]
  pub fn is_overdue(&self) -> bool {
    if self.status == TaskStatus::Done {
      return false;
    }

    match &self.due_date {
      None => false,
      Some(date_str) => {
        // Try to parse the ISO 8601 date and compare with current time
        match chrono::DateTime::parse_from_rfc3339(date_str) {
          Ok(due_date) => {
            let now: chrono::DateTime<chrono::Utc> = chrono::Utc::now();
            let due_date_utc: chrono::DateTime<chrono::Utc> =
              chrono::DateTime::from_naive_utc_and_offset(due_date.naive_utc(), chrono::Utc);
            due_date_utc < now
          }
          Err(_) => false, // Invalid date format - not considered overdue
        }
      }
    }
  }
}

/// Represents a plan with tasks and dependencies
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Plan {
  /// Plan title
  #[serde(rename = "title")]
  pub title: String,

  /// Plan description
  #[serde(rename = "description")]
  pub description: String,

  /// Tasks in the plan
  #[serde(rename = "tasks")]
  pub tasks: Vec<Task>,

  /// Dependencies between tasks
  #[serde(rename = "dependencies")]
  pub dependencies: Vec<TaskDependency>,
}

impl Plan {
  /// Create a new plan with validation
  ///
  /// # Errors
  /// - Returns `PlanningError::Validation` if title is empty
  /// - Returns `PlanningError::DuplicateId` if task IDs are not unique
  /// - Returns `PlanningError::MissingDependency` if dependency references non-existent task
  /// - Returns `PlanningError::CyclicDependency` if dependencies form a cycle
  pub fn new(
    title: String,
    description: String,
    tasks: Vec<Task>,
    dependencies: Vec<TaskDependency>,
  ) -> Result<Self, PlanningError> {
    let trimmed_title = title.trim();
    if trimmed_title.is_empty() {
      return Err(PlanningError::Validation {
        field: "title".to_string(),
        reason: "title cannot be empty".to_string(),
      });
    }

    // Check for duplicate task IDs
    let mut seen_ids = HashSet::new();
    for task in &tasks {
      if !seen_ids.insert(&task.id) {
        return Err(PlanningError::DuplicateId {
          id: task.id.clone(),
        });
      }
    }

    // Collect all task IDs for dependency validation
    let task_ids: HashSet<&String> = tasks.iter().map(|t| &t.id).collect();

    // Validate dependencies
    for dep in &dependencies {
      if !task_ids.contains(&dep.depends_on) {
        return Err(PlanningError::MissingDependency {
          task_id: dep.task_id.clone(),
          dependency_id: dep.depends_on.clone(),
        });
      }
    }

    // Check for cyclic dependencies
    let cycle = detect_cycle(&tasks, &dependencies);
    if let Some(cycle_ids) = cycle {
      return Err(PlanningError::CyclicDependency { cycle: cycle_ids });
    }

    Ok(Self {
      title: trimmed_title.to_string(),
      description,
      tasks,
      dependencies,
    })
  }

  /// Calculate completion percentage (0.0 to 100.0)
  #[must_use]
  pub fn completion_percentage(&self) -> f64 {
    if self.tasks.is_empty() {
      return 0.0;
    }

    let done_count = self
      .tasks
      .iter()
      .filter(|t| t.status == TaskStatus::Done)
      .count();

    (done_count as f64 / self.tasks.len() as f64) * 100.0
  }

  /// Get all blocked tasks
  #[must_use]
  pub fn blocked_tasks(&self) -> Vec<Task> {
    self
      .tasks
      .iter()
      .filter(|t| t.status == TaskStatus::Blocked)
      .cloned()
      .collect()
  }

  /// Get tasks that are ready to start (all dependencies satisfied)
  #[must_use]
  pub fn ready_tasks(&self) -> Vec<Task> {
    let mut ready = Vec::new();

    for task in &self.tasks {
      // Only Todo and InProgress tasks can be ready
      if !matches!(task.status, TaskStatus::Todo | TaskStatus::InProgress) {
        continue;
      }

      // Check if all dependencies are satisfied
      let dependencies_satisfied = self
        .dependencies
        .iter()
        .filter(|d| &d.task_id == &task.id)
        .all(|dep| {
          self
            .tasks
            .iter()
            .find(|t| t.id == dep.depends_on)
            .is_some_and(|dep_task| dep_task.status == TaskStatus::Done)
        });

      if dependencies_satisfied {
        ready.push(task.clone());
      }
    }

    ready
  }

  /// Get tasks in topological order (respecting dependencies)
  ///
  /// # Errors
  /// Returns `PlanningError::CyclicDependency` if cycle detected (shouldn't happen after construction)
  pub fn topological_order(&self) -> Result<Vec<Task>, PlanningError> {
    if self.tasks.is_empty() {
      return Ok(Vec::new());
    }

    // Build adjacency list and in-degree count
    let mut graph: HashMap<&String, Vec<&String>> = HashMap::new();
    let mut in_degree: HashMap<&String, usize> = HashMap::new();

    for task in &self.tasks {
      graph.insert(&task.id, Vec::new());
      in_degree.insert(&task.id, 0);
    }

    for dep in &self.dependencies {
      graph.entry(&dep.depends_on).or_default().push(&dep.task_id);
      *in_degree.entry(&dep.task_id).or_insert(0) += 1;
    }

    // Kahn's algorithm
    let mut queue: Vec<&String> = in_degree
      .iter()
      .filter(|(_, &degree)| degree == 0)
      .map(|(id, _)| *id)
      .collect();

    let mut result = Vec::new();
    let mut visited_count = 0;

    while let Some(task_id) = queue.pop() {
      if let Some(task) = self.tasks.iter().find(|t| &t.id == task_id) {
        result.push(task.clone());
        visited_count += 1;
      }

      if let Some(neighbors) = graph.get(task_id) {
        for neighbor in neighbors {
          if let Some(degree) = in_degree.get_mut(neighbor) {
            if *degree > 0 {
              *degree -= 1;
              if *degree == 0 {
                queue.push(neighbor);
              }
            }
          }
        }
      }
    }

    // Check for cycle
    if visited_count != self.tasks.len() {
      // This shouldn't happen if we validated on construction
      let cycle = detect_cycle(&self.tasks, &self.dependencies);
      return Err(PlanningError::CyclicDependency {
        cycle: cycle.unwrap_or_default(),
      });
    }

    Ok(result)
  }

  /// Calculate total estimate (sum of all task estimates)
  #[must_use]
  pub fn total_estimate(&self) -> f64 {
    self.tasks.iter().filter_map(|t| t.estimate_hours).sum()
  }

  /// Serialize plan to JSON
  ///
  /// # Errors
  /// Returns `PlanningError::Serialization` if JSON serialization fails
  pub fn to_json(&self) -> Result<String, PlanningError> {
    serde_json::to_string_pretty(self).map_err(|e| PlanningError::Validation {
      field: "serialization".to_string(),
      reason: format!("JSON serialization failed: {e}"),
    })
  }

  /// Deserialize plan from JSON
  ///
  /// # Errors
  /// Returns `PlanningError::Serialization` if JSON deserialization fails
  /// Returns other `PlanningError` variants if validation fails
  pub fn from_json(json: &str) -> Result<Self, PlanningError> {
    let plan: Self = serde_json::from_str(json).map_err(|e| PlanningError::Validation {
      field: "deserialization".to_string(),
      reason: format!("JSON deserialization failed: {e}"),
    })?;

    // Re-validate to ensure invariants
    Self::new(plan.title, plan.description, plan.tasks, plan.dependencies)
  }
}

/// Detect if there's a cycle in the dependency graph
fn detect_cycle(tasks: &[Task], dependencies: &[TaskDependency]) -> Option<Vec<String>> {
  if tasks.is_empty() {
    return None;
  }

  let mut graph: HashMap<&String, Vec<&String>> = HashMap::new();
  for task in tasks {
    graph.insert(&task.id, Vec::new());
  }

  for dep in dependencies {
    graph
      .entry(&dep.task_id)
      .or_insert_with(Vec::new)
      .push(&dep.depends_on);
  }

  let mut visited: HashSet<&String> = HashSet::new();
  let mut rec_stack: HashSet<&String> = HashSet::new();
  let mut path: Vec<String> = Vec::new();

  for task in tasks {
    if !visited.contains(&task.id) {
      if let Some(cycle) = dfs_cycle(&task.id, &graph, &mut visited, &mut rec_stack, &mut path) {
        return Some(cycle);
      }
    }
  }

  None
}

/// DFS helper for cycle detection
fn dfs_cycle<'a>(
  node: &'a String,
  graph: &HashMap<&'a String, Vec<&'a String>>,
  visited: &mut HashSet<&'a String>,
  rec_stack: &mut HashSet<&'a String>,
  path: &mut Vec<String>,
) -> Option<Vec<String>> {
  visited.insert(node);
  rec_stack.insert(node);
  path.push(node.clone());

  if let Some(neighbors) = graph.get(node) {
    for neighbor in neighbors {
      if !visited.contains(neighbor) {
        if let Some(cycle) = dfs_cycle(neighbor, graph, visited, rec_stack, path) {
          return Some(cycle);
        }
      } else if rec_stack.contains(neighbor) {
        // Found a cycle - extract the cycle from the path
        let cycle_start = path.iter().position(|p| *p == **neighbor).unwrap_or(0);
        let mut cycle = path[cycle_start..].to_vec();
        cycle.push((**neighbor).clone());
        return Some(cycle);
      }
    }
  }

  path.pop();
  rec_stack.remove(node);
  None
}

#[cfg(test)]
mod tests {
  use super::*;

  // Test 1: test_task_should_create_with_valid_fields
  #[test]
  fn test_task_should_create_with_valid_fields() {
    let result = Task::new(
      "task-1".to_string(),
      "Valid Task".to_string(),
      "Description".to_string(),
      TaskStatus::Todo,
      Priority::P1,
      None,
      Some(2.0),
      vec!["urgent".to_string()],
    );

    assert!(result.is_ok());
    let task = result.unwrap();
    assert_eq!(task.id, "task-1");
    assert_eq!(task.title, "Valid Task");
    assert_eq!(task.status, TaskStatus::Todo);
    assert_eq!(task.priority, Priority::P1);
    assert_eq!(task.estimate_hours, Some(2.0));
    assert_eq!(task.tags, vec!["urgent"]);
  }

  // Test 2: test_task_should_require_unique_id_in_plan
  #[test]
  fn test_task_should_require_unique_id_in_plan() {
    let task1 = Task::new(
      "task-1".to_string(),
      "Task 1".to_string(),
      "Desc 1".to_string(),
      TaskStatus::Todo,
      Priority::P1,
      None,
      None,
      vec![],
    )
    .unwrap();

    let task2 = Task::new(
      "task-1".to_string(),
      "Task 2".to_string(),
      "Desc 2".to_string(),
      TaskStatus::Todo,
      Priority::P2,
      None,
      None,
      vec![],
    )
    .unwrap();

    let result = Plan::new(
      "Test Plan".to_string(),
      "Description".to_string(),
      vec![task1, task2],
      vec![],
    );

    assert!(result.is_err());
    assert!(matches!(
      result,
      Err(PlanningError::DuplicateId { id }) if id == "task-1"
    ));
  }

  // Test 3: test_task_status_should_transition_from_todo_to_in_progress
  #[test]
  fn test_task_status_should_transition_from_todo_to_in_progress() {
    let mut task = Task::new(
      "task-1".to_string(),
      "Task".to_string(),
      "Desc".to_string(),
      TaskStatus::Todo,
      Priority::P1,
      None,
      None,
      vec![],
    )
    .unwrap();

    let result = task.transition_to(TaskStatus::InProgress);

    assert!(result.is_ok());
    assert_eq!(task.status, TaskStatus::InProgress);
  }

  // Test 4: test_task_status_should_not_transition_from_done_to_todo
  #[test]
  fn test_task_status_should_not_transition_from_done_to_todo() {
    let mut task = Task::new(
      "task-1".to_string(),
      "Task".to_string(),
      "Desc".to_string(),
      TaskStatus::Done,
      Priority::P1,
      None,
      None,
      vec![],
    )
    .unwrap();

    let result = task.transition_to(TaskStatus::Todo);

    assert!(result.is_err());
    assert!(matches!(
      result,
      Err(PlanningError::InvalidTransition { from, to, .. })
      if from == TaskStatus::Done && to == TaskStatus::Todo
    ));
    assert_eq!(task.status, TaskStatus::Done); // Status unchanged
  }

  // Test 5: test_task_status_should_support_all_valid_transitions
  #[test]
  fn test_task_status_should_support_all_valid_transitions() {
    // Todo -> InProgress: Ok
    let mut task = Task::new(
      "task-1".to_string(),
      "Task".to_string(),
      "Desc".to_string(),
      TaskStatus::Todo,
      Priority::P1,
      None,
      None,
      vec![],
    )
    .unwrap();
    assert!(task.transition_to(TaskStatus::InProgress).is_ok());

    // Todo -> Blocked: Ok
    let mut task = Task::new(
      "task-2".to_string(),
      "Task".to_string(),
      "Desc".to_string(),
      TaskStatus::Todo,
      Priority::P1,
      None,
      None,
      vec![],
    )
    .unwrap();
    assert!(task.transition_to(TaskStatus::Blocked).is_ok());

    // InProgress -> Done: Ok
    let mut task = Task::new(
      "task-3".to_string(),
      "Task".to_string(),
      "Desc".to_string(),
      TaskStatus::InProgress,
      Priority::P1,
      None,
      None,
      vec![],
    )
    .unwrap();
    assert!(task.transition_to(TaskStatus::Done).is_ok());

    // InProgress -> Blocked: Ok
    let mut task = Task::new(
      "task-4".to_string(),
      "Task".to_string(),
      "Desc".to_string(),
      TaskStatus::InProgress,
      Priority::P1,
      None,
      None,
      vec![],
    )
    .unwrap();
    assert!(task.transition_to(TaskStatus::Blocked).is_ok());

    // Blocked -> Todo: Ok
    let mut task = Task::new(
      "task-5".to_string(),
      "Task".to_string(),
      "Desc".to_string(),
      TaskStatus::Blocked,
      Priority::P1,
      None,
      None,
      vec![],
    )
    .unwrap();
    assert!(task.transition_to(TaskStatus::Todo).is_ok());

    // Blocked -> InProgress: Ok
    let mut task = Task::new(
      "task-6".to_string(),
      "Task".to_string(),
      "Desc".to_string(),
      TaskStatus::Blocked,
      Priority::P1,
      None,
      None,
      vec![],
    )
    .unwrap();
    assert!(task.transition_to(TaskStatus::InProgress).is_ok());

    // Done -> Todo: Err
    let mut task = Task::new(
      "task-7".to_string(),
      "Task".to_string(),
      "Desc".to_string(),
      TaskStatus::Done,
      Priority::P1,
      None,
      None,
      vec![],
    )
    .unwrap();
    assert!(task.transition_to(TaskStatus::Todo).is_err());
  }

  // Test 6: test_plan_should_detect_cyclic_dependencies
  #[test]
  fn test_plan_should_detect_cyclic_dependencies() {
    let task_a = Task::new(
      "A".to_string(),
      "Task A".to_string(),
      "Desc A".to_string(),
      TaskStatus::Todo,
      Priority::P1,
      None,
      None,
      vec![],
    )
    .unwrap();

    let task_b = Task::new(
      "B".to_string(),
      "Task B".to_string(),
      "Desc B".to_string(),
      TaskStatus::Todo,
      Priority::P1,
      None,
      None,
      vec![],
    )
    .unwrap();

    let task_c = Task::new(
      "C".to_string(),
      "Task C".to_string(),
      "Desc C".to_string(),
      TaskStatus::Todo,
      Priority::P1,
      None,
      None,
      vec![],
    )
    .unwrap();

    let dep_a_b = TaskDependency::new("A".to_string(), "B".to_string()).unwrap();
    let dep_b_c = TaskDependency::new("B".to_string(), "C".to_string()).unwrap();
    let dep_c_a = TaskDependency::new("C".to_string(), "A".to_string()).unwrap();

    let result = Plan::new(
      "Test Plan".to_string(),
      "Description".to_string(),
      vec![task_a, task_b, task_c],
      vec![dep_a_b, dep_b_c, dep_c_a],
    );

    assert!(result.is_err());
    assert!(matches!(
      result,
      Err(PlanningError::CyclicDependency { .. })
    ));
  }

  // Test 7: test_plan_should_allow_valid_dependency_chain
  #[test]
  fn test_plan_should_allow_valid_dependency_chain() {
    let task_a = Task::new(
      "A".to_string(),
      "Task A".to_string(),
      "Desc A".to_string(),
      TaskStatus::Todo,
      Priority::P1,
      None,
      None,
      vec![],
    )
    .unwrap();

    let task_b = Task::new(
      "B".to_string(),
      "Task B".to_string(),
      "Desc B".to_string(),
      TaskStatus::Todo,
      Priority::P1,
      None,
      None,
      vec![],
    )
    .unwrap();

    let task_c = Task::new(
      "C".to_string(),
      "Task C".to_string(),
      "Desc C".to_string(),
      TaskStatus::Todo,
      Priority::P1,
      None,
      None,
      vec![],
    )
    .unwrap();

    let dep_a_b = TaskDependency::new("A".to_string(), "B".to_string()).unwrap();
    let dep_b_c = TaskDependency::new("B".to_string(), "C".to_string()).unwrap();

    let result = Plan::new(
      "Test Plan".to_string(),
      "Description".to_string(),
      vec![task_a, task_b, task_c],
      vec![dep_a_b, dep_b_c],
    );

    assert!(result.is_ok());
  }

  // Test 8: test_plan_should_reject_dependency_on_non_existent_task
  #[test]
  fn test_plan_should_reject_dependency_on_non_existent_task() {
    let task_a = Task::new(
      "A".to_string(),
      "Task A".to_string(),
      "Desc A".to_string(),
      TaskStatus::Todo,
      Priority::P1,
      None,
      None,
      vec![],
    )
    .unwrap();

    let dep_a_c = TaskDependency::new("A".to_string(), "C".to_string()).unwrap();

    let result = Plan::new(
      "Test Plan".to_string(),
      "Description".to_string(),
      vec![task_a],
      vec![dep_a_c],
    );

    assert!(result.is_err());
    assert!(matches!(
      result,
      Err(PlanningError::MissingDependency { task_id, dependency_id })
      if task_id == "A" && dependency_id == "C"
    ));
  }

  // Test 9: test_task_priority_should_be_orderable
  #[test]
  fn test_task_priority_should_be_orderable() {
    assert!(Priority::P0 < Priority::P1);
    assert!(Priority::P1 < Priority::P2);
    assert!(Priority::P2 < Priority::P3);
    assert!(Priority::P0 < Priority::P3);

    let mut priorities = vec![Priority::P2, Priority::P0, Priority::P3, Priority::P1];
    priorities.sort();
    assert_eq!(
      priorities,
      vec![Priority::P0, Priority::P1, Priority::P2, Priority::P3]
    );
  }

  // Test 10: test_plan_should_calculate_completion_percentage
  #[test]
  fn test_plan_should_calculate_completion_percentage() {
    let tasks = (1..=10)
      .map(|i| {
        Task::new(
          format!("task-{i}"),
          format!("Task {i}"),
          "Desc".to_string(),
          if i <= 5 {
            TaskStatus::Done
          } else if i <= 8 {
            TaskStatus::InProgress
          } else {
            TaskStatus::Todo
          },
          Priority::P1,
          None,
          None,
          vec![],
        )
        .unwrap()
      })
      .collect();

    let plan = Plan::new(
      "Test Plan".to_string(),
      "Description".to_string(),
      tasks,
      vec![],
    )
    .unwrap();

    assert!((plan.completion_percentage() - 50.0).abs() < 0.01);
  }

  // Test 11: test_plan_should_list_blocked_tasks
  #[test]
  fn test_plan_should_list_blocked_tasks() {
    let tasks = vec![
      Task::new(
        "task-1".to_string(),
        "Task 1".to_string(),
        "Desc".to_string(),
        TaskStatus::Todo,
        Priority::P1,
        None,
        None,
        vec![],
      )
      .unwrap(),
      Task::new(
        "task-2".to_string(),
        "Task 2".to_string(),
        "Desc".to_string(),
        TaskStatus::Blocked,
        Priority::P1,
        None,
        None,
        vec![],
      )
      .unwrap(),
      Task::new(
        "task-3".to_string(),
        "Task 3".to_string(),
        "Desc".to_string(),
        TaskStatus::Blocked,
        Priority::P2,
        None,
        None,
        vec![],
      )
      .unwrap(),
    ];

    let plan = Plan::new(
      "Test Plan".to_string(),
      "Description".to_string(),
      tasks,
      vec![],
    )
    .unwrap();

    let blocked = plan.blocked_tasks();
    assert_eq!(blocked.len(), 2);
    assert!(blocked.iter().all(|t| t.status == TaskStatus::Blocked));
  }

  // Test 12: test_plan_should_list_ready_tasks
  #[allow(clippy::similar_names)]
  #[test]
  fn test_plan_should_list_ready_tasks() {
    let task_a = Task::new(
      "A".to_string(),
      "Task A".to_string(),
      "Desc A".to_string(),
      TaskStatus::Todo,
      Priority::P1,
      None,
      None,
      vec![],
    )
    .unwrap();

    let task_b = Task::new(
      "B".to_string(),
      "Task B".to_string(),
      "Desc B".to_string(),
      TaskStatus::Todo,
      Priority::P1,
      None,
      None,
      vec![],
    )
    .unwrap();

    let task_c = Task::new(
      "C".to_string(),
      "Task C".to_string(),
      "Desc C".to_string(),
      TaskStatus::Done,
      Priority::P1,
      None,
      None,
      vec![],
    )
    .unwrap();

    let task_d = Task::new(
      "D".to_string(),
      "Task D".to_string(),
      "Desc D".to_string(),
      TaskStatus::Todo,
      Priority::P1,
      None,
      None,
      vec![],
    )
    .unwrap();

    let dep_a_c = TaskDependency::new("A".to_string(), "C".to_string()).unwrap();
    let dep_b_c = TaskDependency::new("B".to_string(), "C".to_string()).unwrap();
    let dep_d_c = TaskDependency::new("D".to_string(), "C".to_string()).unwrap();

    let plan = Plan::new(
      "Test Plan".to_string(),
      "Description".to_string(),
      vec![task_a, task_b, task_c, task_d],
      vec![dep_a_c, dep_b_c, dep_d_c],
    )
    .unwrap();

    let ready = plan.ready_tasks();
    assert_eq!(ready.len(), 3); // A, B, D are ready (C is Done)
    assert!(ready.iter().all(|t| t.id != "C"));
  }

  // Test 13: test_plan_should_order_tasks_topologically
  #[allow(clippy::similar_names)]
  #[test]
  fn test_plan_should_order_tasks_topologically() {
    let task_a = Task::new(
      "A".to_string(),
      "Task A".to_string(),
      "Desc A".to_string(),
      TaskStatus::Todo,
      Priority::P1,
      None,
      None,
      vec![],
    )
    .unwrap();

    let task_b = Task::new(
      "B".to_string(),
      "Task B".to_string(),
      "Desc B".to_string(),
      TaskStatus::Todo,
      Priority::P1,
      None,
      None,
      vec![],
    )
    .unwrap();

    let task_c = Task::new(
      "C".to_string(),
      "Task C".to_string(),
      "Desc C".to_string(),
      TaskStatus::Todo,
      Priority::P1,
      None,
      None,
      vec![],
    )
    .unwrap();

    let task_d = Task::new(
      "D".to_string(),
      "Task D".to_string(),
      "Desc D".to_string(),
      TaskStatus::Todo,
      Priority::P1,
      None,
      None,
      vec![],
    )
    .unwrap();

    // Dependencies: A depends on B, A depends on C, B depends on D, C depends on D
    // So: B->A, C->A, D->B, D->C (edges point from dependency to dependent)
    // Valid order: D must be before B and C, B and C must be before A
    let dep_a_b = TaskDependency::new("A".to_string(), "B".to_string()).unwrap();
    let dep_a_c = TaskDependency::new("A".to_string(), "C".to_string()).unwrap();
    let dep_b_d = TaskDependency::new("B".to_string(), "D".to_string()).unwrap();
    let dep_c_d = TaskDependency::new("C".to_string(), "D".to_string()).unwrap();

    let plan = Plan::new(
      "Test Plan".to_string(),
      "Description".to_string(),
      vec![task_a, task_b, task_c, task_d],
      vec![dep_a_b, dep_a_c, dep_b_d, dep_c_d],
    )
    .unwrap();

    let result = plan.topological_order();
    assert!(result.is_ok());

    let ordered = result.unwrap();
    let pos_a = ordered.iter().position(|t| t.id == "A").unwrap();
    let pos_b = ordered.iter().position(|t| t.id == "B").unwrap();
    let pos_c = ordered.iter().position(|t| t.id == "C").unwrap();
    let pos_d = ordered.iter().position(|t| t.id == "D").unwrap();

    // D must come before B and C (since B and C depend on D)
    assert!(pos_d < pos_b);
    assert!(pos_d < pos_c);

    // B and C must come before A (since A depends on both)
    assert!(pos_b < pos_a);
    assert!(pos_c < pos_a);
  }

  // Test 14: test_task_should_support_due_dates
  #[test]
  fn test_task_should_support_due_dates() {
    let task = Task::new(
      "task-1".to_string(),
      "Task".to_string(),
      "Desc".to_string(),
      TaskStatus::Todo,
      Priority::P1,
      Some("2025-12-31T23:59:59Z".to_string()),
      None,
      vec![],
    )
    .unwrap();

    assert_eq!(task.due_date, Some("2025-12-31T23:59:59Z".to_string()));
  }

  // Test 15: test_task_should_detect_overdue
  #[test]
  fn test_task_should_detect_overdue() {
    // Past date, not Done -> overdue
    let task1 = Task::new(
      "task-1".to_string(),
      "Task".to_string(),
      "Desc".to_string(),
      TaskStatus::Todo,
      Priority::P1,
      Some("2020-01-01T00:00:00Z".to_string()),
      None,
      vec![],
    )
    .unwrap();

    assert!(task1.is_overdue());

    // Past date, Done -> not overdue
    let task2 = Task::new(
      "task-2".to_string(),
      "Task".to_string(),
      "Desc".to_string(),
      TaskStatus::Done,
      Priority::P1,
      Some("2020-01-01T00:00:00Z".to_string()),
      None,
      vec![],
    )
    .unwrap();

    assert!(!task2.is_overdue());

    // No due date -> not overdue
    let task3 = Task::new(
      "task-3".to_string(),
      "Task".to_string(),
      "Desc".to_string(),
      TaskStatus::Todo,
      Priority::P1,
      None,
      None,
      vec![],
    )
    .unwrap();

    assert!(!task3.is_overdue());
  }

  // Test 16: test_plan_should_support_estimates
  #[test]
  fn test_plan_should_support_estimates() {
    let task = Task::new(
      "task-1".to_string(),
      "Task".to_string(),
      "Desc".to_string(),
      TaskStatus::Todo,
      Priority::P1,
      None,
      Some(3.5),
      vec![],
    )
    .unwrap();

    assert_eq!(task.estimate_hours, Some(3.5));
  }

  // Test 17: test_plan_should_calculate_total_estimate
  #[test]
  fn test_plan_should_calculate_total_estimate() {
    let tasks = vec![
      Task::new(
        "task-1".to_string(),
        "Task 1".to_string(),
        "Desc".to_string(),
        TaskStatus::Todo,
        Priority::P1,
        None,
        Some(2.0),
        vec![],
      )
      .unwrap(),
      Task::new(
        "task-2".to_string(),
        "Task 2".to_string(),
        "Desc".to_string(),
        TaskStatus::Todo,
        Priority::P1,
        None,
        Some(3.0),
        vec![],
      )
      .unwrap(),
      Task::new(
        "task-3".to_string(),
        "Task 3".to_string(),
        "Desc".to_string(),
        TaskStatus::Todo,
        Priority::P1,
        None,
        None, // No estimate
        vec![],
      )
      .unwrap(),
    ];

    let plan = Plan::new(
      "Test Plan".to_string(),
      "Description".to_string(),
      tasks,
      vec![],
    )
    .unwrap();

    assert_eq!(plan.total_estimate(), 5.0);
  }

  // Test 18: test_plan_should_serialize_to_json
  #[test]
  fn test_plan_should_serialize_to_json() {
    let tasks = vec![Task::new(
      "task-1".to_string(),
      "Task 1".to_string(),
      "Desc".to_string(),
      TaskStatus::Todo,
      Priority::P1,
      None,
      Some(2.0),
      vec!["urgent".to_string()],
    )
    .unwrap()];

    let plan = Plan::new(
      "Test Plan".to_string(),
      "Description".to_string(),
      tasks,
      vec![],
    )
    .unwrap();

    let json = plan.to_json();
    assert!(json.is_ok());

    let json_str = json.unwrap();
    assert!(json_str.contains("Test Plan"));
    assert!(json_str.contains("task-1"));
  }

  // Test 19: test_plan_should_deserialize_from_json
  #[test]
  fn test_plan_should_deserialize_from_json() {
    let json = r#"{
      "title": "Test Plan",
      "description": "Description",
      "tasks": [
        {
          "id": "task-1",
          "title": "Task 1",
          "description": "Desc",
          "status": "todo",
          "priority": "P1",
          "due_date": null,
          "estimate_hours": 2.0,
          "tags": ["urgent"]
        }
      ],
      "dependencies": []
    }"#;

    let result = Plan::from_json(json);
    assert!(result.is_ok());

    let plan = result.unwrap();
    assert_eq!(plan.title, "Test Plan");
    assert_eq!(plan.tasks.len(), 1);
    assert_eq!(plan.tasks[0].id, "task-1");
  }

  // Test 20: test_plan_should_support_tags_labels
  #[test]
  fn test_plan_should_support_tags_labels() {
    let task = Task::new(
      "task-1".to_string(),
      "Task".to_string(),
      "Desc".to_string(),
      TaskStatus::Todo,
      Priority::P1,
      None,
      None,
      vec![
        "urgent".to_string(),
        "frontend".to_string(),
        "bug".to_string(),
      ],
    )
    .unwrap();

    assert_eq!(task.tags.len(), 3);
    assert!(task.tags.contains(&"urgent".to_string()));
    assert!(task.tags.contains(&"frontend".to_string()));
    assert!(task.tags.contains(&"bug".to_string()));
  }

  // Additional edge case tests

  #[test]
  fn test_empty_plan() {
    let plan = Plan::new(
      "Empty Plan".to_string(),
      "Description".to_string(),
      vec![],
      vec![],
    );

    assert!(plan.is_ok());
    let plan = plan.unwrap();
    assert_eq!(plan.completion_percentage(), 0.0);
    assert_eq!(plan.total_estimate(), 0.0);
  }

  #[test]
  fn test_task_with_empty_title_rejected() {
    let result = Task::new(
      "task-1".to_string(),
      "".to_string(),
      "Desc".to_string(),
      TaskStatus::Todo,
      Priority::P1,
      None,
      None,
      vec![],
    );

    assert!(result.is_err());
  }

  #[test]
  fn test_task_with_whitespace_title_rejected() {
    let result = Task::new(
      "task-1".to_string(),
      "   ".to_string(),
      "Desc".to_string(),
      TaskStatus::Todo,
      Priority::P1,
      None,
      None,
      vec![],
    );

    assert!(result.is_err());
  }

  #[test]
  fn test_task_with_negative_estimate_rejected() {
    let result = Task::new(
      "task-1".to_string(),
      "Task".to_string(),
      "Desc".to_string(),
      TaskStatus::Todo,
      Priority::P1,
      None,
      Some(-1.0),
      vec![],
    );

    assert!(result.is_err());
  }

  #[test]
  fn test_self_dependency_rejected() {
    let result = TaskDependency::new("task-1".to_string(), "task-1".to_string());

    assert!(result.is_err());
    assert!(matches!(
      result,
      Err(PlanningError::SelfDependency { task_id }) if task_id == "task-1"
    ));
  }

  #[test]
  fn test_plan_with_empty_title_rejected() {
    let result = Plan::new("".to_string(), "Description".to_string(), vec![], vec![]);

    assert!(result.is_err());
  }

  #[test]
  fn test_zero_estimate_allowed() {
    let result = Task::new(
      "task-1".to_string(),
      "Task".to_string(),
      "Desc".to_string(),
      TaskStatus::Todo,
      Priority::P1,
      None,
      Some(0.0),
      vec![],
    );

    assert!(result.is_ok());
  }

  #[test]
  fn test_priority_display() {
    assert_eq!(format!("{}", Priority::P0), "P0");
    assert_eq!(format!("{}", Priority::P1), "P1");
    assert_eq!(format!("{}", Priority::P2), "P2");
    assert_eq!(format!("{}", Priority::P3), "P3");
  }

  #[test]
  fn test_task_title_is_trimmed() {
    let task = Task::new(
      "task-1".to_string(),
      "  Task Title  ".to_string(),
      "Desc".to_string(),
      TaskStatus::Todo,
      Priority::P1,
      None,
      None,
      vec![],
    )
    .unwrap();

    assert_eq!(task.title, "Task Title");
  }

  #[test]
  fn test_plan_title_is_trimmed() {
    let plan = Plan::new(
      "  Plan Title  ".to_string(),
      "Description".to_string(),
      vec![],
      vec![],
    )
    .unwrap();

    assert_eq!(plan.title, "Plan Title");
  }
}
