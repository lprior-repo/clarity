//! Quality types and validation system for clarity-core
//!
//! This module provides comprehensive quality metrics, validation reports,
//! and composable validators with functional error handling.
//!
//! # Core Types
//! - [`QualityScore`]: Represents quality metrics (0.0-1.0)
//! - [`ValidationReport`]: Aggregates multiple validation results
//! - [`QualityMetrics`]: Tracks code quality indicators
//! - [`Severity`]: Error severity levels
//!
//! # Invariants
//! - All validators return `Result<T, ValidationError>`
//! - Zero unwrap/expect/panic
//! - Immutable validation results
//! - Thread-safe (Send + Sync)
//! - No side effects in validators

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use crate::validation::ValidationError;

/// Severity level for validation messages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Severity {
  /// Info message - logged but doesn't block operation
  Info,
  /// Warning - allows operation but notifies user
  Warning,
  /// Error - prevents operation
  Error,
}

impl fmt::Display for Severity {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Info => write!(f, "info"),
      Self::Warning => write!(f, "warning"),
      Self::Error => write!(f, "error"),
    }
  }
}

/// Quality score representing a metric value between 0.0 and 1.0
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct QualityScore(f64);

impl QualityScore {
  /// Default passing threshold
  const DEFAULT_THRESHOLD: f64 = 0.7;

  /// Create a new quality score with validation
  ///
  /// # Errors
  ///
  /// Returns `ValidationError::InvalidRange` if score is outside [0.0, 1.0]
  pub fn new(score: f64) -> Result<Self, ValidationError> {
    if score >= 0.0 && score <= 1.0 {
      Ok(Self(score))
    } else {
      Err(ValidationError::InvalidFormat {
        reason: format!("Quality score {score} is outside valid range [0.0, 1.0]"),
      })
    }
  }

  /// Get the underlying score value
  #[must_use]
  pub const fn value(self) -> f64 {
    self.0
  }

  /// Check if this score meets the passing threshold (>= 0.7)
  #[must_use]
  pub fn is_passing(self) -> bool {
    self.0 >= Self::DEFAULT_THRESHOLD
  }

  /// Check if this score is failing (< 0.7)
  #[must_use]
  pub fn is_failing(self) -> bool {
    !self.is_passing()
  }

  /// Check if score is excellent (>= 0.9)
  #[must_use]
  pub fn is_excellent(self) -> bool {
    self.0 >= 0.9
  }

  /// Check if score is poor (< 0.5)
  #[must_use]
  pub fn is_poor(self) -> bool {
    self.0 < 0.5
  }
}

impl fmt::Display for QualityScore {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{:.2}", self.0)
  }
}

/// A single validation result with context
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationMessage {
  /// Severity level
  severity: Severity,
  /// Field path (e.g., "user.profile.email")
  field_path: String,
  /// Error or warning message
  message: String,
}

impl ValidationMessage {
  /// Create a new validation message
  #[must_use]
  pub fn new(severity: Severity, field_path: String, message: String) -> Self {
    Self {
      severity,
      field_path,
      message,
    }
  }

  /// Get the severity level
  #[must_use]
  pub const fn severity(&self) -> Severity {
    self.severity
  }

  /// Get the field path
  #[must_use]
  pub fn field_path(&self) -> &str {
    &self.field_path
  }

  /// Get the message
  #[must_use]
  pub fn message(&self) -> &str {
    &self.message
  }

  /// Check if this is an error
  #[must_use]
  pub const fn is_error(&self) -> bool {
    matches!(self.severity, Severity::Error)
  }

  /// Check if this is a warning
  #[must_use]
  pub const fn is_warning(&self) -> bool {
    matches!(self.severity, Severity::Warning)
  }

  /// Check if this is info
  #[must_use]
  pub const fn is_info(&self) -> bool {
    matches!(self.severity, Severity::Info)
  }
}

impl fmt::Display for ValidationMessage {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{}: {}: {}",
      self.severity, self.field_path, self.message
    )
  }
}

/// Validation report aggregating multiple validation results
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationReport {
  /// All validation messages
  messages: Vec<ValidationMessage>,
  /// Overall validity (true if no errors)
  is_valid: bool,
}

impl ValidationReport {
  /// Create a new validation report from messages
  #[must_use]
  pub fn new(messages: Vec<ValidationMessage>) -> Self {
    let is_valid = !messages.iter().any(|m| m.is_error());
    Self { messages, is_valid }
  }

  /// Create an empty (valid) report
  #[must_use]
  pub fn valid() -> Self {
    Self {
      messages: Vec::new(),
      is_valid: true,
    }
  }

  /// Aggregate multiple reports into one
  #[must_use]
  pub fn aggregate(reports: Vec<Self>) -> Self {
    let all_messages: Vec<ValidationMessage> =
      reports.into_iter().flat_map(|r| r.messages).collect();

    Self::new(all_messages)
  }

  /// Check if validation passed (no errors)
  #[must_use]
  pub const fn is_valid(&self) -> bool {
    self.is_valid
  }

  /// Get all error messages
  #[must_use]
  pub fn errors(&self) -> Vec<&ValidationMessage> {
    self.messages.iter().filter(|m| m.is_error()).collect()
  }

  /// Get all warning messages
  #[must_use]
  pub fn warnings(&self) -> Vec<&ValidationMessage> {
    self.messages.iter().filter(|m| m.is_warning()).collect()
  }

  /// Get all info messages
  #[must_use]
  pub fn info(&self) -> Vec<&ValidationMessage> {
    self.messages.iter().filter(|m| m.is_info()).collect()
  }

  /// Get all messages
  #[must_use]
  pub fn messages(&self) -> &[ValidationMessage] {
    &self.messages
  }

  /// Get the number of errors
  #[must_use]
  pub fn error_count(&self) -> usize {
    self.messages.iter().filter(|m| m.is_error()).count()
  }

  /// Get the number of warnings
  #[must_use]
  pub fn warning_count(&self) -> usize {
    self.messages.iter().filter(|m| m.is_warning()).count()
  }

  /// Get the number of info messages
  #[must_use]
  pub fn info_count(&self) -> usize {
    self.messages.iter().filter(|m| m.is_info()).count()
  }

  /// Check if report has any messages
  #[must_use]
  pub fn is_empty(&self) -> bool {
    self.messages.is_empty()
  }

  /// Convert report to JSON representation
  #[must_use]
  pub fn to_json(&self) -> String {
    let messages: Vec<String> = self
      .messages
      .iter()
      .map(|m| {
        format!(
          r#"{{"severity":"{}","field_path":"{}","message":"{}"}}"#,
          m.severity,
          escape_json(&m.field_path),
          escape_json(&m.message)
        )
      })
      .collect();

    format!(
      r#"{{"valid":{},"messages":[{}]}}"#,
      self.is_valid,
      messages.join(",")
    )
  }
}

impl fmt::Display for ValidationReport {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if self.is_valid() {
      if self.is_empty() {
        write!(f, "Validation passed")
      } else {
        writeln!(f, "Validation passed with messages:")?;
        for msg in &self.messages {
          writeln!(f, "  {}", msg)?;
        }
        Ok(())
      }
    } else {
      writeln!(f, "Validation failed:")?;
      for msg in &self.messages {
        writeln!(f, "  {}", msg)?;
      }
      Ok(())
    }
  }
}

/// Escape string for JSON
fn escape_json(s: &str) -> String {
  s.replace('\\', "\\\\")
    .replace('"', "\\\"")
    .replace('\n', "\\n")
    .replace('\r', "\\r")
    .replace('\t', "\\t")
}

/// Quality metrics for code analysis
#[derive(Debug, Clone, PartialEq)]
pub struct QualityMetrics {
  /// Test coverage (0.0-1.0)
  test_coverage: QualityScore,
  /// Cyclomatic complexity
  complexity: u32,
  /// Lines of code
  lines_of_code: usize,
  /// Additional custom metrics
  custom_metrics: HashMap<String, f64>,
}

impl QualityMetrics {
  /// Create new quality metrics
  ///
  /// # Errors
  ///
  /// Returns `ValidationError::InvalidRange` if test_coverage is outside [0.0, 1.0]
  pub fn new(
    test_coverage: f64,
    complexity: u32,
    lines_of_code: usize,
  ) -> Result<Self, ValidationError> {
    Ok(Self {
      test_coverage: QualityScore::new(test_coverage)?,
      complexity,
      lines_of_code,
      custom_metrics: HashMap::new(),
    })
  }

  /// Get test coverage score
  #[must_use]
  pub const fn test_coverage(&self) -> QualityScore {
    self.test_coverage
  }

  /// Get cyclomatic complexity
  #[must_use]
  pub const fn complexity(&self) -> u32 {
    self.complexity
  }

  /// Get lines of code
  #[must_use]
  pub const fn lines_of_code(&self) -> usize {
    self.lines_of_code
  }

  /// Calculate overall quality score (weighted average)
  ///
  /// Weights:
  /// - Test coverage: 50%
  /// - Complexity (inverted, normalized): 30%
  /// - Custom metrics: 20%
  #[must_use]
  pub fn quality_score(&self) -> QualityScore {
    // Test coverage weight: 50%
    let coverage_score = self.test_coverage.value() * 0.5;

    // Complexity score: 30% (lower is better, normalized to 0-1)
    // Assume complexity of 1 is perfect, 20 is poor
    let complexity_score = (1.0 - (self.complexity as f64 / 20.0).min(1.0)) * 0.3;

    // Base score from coverage and complexity
    let base_score = coverage_score + complexity_score;

    // If we have custom metrics, incorporate them (20% weight)
    if self.custom_metrics.is_empty() {
      // Without custom metrics, scale up to 0-1 range
      QualityScore::new((base_score / 0.8).min(1.0)).unwrap_or_else(|_| QualityScore(0.0))
    } else {
      // Average custom metrics and apply 20% weight
      let custom_avg: f64 =
        self.custom_metrics.values().sum::<f64>() / self.custom_metrics.len() as f64;
      let final_score = base_score + (custom_avg * 0.2);
      QualityScore::new(final_score.min(1.0)).unwrap_or_else(|_| QualityScore(0.0))
    }
  }

  /// Add a custom metric
  ///
  /// # Errors
  ///
  /// Returns `ValidationError::InvalidRange` if value is outside [0.0, 1.0]
  pub fn with_custom_metric(mut self, name: String, value: f64) -> Result<Self, ValidationError> {
    if value >= 0.0 && value <= 1.0 {
      self.custom_metrics.insert(name, value);
      Ok(self)
    } else {
      Err(ValidationError::InvalidFormat {
        reason: format!("Custom metric '{name}' value {value} is outside valid range [0.0, 1.0]"),
      })
    }
  }

  /// Get custom metrics
  #[must_use]
  pub fn custom_metrics(&self) -> &HashMap<String, f64> {
    &self.custom_metrics
  }
}

/// Custom validator with user-defined validation logic for strings
#[derive(Clone)]
pub struct CustomValidator {
  validator: ValidatorFnStr,
  error_message: String,
}

impl CustomValidator {
  /// Create a new custom validator
  ///
  /// The validator function should return:
  /// - `Ok(value)` if validation passes
  /// - `Err(ValidationError)` if validation fails
  #[must_use]
  pub fn new<F>(validator: F, error_message: String) -> Self
  where
    F: Fn(&str) -> Result<String, ValidationError> + Send + Sync + 'static,
  {
    Self {
      validator: Arc::new(validator),
      error_message,
    }
  }

  /// Validate input using the custom validator
  ///
  /// # Errors
  ///
  /// Returns the validation error from the custom validator or a custom error message
  pub fn validate(&self, input: &str) -> Result<String, ValidationError> {
    (self.validator)(input)
  }

  /// Get the error message
  #[must_use]
  pub fn error_message(&self) -> &str {
    &self.error_message
  }
}

/// Composable validator for strings with AND/OR logic
#[derive(Clone)]
pub enum Validator {
  /// Single validator function
  Single(ValidatorFnStr),
  /// AND composition: both validators must pass
  And(Box<Validator>, Box<Validator>),
  /// OR composition: at least one validator must pass
  Or(Box<Validator>, Box<Validator>),
}

/// Validator function type for strings
type ValidatorFnStr = Arc<dyn Fn(&str) -> Result<String, ValidationError> + Send + Sync>;

impl Validator {
  /// Create a single validator from a function
  #[must_use]
  pub fn single<F>(validator: F) -> Self
  where
    F: Fn(&str) -> Result<String, ValidationError> + Send + Sync + 'static,
  {
    Self::Single(Arc::new(validator))
  }

  /// Combine two validators with AND logic (both must pass)
  #[must_use]
  pub fn and(self, other: Validator) -> Self {
    Validator::And(Box::new(self), Box::new(other))
  }

  /// Combine two validators with OR logic (at least one must pass)
  #[must_use]
  pub fn or(self, other: Validator) -> Self {
    Validator::Or(Box::new(self), Box::new(other))
  }

  /// Validate input using this validator
  ///
  /// # Errors
  ///
  /// Returns `ValidationError` if validation fails
  pub fn validate(&self, input: &str) -> Result<String, ValidationError> {
    match self {
      Self::Single(validator) => (validator)(input),
      Self::And(left, right) => left
        .validate(input)
        .and_then(|validated| right.validate(&validated)),
      Self::Or(left, right) => {
        match left.validate(input) {
          Ok(result) => return Ok(result),
          Err(_) => {
            // Try right validator
          }
        }
        right
          .validate(input)
          .map_err(|_| ValidationError::InvalidFormat {
            reason: "Both validators in OR composition failed".to_string(),
          })
      }
    }
  }
}

#[cfg(test)]
mod tests {
  #![allow(clippy::unwrap_used)]
  #![allow(clippy::expect_used)]
  #![allow(clippy::panic)]
  #![allow(clippy::float_cmp)]
  #![allow(clippy::unnecessary_to_owned)]
  #![allow(clippy::manual_string_new)]
  use super::*;

  // Test 1: Should Create QualityScore From Valid Float
  #[test]
  fn test_quality_score_valid() {
    let result = QualityScore::new(0.85);
    assert!(result.is_ok());
    let score = result.unwrap();
    assert_eq!(score.value(), 0.85);
    assert!(score.is_passing());
    assert!(!score.is_failing());
    assert!(!score.is_excellent());
    assert!(!score.is_poor());
  }

  // Test 2: Should Reject QualityScore Outside Range
  #[test]
  fn test_quality_score_too_high() {
    let result = QualityScore::new(1.5);
    assert!(result.is_err());
    match result {
      Err(ValidationError::InvalidFormat { reason }) => {
        assert!(reason.contains("outside valid range"));
        assert!(reason.contains("[0.0, 1.0]"));
      }
      _ => panic!("Expected InvalidFormat error"),
    }
  }

  #[test]
  fn test_quality_score_negative() {
    let result = QualityScore::new(-0.1);
    assert!(result.is_err());
  }

  #[test]
  fn test_quality_score_boundaries() {
    assert!(QualityScore::new(0.0).is_ok());
    assert!(QualityScore::new(1.0).is_ok());
  }

  // Test 3: Should Aggregate Validation Results
  #[test]
  fn test_validation_report_aggregate() {
    let msg1 = ValidationMessage::new(Severity::Error, "title".to_string(), "too long".to_string());
    let msg2 = ValidationMessage::new(
      Severity::Warning,
      "description".to_string(),
      "vague".to_string(),
    );
    let msg3 = ValidationMessage::new(
      Severity::Info,
      "spec_name".to_string(),
      "not following convention".to_string(),
    );

    let report1 = ValidationReport::new(vec![msg1.clone()]);
    let report2 = ValidationReport::new(vec![msg2.clone(), msg3.clone()]);

    let aggregated = ValidationReport::aggregate(vec![report1, report2]);

    assert!(!aggregated.is_valid());
    assert_eq!(aggregated.error_count(), 1);
    assert_eq!(aggregated.warning_count(), 1);
    assert_eq!(aggregated.info_count(), 1);
    assert_eq!(aggregated.messages().len(), 3);
  }

  #[test]
  fn test_validation_report_empty_is_valid() {
    let report = ValidationReport::valid();
    assert!(report.is_valid());
    assert!(report.is_empty());
    assert_eq!(report.error_count(), 0);
    assert_eq!(report.warning_count(), 0);
    assert_eq!(report.info_count(), 0);
  }

  #[test]
  fn test_validation_report_with_warnings_only() {
    let msg = ValidationMessage::new(
      Severity::Warning,
      "field".to_string(),
      "warning".to_string(),
    );
    let report = ValidationReport::new(vec![msg]);

    // Warnings don't make it invalid
    assert!(report.is_valid());
    assert_eq!(report.warning_count(), 1);
  }

  #[test]
  fn test_validation_report_to_json() {
    let msg1 = ValidationMessage::new(Severity::Error, "title".to_string(), "too long".to_string());
    let report = ValidationReport::new(vec![msg1]);

    let json = report.to_json();
    assert!(json.contains(r#""valid":false"#));
    assert!(json.contains(r#""severity":"error""#));
    assert!(json.contains(r#""field_path":"title""#));
    assert!(json.contains(r#""message":"too long""#));
  }

  // Test 4: Should Chain Validators With AndThen
  #[test]
  fn test_validator_and_both_pass() {
    let validate_non_empty = Validator::single(|s: &str| {
      if s.is_empty() {
        Err(ValidationError::EmptyInput)
      } else {
        Ok(s.to_string())
      }
    });

    let validate_max_length = Validator::single(|s: &str| {
      if s.len() > 10 {
        Err(ValidationError::InputTooLong { max_length: 10 })
      } else {
        Ok(s.to_string())
      }
    });

    let combined = validate_non_empty.and(validate_max_length);
    let result = combined.validate(&"test".to_string());

    assert!(result.is_ok());
  }

  #[test]
  fn test_validator_and_first_fails() {
    let validate_non_empty = Validator::single(|s: &str| {
      if s.is_empty() {
        Err(ValidationError::EmptyInput)
      } else {
        Ok(s.to_string())
      }
    });

    let validate_max_length = Validator::single(|s: &str| {
      if s.len() > 10 {
        Err(ValidationError::InputTooLong { max_length: 10 })
      } else {
        Ok(s.to_string())
      }
    });

    let combined = validate_non_empty.and(validate_max_length);
    let result = combined.validate(&"".to_string());

    assert!(result.is_err());
    assert_eq!(result, Err(ValidationError::EmptyInput));
  }

  #[test]
  fn test_validator_and_second_fails() {
    let validate_non_empty = Validator::single(|s: &str| {
      if s.is_empty() {
        Err(ValidationError::EmptyInput)
      } else {
        Ok(s.to_string())
      }
    });

    let validate_max_length = Validator::single(|s: &str| {
      if s.len() > 10 {
        Err(ValidationError::InputTooLong { max_length: 10 })
      } else {
        Ok(s.to_string())
      }
    });

    let combined = validate_non_empty.and(validate_max_length);
    let result = combined.validate(&"this is a very long string".to_string());

    assert!(result.is_err());
  }

  // Test 5: Should Combine Validators With Or Logic
  #[test]
  fn test_validator_or_first_passes() {
    let validate_email = Validator::single(|s: &str| {
      if s.contains('@') && s.contains('.') {
        Ok(s.to_string())
      } else {
        Err(ValidationError::InvalidFormat {
          reason: "Invalid email".to_string(),
        })
      }
    });

    let validate_phone = Validator::single(|s: &str| {
      if s
        .chars()
        .all(|c| c.is_ascii_digit() || c == '-' || c == '+')
      {
        Ok(s.to_string())
      } else {
        Err(ValidationError::InvalidFormat {
          reason: "Invalid phone".to_string(),
        })
      }
    });

    let combined = validate_email.or(validate_phone);
    let result = combined.validate(&"test@example.com".to_string());

    assert!(result.is_ok());
  }

  #[test]
  fn test_validator_or_second_passes() {
    let validate_email = Validator::single(|s: &str| {
      if s.contains('@') && s.contains('.') {
        Ok(s.to_string())
      } else {
        Err(ValidationError::InvalidFormat {
          reason: "Invalid email".to_string(),
        })
      }
    });

    let validate_phone = Validator::single(|s: &str| {
      if s
        .chars()
        .all(|c| c.is_ascii_digit() || c == '-' || c == '+')
      {
        Ok(s.to_string())
      } else {
        Err(ValidationError::InvalidFormat {
          reason: "Invalid phone".to_string(),
        })
      }
    });

    let combined = validate_email.or(validate_phone);
    let result = combined.validate(&"+1-555-0123".to_string());

    assert!(result.is_ok());
  }

  #[test]
  fn test_validator_or_both_fail() {
    let validate_email = Validator::single(|s: &str| {
      if s.contains('@') && s.contains('.') {
        Ok(s.to_string())
      } else {
        Err(ValidationError::InvalidFormat {
          reason: "Invalid email".to_string(),
        })
      }
    });

    let validate_phone = Validator::single(|s: &str| {
      if s
        .chars()
        .all(|c| c.is_ascii_digit() || c == '-' || c == '+')
      {
        Ok(s.to_string())
      } else {
        Err(ValidationError::InvalidFormat {
          reason: "Invalid phone".to_string(),
        })
      }
    });

    let combined = validate_email.or(validate_phone);
    let result = combined.validate(&"invalid input".to_string());

    assert!(result.is_err());
    match result {
      Err(ValidationError::InvalidFormat { reason }) => {
        assert!(reason.contains("Both validators"));
      }
      _ => panic!("Expected InvalidFormat error"),
    }
  }

  // Test 6: Should Provide Context In Validation Errors
  #[test]
  fn test_validation_message_context() {
    let msg = ValidationMessage::new(Severity::Error, "title".to_string(), "too long".to_string());

    assert_eq!(msg.field_path(), "title");
    assert_eq!(msg.message(), "too long");
    assert_eq!(msg.severity(), Severity::Error);
    assert!(msg.is_error());
    assert!(!msg.is_warning());
    assert!(!msg.is_info());
  }

  #[test]
  fn test_validation_message_display() {
    let msg = ValidationMessage::new(Severity::Error, "title".to_string(), "too long".to_string());

    let display = format!("{}", msg);
    assert!(display.contains("error"));
    assert!(display.contains("title"));
    assert!(display.contains("too long"));
  }

  // Test 7: Should Track Quality Metrics
  #[test]
  fn test_quality_metrics_new() {
    let result = QualityMetrics::new(0.85, 5, 1000);
    assert!(result.is_ok());

    let metrics = result.unwrap();
    assert_eq!(metrics.test_coverage().value(), 0.85);
    assert_eq!(metrics.complexity(), 5);
    assert_eq!(metrics.lines_of_code(), 1000);
  }

  #[test]
  fn test_quality_metrics_invalid_coverage() {
    let result = QualityMetrics::new(1.5, 5, 1000);
    assert!(result.is_err());
  }

  #[test]
  fn test_quality_metrics_quality_score() {
    let metrics = QualityMetrics::new(0.9, 5, 1000).unwrap();
    let score = metrics.quality_score();

    // Should pass given good coverage and low complexity
    assert!(score.is_passing());
  }

  #[test]
  fn test_quality_metrics_with_custom() {
    let metrics = QualityMetrics::new(0.8, 10, 1000)
      .unwrap()
      .with_custom_metric("documentation".to_string(), 0.9)
      .unwrap();

    assert!(metrics.custom_metrics().contains_key("documentation"));
    assert_eq!(metrics.custom_metrics().get("documentation"), Some(&0.9));
  }

  #[test]
  fn test_quality_metrics_invalid_custom() {
    let result = QualityMetrics::new(0.8, 10, 1000)
      .unwrap()
      .with_custom_metric("test".to_string(), 1.5);

    assert!(result.is_err());
  }

  // Test 8: Should Support Custom Validators
  #[test]
  fn test_custom_validator_passes() {
    let validator = CustomValidator::new(
      |s: &str| {
        if s.contains("security") {
          Ok(s.to_string())
        } else {
          Err(ValidationError::InvalidFormat {
            reason: "Must contain 'security'".to_string(),
          })
        }
      },
      "Security keyword required".to_string(),
    );

    let result = validator.validate(&"this discusses security topics");
    assert!(result.is_ok());
  }

  #[test]
  fn test_custom_validator_fails() {
    let validator = CustomValidator::new(
      |s: &str| {
        if s.contains("security") {
          Ok(s.to_string())
        } else {
          Err(ValidationError::InvalidFormat {
            reason: "Must contain 'security'".to_string(),
          })
        }
      },
      "Security keyword required".to_string(),
    );

    let result = validator.validate(&"this discusses other topics");
    assert!(result.is_err());
  }

  #[test]
  fn test_custom_validator_reusable() {
    let validator = CustomValidator::new(
      |s: &str| {
        if s.contains("security") {
          Ok(s.to_string())
        } else {
          Err(ValidationError::InvalidFormat {
            reason: "Must contain 'security'".to_string(),
          })
        }
      },
      "Security keyword required".to_string(),
    );

    assert!(validator.validate(&"security first").is_ok());
    assert!(validator.validate(&"security second").is_ok());
    assert!(validator.validate(&"no keyword").is_err());
  }

  // Test 9: Should Validate With Severity Levels
  #[test]
  fn test_severity_error_blocks() {
    let msg = ValidationMessage::new(Severity::Error, "field".to_string(), "error".to_string());
    let report = ValidationReport::new(vec![msg]);

    assert!(!report.is_valid());
  }

  #[test]
  fn test_severity_warning_allows() {
    let msg = ValidationMessage::new(
      Severity::Warning,
      "field".to_string(),
      "warning".to_string(),
    );
    let report = ValidationReport::new(vec![msg]);

    assert!(report.is_valid());
  }

  #[test]
  fn test_severity_info_allows() {
    let msg = ValidationMessage::new(Severity::Info, "field".to_string(), "info".to_string());
    let report = ValidationReport::new(vec![msg]);

    assert!(report.is_valid());
  }

  #[test]
  fn test_severity_ordering() {
    assert!(Severity::Error > Severity::Warning);
    assert!(Severity::Warning > Severity::Info);
  }

  // Edge cases
  #[test]
  fn test_quality_score_excellent() {
    let score = QualityScore::new(0.95).unwrap();
    assert!(score.is_excellent());
    assert!(score.is_passing());
  }

  #[test]
  fn test_quality_score_poor() {
    let score = QualityScore::new(0.3).unwrap();
    assert!(score.is_poor());
    assert!(score.is_failing());
  }

  #[test]
  fn test_quality_score_exactly_threshold() {
    let score = QualityScore::new(0.7).unwrap();
    assert!(score.is_passing());
    assert!(!score.is_failing());
  }

  #[test]
  fn test_validation_report_display_valid() {
    let report = ValidationReport::valid();
    let display = format!("{}", report);
    assert!(display.contains("Validation passed"));
  }

  #[test]
  fn test_validation_report_display_invalid() {
    let msg = ValidationMessage::new(Severity::Error, "field".to_string(), "error".to_string());
    let report = ValidationReport::new(vec![msg]);
    let display = format!("{}", report);
    assert!(display.contains("Validation failed"));
  }
}
