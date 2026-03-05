use super::{ContextualError, IntentError};

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

  output.push_str(&format!("Error: {error_type}\n"));
  output.push_str(&format!("  Message: {}\n", error.message));

  if let Some(location) = error.location_string() {
    output.push_str(&format!("  Location: {location}\n"));
  }
  if let Some(json_path) = &error.json_path {
    output.push_str(&format!("  JSON Path: {json_path}\n"));
  }

  if !error.suggestions.is_empty() {
    output.push_str("  Suggestions:\n");
    for suggestion in &error.suggestions {
      output.push_str(&format!(
        "    - {} (edit distance: {})\n",
        suggestion.text, suggestion.distance
      ));
    }
  }

  if !error.context.is_empty() {
    output.push_str("  Context:\n");
    for (key, value) in &error.context {
      output.push_str(&format!("    {key}: {value}\n"));
    }
  }

  output.trim_end().to_string()
}
