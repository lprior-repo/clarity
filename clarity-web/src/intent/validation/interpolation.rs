//! Variable Interpolation System (WP20)
//!
//! Provides variable interpolation for strings with support for:
//! - Simple variable references: `${var}`
//! - Nested path navigation: `${obj.field}`
//! - Array indexing: `${items[0]}`, `${items[-1]}`, `${items[*]}`
//! - Request/response body access: `${request.body.field}`
//!
//! ## Example
//!
//! ```ignore
//! use intent::validation::interpolation::{interpolate_string, Context};
//!
//! let context = Context {
//!     variables: [("name".to_string(), "Alice".to_string())].into_iter().collect(),
//!     request_body: Some(json!({"email": "alice@example.com"})),
//!     response_body: None,
//! };
//!
//! let result = interpolate_string("Hello, ${name}!", &context)?;
//! assert_eq!(result, "Hello, Alice!");
//! ```

#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::intent::util::array_indexing::{navigate_path, split_path, ArrayIndexError};
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

// =============================================================================
// Error Types
// =============================================================================

/// Error taxonomy for interpolation operations
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum InterpolationError {
  /// Variable not found in context
  #[error("variable not found: {0}")]
  VariableNotFound(String),

  /// Invalid path syntax
  #[error("invalid path: {0}")]
  InvalidPath(String),

  /// Array index out of bounds
  #[error("array index out of bounds: {index} in array of length {length}")]
  IndexOutOfBounds {
    /// The requested index
    index: usize,
    /// The actual array length
    length: usize,
  },

  /// Tried to index a non-array value
  #[error("not an array: {0}")]
  NotAnArray(String),

  /// JSON processing error
  #[error("JSON error: {0}")]
  JsonError(String),

  /// Regex error during pattern matching
  #[error("regex error: {0}")]
  RegexError(String),

  /// Malformed interpolation syntax
  #[error("malformed interpolation: {0}")]
  MalformedInterpolation(String),
}

impl From<ArrayIndexError> for InterpolationError {
  fn from(err: ArrayIndexError) -> Self {
    match err {
      ArrayIndexError::InvalidPath(path) => Self::InvalidPath(path),
      ArrayIndexError::IndexOutOfBounds { index, length } => Self::IndexOutOfBounds {
        index: index.max(0).cast_unsigned(),
        length,
      },
      ArrayIndexError::NotAnArray { field, .. } => Self::NotAnArray(field),
      ArrayIndexError::FieldNotFound(field) => Self::VariableNotFound(field),
    }
  }
}

// =============================================================================
// Context Type
// =============================================================================

/// Context for variable interpolation
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Context {
  /// Simple variable substitutions
  pub variables: HashMap<String, String>,
  /// Optional request body for HTTP context
  pub request_body: Option<Value>,
  /// Optional response body for HTTP context
  pub response_body: Option<Value>,
}

impl Context {
  /// Create a new empty context
  #[must_use]
  pub fn new() -> Self {
    Self {
      variables: HashMap::new(),
      request_body: None,
      response_body: None,
    }
  }

  /// Create a context with variables from an iterator
  #[must_use]
  pub fn from_variables<I, K, V>(vars: I) -> Self
  where
    I: IntoIterator<Item = (K, V)>,
    K: Into<String>,
    V: Into<String>,
  {
    Self {
      variables: vars
        .into_iter()
        .map(|(k, v)| (k.into(), v.into()))
        .collect(),
      request_body: None,
      response_body: None,
    }
  }

  /// Add a variable to the context
  #[must_use]
  pub fn with_variable(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
    self.variables.insert(key.into(), value.into());
    self
  }

  /// Set the request body
  #[must_use]
  pub fn with_request_body(mut self, body: Value) -> Self {
    self.request_body = Some(body);
    self
  }

  /// Set the response body
  #[must_use]
  pub fn with_response_body(mut self, body: Value) -> Self {
    self.response_body = Some(body);
    self
  }
}

// =============================================================================
// Interpolation Functions
// =============================================================================

/// Find all interpolation placeholders in a string
///
/// Returns a list of (start, end, `variable_name`) tuples
fn find_placeholders(input: &str) -> Vec<(usize, usize, String)> {
  let mut placeholders = Vec::new();
  let chars: Vec<char> = input.chars().collect();
  let mut i = 0;

  while i < chars.len() {
    // Look for ${
    if i + 1 < chars.len() && chars[i] == '$' && chars[i + 1] == '{' {
      let start = i;
      i += 2; // Skip ${
      let mut var_name = String::new();
      let mut brace_count = 1;

      // Find matching closing brace
      while i < chars.len() && brace_count > 0 {
        match chars[i] {
          '{' => {
            brace_count += 1;
            var_name.push(chars[i]);
          }
          '}' => {
            brace_count -= 1;
            if brace_count > 0 {
              var_name.push(chars[i]);
            }
          }
          _ => {
            var_name.push(chars[i]);
          }
        }
        i += 1;
      }

      // Only add if we found the closing brace
      if brace_count == 0 {
        let var_name = var_name.trim().to_string();
        if !var_name.is_empty() {
          placeholders.push((start, i, var_name));
        }
      }
    } else {
      i += 1;
    }
  }

  placeholders
}

/// Interpolate variables in a string using the provided context
///
/// Supports:
/// - Simple variables: `${name}` -> looks up "name" in context.variables
/// - Nested paths: `${user.email}` -> navigates path in request/response body
/// - Array indexing: `${items[0]}`, `${items[-1]}`, `${items[*]}`
/// - Request/response: `${request.body.field}`, `${response.body.field}`
///
/// # Errors
///
/// Returns `InterpolationError` if:
/// - A variable is not found in the context
/// - A path is invalid or cannot be navigated
/// - An array index is out of bounds
///
/// # Example
///
/// ```
/// # use clarity_web::intent::validation::interpolation::{interpolate_string, Context};
/// # use std::collections::HashMap;
/// let context = Context::from_variables([("name", "Alice")]);
///
/// let result = interpolate_string("Hello, ${name}!", &context)?;
/// assert_eq!(result, "Hello, Alice!");
/// # Ok::<(), clarity_web::intent::validation::interpolation::InterpolationError>(())
/// ```
pub fn interpolate_string(input: &str, context: &Context) -> Result<String, InterpolationError> {
  let placeholders = find_placeholders(input);

  // If no placeholders, return input as-is
  if placeholders.is_empty() {
    return Ok(input.to_string());
  }

  // Build the result by replacing placeholders
  let mut result = String::new();
  let mut last_end = 0;

  for (start, end, var_name) in placeholders {
    // Add text before this placeholder
    result.push_str(&input[last_end..start]);

    // Resolve the variable
    let resolved = resolve_variable(&var_name, context)?;
    result.push_str(&resolved);

    last_end = end;
  }

  // Add remaining text after last placeholder
  if last_end < input.len() {
    result.push_str(&input[last_end..]);
  }

  Ok(result)
}

/// Resolve a variable name to its value
///
/// Handles special prefixes:
/// - `request.` -> looks up in `request_body`
/// - `response.` -> looks up in `response_body`
/// - Otherwise -> checks variables first, then bodies
fn resolve_variable(var_name: &str, context: &Context) -> Result<String, InterpolationError> {
  // Handle special prefixes
  if let Some(rest) = var_name.strip_prefix("request.") {
    return resolve_from_body(rest, context.request_body.as_ref(), "request");
  }

  if let Some(rest) = var_name.strip_prefix("response.") {
    return resolve_from_body(rest, context.response_body.as_ref(), "response");
  }

  // Check simple variables first
  if let Some(value) = context.variables.get(var_name) {
    return Ok(value.clone());
  }

  // Try to resolve from request body
  if let Some(ref body) = context.request_body {
    if let Ok(value) = resolve_from_body(var_name, Some(body), "request") {
      return Ok(value);
    }
  }

  // Try to resolve from response body
  if let Some(ref body) = context.response_body {
    if let Ok(value) = resolve_from_body(var_name, Some(body), "response") {
      return Ok(value);
    }
  }

  Err(InterpolationError::VariableNotFound(var_name.to_string()))
}

/// Resolve a path from a JSON body
fn resolve_from_body(
  path: &str,
  body: Option<&Value>,
  body_name: &str,
) -> Result<String, InterpolationError> {
  let body =
    body.ok_or_else(|| InterpolationError::VariableNotFound(format!("{body_name}.body")))?;

  // Handle "body" prefix in path
  let actual_path = path.strip_prefix("body.").unwrap_or(path);

  if actual_path.is_empty() || actual_path == "body" {
    // Return the entire body as JSON string
    return serde_json::to_string(body).map_err(|e| InterpolationError::JsonError(e.to_string()));
  }

  let path_parts = split_path(actual_path);
  let value = navigate_path(body, &path_parts).map_err(InterpolationError::from)?;

  // Convert value to string
  value_to_string(&value)
}

/// Convert a JSON value to a string representation
fn value_to_string(value: &Value) -> Result<String, InterpolationError> {
  match value {
    Value::Null => Ok(String::new()),
    Value::Bool(b) => Ok(b.to_string()),
    Value::Number(n) => Ok(n.to_string()),
    Value::String(s) => Ok(s.clone()),
    Value::Array(_) | Value::Object(_) => {
      serde_json::to_string(value).map_err(|e| InterpolationError::JsonError(e.to_string()))
    }
  }
}

/// Resolve a path to a string value from the context
///
/// This is a convenience function that combines path splitting and navigation.
///
/// # Errors
///
/// Returns `InterpolationError` if the path cannot be resolved.
///
/// # Example
///
/// ```
/// # use clarity_web::intent::validation::interpolation::{resolve_path, Context};
/// # use serde_json::json;
/// let context = Context::new()
///     .with_request_body(json!({
///         "user": {
///             "name": "Alice",
///             "emails": ["alice@work.com", "alice@home.com"]
///         }
///     }));
///
/// let name = resolve_path("request.user.name", &context)?;
/// assert_eq!(name, "Alice");
///
/// let email = resolve_path("request.user.emails[0]", &context)?;
/// assert_eq!(email, "alice@work.com");
/// # Ok::<(), clarity_web::intent::validation::interpolation::InterpolationError>(())
/// ```
pub fn resolve_path(path: &str, context: &Context) -> Result<String, InterpolationError> {
  let trimmed = path.trim();

  if trimmed.is_empty() {
    return Err(InterpolationError::InvalidPath("empty path".into()));
  }

  // Handle special prefixes
  if let Some(rest) = trimmed.strip_prefix("request.") {
    return resolve_from_body(rest, context.request_body.as_ref(), "request");
  }

  if let Some(rest) = trimmed.strip_prefix("response.") {
    return resolve_from_body(rest, context.response_body.as_ref(), "response");
  }

  // Check simple variables first
  if let Some(value) = context.variables.get(trimmed) {
    return Ok(value.clone());
  }

  // Try request body
  if let Some(ref body) = context.request_body {
    let path_parts = split_path(trimmed);
    if let Ok(value) = navigate_path(body, &path_parts) {
      return value_to_string(&value);
    }
  }

  // Try response body
  if let Some(ref body) = context.response_body {
    let path_parts = split_path(trimmed);
    if let Ok(value) = navigate_path(body, &path_parts) {
      return value_to_string(&value);
    }
  }

  Err(InterpolationError::VariableNotFound(trimmed.to_string()))
}

/// Check if a string contains any interpolation placeholders
///
/// # Example
///
/// ```
/// # use clarity_web::intent::validation::interpolation::has_placeholders;
/// assert!(has_placeholders("Hello, ${name}!"));
/// assert!(!has_placeholders("Hello, World!"));
/// ```
#[must_use]
pub fn has_placeholders(input: &str) -> bool {
  input.contains("${")
}

/// Extract all variable names from a string
///
/// Returns a list of variable names found in `${...}` placeholders.
///
/// # Example
///
/// ```
/// # use clarity_web::intent::validation::interpolation::extract_variables;
/// let vars = extract_variables("Hello, ${name}! Your email is ${user.email}");
/// assert_eq!(vars, vec!["name", "user.email"]);
/// ```
#[must_use]
pub fn extract_variables(input: &str) -> Vec<String> {
  find_placeholders(input)
    .into_iter()
    .map(|(_, _, var_name)| var_name)
    .collect()
}

/// Validate that all variables in a string can be resolved
///
/// Returns a list of variables that cannot be resolved.
///
/// # Example
///
/// ```
/// # use clarity_web::intent::validation::interpolation::{validate_variables, Context};
/// let context = Context::from_variables([("name", "Alice")]);
///
/// let missing = validate_variables("Hello, ${name}! Age: ${age}", &context);
/// assert_eq!(missing, vec!["age"]);
/// ```
#[must_use]
pub fn validate_variables(input: &str, context: &Context) -> Vec<String> {
  extract_variables(input)
    .into_iter()
    .filter(|var| resolve_variable(var, context).is_err())
    .collect()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::float_cmp, clippy::needless_collect, clippy::unnecessary_debug_formatting, clippy::match_same_arms, clippy::option_if_let_else, clippy::suspicious_else_formatting, clippy::manual_let_else, clippy::match_wild_err_arm, clippy::match_like_matches_macro, clippy::needless_pass_by_value)]
mod tests {
  use super::*;
  use serde_json::json;

  // -------------------------------------------------------------------------
  // Context Tests
  // -------------------------------------------------------------------------

  #[test]
  fn test_context_new() {
    let ctx = Context::new();
    assert!(ctx.variables.is_empty());
    assert!(ctx.request_body.is_none());
    assert!(ctx.response_body.is_none());
  }

  #[test]
  fn test_context_from_variables() {
    let ctx = Context::from_variables([("name", "Alice"), ("age", "30")]);

    assert_eq!(ctx.variables.get("name"), Some(&"Alice".to_string()));
    assert_eq!(ctx.variables.get("age"), Some(&"30".to_string()));
  }

  #[test]
  fn test_context_builder() {
    let ctx = Context::new()
      .with_variable("name", "Bob")
      .with_request_body(json!({"id": 1}))
      .with_response_body(json!({"status": "ok"}));

    assert_eq!(ctx.variables.get("name"), Some(&"Bob".to_string()));
    assert!(ctx.request_body.is_some());
    assert!(ctx.response_body.is_some());
  }

  // -------------------------------------------------------------------------
  // find_placeholders Tests
  // -------------------------------------------------------------------------

  #[test]
  fn test_find_placeholders_empty() {
    assert!(find_placeholders("").is_empty());
    assert!(find_placeholders("no placeholders").is_empty());
  }

  #[test]
  fn test_find_placeholders_simple() {
    let placeholders = find_placeholders("Hello, ${name}!");
    assert_eq!(placeholders.len(), 1);
    assert_eq!(placeholders[0].2, "name");
  }

  #[test]
  fn test_find_placeholders_multiple() {
    let placeholders = find_placeholders("${a} and ${b} and ${c}");
    assert_eq!(placeholders.len(), 3);
    assert_eq!(placeholders[0].2, "a");
    assert_eq!(placeholders[1].2, "b");
    assert_eq!(placeholders[2].2, "c");
  }

  #[test]
  fn test_find_placeholders_nested_path() {
    let placeholders = find_placeholders("${user.profile.name}");
    assert_eq!(placeholders.len(), 1);
    assert_eq!(placeholders[0].2, "user.profile.name");
  }

  #[test]
  fn test_find_placeholders_with_array() {
    let placeholders = find_placeholders("${items[0]} and ${items[-1]}");
    assert_eq!(placeholders.len(), 2);
    assert_eq!(placeholders[0].2, "items[0]");
    assert_eq!(placeholders[1].2, "items[-1]");
  }

  #[test]
  fn test_find_placeholders_unclosed() {
    // Unclosed braces should not be captured
    let placeholders = find_placeholders("Hello, ${name");
    assert!(placeholders.is_empty());
  }

  // -------------------------------------------------------------------------
  // interpolate_string Tests
  // -------------------------------------------------------------------------

  #[test]
  fn test_interpolate_string_no_placeholders() -> Result<(), InterpolationError> {
    let ctx = Context::new();
    let result = interpolate_string("Hello, World!", &ctx)?;
    assert_eq!(result, "Hello, World!");
    Ok(())
  }

  #[test]
  fn test_interpolate_string_simple() -> Result<(), InterpolationError> {
    let ctx = Context::from_variables([("name", "Alice")]);
    let result = interpolate_string("Hello, ${name}!", &ctx)?;
    assert_eq!(result, "Hello, Alice!");
    Ok(())
  }

  #[test]
  fn test_interpolate_string_multiple() -> Result<(), InterpolationError> {
    let ctx = Context::from_variables([("first", "Alice"), ("last", "Smith")]);
    let result = interpolate_string("${first} ${last}", &ctx)?;
    assert_eq!(result, "Alice Smith");
    Ok(())
  }

  #[test]
  fn test_interpolate_string_missing_variable() {
    let ctx = Context::new();
    let result = interpolate_string("Hello, ${name}!", &ctx);
    assert!(matches!(
      result,
      Err(InterpolationError::VariableNotFound(_))
    ));
  }

  #[test]
  fn test_interpolate_string_from_request_body() -> Result<(), InterpolationError> {
    let ctx = Context::new().with_request_body(json!({
        "user": {
            "name": "Bob"
        }
    }));

    let result = interpolate_string("Hello, ${request.user.name}!", &ctx)?;
    assert_eq!(result, "Hello, Bob!");
    Ok(())
  }

  #[test]
  fn test_interpolate_string_from_response_body() -> Result<(), InterpolationError> {
    let ctx = Context::new().with_response_body(json!({
        "status": "success",
        "data": {
            "id": 42
        }
    }));

    let result = interpolate_string("Status: ${response.status}, ID: ${response.data.id}", &ctx)?;
    assert_eq!(result, "Status: success, ID: 42");
    Ok(())
  }

  #[test]
  fn test_interpolate_string_with_array() -> Result<(), InterpolationError> {
    let ctx = Context::new().with_request_body(json!({
        "items": ["apple", "banana", "cherry"]
    }));

    let result = interpolate_string(
      "First: ${request.items[0]}, Last: ${request.items[-1]}",
      &ctx,
    )?;
    assert_eq!(result, "First: apple, Last: cherry");
    Ok(())
  }

  #[test]
  fn test_interpolate_string_mixed_sources() -> Result<(), InterpolationError> {
    let ctx = Context::new()
      .with_variable("greeting", "Hello")
      .with_request_body(json!({"name": "World"}));

    let result = interpolate_string("${greeting}, ${request.name}!", &ctx)?;
    assert_eq!(result, "Hello, World!");
    Ok(())
  }

  #[test]
  fn test_interpolate_string_empty_variable() -> Result<(), InterpolationError> {
    let ctx = Context::from_variables([("name", "")]);
    let result = interpolate_string("Hello, ${name}!", &ctx)?;
    assert_eq!(result, "Hello, !");
    Ok(())
  }

  // -------------------------------------------------------------------------
  // resolve_path Tests
  // -------------------------------------------------------------------------

  #[test]
  fn test_resolve_path_simple_variable() -> Result<(), InterpolationError> {
    let ctx = Context::from_variables([("name", "Alice")]);
    let result = resolve_path("name", &ctx)?;
    assert_eq!(result, "Alice");
    Ok(())
  }

  #[test]
  fn test_resolve_path_request_body() -> Result<(), InterpolationError> {
    let ctx = Context::new().with_request_body(json!({
        "user": {"email": "test@example.com"}
    }));

    let result = resolve_path("request.user.email", &ctx)?;
    assert_eq!(result, "test@example.com");
    Ok(())
  }

  #[test]
  fn test_resolve_path_response_body() -> Result<(), InterpolationError> {
    let ctx = Context::new().with_response_body(json!({
        "code": 200
    }));

    let result = resolve_path("response.code", &ctx)?;
    assert_eq!(result, "200");
    Ok(())
  }

  #[test]
  fn test_resolve_path_array_index() -> Result<(), InterpolationError> {
    let ctx = Context::new().with_request_body(json!({
        "items": [1, 2, 3]
    }));

    let result = resolve_path("request.items[0]", &ctx)?;
    assert_eq!(result, "1");

    let result = resolve_path("request.items[-1]", &ctx)?;
    assert_eq!(result, "3");
    Ok(())
  }

  #[test]
  fn test_resolve_path_not_found() {
    let ctx = Context::new();
    let result = resolve_path("missing", &ctx);
    assert!(matches!(
      result,
      Err(InterpolationError::VariableNotFound(_))
    ));
  }

  #[test]
  fn test_resolve_path_empty() {
    let ctx = Context::new();
    let result = resolve_path("", &ctx);
    assert!(matches!(result, Err(InterpolationError::InvalidPath(_))));
  }

  // -------------------------------------------------------------------------
  // Helper Function Tests
  // -------------------------------------------------------------------------

  #[test]
  fn test_has_placeholders() {
    assert!(has_placeholders("${var}"));
    assert!(has_placeholders("text ${var} text"));
    assert!(!has_placeholders("no placeholders"));
    assert!(!has_placeholders("$not a placeholder"));
  }

  #[test]
  fn test_extract_variables() {
    let vars = extract_variables("${a} ${b} ${c}");
    assert_eq!(vars, vec!["a", "b", "c"]);
  }

  #[test]
  fn test_extract_variables_dedup_not_applied() {
    // extract_variables does not deduplicate
    let vars = extract_variables("${a} ${a} ${b}");
    assert_eq!(vars, vec!["a", "a", "b"]);
  }

  #[test]
  fn test_validate_variables_all_present() {
    let ctx = Context::from_variables([("name", "Alice"), ("age", "30")]);
    let missing = validate_variables("${name} is ${age}", &ctx);
    assert!(missing.is_empty());
  }

  #[test]
  fn test_validate_variables_some_missing() {
    let ctx = Context::from_variables([("name", "Alice")]);
    let missing = validate_variables("${name} is ${age} from ${city}", &ctx);
    assert_eq!(missing, vec!["age", "city"]);
  }

  // -------------------------------------------------------------------------
  // Error Display Tests
  // -------------------------------------------------------------------------

  #[test]
  fn test_error_display() {
    let err = InterpolationError::VariableNotFound("foo".into());
    assert!(format!("{err}").contains("foo"));

    let err = InterpolationError::InvalidPath("bad.path".into());
    assert!(format!("{err}").contains("bad.path"));

    let err = InterpolationError::IndexOutOfBounds {
      index: 5,
      length: 3,
    };
    let msg = format!("{err}");
    assert!(msg.contains('5'));
    assert!(msg.contains('3'));
  }

  // -------------------------------------------------------------------------
  // Edge Cases
  // -------------------------------------------------------------------------

  #[test]
  fn test_interpolate_adjacent_placeholders() -> Result<(), InterpolationError> {
    let ctx = Context::from_variables([("a", "X"), ("b", "Y")]);
    let result = interpolate_string("${a}${b}", &ctx)?;
    assert_eq!(result, "XY");
    Ok(())
  }

  #[test]
  fn test_interpolate_whitespace_in_placeholder() -> Result<(), InterpolationError> {
    let ctx = Context::from_variables([("name", "Alice")]);
    let result = interpolate_string("Hello, ${ name }!", &ctx)?;
    assert_eq!(result, "Hello, Alice!");
    Ok(())
  }

  #[test]
  fn test_interpolate_json_value() -> Result<(), InterpolationError> {
    let ctx = Context::new().with_request_body(json!({
        "data": {"nested": [1, 2, 3]}
    }));

    let result = interpolate_string("Data: ${request.data}", &ctx)?;
    assert!(result.contains("nested"));
    Ok(())
  }

  #[test]
  fn test_interpolate_null_value() -> Result<(), InterpolationError> {
    let ctx = Context::new().with_request_body(json!({
        "value": null
    }));

    let result = interpolate_string("Value: ${request.value}", &ctx)?;
    assert_eq!(result, "Value: ");
    Ok(())
  }

  #[test]
  fn test_interpolate_boolean_value() -> Result<(), InterpolationError> {
    let ctx = Context::new().with_request_body(json!({
        "active": true,
        "deleted": false
    }));

    let result = interpolate_string("Active: ${request.active}", &ctx)?;
    assert_eq!(result, "Active: true");

    let result = interpolate_string("Deleted: ${request.deleted}", &ctx)?;
    assert_eq!(result, "Deleted: false");
    Ok(())
  }

  #[test]
  fn test_interpolate_number_value() -> Result<(), InterpolationError> {
    let ctx = Context::new().with_request_body(json!({
        "count": 42,
        "price": 19.99
    }));

    let result = interpolate_string("Count: ${request.count}", &ctx)?;
    assert_eq!(result, "Count: 42");

    let result = interpolate_string("Price: ${request.price}", &ctx)?;
    assert_eq!(result, "Price: 19.99");
    Ok(())
  }

  #[test]
  fn test_interpolate_wildcard_returns_array() -> Result<(), InterpolationError> {
    let ctx = Context::new().with_request_body(json!({
        "items": [1, 2, 3]
    }));

    let result = interpolate_string("All: ${request.items[*]}", &ctx)?;
    assert!(result.contains('['));
    Ok(())
  }
}
