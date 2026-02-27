use super::SpecValidationError;

#[derive(Debug, Clone, PartialEq)]
pub struct ValidationResult {
  pub is_valid: bool,
  pub errors: Vec<SpecValidationError>,
  pub warnings: Vec<ValidationWarning>,
}

impl ValidationResult {
  #[must_use]
  pub fn new() -> Self {
    Self {
      is_valid: true,
      errors: Vec::new(),
      warnings: Vec::new(),
    }
  }

  pub fn add_error(&mut self, error: SpecValidationError) {
    self.is_valid = false;
    self.errors.push(error);
  }

  pub fn add_warning(&mut self, warning: ValidationWarning) {
    self.warnings.push(warning);
  }

  pub fn merge(&mut self, other: ValidationResult) {
    if !other.is_valid {
      self.is_valid = false;
    }
    self.errors.extend(other.errors);
    self.warnings.extend(other.warnings);
  }

  #[must_use]
  pub fn has_errors(&self) -> bool {
    !self.errors.is_empty()
  }

  #[must_use]
  pub fn has_warnings(&self) -> bool {
    !self.warnings.is_empty()
  }
}

impl Default for ValidationResult {
  fn default() -> Self {
    Self::new()
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValidationWarning {
  pub message: String,
  pub context: Option<String>,
}

impl ValidationWarning {
  #[must_use]
  pub fn new(message: String, context: Option<String>) -> Self {
    Self { message, context }
  }
}
