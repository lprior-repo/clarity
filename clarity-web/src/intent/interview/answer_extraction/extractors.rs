//! Answer Extractors
//!
//! This module provides pure functions for extracting structured data from
//! free-text interview responses. Each extractor handles a specific data type.

use std::collections::HashMap;

use crate::intent::interview::answer_extraction::types::{ExtractedValue, ExtractionError};
use crate::intent::interview::answer_extraction::helpers::{extract_email_pattern, extract_number_sequence, extract_url_pattern};
use crate::intent::interview::answer_extraction::parsers::parse_numbered_list;

/// Extracts a text string from a response, trimming whitespace.
///
/// This is the simplest extraction function, returning the input text
/// with leading and trailing whitespace removed. It serves as the fallback
/// for unknown types in [`extract_by_type`].
///
/// # Example
///
/// ```ignore
/// use clarity_web::intent::interview::answer_extraction::extractors::extract_text;
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
/// use clarity_web::intent::interview::answer_extraction::extractors::extract_name;
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
/// use clarity_web::intent::interview::answer_extraction::extractors::extract_integer;
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
/// use clarity_web::intent::interview::answer_extraction::extractors::extract_float;
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
/// use clarity_web::intent::interview::answer_extraction::extractors::extract_boolean;
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
/// use clarity_web::intent::interview::answer_extraction::extractors::extract_url;
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
/// use clarity_web::intent::interview::answer_extraction::extractors::extract_email;
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
/// use clarity_web::intent::interview::answer_extraction::extractors::extract_list;
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

  // Try numbered list format first
  let numbered = parse_numbered_list(trimmed);
  if !numbered.is_empty() {
    return numbered;
  }

  // Try newline-separated
  let newline_items: Vec<String> = trimmed
    .lines()
    .map(str::trim)
    .filter(|s| !s.is_empty())
    .map(String::from)
    .collect();

  if newline_items.len() > 1 {
    return newline_items;
  }

  // Fall back to comma-separated
  trimmed
    .split(',')
    .map(str::trim)
    .filter(|s| !s.is_empty())
    .map(String::from)
    .collect()
}

/// Extracts a value by type name, dispatching to the appropriate extractor.
///
/// This is the main entry point for type-based extraction. Given a response
/// and a type name, it selects and applies the appropriate extraction function.
///
/// # Supported Types
///
/// | Type Name | Extractor | Return Type |
/// |-----------|-----------|-------------|
/// | `"text"` | [`extract_text`] | `String` |
/// | `"name"` | [`extract_name`] | `String` |
/// | `"integer"` | [`extract_integer`] | `i64` |
/// | `"float"` | [`extract_float`] | `f64` |
/// | `"boolean"` | [`extract_boolean`] | `bool` |
/// | `"url"` | [`extract_url`] | `String` |
/// | `"email"` | [`extract_email`] | `String` |
/// | `"list"` | [`extract_list`] | `Vec<String>` |
/// | (unknown) | [`extract_text`] | `String` |
///
/// # Example
///
/// ```ignore
/// use clarity_web::intent::interview::answer_extraction::extractors::extract_by_type;
///
/// assert_eq!(extract_by_type("42", "integer"), Ok(ExtractedValue::Integer(42)));
/// assert_eq!(extract_by_type("yes", "boolean"), Ok(ExtractedValue::Boolean(true)));
/// assert_eq!(extract_by_type("https://example.com", "url"),
///     Ok(ExtractedValue::Url("https://example.com".to_string())));
/// ```
///
/// # Errors
///
/// Returns error if the underlying extractor fails for the given type.
pub fn extract_by_type(response: &str, type_name: &str) -> Result<ExtractedValue, ExtractionError> {
  match type_name.to_lowercase().as_str() {
    "text" | "string" => extract_text(response).map(ExtractedValue::Text),
    "name" => extract_name(response).map(ExtractedValue::Text),
    "integer" | "int" | "number" => extract_integer(response).map(ExtractedValue::Integer),
    "float" | "decimal" => extract_float(response).map(ExtractedValue::Float),
    "boolean" | "bool" => extract_boolean(response).map(ExtractedValue::Boolean),
    "url" | "uri" => extract_url(response).map(ExtractedValue::Url),
    "email" => extract_email(response).map(ExtractedValue::Email),
    "list" | "array" => Ok(ExtractedValue::List(extract_list(response))),
    // Default: treat as plain text
    _ => extract_text(response).map(ExtractedValue::Text),
  }
}

/// Extracts multiple fields from a response, returning string values.
///
/// This function applies [`extract_by_type`] to each field but always returns
/// the string representation of the extracted value (via [`ExtractedValue::to_string_value`]).
///
/// # Arguments
///
/// * `response` - The free-text response to extract from
/// * `fields` - Slice of field specifications in "name:type" format
///
/// # Example
///
/// ```ignore
/// use clarity_web::intent::interview::answer_extraction::extractors::extract_fields;
///
/// let response = "42 items";
/// let fields = vec!["count:integer", "name:text"];
/// let result = extract_fields(response, &fields);
/// assert_eq!(result.get("count"), Some(&"42".to_string()));
/// ```
#[must_use]
pub fn extract_fields(response: &str, fields: &[String]) -> HashMap<String, String> {
  let mut result = HashMap::new();

  for field_spec in fields {
    // Parse field name and type from "name:type" format
    // If no colon, default to text type
    let (field_name, type_name) = match field_spec.split_once(':') {
      Some((name, type_n)) => (name.trim(), type_n.trim()),
      None => (field_spec.as_str().trim(), "text"),
    };

    if field_name.is_empty() {
      continue;
    }

    match extract_by_type(response, type_name) {
      Ok(value) => {
        if !value.is_empty() {
          result.insert(field_name.to_string(), value.to_string_value());
          break; // Only populate first field
        }
      }
      Err(_) => {}
    }
  }

  result
}

/// Extracts multiple fields with explicit type mapping.
///
/// This is a more type-safe version of [`extract_fields`] that accepts a pre-parsed
/// HashMap of field names to type names.
///
/// # Type Parameter
///
/// * `S` - The hasher used for the input HashMap (typically `std::collections::hash_map::RandomState`)
///
/// # Example
///
/// ```ignore
/// use std::collections::HashMap;
/// use clarity_web::intent::interview::answer_extraction::extractors::extract_fields_with_types;
///
/// let response = "https://example.com";
/// let mut fields: HashMap<String, String> = HashMap::new();
/// fields.insert("url".to_string(), "url".to_string());
///
/// let result = extract_fields_with_types(response, &fields);
/// assert_eq!(result.get("url"), Some(&"https://example.com".to_string()));
/// ```
#[must_use]
pub fn extract_fields_with_types<S: std::hash::BuildHasher>(
  response: &str,
  fields: &HashMap<String, String, S>,
) -> HashMap<String, String> {
  let mut result = HashMap::new();

  for (field_name, type_name) in fields {
    match extract_by_type(response, type_name) {
      Ok(value) => {
        if !value.is_empty() {
          result.insert(field_name.clone(), value.to_string_value());
          break; // Only populate first field
        }
      }
      Err(_) => {}
    }
  }

  result
}
