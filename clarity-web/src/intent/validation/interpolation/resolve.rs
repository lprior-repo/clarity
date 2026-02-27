use super::context::Context;
use super::errors::InterpolationError;
use crate::intent::util::array_indexing::{navigate_path, split_path};
use serde_json::Value;

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

  if let Some(value) = context.variables.get(var_name) {
    return Ok(value.clone());
  }

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

fn resolve_from_body(
  path: &str,
  body: Option<&Value>,
  body_name: &str,
) -> Result<String, InterpolationError> {
  let body =
    body.ok_or_else(|| InterpolationError::VariableNotFound(format!("{body_name}.body")))?;

  let actual_path = match path.strip_prefix("body.") {
    Some(stripped) => stripped,
    None => path,
  };
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
