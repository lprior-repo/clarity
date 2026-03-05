use itertools::Itertools;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ValidationError {
  pub message: String,
  pub field_failures: Vec<FieldFailure>,
  pub total_errors: usize,
}

impl ValidationError {
  #[must_use]
  pub fn new(message: impl Into<String>) -> Self {
    Self {
      message: message.into(),
      field_failures: Vec::new(),
      total_errors: 0,
    }
  }

  #[must_use]
  pub fn with_failures(message: impl Into<String>, failures: Vec<FieldFailure>) -> Self {
    Self {
      message: message.into(),
      total_errors: failures.len(),
      field_failures: failures,
    }
  }

  #[must_use]
  pub fn add_failure(mut self, failure: FieldFailure) -> Self {
    self.field_failures.push(failure);
    self.total_errors += 1;
    self
  }

  #[must_use]
  pub const fn has_failures(&self) -> bool {
    !self.field_failures.is_empty()
  }

  #[must_use]
  pub fn failures_for_field(&self, field: &str) -> Vec<&FieldFailure> {
    self
      .field_failures
      .iter()
      .filter(|failure| failure.field == field)
      .collect()
  }
}

impl fmt::Display for ValidationError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "Validation Error: {}", self.message)?;
    for failure in &self.field_failures {
      writeln!(f, "  - {failure}")?;
    }
    if self.total_errors > self.field_failures.len() {
      writeln!(
        f,
        "  ... and {} more errors",
        self.total_errors - self.field_failures.len()
      )?;
    }
    Ok(())
  }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FieldFailure {
  pub field: String,
  pub code: String,
  pub message: String,
  pub actual_value: Option<String>,
  pub expected: Option<String>,
  pub suggestions: Vec<Suggestion>,
}

impl FieldFailure {
  #[must_use]
  pub fn new(
    field: impl Into<String>,
    code: impl Into<String>,
    message: impl Into<String>,
  ) -> Self {
    Self {
      field: field.into(),
      code: code.into(),
      message: message.into(),
      actual_value: None,
      expected: None,
      suggestions: Vec::new(),
    }
  }

  #[must_use]
  pub fn required(field: impl Into<String>) -> Self {
    Self::new(field, "required", "This field is required")
  }

  #[must_use]
  pub fn invalid_type(
    field: impl Into<String>,
    expected: impl Into<String>,
    actual: impl Into<String>,
  ) -> Self {
    let expected = expected.into();
    let actual = actual.into();

    Self::new(
      field,
      "invalid_type",
      format!("Expected {expected}, got {actual}"),
    )
    .with_expected(expected)
    .with_actual(actual)
  }

  #[must_use]
  pub fn unknown_field(field: impl Into<String>, suggestions: Vec<Suggestion>) -> Self {
    let field = field.into();
    Self::new(
      field.clone(),
      "unknown_field",
      format!("Unknown field '{field}'"),
    )
    .with_suggestions(suggestions)
  }

  #[must_use]
  pub fn with_actual(mut self, value: impl Into<String>) -> Self {
    self.actual_value = Some(value.into());
    self
  }

  #[must_use]
  pub fn with_expected(mut self, value: impl Into<String>) -> Self {
    self.expected = Some(value.into());
    self
  }

  #[must_use]
  pub fn with_suggestions(mut self, suggestions: Vec<Suggestion>) -> Self {
    self.suggestions = suggestions;
    self
  }
}

impl fmt::Display for FieldFailure {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "Field '{}': {}", self.field, self.message)?;
    if !self.suggestions.is_empty() {
      let suggestions = self.suggestions.iter().map(|s| s.text.as_str()).join(", ");
      write!(f, " (did you mean: {suggestions}?)")?;
    }
    Ok(())
  }
}

#[derive(
  Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct Suggestion {
  pub text: String,
  pub distance: usize,
}

impl Suggestion {
  #[must_use]
  pub fn new(text: impl Into<String>, distance: usize) -> Self {
    Self {
      text: text.into(),
      distance,
    }
  }
}

impl fmt::Display for Suggestion {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{} (distance: {})", self.text, self.distance)
  }
}
