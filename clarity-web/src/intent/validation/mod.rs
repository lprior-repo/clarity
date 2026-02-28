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
pub mod spec_validator;

#[cfg(test)]
mod semantic_adversarial;

// Re-export key types for convenience
pub use spec_validator::{
  feature_execution_order, has_circular_dependencies, validate_spec, BehaviorPriority,
  DependencyGraph, SpecValidationError, SpecValidator, ValidationChecks, ValidationResult,
  ValidationWarning,
};

// Re-export interpolation types (WP20)
pub use interpolation::{
  extract_capture, extract_variables, has_placeholders, interpolate_headers, interpolate_string,
  json_to_string, resolve_path, validate_variables, Context, InterpolationError,
};

// Re-export rule types (WP21)
pub use rule::{
  all_rules_pass, apply_rule, failing_rules, validate_with_rules, Rule, RuleError, RuleResult,
};

// Re-export semantic validator types (WP31)
pub use semantic::{
  consistency_checks, cross_reference_validation, validate_semantics, CrossReferenceResult,
  SemanticError, SemanticResult, SemanticValidationResult, SemanticValidator, TerminologyCheck,
};
