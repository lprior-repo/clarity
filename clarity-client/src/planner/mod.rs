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
pub mod state;
pub mod types;
pub mod validation;

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
