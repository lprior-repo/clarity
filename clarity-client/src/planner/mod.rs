//! Planner module - Diamond design methodology implementation
//!
//! This module implements the Diamond design methodology for product planning,
//! organized into four phases (top, right, bottom, left of the diamond).

pub mod application;
pub mod domain;
pub mod presentation;

pub mod adversarial_tests;
pub mod components;
pub mod hostile_attacks;
pub mod parser;
// pub mod prompts;  // Temporarily disabled - used by coach component
pub mod state;
pub mod tests;
pub mod types;
// pub mod types_coach;  // Temporarily disabled - used by coach component
pub mod validation;

pub use parser::{parse_use_case, parse_use_case_json, parse_use_cases, validate_use_case};
pub use state::{PlannerContext, PlannerState, PlannerTab, PlannerUIState, SelectedEntity};
pub use types::{
  Contracts, DiamondPhase, EarsRequirements, EarsValue, Effort, GraphHealth, Implementation,
  NorthStarScenario, Persona, PlanSession, PlanTask, ProductThesis, Research, StateError,
  TaskPriority, TaskType, Tests, UseCase, UseCasePriority, ValidationCheck, ValidationSeverity,
  COMPLETED_EPSILON, MAX_COLLECTION_SIZE, MAX_DEPTH,
};
pub use validation::{
  detect_cycles, detect_cycles_with_path, get_graph_health, is_task_ready, validate_all_tasks,
  validate_task, CycleInfo, ValidationError,
};

// DDD-friendly exports
pub use application as app;
pub use domain as core;
pub use presentation as ui;
