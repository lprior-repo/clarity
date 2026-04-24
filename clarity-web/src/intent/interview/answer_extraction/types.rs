//! Answer Extraction Types
//!
//! This module provides the core types for answer extraction:
//! - [`ExtractionError`] - Error types for extraction failures
//! - [`ExtractedValue`] - Successfully extracted value container

use thiserror::Error;

/// Errors that can occur during answer extraction.
///
/// This enum represents all possible failure modes when extracting
/// structured data from free-text responses. Each variant provides
/// context about what went wrong.
///
/// # Example
///
/// ```ignore
/// use clarity_web::intent::interview::answer_extraction::{
///     extract_boolean, ExtractionError,
/// };
///
/// let result = extract_boolean("maybe");
/// match result {
///     Err(ExtractionError::InvalidBoolean(s)) => {
///         println!("Could not parse '{}' as boolean", s);
///     }
///     Err(ExtractionError::EmptyResponse) => {
///         println!("Response was empty");
///     }
///     _ => {}
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ExtractionError {
  /// The response text was empty or contained only whitespace.
  ///
  /// This error is returned when attempting to extract data from
  /// an empty string or a string containing only whitespace characters.
  #[error("empty response text")]
  EmptyResponse,

  /// The number format was invalid or could not be parsed.
  ///
  /// Contains the original input string that failed to parse as a number.
  /// This can occur with [`crate::extractors::extract_integer`] or [`crate::extractors::extract_float`] when
  /// the input does not contain a valid numeric sequence.
  #[error("invalid number format: {0}")]
  InvalidNumber(String),

  /// The boolean format was invalid or unrecognized.
  ///
  /// Contains the original input string that failed to parse as a boolean.
  /// Valid boolean values are: yes/no, true/false, on/off, 1/0, y/n, t/f.
  #[error("invalid boolean format: {0}")]
  InvalidBoolean(String),

  /// The URL format was invalid or no URL was found.
  ///
  /// Contains a description of why the URL extraction failed.
  /// Only HTTP and HTTPS URLs are recognized.
  #[error("invalid URL format: {0}")]
  InvalidUrl(String),

  /// The email format was invalid or no email was found.
  ///
  /// Contains a description of why the email extraction failed.
  /// Emails must contain an @ symbol with text on both sides.
  #[error("invalid email format: {0}")]
  InvalidEmail(String),

  /// A required field was not found in the response.
  ///
  /// Contains the name of the field that could not be located.
  #[error("field not found: {0}")]
  FieldNotFound(String),

  /// A general extraction failure occurred.
  ///
  /// Contains the expected type and the reason for failure,
  /// useful for debugging complex extraction scenarios.
  #[error("extraction failed for type '{expected}': {reason}")]
  ExtractionFailed {
    /// The type that was expected during extraction.
    expected: String,
    /// A human-readable explanation of why extraction failed.
    reason: String,
  },
}

/// Represents a successfully extracted value with its type information.
///
/// This enum provides a type-safe container for values extracted from
/// free-text responses. Each variant corresponds to a supported data type.
///
/// # Variants
///
/// - `Text`: A plain text string (trimmed of whitespace)
/// - `Integer`: A 64-bit signed integer
/// - `Float`: A 64-bit floating-point number
/// - `Boolean`: A boolean value parsed from various formats
/// - `Url`: A URL string (HTTP or HTTPS only)
/// - `Email`: An email address string
/// - `List`: A vector of string items
///
/// # Example
///
/// ```ignore
/// use clarity_web::intent::interview::answer_extraction::{
///     extract_by_type, ExtractedValue,
/// };
///
/// let value = extract_by_type("yes", "boolean")?;
/// match value {
///     ExtractedValue::Boolean(b) => println!("Got boolean: {}", b),
///     ExtractedValue::Integer(n) => println!("Got integer: {}", n),
///     ExtractedValue::Text(s) => println!("Got text: {}", s),
///     _ => println!("Got other type"),
/// }
/// ```
///
/// # Conversions
///
/// Use [`ExtractedValue::to_string_value`] to convert any variant to a string
/// representation. Use [`ExtractedValue::is_empty`] to check if a value
/// represents empty content.
#[derive(Debug, Clone, PartialEq)]
pub enum ExtractedValue {
  /// A plain text string, trimmed of leading/trailing whitespace.
  Text(String),
  /// A 64-bit signed integer extracted from the response.
  Integer(i64),
  /// A 64-bit floating-point number extracted from the response.
  Float(f64),
  /// A boolean value parsed from various formats.
  Boolean(bool),
  /// A URL string (HTTP or HTTPS protocol only).
  Url(String),
  /// An email address string.
  Email(String),
  /// A list of string items from comma/newline/numbered lists.
  List(Vec<String>),
}

impl ExtractedValue {
  /// Converts the extracted value to a string representation.
  ///
  /// Each variant is converted to a human-readable string:
  /// - `Text`, `Url`, `Email`: returned as-is
  /// - `Integer`, `Float`, `Boolean`: converted via `Display`
  /// - `List`: items joined with ", " separator
  ///
  /// # Example
  ///
  /// ```ignore
  /// use clarity_web::intent::interview::answer_extraction::ExtractedValue;
  ///
  /// let value = ExtractedValue::Integer(42);
  /// assert_eq!(value.to_string_value(), "42");
  ///
  /// let value = ExtractedValue::List(vec!["a".into(), "b".into()]);
  /// assert_eq!(value.to_string_value(), "a, b");
  /// ```
  #[must_use]
  pub fn to_string_value(&self) -> String {
    match self {
      Self::Text(s) => s.clone(),
      Self::Integer(n) => n.to_string(),
      Self::Float(f) => f.to_string(),
      Self::Boolean(b) => b.to_string(),
      Self::Url(u) => u.clone(),
      Self::Email(e) => e.clone(),
      Self::List(items) => items.join(", "),
    }
  }

  /// Checks if the extracted value represents empty content.
  ///
  /// Returns `true` for:
  /// - Empty or whitespace-only text, URLs, or emails
  /// - Empty lists
  ///
  /// Returns `false` for:
  /// - Any integer, float, or boolean value (including 0 and false)
  /// - Non-empty text, URLs, emails, or lists
  ///
  /// # Example
  ///
  /// ```ignore
  /// use clarity_web::intent::interview::answer_extraction::ExtractedValue;
  ///
  /// assert!(ExtractedValue::Text("".into()).is_empty());
  /// assert!(ExtractedValue::Text("   ".into()).is_empty());
  /// assert!(ExtractedValue::List(vec![]).is_empty());
  ///
  /// assert!(!ExtractedValue::Integer(0).is_empty());
  /// assert!(!ExtractedValue::Boolean(false).is_empty());
  /// assert!(!ExtractedValue::Text("hello".into()).is_empty());
  /// ```
  #[must_use]
  pub fn is_empty(&self) -> bool {
    match self {
      Self::Text(s) => s.trim().is_empty(),
      Self::Integer(_) | Self::Float(_) | Self::Boolean(_) => false,
      Self::Url(u) => u.trim().is_empty(),
      Self::Email(e) => e.trim().is_empty(),
      Self::List(items) => items.is_empty(),
    }
  }
}
