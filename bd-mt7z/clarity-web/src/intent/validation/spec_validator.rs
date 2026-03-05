#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

mod categorize;
mod errors;
mod graph;
mod priority;
mod result;
mod validator;

use crate::intent::types::Spec;

pub use errors::SpecValidationError;
pub use graph::DependencyGraph;
pub use priority::BehaviorPriority;
pub use result::{ValidationResult, ValidationWarning};
pub use validator::{SpecValidator, ValidationChecks};

#[must_use]
pub fn validate_spec(spec: &Spec) -> ValidationResult {
  SpecValidator::new().validate(spec)
}

#[must_use]
pub fn has_circular_dependencies(spec: &Spec) -> bool {
  let validator = SpecValidator::new();
  validator
    .build_feature_dependency_graph(spec)
    .detect_cycles()
    .is_some()
}

#[must_use]
pub fn feature_execution_order(spec: &Spec) -> Option<Vec<String>> {
  let validator = SpecValidator::new();
  validator
    .build_feature_dependency_graph(spec)
    .topological_sort()
}
