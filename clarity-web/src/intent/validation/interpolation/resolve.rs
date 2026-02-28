use super::context::Context;
use super::errors::InterpolationError;
use crate::intent::util::array_indexing::{
  navigate_path, parse_path_component, split_path, ArraySpec,
};
use serde_json::Value;

/// Resolves a path expression against interpolation context.
///
/// # Errors
/// Returns `InterpolationError` for invalid paths or missing variables.
pub fn resolve_path(path: &str, context: &Context) -> Result<String, InterpolationError> {
  let trimmed = path.trim();
  if trimmed.is_empty() {
    return Err(InterpolationError::InvalidPath("empty path".into()));
  }

  resolve_variable(trimmed, context)
}

pub fn resolve_variable(var_name: &str, context: &Context) -> Result<String, InterpolationError> {
  if let Some(rest) = var_name.strip_prefix("request.") {
    return resolve_from_body(rest, context.request_body.as_ref(), "request");
  }
  if let Some(rest) = var_name.strip_prefix("response.") {
    return resolve_from_body(rest, context.response_body.as_ref(), "response");
  }

  // Validate path doesn't contain consecutive dots (empty components)
  if var_name.contains("..") {
    return Err(InterpolationError::InvalidPath("empty component".into()));
  }

  // Split the path and parse the first component to get the base variable name
  let path_parts = split_path(var_name);
  if path_parts.is_empty() {
    return Err(InterpolationError::InvalidPath("empty path".into()));
  }

  // Parse the first component to extract base variable name and any array spec
  let (base_var, first_spec) =
    parse_path_component(&path_parts[0]).map_err(InterpolationError::from)?;

  // Look up the base variable in context
  if let Some(base_value) = context.variables.get(&base_var) {
    // Start with the base value, apply first spec if it's an array access
    let initial_value = match first_spec {
      ArraySpec::NoArray => base_value.clone(),
      spec @ (ArraySpec::Index(_) | ArraySpec::NegativeIndex(_) | ArraySpec::All) => {
        // Navigate array on the base value
        navigate_with_spec(base_value, spec, &base_var)?
      }
    };

    // If there are more path components, navigate them
    if path_parts.len() > 1 {
      let navigated =
        navigate_path(&initial_value, &path_parts[1..]).map_err(InterpolationError::from)?;
      return value_to_string(&navigated);
    }
    return value_to_string(&initial_value);
  }

  // Fall back to request/response body lookup
  if let Some(body) = context.request_body.as_ref() {
    if let Ok(value) = resolve_from_body(var_name, Some(body), "request") {
      return Ok(value);
    }
  }
  if let Some(body) = context.response_body.as_ref() {
    if let Ok(value) = resolve_from_body(var_name, Some(body), "response") {
      return Ok(value);
    }
  }

  Err(InterpolationError::VariableNotFound(var_name.to_string()))
}

fn navigate_with_spec(
  value: &Value,
  spec: ArraySpec,
  field: &str,
) -> Result<Value, InterpolationError> {
  let array = value
    .as_array()
    .ok_or_else(|| InterpolationError::NotAnArray(field.to_string()))?;

  let length = array.len();
  if length == 0 {
    return match spec {
      ArraySpec::Index(i) => Err(InterpolationError::IndexOutOfBounds {
        index: i,
        length: 0,
      }),
      ArraySpec::NegativeIndex(_) | ArraySpec::All => Ok(Value::Array(Vec::new())),
      ArraySpec::NoArray => Err(InterpolationError::InvalidPath(
        "no-array spec in array navigation".into(),
      )),
    };
  }

  let indices = spec
    .resolve_indices(length)
    .map_err(InterpolationError::from)?;
  match indices.len() {
    0 => Ok(Value::Array(Vec::new())),
    1 => Ok(
      indices
        .first()
        .and_then(|index| array.get(*index).cloned())
        .map_or(Value::Null, |v| v),
    ),
    _ => Ok(Value::Array(
      indices
        .into_iter()
        .filter_map(|i| array.get(i).cloned())
        .collect(),
    )),
  }
}

fn resolve_from_body(
  path: &str,
  body: Option<&Value>,
  body_name: &str,
) -> Result<String, InterpolationError> {
  let body =
    body.ok_or_else(|| InterpolationError::VariableNotFound(format!("{body_name}.body")))?;

  let actual_path = path.strip_prefix("body.").map_or(path, |stripped| stripped);
  if actual_path.is_empty() || actual_path == "body" {
    return serde_json::to_string(body).map_err(|e| InterpolationError::JsonError(e.to_string()));
  }

  let path_parts = split_path(actual_path);
  let value = navigate_path(body, &path_parts).map_err(InterpolationError::from)?;
  value_to_string(&value)
}

pub fn value_to_string(value: &Value) -> Result<String, InterpolationError> {
  match value {
    Value::Null => Ok(String::new()),
    Value::Bool(v) => Ok(v.to_string()),
    Value::Number(v) => Ok(v.to_string()),
    Value::String(v) => Ok(v.clone()),
    Value::Array(_) | Value::Object(_) => {
      serde_json::to_string(value).map_err(|e| InterpolationError::JsonError(e.to_string()))
    }
  }
}
