use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum RuleError {
  #[error("rule '{0}' failed: {1}")]
  RuleFailed(String, String),
  #[error("pattern '{pattern}' did not match value '{value}'")]
  PatternMismatch { pattern: String, value: String },
  #[error("value {value} out of range [{min}, {max}]")]
  OutOfRange { value: f64, min: f64, max: f64 },
  #[error("invalid regex pattern: {0}")]
  InvalidPattern(String),
  #[error("not a number: {0}")]
  NotANumber(String),
  #[error("custom rule '{name}' failed: {message}")]
  CustomFailed { name: String, message: String },
}
