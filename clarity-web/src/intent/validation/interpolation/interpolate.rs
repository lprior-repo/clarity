use std::collections::HashMap;

use super::context::Context;
use super::errors::InterpolationError;
use super::placeholders::{extract_variables, find_placeholders};
use super::resolve::{resolve_variable, value_to_string};
use serde_json::Value;

pub fn interpolate_string(input: &str, context: &Context) -> Result<String, InterpolationError> {
  let placeholders = find_placeholders(input);
  if placeholders.is_empty() {
    return Ok(input.to_string());
  }

  let mut result = String::new();
  let mut last_end = 0;

  for (start, end, var_name) in placeholders {
    result.push_str(&input[last_end..start]);
    let resolved = resolve_variable(&var_name, context)?;
    result.push_str(&resolved);
    last_end = end;
  }

  if last_end < input.len() {
    result.push_str(&input[last_end..]);
  }

  Ok(result)
}

#[must_use]
pub fn validate_variables(input: &str, context: &Context) -> Vec<String> {
  extract_variables(input)
    .into_iter()
    .filter(|name| resolve_variable(name, context).is_err())
    .collect()
}

/// Interpolate variables in a headers map.
/// Replaces ${var_name} placeholders with resolved values.
pub fn interpolate_headers(
  headers: &HashMap<String, String>,
  context: &Context,
) -> Result<HashMap<String, String>, InterpolationError> {
  headers
    .iter()
    .map(|(key, value)| {
      interpolate_string(value, context).map(|interpolated| (key.clone(), interpolated))
    })
    .collect()
}

/// Extract a value from JSON using a capture path like "response.body.id".
/// Returns the resolved value as a JSON Value.
pub fn extract_capture(
  capture_path: &str,
  context: &Context,
) -> Result<Value, InterpolationError> {
  // Use resolve_variable but get the raw Value instead of stringifying
  extract_capture_value(capture_path, context)
}

fn extract_capture_value(path: &str, context: &Context) -> Result<Value, InterpolationError> {
  let trimmed = path.trim();
  if trimmed.is_empty() {
    return Err(InterpolationError::InvalidPath("empty path".into()));
  }

  // Handle request.body and response.body paths
  if let Some(rest) = trimmed.strip_prefix("request.") {
    return extract_from_body(rest, context.request_body.as_ref(), "request");
  }
  if let Some(rest) = trimmed.strip_prefix("response.") {
    return extract_from_body(rest, context.response_body.as_ref(), "response");
  }

  // Use the resolve module's logic for variable lookup
  let string_result = resolve_variable(trimmed, context)?;
  // Parse the string back to JSON if it's a complex type
  if string_result.starts_with('{') || string_result.starts_with('[') {
    serde_json::from_str(&string_result).map_err(|e| InterpolationError::JsonError(e.to_string()))
  } else {
    // Return as string value
    Ok(Value::String(string_result))
  }
}

fn extract_from_body(
  path: &str,
  body: Option<&Value>,
  body_name: &str,
) -> Result<Value, InterpolationError> {
  let body =
    body.ok_or_else(|| InterpolationError::VariableNotFound(format!("{body_name}.body")))?;

  let actual_path = match path.strip_prefix("body.") {
    Some(stripped) => stripped,
    None => path,
  };

  if actual_path.is_empty() || actual_path == "body" {
    return Ok(body.clone());
  }

  // Navigate the path
  let path_parts = crate::intent::util::array_indexing::split_path(actual_path);
  let value = crate::intent::util::array_indexing::navigate_path(body, &path_parts)
    .map_err(InterpolationError::from)?;
  Ok(value)
}

/// Convert a JSON value to a string representation.
/// For strings, returns the raw value (not JSON-encoded).
/// For other types, returns the JSON representation.
#[must_use]
pub fn json_to_string(value: &Value) -> String {
  value_to_string(value).unwrap_or_default()
}
