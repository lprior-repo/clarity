use super::context::Context;
use super::errors::InterpolationError;
use super::placeholders::{extract_variables, find_placeholders};
use super::resolve::resolve_variable;

/// Interpolates `${...}` placeholders in a string using the provided context.
///
/// # Errors
/// Returns `InterpolationError` when any referenced variable/path cannot be resolved.
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
