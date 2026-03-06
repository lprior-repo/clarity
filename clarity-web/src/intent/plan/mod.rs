//! Planning Submodule
//!
//! Execution planning and dependency management including:
//! - Execution plan computation
//! - Dependency graph building
//! - Phase gating
//! - Bead emission with idempotency
//! - Next action determination
//! - Dependency resolution with topological sorting

#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

// Legacy module (WP22)
pub mod plan_mode;

// New modules (WP23-WP25)
pub mod plan_emit_beads;
pub mod plan_next;
pub mod resolver;
pub mod types;

// Re-export legacy types for backwards compatibility
pub use plan_mode::{
  apply_phase_gating, compute_plan, get_actionable_beads as get_actionable_beads_legacy,
  validate_plan_dependencies as validate_plan_dependencies_legacy, BeadStatus, Phase, PhaseStatus,
};

// Re-export new types at module level with distinct names to avoid conflicts
pub use plan_emit_beads::{
  check_existing_beads, emit_beads, generate_profile_beads, EmissionResult,
};
pub use plan_next::{
  can_proceed, determine_next_phase, get_actionable_beads, get_blocking_gaps, get_next_action,
  ActionType, NextAction,
};
pub use resolver::{
  apply_resolution_to_plan, compute_critical_path, compute_parallelism, detect_cycles,
  get_dependencies, get_dependents, resolve_dependencies, topological_sort,
  validate_plan_dependencies, ResolutionResult,
};

// Re-export types module types with "New" suffix to distinguish from legacy
pub use types::{
  ExecutionPlan as ExecutionPlanNew, PlanBead as PlanBeadNew, PlanError as PlanErrorNew, PlanPhase,
};

// Re-export legacy PlanError and ExecutionPlan as the primary types for backwards compat
pub use plan_mode::{ExecutionPlan, PlanBead, PlanError};
