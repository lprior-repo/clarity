use super::{format_error, IntentError, InternalErrorDetails, Suggestion};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ContextualError {
  pub error: IntentError,
  pub message: String,
  pub source_file: Option<String>,
  pub line: Option<usize>,
  pub column: Option<usize>,
  pub json_path: Option<String>,
  pub suggestions: Vec<Suggestion>,
  pub context: Vec<(String, String)>,
}

impl ContextualError {
  /// Creates a contextual error with a validated, non-empty message.
  ///
  /// # Errors
  /// Returns `IntentError::Internal` when `message` is empty.
  pub fn new(error: IntentError, message: impl Into<String>) -> Result<Self, IntentError> {
    let message = message.into();
    if message.is_empty() {
      return Err(IntentError::Internal {
        details: InternalErrorDetails::Generic {
          message: "error message cannot be empty".into(),
        },
      });
    }

    Ok(Self {
      error,
      message,
      source_file: None,
      line: None,
      column: None,
      json_path: None,
      suggestions: Vec::new(),
      context: Vec::new(),
    })
  }

  #[must_use]
  pub fn with_source_file(mut self, path: impl Into<String>) -> Self {
    self.source_file = Some(path.into());
    self
  }

  #[must_use]
  pub const fn with_line(mut self, line: usize) -> Self {
    self.line = Some(line);
    self
  }

  #[must_use]
  pub const fn with_column(mut self, column: usize) -> Self {
    self.column = Some(column);
    self
  }

  #[must_use]
  pub fn with_json_path(mut self, path: impl Into<String>) -> Self {
    self.json_path = Some(path.into());
    self
  }

  #[must_use]
  pub fn with_suggestion(mut self, suggestion: Suggestion) -> Self {
    self.suggestions.push(suggestion);
    self
  }

  #[must_use]
  pub fn with_suggestions(mut self, suggestions: Vec<Suggestion>) -> Self {
    self.suggestions.extend(suggestions);
    self
  }

  #[must_use]
  pub fn with_context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
    self.context.push((key.into(), value.into()));
    self
  }

  #[must_use]
  pub const fn has_location(&self) -> bool {
    self.source_file.is_some() || self.line.is_some() || self.json_path.is_some()
  }

  #[must_use]
  pub fn location_string(&self) -> Option<String> {
    match (&self.source_file, &self.line, &self.column) {
      (Some(file), Some(line), Some(column)) => Some(format!("{file}:{line}:{column}")),
      (Some(file), Some(line), None) => Some(format!("{file}:{line}")),
      (Some(file), None, None) => Some(file.clone()),
      (None, Some(line), Some(column)) => Some(format!("line {line}, column {column}")),
      (None, Some(line), None) => Some(format!("line {line}")),
      _ => None,
    }
  }
}

impl fmt::Display for ContextualError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", format_error(self))
  }
}

impl std::error::Error for ContextualError {
  fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    Some(&self.error)
  }
}
