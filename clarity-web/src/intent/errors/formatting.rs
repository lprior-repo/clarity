use super::{ContextualError, IntentError};
use std::fmt::Write as _;

#[must_use]
pub fn format_error(error: &ContextualError) -> String {
  let mut output = String::new();

  let error_type = match &error.error {
    IntentError::JsonParse { .. } => "JSON Parse Error",
    IntentError::MissingField { .. } => "Missing Field",
    IntentError::InvalidType { .. } => "Type Error",
    IntentError::InvalidValue { .. } => "Value Error",
    IntentError::UnknownField { .. } => "Unknown Field",
    IntentError::ValidationFailed { .. } => "Validation Error",
    IntentError::Io { .. } => "IO Error",
    IntentError::FileNotFound { .. } => "File Not Found",
    IntentError::InvalidPath { .. } => "Invalid Path",
    IntentError::CircularDependency { .. } => "Circular Dependency",
    IntentError::ConstraintViolation { .. } => "Constraint Violation",
    IntentError::Configuration { .. } => "Configuration Error",
    IntentError::Internal { .. } => "Internal Error",
  };

  let _ = writeln!(output, "Error: {error_type}");
  let _ = writeln!(output, "  Message: {}", error.message);

  if let Some(location) = error.location_string() {
    let _ = writeln!(output, "  Location: {location}");
  }
  if let Some(json_path) = &error.json_path {
    let _ = writeln!(output, "  JSON Path: {json_path}");
  }

  if !error.suggestions.is_empty() {
    output.push_str("  Suggestions:\n");
    for suggestion in &error.suggestions {
      let _ = writeln!(
        output,
        "    - {} (edit distance: {})",
        suggestion.text, suggestion.distance
      );
    }
  }

  if !error.context.is_empty() {
    output.push_str("  Context:\n");
    for (key, value) in &error.context {
      let _ = writeln!(output, "    {key}: {value}");
    }
  }

  output.trim_end().to_string()
}
