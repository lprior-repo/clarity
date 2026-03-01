//! Answer Extraction
//!
//! This module provides functionality to extract structured data from free-text
//! interview answers. It parses various data types from natural language responses,
//! enabling the conversion of unstructured user input into typed, validated values.
//!
//! # Overview
//!
//! The extraction system supports multiple data types:
//! - **Text**: Raw string extraction with whitespace trimming
//! - **Name**: Single-line text (first line only)
//! - **Integer**: Whole numbers extracted from surrounding text
//! - **Float**: Decimal numbers extracted from surrounding text
//! - **Boolean**: True/false values from various formats (yes/no, true/false, 1/0)
//! - **URL**: HTTP/HTTPS URLs extracted from text
//! - **Email**: Email addresses extracted from text
//! - **List**: Items from comma-separated, newline-separated, or numbered lists
//!
//! # Architecture
//!
//! The module follows a functional, pure-core design:
//! - All extraction functions are pure and deterministic
//! - Errors are represented via [`ExtractionError`] using `thiserror`
//! - Results use [`ExtractedValue`] for type-safe return values
//!
//! # Example Usage
//!
//! ```ignore
//! use clarity_web::intent::interview::answer_extraction::{
//!     extract_by_type, extract_text, ExtractedValue,
//! };
//!
//! // Extract typed values from natural language
//! let response = "The project uses 42 services";
//! let value = extract_by_type(response, "integer");
//! assert_eq!(value, Ok(ExtractedValue::Integer(42)));
//!
//! // Extract URLs from text
//! let response = "Visit https://example.com for more info";
//! let value = extract_by_type(response, "url");
//! assert_eq!(value, Ok(ExtractedValue::Url("https://example.com".to_string())));
//! ```
//!
//! # Interview-Specific Extraction
//!
//! For interview responses, use [`extract_from_answer`] which provides specialized
//! extraction patterns for common fields like `auth_method`, `entities`, and `audience`.
//!
//! # Error Handling
//!
//! All fallible operations return `Result<T, ExtractionError>`. The error type
//! provides detailed information about what went wrong during extraction.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use std::collections::HashMap;

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
  /// This can occur with [`extract_integer`] or [`extract_float`] when
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
/// - `Boolean`: A boolean value
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

/// Extracts a text string from a response, trimming whitespace.
///
/// This is the simplest extraction function, returning the input text
/// with leading and trailing whitespace removed. It serves as the fallback
/// for unknown types in [`extract_by_type`].
///
/// # Example
///
/// ```ignore
/// use clarity_web::intent::interview::answer_extraction::extract_text;
///
/// let result = extract_text("  Hello, world!  ");
/// assert_eq!(result, Ok("Hello, world!".to_string()));
///
/// let result = extract_text("");
/// assert!(matches!(result, Err(ExtractionError::EmptyResponse)));
/// ```
///
/// # Errors
///
/// Returns [`ExtractionError::EmptyResponse`] if the response is empty
/// or contains only whitespace characters.
pub fn extract_text(response: &str) -> Result<String, ExtractionError> {
  let trimmed = response.trim();
  if trimmed.is_empty() {
    return Err(ExtractionError::EmptyResponse);
  }
  Ok(trimmed.to_string())
}

/// Extracts a name (trimmed, single-line text) from a response.
///
/// This function is designed for extracting names or titles where only
/// the first line of input is relevant. It trims whitespace and returns
/// only the first non-empty line.
///
/// # Example
///
/// ```ignore
/// use clarity_web::intent::interview::answer_extraction::extract_name;
///
/// let result = extract_name("My API\nSecond line is ignored");
/// assert_eq!(result, Ok("My API".to_string()));
///
/// let result = extract_name("  Trimmed Name  ");
/// assert_eq!(result, Ok("Trimmed Name".to_string()));
/// ```
///
/// # Errors
///
/// Returns [`ExtractionError::EmptyResponse`] if the name is empty after
/// processing (including when the first line is empty).
pub fn extract_name(response: &str) -> Result<String, ExtractionError> {
  let trimmed = response.trim();
  if trimmed.is_empty() {
    return Err(ExtractionError::EmptyResponse);
  }

  // Take first line only
  let first_line = trimmed
    .lines()
    .next()
    .map(str::trim)
    .filter(|s| !s.is_empty())
    .ok_or(ExtractionError::EmptyResponse)?;

  Ok(first_line.to_string())
}

/// Extracts an integer from a response, handling surrounding text.
///
/// This function attempts to find and parse the first number-like sequence
/// in the input text. It handles:
/// - Pure numbers: "42" -> 42
/// - Negative numbers: "-42" -> -42
/// - Numbers in text: "The answer is 42 items" -> 42
/// - Numbers with sign prefix: "+42" -> 42
///
/// # Example
///
/// ```ignore
/// use clarity_web::intent::interview::answer_extraction::extract_integer;
///
/// assert_eq!(extract_integer("42"), Ok(42));
/// assert_eq!(extract_integer("-42"), Ok(-42));
/// assert_eq!(extract_integer("The answer is 42 items"), Ok(42));
/// assert_eq!(extract_integer("10 items and 20 more"), Ok(10)); // First number wins
/// ```
///
/// # Limitations
///
/// - Does not support thousand separators (commas)
/// - Stops at the first non-numeric character after the number starts
/// - Scientific notation is not supported
///
/// # Errors
///
/// Returns [`ExtractionError::InvalidNumber`] if no valid number can be
/// extracted from the response.
pub fn extract_integer(response: &str) -> Result<i64, ExtractionError> {
  let trimmed = response.trim();

  // Try to extract the first number-like sequence from the text.
  // This handles cases like "I have 42 items" -> extract "42"
  let number_str = extract_number_sequence(trimmed);
  number_str
    .parse::<i64>()
    .map_err(|_| ExtractionError::InvalidNumber(trimmed.to_string()))
}

/// Extracts a floating-point number from a response, handling surrounding text.
///
/// This function attempts to find and parse the first decimal number sequence
/// in the input text. It handles:
/// - Pure floats: "3.14" -> 3.14
/// - Negative floats: "-2.5" -> -2.5
/// - Floats in text: "The value is 3.14159 approximately" -> 3.14159
/// - Integers (converted to float): "42" -> 42.0
///
/// # Example
///
/// ```ignore
/// use clarity_web::intent::interview::answer_extraction::extract_float;
///
/// assert!((extract_float("3.14").unwrap() - 3.14).abs() < f64::EPSILON);
/// assert!((extract_float("-2.5").unwrap() - (-2.5)).abs() < f64::EPSILON);
/// assert!((extract_float("42").unwrap() - 42.0).abs() < f64::EPSILON);
/// ```
///
/// # Limitations
///
/// - Does not support thousand separators (commas)
/// - Scientific notation (e.g., "1.5e10") is not fully supported
/// - Stops at the first non-numeric character after the number starts
///
/// # Errors
///
/// Returns [`ExtractionError::InvalidNumber`] if no valid number can be
/// extracted from the response.
pub fn extract_float(response: &str) -> Result<f64, ExtractionError> {
  let trimmed = response.trim();

  // Try to extract the first number-like sequence
  let number_str = extract_number_sequence(trimmed);
  number_str
    .parse::<f64>()
    .map_err(|_| ExtractionError::InvalidNumber(trimmed.to_string()))
}

/// Extracts a boolean from a response, recognizing multiple formats.
///
/// This function parses boolean values from various common representations.
/// All comparisons are case-insensitive and whitespace-trimmed.
///
/// # Recognized True Values
///
/// - "yes", "y"
/// - "true", "t"
/// - "on"
/// - "1"
///
/// # Recognized False Values
///
/// - "no", "n"
/// - "false", "f"
/// - "off"
/// - "0"
///
/// # Example
///
/// ```ignore
/// use clarity_web::intent::interview::answer_extraction::extract_boolean;
///
/// assert_eq!(extract_boolean("yes"), Ok(true));
/// assert_eq!(extract_boolean("YES"), Ok(true));
/// assert_eq!(extract_boolean("no"), Ok(false));
/// assert_eq!(extract_boolean("  true  "), Ok(true));
/// assert_eq!(extract_boolean("1"), Ok(true));
/// assert_eq!(extract_boolean("0"), Ok(false));
/// ```
///
/// # Errors
///
/// Returns [`ExtractionError::InvalidBoolean`] if the input does not match
/// any recognized boolean format. Note that phrases like "yes, please" are
/// not recognized - only single-word boolean values are accepted.
pub fn extract_boolean(response: &str) -> Result<bool, ExtractionError> {
  let trimmed = response.trim().to_lowercase();

  match trimmed.as_str() {
    "yes" | "true" | "on" | "1" | "y" | "t" => Ok(true),
    "no" | "false" | "off" | "0" | "n" | "f" => Ok(false),
    _ => Err(ExtractionError::InvalidBoolean(response.trim().to_string())),
  }
}

/// Extracts a URL from a response, finding HTTP/HTTPS URLs in text.
///
/// This function locates and extracts the first HTTP or HTTPS URL from
/// the input text. It handles URLs embedded in sentences and strips
/// trailing punctuation that is likely not part of the URL.
///
/// # Supported Protocols
///
/// Only HTTP and HTTPS URLs are recognized. Other protocols (FTP, mailto, etc.)
/// are not supported.
///
/// # Example
///
/// ```ignore
/// use clarity_web::intent::interview::answer_extraction::extract_url;
///
/// assert_eq!(extract_url("https://example.com"), Ok("https://example.com".to_string()));
/// assert_eq!(extract_url("Visit https://example.com for info"), Ok("https://example.com".to_string()));
/// assert_eq!(extract_url("Go to https://example.com."), Ok("https://example.com".to_string()));
/// assert_eq!(extract_url("Check (https://example.com)"), Ok("https://example.com".to_string()));
/// ```
///
/// # URL Features Supported
///
/// - Paths: `https://example.com/path/to/resource`
/// - Query strings: `https://example.com/search?q=test`
/// - Ports: `https://example.com:8080/path`
/// - Fragments: `https://example.com/page#section`
///
/// # Errors
///
/// Returns [`ExtractionError::InvalidUrl`] if:
/// - The response is empty
/// - No HTTP/HTTPS URL is found in the text
pub fn extract_url(response: &str) -> Result<String, ExtractionError> {
  let trimmed = response.trim();

  if trimmed.is_empty() {
    return Err(ExtractionError::InvalidUrl("empty".to_string()));
  }

  // Look for URL-like patterns
  let url = extract_url_pattern(trimmed)
    .ok_or_else(|| ExtractionError::InvalidUrl(format!("no valid URL found in: {trimmed}")))?;

  Ok(url)
}

/// Extracts an email address from a response.
///
/// This function locates and extracts the first email address from the
/// input text. It handles emails embedded in various contexts and strips
/// surrounding delimiters like angle brackets or parentheses.
///
/// # Email Detection
///
/// Emails are detected by finding the `@` symbol and expanding to include
/// the local part (before @) and domain part (after @). The function
/// handles:
/// - Plain emails: `user@example.com`
/// - Emails in angle brackets: `<user@example.com>`
/// - Emails in parentheses: `(user@example.com)`
/// - Emails at sentence end: `Send to user@example.com.`
///
/// # Example
///
/// ```ignore
/// use clarity_web::intent::interview::answer_extraction::extract_email;
///
/// assert_eq!(extract_email("user@example.com"), Ok("user@example.com".to_string()));
/// assert_eq!(extract_email("Contact support@example.com for help"), Ok("support@example.com".to_string()));
/// assert_eq!(extract_email("<noreply@example.com>"), Ok("noreply@example.com".to_string()));
/// assert_eq!(extract_email("user+tag@example.com"), Ok("user+tag@example.com".to_string()));
/// ```
///
/// # Limitations
///
/// - Performs basic pattern matching, not full RFC 5322 validation
/// - May accept technically invalid emails (e.g., double @ symbols)
///
/// # Errors
///
/// Returns [`ExtractionError::InvalidEmail`] if:
/// - The response is empty
/// - No email pattern (containing @) is found
pub fn extract_email(response: &str) -> Result<String, ExtractionError> {
  let trimmed = response.trim();

  if trimmed.is_empty() {
    return Err(ExtractionError::InvalidEmail("empty".to_string()));
  }

  // Look for email-like patterns
  let email = extract_email_pattern(trimmed)
    .ok_or_else(|| ExtractionError::InvalidEmail(format!("no valid email found in: {trimmed}")))?;

  Ok(email.to_string())
}

/// Extracts a list of items from a response.
///
/// This function parses lists in various formats, automatically detecting
/// the list style. It tries formats in this order:
/// 1. Numbered lists (1. item, 2. item, etc.)
/// 2. Newline-separated lists
/// 3. Comma-separated lists
///
/// # Supported List Formats
///
/// - **Numbered lists**: `1. apple\n2. banana\n3. cherry`
/// - **Parens lists**: `1) apple\n2) banana\n3) cherry`
/// - **Dash lists**: `1- apple\n2- banana\n3- cherry`
/// - **Colon lists**: `1: apple\n2: banana\n3: cherry`
/// - **Newline-separated**: `apple\nbanana\ncherry`
/// - **Comma-separated**: `apple, banana, cherry`
///
/// # Example
///
/// ```ignore
/// use clarity_web::intent::interview::answer_extraction::extract_list;
///
/// assert_eq!(extract_list("apple, banana, cherry"), vec!["apple", "banana", "cherry"]);
/// assert_eq!(extract_list("apple\nbanana\ncherry"), vec!["apple", "banana", "cherry"]);
/// assert_eq!(extract_list("1. apple\n2. banana\n3. cherry"), vec!["apple", "banana", "cherry"]);
/// assert_eq!(extract_list(""), Vec::<String>::new());
/// ```
///
/// # Processing
///
/// - All items are trimmed of whitespace
/// - Empty items are filtered out
/// - The first matching format is used (numbered > newline > comma)
#[must_use]
pub fn extract_list(response: &str) -> Vec<String> {
  let trimmed = response.trim();

  if trimmed.is_empty() {
    return Vec::new();
  }

  // Try numbered list first (1. item, 2. item, etc.)
  let numbered = parse_numbered_list(trimmed);
  if !numbered.is_empty() {
    return numbered;
  }

  // Try newline-separated
  if trimmed.contains('\n') {
    return trimmed
      .lines()
      .map(str::trim)
      .filter(|s| !s.is_empty())
      .map(String::from)
      .collect();
  }

  // Fall back to comma-separated
  trimmed
    .split(',')
    .map(str::trim)
    .filter(|s| !s.is_empty())
    .map(String::from)
    .collect()
}

/// Extracts a value from a response based on the specified type name.
///
/// This is the main dispatch function that routes to the appropriate
/// extraction function based on the type name. It provides a unified
/// interface for extracting typed values.
///
/// # Supported Type Names
///
/// | Type Name(s) | Result Type | Extractor |
/// |--------------|-------------|-----------|
/// | `name` | `Text` | [`extract_name`] |
/// | `integer`, `int`, `number` | `Integer` | [`extract_integer`] |
/// | `float`, `decimal` | `Float` | [`extract_float`] |
/// | `boolean`, `bool` | `Boolean` | [`extract_boolean`] |
/// | `url`, `uri` | `Url` | [`extract_url`] |
/// | `email` | `Email` | [`extract_email`] |
/// | `list`, `array` | `List` | [`extract_list`] |
/// | (any other) | `Text` | [`extract_text`] |
///
/// # Example
///
/// ```ignore
/// use clarity_web::intent::interview::answer_extraction::{extract_by_type, ExtractedValue};
///
/// let result = extract_by_type("42", "integer");
/// assert_eq!(result, Ok(ExtractedValue::Integer(42)));
///
/// let result = extract_by_type("yes", "bool");
/// assert_eq!(result, Ok(ExtractedValue::Boolean(true)));
///
/// let result = extract_by_type("hello", "unknown");
/// assert_eq!(result, Ok(ExtractedValue::Text("hello".to_string())));
/// ```
///
/// # Errors
///
/// Returns the appropriate [`ExtractionError`] based on the underlying
/// extraction function that was called.
pub fn extract_by_type(response: &str, type_name: &str) -> Result<ExtractedValue, ExtractionError> {
  match type_name {
    "name" => extract_name(response).map(ExtractedValue::Text),
    "integer" | "int" | "number" => extract_integer(response).map(ExtractedValue::Integer),
    "float" | "decimal" => extract_float(response).map(ExtractedValue::Float),
    "boolean" | "bool" => extract_boolean(response).map(ExtractedValue::Boolean),
    "url" | "uri" => extract_url(response).map(ExtractedValue::Url),
    "email" => extract_email(response).map(ExtractedValue::Email),
    "list" | "array" => Ok(ExtractedValue::List(extract_list(response))),
    _ => extract_text(response).map(ExtractedValue::Text),
  }
}

/// Extracts multiple fields from a response based on field specifications.
///
/// This function provides batch extraction for multiple fields. For single-field
/// extraction, the entire (trimmed) response is assigned to that field. For
/// multi-field extraction, the response is assigned only to the first field.
///
/// # Arguments
///
/// * `response` - The free-text response to extract from
/// * `fields` - List of field names to extract (uses default "text" type)
///
/// # Returns
///
/// A `HashMap` of field names to extracted string values. Fields that could
/// not be extracted are not included in the result.
///
/// # Example
///
/// ```ignore
/// use clarity_web::intent::interview::answer_extraction::extract_fields;
///
/// // Single field - entire response is assigned
/// let fields = vec!["name".to_string()];
/// let result = extract_fields("My API", &fields);
/// assert_eq!(result.get("name"), Some(&"My API".to_string()));
///
/// // Multiple fields - only first field is populated
/// let fields = vec!["name".to_string(), "description".to_string()];
/// let result = extract_fields("My API", &fields);
/// assert_eq!(result.get("name"), Some(&"My API".to_string()));
/// assert_eq!(result.get("description"), None);
/// ```
///
/// # Note
///
/// For multi-field extraction from a single response, consider using
/// [`extract_from_answer`] which provides more sophisticated field detection.
#[must_use]
pub fn extract_fields(response: &str, fields: &[String]) -> HashMap<String, String> {
  let mut result = HashMap::new();

  if fields.is_empty() {
    return result;
  }

  // For single field, assign entire (trimmed) response
  if fields.len() == 1 {
    if let Ok(text) = extract_text(response) {
      result.insert(fields[0].clone(), text);
    }
    return result;
  }

  // For multiple fields, assign to first field only
  // (Multi-field extraction from single response requires more context)
  if let Ok(text) = extract_text(response) {
    result.insert(fields[0].clone(), text);
  }

  result
}

/// Extracts fields with type specifications from a response.
///
/// This function is similar to [`extract_fields`] but allows specifying
/// the type for each field. The type determines how the value is extracted
/// and validated.
///
/// # Arguments
///
/// * `response` - The free-text response to extract from
/// * `spec` - A `HashMap` mapping field names to type names (see [`extract_by_type`]
///   for supported type names)
///
/// # Returns
///
/// A `HashMap` of field names to extracted string values. Fields that fail
/// extraction are not included in the result.
///
/// # Example
///
/// ```ignore
/// use std::collections::HashMap;
/// use clarity_web::intent::interview::answer_extraction::extract_fields_with_types;
///
/// let mut spec = HashMap::new();
/// spec.insert("count".to_string(), "integer".to_string());
/// spec.insert("enabled".to_string(), "boolean".to_string());
///
/// let result = extract_fields_with_types("42 items", &spec);
/// assert_eq!(result.get("count"), Some(&"42".to_string()));
/// ```
///
/// # Note
///
/// For single-field specs, the entire response is extracted with that field's type.
/// For multi-field specs, only the first field (in iteration order) is populated.
#[must_use]
pub fn extract_fields_with_types<S: std::hash::BuildHasher>(
  response: &str,
  spec: &HashMap<String, String, S>,
) -> HashMap<String, String> {
  let mut result = HashMap::new();

  if spec.is_empty() {
    return result;
  }

  // For single field, extract with its type
  if spec.len() == 1 {
    if let Some((field_name, type_name)) = spec.iter().next() {
      if let Ok(value) = extract_by_type(response, type_name) {
        result.insert(field_name.clone(), value.to_string_value());
      }
    }
    return result;
  }

  // For multiple fields, use first field's type for entire response
  if let Some((field_name, type_name)) = spec.iter().next() {
    if let Ok(value) = extract_by_type(response, type_name) {
      result.insert(field_name.clone(), value.to_string_value());
    }
  }

  result
}

// =============================================================================
// Helper Functions (Internal)
// =============================================================================

/// Extracts the first number-like sequence from a string.
///
/// This function scans through the input character by character to find
/// and extract a numeric sequence. It handles:
/// - Optional sign prefix (+ or -) before any digits
/// - Decimal point after at least one digit
/// - Stops at the first non-numeric character after digits start
///
/// # Algorithm
///
/// 1. Scan characters, looking for optional sign or digits
/// 2. Once a digit is found, mark as "started"
/// 3. Allow one decimal point after start
/// 4. Stop at first non-digit character (after start)
///
/// # Returns
///
/// - The extracted number string if found
/// - The original string if no number sequence found (for error reporting)
fn extract_number_sequence(s: &str) -> String {
  let mut result = String::new();
  // Track state during scanning
  let mut seen_decimal = false; // Have we seen a decimal point?
  let mut seen_sign = false; // Have we seen a sign (+/-)?
  let mut started = false; // Have we started collecting digits?

  for ch in s.chars() {
    match ch {
      // Sign is only valid at the start, before any digits
      '-' | '+' if !started && !seen_sign => {
        result.push(ch);
        seen_sign = true;
      }
      // Decimal point is only valid after at least one digit
      '.' if started && !seen_decimal => {
        result.push(ch);
        seen_decimal = true;
      }
      // Digits are always valid after sign or on their own
      '0'..='9' => {
        result.push(ch);
        started = true;
      }
      // Any other character after we've started means end of number
      _ if started => {
        break;
      }
      // Ignore other characters before number starts (e.g., leading text)
      _ => {}
    }
  }

  // If we didn't extract anything, return original for error message
  if result.is_empty() {
    s.to_string()
  } else {
    result
  }
}

/// Extracts a URL pattern from text by finding HTTP/HTTPS protocol.
///
/// This function searches for `http://` or `https://` (case-insensitive)
/// and extracts the URL up to the first delimiter character.
///
/// # Algorithm
///
/// 1. Find the start of the URL by looking for http:// or https://
/// 2. Scan forward until hitting a delimiter (whitespace, comma, bracket)
/// 3. Strip trailing period if present (likely sentence punctuation)
///
/// # Returns
///
/// - `Some(url)` if a URL pattern is found
/// - `None` if no HTTP/HTTPS URL is found
fn extract_url_pattern(s: &str) -> Option<String> {
  // Case-insensitive search for protocol marker
  let lower = s.to_lowercase();
  let start = lower.find("http://").or_else(|| lower.find("https://"))?;

  // Extract from the original string (preserving case in URL)
  let rest = &s[start..];

  // Find the end of the URL - stop at common delimiters
  let end = rest
    .find(|c: char| c.is_whitespace() || c == ',' || c == ')' || c == ']' || c == '}')
    .unwrap_or(rest.len());

  let url = &rest[..end];

  // Strip trailing period - it's likely sentence punctuation, not part of URL
  // (URLs can technically end with a period, but this is rare in practice)
  let url = url.strip_suffix('.').unwrap_or(url);

  Some(url.to_string())
}

/// Extracts an email pattern from text by finding the @ symbol.
///
/// This function locates an email address by finding the @ symbol and
/// expanding outward to capture the local and domain parts.
///
/// # Algorithm
///
/// 1. Find the @ symbol position
/// 2. Scan backwards from @ to find the start (stop at whitespace or delimiters)
/// 3. Scan forwards from @ to find the end (stop at whitespace or delimiters)
/// 4. Strip trailing period if present (likely sentence punctuation)
///
/// # Returns
///
/// - `Some(email)` if an @ symbol is found and text surrounds it
/// - `None` if no @ symbol is present
fn extract_email_pattern(s: &str) -> Option<&str> {
  // Find the @ symbol - required for any email
  let at_pos = s.find('@')?;

  // Scan backwards from @ to find the start of the local part
  // Stop at whitespace or common delimiters that precede emails
  let start = s[..at_pos]
    .rfind(|c: char| c.is_whitespace() || c == '<' || c == '(' || c == '[')
    .map_or(0, |pos| pos + 1);

  // Scan forwards from @ to find the end of the domain part
  // Stop at whitespace or common delimiters that follow emails
  let rest = &s[at_pos..];
  let end_offset = rest
    .find(|c: char| c.is_whitespace() || c == '>' || c == ')' || c == ']' || c == ',')
    .unwrap_or(rest.len());

  let email = &s[start..at_pos + end_offset];

  // Strip trailing period if present (likely end of sentence, not part of email)
  let email = email.strip_suffix('.').unwrap_or(email);

  Some(email)
}

/// Parses a numbered list from text.
///
/// This function extracts items from numbered list formats like:
/// - `1. item` (period separator)
/// - `1) item` (parenthesis separator)
/// - `1- item` (dash separator)
/// - `1: item` (colon separator)
///
/// # Algorithm
///
/// 1. Process each line independently
/// 2. Check if line starts with a digit
/// 3. Skip all consecutive digits (handles numbers >= 10)
/// 4. Skip the separator character
/// 5. Collect the remaining text as the list item
///
/// # Returns
///
/// A vector of list item strings. Empty if no numbered list pattern is found.
fn parse_numbered_list(s: &str) -> Vec<String> {
  let mut items = Vec::new();

  for line in s.lines() {
    let trimmed = line.trim();

    // Check if line starts with a digit (first digit of list number)
    if let Some(after_number) = trimmed
      .strip_prefix(|c: char| c.is_ascii_digit())
      .map(str::trim_start)
    {
      // Skip remaining digits for multi-digit numbers (10, 11, etc.)
      let after_number = after_number
        .trim_start_matches(|c: char| c.is_ascii_digit())
        .trim_start();

      // Strip the separator character (. or ) or - or :)
      // If no separator, use whatever remains (handles edge cases)
      let content = after_number
        .strip_prefix(['.', ')', '-', ':'])
        .map_or_else(|| after_number.trim(), str::trim);

      if !content.is_empty() {
        items.push(content.to_string());
      }
    }
  }

  items
}

// =============================================================================
// Interview-specific Extraction Functions
// =============================================================================

/// Extracts fields from answer text for interview responses.
///
/// This function is the main entry point for extracting structured data from
/// interview answers. It uses pattern matching to identify known field types
/// and extracts appropriate values.
///
/// # Arguments
///
/// * `_question_id` - The ID of the question (unused but kept for API compatibility)
/// * `response` - The free-text response from the user
/// * `extract_fields` - List of field names to attempt to extract
///
/// # Returns
///
/// A `HashMap` containing the successfully extracted field-value pairs.
/// Fields that couldn't be extracted are simply not included in the result.
///
/// # Supported Field Types
///
/// | Field Name | Extraction Method |
/// |------------|-------------------|
/// | `auth_method` | Recognizes jwt, oauth, session, `api_key`, none |
/// | `entities` | Extracts capitalized words (entity names) |
/// | `audience` | Recognizes mobile, web, api, cli, internal |
/// | (any other) | Returns trimmed text if non-empty |
///
/// # Example
///
/// ```ignore
/// use clarity_web::intent::interview::answer_extraction::extract_from_answer;
///
/// let response = "We use JWT tokens for our mobile app. Main entities are Users and Orders.";
/// let fields = vec!["auth_method".to_string(), "audience".to_string(), "entities".to_string()];
/// let result = extract_from_answer("q1", response, &fields);
///
/// assert_eq!(result.get("auth_method"), Some(&"jwt".to_string()));
/// assert_eq!(result.get("audience"), Some(&"mobile".to_string()));
/// assert!(result.contains_key("entities"));
/// ```
#[must_use]
pub fn extract_from_answer(
  _question_id: &str,
  response: &str,
  extract_fields: &[String],
) -> HashMap<String, String> {
  let mut result = HashMap::new();

  for field in extract_fields {
    if let Some(value) = simple_extract(field, response) {
      result.insert(field.clone(), value);
    }
  }

  result
}

/// Dispatches to the appropriate extraction function based on field name.
///
/// This internal function maps field names to their extraction handlers.
/// For unknown field names, it returns the trimmed text as a fallback.
///
/// # Returns
///
/// - `Some(value)` if extraction was successful
/// - `None` if extraction failed or produced no meaningful result
fn simple_extract(field: &str, text: &str) -> Option<String> {
  match field {
    // Authentication method detection
    "auth_method" => extract_auth_method(text),

    // Entity/data model name extraction
    "entities" => extract_entities(text),

    // Target audience detection
    "audience" => extract_audience(text),

    // Generic fallback: return trimmed text if non-empty
    _ => {
      let trimmed = text.trim();
      if trimmed.is_empty() {
        None
      } else {
        Some(trimmed.to_string())
      }
    }
  }
}

/// Extracts authentication method from text.
///
/// Performs case-insensitive substring matching to identify common
/// authentication methods mentioned in the response.
///
/// # Recognized Values
///
/// | Pattern | Extracted Value |
/// |---------|-----------------|
/// | "jwt" | "jwt" |
/// | "oauth" | "oauth" |
/// | "session" | "session" |
/// | "api key" or "`api_key`" | "`api_key`" |
/// | "none" | "none" |
///
/// # Priority
///
/// Patterns are checked in order; the first match is returned.
fn extract_auth_method(text: &str) -> Option<String> {
  let lower = text.to_lowercase();

  if lower.contains("jwt") {
    Some("jwt".to_string())
  } else if lower.contains("oauth") {
    Some("oauth".to_string())
  } else if lower.contains("session") {
    Some("session".to_string())
  } else if lower.contains("api key") || lower.contains("api_key") {
    Some("api_key".to_string())
  } else if lower.contains("none") {
    Some("none".to_string())
  } else {
    None
  }
}

/// Extracts entity names from text.
///
/// Identifies potential entity names by looking for capitalized words.
/// This heuristic works well for domain entities like "Users", "Orders", etc.
///
/// # Algorithm
///
/// 1. Split text into words
/// 2. For each word, strip trailing punctuation
/// 3. Check if word starts with uppercase and is > 2 characters
/// 4. Filter out common non-entity words (the, and, for, etc.)
/// 5. Join remaining words with ", "
///
/// # Returns
///
/// - `Some(entities)` with comma-separated entity names if any found
/// - `None` if no entity-like words are found
fn extract_entities(text: &str) -> Option<String> {
  let entities: Vec<String> = text
    .split_whitespace()
    .filter_map(|word| {
      // Strip trailing punctuation that might follow entity names
      let clean_word = word.trim_end_matches(',').trim_end_matches('.');

      // Entity candidates must start with uppercase and be meaningful length
      if let Some(first_char) = clean_word.chars().next() {
        if first_char.is_uppercase() && clean_word.len() > 2 {
          // Exclude common words that happen to be capitalized
          let is_common_word = matches!(
            clean_word.to_lowercase().as_str(),
            "the" | "and" | "for" | "use" | "can" | "all" | "our" | "you" | "are"
          );
          if !is_common_word {
            return Some(clean_word.to_string());
          }
        }
      }
      None
    })
    .collect();

  if entities.is_empty() {
    None
  } else {
    Some(entities.join(", "))
  }
}

/// Extracts audience type from text.
///
/// Performs case-insensitive substring matching to identify the target
/// audience for the API or application.
///
/// # Recognized Values
///
/// | Pattern | Extracted Value |
/// |---------|-----------------|
/// | "mobile" | "mobile" |
/// | "web" | "web" |
/// | "api" | "api" |
/// | "cli" | "cli" |
/// | "internal" | "internal" |
///
/// # Priority
///
/// Patterns are checked in order; the first match is returned.
fn extract_audience(text: &str) -> Option<String> {
  let lower = text.to_lowercase();

  if lower.contains("mobile") {
    Some("mobile".to_string())
  } else if lower.contains("web") {
    Some("web".to_string())
  } else if lower.contains("api") {
    Some("api".to_string())
  } else if lower.contains("cli") {
    Some("cli".to_string())
  } else if lower.contains("internal") {
    Some("internal".to_string())
  } else {
    None
  }
}

/// Calculates confidence score for answer extraction quality.
///
/// This function provides a heuristic confidence score (0.0 to 1.0) indicating
/// how reliable the extraction results are likely to be. Higher scores indicate
/// more confident extractions.
///
/// # Scoring Logic
///
/// | Condition | Confidence |
/// |-----------|------------|
/// | Response > 50 chars AND fields extracted | 0.85 |
/// | Otherwise | 0.60 |
///
/// # Arguments
///
/// * `_question_id` - The ID of the question (reserved for future use)
/// * `response` - The original response text
/// * `extracted` - The `HashMap` of extracted field-value pairs
///
/// # Example
///
/// ```ignore
/// use std::collections::HashMap;
/// use clarity_web::intent::interview::answer_extraction::calculate_confidence;
///
/// let mut extracted = HashMap::new();
/// extracted.insert("auth_method".to_string(), "jwt".to_string());
/// let response = "This is a longer response with more than fifty characters for testing.";
///
/// let confidence = calculate_confidence("q1", response, &extracted);
/// assert!((confidence - 0.85).abs() < 0.001);
/// ```
///
/// # Future Improvements
///
/// The confidence calculation could be enhanced to consider:
/// - Number of fields successfully extracted vs. requested
/// - Quality of individual extractions (e.g., valid URL, valid email)
/// - Response coherence and structure
#[must_use]
pub fn calculate_confidence<S: std::hash::BuildHasher>(
  _question_id: &str,
  response: &str,
  extracted: &HashMap<String, String, S>,
) -> f64 {
  let response_length = response.trim().len();
  let _field_count = extracted.len();

  // Heuristic: longer responses with successful extractions indicate
  // more thoughtful answers and thus higher confidence
  if response_length > 50 && !extracted.is_empty() {
    0.85
  } else {
    0.6
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  // =============================================================================
  // extract_text tests
  // =============================================================================

  #[test]
  fn test_extract_text_basic() {
    let result = extract_text("Hello, world!");
    assert!(result.is_ok());
    assert_eq!(result, Ok("Hello, world!".to_string()));
  }

  #[test]
  fn test_extract_text_with_whitespace() {
    let result = extract_text("  trimmed text  ");
    assert!(result.is_ok());
    assert_eq!(result, Ok("trimmed text".to_string()));
  }

  #[test]
  fn test_extract_text_empty() {
    let result = extract_text("");
    assert!(matches!(result, Err(ExtractionError::EmptyResponse)));
  }

  #[test]
  fn test_extract_text_whitespace_only() {
    let result = extract_text("   \n\t  ");
    assert!(matches!(result, Err(ExtractionError::EmptyResponse)));
  }

  #[test]
  fn test_extract_text_unicode() {
    let result = extract_text("  Hello \u{4e16}\u{754c}  ");
    assert!(result.is_ok());
    assert_eq!(result, Ok("Hello \u{4e16}\u{754c}".to_string()));
  }

  #[test]
  fn test_extract_text_multiline() {
    let result = extract_text("Line 1\nLine 2\nLine 3");
    assert!(result.is_ok());
    assert_eq!(result, Ok("Line 1\nLine 2\nLine 3".to_string()));
  }

  // =============================================================================
  // extract_name tests
  // =============================================================================

  #[test]
  fn test_extract_name_basic() {
    let result = extract_name("My API");
    assert!(result.is_ok());
    assert_eq!(result, Ok("My API".to_string()));
  }

  #[test]
  fn test_extract_name_multiline_takes_first() {
    let result = extract_name("First Line\nSecond Line");
    assert!(result.is_ok());
    assert_eq!(result, Ok("First Line".to_string()));
  }

  #[test]
  fn test_extract_name_with_whitespace() {
    let result = extract_name("  Trimmed Name  ");
    assert!(result.is_ok());
    assert_eq!(result, Ok("Trimmed Name".to_string()));
  }

  #[test]
  fn test_extract_name_empty() {
    let result = extract_name("");
    assert!(matches!(result, Err(ExtractionError::EmptyResponse)));
  }

  #[test]
  fn test_extract_name_only_newlines() {
    let result = extract_name("\n\n");
    assert!(matches!(result, Err(ExtractionError::EmptyResponse)));
  }

  #[test]
  fn test_extract_name_first_line_empty() {
    let result = extract_name("\nSecond Line");
    assert!(result.is_ok());
    assert_eq!(result, Ok("Second Line".to_string()));
  }

  // =============================================================================
  // extract_integer tests
  // =============================================================================

  #[test]
  fn test_extract_integer_basic() {
    let result = extract_integer("42");
    assert!(result.is_ok());
    assert_eq!(result, Ok(42));
  }

  #[test]
  fn test_extract_integer_negative() {
    let result = extract_integer("-42");
    assert!(result.is_ok());
    assert_eq!(result, Ok(-42));
  }

  #[test]
  fn test_extract_integer_with_text() {
    let result = extract_integer("The answer is 42 items");
    assert!(result.is_ok());
    assert_eq!(result, Ok(42));
  }

  #[test]
  fn test_extract_integer_with_whitespace() {
    let result = extract_integer("  123  ");
    assert!(result.is_ok());
    assert_eq!(result, Ok(123));
  }

  #[test]
  fn test_extract_integer_invalid() {
    let result = extract_integer("not a number");
    assert!(matches!(result, Err(ExtractionError::InvalidNumber(_))));
  }

  #[test]
  fn test_extract_integer_empty() {
    let result = extract_integer("");
    assert!(matches!(result, Err(ExtractionError::InvalidNumber(_))));
  }

  #[test]
  fn test_extract_integer_large() {
    let result = extract_integer("9223372036854775807");
    assert!(result.is_ok());
    assert_eq!(result, Ok(i64::MAX));
  }

  #[test]
  fn test_extract_integer_takes_first() {
    let result = extract_integer("10 items and 20 more");
    assert!(result.is_ok());
    assert_eq!(result, Ok(10));
  }

  // =============================================================================
  // extract_float tests
  // =============================================================================

  #[test]
  fn test_extract_float_basic() {
    let result = extract_float("3.14");
    assert!(result.is_ok());
    assert!((result.unwrap() - 3.14).abs() < f64::EPSILON);
  }

  #[test]
  fn test_extract_float_negative() {
    let result = extract_float("-2.5");
    assert!(result.is_ok());
    assert!((result.unwrap() - (-2.5)).abs() < f64::EPSILON);
  }

  #[test]
  fn test_extract_float_with_text() {
    let result = extract_float("The value is 3.14159 approximately");
    assert!(result.is_ok());
    assert!((result.unwrap() - 3.14159).abs() < f64::EPSILON);
  }

  #[test]
  fn test_extract_float_integer_input() {
    let result = extract_float("42");
    assert!(result.is_ok());
    assert!((result.unwrap() - 42.0).abs() < f64::EPSILON);
  }

  #[test]
  fn test_extract_float_invalid() {
    let result = extract_float("not a number");
    assert!(matches!(result, Err(ExtractionError::InvalidNumber(_))));
  }

  #[test]
  fn test_extract_float_empty() {
    let result = extract_float("");
    assert!(matches!(result, Err(ExtractionError::InvalidNumber(_))));
  }

  // =============================================================================
  // extract_boolean tests
  // =============================================================================

  #[test]
  fn test_extract_boolean_true_variants() {
    assert_eq!(extract_boolean("yes"), Ok(true));
    assert_eq!(extract_boolean("Yes"), Ok(true));
    assert_eq!(extract_boolean("YES"), Ok(true));
    assert_eq!(extract_boolean("true"), Ok(true));
    assert_eq!(extract_boolean("True"), Ok(true));
    assert_eq!(extract_boolean("TRUE"), Ok(true));
    assert_eq!(extract_boolean("on"), Ok(true));
    assert_eq!(extract_boolean("ON"), Ok(true));
    assert_eq!(extract_boolean("1"), Ok(true));
    assert_eq!(extract_boolean("y"), Ok(true));
    assert_eq!(extract_boolean("Y"), Ok(true));
    assert_eq!(extract_boolean("t"), Ok(true));
    assert_eq!(extract_boolean("T"), Ok(true));
  }

  #[test]
  fn test_extract_boolean_false_variants() {
    assert_eq!(extract_boolean("no"), Ok(false));
    assert_eq!(extract_boolean("No"), Ok(false));
    assert_eq!(extract_boolean("NO"), Ok(false));
    assert_eq!(extract_boolean("false"), Ok(false));
    assert_eq!(extract_boolean("False"), Ok(false));
    assert_eq!(extract_boolean("FALSE"), Ok(false));
    assert_eq!(extract_boolean("off"), Ok(false));
    assert_eq!(extract_boolean("OFF"), Ok(false));
    assert_eq!(extract_boolean("0"), Ok(false));
    assert_eq!(extract_boolean("n"), Ok(false));
    assert_eq!(extract_boolean("N"), Ok(false));
    assert_eq!(extract_boolean("f"), Ok(false));
    assert_eq!(extract_boolean("F"), Ok(false));
  }

  #[test]
  fn test_extract_boolean_with_whitespace() {
    assert_eq!(extract_boolean("  yes  "), Ok(true));
    assert_eq!(extract_boolean("  no  "), Ok(false));
  }

  #[test]
  fn test_extract_boolean_invalid() {
    let result = extract_boolean("maybe");
    assert!(matches!(result, Err(ExtractionError::InvalidBoolean(_))));
  }

  #[test]
  fn test_extract_boolean_empty() {
    let result = extract_boolean("");
    assert!(matches!(result, Err(ExtractionError::InvalidBoolean(_))));
  }

  #[test]
  fn test_extract_boolean_not_a_word() {
    let result = extract_boolean("yes, please");
    assert!(matches!(result, Err(ExtractionError::InvalidBoolean(_))));
  }

  // =============================================================================
  // extract_url tests
  // =============================================================================

  #[test]
  fn test_extract_url_http() {
    let result = extract_url("http://example.com");
    assert!(result.is_ok());
    assert_eq!(result, Ok("http://example.com".to_string()));
  }

  #[test]
  fn test_extract_url_https() {
    let result = extract_url("https://example.com");
    assert!(result.is_ok());
    assert_eq!(result, Ok("https://example.com".to_string()));
  }

  #[test]
  fn test_extract_url_with_path() {
    let result = extract_url("https://example.com/path/to/resource");
    assert!(result.is_ok());
    assert_eq!(
      result,
      Ok("https://example.com/path/to/resource".to_string())
    );
  }

  #[test]
  fn test_extract_url_with_query() {
    let result = extract_url("https://example.com/search?q=test");
    assert!(result.is_ok());
    assert_eq!(result, Ok("https://example.com/search?q=test".to_string()));
  }

  #[test]
  fn test_extract_url_embedded_in_text() {
    let result = extract_url("Visit https://example.com for more info");
    assert!(result.is_ok());
    assert_eq!(result, Ok("https://example.com".to_string()));
  }

  #[test]
  fn test_extract_url_at_end_of_sentence() {
    let result = extract_url("Go to https://example.com.");
    assert!(result.is_ok());
    assert_eq!(result, Ok("https://example.com".to_string()));
  }

  #[test]
  fn test_extract_url_in_parentheses() {
    let result = extract_url("Check this (https://example.com) out");
    assert!(result.is_ok());
    assert_eq!(result, Ok("https://example.com".to_string()));
  }

  #[test]
  fn test_extract_url_invalid_empty() {
    let result = extract_url("");
    assert!(matches!(result, Err(ExtractionError::InvalidUrl(_))));
  }

  #[test]
  fn test_extract_url_invalid_no_protocol() {
    let result = extract_url("example.com");
    assert!(matches!(result, Err(ExtractionError::InvalidUrl(_))));
  }

  #[test]
  fn test_extract_url_invalid_ftp() {
    let result = extract_url("ftp://files.example.com");
    assert!(matches!(result, Err(ExtractionError::InvalidUrl(_))));
  }

  // =============================================================================
  // extract_email tests
  // =============================================================================

  #[test]
  fn test_extract_email_basic() {
    let result = extract_email("user@example.com");
    assert!(result.is_ok());
    assert_eq!(result, Ok("user@example.com".to_string()));
  }

  #[test]
  fn test_extract_email_with_subdomain() {
    let result = extract_email("user@mail.example.com");
    assert!(result.is_ok());
    assert_eq!(result, Ok("user@mail.example.com".to_string()));
  }

  #[test]
  fn test_extract_email_embedded_in_text() {
    let result = extract_email("Contact us at support@example.com for help");
    assert!(result.is_ok());
    assert_eq!(result, Ok("support@example.com".to_string()));
  }

  #[test]
  fn test_extract_email_in_angle_brackets() {
    let result = extract_email("<noreply@example.com>");
    assert!(result.is_ok());
    assert_eq!(result, Ok("noreply@example.com".to_string()));
  }

  #[test]
  fn test_extract_email_in_parentheses() {
    let result = extract_email("Email (admin@example.com) for access");
    assert!(result.is_ok());
    assert_eq!(result, Ok("admin@example.com".to_string()));
  }

  #[test]
  fn test_extract_email_at_end_of_sentence() {
    let result = extract_email("Send to user@example.com.");
    assert!(result.is_ok());
    assert_eq!(result, Ok("user@example.com".to_string()));
  }

  #[test]
  fn test_extract_email_invalid_empty() {
    let result = extract_email("");
    assert!(matches!(result, Err(ExtractionError::InvalidEmail(_))));
  }

  #[test]
  fn test_extract_email_invalid_no_at() {
    let result = extract_email("userexample.com");
    assert!(matches!(result, Err(ExtractionError::InvalidEmail(_))));
  }

  #[test]
  fn test_extract_email_invalid_double_at() {
    let result = extract_email("user@@example.com");
    // This should still extract something, even if technically invalid
    assert!(result.is_ok());
  }

  // =============================================================================
  // extract_list tests
  // =============================================================================

  #[test]
  fn test_extract_list_comma_separated() {
    let result = extract_list("apple, banana, cherry");
    assert_eq!(result, vec!["apple", "banana", "cherry"]);
  }

  #[test]
  fn test_extract_list_newline_separated() {
    let result = extract_list("apple\nbanana\ncherry");
    assert_eq!(result, vec!["apple", "banana", "cherry"]);
  }

  #[test]
  fn test_extract_list_numbered() {
    let result = extract_list("1. apple\n2. banana\n3. cherry");
    assert_eq!(result, vec!["apple", "banana", "cherry"]);
  }

  #[test]
  fn test_extract_list_numbered_parens() {
    let result = extract_list("1) apple\n2) banana\n3) cherry");
    assert_eq!(result, vec!["apple", "banana", "cherry"]);
  }

  #[test]
  fn test_extract_list_numbered_dashes() {
    let result = extract_list("1- apple\n2- banana\n3- cherry");
    assert_eq!(result, vec!["apple", "banana", "cherry"]);
  }

  #[test]
  fn test_extract_list_numbered_colons() {
    let result = extract_list("1: apple\n2: banana\n3: cherry");
    assert_eq!(result, vec!["apple", "banana", "cherry"]);
  }

  #[test]
  fn test_extract_list_empty() {
    let result = extract_list("");
    assert!(result.is_empty());
  }

  #[test]
  fn test_extract_list_whitespace_only() {
    let result = extract_list("   ");
    assert!(result.is_empty());
  }

  #[test]
  fn test_extract_list_with_extra_whitespace() {
    let result = extract_list("  apple  ,  banana  ,  cherry  ");
    assert_eq!(result, vec!["apple", "banana", "cherry"]);
  }

  #[test]
  fn test_extract_list_filters_empty_items() {
    let result = extract_list("apple, , banana, , cherry");
    assert_eq!(result, vec!["apple", "banana", "cherry"]);
  }

  #[test]
  fn test_extract_list_double_digit_numbers() {
    let result = extract_list("1. first\n10. tenth\n11. eleventh");
    assert_eq!(result, vec!["first", "tenth", "eleventh"]);
  }

  // =============================================================================
  // extract_by_type tests
  // =============================================================================

  #[test]
  fn test_extract_by_type_text() {
    let result = extract_by_type("hello world", "text");
    assert!(result.is_ok());
    assert_eq!(result, Ok(ExtractedValue::Text("hello world".to_string())));
  }

  #[test]
  fn test_extract_by_type_string_alias() {
    let result = extract_by_type("hello world", "string");
    assert!(result.is_ok());
    assert_eq!(result, Ok(ExtractedValue::Text("hello world".to_string())));
  }

  #[test]
  fn test_extract_by_type_name() {
    let result = extract_by_type("My API\nExtra line", "name");
    assert!(result.is_ok());
    assert_eq!(result, Ok(ExtractedValue::Text("My API".to_string())));
  }

  #[test]
  fn test_extract_by_type_integer() {
    let result = extract_by_type("42", "integer");
    assert!(result.is_ok());
    assert_eq!(result, Ok(ExtractedValue::Integer(42)));
  }

  #[test]
  fn test_extract_by_type_int_alias() {
    let result = extract_by_type("42", "int");
    assert!(result.is_ok());
    assert_eq!(result, Ok(ExtractedValue::Integer(42)));
  }

  #[test]
  fn test_extract_by_type_number_alias() {
    let result = extract_by_type("42", "number");
    assert!(result.is_ok());
    assert_eq!(result, Ok(ExtractedValue::Integer(42)));
  }

  #[test]
  fn test_extract_by_type_float() {
    let result = extract_by_type("3.14", "float");
    assert!(result.is_ok());
    if let Ok(ExtractedValue::Float(f)) = result {
      assert!((f - 3.14).abs() < f64::EPSILON);
    } else {
      panic!("Expected Float variant");
    }
  }

  #[test]
  fn test_extract_by_type_decimal_alias() {
    let result = extract_by_type("3.14", "decimal");
    assert!(result.is_ok());
    if let Ok(ExtractedValue::Float(f)) = result {
      assert!((f - 3.14).abs() < f64::EPSILON);
    } else {
      panic!("Expected Float variant");
    }
  }

  #[test]
  fn test_extract_by_type_boolean() {
    let result = extract_by_type("yes", "boolean");
    assert!(result.is_ok());
    assert_eq!(result, Ok(ExtractedValue::Boolean(true)));
  }

  #[test]
  fn test_extract_by_type_bool_alias() {
    let result = extract_by_type("no", "bool");
    assert!(result.is_ok());
    assert_eq!(result, Ok(ExtractedValue::Boolean(false)));
  }

  #[test]
  fn test_extract_by_type_url() {
    let result = extract_by_type("https://example.com", "url");
    assert!(result.is_ok());
    assert_eq!(
      result,
      Ok(ExtractedValue::Url("https://example.com".to_string()))
    );
  }

  #[test]
  fn test_extract_by_type_uri_alias() {
    let result = extract_by_type("https://example.com", "uri");
    assert!(result.is_ok());
    assert_eq!(
      result,
      Ok(ExtractedValue::Url("https://example.com".to_string()))
    );
  }

  #[test]
  fn test_extract_by_type_email() {
    let result = extract_by_type("user@example.com", "email");
    assert!(result.is_ok());
    assert_eq!(
      result,
      Ok(ExtractedValue::Email("user@example.com".to_string()))
    );
  }

  #[test]
  fn test_extract_by_type_list() {
    let result = extract_by_type("a, b, c", "list");
    assert!(result.is_ok());
    assert_eq!(
      result,
      Ok(ExtractedValue::List(vec![
        "a".to_string(),
        "b".to_string(),
        "c".to_string()
      ]))
    );
  }

  #[test]
  fn test_extract_by_type_array_alias() {
    let result = extract_by_type("a, b, c", "array");
    assert!(result.is_ok());
    assert_eq!(
      result,
      Ok(ExtractedValue::List(vec![
        "a".to_string(),
        "b".to_string(),
        "c".to_string()
      ]))
    );
  }

  #[test]
  fn test_extract_by_type_unknown_defaults_to_text() {
    let result = extract_by_type("hello world", "unknown_type");
    assert!(result.is_ok());
    assert_eq!(result, Ok(ExtractedValue::Text("hello world".to_string())));
  }

  // =============================================================================
  // ExtractedValue tests
  // =============================================================================

  #[test]
  fn test_extracted_value_to_string_text() {
    let value = ExtractedValue::Text("hello".to_string());
    assert_eq!(value.to_string_value(), "hello");
  }

  #[test]
  fn test_extracted_value_to_string_integer() {
    let value = ExtractedValue::Integer(42);
    assert_eq!(value.to_string_value(), "42");
  }

  #[test]
  fn test_extracted_value_to_string_float() {
    let value = ExtractedValue::Float(3.14);
    assert_eq!(value.to_string_value(), "3.14");
  }

  #[test]
  fn test_extracted_value_to_string_boolean() {
    assert_eq!(ExtractedValue::Boolean(true).to_string_value(), "true");
    assert_eq!(ExtractedValue::Boolean(false).to_string_value(), "false");
  }

  #[test]
  fn test_extracted_value_to_string_url() {
    let value = ExtractedValue::Url("https://example.com".to_string());
    assert_eq!(value.to_string_value(), "https://example.com");
  }

  #[test]
  fn test_extracted_value_to_string_email() {
    let value = ExtractedValue::Email("user@example.com".to_string());
    assert_eq!(value.to_string_value(), "user@example.com");
  }

  #[test]
  fn test_extracted_value_to_string_list() {
    let value = ExtractedValue::List(vec!["a".to_string(), "b".to_string(), "c".to_string()]);
    assert_eq!(value.to_string_value(), "a, b, c");
  }

  #[test]
  fn test_extracted_value_is_empty_text() {
    assert!(ExtractedValue::Text(String::new()).is_empty());
    assert!(ExtractedValue::Text("  ".to_string()).is_empty());
    assert!(!ExtractedValue::Text("hello".to_string()).is_empty());
  }

  #[test]
  fn test_extracted_value_is_empty_integer() {
    assert!(!ExtractedValue::Integer(0).is_empty());
    assert!(!ExtractedValue::Integer(42).is_empty());
  }

  #[test]
  fn test_extracted_value_is_empty_float() {
    assert!(!ExtractedValue::Float(0.0).is_empty());
    assert!(!ExtractedValue::Float(3.14).is_empty());
  }

  #[test]
  fn test_extracted_value_is_empty_boolean() {
    assert!(!ExtractedValue::Boolean(true).is_empty());
    assert!(!ExtractedValue::Boolean(false).is_empty());
  }

  #[test]
  fn test_extracted_value_is_empty_url() {
    assert!(ExtractedValue::Url(String::new()).is_empty());
    assert!(!ExtractedValue::Url("https://example.com".to_string()).is_empty());
  }

  #[test]
  fn test_extracted_value_is_empty_email() {
    assert!(ExtractedValue::Email(String::new()).is_empty());
    assert!(!ExtractedValue::Email("user@example.com".to_string()).is_empty());
  }

  #[test]
  fn test_extracted_value_is_empty_list() {
    assert!(ExtractedValue::List(Vec::new()).is_empty());
    assert!(!ExtractedValue::List(vec!["item".to_string()]).is_empty());
  }

  // =============================================================================
  // extract_fields tests
  // =============================================================================

  #[test]
  fn test_extract_fields_single_field() {
    let fields = vec!["name".to_string()];
    let result = extract_fields("My API", &fields);
    assert_eq!(result.get("name"), Some(&"My API".to_string()));
    assert_eq!(result.len(), 1);
  }

  #[test]
  fn test_extract_fields_single_field_with_whitespace() {
    let fields = vec!["name".to_string()];
    let result = extract_fields("  Trimmed API  ", &fields);
    assert_eq!(result.get("name"), Some(&"Trimmed API".to_string()));
  }

  #[test]
  fn test_extract_fields_empty_fields() {
    let fields: Vec<String> = Vec::new();
    let result = extract_fields("Some text", &fields);
    assert!(result.is_empty());
  }

  #[test]
  fn test_extract_fields_empty_response() {
    let fields = vec!["name".to_string()];
    let result = extract_fields("", &fields);
    assert!(result.is_empty());
  }

  #[test]
  fn test_extract_fields_multiple_fields() {
    let fields = vec!["name".to_string(), "description".to_string()];
    let result = extract_fields("My API", &fields);
    // For multiple fields, only first field is populated
    assert_eq!(result.get("name"), Some(&"My API".to_string()));
    assert_eq!(result.get("description"), None);
  }

  // =============================================================================
  // extract_fields_with_types tests
  // =============================================================================

  #[test]
  fn test_extract_fields_with_types_single_text() {
    let mut spec = HashMap::new();
    spec.insert("name".to_string(), "text".to_string());
    let result = extract_fields_with_types("My API", &spec);
    assert_eq!(result.get("name"), Some(&"My API".to_string()));
  }

  #[test]
  fn test_extract_fields_with_types_single_integer() {
    let mut spec = HashMap::new();
    spec.insert("count".to_string(), "integer".to_string());
    let result = extract_fields_with_types("42 items", &spec);
    assert_eq!(result.get("count"), Some(&"42".to_string()));
  }

  #[test]
  fn test_extract_fields_with_types_single_boolean() {
    let mut spec = HashMap::new();
    spec.insert("enabled".to_string(), "boolean".to_string());
    let result = extract_fields_with_types("yes", &spec);
    assert_eq!(result.get("enabled"), Some(&"true".to_string()));
  }

  #[test]
  fn test_extract_fields_with_types_single_url() {
    let mut spec = HashMap::new();
    spec.insert("endpoint".to_string(), "url".to_string());
    let result = extract_fields_with_types("Visit https://api.example.com", &spec);
    assert_eq!(
      result.get("endpoint"),
      Some(&"https://api.example.com".to_string())
    );
  }

  #[test]
  fn test_extract_fields_with_types_empty_spec() {
    let spec = HashMap::new();
    let result = extract_fields_with_types("Some text", &spec);
    assert!(result.is_empty());
  }

  #[test]
  fn test_extract_fields_with_types_empty_response() {
    let mut spec = HashMap::new();
    spec.insert("name".to_string(), "text".to_string());
    let result = extract_fields_with_types("", &spec);
    assert!(result.is_empty());
  }

  #[test]
  fn test_extract_fields_with_types_multiple_fields() {
    let mut spec = HashMap::new();
    // Use two text fields to ensure extraction succeeds regardless of iteration order
    spec.insert("name".to_string(), "text".to_string());
    spec.insert("description".to_string(), "text".to_string());
    let result = extract_fields_with_types("My API", &spec);
    // Only first field is populated
    assert_eq!(result.len(), 1);
  }

  // =============================================================================
  // Edge cases and unicode tests
  // =============================================================================

  #[test]
  fn test_unicode_text_extraction() {
    let result = extract_text("\u{4e2d}\u{6587}\u{6d4b}\u{8bd5}");
    assert!(result.is_ok());
    assert_eq!(result, Ok("\u{4e2d}\u{6587}\u{6d4b}\u{8bd5}".to_string()));
  }

  #[test]
  fn test_unicode_name_extraction() {
    let result = extract_name("\u{65e5}\u{672c}\u{8a9e}\u{540d}\u{524d}");
    assert!(result.is_ok());
  }

  #[test]
  fn test_unicode_list() {
    let result =
      extract_list("\u{308a}\u{3093}\u{3054}, \u{308a}\u{3093}\u{3054}, \u{308a}\u{3093}\u{3054}");
    assert_eq!(result.len(), 3);
  }

  #[test]
  fn test_mixed_language_text() {
    let result = extract_text("Hello \u{4e16}\u{754c} World \u{4e16}\u{754c}");
    assert!(result.is_ok());
  }

  #[test]
  fn test_emoji_in_text() {
    let result = extract_text("Hello \u{1f44b} World");
    assert!(result.is_ok());
    assert_eq!(result, Ok("Hello \u{1f44b} World".to_string()));
  }

  #[test]
  fn test_number_with_thousands_separator() {
    // Note: This currently extracts just the first number
    let result = extract_integer("1,234,567");
    // The comma stops number parsing, so we get 1
    assert!(result.is_ok());
    assert_eq!(result, Ok(1));
  }

  #[test]
  fn test_scientific_notation_not_supported() {
    // Scientific notation is not supported
    let result = extract_float("1.5e10");
    assert!(result.is_ok());
    // Should extract 1.5
    if let Ok(ExtractedValue::Float(f)) = extract_by_type("1.5e10", "float") {
      assert!((f - 1.5).abs() < f64::EPSILON);
    }
  }

  #[test]
  fn test_url_with_port() {
    let result = extract_url("https://example.com:8080/path");
    assert!(result.is_ok());
    assert_eq!(result, Ok("https://example.com:8080/path".to_string()));
  }

  #[test]
  fn test_url_with_fragment() {
    let result = extract_url("https://example.com/page#section");
    assert!(result.is_ok());
    assert_eq!(result, Ok("https://example.com/page#section".to_string()));
  }

  #[test]
  fn test_email_with_plus() {
    let result = extract_email("user+tag@example.com");
    assert!(result.is_ok());
    assert_eq!(result, Ok("user+tag@example.com".to_string()));
  }

  #[test]
  fn test_empty_items_in_list() {
    let result = extract_list("a,,b,,,c");
    assert_eq!(result, vec!["a", "b", "c"]);
  }

  // =============================================================================
  // extract_from_answer tests - Interview answer extraction
  // =============================================================================

  #[test]
  fn test_extract_from_answer_jwt() {
    let response = "We use JWT tokens for authentication";
    let fields = vec!["auth_method".to_string()];
    let result = extract_from_answer("q1", response, &fields);
    assert_eq!(result.get("auth_method"), Some(&"jwt".to_string()));
  }

  #[test]
  fn test_extract_from_answer_oauth() {
    let response = "OAuth 2.0 is our auth standard";
    let fields = vec!["auth_method".to_string()];
    let result = extract_from_answer("q1", response, &fields);
    assert_eq!(result.get("auth_method"), Some(&"oauth".to_string()));
  }

  #[test]
  fn test_extract_from_answer_session() {
    let response = "We use session-based authentication";
    let fields = vec!["auth_method".to_string()];
    let result = extract_from_answer("q1", response, &fields);
    assert_eq!(result.get("auth_method"), Some(&"session".to_string()));
  }

  #[test]
  fn test_extract_from_answer_api_key() {
    let response = "We authenticate using api key";
    let fields = vec!["auth_method".to_string()];
    let result = extract_from_answer("q1", response, &fields);
    assert_eq!(result.get("auth_method"), Some(&"api_key".to_string()));
  }

  #[test]
  fn test_extract_from_answer_none() {
    let response = "No authentication needed, none required";
    let fields = vec!["auth_method".to_string()];
    let result = extract_from_answer("q1", response, &fields);
    assert_eq!(result.get("auth_method"), Some(&"none".to_string()));
  }

  #[test]
  fn test_extract_from_answer_unknown_auth() {
    let response = "We use some custom authentication";
    let fields = vec!["auth_method".to_string()];
    let result = extract_from_answer("q1", response, &fields);
    assert!(!result.contains_key("auth_method"));
  }

  #[test]
  fn test_extract_from_answer_entities() {
    let response = "Users, Orders, Products, Payments";
    let fields = vec!["entities".to_string()];
    let result = extract_from_answer("q1", response, &fields);
    assert!(result.contains_key("entities"));
    let entities = result.get("entities").expect("entities should be present");
    assert!(entities.contains("Users"));
    assert!(entities.contains("Orders"));
    assert!(entities.contains("Products"));
    assert!(entities.contains("Payments"));
  }

  #[test]
  fn test_extract_from_answer_audience_mobile() {
    let response = "Mainly mobile app users";
    let fields = vec!["audience".to_string()];
    let result = extract_from_answer("q1", response, &fields);
    assert_eq!(result.get("audience"), Some(&"mobile".to_string()));
  }

  #[test]
  fn test_extract_from_answer_audience_web() {
    let response = "Our web application users";
    let fields = vec!["audience".to_string()];
    let result = extract_from_answer("q1", response, &fields);
    assert_eq!(result.get("audience"), Some(&"web".to_string()));
  }

  #[test]
  fn test_extract_from_answer_audience_api() {
    let response = "For our API consumers";
    let fields = vec!["audience".to_string()];
    let result = extract_from_answer("q1", response, &fields);
    assert_eq!(result.get("audience"), Some(&"api".to_string()));
  }

  #[test]
  fn test_extract_from_answer_audience_cli() {
    let response = "CLI users will interact with this";
    let fields = vec!["audience".to_string()];
    let result = extract_from_answer("q1", response, &fields);
    assert_eq!(result.get("audience"), Some(&"cli".to_string()));
  }

  #[test]
  fn test_extract_from_answer_audience_internal() {
    let response = "Internal team members only";
    let fields = vec!["audience".to_string()];
    let result = extract_from_answer("q1", response, &fields);
    assert_eq!(result.get("audience"), Some(&"internal".to_string()));
  }

  #[test]
  fn test_extract_from_answer_multiple_fields() {
    let response =
      "We use JWT tokens for our mobile app users. Main entities are Users and Orders.";
    let fields = vec![
      "auth_method".to_string(),
      "audience".to_string(),
      "entities".to_string(),
    ];
    let result = extract_from_answer("q1", response, &fields);
    assert_eq!(result.get("auth_method"), Some(&"jwt".to_string()));
    assert_eq!(result.get("audience"), Some(&"mobile".to_string()));
    assert!(result.contains_key("entities"));
  }

  #[test]
  fn test_extract_from_answer_generic_field() {
    let response = "Some generic response text";
    let fields = vec!["custom_field".to_string()];
    let result = extract_from_answer("q1", response, &fields);
    assert_eq!(result.get("custom_field"), Some(&response.to_string()));
  }

  #[test]
  fn test_extract_from_answer_empty_response() {
    let response = "";
    let fields = vec!["auth_method".to_string()];
    let result = extract_from_answer("q1", response, &fields);
    assert!(!result.contains_key("auth_method"));
  }

  #[test]
  fn test_extract_from_answer_whitespace_response() {
    let response = "   ";
    let fields = vec!["auth_method".to_string()];
    let result = extract_from_answer("q1", response, &fields);
    assert!(!result.contains_key("auth_method"));
  }

  #[test]
  fn test_extract_from_answer_no_matching_fields() {
    let response = "Some text without any recognizable patterns";
    let fields = vec!["auth_method".to_string(), "audience".to_string()];
    let result = extract_from_answer("q1", response, &fields);
    assert!(result.is_empty());
  }

  #[test]
  fn test_extract_from_answer_empty_fields() {
    let response = "Some response";
    let fields: Vec<String> = vec![];
    let result = extract_from_answer("q1", response, &fields);
    assert!(result.is_empty());
  }

  // =============================================================================
  // extract_auth_method tests
  // =============================================================================

  #[test]
  fn test_extract_auth_method_jwt_case_insensitive() {
    assert_eq!(extract_auth_method("JWT tokens"), Some("jwt".to_string()));
    assert_eq!(extract_auth_method("jwt"), Some("jwt".to_string()));
    assert_eq!(extract_auth_method("Jwt"), Some("jwt".to_string()));
  }

  #[test]
  fn test_extract_auth_method_oauth_case_insensitive() {
    assert_eq!(extract_auth_method("OAuth 2.0"), Some("oauth".to_string()));
    assert_eq!(extract_auth_method("oauth"), Some("oauth".to_string()));
    assert_eq!(extract_auth_method("OAUTH"), Some("oauth".to_string()));
  }

  #[test]
  fn test_extract_auth_method_session_case_insensitive() {
    assert_eq!(
      extract_auth_method("Session based auth"),
      Some("session".to_string())
    );
    assert_eq!(extract_auth_method("SESSION"), Some("session".to_string()));
  }

  #[test]
  fn test_extract_auth_method_api_key_variants() {
    assert_eq!(extract_auth_method("api key"), Some("api_key".to_string()));
    assert_eq!(extract_auth_method("api_key"), Some("api_key".to_string()));
    assert_eq!(extract_auth_method("API KEY"), Some("api_key".to_string()));
    assert_eq!(extract_auth_method("API_KEY"), Some("api_key".to_string()));
  }

  #[test]
  fn test_extract_auth_method_none() {
    assert_eq!(
      extract_auth_method("none required"),
      Some("none".to_string())
    );
    assert_eq!(extract_auth_method("None"), Some("none".to_string()));
  }

  #[test]
  fn test_extract_auth_method_not_found() {
    assert_eq!(extract_auth_method("custom auth"), None);
    assert_eq!(extract_auth_method("password"), None);
    assert_eq!(extract_auth_method(""), None);
  }

  // =============================================================================
  // extract_entities tests
  // =============================================================================

  #[test]
  fn test_extract_entities_multiple() {
    let result = extract_entities("Users, Orders, Products");
    assert!(result.is_some());
    let entities = result.expect("should have entities");
    assert!(entities.contains("Users"));
    assert!(entities.contains("Orders"));
    assert!(entities.contains("Products"));
  }

  #[test]
  fn test_extract_entities_filters_common_words() {
    let result = extract_entities("The And For Use Can All Our You Are");
    assert!(result.is_none());
  }

  #[test]
  fn test_extract_entities_handles_punctuation() {
    let result = extract_entities("Users, Orders. Products!");
    assert!(result.is_some());
    let entities = result.expect("should have entities");
    assert!(entities.contains("Users"));
    assert!(entities.contains("Orders"));
    assert!(entities.contains("Products"));
  }

  #[test]
  fn test_extract_entities_short_words_filtered() {
    // "Cd" is only 2 chars, should be filtered
    let result = extract_entities("A B Cd Efg");
    assert_eq!(result, Some("Efg".to_string()));
  }

  #[test]
  fn test_extract_entities_mixed_case() {
    let result = extract_entities("userName Product API");
    // Both Product and API start with uppercase and are > 2 chars
    let entities = result.expect("should have entities");
    assert!(entities.contains("Product"));
    assert!(entities.contains("API"));
  }

  #[test]
  fn test_extract_entities_empty() {
    assert_eq!(extract_entities(""), None);
    assert_eq!(extract_entities("the a"), None);
  }

  #[test]
  fn test_extract_entities_with_text() {
    let result = extract_entities("The main entities are Users and Orders in the system");
    assert!(result.is_some());
    let entities = result.expect("should have entities");
    assert!(entities.contains("Users"));
    assert!(entities.contains("Orders"));
  }

  // =============================================================================
  // extract_audience tests
  // =============================================================================

  #[test]
  fn test_extract_audience_mobile() {
    assert_eq!(extract_audience("mobile app"), Some("mobile".to_string()));
    assert_eq!(extract_audience("Mobile users"), Some("mobile".to_string()));
    assert_eq!(extract_audience("MOBILE"), Some("mobile".to_string()));
  }

  #[test]
  fn test_extract_audience_web() {
    assert_eq!(extract_audience("web application"), Some("web".to_string()));
    assert_eq!(extract_audience("Web users"), Some("web".to_string()));
    assert_eq!(extract_audience("WEB"), Some("web".to_string()));
  }

  #[test]
  fn test_extract_audience_api() {
    assert_eq!(extract_audience("API consumers"), Some("api".to_string()));
    assert_eq!(extract_audience("api users"), Some("api".to_string()));
    assert_eq!(extract_audience("API"), Some("api".to_string()));
  }

  #[test]
  fn test_extract_audience_cli() {
    assert_eq!(extract_audience("CLI tool"), Some("cli".to_string()));
    assert_eq!(extract_audience("a cli client"), Some("cli".to_string()));
    assert_eq!(extract_audience("CLI"), Some("cli".to_string()));
  }

  #[test]
  fn test_extract_audience_internal() {
    assert_eq!(
      extract_audience("internal team"),
      Some("internal".to_string())
    );
    assert_eq!(
      extract_audience("Internal use"),
      Some("internal".to_string())
    );
    assert_eq!(extract_audience("INTERNAL"), Some("internal".to_string()));
  }

  #[test]
  fn test_extract_audience_not_found() {
    assert_eq!(extract_audience("general users"), None);
    assert_eq!(extract_audience("everyone"), None);
    assert_eq!(extract_audience(""), None);
  }

  // =============================================================================
  // calculate_confidence tests
  // =============================================================================

  #[test]
  fn test_calculate_confidence_high() {
    let mut extracted = HashMap::new();
    extracted.insert("field1".to_string(), "value1".to_string());
    let response = "This is a longer response with more than fifty characters for testing.";
    let confidence = calculate_confidence("q1", response, &extracted);
    assert!((confidence - 0.85).abs() < 0.001);
  }

  #[test]
  fn test_calculate_confidence_low() {
    let extracted = HashMap::new();
    let response = "Short";
    let confidence = calculate_confidence("q1", response, &extracted);
    assert!((confidence - 0.6).abs() < 0.001);
  }

  #[test]
  fn test_calculate_confidence_medium_no_fields() {
    let extracted = HashMap::new();
    let response = "This is a longer response with more than fifty characters for testing.";
    let confidence = calculate_confidence("q1", response, &extracted);
    assert!((confidence - 0.6).abs() < 0.001);
  }

  #[test]
  fn test_calculate_confidence_medium_short_response() {
    let mut extracted = HashMap::new();
    extracted.insert("field1".to_string(), "value1".to_string());
    let response = "Short";
    let confidence = calculate_confidence("q1", response, &extracted);
    assert!((confidence - 0.6).abs() < 0.001);
  }

  // =============================================================================
  // Edge case tests
  // =============================================================================

  #[test]
  fn test_extract_from_answer_question_id_unused() {
    // The question_id is unused but should not cause issues
    let response = "We use JWT for auth";
    let fields = vec!["auth_method".to_string()];
    let result = extract_from_answer("different_id", response, &fields);
    assert_eq!(result.get("auth_method"), Some(&"jwt".to_string()));
  }

  #[test]
  fn test_extract_from_answer_case_preservation() {
    // Extracted values should be lowercase for auth_method
    let response = "We use JWT tokens";
    let fields = vec!["auth_method".to_string()];
    let result = extract_from_answer("q1", response, &fields);
    assert_eq!(result.get("auth_method"), Some(&"jwt".to_string()));
  }

  #[test]
  fn test_entities_with_numbers() {
    let result = extract_entities("User123 Account456");
    // Numbers are kept as part of the entity
    assert!(result.is_some());
  }

  #[test]
  fn test_audience_priority_order() {
    // When multiple audiences mentioned, first match wins
    // mobile comes before web in the implementation
    let result = extract_audience("mobile and web users");
    assert_eq!(result, Some("mobile".to_string()));
  }
}
