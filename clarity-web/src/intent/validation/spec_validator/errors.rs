use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum SpecValidationError {
  #[error("missing required field: {0}")]
  MissingRequiredField(String),
  #[error("invalid field '{field}': expected {expected}, got {actual}")]
  InvalidFieldType {
    field: String,
    expected: String,
    actual: String,
  },
  #[error("circular dependency detected in behavior '{behavior_a}' -> behavior '{behavior_b}'")]
  CircularDependency {
    behavior_a: String,
    behavior_b: String,
  },
  #[error("circular dependency detected: cycle path {}", path.join(" -> "))]
  CircularDependencyPath { path: Vec<String> },
  #[error("duplicate behavior detected: behavior '{behavior_a}' is duplicated with behavior '{behavior_b}'")]
  DuplicateBehavior {
    behavior_a: String,
    behavior_b: String,
    description: String,
    impact: String,
  },
  #[error("unknown dependency: '{0}' references non-existent behavior or feature")]
  UnknownDependency(String),
  #[error("validation failed with {count} errors")]
  MultipleValidationErrors {
    count: usize,
    errors: Vec<SpecValidationError>,
  },
}
