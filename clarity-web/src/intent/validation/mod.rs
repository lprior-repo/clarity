//! Validation Submodule
//!
//! Spec validation and linting including:
//! - Structure validation
//! - Circular dependency detection
//! - Semantic validation
//! - Variable interpolation
//! - Validation rule engine

pub mod interpolation;
pub mod rule;
pub mod semantic;
#[cfg(test)]
pub mod semantic_bdd_tests;
pub mod spec_validator;

// Re-export key types for convenience
pub use spec_validator::{
  feature_execution_order, has_circular_dependencies, validate_spec, BehaviorPriority,
  DependencyGraph, SpecValidationError, SpecValidator, ValidationResult, ValidationWarning,
};

// Re-export interpolation types (WP20)
pub use interpolation::{
  extract_variables, has_placeholders, interpolate_string, resolve_path, validate_variables,
  Context, InterpolationError,
};

// Re-export rule types (WP21)
pub use rule::{
  all_rules_pass, apply_rule, failing_rules, validate_with_rules, Rule, RuleError, RuleResult,
};
