#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use thiserror::Error;

/// Spec template errors following Scott Wlaschin's DDD principles:
/// - Specific variants with structured data
/// - No opaque String variants that lose type information
/// - Helper constructors for common cases
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SpecTemplateError {
  /// Placeholder could not be resolved in template
  #[error("placeholder not found: {placeholder} in template '{template_name}'")]
  PlaceholderNotFound {
    /// The placeholder that could not be found
    placeholder: String,
    /// The template where the placeholder was expected
    template_name: String,
  },

  /// Multiple placeholders could not be resolved
  #[error("unresolved placeholders in template '{template_name}': {count} remaining")]
  UnresolvedPlaceholders {
    /// The template with unresolved placeholders
    template_name: String,
    /// Number of unresolved placeholders
    count: usize,
    /// The list of unresolved placeholder names
    placeholders: Vec<String>,
  },

  /// Template content is empty
  #[error("template '{name}' is empty")]
  EmptyTemplate {
    /// Name of the empty template
    name: String,
  },

  /// Session has no answers to fill template
  #[error("session '{session_id}' has no answers to fill template")]
  NoAnswers {
    /// The session ID
    session_id: String,
  },

  /// Required field is missing from template data
  #[error("missing required field '{field}' in {context}")]
  MissingField {
    /// The missing field name
    field: String,
    /// Where the field was expected
    context: String,
  },

  /// JSON serialization or deserialization failed
  #[error("JSON error during {operation}: {reason}")]
  JsonError {
    /// The operation that failed
    operation: JsonOperation,
    /// The specific error
    reason: JsonErrorDetail,
  },

  /// Template rendering failed
  #[error("template rendering failed at {stage}: {reason}")]
  RenderingError {
    /// The stage where rendering failed
    stage: RenderingStage,
    /// Why rendering failed
    reason: RenderingFailureReason,
  },
}

/// Types of JSON operations that can fail
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonOperation {
  /// Serializing data to JSON
  Serialization,
  /// Deserializing JSON to data
  Deserialization,
  /// Parsing JSON string
  Parsing,
}

impl std::fmt::Display for JsonOperation {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Serialization => write!(f, "serialization"),
      Self::Deserialization => write!(f, "deserialization"),
      Self::Parsing => write!(f, "parsing"),
    }
  }
}

/// Details about JSON errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonErrorDetail {
  /// Syntax error in JSON
  SyntaxError { message: String },
  /// Type mismatch
  TypeMismatch { expected: String, actual: String },
  /// Missing field
  MissingField { field: String },
  /// Invalid UTF-8
  InvalidUtf8,
}

impl std::fmt::Display for JsonErrorDetail {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::SyntaxError { message } => write!(f, "syntax error: {message}"),
      Self::TypeMismatch { expected, actual } => {
        write!(f, "type mismatch: expected {expected}, got {actual}")
      }
      Self::MissingField { field } => write!(f, "missing field: {field}"),
      Self::InvalidUtf8 => write!(f, "invalid UTF-8"),
    }
  }
}

/// Stages in template rendering
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderingStage {
  /// Loading the template
  Load,
  /// Parsing the template
  Parse,
  /// Resolving placeholders
  Resolve,
  /// Final assembly
  Assembly,
}

impl std::fmt::Display for RenderingStage {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Load => write!(f, "load"),
      Self::Parse => write!(f, "parse"),
      Self::Resolve => write!(f, "resolve"),
      Self::Assembly => write!(f, "assembly"),
    }
  }
}

/// Reasons why rendering failed
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderingFailureReason {
  /// Circular reference in template
  CircularReference { reference: String },
  /// Invalid placeholder syntax
  InvalidSyntax { placeholder: String, error: String },
  /// Value could not be converted to string
  ConversionFailed { field: String, value_type: String },
  /// Template source not found
  SourceNotFound { source: String },
  /// Custom rendering error
  Custom { message: String },
}

impl std::fmt::Display for RenderingFailureReason {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::CircularReference { reference } => write!(f, "circular reference: {reference}"),
      Self::InvalidSyntax { placeholder, error } => {
        write!(f, "invalid syntax in '{placeholder}': {error}")
      }
      Self::ConversionFailed { field, value_type } => {
        write!(f, "cannot convert {field} ({value_type}) to string")
      }
      Self::SourceNotFound { source } => write!(f, "source not found: {source}"),
      Self::Custom { message } => write!(f, "{message}"),
    }
  }
}

// ============================================================================
// Helper constructors for common error cases
// ============================================================================

impl SpecTemplateError {
  /// Create a placeholder not found error
  #[must_use]
  pub fn placeholder_missing(
    placeholder: impl Into<String>,
    template_name: impl Into<String>,
  ) -> Self {
    Self::PlaceholderNotFound {
      placeholder: placeholder.into(),
      template_name: template_name.into(),
    }
  }

  /// Create an unresolved placeholders error
  #[must_use]
  pub fn unresolved_placeholders(
    template_name: impl Into<String>,
    placeholders: Vec<String>,
  ) -> Self {
    let count = placeholders.len();
    Self::UnresolvedPlaceholders {
      template_name: template_name.into(),
      count,
      placeholders,
    }
  }

  /// Create an empty template error
  #[must_use]
  pub fn empty_template(name: impl Into<String>) -> Self {
    Self::EmptyTemplate { name: name.into() }
  }

  /// Create a no answers error
  #[must_use]
  pub fn no_answers(session_id: impl Into<String>) -> Self {
    Self::NoAnswers {
      session_id: session_id.into(),
    }
  }

  /// Create a missing field error
  #[must_use]
  pub fn missing_field(field: impl Into<String>, context: impl Into<String>) -> Self {
    Self::MissingField {
      field: field.into(),
      context: context.into(),
    }
  }

  /// Create a JSON serialization error
  #[must_use]
  pub fn json_serialization(message: impl Into<String>) -> Self {
    Self::JsonError {
      operation: JsonOperation::Serialization,
      reason: JsonErrorDetail::SyntaxError {
        message: message.into(),
      },
    }
  }

  /// Create a rendering error with custom message
  #[must_use]
  pub fn rendering_custom(stage: RenderingStage, message: impl Into<String>) -> Self {
    Self::RenderingError {
      stage,
      reason: RenderingFailureReason::Custom {
        message: message.into(),
      },
    }
  }
}
