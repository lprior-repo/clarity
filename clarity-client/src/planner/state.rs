//! Planner state management module
//!
//! Immutable state management for the Diamond methodology planning system.
//! All state updates return new instances, following functional core patterns.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::planner::types::{
  DiamondPhase, NorthStarScenario, Persona, PlanSession, PlanTask, ProductThesis, StateError,
  UseCase, MAX_COLLECTION_SIZE, MIN_DISCOVERY_PERSONAS, MIN_DISCOVERY_SCENARIOS,
};
use clarity_core::progress::{ProgressMetrics, ProgressStatus};
use rpds::Vector;
use std::rc::Rc;
use uuid::Uuid;

/// Immutable planner state snapshot
///
/// This represents the core planning state using persistent data structures
/// for efficient updates with structural sharing.
#[derive(Clone, Debug, PartialEq)]
pub struct PlannerState {
  /// Current plan session
  pub session: Option<Rc<PlanSession>>,
  /// Product thesis
  pub thesis: Option<Rc<ProductThesis>>,
  /// User personas
  pub personas: Vector<Rc<Persona>>,
  /// North star scenarios
  pub scenarios: Vector<Rc<NorthStarScenario>>,
  /// Use cases
  pub use_cases: Vector<Rc<UseCase>>,
  /// Context information
  pub context: PlannerContext,
  /// Tasks
  pub tasks: Vector<Rc<PlanTask>>,
  /// Current diamond phase
  pub current_phase: DiamondPhase,
}

/// Planner context information
///
/// Additional context for the planning session.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlannerContext {
  /// Project name
  pub project_name: String,
  /// Additional notes
  pub notes: String,
  /// Tags for categorization
  pub tags: Vector<String>,
}

impl PlannerContext {
  /// Create a new empty context
  #[must_use]
  pub fn new() -> Self {
    Self {
      project_name: String::new(),
      notes: String::new(),
      tags: Vector::new(),
    }
  }

  /// Update project name
  #[must_use]
  pub fn with_project_name(mut self, name: String) -> Self {
    self.project_name = name;
    self
  }

  /// Update notes
  #[must_use]
  pub fn with_notes(mut self, notes: String) -> Self {
    self.notes = notes;
    self
  }

  /// Add a tag
  #[must_use]
  pub fn with_tag(mut self, tag: String) -> Self {
    self.tags = self.tags.push_back(tag);
    self
  }
}

impl Default for PlannerContext {
  fn default() -> Self {
    Self::new()
  }
}

impl PlannerState {
  /// Create a new empty planner state
  #[must_use]
  pub fn new() -> Self {
    Self {
      session: None,
      thesis: None,
      personas: Vector::new(),
      scenarios: Vector::new(),
      use_cases: Vector::new(),
      context: PlannerContext::default(),
      tasks: Vector::new(),
      current_phase: DiamondPhase::default(),
    }
  }

  /// Create a new planner state from a session
  ///
  /// # Errors
  /// Returns `StateError` if the session exceeds maximum collection size
  #[must_use]
  pub fn from_session(session: PlanSession) -> Result<Self, StateError> {
    // Check collection sizes
    if session.personas.len() > MAX_COLLECTION_SIZE {
      return Err(StateError::CollectionTooLarge);
    }
    if session.north_star_scenarios.len() > MAX_COLLECTION_SIZE {
      return Err(StateError::CollectionTooLarge);
    }
    if session.use_cases.len() > MAX_COLLECTION_SIZE {
      return Err(StateError::CollectionTooLarge);
    }
    if session.tasks.len() > MAX_COLLECTION_SIZE {
      return Err(StateError::CollectionTooLarge);
    }

    // Check for duplicate IDs
    let persona_ids: std::collections::HashSet<Uuid> =
      session.personas.iter().map(|p| p.id).collect();
    if persona_ids.len() != session.personas.len() {
      return Err(StateError::DuplicateId("persona".to_string()));
    }

    let task_ids: std::collections::HashSet<Uuid> = session.tasks.iter().map(|t| t.id).collect();
    if task_ids.len() != session.tasks.len() {
      return Err(StateError::DuplicateId("task".to_string()));
    }

    let thesis = Rc::new(session.thesis.clone());
    let personas = session.personas.clone().into_iter().map(Rc::new).collect();
    let scenarios = session
      .north_star_scenarios
      .clone()
      .into_iter()
      .map(Rc::new)
      .collect();
    let use_cases = session.use_cases.clone().into_iter().map(Rc::new).collect();
    let tasks = session.tasks.clone().into_iter().map(Rc::new).collect();
    let current_phase = session.current_phase;

    Ok(Self {
      session: Some(Rc::new(session)),
      thesis: Some(thesis),
      personas,
      scenarios,
      use_cases,
      context: PlannerContext::default(),
      tasks,
      current_phase,
    })
  }

  /// Update the product thesis (immutable)
  #[must_use]
  pub fn update_thesis(&self, thesis: ProductThesis) -> Self {
    Self {
      thesis: Some(Rc::new(thesis)),
      ..self.clone()
    }
  }

  /// Update personas (immutable)
  ///
  /// # Errors
  /// Returns `StateError` if collection exceeds maximum size or has duplicates
  #[must_use]
  pub fn update_personas(&self, personas: Vec<Persona>) -> Result<Self, StateError> {
    if personas.len() > MAX_COLLECTION_SIZE {
      return Err(StateError::CollectionTooLarge);
    }

    // Check for duplicate IDs
    let ids: std::collections::HashSet<Uuid> = personas.iter().map(|p| p.id).collect();
    if ids.len() != personas.len() {
      return Err(StateError::DuplicateId("persona".to_string()));
    }

    Ok(Self {
      personas: personas.into_iter().map(Rc::new).collect(),
      ..self.clone()
    })
  }

  /// Add a single persona (immutable)
  ///
  /// # Errors
  /// Returns `StateError` if collection would exceed maximum size or ID is duplicate
  #[must_use]
  pub fn add_persona(&self, persona: Persona) -> Result<Self, StateError> {
    if self.personas.len() >= MAX_COLLECTION_SIZE {
      return Err(StateError::CollectionTooLarge);
    }

    // Check for duplicate ID
    let id_exists = self.personas.iter().any(|p| p.id == persona.id);
    if id_exists {
      return Err(StateError::DuplicateId("persona".to_string()));
    }

    Ok(Self {
      personas: self.personas.push_back(Rc::new(persona)),
      ..self.clone()
    })
  }

  /// Remove a persona by ID (immutable)
  #[must_use]
  pub fn remove_persona(&self, id: Uuid) -> Self {
    let personas = self
      .personas
      .iter()
      .filter(|p| p.id != id)
      .map(|p| p.clone())
      .collect();

    Self {
      personas,
      ..self.clone()
    }
  }

  /// Update scenarios (immutable)
  ///
  /// # Errors
  /// Returns `StateError` if collection exceeds maximum size or has duplicates
  #[must_use]
  pub fn update_scenarios(&self, scenarios: Vec<NorthStarScenario>) -> Result<Self, StateError> {
    if scenarios.len() > MAX_COLLECTION_SIZE {
      return Err(StateError::CollectionTooLarge);
    }

    // Check for duplicate IDs
    let ids: std::collections::HashSet<Uuid> = scenarios.iter().map(|s| s.id).collect();
    if ids.len() != scenarios.len() {
      return Err(StateError::DuplicateId("scenario".to_string()));
    }

    Ok(Self {
      scenarios: scenarios.into_iter().map(Rc::new).collect(),
      ..self.clone()
    })
  }

  /// Add a single scenario (immutable)
  ///
  /// # Errors
  /// Returns `StateError` if collection would exceed maximum size or ID is duplicate
  #[must_use]
  pub fn add_scenario(&self, scenario: NorthStarScenario) -> Result<Self, StateError> {
    if self.scenarios.len() >= MAX_COLLECTION_SIZE {
      return Err(StateError::CollectionTooLarge);
    }

    // Check for duplicate ID
    let id_exists = self.scenarios.iter().any(|s| s.id == scenario.id);
    if id_exists {
      return Err(StateError::DuplicateId("scenario".to_string()));
    }

    Ok(Self {
      scenarios: self.scenarios.push_back(Rc::new(scenario)),
      ..self.clone()
    })
  }

  /// Remove a scenario by ID (immutable)
  #[must_use]
  pub fn remove_scenario(&self, id: Uuid) -> Self {
    let scenarios = self
      .scenarios
      .iter()
      .filter(|s| s.id != id)
      .map(|s| s.clone())
      .collect();

    Self {
      scenarios,
      ..self.clone()
    }
  }

  /// Update use cases (immutable)
  ///
  /// # Errors
  /// Returns `StateError` if collection exceeds maximum size or has duplicates
  #[must_use]
  pub fn update_use_cases(&self, use_cases: Vec<UseCase>) -> Result<Self, StateError> {
    if use_cases.len() > MAX_COLLECTION_SIZE {
      return Err(StateError::CollectionTooLarge);
    }

    // Check for duplicate IDs
    let ids: std::collections::HashSet<Uuid> = use_cases.iter().map(|u| u.id).collect();
    if ids.len() != use_cases.len() {
      return Err(StateError::DuplicateId("use_case".to_string()));
    }

    Ok(Self {
      use_cases: use_cases.into_iter().map(Rc::new).collect(),
      ..self.clone()
    })
  }

  /// Add a single use case (immutable)
  ///
  /// # Errors
  /// Returns `StateError` if collection would exceed maximum size or ID is duplicate
  #[must_use]
  pub fn add_use_case(&self, use_case: UseCase) -> Result<Self, StateError> {
    if self.use_cases.len() >= MAX_COLLECTION_SIZE {
      return Err(StateError::CollectionTooLarge);
    }

    // Check for duplicate ID
    let id_exists = self.use_cases.iter().any(|u| u.id == use_case.id);
    if id_exists {
      return Err(StateError::DuplicateId("use_case".to_string()));
    }

    Ok(Self {
      use_cases: self.use_cases.push_back(Rc::new(use_case)),
      ..self.clone()
    })
  }

  /// Remove a use case by ID (immutable)
  #[must_use]
  pub fn remove_use_case(&self, id: Uuid) -> Self {
    let use_cases = self
      .use_cases
      .iter()
      .filter(|u| u.id != id)
      .map(|u| u.clone())
      .collect();

    Self {
      use_cases,
      ..self.clone()
    }
  }

  /// Update a single use case by ID (immutable)
  #[must_use]
  pub fn update_use_case(&self, id: Uuid, updated_use_case: UseCase) -> Self {
    let use_cases = self
      .use_cases
      .iter()
      .map(|u| match u.id == id {
        true => Rc::new(updated_use_case.clone()),
        false => u.clone(),
      })
      .collect();

    Self {
      use_cases,
      ..self.clone()
    }
  }

  /// Update context (immutable)
  #[must_use]
  pub fn update_context(&self, context: PlannerContext) -> Self {
    Self {
      context,
      ..self.clone()
    }
  }

  /// Update project name in context (immutable)
  #[must_use]
  pub fn update_project_name(&self, name: String) -> Self {
    Self {
      context: self.context.clone().with_project_name(name),
      ..self.clone()
    }
  }

  /// Update notes in context (immutable)
  #[must_use]
  pub fn update_notes(&self, notes: String) -> Self {
    Self {
      context: self.context.clone().with_notes(notes),
      ..self.clone()
    }
  }

  /// Update all tasks (immutable)
  ///
  /// # Errors
  /// Returns `StateError` if collection exceeds maximum size or has duplicates
  #[must_use]
  pub fn update_tasks(&self, tasks: Vec<PlanTask>) -> Result<Self, StateError> {
    if tasks.len() > MAX_COLLECTION_SIZE {
      return Err(StateError::CollectionTooLarge);
    }

    // Check for duplicate IDs
    let ids: std::collections::HashSet<Uuid> = tasks.iter().map(|t| t.id).collect();
    if ids.len() != tasks.len() {
      return Err(StateError::DuplicateId("task".to_string()));
    }

    Ok(Self {
      tasks: tasks.into_iter().map(Rc::new).collect(),
      ..self.clone()
    })
  }

  /// Update a single task by ID (immutable)
  #[must_use]
  pub fn update_task(&self, id: Uuid, updated_task: PlanTask) -> Self {
    let tasks = self
      .tasks
      .iter()
      .map(|t| match t.id == id {
        true => Rc::new(updated_task.clone()),
        false => t.clone(),
      })
      .collect();

    Self {
      tasks,
      ..self.clone()
    }
  }

  /// Add a single task (immutable)
  ///
  /// # Errors
  /// Returns `StateError` if collection would exceed maximum size or ID is duplicate
  #[must_use]
  pub fn add_task(&self, task: PlanTask) -> Result<Self, StateError> {
    if self.tasks.len() >= MAX_COLLECTION_SIZE {
      return Err(StateError::CollectionTooLarge);
    }

    // Check for duplicate ID
    let id_exists = self.tasks.iter().any(|t| t.id == task.id);
    if id_exists {
      return Err(StateError::DuplicateId("task".to_string()));
    }

    // Check for self-dependency
    if task.dependencies.contains(&task.id) {
      return Err(StateError::SelfDependency(task.id));
    }

    Ok(Self {
      tasks: self.tasks.push_back(Rc::new(task)),
      ..self.clone()
    })
  }

  /// Remove a task by ID (immutable)
  #[must_use]
  pub fn remove_task(&self, id: Uuid) -> Self {
    let tasks = self
      .tasks
      .iter()
      .filter(|t| t.id != id)
      .map(|t| t.clone())
      .collect();

    Self {
      tasks,
      ..self.clone()
    }
  }

  /// Move to the next phase (immutable) with validated completion criteria
  ///
  /// # Errors
  /// Returns `StateError` if phase transition is not allowed
  #[must_use]
  pub fn next_phase(&self) -> Result<Self, StateError> {
    let next = match self.current_phase {
      DiamondPhase::Top => {
        // Check Discovery phase completion requirements
        if self.thesis.is_none() {
          return Err(StateError::PhaseNotReady(DiamondPhase::Top));
        }

        // Validate thesis quality (not just existence)
        match &self.thesis {
          Some(thesis) => {
            if thesis.title.trim().is_empty() {
              return Err(StateError::PhaseNotReady(DiamondPhase::Top));
            }
            if thesis.problem.trim().is_empty() {
              return Err(StateError::PhaseNotReady(DiamondPhase::Top));
            }
          }
          None => return Err(StateError::PhaseNotReady(DiamondPhase::Top)),
        }

        if self.personas.len() < MIN_DISCOVERY_PERSONAS {
          return Err(StateError::PhaseNotReady(DiamondPhase::Top));
        }

        // Validate persona completeness
        for persona in self.personas.iter() {
          if persona.name.trim().is_empty() || persona.role.trim().is_empty() {
            return Err(StateError::PhaseNotReady(DiamondPhase::Top));
          }
        }

        if self.scenarios.len() < MIN_DISCOVERY_SCENARIOS {
          return Err(StateError::PhaseNotReady(DiamondPhase::Top));
        }

        // Validate scenario completeness
        for scenario in self.scenarios.iter() {
          if scenario.title.trim().is_empty() || scenario.narrative.trim().is_empty() {
            return Err(StateError::PhaseNotReady(DiamondPhase::Top));
          }
        }

        DiamondPhase::Right
      }
      DiamondPhase::Right => {
        // Design phase: Require at least one use case
        if self.use_cases.is_empty() {
          return Err(StateError::PhaseNotReady(DiamondPhase::Right));
        }

        // Validate use case completeness
        for use_case in self.use_cases.iter() {
          if use_case.title.trim().is_empty() || use_case.description.trim().is_empty() {
            return Err(StateError::PhaseNotReady(DiamondPhase::Right));
          }
        }

        DiamondPhase::Bottom
      }
      DiamondPhase::Bottom => {
        // Development phase: Require at least one task
        if self.tasks.is_empty() {
          return Err(StateError::PhaseNotReady(DiamondPhase::Bottom));
        }

        // Validate task completeness (each task must have a title)
        for task in self.tasks.iter() {
          if task.title.trim().is_empty() || task.description.trim().is_empty() {
            return Err(StateError::PhaseNotReady(DiamondPhase::Bottom));
          }
        }

        DiamondPhase::Left
      }
      DiamondPhase::Left => {
        return Err(StateError::InvalidPhaseTransition {
          current_phase: DiamondPhase::Left,
          next_phase: DiamondPhase::Left,
        })
      }
    };

    Ok(Self {
      current_phase: next,
      ..self.clone()
    })
  }

  /// Move to the previous phase (immutable)
  #[must_use]
  pub fn prev_phase(&self) -> Self {
    let prev = match self.current_phase {
      DiamondPhase::Top => DiamondPhase::Top, // Already at start
      DiamondPhase::Right => DiamondPhase::Top,
      DiamondPhase::Bottom => DiamondPhase::Right,
      DiamondPhase::Left => DiamondPhase::Bottom,
    };

    Self {
      current_phase: prev,
      ..self.clone()
    }
  }

  /// Set a specific phase (immutable)
  #[must_use]
  pub fn set_phase(&self, phase: DiamondPhase) -> Self {
    Self {
      current_phase: phase,
      ..self.clone()
    }
  }

  /// Check if can move to next phase with phase gate validation for all phases
  #[must_use]
  pub fn can_advance(&self) -> bool {
    match self.current_phase {
      DiamondPhase::Left => false,
      DiamondPhase::Top => {
        // Check Discovery phase completion
        if self.thesis.is_none() {
          return false;
        }

        // Validate thesis quality
        match &self.thesis {
          Some(thesis) => {
            if thesis.title.trim().is_empty() || thesis.problem.trim().is_empty() {
              return false;
            }
          }
          None => return false,
        }

        if self.personas.len() < MIN_DISCOVERY_PERSONAS {
          return false;
        }

        // Validate persona completeness
        for persona in self.personas.iter() {
          if persona.name.trim().is_empty() || persona.role.trim().is_empty() {
            return false;
          }
        }

        if self.scenarios.len() < MIN_DISCOVERY_SCENARIOS {
          return false;
        }

        // Validate scenario completeness
        for scenario in self.scenarios.iter() {
          if scenario.title.trim().is_empty() || scenario.narrative.trim().is_empty() {
            return false;
          }
        }

        true
      }
      DiamondPhase::Right => {
        // Design phase: Require at least one use case
        if self.use_cases.is_empty() {
          return false;
        }

        // Validate use case completeness
        for use_case in self.use_cases.iter() {
          if use_case.title.trim().is_empty() || use_case.description.trim().is_empty() {
            return false;
          }
        }

        true
      }
      DiamondPhase::Bottom => {
        // Development phase: Require at least one task
        if self.tasks.is_empty() {
          return false;
        }

        // Validate task completeness
        for task in self.tasks.iter() {
          if task.title.trim().is_empty() || task.description.trim().is_empty() {
            return false;
          }
        }

        true
      }
    }
  }

  /// Check if can move to previous phase
  #[must_use]
  pub fn can_retreat(&self) -> bool {
    !matches!(self.current_phase, DiamondPhase::Top)
  }

  /// Get progress percentage (0.0 to 1.0)
  #[must_use]
  pub fn progress(&self) -> f32 {
    match self.current_phase {
      DiamondPhase::Top => 0.0,
      DiamondPhase::Right => 0.33,
      DiamondPhase::Bottom => 0.66,
      DiamondPhase::Left => 1.0,
    }
  }

  /// Get all task statuses
  #[must_use]
  pub fn get_all_task_statuses(&self) -> Vec<ProgressStatus> {
    self
      .tasks
      .iter()
      .map(|task| self.get_task_status(task))
      .collect()
  }

  /// Get status for a single task
  #[must_use]
  pub fn get_task_status(&self, task: &Rc<PlanTask>) -> ProgressStatus {
    if task.completion >= 1.0 {
      ProgressStatus::Completed
    } else if task.completion > 0.0 {
      ProgressStatus::InProgress
    } else {
      // Check if task has dependencies that are not completed
      let all_deps_completed = task.dependencies.iter().all(|dep_id| {
        self
          .tasks
          .iter()
          .any(|t| t.id == *dep_id && t.completion >= 1.0)
      });

      if !all_deps_completed {
        ProgressStatus::Blocked
      } else {
        ProgressStatus::NotStarted
      }
    }
  }

  /// Calculate status metrics from all tasks
  #[must_use]
  pub fn calculate_status_metrics(&self) -> ProgressMetrics {
    let statuses = self.get_all_task_statuses();

    let counts = statuses.iter().fold(
      (0usize, 0usize, 0usize, 0usize, 0usize),
      |(completed, in_progress, blocked, deferred, not_started), status| match status {
        ProgressStatus::Completed => (completed + 1, in_progress, blocked, deferred, not_started),
        ProgressStatus::InProgress => (completed, in_progress + 1, blocked, deferred, not_started),
        ProgressStatus::Blocked => (completed, in_progress, blocked + 1, deferred, not_started),
        ProgressStatus::Deferred => (completed, in_progress, blocked, deferred + 1, not_started),
        ProgressStatus::NotStarted => (completed, in_progress, blocked, deferred, not_started + 1),
      },
    );

    ProgressMetrics::new(
      counts.0 + counts.1 + counts.2 + counts.3 + counts.4,
      counts.0,
      counts.1,
      counts.2,
      counts.3,
      counts.4,
    )
    .unwrap_or_else(|_| ProgressMetrics::empty())
  }
}

impl Default for PlannerState {
  fn default() -> Self {
    Self::new()
  }
}

/// UI-specific state for the planner
///
/// Contains transient UI state that doesn't need to persist.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlannerUIState {
  /// Currently selected tab/section
  pub active_tab: PlannerTab,
  /// Whether to show validation errors
  pub show_validation: bool,
  /// Whether to show graph visualization
  pub show_graph: bool,
  /// Whether sidebar is expanded
  pub sidebar_expanded: bool,
  /// Currently selected entity (for editing)
  pub selected_entity: Option<SelectedEntity>,
}

/// Tabs in the planner UI
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlannerTab {
  /// Thesis and context
  Overview,
  /// Personas and scenarios
  Discovery,
  /// Use cases and requirements
  Design,
  /// Tasks and implementation
  Development,
  /// Validation and delivery
  Delivery,
  /// Settings
  Settings,
}

impl Default for PlannerTab {
  fn default() -> Self {
    Self::Overview
  }
}

/// Currently selected entity for editing
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SelectedEntity {
  /// No entity selected
  None,
  /// Selected persona
  Persona(Uuid),
  /// Selected scenario
  Scenario(Uuid),
  /// Selected use case
  UseCase(Uuid),
  /// Selected task
  Task(Uuid),
}

impl Default for SelectedEntity {
  fn default() -> Self {
    Self::None
  }
}

impl SelectedEntity {
  /// Clear selection (return to None)
  #[must_use]
  pub const fn clear() -> Self {
    Self::None
  }

  /// Set entity selection
  #[must_use]
  pub const fn with_entity(self, entity: Self) -> Self {
    entity
  }
}

impl PlannerUIState {
  /// Create a new UI state with defaults
  #[must_use]
  pub fn new() -> Self {
    Self {
      active_tab: PlannerTab::default(),
      show_validation: false,
      show_graph: false,
      sidebar_expanded: true,
      selected_entity: None,
    }
  }

  /// Update active tab (immutable)
  #[must_use]
  pub const fn with_tab(mut self, tab: PlannerTab) -> Self {
    self.active_tab = tab;
    self
  }

  /// Toggle validation display (immutable)
  #[must_use]
  pub fn toggle_validation(&self) -> Self {
    Self {
      show_validation: !self.show_validation,
      ..self.clone()
    }
  }

  /// Set validation display (immutable)
  #[must_use]
  pub const fn with_validation(mut self, show: bool) -> Self {
    self.show_validation = show;
    self
  }

  /// Toggle graph display (immutable)
  #[must_use]
  pub fn toggle_graph(&self) -> Self {
    Self {
      show_graph: !self.show_graph,
      ..self.clone()
    }
  }

  /// Set graph display (immutable)
  #[must_use]
  pub const fn with_graph(mut self, show: bool) -> Self {
    self.show_graph = show;
    self
  }

  /// Toggle sidebar (immutable)
  #[must_use]
  pub fn toggle_sidebar(&self) -> Self {
    Self {
      sidebar_expanded: !self.sidebar_expanded,
      ..self.clone()
    }
  }

  /// Set sidebar state (immutable)
  #[must_use]
  pub const fn with_sidebar(mut self, expanded: bool) -> Self {
    self.sidebar_expanded = expanded;
    self
  }

  /// Set selected entity (immutable)
  #[must_use]
  pub fn with_entity(mut self, entity: SelectedEntity) -> Self {
    self.selected_entity = Some(entity);
    self
  }

  /// Clear selected entity (immutable)
  #[must_use]
  pub fn clear_entity(self) -> Self {
    Self {
      selected_entity: Some(SelectedEntity::clear()),
      ..self
    }
  }
}

impl Default for PlannerUIState {
  fn default() -> Self {
    Self::new()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::planner::types::TaskType;

  #[test]
  fn test_planner_state_new() {
    let state = PlannerState::new();

    assert!(state.session.is_none());
    assert!(state.thesis.is_none());
    assert!(state.personas.is_empty());
    assert!(state.scenarios.is_empty());
    assert!(state.use_cases.is_empty());
    assert!(state.tasks.is_empty());
    assert_eq!(state.current_phase, DiamondPhase::Top);
  }

  #[test]
  fn test_planner_state_add_persona() {
    let state = PlannerState::new();
    let persona = Persona::new(
      "User".to_string(),
      "Developer".to_string(),
      "A dev".to_string(),
    );

    let updated = state.add_persona(persona.clone()).unwrap();
    assert_eq!(updated.personas.len(), 1);
    assert_eq!(
      updated.personas.get(0).map(|p| &p.name),
      Some(&persona.name)
    );
  }

  #[test]
  fn test_planner_state_remove_persona() {
    let persona1 = Persona::new("User1".to_string(), "Dev".to_string(), "A".to_string());
    let persona2 = Persona::new("User2".to_string(), "Dev".to_string(), "B".to_string());

    let state = PlannerState::new()
      .add_persona(persona1.clone())
      .unwrap()
      .add_persona(persona2.clone())
      .unwrap();

    assert_eq!(state.personas.len(), 2);

    let updated = state.remove_persona(persona1.id);
    assert_eq!(updated.personas.len(), 1);
    assert_eq!(
      updated.personas.get(0).map(|p| &p.name),
      Some(&persona2.name)
    );
  }

  #[test]
  fn test_planner_state_next_phase() {
    // Setup state with required elements for phase transition
    let thesis = ProductThesis::new(
      "Thesis".to_string(),
      "Problem".to_string(),
      "Audience".to_string(),
      "Solution".to_string(),
      "Value".to_string(),
    );
    let persona = Persona::new("User".to_string(), "Dev".to_string(), "A dev".to_string());
    let scenario = NorthStarScenario::new("Scenario".to_string(), "Narrative".to_string());

    let state = PlannerState::new()
      .update_thesis(thesis)
      .add_persona(persona)
      .unwrap()
      .add_scenario(scenario)
      .unwrap();

    assert_eq!(state.current_phase, DiamondPhase::Top);

    let state = state.next_phase().unwrap();
    assert_eq!(state.current_phase, DiamondPhase::Right);

    // Add a use case for Design phase completion
    let use_case = UseCase::new(
      "Use Case".to_string(),
      "Description".to_string(),
      "Trigger".to_string(),
    );
    let state = state.add_use_case(use_case).unwrap();

    let state = state.next_phase().unwrap();
    assert_eq!(state.current_phase, DiamondPhase::Bottom);

    // Add a task for Development phase completion
    let task = PlanTask::new(
      "Task".to_string(),
      "Description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    );
    let state = state.add_task(task).unwrap();

    let state = state.next_phase().unwrap();
    assert_eq!(state.current_phase, DiamondPhase::Left);
  }

  #[test]
  fn test_planner_state_prev_phase() {
    let state = PlannerState::new().set_phase(DiamondPhase::Left);

    assert_eq!(state.current_phase, DiamondPhase::Left);

    let state = state.prev_phase();
    assert_eq!(state.current_phase, DiamondPhase::Bottom);

    let state = state.prev_phase();
    assert_eq!(state.current_phase, DiamondPhase::Right);

    let state = state.prev_phase();
    assert_eq!(state.current_phase, DiamondPhase::Top);

    let state = state.prev_phase(); // Stays at Top
    assert_eq!(state.current_phase, DiamondPhase::Top);
  }

  #[test]
  fn test_planner_state_update_task() {
    let state = PlannerState::new();
    let task = PlanTask::new(
      "Task".to_string(),
      "Description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    );

    let state = state.add_task(task.clone()).unwrap();
    assert_eq!(state.tasks.len(), 1);

    let updated_task = PlanTask {
      title: "Updated Task".to_string(),
      ..task.clone()
    };

    let state = state.update_task(task.id, updated_task.clone());
    assert_eq!(state.tasks.len(), 1);
    assert_eq!(
      state.tasks.get(0).map(|t| &t.title),
      Some(&updated_task.title)
    );
  }

  #[test]
  fn test_planner_state_progress() {
    let state = PlannerState::new();

    assert_eq!(state.progress(), 0.0);

    let state = state.set_phase(DiamondPhase::Right);
    assert!((state.progress() - 0.33).abs() < 0.01);

    let state = state.set_phase(DiamondPhase::Bottom);
    assert!((state.progress() - 0.66).abs() < 0.01);

    let state = state.set_phase(DiamondPhase::Left);
    assert_eq!(state.progress(), 1.0);
  }

  #[test]
  fn test_planner_context_new() {
    let context = PlannerContext::new();

    assert!(context.project_name.is_empty());
    assert!(context.notes.is_empty());
    assert!(context.tags.is_empty());
  }

  #[test]
  fn test_planner_context_with_updates() {
    let context = PlannerContext::new()
      .with_project_name("My Project".to_string())
      .with_notes("Some notes".to_string())
      .with_tag("tag1".to_string());

    assert_eq!(context.project_name, "My Project");
    assert_eq!(context.notes, "Some notes");
    assert_eq!(context.tags.len(), 1);
  }

  #[test]
  fn test_ui_state_new() {
    let ui = PlannerUIState::new();

    assert_eq!(ui.active_tab, PlannerTab::Overview);
    assert!(!ui.show_validation);
    assert!(!ui.show_graph);
    assert!(ui.sidebar_expanded);
    assert!(ui.selected_entity.is_none());
  }

  #[test]
  fn test_ui_state_toggles() {
    let ui = PlannerUIState::new();

    let ui = ui.toggle_validation();
    assert!(ui.show_validation);

    let ui = ui.toggle_graph();
    assert!(ui.show_graph);

    let ui = ui.toggle_sidebar();
    assert!(!ui.sidebar_expanded);
  }

  // CRITICAL-008 HOSTILE TEST: Concurrent toggle safety
  #[test]
  fn test_ui_state_concurrent_toggle_safety() {
    let ui = PlannerUIState::new();

    // Simulate concurrent operations from the same base state
    let ui1 = ui.clone().toggle_validation();
    let ui2 = ui.clone().toggle_graph();
    let ui3 = ui.clone().toggle_sidebar();

    // Each operation should create a consistent state
    assert!(ui1.show_validation);
    assert!(!ui1.show_graph);
    assert!(ui1.sidebar_expanded);

    assert!(!ui2.show_validation);
    assert!(ui2.show_graph);
    assert!(ui2.sidebar_expanded);

    assert!(!ui3.show_validation);
    assert!(!ui3.show_graph);
    assert!(!ui3.sidebar_expanded);

    // Chaining operations should be deterministic
    let chained = ui
      .toggle_validation()
      .toggle_graph()
      .toggle_sidebar()
      .toggle_validation()
      .toggle_graph();

    assert!(!chained.show_validation); // toggled twice
    assert!(!chained.show_graph); // toggled twice
    assert!(!chained.sidebar_expanded); // toggled once
  }

  // CRITICAL-008 HOSTILE TEST: Rapid state transitions
  #[test]
  fn test_ui_state_rapid_transitions() {
    let ui = PlannerUIState::new();

    // Rapidly toggle validation 10 times
    let mut ui = ui;
    for _ in 0..10 {
      ui = ui.toggle_validation();
    }
    // Should end up at original state (even number of toggles)
    assert!(!ui.show_validation);

    // Rapidly toggle all fields
    let mut ui = PlannerUIState::new();
    for i in 0..99 {
      ui = match i % 3 {
        0 => ui.toggle_validation(),
        1 => ui.toggle_graph(),
        _ => ui.toggle_sidebar(),
      };
    }

    // State should be consistent
    // 99 operations / 3 = 33 toggles of each (odd number)
    // Original: all false except sidebar=true
    // After 33 toggles: validation=true, graph=true, sidebar=false
    assert!(ui.show_validation);
    assert!(ui.show_graph);
    assert!(!ui.sidebar_expanded);
  }

  // CRITICAL-008 HOSTILE TEST: Entity selection race safety
  #[test]
  fn test_ui_state_entity_selection_safety() {
    let ui = PlannerUIState::new();

    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();

    // Simulate concurrent entity selections from same base state
    let ui1 = ui.clone().with_entity(SelectedEntity::Task(id1));
    let ui2 = ui.clone().with_entity(SelectedEntity::Task(id2));

    // Each state should be independent and consistent
    match &ui1.selected_entity {
      Some(SelectedEntity::Task(id)) => assert_eq!(*id, id1),
      _ => panic!("Expected Task(id1)"),
    }

    match &ui2.selected_entity {
      Some(SelectedEntity::Task(id)) => assert_eq!(*id, id2),
      _ => panic!("Expected Task(id2)"),
    }

    // Clearing should work correctly
    let ui1_cleared = ui1.clear_entity();
    match &ui1_cleared.selected_entity {
      Some(SelectedEntity::None) => {}
      _ => panic!("Expected None after clear"),
    }
  }

  // CRITICAL-008 HOSTILE TEST: Tab switching consistency
  #[test]
  fn test_ui_state_tab_switching_consistency() {
    let ui = PlannerUIState::new();

    // Rapid tab switching
    let tabs = [
      PlannerTab::Overview,
      PlannerTab::Discovery,
      PlannerTab::Design,
      PlannerTab::Development,
      PlannerTab::Delivery,
      PlannerTab::Settings,
    ];

    let mut ui = ui;
    for tab in tabs.iter().cycle().take(100) {
      ui = ui.with_tab(*tab);
    }

    // Final state should be deterministic (100th tab, zero-indexed)
    // We iterate 100 times starting from Overview (index 0)
    // So we end up at index 99 (0-indexed)
    // 99 % 6 = 3, which is the 4th tab (index 3) = Development
    assert_eq!(ui.active_tab, PlannerTab::Development);
  }

  // CRITICAL-008 HOSTILE TEST: State cloning independence
  #[test]
  fn test_ui_state_clone_independence() {
    let ui = PlannerUIState::new();

    let ui1 = ui.clone().toggle_validation();
    let ui2 = ui.clone().toggle_graph();
    let ui3 = ui1.clone().toggle_sidebar();

    // All clones should be independent
    assert!(ui1.show_validation);
    assert!(!ui2.show_validation);
    assert!(ui3.show_validation);
    assert!(!ui3.show_graph); // ui3 was cloned from ui1, which didn't toggle graph
    assert!(!ui3.sidebar_expanded);

    // Original should be unchanged
    assert!(!ui.show_validation);
    assert!(!ui.show_graph);
    assert!(ui.sidebar_expanded);
  }

  // CRITICAL-008 HOSTILE TEST: With methods are deterministic
  #[test]
  fn test_ui_state_with_methods_deterministic() {
    let ui = PlannerUIState::new();

    // Calling with_* multiple times with same value should be idempotent
    let ui1 = ui.clone().with_validation(true);
    let ui2 = ui1.clone().with_validation(true);
    let ui3 = ui2.clone().with_validation(true);

    assert!(ui3.show_validation);
    assert_eq!(ui1, ui2);
    assert_eq!(ui2, ui3);

    // Same for other fields
    let ui1 = ui.clone().with_graph(true);
    let ui2 = ui1.clone().with_graph(true);

    assert!(ui2.show_graph);
    assert_eq!(ui1, ui2);
  }

  #[test]
  fn test_add_task_within_bounds() {
    let state = PlannerState::new();
    let task = PlanTask::new(
      "Task".to_string(),
      "Description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    );

    let result = state.add_task(task);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().tasks.len(), 1);
  }

  #[test]
  fn test_add_persona_checks_duplicate_id() {
    let state = PlannerState::new();
    let persona = Persona::new("User".to_string(), "Dev".to_string(), "A dev".to_string());

    let state = state.add_persona(persona.clone()).unwrap();
    let result = state.add_persona(persona); // Same ID

    assert!(result.is_err());
    assert!(matches!(result, Err(StateError::DuplicateId(_))));
  }

  #[test]
  fn test_add_scenario_checks_duplicate_id() {
    let state = PlannerState::new();
    let scenario = NorthStarScenario::new("Scenario".to_string(), "Narrative".to_string());

    let state = state.add_scenario(scenario.clone()).unwrap();
    let result = state.add_scenario(scenario); // Same ID

    assert!(result.is_err());
    assert!(matches!(result, Err(StateError::DuplicateId(_))));
  }

  #[test]
  fn test_add_use_case_checks_duplicate_id() {
    let state = PlannerState::new();
    let use_case = UseCase::new(
      "Use Case".to_string(),
      "Description".to_string(),
      "Trigger".to_string(),
    );

    let state = state.add_use_case(use_case.clone()).unwrap();
    let result = state.add_use_case(use_case); // Same ID

    assert!(result.is_err());
    assert!(matches!(result, Err(StateError::DuplicateId(_))));
  }

  #[test]
  fn test_next_phase_validates_discovery_completion() {
    let state = PlannerState::new();

    // Can't advance from Top without thesis, personas, and scenarios
    let result = state.next_phase();
    assert!(result.is_err());
    assert!(matches!(result, Err(StateError::PhaseNotReady(_))));

    // Add thesis but still missing personas and scenarios
    let thesis = ProductThesis::new(
      "Thesis".to_string(),
      "Problem".to_string(),
      "Audience".to_string(),
      "Solution".to_string(),
      "Value".to_string(),
    );
    let state = state.update_thesis(thesis);

    let result = state.next_phase();
    assert!(result.is_err());

    // Add persona
    let persona = Persona::new("User".to_string(), "Dev".to_string(), "A dev".to_string());
    let state = state.add_persona(persona).unwrap();

    let result = state.next_phase();
    assert!(result.is_err());

    // Add scenario - now should succeed
    let scenario = NorthStarScenario::new("Scenario".to_string(), "Narrative".to_string());
    let state = state.add_scenario(scenario).unwrap();

    let result = state.next_phase();
    assert!(result.is_ok());
    assert_eq!(result.unwrap().current_phase, DiamondPhase::Right);
  }

  #[test]
  fn test_can_advance_checks_discovery_completion() {
    let state = PlannerState::new();
    assert!(!state.can_advance()); // No thesis, personas, or scenarios

    let thesis = ProductThesis::new(
      "Thesis".to_string(),
      "Problem".to_string(),
      "Audience".to_string(),
      "Solution".to_string(),
      "Value".to_string(),
    );
    let state = state.update_thesis(thesis);
    assert!(!state.can_advance()); // Still missing personas and scenarios

    let persona = Persona::new("User".to_string(), "Dev".to_string(), "A dev".to_string());
    let state = state.add_persona(persona).unwrap();
    assert!(!state.can_advance()); // Still missing scenarios

    let scenario = NorthStarScenario::new("Scenario".to_string(), "Narrative".to_string());
    let state = state.add_scenario(scenario).unwrap();
    assert!(state.can_advance()); // All requirements met
  }

  #[test]
  fn test_next_phase_validates_design_phase() {
    let mut state = PlannerState::new();

    // Complete Discovery phase
    let thesis = ProductThesis::new(
      "Thesis".to_string(),
      "Problem".to_string(),
      "Audience".to_string(),
      "Solution".to_string(),
      "Value".to_string(),
    );
    state = state.update_thesis(thesis);
    let persona = Persona::new("User".to_string(), "Dev".to_string(), "A dev".to_string());
    state = state.add_persona(persona).unwrap();
    let scenario = NorthStarScenario::new("Scenario".to_string(), "Narrative".to_string());
    state = state.add_scenario(scenario).unwrap();

    // Move to Design phase
    state = state.next_phase().unwrap();
    assert_eq!(state.current_phase, DiamondPhase::Right);

    // Can't advance to Development without use cases
    let result = state.next_phase();
    assert!(result.is_err());

    // Add a use case
    let use_case = UseCase::new(
      "Use Case".to_string(),
      "Description".to_string(),
      "Trigger".to_string(),
    );
    state = state.add_use_case(use_case).unwrap();

    // Now can advance
    let result = state.next_phase();
    assert!(result.is_ok());
    assert_eq!(result.unwrap().current_phase, DiamondPhase::Bottom);
  }

  #[test]
  fn test_next_phase_validates_development_phase() {
    let mut state = PlannerState::new();

    // Complete Discovery phase
    let thesis = ProductThesis::new(
      "Thesis".to_string(),
      "Problem".to_string(),
      "Audience".to_string(),
      "Solution".to_string(),
      "Value".to_string(),
    );
    state = state.update_thesis(thesis);
    let persona = Persona::new("User".to_string(), "Dev".to_string(), "A dev".to_string());
    state = state.add_persona(persona).unwrap();
    let scenario = NorthStarScenario::new("Scenario".to_string(), "Narrative".to_string());
    state = state.add_scenario(scenario).unwrap();
    state = state.next_phase().unwrap();

    // Complete Design phase
    let use_case = UseCase::new(
      "Use Case".to_string(),
      "Description".to_string(),
      "Trigger".to_string(),
    );
    state = state.add_use_case(use_case).unwrap();
    state = state.next_phase().unwrap();

    // Can't advance to Delivery without tasks
    let result = state.next_phase();
    assert!(result.is_err());

    // Add a task
    let task = PlanTask::new(
      "Task".to_string(),
      "Description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    );
    state = state.add_task(task).unwrap();

    // Now can advance
    let result = state.next_phase();
    assert!(result.is_ok());
    assert_eq!(result.unwrap().current_phase, DiamondPhase::Left);
  }

  #[test]
  fn test_next_phase_rejects_empty_thesis() {
    let mut state = PlannerState::new();

    // Create thesis with empty title (should fail validation)
    let thesis = ProductThesis::new(
      "".to_string(), // Empty title
      "Problem".to_string(),
      "Audience".to_string(),
      "Solution".to_string(),
      "Value".to_string(),
    );
    state = state.update_thesis(thesis);

    let persona = Persona::new("User".to_string(), "Dev".to_string(), "A dev".to_string());
    state = state.add_persona(persona).unwrap();
    let scenario = NorthStarScenario::new("Scenario".to_string(), "Narrative".to_string());
    state = state.add_scenario(scenario).unwrap();

    // Should fail due to empty thesis title
    let result = state.next_phase();
    assert!(result.is_err());
  }

  #[test]
  fn test_next_phase_rejects_incomplete_personas() {
    let mut state = PlannerState::new();

    let thesis = ProductThesis::new(
      "Thesis".to_string(),
      "Problem".to_string(),
      "Audience".to_string(),
      "Solution".to_string(),
      "Value".to_string(),
    );
    state = state.update_thesis(thesis);

    // Create persona with empty name
    let persona = Persona::new("".to_string(), "Dev".to_string(), "A dev".to_string());
    state = state.add_persona(persona).unwrap();

    let scenario = NorthStarScenario::new("Scenario".to_string(), "Narrative".to_string());
    state = state.add_scenario(scenario).unwrap();

    // Should fail due to empty persona name
    let result = state.next_phase();
    assert!(result.is_err());
  }

  // ========================================================================
  // ROUND 2: STATE MACHINE TRANSITIONS AND INTEGRATION TESTS
  // ========================================================================

  // -------------------------------------------------------------------------
  // Phase Transition Behaviors (DDD - Aggregate Root State Changes)
  // -------------------------------------------------------------------------

  #[test]
  fn round2_phase_transition_discovery_to_design_validates_thesis_quality() {
    // GIVEN: Discovery phase with thesis, personas, and scenarios
    let thesis = ProductThesis::new(
      "Valid Thesis".to_string(),
      "Real Problem".to_string(),
      "Target Audience".to_string(),
      "Our Solution".to_string(),
      "Clear Value".to_string(),
    );
    let persona1 = Persona::new(
      "Alice".to_string(),
      "Developer".to_string(),
      "Dev persona".to_string(),
    );
    let persona2 = Persona::new(
      "Bob".to_string(),
      "Designer".to_string(),
      "Design persona".to_string(),
    );
    let scenario = NorthStarScenario::new(
      "Happy Path".to_string(),
      "User completes workflow successfully".to_string(),
    );

    let state = PlannerState::new()
      .update_thesis(thesis)
      .add_persona(persona1)
      .unwrap()
      .add_persona(persona2)
      .unwrap()
      .add_scenario(scenario)
      .unwrap();

    assert_eq!(state.current_phase, DiamondPhase::Top);
    assert!(state.can_advance());

    // WHEN: Transition to Design phase
    let result = state.next_phase();

    // THEN: State validates and phase changes
    assert!(result.is_ok());
    let new_state = result.unwrap();
    assert_eq!(new_state.current_phase, DiamondPhase::Right);
    // Original state unchanged (immutability)
    assert_eq!(state.current_phase, DiamondPhase::Top);
  }

  #[test]
  fn round2_phase_transition_design_to_development_locks_requirements() {
    // GIVEN: Design phase with use cases
    let thesis = ProductThesis::new(
      "Thesis".to_string(),
      "Problem".to_string(),
      "Audience".to_string(),
      "Solution".to_string(),
      "Value".to_string(),
    );
    let persona = Persona::new("User".to_string(), "Role".to_string(), "Desc".to_string());
    let scenario = NorthStarScenario::new("Scenario".to_string(), "Narrative".to_string());

    let mut state = PlannerState::new()
      .update_thesis(thesis)
      .add_persona(persona)
      .unwrap()
      .add_scenario(scenario)
      .unwrap()
      .next_phase()
      .unwrap();

    let use_case1 = UseCase::new(
      "User Authentication".to_string(),
      "User can log in".to_string(),
      "User visits login page".to_string(),
    );
    let use_case2 = UseCase::new(
      "Data Export".to_string(),
      "User can export data".to_string(),
      "User clicks export".to_string(),
    );

    state = state
      .add_use_case(use_case1)
      .unwrap()
      .add_use_case(use_case2)
      .unwrap();

    assert_eq!(state.current_phase, DiamondPhase::Right);
    assert!(state.can_advance());

    // WHEN: Transition to Development phase
    let result = state.next_phase();

    // THEN: Requirements locked (phase changes, can_retreat allows going back)
    assert!(result.is_ok());
    let new_state = result.unwrap();
    assert_eq!(new_state.current_phase, DiamondPhase::Bottom);
    assert_eq!(new_state.use_cases.len(), 2);
  }

  #[test]
  fn round2_phase_transition_development_to_delivery_validates_tasks() {
    // GIVEN: Development phase with tasks
    let thesis = ProductThesis::new(
      "Thesis".to_string(),
      "Problem".to_string(),
      "Audience".to_string(),
      "Solution".to_string(),
      "Value".to_string(),
    );
    let persona = Persona::new("User".to_string(), "Role".to_string(), "Desc".to_string());
    let scenario = NorthStarScenario::new("Scenario".to_string(), "Narrative".to_string());
    let use_case = UseCase::new(
      "Use Case".to_string(),
      "Desc".to_string(),
      "Trigger".to_string(),
    );

    let mut state = PlannerState::new()
      .update_thesis(thesis)
      .add_persona(persona)
      .unwrap()
      .add_scenario(scenario)
      .unwrap()
      .next_phase()
      .unwrap()
      .add_use_case(use_case)
      .unwrap()
      .next_phase()
      .unwrap();

    let task1 = PlanTask::new(
      "Setup Project".to_string(),
      "Initialize repository".to_string(),
      TaskType::Infrastructure,
      DiamondPhase::Bottom,
    );
    let task2 = PlanTask::new(
      "Implement Core".to_string(),
      "Build main feature".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    );

    state = state.add_task(task1).unwrap().add_task(task2).unwrap();

    assert_eq!(state.current_phase, DiamondPhase::Bottom);
    assert!(state.can_advance());

    // WHEN: Transition to Delivery phase
    let result = state.next_phase();

    // THEN: Validation runs (tasks validated), phase changes
    assert!(result.is_ok());
    let new_state = result.unwrap();
    assert_eq!(new_state.current_phase, DiamondPhase::Left);
    assert_eq!(new_state.tasks.len(), 2);
    // Cannot advance from Delivery
    assert!(!new_state.can_advance());
  }

  #[test]
  fn round2_phase_transition_rejects_invalid_discovery_to_design() {
    // GIVEN: Discovery phase with incomplete thesis (empty problem)
    let thesis = ProductThesis::new(
      "Title".to_string(),
      "".to_string(), // Empty problem - should fail
      "Audience".to_string(),
      "Solution".to_string(),
      "Value".to_string(),
    );
    let persona = Persona::new("User".to_string(), "Role".to_string(), "Desc".to_string());
    let scenario = NorthStarScenario::new("Scenario".to_string(), "Narrative".to_string());

    let state = PlannerState::new()
      .update_thesis(thesis)
      .add_persona(persona)
      .unwrap()
      .add_scenario(scenario)
      .unwrap();

    // WHEN: Attempt to transition to Design phase
    let result = state.next_phase();

    // THEN: State rejects change
    assert!(result.is_err());
    match result {
      Err(StateError::PhaseNotReady(DiamondPhase::Top)) => {}
      _ => panic!("Expected PhaseNotReady error"),
    }
    // Verify state unchanged
    assert_eq!(state.current_phase, DiamondPhase::Top);
  }

  #[test]
  fn round2_phase_transition_rejects_invalid_design_to_development() {
    // GIVEN: Design phase with incomplete use case (empty description)
    let thesis = ProductThesis::new(
      "Thesis".to_string(),
      "Problem".to_string(),
      "Audience".to_string(),
      "Solution".to_string(),
      "Value".to_string(),
    );
    let persona = Persona::new("User".to_string(), "Role".to_string(), "Desc".to_string());
    let scenario = NorthStarScenario::new("Scenario".to_string(), "Narrative".to_string());

    let mut state = PlannerState::new()
      .update_thesis(thesis)
      .add_persona(persona)
      .unwrap()
      .add_scenario(scenario)
      .unwrap()
      .next_phase()
      .unwrap();

    // Add use case with empty description
    let incomplete_use_case = UseCase::new(
      "Title".to_string(),
      "".to_string(), // Empty description
      "Trigger".to_string(),
    );
    state = state.add_use_case(incomplete_use_case).unwrap();

    // WHEN: Attempt to transition to Development phase
    let result = state.next_phase();

    // THEN: State rejects change
    assert!(result.is_err());
    match result {
      Err(StateError::PhaseNotReady(DiamondPhase::Right)) => {}
      _ => panic!("Expected PhaseNotReady error"),
    }
  }

  #[test]
  fn round2_phase_transition_rejects_invalid_development_to_delivery() {
    // GIVEN: Development phase with incomplete task (empty description)
    let thesis = ProductThesis::new(
      "Thesis".to_string(),
      "Problem".to_string(),
      "Audience".to_string(),
      "Solution".to_string(),
      "Value".to_string(),
    );
    let persona = Persona::new("User".to_string(), "Role".to_string(), "Desc".to_string());
    let scenario = NorthStarScenario::new("Scenario".to_string(), "Narrative".to_string());
    let use_case = UseCase::new(
      "Use Case".to_string(),
      "Desc".to_string(),
      "Trigger".to_string(),
    );

    let mut state = PlannerState::new()
      .update_thesis(thesis)
      .add_persona(persona)
      .unwrap()
      .add_scenario(scenario)
      .unwrap()
      .next_phase()
      .unwrap()
      .add_use_case(use_case)
      .unwrap()
      .next_phase()
      .unwrap();

    // Add task with empty description
    let incomplete_task = PlanTask::new(
      "Title".to_string(),
      "".to_string(), // Empty description
      TaskType::Development,
      DiamondPhase::Bottom,
    );
    state = state.add_task(incomplete_task).unwrap();

    // WHEN: Attempt to transition to Delivery phase
    let result = state.next_phase();

    // THEN: State rejects change
    assert!(result.is_err());
    match result {
      Err(StateError::PhaseNotReady(DiamondPhase::Bottom)) => {}
      _ => panic!("Expected PhaseNotReady error"),
    }
  }

  #[test]
  fn round2_phase_transition_delivery_cannot_advance() {
    // GIVEN: Delivery phase (final phase)
    let state = PlannerState::new().set_phase(DiamondPhase::Left);

    // WHEN: Attempt to advance past Delivery
    let result = state.next_phase();

    // THEN: Transition rejected
    assert!(result.is_err());
    match result {
      Err(StateError::InvalidPhaseTransition {
        current_phase: DiamondPhase::Left,
        next_phase: DiamondPhase::Left,
      }) => {}
      _ => panic!("Expected InvalidPhaseTransition error"),
    }
    assert!(!state.can_advance());
  }

  #[test]
  fn round2_phase_transition_prev_phase_always_succeeds() {
    // GIVEN: Any phase
    let state = PlannerState::new().set_phase(DiamondPhase::Bottom);

    // WHEN: Move to previous phase
    let new_state = state.prev_phase();

    // THEN: Phase changes (no validation on retreat)
    assert_eq!(new_state.current_phase, DiamondPhase::Right);
    assert!(new_state.can_retreat());

    // Can retreat to Top
    let new_state = new_state.prev_phase();
    assert_eq!(new_state.current_phase, DiamondPhase::Top);
    assert!(!new_state.can_retreat()); // Cannot retreat from Top

    // Stays at Top
    let new_state = new_state.prev_phase();
    assert_eq!(new_state.current_phase, DiamondPhase::Top);
  }

  // -------------------------------------------------------------------------
  // Cross-Entity Relationships (DDD - Aggregate Boundaries)
  // -------------------------------------------------------------------------

  #[test]
  fn round2_cross_entity_scenario_with_persona_relationship() {
    // GIVEN: Persona exists
    let persona = Persona::new(
      "Alice".to_string(),
      "Developer".to_string(),
      "Senior engineer persona".to_string(),
    );

    let state = PlannerState::new().add_persona(persona.clone()).unwrap();

    // WHEN: Create scenario linked to persona
    let mut scenario = NorthStarScenario::new(
      "Code Review Workflow".to_string(),
      "Alice reviews pull requests efficiently".to_string(),
    );
    scenario = scenario.with_persona(persona.id);

    let state = state.add_scenario(scenario).unwrap();

    // THEN: Relationship established (persona_id stored in scenario)
    let retrieved_scenario = state.scenarios.get(0);
    assert!(retrieved_scenario.is_some());
    match retrieved_scenario {
      Some(s) => {
        assert_eq!(s.persona_id, Some(persona.id));
        assert_eq!(s.title, "Code Review Workflow");
      }
      None => panic!("Scenario not found"),
    }

    // Persona still exists independently
    let retrieved_persona = state.personas.iter().find(|p| p.id == persona.id);
    assert!(retrieved_persona.is_some());
  }

  #[test]
  fn round2_cross_entity_use_case_with_persona_relationship() {
    // GIVEN: Persona exists
    let persona = Persona::new(
      "Bob".to_string(),
      "Designer".to_string(),
      "UX designer persona".to_string(),
    );

    let state = PlannerState::new().add_persona(persona.clone()).unwrap();

    // WHEN: Create use case linked to persona
    let mut use_case = UseCase::new(
      "Design Component Library".to_string(),
      "Bob creates reusable UI components".to_string(),
      "Project requires new component".to_string(),
    );
    use_case = use_case.with_persona(persona.id);

    let state = state.add_use_case(use_case).unwrap();

    // THEN: Relationship established
    let retrieved_use_case = state.use_cases.get(0);
    assert!(retrieved_use_case.is_some());
    match retrieved_use_case {
      Some(u) => {
        assert_eq!(u.persona_id, Some(persona.id));
        assert_eq!(u.title, "Design Component Library");
      }
      None => panic!("Use case not found"),
    }
  }

  #[test]
  fn round2_cross_entity_scenario_handles_missing_persona_gracefully() {
    // GIVEN: Scenario with persona_id pointing to non-existent persona
    let fake_persona_id = Uuid::new_v4(); // Doesn't exist in state

    let mut scenario = NorthStarScenario::new(
      "Orphan Scenario".to_string(),
      "This scenario references a missing persona".to_string(),
    );
    scenario.persona_id = Some(fake_persona_id);

    let state = PlannerState::new().add_scenario(scenario).unwrap();

    // WHEN: Access the scenario
    let retrieved_scenario = state.scenarios.get(0);

    // THEN: Scenario handles missing persona gracefully (no crash, just Option<Uuid>)
    assert!(retrieved_scenario.is_some());
    match retrieved_scenario {
      Some(s) => {
        assert_eq!(s.persona_id, Some(fake_persona_id));
        // State doesn't validate foreign keys - it's just data
        // This is correct for an aggregate root pattern
      }
      None => panic!("Scenario not found"),
    }

    // Verify persona truly doesn't exist
    let persona_exists = state.personas.iter().any(|p| p.id == fake_persona_id);
    assert!(!persona_exists);
  }

  #[test]
  fn round2_cross_entity_use_case_handles_orphan_gracefully() {
    // GIVEN: Use case with persona_id pointing to non-existent persona
    let fake_persona_id = Uuid::new_v4();

    let mut use_case = UseCase::new(
      "Orphan Use Case".to_string(),
      "References missing persona".to_string(),
      "Trigger".to_string(),
    );
    use_case.persona_id = Some(fake_persona_id);

    let state = PlannerState::new().add_use_case(use_case).unwrap();

    // WHEN: Access the use case
    let retrieved_use_case = state.use_cases.get(0);

    // THEN: Use case handles orphan gracefully
    assert!(retrieved_use_case.is_some());
    match retrieved_use_case {
      Some(u) => {
        assert_eq!(u.persona_id, Some(fake_persona_id));
        // No foreign key validation - aggregate boundary
      }
      None => panic!("Use case not found"),
    }
  }

  #[test]
  fn round2_cross_entity_remove_persona_breaks_relationship() {
    // GIVEN: Scenario linked to persona
    let persona = Persona::new(
      "Charlie".to_string(),
      "Manager".to_string(),
      "Product manager persona".to_string(),
    );

    let mut scenario = NorthStarScenario::new(
      "Sprint Planning".to_string(),
      "Charlie plans the sprint".to_string(),
    );
    scenario = scenario.with_persona(persona.id);

    let state = PlannerState::new()
      .add_persona(persona.clone())
      .unwrap()
      .add_scenario(scenario)
      .unwrap();

    // Verify relationship exists
    let state_scenario = state.scenarios.get(0);
    assert_eq!(state_scenario.unwrap().persona_id, Some(persona.id));

    // WHEN: Remove persona
    let updated_state = state.remove_persona(persona.id);

    // THEN: Scenario retains persona_id but persona is gone
    assert_eq!(updated_state.personas.len(), 0);
    assert_eq!(updated_state.scenarios.len(), 1);

    let orphan_scenario = updated_state.scenarios.get(0);
    assert_eq!(orphan_scenario.unwrap().persona_id, Some(persona.id));

    // Persona truly removed
    let persona_exists = updated_state.personas.iter().any(|p| p.id == persona.id);
    assert!(!persona_exists);
  }

  #[test]
  fn round2_cross_entity_multiple_scenarios_same_persona() {
    // GIVEN: Single persona
    let persona = Persona::new(
      "Diana".to_string(),
      "Developer".to_string(),
      "Full-stack developer".to_string(),
    );

    // WHEN: Create multiple scenarios linked to same persona
    let mut scenario1 = NorthStarScenario::new(
      "Daily Standup".to_string(),
      "Diana attends standup".to_string(),
    );
    scenario1 = scenario1.with_persona(persona.id);

    let mut scenario2 =
      NorthStarScenario::new("Code Review".to_string(), "Diana reviews code".to_string());
    scenario2 = scenario2.with_persona(persona.id);

    let mut scenario3 = NorthStarScenario::new(
      "Deployment".to_string(),
      "Diana deploys to production".to_string(),
    );
    scenario3 = scenario3.with_persona(persona.id);

    let state = PlannerState::new()
      .add_persona(persona)
      .unwrap()
      .add_scenario(scenario1)
      .unwrap()
      .add_scenario(scenario2)
      .unwrap()
      .add_scenario(scenario3)
      .unwrap();

    // THEN: All scenarios reference same persona
    assert_eq!(state.personas.len(), 1);
    assert_eq!(state.scenarios.len(), 3);

    for scenario in state.scenarios.iter() {
      assert_eq!(scenario.persona_id, state.personas.get(0).map(|p| p.id));
    }
  }

  // -------------------------------------------------------------------------
  // State Persistence and Rollback (DDD - Event Sourcing patterns)
  // -------------------------------------------------------------------------

  #[test]
  fn round2_state_immutability_original_unchanged_after_update() {
    // GIVEN: Original state
    let original = PlannerState::new().update_project_name("Original Project".to_string());

    // WHEN: Make multiple changes
    let updated1 = original.update_thesis(ProductThesis::new(
      "Thesis".to_string(),
      "Problem".to_string(),
      "Audience".to_string(),
      "Solution".to_string(),
      "Value".to_string(),
    ));

    let updated2 = updated1
      .add_persona(Persona::new(
        "User".to_string(),
        "Role".to_string(),
        "Desc".to_string(),
      ))
      .unwrap();

    let updated3 = updated2
      .add_scenario(NorthStarScenario::new(
        "Scenario".to_string(),
        "Narrative".to_string(),
      ))
      .unwrap();

    // THEN: Original unchanged (immutability preserved)
    assert!(original.thesis.is_none());
    assert!(original.personas.is_empty());
    assert!(original.scenarios.is_empty());
    assert_eq!(original.context.project_name, "Original Project");

    // Each step creates new state
    assert!(updated1.thesis.is_some());
    assert!(updated1.personas.is_empty());

    assert!(updated2.thesis.is_some());
    assert_eq!(updated2.personas.len(), 1);

    assert!(updated3.thesis.is_some());
    assert_eq!(updated3.personas.len(), 1);
    assert_eq!(updated3.scenarios.len(), 1);
  }

  #[test]
  fn round2_state_rollback_by_discarding_new_state() {
    // GIVEN: State with valid data
    let persona = Persona::new("User".to_string(), "Role".to_string(), "Desc".to_string());
    let scenario = NorthStarScenario::new("Scenario".to_string(), "Narrative".to_string());

    let original = PlannerState::new()
      .add_persona(persona)
      .unwrap()
      .add_scenario(scenario)
      .unwrap();

    // WHEN: Make changes but discard them (keep original reference)
    let _discarded = original
      .remove_persona(original.personas.get(0).unwrap().id)
      .remove_scenario(original.scenarios.get(0).unwrap().id);

    // THEN: Original state unchanged (rollback by not using new state)
    assert_eq!(original.personas.len(), 1);
    assert_eq!(original.scenarios.len(), 1);
  }

  #[test]
  fn round2_state_branching_multiple_futures() {
    // GIVEN: Base state
    let base = PlannerState::new()
      .add_persona(Persona::new(
        "User".to_string(),
        "Role".to_string(),
        "Desc".to_string(),
      ))
      .unwrap();

    // WHEN: Create multiple branches from same base
    let branch1 = base
      .add_scenario(NorthStarScenario::new(
        "Scenario1".to_string(),
        "Narrative1".to_string(),
      ))
      .unwrap();

    let branch2 = base
      .add_scenario(NorthStarScenario::new(
        "Scenario2".to_string(),
        "Narrative2".to_string(),
      ))
      .unwrap();

    let branch3 = base
      .add_use_case(UseCase::new(
        "UseCase".to_string(),
        "Desc".to_string(),
        "Trigger".to_string(),
      ))
      .unwrap();

    // THEN: All branches independent
    assert_eq!(base.scenarios.len(), 0);
    assert_eq!(base.use_cases.len(), 0);

    assert_eq!(branch1.scenarios.len(), 1);
    assert_eq!(branch1.scenarios.get(0).unwrap().title, "Scenario1");
    assert_eq!(branch1.use_cases.len(), 0);

    assert_eq!(branch2.scenarios.len(), 1);
    assert_eq!(branch2.scenarios.get(0).unwrap().title, "Scenario2");
    assert_eq!(branch2.use_cases.len(), 0);

    assert_eq!(branch3.scenarios.len(), 0);
    assert_eq!(branch3.use_cases.len(), 1);
  }

  #[test]
  fn round2_state_error_rejection_preserves_state() {
    // GIVEN: State at max capacity
    let mut state = PlannerState::new();
    for i in 0..MAX_COLLECTION_SIZE {
      let persona = Persona::new(
        format!("Persona{}", i),
        format!("Role{}", i),
        format!("Description{}", i),
      );
      state = state.add_persona(persona).unwrap();
    }

    assert_eq!(state.personas.len(), MAX_COLLECTION_SIZE);

    // WHEN: Attempt invalid operation (exceed max)
    let result = state.add_persona(Persona::new(
      "Extra".to_string(),
      "Role".to_string(),
      "Desc".to_string(),
    ));

    // THEN: State rejects change, original preserved
    assert!(result.is_err());
    match result {
      Err(StateError::CollectionTooLarge) => {}
      _ => panic!("Expected CollectionTooLarge error"),
    }
    assert_eq!(state.personas.len(), MAX_COLLECTION_SIZE);
  }

  // -------------------------------------------------------------------------
  // Collection-Level Invariants (DDD - Aggregate Rules)
  // -------------------------------------------------------------------------

  #[test]
  fn round2_collection_max_capacity_all_entity_types() {
    // GIVEN: State at max capacity for all collections
    let mut state = PlannerState::new();

    // Fill personas to max
    for i in 0..MAX_COLLECTION_SIZE {
      let persona = Persona::new(
        format!("Persona{}", i),
        "Role".to_string(),
        "Desc".to_string(),
      );
      state = state.add_persona(persona).unwrap();
    }

    // Fill scenarios to max
    for i in 0..MAX_COLLECTION_SIZE {
      let scenario = NorthStarScenario::new(format!("Scenario{}", i), "Narrative".to_string());
      state = state.add_scenario(scenario).unwrap();
    }

    // Fill use cases to max
    for i in 0..MAX_COLLECTION_SIZE {
      let use_case = UseCase::new(
        format!("UseCase{}", i),
        "Desc".to_string(),
        "Trigger".to_string(),
      );
      state = state.add_use_case(use_case).unwrap();
    }

    // Fill tasks to max
    for i in 0..MAX_COLLECTION_SIZE {
      let task = PlanTask::new(
        format!("Task{}", i),
        "Desc".to_string(),
        TaskType::Development,
        DiamondPhase::Bottom,
      );
      state = state.add_task(task).unwrap();
    }

    // WHEN: Attempt to add any entity
    let result_persona = state.add_persona(Persona::new(
      "Extra".to_string(),
      "Role".to_string(),
      "Desc".to_string(),
    ));
    let result_scenario = state.add_scenario(NorthStarScenario::new(
      "Extra".to_string(),
      "Narrative".to_string(),
    ));
    let result_use_case = state.add_use_case(UseCase::new(
      "Extra".to_string(),
      "Desc".to_string(),
      "Trigger".to_string(),
    ));
    let result_task = state.add_task(PlanTask::new(
      "Extra".to_string(),
      "Desc".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    ));

    // THEN: All return errors
    assert!(result_persona.is_err());
    assert!(result_scenario.is_err());
    assert!(result_use_case.is_err());
    assert!(result_task.is_err());

    // Verify state unchanged
    assert_eq!(state.personas.len(), MAX_COLLECTION_SIZE);
    assert_eq!(state.scenarios.len(), MAX_COLLECTION_SIZE);
    assert_eq!(state.use_cases.len(), MAX_COLLECTION_SIZE);
    assert_eq!(state.tasks.len(), MAX_COLLECTION_SIZE);
  }

  #[test]
  fn round2_collection_duplicate_id_rejection_all_types() {
    // GIVEN: State with entities
    let persona1 = Persona::new("User".to_string(), "Role".to_string(), "Desc".to_string());
    let scenario1 = NorthStarScenario::new("Scenario".to_string(), "Narrative".to_string());
    let use_case1 = UseCase::new(
      "UseCase".to_string(),
      "Desc".to_string(),
      "Trigger".to_string(),
    );
    let task1 = PlanTask::new(
      "Task".to_string(),
      "Desc".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    );

    let state = PlannerState::new()
      .add_persona(persona1.clone())
      .unwrap()
      .add_scenario(scenario1.clone())
      .unwrap()
      .add_use_case(use_case1.clone())
      .unwrap()
      .add_task(task1.clone())
      .unwrap();

    // WHEN: Attempt to add entities with duplicate IDs
    let result_persona = state.add_persona(persona1.clone());
    let result_scenario = state.add_scenario(scenario1.clone());
    let result_use_case = state.add_use_case(use_case1.clone());
    let result_task = state.add_task(task1.clone());

    // THEN: All return duplicate ID errors
    assert!(result_persona.is_err());
    assert!(result_scenario.is_err());
    assert!(result_use_case.is_err());
    assert!(result_task.is_err());

    // Verify state unchanged
    assert_eq!(state.personas.len(), 1);
    assert_eq!(state.scenarios.len(), 1);
    assert_eq!(state.use_cases.len(), 1);
    assert_eq!(state.tasks.len(), 1);
  }

  #[test]
  fn round2_collection_atomicity_update_all_or_none() {
    // GIVEN: State with entities
    let persona1 = Persona::new(
      "User1".to_string(),
      "Role1".to_string(),
      "Desc1".to_string(),
    );
    let persona2 = Persona::new(
      "User2".to_string(),
      "Role2".to_string(),
      "Desc2".to_string(),
    );
    let persona3 = Persona::new(
      "User3".to_string(),
      "Role3".to_string(),
      "Desc3".to_string(),
    );

    let state = PlannerState::new()
      .add_persona(persona1)
      .unwrap()
      .add_persona(persona2)
      .unwrap();

    // WHEN: Bulk update with duplicate IDs within the update itself (should fail atomically)
    // persona3 appears twice in the update list
    let personas_with_internal_dup = vec![persona3.clone(), persona3.clone()];
    let result = state.update_personas(personas_with_internal_dup);

    // THEN: Entire operation rejected (atomicity)
    assert!(result.is_err());
    match result {
      Err(StateError::DuplicateId(_)) => {}
      _ => panic!("Expected DuplicateId error"),
    }

    // Verify state unchanged (atomicity preserved)
    assert_eq!(state.personas.len(), 2);
  }

  #[test]
  fn round2_collection_update_replaces_all_entities() {
    // GIVEN: State with existing entities
    let persona1 = Persona::new(
      "User1".to_string(),
      "Role1".to_string(),
      "Desc1".to_string(),
    );
    let persona2 = Persona::new(
      "User2".to_string(),
      "Role2".to_string(),
      "Desc2".to_string(),
    );

    let state = PlannerState::new()
      .add_persona(persona1)
      .unwrap()
      .add_persona(persona2)
      .unwrap();

    assert_eq!(state.personas.len(), 2);

    // WHEN: Update with entirely new collection
    let persona3 = Persona::new(
      "User3".to_string(),
      "Role3".to_string(),
      "Desc3".to_string(),
    );
    let persona4 = Persona::new(
      "User4".to_string(),
      "Role4".to_string(),
      "Desc4".to_string(),
    );

    let result = state.update_personas(vec![persona3.clone(), persona4.clone()]);

    // THEN: All entities replaced
    assert!(result.is_ok());
    let updated = result.unwrap();
    assert_eq!(updated.personas.len(), 2);

    // Old entities gone
    let has_old1 = updated.personas.iter().any(|p| p.name == "User1");
    let has_old2 = updated.personas.iter().any(|p| p.name == "User2");
    assert!(!has_old1);
    assert!(!has_old2);

    // New entities present
    let has_new3 = updated.personas.iter().any(|p| p.name == "User3");
    let has_new4 = updated.personas.iter().any(|p| p.name == "User4");
    assert!(has_new3);
    assert!(has_new4);

    // Original state unchanged
    assert_eq!(state.personas.len(), 2);
    let state_has_old1 = state.personas.iter().any(|p| p.name == "User1");
    assert!(state_has_old1);
  }

  // -------------------------------------------------------------------------
  // Domain Event Sequences (BDT - User Journeys)
  // -------------------------------------------------------------------------

  #[test]
  fn round2_workflow_complete_discovery_phase() {
    // GIVEN: New project (empty state)
    let state = PlannerState::new();
    assert_eq!(state.current_phase, DiamondPhase::Top);
    assert!(!state.can_advance());

    // WHEN: Complete full discovery workflow in order
    // Step 1: Add thesis
    let thesis = ProductThesis::new(
      "Build Project Planning Tool".to_string(),
      "Teams struggle to organize complex projects".to_string(),
      "Software development teams".to_string(),
      "AI-powered planning assistant".to_string(),
      "Reduce planning overhead by 50%".to_string(),
    );
    let state = state.update_thesis(thesis);
    assert!(!state.can_advance()); // Still missing personas and scenarios

    // Step 2: Add first persona
    let persona1 = Persona::new(
      "Tech Lead".to_string(),
      "Engineering Lead".to_string(),
      "Leads development teams".to_string(),
    );
    let state = state.add_persona(persona1).unwrap();
    assert!(!state.can_advance()); // Still missing scenarios

    // Step 3: Add second persona (optional but valid)
    let persona2 = Persona::new(
      "Product Manager".to_string(),
      "PM".to_string(),
      "Defines product strategy".to_string(),
    );
    let state = state.add_persona(persona2).unwrap();
    assert!(!state.can_advance()); // Still missing scenarios

    // Step 4: Add scenario (completes discovery)
    let scenario = NorthStarScenario::new(
      "Project Initialization".to_string(),
      "Tech Lead creates new project with AI assistance".to_string(),
    );
    let state = state.add_scenario(scenario).unwrap();

    // THEN: Can advance to next phase
    assert!(state.can_advance());
    assert_eq!(state.personas.len(), 2);
    assert_eq!(state.scenarios.len(), 1);
    assert!(state.thesis.is_some());

    // Transition successful
    let result = state.next_phase();
    assert!(result.is_ok());
    assert_eq!(result.unwrap().current_phase, DiamondPhase::Right);
  }

  #[test]
  fn round2_workflow_complete_all_phases() {
    // GIVEN: New project
    let mut state = PlannerState::new();

    // WHEN: Complete full workflow through all phases

    // Discovery phase
    let thesis = ProductThesis::new(
      "Thesis".to_string(),
      "Problem".to_string(),
      "Audience".to_string(),
      "Solution".to_string(),
      "Value".to_string(),
    );
    state = state.update_thesis(thesis);

    let persona = Persona::new("User".to_string(), "Role".to_string(), "Desc".to_string());
    state = state.add_persona(persona).unwrap();

    let scenario = NorthStarScenario::new("Scenario".to_string(), "Narrative".to_string());
    state = state.add_scenario(scenario).unwrap();

    state = state.next_phase().unwrap(); // Discovery -> Design
    assert_eq!(state.current_phase, DiamondPhase::Right);

    // Design phase
    let use_case = UseCase::new(
      "Use Case".to_string(),
      "Desc".to_string(),
      "Trigger".to_string(),
    );
    state = state.add_use_case(use_case).unwrap();

    state = state.next_phase().unwrap(); // Design -> Development
    assert_eq!(state.current_phase, DiamondPhase::Bottom);

    // Development phase
    let task = PlanTask::new(
      "Task".to_string(),
      "Desc".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    );
    state = state.add_task(task).unwrap();

    state = state.next_phase().unwrap(); // Development -> Delivery
    assert_eq!(state.current_phase, DiamondPhase::Left);

    // THEN: All phases completed, at delivery
    assert!(!state.can_advance()); // Cannot advance past delivery
    assert_eq!(state.progress(), 1.0);
  }

  #[test]
  fn round2_workflow_task_completion_affects_validation() {
    // GIVEN: Project in Development phase with tasks
    let thesis = ProductThesis::new(
      "Thesis".to_string(),
      "Problem".to_string(),
      "Audience".to_string(),
      "Solution".to_string(),
      "Value".to_string(),
    );
    let persona = Persona::new("User".to_string(), "Role".to_string(), "Desc".to_string());
    let scenario = NorthStarScenario::new("Scenario".to_string(), "Narrative".to_string());
    let use_case = UseCase::new(
      "Use Case".to_string(),
      "Desc".to_string(),
      "Trigger".to_string(),
    );

    let mut state = PlannerState::new()
      .update_thesis(thesis)
      .add_persona(persona)
      .unwrap()
      .add_scenario(scenario)
      .unwrap()
      .next_phase()
      .unwrap()
      .add_use_case(use_case)
      .unwrap()
      .next_phase()
      .unwrap();

    // Add incomplete task (empty description)
    let incomplete_task = PlanTask::new(
      "Incomplete".to_string(),
      "".to_string(), // Empty - should fail validation
      TaskType::Development,
      DiamondPhase::Bottom,
    );
    state = state.add_task(incomplete_task).unwrap();

    // WHEN: Check if can advance
    let can_advance = state.can_advance();

    // THEN: Validation fails due to incomplete task
    assert!(!can_advance);

    // Fix the task
    let task_id = state.tasks.get(0).unwrap().id;
    let complete_task = PlanTask::new(
      "Complete Task".to_string(),
      "Now has description".to_string(),
      TaskType::Development,
      DiamondPhase::Bottom,
    );
    state = state.update_task(task_id, complete_task);

    // Now validation passes
    assert!(state.can_advance());
  }

  #[test]
  fn round2_workflow_interleaved_entity_creation() {
    // GIVEN: New project
    let mut state = PlannerState::new();

    // WHEN: Interleave entity creation (not strictly sequential)
    // Add thesis
    state = state.update_thesis(ProductThesis::new(
      "Thesis".to_string(),
      "Problem".to_string(),
      "Audience".to_string(),
      "Solution".to_string(),
      "Value".to_string(),
    ));

    // Add persona
    state = state
      .add_persona(Persona::new(
        "User".to_string(),
        "Role".to_string(),
        "Desc".to_string(),
      ))
      .unwrap();

    // Add use case (preemptive, before phase transition)
    state = state
      .add_use_case(UseCase::new(
        "Use Case".to_string(),
        "Desc".to_string(),
        "Trigger".to_string(),
      ))
      .unwrap();

    // Add scenario
    state = state
      .add_scenario(NorthStarScenario::new(
        "Scenario".to_string(),
        "Narrative".to_string(),
      ))
      .unwrap();

    // Add task (preemptive)
    state = state
      .add_task(PlanTask::new(
        "Task".to_string(),
        "Desc".to_string(),
        TaskType::Development,
        DiamondPhase::Bottom,
      ))
      .unwrap();

    // THEN: State accepts all entities, can advance through phases
    assert_eq!(state.current_phase, DiamondPhase::Top);
    assert!(state.can_advance()); // Discovery complete

    state = state.next_phase().unwrap(); // Discovery -> Design
    assert!(state.can_advance()); // Already has use case

    state = state.next_phase().unwrap(); // Design -> Development
    assert!(state.can_advance()); // Already has task

    state = state.next_phase().unwrap(); // Development -> Delivery
    assert_eq!(state.current_phase, DiamondPhase::Left);
  }

  #[test]
  fn round2_workflow_phase_retreat_and_resume() {
    // GIVEN: Project in Design phase
    let thesis = ProductThesis::new(
      "Thesis".to_string(),
      "Problem".to_string(),
      "Audience".to_string(),
      "Solution".to_string(),
      "Value".to_string(),
    );
    let persona = Persona::new("User".to_string(), "Role".to_string(), "Desc".to_string());
    let scenario = NorthStarScenario::new("Scenario".to_string(), "Narrative".to_string());

    let mut state = PlannerState::new()
      .update_thesis(thesis)
      .add_persona(persona)
      .unwrap()
      .add_scenario(scenario)
      .unwrap()
      .next_phase()
      .unwrap();

    assert_eq!(state.current_phase, DiamondPhase::Right);

    // WHEN: Retreat to Discovery, add more entities, then advance again
    state = state.prev_phase();
    assert_eq!(state.current_phase, DiamondPhase::Top);

    // Add another persona
    state = state
      .add_persona(Persona::new(
        "User2".to_string(),
        "Role2".to_string(),
        "Desc2".to_string(),
      ))
      .unwrap();

    // Add another scenario
    state = state
      .add_scenario(NorthStarScenario::new(
        "Scenario2".to_string(),
        "Narrative2".to_string(),
      ))
      .unwrap();

    // Advance again
    assert!(state.can_advance());
    state = state.next_phase().unwrap();

    // THEN: State includes newly added entities
    assert_eq!(state.current_phase, DiamondPhase::Right);
    assert_eq!(state.personas.len(), 2);
    assert_eq!(state.scenarios.len(), 2);
  }
}
