//! Answer Extraction
//!
//! Extracts structured data from free-text interview answers.
//! Parses various data types (strings, numbers, booleans, URLs, emails, lists)
//! from natural language responses.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use std::collections::HashMap;

use thiserror::Error;

/// Errors during answer extraction
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ExtractionError {
  #[error("empty response text")]
  EmptyResponse,

  #[error("invalid number format: {0}")]
  InvalidNumber(String),

  #[error("invalid boolean format: {0}")]
  InvalidBoolean(String),

  #[error("invalid URL format: {0}")]
  InvalidUrl(String),

  #[error("invalid email format: {0}")]
  InvalidEmail(String),

  #[error("field not found: {0}")]
  FieldNotFound(String),

  #[error("extraction failed for type '{expected}': {reason}")]
  ExtractionFailed { expected: String, reason: String },
}

/// Result of extracting a single field
#[derive(Debug, Clone, PartialEq)]
pub enum ExtractedValue {
  Text(String),
  Integer(i64),
  Float(f64),
  Boolean(bool),
  Url(String),
  Email(String),
  List(Vec<String>),
}

impl ExtractedValue {
  /// Convert to a string representation
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

  /// Check if the value is empty
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

/// Extract a text string from a response
///
/// # Errors
/// Returns `ExtractionError::EmptyResponse` if the response is empty or whitespace only.
pub fn extract_text(response: &str) -> Result<String, ExtractionError> {
  let trimmed = response.trim();
  if trimmed.is_empty() {
    return Err(ExtractionError::EmptyResponse);
  }
  Ok(trimmed.to_string())
}

/// Extract a name (trimmed, single-line text) from a response
///
/// # Errors
/// Returns `ExtractionError::EmptyResponse` if the name is empty after processing.
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

/// Extract an integer from a response
///
/// # Errors
/// Returns `ExtractionError::InvalidNumber` if the number cannot be parsed.
pub fn extract_integer(response: &str) -> Result<i64, ExtractionError> {
  let trimmed = response.trim();

  // Try to extract the first number-like sequence
  let number_str = extract_number_sequence(trimmed);
  number_str
    .parse::<i64>()
    .map_err(|_| ExtractionError::InvalidNumber(trimmed.to_string()))
}

/// Extract a float from a response
///
/// # Errors
/// Returns `ExtractionError::InvalidNumber` if the number cannot be parsed.
pub fn extract_float(response: &str) -> Result<f64, ExtractionError> {
  let trimmed = response.trim();

  // Try to extract the first number-like sequence
  let number_str = extract_number_sequence(trimmed);
  number_str
    .parse::<f64>()
    .map_err(|_| ExtractionError::InvalidNumber(trimmed.to_string()))
}

/// Extract a boolean from a response
///
/// Recognizes: yes/no, true/false, on/off, 1/0
///
/// # Errors
/// Returns `ExtractionError::InvalidBoolean` if the boolean cannot be parsed.
pub fn extract_boolean(response: &str) -> Result<bool, ExtractionError> {
  let trimmed = response.trim().to_lowercase();

  match trimmed.as_str() {
    "yes" | "true" | "on" | "1" | "y" | "t" => Ok(true),
    "no" | "false" | "off" | "0" | "n" | "f" => Ok(false),
    _ => Err(ExtractionError::InvalidBoolean(response.trim().to_string())),
  }
}

/// Extract a URL from a response
///
/// # Errors
/// Returns `ExtractionError::InvalidUrl` if the URL is invalid.
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

/// Extract an email from a response
///
/// # Errors
/// Returns `ExtractionError::InvalidEmail` if the email is invalid.
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

/// Extract a list of items from a response
///
/// Parses comma-separated, newline-separated, or numbered lists.
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

/// Extract a value by type name
///
/// # Errors
/// Returns an appropriate `ExtractionError` if extraction fails.
pub fn extract_by_type(response: &str, type_name: &str) -> Result<ExtractedValue, ExtractionError> {
  match type_name {
    "name" => extract_name(response).map(ExtractedValue::Text),
    "integer" | "int" | "number" => {
      extract_integer(response).map(ExtractedValue::Integer)
    }
    "float" | "decimal" => extract_float(response).map(ExtractedValue::Float),
    "boolean" | "bool" => extract_boolean(response).map(ExtractedValue::Boolean),
    "url" | "uri" => extract_url(response).map(ExtractedValue::Url),
    "email" => extract_email(response).map(ExtractedValue::Email),
    "list" | "array" => Ok(ExtractedValue::List(extract_list(response))),
    _ => extract_text(response).map(ExtractedValue::Text),
  }
}

/// Extract multiple fields from a response based on field specifications
///
/// # Arguments
/// * `response` - The free-text response to extract from
/// * `fields` - List of field names to extract (uses default "text" type)
///
/// # Returns
/// A `HashMap` of field names to extracted string values.
/// For single-field extraction, the entire response is assigned to that field.
/// For multi-field extraction, the response is assigned to the first field.
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

/// Extract fields with type specifications
///
/// # Arguments
/// * `response` - The free-text response to extract from
/// * `spec` - `HashMap` of field names to type names
///
/// # Returns
/// A `HashMap` of field names to extracted string values.
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
// Helper Functions
// =============================================================================

/// Extract the first number-like sequence from a string
fn extract_number_sequence(s: &str) -> String {
  let mut result = String::new();
  let mut seen_decimal = false;
  let mut seen_sign = false;
  let mut started = false;

  for ch in s.chars() {
    match ch {
      '-' | '+' if !started && !seen_sign => {
        result.push(ch);
        seen_sign = true;
      }
      '.' if started && !seen_decimal => {
        result.push(ch);
        seen_decimal = true;
      }
      '0'..='9' => {
        result.push(ch);
        started = true;
      }
      _ if started => {
        break;
      }
      _ => {}
    }
  }

  if result.is_empty() {
    s.to_string()
  } else {
    result
  }
}

/// Extract a URL pattern from text
fn extract_url_pattern(s: &str) -> Option<String> {
  // Look for http:// or https:// URLs
  let lower = s.to_lowercase();
  let start = lower.find("http://").or_else(|| lower.find("https://"))?;

  let rest = &s[start..];
  let end = rest
    .find(|c: char| c.is_whitespace() || c == ',' || c == ')' || c == ']' || c == '}')
    .unwrap_or(rest.len());

  let url = &rest[..end];

  // Strip trailing period if present (likely end of sentence, not part of URL)
  let url = url.strip_suffix('.').unwrap_or(url);

  Some(url.to_string())
}

/// Extract an email pattern from text
fn extract_email_pattern(s: &str) -> Option<&str> {
  // Find @ symbol
  let at_pos = s.find('@')?;

  // Find start of email (look backwards for whitespace or start)
  let start = s[..at_pos]
    .rfind(|c: char| c.is_whitespace() || c == '<' || c == '(' || c == '[')
    .map_or(0, |pos| pos + 1);

  // Find end of email (look forwards for whitespace or end)
  let rest = &s[at_pos..];
  let end_offset = rest
    .find(|c: char| c.is_whitespace() || c == '>' || c == ')' || c == ']' || c == ',')
    .unwrap_or(rest.len());

  let email = &s[start..at_pos + end_offset];

  // Strip trailing period if present (likely end of sentence, not part of email)
  let email = email.strip_suffix('.').unwrap_or(email);

  Some(email)
}

/// Parse a numbered list (1. item, 2. item, etc.)
fn parse_numbered_list(s: &str) -> Vec<String> {
  let mut items = Vec::new();

  for line in s.lines() {
    let trimmed = line.trim();

    // Match patterns like "1. item", "2) item", "3- item", etc.
    if let Some(after_number) = trimmed
      .strip_prefix(|c: char| c.is_ascii_digit())
      .map(str::trim_start)
    {
      // Skip any additional digits (for numbers >= 10)
      let after_number = after_number
        .trim_start_matches(|c: char| c.is_ascii_digit())
        .trim_start();

      // Skip the separator (. or ) or - or :)
      let content = after_number
        .strip_prefix(['.', ')', '-', ':'])
        .map(str::trim)
        .unwrap_or_else(|| after_number.trim());

      if !content.is_empty() {
        items.push(content.to_string());
      }
    }
  }

  items
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
    assert_eq!(result, Ok("https://example.com/path/to/resource".to_string()));
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
    assert_eq!(result, Ok(ExtractedValue::Url("https://example.com".to_string())));
  }

  #[test]
  fn test_extract_by_type_uri_alias() {
    let result = extract_by_type("https://example.com", "uri");
    assert!(result.is_ok());
    assert_eq!(result, Ok(ExtractedValue::Url("https://example.com".to_string())));
  }

  #[test]
  fn test_extract_by_type_email() {
    let result = extract_by_type("user@example.com", "email");
    assert!(result.is_ok());
    assert_eq!(result, Ok(ExtractedValue::Email("user@example.com".to_string())));
  }

  #[test]
  fn test_extract_by_type_list() {
    let result = extract_by_type("a, b, c", "list");
    assert!(result.is_ok());
    assert_eq!(result, Ok(ExtractedValue::List(vec![
      "a".to_string(),
      "b".to_string(),
      "c".to_string()
    ])));
  }

  #[test]
  fn test_extract_by_type_array_alias() {
    let result = extract_by_type("a, b, c", "array");
    assert!(result.is_ok());
    assert_eq!(result, Ok(ExtractedValue::List(vec![
      "a".to_string(),
      "b".to_string(),
      "c".to_string()
    ])));
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
    let value = ExtractedValue::List(vec![
      "a".to_string(),
      "b".to_string(),
      "c".to_string(),
    ]);
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
    spec.insert("name".to_string(), "text".to_string());
    spec.insert("count".to_string(), "integer".to_string());
    let result = extract_fields_with_types("My API", &spec);
    // Only first field (in arbitrary order) is populated
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
    let result = extract_list("\u{308a}\u{3093}\u{3054}, \u{308a}\u{3093}\u{3054}, \u{308a}\u{3093}\u{3054}");
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
}
