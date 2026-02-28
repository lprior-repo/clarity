use super::{parse_path_component, ArrayIndexError, ArraySpec};
use serde_json::Value;

/// Navigates a dotted path with optional array access against a JSON value.
///
/// # Errors
/// Returns `ArrayIndexError` when path components are invalid or navigation fails.
pub fn navigate_path(value: &Value, path: &[String]) -> Result<Value, ArrayIndexError> {
  if path.is_empty() {
    return Ok(value.clone());
  }

  path.iter().try_fold(value.clone(), |current, component| {
    let (field, spec) = parse_path_component(component)?;
    let field_value = current
      .get(&field)
      .ok_or_else(|| ArrayIndexError::FieldNotFound(field.clone()))?;

    match spec {
      ArraySpec::NoArray => Ok(field_value.clone()),
      ArraySpec::Index(_) | ArraySpec::NegativeIndex(_) | ArraySpec::All => {
        navigate_array(field_value, spec, &field)
      }
    }
  })
}

fn navigate_array(value: &Value, spec: ArraySpec, field: &str) -> Result<Value, ArrayIndexError> {
  let array = value
    .as_array()
    .ok_or_else(|| ArrayIndexError::NotAnArray {
      field: field.to_string(),
      actual_type: value_type_name(value),
    })?;

  let length = array.len();
  if length == 0 {
    return match spec {
      ArraySpec::Index(i) => Err(ArrayIndexError::IndexOutOfBounds {
        index: isize::try_from(i).unwrap_or(isize::MAX),
        length: 0,
      }),
      ArraySpec::NegativeIndex(_) | ArraySpec::All => Ok(Value::Array(Vec::new())),
      ArraySpec::NoArray => Err(ArrayIndexError::InvalidPath(
        "no-array spec in array navigation".into(),
      )),
    };
  }

  let indices = spec.resolve_indices(length)?;
  match indices.len() {
    0 => Ok(Value::Array(Vec::new())),
    1 => Ok(
      indices
        .first()
        .and_then(|index| array.get(*index).cloned())
        .map_or(Value::Null, |value| value),
    ),
    _ => Ok(Value::Array(
      indices
        .into_iter()
        .filter_map(|i| array.get(i).cloned())
        .collect(),
    )),
  }
}

fn value_type_name(value: &Value) -> String {
  match value {
    Value::Null => "null".to_string(),
    Value::Bool(_) => "boolean".to_string(),
    Value::Number(_) => "number".to_string(),
    Value::String(_) => "string".to_string(),
    Value::Array(_) => "array".to_string(),
    Value::Object(_) => "object".to_string(),
  }
}
