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
pub mod spec_validator;

// Re-export key types for convenience
pub use spec_validator::{
    has_circular_dependencies, feature_execution_order, validate_spec,
    BehaviorPriority, DependencyGraph, SpecValidationError, SpecValidator, ValidationResult,
    ValidationWarning,
};

// Re-export interpolation types (WP20)
pub use interpolation::{
    interpolate_string, resolve_path, has_placeholders, extract_variables, validate_variables,
    Context, InterpolationError,
};

// Re-export rule types (WP21)
pub use rule::{
    apply_rule, validate_with_rules, all_rules_pass, failing_rules,
    Rule, RuleError, RuleResult,
};
