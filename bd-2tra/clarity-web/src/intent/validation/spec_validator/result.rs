use super::SpecValidationError;

/// Aggregates validation results including errors and warnings.
///
/// Validity is derived from the absence of errors, following Scott Wlaschin's
/// DDD principle of making states explicit rather than storing redundant flags.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidationResult {
  /// All errors found during validation
  pub errors: Vec<SpecValidationError>,
  /// All warnings found during validation
  pub warnings: Vec<ValidationWarning>,
}

impl ValidationResult {
  /// Create a new validation result with no errors or warnings
  #[must_use]
  pub const fn new() -> Self {
    Self {
      errors: Vec::new(),
      warnings: Vec::new(),
    }
  }

  /// Check if validation passed (no errors found)
  ///
  /// This method derives validity from the absence of errors,
  /// following Scott Wlaschin's DDD principle of making states
  /// explicit rather than storing redundant flags.
  ///
  /// # Returns
  ///
  /// `true` if no errors were found during validation
  #[must_use]
  pub const fn is_valid(&self) -> bool {
    self.errors.is_empty()
  }

  /// Add an error to the result, returning a new result
  ///
  /// This follows functional principles by not mutating in place.
  #[must_use]
  pub fn with_error(mut self, error: SpecValidationError) -> Self {
    self.errors.push(error);
    self
  }

  /// Add a warning to the result, returning a new result
  ///
  /// This follows functional principles by not mutating in place.
  #[must_use]
  pub fn with_warning(mut self, warning: ValidationWarning) -> Self {
    self.warnings.push(warning);
    self
  }

  /// Merge another validation result into this one
  ///
  /// This follows functional principles by consuming both results
  /// and producing a new combined result.
  #[must_use]
  pub fn merge(self, other: Self) -> Self {
    Self {
      errors: self.errors.into_iter().chain(other.errors).collect(),
      warnings: self.warnings.into_iter().chain(other.warnings).collect(),
    }
  }

  /// Check if any errors were found
  #[must_use]
  pub const fn has_errors(&self) -> bool {
    !self.errors.is_empty()
  }

  /// Check if any warnings were found
  #[must_use]
  pub const fn has_warnings(&self) -> bool {
    !self.warnings.is_empty()
  }

  /// Add an error to the result (mutable for internal validator use)
  pub(super) fn add_error(&mut self, error: SpecValidationError) {
    self.errors.push(error);
  }

  /// Add a warning to the result (mutable for internal validator use)
  pub fn add_warning(&mut self, warning: ValidationWarning) {
    self.warnings.push(warning);
  }
}

impl Default for ValidationResult {
  fn default() -> Self {
    Self::new()
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationWarning {
  pub message: String,
  pub context: Option<String>,
}

impl ValidationWarning {
  #[must_use]
  pub const fn new(message: String, context: Option<String>) -> Self {
    Self { message, context }
  }
}
