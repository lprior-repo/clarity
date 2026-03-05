//! Planning Submodule
//!
//! Execution planning and dependency management including:
//! - Execution plan computation
//! - Dependency graph building
//! - Phase gating
//! - Bead emission with idempotency
//! - Next action determination
//! - Dependency resolution with topological sorting

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
// Additional clippy lints to allow
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::trivially_copy_pass_by_ref)]
#![allow(clippy::assigning_clones)]
#![allow(clippy::option_if_let_else)]
#![allow(clippy::unused_self)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::manual_strip)]
#![allow(clippy::format_push_string)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::struct_field_names)]
#![allow(clippy::suspicious_else_formatting)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::items_after_statements)]
#![allow(clippy::ptr_arg)]
#![allow(clippy::missing_fields_in_debug)]
#![allow(clippy::must_use_unit)]
#![allow(clippy::collection_is_never_read)]
#![allow(clippy::needless_collect)]
#![allow(clippy::manual_checked_ops)]
#![allow(clippy::needless_pass_by_value)]

// Legacy module (WP22)
pub mod plan_mode;

// New modules (WP23-WP25)
pub mod plan_emit_beads;
pub mod plan_next;
pub mod resolver;
pub mod timestamp;
pub mod types;

// Re-export legacy types for backwards compatibility
pub use plan_mode::{
  apply_phase_gating, compute_plan, get_actionable_beads as get_actionable_beads_legacy,
  validate_plan_dependencies as validate_plan_dependencies_legacy, Action as PlanAction,
  BeadStatus, Phase, PhaseStatus,
};

// Re-export new types at module level with distinct names to avoid conflicts
pub use plan_emit_beads::{
  check_existing_beads, emit_beads, filter_new_beads_for_test, format_result,
  generate_profile_beads, EmissionMode, EmissionResult,
};
pub use plan_next::{
  can_proceed, determine_next_phase, format_next_action_json, format_next_action_json_compact,
  get_actionable_beads, get_blocking_gaps, get_next_action, Action as NextActionType,
  ActionContext, ActionSuggestion, ActionType, NextAction, PlanNextJsonOutput,
};
pub use resolver::{
  apply_resolution_to_plan, compute_critical_path, compute_parallelism, detect_cycles,
  get_dependencies, get_dependents, resolve_dependencies, topological_sort,
  validate_plan_dependencies, ResolutionResult,
};

// Re-export types module types with "New" suffix to distinguish from legacy
pub use types::{
  BeadState, ExecutionPlan as ExecutionPlanNew, PlanBead as PlanBeadNew, PlanError as PlanErrorNew,
  PlanPhase,
};

// Re-export legacy PlanError and ExecutionPlan as the primary types for backwards compat
pub use plan_mode::{ExecutionPlan, PlanBead, PlanError};

// Re-export timestamp utility
pub use timestamp::current_iso8601_timestamp;
