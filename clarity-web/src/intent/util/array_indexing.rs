#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! JSON Array Navigation (WP08)
//!
//! Provides utilities for navigating JSON structures with array indexing support.
//! Supports positive indices, negative indices (from end), and wildcards.
//!
//! ## Key Types
//!
//! - [`ArraySpec`]: Specifies array navigation behavior
//! - [`ArrayIndexError`]: Error taxonomy for array operations
//!
//! ## Array Indexing Syntax
//!
//! - `[0]` - Positive index (zero-based)
//! - `[-1]` - Negative index (from end, -1 is last element)
//! - `[*]` - Wildcard (all elements)
//!
//! ## Example
//!
//! ```ignore
//! use intent::util::array_indexing::{split_path, parse_path_component, navigate_path, ArraySpec};
//!
//! let path_parts = split_path("users[0].name");
//! // Returns: ["users[0]", "name"]
//!
//! let (field, spec) = parse_path_component("users[0]")?;
//! // Returns: ("users", ArraySpec::Index(0))
//!
//! let value = navigate_path(&json, &path_parts)?;
//! ```

use serde_json::Value;
use std::fmt;
use thiserror::Error;

// =============================================================================
// Error Types
// =============================================================================

/// Error taxonomy for array indexing operations
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ArrayIndexError {
  /// Invalid path syntax
  #[error("invalid path: {0}")]
  InvalidPath(String),

  /// Array index out of bounds
  #[error("index {index} out of bounds for array of length {length}")]
  IndexOutOfBounds {
    /// The requested index
    index: isize,
    /// The actual array length
    length: usize,
  },

  /// Tried to index a non-array value
  #[error("not an array: attempted to index '{field}' which is a {actual_type}")]
  NotAnArray {
    /// Field that was accessed
    field: String,
    /// Actual type of the value
    actual_type: String,
  },

  /// Field not found in object
  #[error("field not found: '{0}'")]
  FieldNotFound(String),
}

// =============================================================================
// Array Specification
// =============================================================================

/// Specifies how to navigate an array field
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArraySpec {
  /// Not an array access - regular field
  NoArray,
  /// Access element at specific positive index (zero-based)
  Index(usize),
  /// Access element at negative index (from end: -1 = last, -2 = second-to-last)
  NegativeIndex(usize),
  /// Access all elements (wildcard)
  All,
}

impl ArraySpec {
  /// Returns true if this represents any kind of array access
  #[must_use]
  pub const fn is_array_access(self) -> bool {
    match self {
      Self::NoArray => false,
      Self::Index(_) | Self::NegativeIndex(_) | Self::All => true,
    }
  }

  /// Resolve the specification against an array length, returning the concrete indices
  ///
  /// # Errors
  ///
  /// Returns `ArrayIndexError::IndexOutOfBounds` if the index is invalid for the array
  pub fn resolve_indices(self, length: usize) -> Result<Vec<usize>, ArrayIndexError> {
    match self {
      Self::NoArray => Ok(vec![]),
      Self::Index(i) => {
        if i < length {
          Ok(vec![i])
        } else {
          Err(ArrayIndexError::IndexOutOfBounds {
            index: i.cast_signed(),
            length,
          })
        }
      }
      Self::NegativeIndex(n) => {
        // -1 maps to last element, -2 to second-to-last, etc.
        if n == 0 || n > length {
          Err(ArrayIndexError::IndexOutOfBounds {
            index: -n.cast_signed(),
            length,
          })
        } else {
          Ok(vec![length - n])
        }
      }
      Self::All => Ok((0..length).collect()),
    }
  }
}

impl fmt::Display for ArraySpec {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::NoArray => write!(f, ""),
      Self::Index(i) => write!(f, "[{i}]"),
      Self::NegativeIndex(n) => write!(f, "[-{n}]"),
      Self::All => write!(f, "[*]"),
    }
  }
}

// =============================================================================
// Path Parsing Functions
// =============================================================================

/// Split a path by dots, preserving array syntax
///
/// Handles paths like:
/// - `field` -> `["field"]`
/// - `field.nested` -> `["field", "nested"]`
/// - `field[0].nested` -> `["field[0]", "nested"]`
/// - `field[*].items[-1]` -> `["field[*]", "items[-1]"]`
///
/// # Example
///
/// ```
/// # use clarity_web::intent::util::array_indexing::split_path;
/// let parts = split_path("users[0].name");
/// assert_eq!(parts, vec!["users[0]", "name"]);
///
/// let parts = split_path("data.items[*].id");
/// assert_eq!(parts, vec!["data", "items[*]", "id"]);
/// ```
#[must_use]
pub fn split_path(path: &str) -> Vec<String> {
  let trimmed = path.trim();
  if trimmed.is_empty() {
    return vec![];
  }

  let mut result = Vec::new();
  let mut current = String::new();
  let mut in_brackets = false;

  for ch in trimmed.chars() {
    match ch {
      '[' => {
        in_brackets = true;
        current.push(ch);
      }
      ']' => {
        in_brackets = false;
        current.push(ch);
      }
      '.' if !in_brackets => {
        if !current.is_empty() {
          result.push(current.clone());
          current.clear();
        }
      }
      _ => {
        current.push(ch);
      }
    }
  }

  if !current.is_empty() {
    result.push(current);
  }

  result
}

/// Parse a path component into field name and array specification
///
/// Supports formats:
/// - `field` -> `("field", ArraySpec::NoArray)`
/// - `field[0]` -> `("field", ArraySpec::Index(0))`
/// - `field[-1]` -> `("field", ArraySpec::NegativeIndex(1))`
/// - `field[-3]` -> `("field", ArraySpec::NegativeIndex(3))`
/// - `field[*]` -> `("field", ArraySpec::All)`
///
/// # Errors
///
/// Returns `ArrayIndexError::InvalidPath` if:
/// - The component is empty
/// - The bracket syntax is malformed
/// - The index is not a valid number or wildcard
///
/// # Example
///
/// ```
/// # use clarity_web::intent::util::array_indexing::{parse_path_component, ArraySpec};
/// let (field, spec) = parse_path_component("users[0]").unwrap();
/// assert_eq!(field, "users");
/// assert_eq!(spec, ArraySpec::Index(0));
///
/// let (field, spec) = parse_path_component("items[-1]").unwrap();
/// assert_eq!(field, "items");
/// assert_eq!(spec, ArraySpec::NegativeIndex(1));
///
/// let (field, spec) = parse_path_component("data[*]").unwrap();
/// assert_eq!(field, "data");
/// assert_eq!(spec, ArraySpec::All);
/// ```
pub fn parse_path_component(component: &str) -> Result<(String, ArraySpec), ArrayIndexError> {
  let trimmed = component.trim();

  if trimmed.is_empty() {
    return Err(ArrayIndexError::InvalidPath("empty component".into()));
  }

  // Find the opening bracket
  match trimmed.find('[') {
    None => {
      // No brackets - simple field name
      if trimmed.contains(']') {
        return Err(ArrayIndexError::InvalidPath(format!(
          "unmatched closing bracket in: {component}"
        )));
      }
      Ok((trimmed.to_string(), ArraySpec::NoArray))
    }
    Some(open_pos) => {
      // Must have closing bracket
      let close_pos = trimmed
        .find(']')
        .ok_or_else(|| ArrayIndexError::InvalidPath(format!("unclosed bracket in: {component}")))?;

      if close_pos <= open_pos {
        return Err(ArrayIndexError::InvalidPath(format!(
          "invalid bracket order in: {component}"
        )));
      }

      // Check for trailing content after closing bracket
      if close_pos != trimmed.len() - 1 {
        return Err(ArrayIndexError::InvalidPath(format!(
          "trailing content after bracket in: {component}"
        )));
      }

      let field_name = trimmed[..open_pos].to_string();
      let index_content = &trimmed[open_pos + 1..close_pos];

      // Validate field name
      if field_name.is_empty() {
        return Err(ArrayIndexError::InvalidPath(format!(
          "missing field name in: {component}"
        )));
      }

      // Parse the index specification
      let spec = parse_index_spec(index_content, component)?;

      Ok((field_name, spec))
    }
  }
}

/// Parse the content inside brackets to determine the array specification
fn parse_index_spec(content: &str, original: &str) -> Result<ArraySpec, ArrayIndexError> {
  let trimmed = content.trim();

  if trimmed.is_empty() {
    return Err(ArrayIndexError::InvalidPath(format!(
      "empty brackets in: {original}"
    )));
  }

  // Wildcard: [*]
  if trimmed == "*" {
    return Ok(ArraySpec::All);
  }

  // Negative index: [-N]
  if let Some(rest) = trimmed.strip_prefix('-') {
    let n: usize = rest.trim().parse().map_err(|_| {
      ArrayIndexError::InvalidPath(format!("invalid negative index in: {original}"))
    })?;
    if n == 0 {
      return Err(ArrayIndexError::InvalidPath(format!(
        "negative zero not allowed in: {original}"
      )));
    }
    return Ok(ArraySpec::NegativeIndex(n));
  }

  // Positive index: [N]
  let index: usize = trimmed
    .parse()
    .map_err(|_| ArrayIndexError::InvalidPath(format!("invalid index in: {original}")))?;

  Ok(ArraySpec::Index(index))
}

// =============================================================================
// Navigation Functions
// =============================================================================

/// Navigate a JSON value using a path with array indexing support
///
/// # Errors
///
/// Returns `ArrayIndexError` if:
/// - A field is not found in an object
/// - An array index is out of bounds
/// - Attempting to index a non-array value
/// - The path is invalid
///
/// # Example
///
/// ```
/// # use clarity_web::intent::util::array_indexing::navigate_path;
/// # use serde_json::json;
/// let data = json!({
///     "users": [
///         {"name": "Alice", "age": 30},
///         {"name": "Bob", "age": 25}
///     ]
/// });
///
/// let path = vec!["users[0]".to_string(), "name".to_string()];
/// let result = navigate_path(&data, &path).unwrap();
/// assert_eq!(result, json!("Alice"));
/// ```
pub fn navigate_path(value: &Value, path: &[String]) -> Result<Value, ArrayIndexError> {
  if path.is_empty() {
    return Ok(value.clone());
  }

  path.iter().try_fold(value.clone(), |current, component| {
    let (field, spec) = parse_path_component(component)?;

    // Get the field value from the current object
    let field_value = current
      .get(&field)
      .ok_or_else(|| ArrayIndexError::FieldNotFound(field.clone()))?;

    // Apply array specification if present
    match spec {
      ArraySpec::NoArray => Ok(field_value.clone()),
      ArraySpec::Index(_) | ArraySpec::NegativeIndex(_) | ArraySpec::All => {
        navigate_array(field_value, spec, &field)
      }
    }
  })
}

/// Navigate an array value with the given specification
fn navigate_array(value: &Value, spec: ArraySpec, field: &str) -> Result<Value, ArrayIndexError> {
  let array = value
    .as_array()
    .ok_or_else(|| ArrayIndexError::NotAnArray {
      field: field.to_string(),
      actual_type: value_type_name(value),
    })?;

  let length = array.len();

  // Handle empty arrays
  if length == 0 {
    return match spec {
      ArraySpec::Index(i) => Err(ArrayIndexError::IndexOutOfBounds {
        index: i.cast_signed(),
        length: 0,
      }),
      ArraySpec::NegativeIndex(_) | ArraySpec::All => Ok(Value::Array(vec![])),
      ArraySpec::NoArray => unreachable!("NoArray should not reach here"),
    };
  }

  let indices = spec.resolve_indices(length)?;

  match indices.len() {
    0 => Ok(Value::Array(vec![])),
    1 => Ok(array.get(indices[0]).cloned().map_or(Value::Null, |v| v)),
    _ => {
      let values: Vec<Value> = indices
        .into_iter()
        .filter_map(|i| array.get(i).cloned())
        .collect();
      Ok(Value::Array(values))
    }
  }
}

/// Get a human-readable type name for a JSON value
fn value_type_name(value: &Value) -> String {
  match value {
    Value::Null => "null".into(),
    Value::Bool(_) => "boolean".into(),
    Value::Number(_) => "number".into(),
    Value::String(_) => "string".into(),
    Value::Array(_) => "array".into(),
    Value::Object(_) => "object".into(),
  }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(
  clippy::unwrap_used,
  clippy::expect_used,
  clippy::panic,
  clippy::float_cmp,
  clippy::needless_collect,
  clippy::unnecessary_debug_formatting,
  clippy::match_same_arms,
  clippy::option_if_let_else,
  clippy::suspicious_else_formatting,
  clippy::manual_let_else,
  clippy::match_wild_err_arm,
  clippy::match_like_matches_macro,
  clippy::needless_pass_by_value
)]
mod tests {
  use super::*;
  use serde_json::json;

  // -------------------------------------------------------------------------
  // ArraySpec Tests
  // -------------------------------------------------------------------------

  #[test]
  fn test_array_spec_is_array_access() {
    assert!(!ArraySpec::NoArray.is_array_access());
    assert!(ArraySpec::Index(0).is_array_access());
    assert!(ArraySpec::NegativeIndex(1).is_array_access());
    assert!(ArraySpec::All.is_array_access());
  }

  #[test]
  fn test_array_spec_resolve_indices() {
    // Index
    assert_eq!(ArraySpec::Index(0).resolve_indices(3), Ok(vec![0]));
    assert_eq!(ArraySpec::Index(2).resolve_indices(3), Ok(vec![2]));
    assert_eq!(
      ArraySpec::Index(3).resolve_indices(3),
      Err(ArrayIndexError::IndexOutOfBounds {
        index: 3,
        length: 3
      })
    );

    // NegativeIndex
    assert_eq!(ArraySpec::NegativeIndex(1).resolve_indices(3), Ok(vec![2]));
    assert_eq!(ArraySpec::NegativeIndex(2).resolve_indices(3), Ok(vec![1]));
    assert_eq!(ArraySpec::NegativeIndex(3).resolve_indices(3), Ok(vec![0]));
    assert_eq!(
      ArraySpec::NegativeIndex(4).resolve_indices(3),
      Err(ArrayIndexError::IndexOutOfBounds {
        index: -4,
        length: 3
      })
    );
    assert_eq!(
      ArraySpec::NegativeIndex(0).resolve_indices(3),
      Err(ArrayIndexError::IndexOutOfBounds {
        index: 0,
        length: 3
      })
    );

    // All
    assert_eq!(ArraySpec::All.resolve_indices(3), Ok(vec![0, 1, 2]));
    assert_eq!(ArraySpec::All.resolve_indices(0), Ok(vec![]));

    // NoArray
    assert_eq!(ArraySpec::NoArray.resolve_indices(3), Ok(vec![]));
  }

  #[test]
  fn test_array_spec_display() {
    assert_eq!(ArraySpec::NoArray.to_string(), "");
    assert_eq!(ArraySpec::Index(0).to_string(), "[0]");
    assert_eq!(ArraySpec::Index(42).to_string(), "[42]");
    assert_eq!(ArraySpec::NegativeIndex(1).to_string(), "[-1]");
    assert_eq!(ArraySpec::NegativeIndex(3).to_string(), "[-3]");
    assert_eq!(ArraySpec::All.to_string(), "[*]");
  }

  // -------------------------------------------------------------------------
  // split_path Tests
  // -------------------------------------------------------------------------

  #[test]
  fn test_split_path_simple() {
    assert_eq!(split_path("field"), vec!["field"]);
    assert_eq!(split_path("field.nested"), vec!["field", "nested"]);
    assert_eq!(split_path("a.b.c.d"), vec!["a", "b", "c", "d"]);
  }

  #[test]
  fn test_split_path_empty() {
    assert_eq!(split_path(""), Vec::<String>::new());
    assert_eq!(split_path("   "), Vec::<String>::new());
  }

  #[test]
  fn test_split_path_with_array_index() {
    assert_eq!(split_path("field[0]"), vec!["field[0]"]);
    assert_eq!(split_path("field[0].nested"), vec!["field[0]", "nested"]);
    assert_eq!(
      split_path("data.items[*].id"),
      vec!["data", "items[*]", "id"]
    );
    assert_eq!(
      split_path("arr[0].items[-1].value"),
      vec!["arr[0]", "items[-1]", "value"]
    );
  }

  #[test]
  fn test_split_path_preserves_brackets() {
    assert_eq!(
      split_path("users[0].posts[*].comments[-1]"),
      vec!["users[0]", "posts[*]", "comments[-1]"]
    );
  }

  // -------------------------------------------------------------------------
  // parse_path_component Tests
  // -------------------------------------------------------------------------

  #[test]
  fn test_parse_simple_field() {
    assert_eq!(
      parse_path_component("field"),
      Ok(("field".into(), ArraySpec::NoArray))
    );
    assert_eq!(
      parse_path_component("nested_field"),
      Ok(("nested_field".into(), ArraySpec::NoArray))
    );
  }

  #[test]
  fn test_parse_positive_index() {
    assert_eq!(
      parse_path_component("field[0]"),
      Ok(("field".into(), ArraySpec::Index(0)))
    );
    assert_eq!(
      parse_path_component("items[42]"),
      Ok(("items".into(), ArraySpec::Index(42)))
    );
  }

  #[test]
  fn test_parse_negative_index() {
    assert_eq!(
      parse_path_component("field[-1]"),
      Ok(("field".into(), ArraySpec::NegativeIndex(1)))
    );
    assert_eq!(
      parse_path_component("items[-3]"),
      Ok(("items".into(), ArraySpec::NegativeIndex(3)))
    );
  }

  #[test]
  fn test_parse_wildcard() {
    assert_eq!(
      parse_path_component("field[*]"),
      Ok(("field".into(), ArraySpec::All))
    );
    assert_eq!(
      parse_path_component("items[*]"),
      Ok(("items".into(), ArraySpec::All))
    );
  }

  #[test]
  fn test_parse_errors() {
    // Empty
    assert!(matches!(
      parse_path_component(""),
      Err(ArrayIndexError::InvalidPath(_))
    ));

    // Missing field name
    assert!(matches!(
      parse_path_component("[0]"),
      Err(ArrayIndexError::InvalidPath(_))
    ));

    // Unclosed bracket
    assert!(matches!(
      parse_path_component("field[0"),
      Err(ArrayIndexError::InvalidPath(_))
    ));

    // Invalid index
    assert!(matches!(
      parse_path_component("field[abc]"),
      Err(ArrayIndexError::InvalidPath(_))
    ));

    // Trailing content
    assert!(matches!(
      parse_path_component("field[0]extra"),
      Err(ArrayIndexError::InvalidPath(_))
    ));

    // Unmatched closing bracket
    assert!(matches!(
      parse_path_component("field]"),
      Err(ArrayIndexError::InvalidPath(_))
    ));

    // Negative zero
    assert!(matches!(
      parse_path_component("field[-0]"),
      Err(ArrayIndexError::InvalidPath(_))
    ));
  }

  // -------------------------------------------------------------------------
  // navigate_path Tests
  // -------------------------------------------------------------------------

  #[test]
  fn test_navigate_simple_field() {
    let data = json!({"name": "Alice", "age": 30});

    let result = navigate_path(&data, &["name".to_string()]);
    assert_eq!(result, Ok(json!("Alice")));

    let result = navigate_path(&data, &["age".to_string()]);
    assert_eq!(result, Ok(json!(30)));
  }

  #[test]
  fn test_navigate_nested_field() {
    let data = json!({
        "user": {
            "profile": {
                "name": "Alice"
            }
        }
    });

    let result = navigate_path(
      &data,
      &[
        "user".to_string(),
        "profile".to_string(),
        "name".to_string(),
      ],
    );
    assert_eq!(result, Ok(json!("Alice")));
  }

  #[test]
  fn test_navigate_array_index() {
    let data = json!({
        "items": ["a", "b", "c"]
    });

    let result = navigate_path(&data, &["items[0]".to_string()]);
    assert_eq!(result, Ok(json!("a")));

    let result = navigate_path(&data, &["items[2]".to_string()]);
    assert_eq!(result, Ok(json!("c")));
  }

  #[test]
  fn test_navigate_array_negative_index() {
    let data = json!({
        "items": ["a", "b", "c"]
    });

    let result = navigate_path(&data, &["items[-1]".to_string()]);
    assert_eq!(result, Ok(json!("c")));

    let result = navigate_path(&data, &["items[-2]".to_string()]);
    assert_eq!(result, Ok(json!("b")));
  }

  #[test]
  fn test_navigate_array_wildcard() {
    let data = json!({
        "items": [{"id": 1}, {"id": 2}, {"id": 3}]
    });

    let result = navigate_path(&data, &["items[*]".to_string()]);
    assert_eq!(result, Ok(json!([{"id": 1}, {"id": 2}, {"id": 3}])));
  }

  #[test]
  fn test_navigate_array_with_nested_field() {
    let data = json!({
        "users": [
            {"name": "Alice", "age": 30},
            {"name": "Bob", "age": 25}
        ]
    });

    let result = navigate_path(&data, &["users[0]".to_string(), "name".to_string()]);
    assert_eq!(result, Ok(json!("Alice")));

    let result = navigate_path(&data, &["users[1]".to_string(), "age".to_string()]);
    assert_eq!(result, Ok(json!(25)));
  }

  #[test]
  fn test_navigate_index_out_of_bounds() {
    let data = json!({
        "items": ["a", "b"]
    });

    let result = navigate_path(&data, &["items[5]".to_string()]);
    assert!(matches!(
      result,
      Err(ArrayIndexError::IndexOutOfBounds { .. })
    ));
  }

  #[test]
  fn test_navigate_field_not_found() {
    let data = json!({"name": "Alice"});

    let result = navigate_path(&data, &["missing".to_string()]);
    assert!(matches!(result, Err(ArrayIndexError::FieldNotFound(_))));
  }

  #[test]
  fn test_navigate_not_an_array() {
    let data = json!({
        "items": "not an array"
    });

    let result = navigate_path(&data, &["items[0]".to_string()]);
    assert!(matches!(result, Err(ArrayIndexError::NotAnArray { .. })));
  }

  #[test]
  fn test_navigate_empty_path() {
    let data = json!({"name": "Alice"});

    let result = navigate_path(&data, &[]);
    assert_eq!(result, Ok(json!({"name": "Alice"})));
  }

  #[test]
  fn test_navigate_complex_path() {
    let data = json!({
        "data": {
            "users": [
                {"posts": [{"title": "First"}, {"title": "Second"}]},
                {"posts": [{"title": "Third"}]}
            ]
        }
    });

    let result = navigate_path(
      &data,
      &[
        "data".to_string(),
        "users[0]".to_string(),
        "posts[-1]".to_string(),
        "title".to_string(),
      ],
    );
    assert_eq!(result, Ok(json!("Second")));
  }

  #[test]
  fn test_navigate_wildcard_returns_array() {
    let data = json!({
        "items": [1, 2, 3]
    });

    let result = navigate_path(&data, &["items[*]".to_string()]);
    assert_eq!(result, Ok(json!([1, 2, 3])));
  }

  #[test]
  fn test_navigate_empty_array_with_wildcard() {
    let data = json!({
        "items": []
    });

    let result = navigate_path(&data, &["items[*]".to_string()]);
    assert_eq!(result, Ok(json!([])));
  }
}
