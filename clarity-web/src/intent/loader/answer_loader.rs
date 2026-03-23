//! Answer Loader Module
//!
//! Loads pre-filled answers from files (JSON format).
//! Supports both plain JSON and CUE files with automatic export.
//!
//! Ported from intent-cli/src/intent/answer_loader.gleam

#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use std::collections::HashMap;

use serde_json::Value;
use thiserror::Error;

/// I/O failure categories for answer loading.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnswerIoErrorKind {
  InvalidData,
  UnexpectedEof,
  Other,
}

impl std::fmt::Display for AnswerIoErrorKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::InvalidData => write!(f, "invalid data"),
      Self::UnexpectedEof => write!(f, "unexpected end of file"),
      Self::Other => write!(f, "other I/O error"),
    }
  }
}

/// JSON decoder failure categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonDecodeCategory {
  Io,
  Syntax,
  Data,
  Eof,
}

impl std::fmt::Display for JsonDecodeCategory {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Io => write!(f, "I/O"),
      Self::Syntax => write!(f, "syntax"),
      Self::Data => write!(f, "data"),
      Self::Eof => write!(f, "EOF"),
    }
  }
}

/// Structured parse failures for answer documents.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum AnswerParseError {
  #[error("invalid JSON ({category}) at line {line}, column {column}")]
  InvalidJson {
    category: JsonDecodeCategory,
    line: usize,
    column: usize,
  },

  #[error("top-level answers must be an object/map, got {actual}")]
  RootMustBeObject { actual: JsonInputKind },
}

/// Structured schema failures for answer documents.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum AnswerSchemaError {
  #[error("missing required field: {field}")]
  MissingField { field: String },

  #[error("unsupported schema shape: {shape}")]
  UnsupportedShape { shape: String },
}

/// Type expectations used by detailed parse helpers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonExpectation {
  JsonDocument,
  RootObject,
}

impl std::fmt::Display for JsonExpectation {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::JsonDocument => write!(f, "JSON"),
      Self::RootObject => write!(f, "Object"),
    }
  }
}

/// JSON input kinds used for typed error reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonInputKind {
  Null,
  Bool,
  Number,
  String,
  Array,
  Object,
  InvalidJson,
}

impl std::fmt::Display for JsonInputKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Null => write!(f, "Null"),
      Self::Bool => write!(f, "Bool"),
      Self::Number => write!(f, "Number"),
      Self::String => write!(f, "String"),
      Self::Array => write!(f, "Array"),
      Self::Object => write!(f, "Object"),
      Self::InvalidJson => write!(f, "invalid"),
    }
  }
}

/// Errors that can occur during answer loading
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum AnswerLoaderError {
  #[error("file not found: {0}")]
  FileNotFound(String),

  #[error("permission denied: {0}")]
  PermissionDenied(String),

  #[error("parse error in {path}: {reason}")]
  ParseError {
    path: String,
    reason: AnswerParseError,
  },

  #[error("schema error: {0}")]
  SchemaError(AnswerSchemaError),

  #[error("I/O error in {path}: {kind}")]
  IoError {
    path: String,
    kind: AnswerIoErrorKind,
  },
}

/// Parse error with detailed information for testing
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum ParseErrorWithDetails {
  #[error("parse error in {path}: expected {expected}, got {actual} - {reason}")]
  DecodeError {
    path: String,
    expected: JsonExpectation,
    actual: JsonInputKind,
    reason: AnswerParseError,
  },
}

/// Load answers from a file (JSON format)
///
/// # Errors
/// Returns `AnswerLoaderError` if the file cannot be read or parsed.
pub fn load_from_file(path: &str) -> Result<HashMap<String, String>, AnswerLoaderError> {
  let contents = std::fs::read_to_string(path).map_err(|e| match e.kind() {
    std::io::ErrorKind::NotFound => AnswerLoaderError::FileNotFound(path.to_string()),
    std::io::ErrorKind::PermissionDenied => AnswerLoaderError::PermissionDenied(path.to_string()),
    kind => AnswerLoaderError::IoError {
      path: path.to_string(),
      kind: map_io_error_kind(kind),
    },
  })?;

  parse_answers(path, &contents)
}

/// Parse answers from file contents
fn parse_answers(path: &str, contents: &str) -> Result<HashMap<String, String>, AnswerLoaderError> {
  let _is_cue_path = path_ends_with_cue(path);
  // For CUE files, we currently parse JSON-compatible content directly.
  parse_answers_json(path, contents)
}

/// Parse JSON content into a flat answer map
fn parse_answers_json(
  path: &str,
  json_str: &str,
) -> Result<HashMap<String, String>, AnswerLoaderError> {
  let value: Value =
    serde_json::from_str(json_str).map_err(|error| AnswerLoaderError::ParseError {
      path: path.to_string(),
      reason: map_json_parse_error(&error),
    })?;

  match value {
    Value::Object(map) => Ok(flatten_answers(map)),
    _ => Err(AnswerLoaderError::ParseError {
      path: path.to_string(),
      reason: AnswerParseError::RootMustBeObject {
        actual: value_type_name(&value),
      },
    }),
  }
}

/// Flatten nested object into dot-notation keys
fn flatten_answers(entries: serde_json::Map<String, Value>) -> HashMap<String, String> {
  let mut result = HashMap::new();
  for (key, value) in entries {
    flatten_value(&key, &value, &mut result);
  }
  result
}

/// Recursively flatten a value into the result map
fn flatten_value(key_path: &str, value: &Value, result: &mut HashMap<String, String>) {
  if let Value::Object(nested) = value {
    // Insert JSON representation of nested object
    let json_repr = value_to_string(value);
    result.insert(key_path.to_string(), json_repr);

    // Recursively flatten nested properties
    for (nested_key, nested_value) in nested {
      let new_path = format!("{key_path}.{nested_key}");
      flatten_value(&new_path, nested_value, result);
    }
  } else {
    // Leaf value: insert full path
    let value_as_text = value_to_string(value);
    result.insert(key_path.to_string(), value_as_text.clone());

    // Also insert short key (last segment) if it doesn't exist
    if let Some(short_key) = last_key_segment(key_path) {
      result.entry(short_key).or_insert(value_as_text);
    }
  }
}

/// Get the last segment of a dot-separated key path
fn last_key_segment(key_path: &str) -> Option<String> {
  key_path.split('.').next_back().map(String::from)
}

/// Convert a JSON value to a string representation
fn value_to_string(value: &Value) -> String {
  match value {
    Value::String(s) => s.clone(),
    Value::Number(n) => n.to_string(),
    Value::Bool(b) => b.to_string(),
    Value::Null => "null".to_string(),
    Value::Array(arr) => {
      let items: Vec<String> = arr.iter().map(value_to_string).collect();
      format!("[{}]", items.join(", "))
    }
    Value::Object(_) => {
      // Serialize object back to JSON string
      serde_json::to_string(value).map_or_else(|_| "{}".to_string(), |v| v)
    }
  }
}

const fn map_io_error_kind(kind: std::io::ErrorKind) -> AnswerIoErrorKind {
  match kind {
    std::io::ErrorKind::InvalidData => AnswerIoErrorKind::InvalidData,
    std::io::ErrorKind::UnexpectedEof => AnswerIoErrorKind::UnexpectedEof,
    _ => AnswerIoErrorKind::Other,
  }
}

fn map_json_parse_error(error: &serde_json::Error) -> AnswerParseError {
  AnswerParseError::InvalidJson {
    category: match error.classify() {
      serde_json::error::Category::Io => JsonDecodeCategory::Io,
      serde_json::error::Category::Syntax => JsonDecodeCategory::Syntax,
      serde_json::error::Category::Data => JsonDecodeCategory::Data,
      serde_json::error::Category::Eof => JsonDecodeCategory::Eof,
    },
    line: error.line(),
    column: error.column(),
  }
}

/// Check if path ends with .cue extension
fn path_ends_with_cue(path: &str) -> bool {
  path.to_lowercase().ends_with(".cue")
}

/// Test-only helper function with enhanced error reporting
///
/// # Errors
/// Returns `ParseErrorWithDetails` with detailed decode error information.
pub fn parse_answers_json_for_test(
  path: &str,
  json_str: &str,
) -> Result<HashMap<String, String>, ParseErrorWithDetails> {
  let value: Value =
    serde_json::from_str(json_str).map_err(|error| ParseErrorWithDetails::DecodeError {
      path: path.to_string(),
      expected: JsonExpectation::JsonDocument,
      actual: JsonInputKind::InvalidJson,
      reason: map_json_parse_error(&error),
    })?;

  match value {
    Value::Object(map) => Ok(flatten_answers(map)),
    _ => Err(ParseErrorWithDetails::DecodeError {
      path: path.to_string(),
      expected: JsonExpectation::RootObject,
      actual: value_type_name(&value),
      reason: AnswerParseError::RootMustBeObject {
        actual: value_type_name(&value),
      },
    }),
  }
}

/// Get type name from JSON value
const fn value_type_name(value: &Value) -> JsonInputKind {
  match value {
    Value::Null => JsonInputKind::Null,
    Value::Bool(_) => JsonInputKind::Bool,
    Value::Number(_) => JsonInputKind::Number,
    Value::String(_) => JsonInputKind::String,
    Value::Array(_) => JsonInputKind::Array,
    Value::Object(_) => JsonInputKind::Object,
  }
}

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

  #[test]
  fn test_load_from_file_not_found() {
    let result = load_from_file("/nonexistent/path/file.json");
    assert!(result.is_err());
    assert!(matches!(result, Err(AnswerLoaderError::FileNotFound(_))));
  }

  #[test]
  fn test_parse_answers_json_simple() {
    let json = r#"{"name": "test", "value": "123"}"#;
    let result = parse_answers_json("test.json", json);
    assert!(result.is_ok());
    let answers = result.unwrap();
    assert_eq!(answers.get("name"), Some(&"test".to_string()));
    assert_eq!(answers.get("value"), Some(&"123".to_string()));
  }

  #[test]
  fn test_parse_answers_json_nested() {
    let json = r#"{"user": {"name": "Alice", "email": "alice@example.com"}}"#;
    let result = parse_answers_json("test.json", json);
    assert!(result.is_ok());
    let answers = result.unwrap();
    assert_eq!(answers.get("user.name"), Some(&"Alice".to_string()));
    assert_eq!(
      answers.get("user.email"),
      Some(&"alice@example.com".to_string())
    );
    // Short keys should also exist
    assert_eq!(answers.get("name"), Some(&"Alice".to_string()));
  }

  #[test]
  fn test_parse_answers_json_not_object() {
    let json = r#"["not", "an", "object"]"#;
    let result = parse_answers_json("test.json", json);
    assert!(matches!(
      result,
      Err(AnswerLoaderError::ParseError {
        path,
        reason: AnswerParseError::RootMustBeObject {
          actual: JsonInputKind::Array,
        },
      }) if path == "test.json"
    ));
  }

  #[test]
  fn test_parse_answers_json_invalid() {
    let json = r"not valid json";
    let result = parse_answers_json("test.json", json);
    assert!(matches!(
      result,
      Err(AnswerLoaderError::ParseError {
        path,
        reason: AnswerParseError::InvalidJson {
          category: JsonDecodeCategory::Syntax,
          ..
        },
      }) if path == "test.json"
    ));
  }

  #[test]
  fn test_flatten_answers_preserves_short_keys() {
    let json = r#"{"config": {"database": {"host": "localhost"}}}"#;
    let value: Value = serde_json::from_str(json).unwrap();
    if let Value::Object(map) = value {
      let answers = flatten_answers(map);
      assert_eq!(answers.get("host"), Some(&"localhost".to_string()));
      assert_eq!(
        answers.get("config.database.host"),
        Some(&"localhost".to_string())
      );
    }
  }

  #[test]
  fn test_value_to_string_number() {
    let value = Value::Number(serde_json::Number::from(42));
    assert_eq!(value_to_string(&value), "42");
  }

  #[test]
  fn test_value_to_string_bool() {
    assert_eq!(value_to_string(&Value::Bool(true)), "true");
    assert_eq!(value_to_string(&Value::Bool(false)), "false");
  }

  #[test]
  fn test_value_to_string_null() {
    assert_eq!(value_to_string(&Value::Null), "null");
  }

  #[test]
  fn test_value_to_string_array() {
    let value = Value::Array(vec![
      Value::String("a".to_string()),
      Value::String("b".to_string()),
    ]);
    assert_eq!(value_to_string(&value), "[a, b]");
  }

  #[test]
  fn test_path_ends_with_cue() {
    assert!(path_ends_with_cue("schema.cue"));
    assert!(path_ends_with_cue("schema.CUE"));
    assert!(!path_ends_with_cue("schema.json"));
  }

  #[test]
  fn test_last_key_segment() {
    assert_eq!(last_key_segment("a.b.c"), Some("c".to_string()));
    assert_eq!(last_key_segment("single"), Some("single".to_string()));
    assert_eq!(last_key_segment(""), Some(String::new()));
  }

  #[test]
  fn test_parse_answers_json_for_test_success() {
    let json = r#"{"key": "value"}"#;
    let result = parse_answers_json_for_test("test.json", json);
    assert!(result.is_ok());
    let answers = result.unwrap();
    assert_eq!(answers.get("key"), Some(&"value".to_string()));
  }

  #[test]
  fn test_parse_answers_json_for_test_invalid_json() {
    let json = r"invalid";
    let result = parse_answers_json_for_test("test.json", json);
    assert!(matches!(
      result,
      Err(ParseErrorWithDetails::DecodeError {
        path,
        expected: JsonExpectation::JsonDocument,
        actual: JsonInputKind::InvalidJson,
        ..
      }) if path == "test.json"
    ));
  }

  #[test]
  fn test_parse_answers_json_for_test_not_object() {
    let json = r"[1, 2, 3]";
    let result = parse_answers_json_for_test("test.json", json);
    assert!(matches!(
      result,
      Err(ParseErrorWithDetails::DecodeError {
        expected: JsonExpectation::RootObject,
        actual: JsonInputKind::Array,
        ..
      })
    ));
  }

  #[test]
  fn test_value_type_name() {
    assert_eq!(value_type_name(&Value::Null), JsonInputKind::Null);
    assert_eq!(value_type_name(&Value::Bool(true)), JsonInputKind::Bool);
    assert_eq!(
      value_type_name(&Value::Number(serde_json::Number::from(1))),
      JsonInputKind::Number
    );
    assert_eq!(
      value_type_name(&Value::String("s".to_string())),
      JsonInputKind::String
    );
    assert_eq!(value_type_name(&Value::Array(vec![])), JsonInputKind::Array);
    assert_eq!(
      value_type_name(&Value::Object(serde_json::Map::new())),
      JsonInputKind::Object
    );
  }
}
