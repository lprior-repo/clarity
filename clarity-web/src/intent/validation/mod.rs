//! Validation Submodule
//!
//! Spec validation and linting including:
//! - Structure validation
//! - Circular dependency detection
//! - Semantic validation
//! - Variable interpolation
//! - Validation rule engine

pub mod spec_validator;

// Re-export key types for convenience
pub use spec_validator::{
    has_circular_dependencies, feature_execution_order, validate_spec,
    BehaviorPriority, DependencyGraph, SpecValidationError, SpecValidator, ValidationResult,
    ValidationWarning,
};

// Modules will be added in WP20-WP21, WP31
// pub mod semantic_validator;
// pub mod validator;
// pub mod rule;
// pub mod interpolation;
