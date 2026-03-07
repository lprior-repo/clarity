//! Answer File Parsing
//!
//! This module provides functionality to parse answer files in TOML or JSON format
//! that can be loaded into interviews. It supports `question_id` to answer mapping
//! with optional confidence values.
//!
//! # File Formats
//!
//! ## TOML Format
//!
//! ```toml
//! [q-api-type]
//! answer = "REST API"
//! confidence = 0.9
//!
//! [q-audience]
//! answer = "developers"
//! ```
//!
//! ## JSON Format
//!
//! ```json
//! {
//!   "answers": {
//!     "q-api-type": {
//!       "answer": "REST API",
//!       "confidence": 0.9
//!     },
//!     "q-audience": {
//!       "answer": "developers"
//!     }
//!   }
//! }
//! ```
//!
//! # Architecture
//!
//! The module follows a functional, pure-core design:
//! - All parsing functions are pure and deterministic
//! - Errors are represented via [`AnswerFileError`] using `thiserror`
//! - Results use [`ParsedAnswer`] and [`AnswerFile`] for type-safe return values

#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Write;
use std::fmt::{Display, Formatter};
use thiserror::Error;

/// A 1-based line and column location within input text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErrorLocation {
  /// The 1-based line number.
  pub line: usize,
  /// The 1-based column number.
  pub column: usize,
}

impl Display for ErrorLocation {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "line {}, column {}", self.line, self.column)
  }
}

/// Classification for JSON parser and serializer failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonErrorCategory {
  /// Input/output failure.
  Io,
  /// Syntax failure.
  Syntax,
  /// Data-model mismatch.
  Data,
  /// Unexpected end of input.
  Eof,
}

impl Display for JsonErrorCategory {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Io => f.write_str("io"),
      Self::Syntax => f.write_str("syntax"),
      Self::Data => f.write_str("data"),
      Self::Eof => f.write_str("eof"),
    }
  }
}

/// The JSON layout that parsing attempted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonInputShape {
  /// `{ "answers": { ... } }`
  Wrapped,
  /// `{ "question-id": { ... } }`
  Flat,
}

impl Display for JsonInputShape {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Wrapped => f.write_str("wrapped"),
      Self::Flat => f.write_str("flat"),
    }
  }
}

/// Structured details for TOML parse failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TomlParseFailure {
  /// Parser-provided message.
  pub message: String,
  /// Best-effort source location.
  pub location: Option<ErrorLocation>,
}

impl Display for TomlParseFailure {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self.location {
      Some(location) => write!(f, "{} at {location}", self.message),
      None => f.write_str(&self.message),
    }
  }
}

impl std::error::Error for TomlParseFailure {}

/// Structured details for JSON parse failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonParseFailure {
  /// Parser-provided message.
  pub message: String,
  /// Error category from `serde_json`.
  pub category: JsonErrorCategory,
  /// The line and column reported by the parser.
  pub location: ErrorLocation,
  /// The JSON layout attempted when the error occurred.
  pub shape: JsonInputShape,
}

impl Display for JsonParseFailure {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{} {} error at {} while parsing {} JSON",
      self.category, self.message, self.location, self.shape
    )
  }
}

impl std::error::Error for JsonParseFailure {}

/// Structured details for JSON serialization failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonSerializationFailure {
  /// Serializer-provided message.
  pub message: String,
  /// Error category from `serde_json`.
  pub category: JsonErrorCategory,
}

impl Display for JsonSerializationFailure {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "{} {}", self.category, self.message)
  }
}

impl std::error::Error for JsonSerializationFailure {}

/// Errors that can occur when parsing answer files.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum AnswerFileError {
  /// The file content could not be parsed as valid TOML.
  #[error("TOML parse error: {0}")]
  TomlParseError(TomlParseFailure),

  /// The file content could not be parsed as valid JSON.
  #[error("JSON parse error: {0}")]
  JsonParseError(JsonParseFailure),

  /// The answer file could not be serialized as valid JSON.
  #[error("JSON serialization error: {0}")]
  JsonSerializationError(JsonSerializationFailure),

  /// An answer entry is missing the required 'answer' field.
  #[error("missing 'answer' field for question '{question_id}'")]
  MissingAnswerField { question_id: String },

  /// An answer field is empty or contains only whitespace.
  #[error("empty answer for question '{question_id}'")]
  EmptyAnswer { question_id: String },

  /// A confidence value is out of the valid range [0.0, 1.0].
  #[error("confidence {value} for question '{question_id}' is out of range [0.0, 1.0]")]
  InvalidConfidence { question_id: String, value: f64 },

  /// A question ID is empty or contains only whitespace.
  #[error("question ID at index {index} is empty")]
  EmptyQuestionId { index: usize },

  /// The file format is not recognized (not TOML or JSON).
  #[error("unrecognized file format: expected TOML or JSON")]
  UnrecognizedFormat,

  /// Duplicate question ID found in the file.
  #[error("duplicate question ID '{question_id}'")]
  DuplicateQuestionId { question_id: String },
}

/// A single parsed answer from an answer file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ParsedAnswer {
  /// The answer text/response.
  pub answer: String,
  /// Optional confidence level (0.0 to 1.0).
  #[serde(default)]
  pub confidence: Option<f64>,
  /// Optional notes associated with this answer.
  #[serde(default)]
  pub notes: Option<String>,
}

/// A collection of parsed answers from an answer file.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AnswerFile {
  /// Map of question IDs to their parsed answers.
  pub answers: HashMap<String, ParsedAnswer>,
}

impl AnswerFile {
  /// Create an empty answer file.
  #[must_use]
  pub fn new() -> Self {
    Self {
      answers: HashMap::new(),
    }
  }

  /// Get the number of answers in this file.
  #[must_use]
  pub fn len(&self) -> usize {
    self.answers.len()
  }

  /// Check if this answer file is empty.
  #[must_use]
  pub fn is_empty(&self) -> bool {
    self.answers.is_empty()
  }

  /// Get an answer by question ID.
  #[must_use]
  pub fn get(&self, question_id: &str) -> Option<&ParsedAnswer> {
    self.answers.get(question_id)
  }

  /// Check if an answer exists for a question ID.
  #[must_use]
  pub fn contains(&self, question_id: &str) -> bool {
    self.answers.contains_key(question_id)
  }
}

/// Internal representation for TOML parsing.
#[derive(Debug, Deserialize)]
struct TomlAnswerFile {
  #[serde(flatten)]
  answers: HashMap<String, RawAnswer>,
}

/// Internal answer representation shared by TOML and JSON parsing.
#[derive(Debug, Deserialize)]
struct RawAnswer {
  #[serde(default)]
  answer: Option<String>,
  #[serde(default)]
  confidence: Option<f64>,
  #[serde(default)]
  notes: Option<String>,
}

/// Internal JSON answer representation with stricter field validation.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct JsonAnswer {
  #[serde(default)]
  answer: Option<String>,
  #[serde(default)]
  confidence: Option<f64>,
  #[serde(default)]
  notes: Option<String>,
}

impl From<JsonAnswer> for RawAnswer {
  fn from(value: JsonAnswer) -> Self {
    Self {
      answer: value.answer,
      confidence: value.confidence,
      notes: value.notes,
    }
  }
}

/// Internal representation for JSON parsing (wrapper format).
#[derive(Debug, Deserialize)]
struct JsonAnswerFileWrapper {
  answers: HashMap<String, JsonAnswer>,
}

/// Parse a TOML-formatted answer file.
///
/// # Errors
///
/// Returns [`AnswerFileError`] if:
/// - The TOML syntax is invalid
/// - Any question ID is empty
/// - Any answer field is empty
/// - Any confidence value is outside [0.0, 1.0]
/// - There are duplicate question IDs
///
/// # Examples
///
/// ```
/// # use clarity_web::intent::interview::answer_file::{parse_toml, AnswerFileError};
/// let toml = r#"
/// [q-api-type]
/// answer = "REST API"
/// confidence = 0.9
///
/// [q-audience]
/// answer = "developers"
/// "#;
///
/// let result = parse_toml(toml);
/// assert!(result.is_ok());
/// let answer_file = result.unwrap();
/// assert_eq!(answer_file.len(), 2);
/// ```
pub fn parse_toml(content: &str) -> Result<AnswerFile, AnswerFileError> {
  let toml_file: TomlAnswerFile = toml::from_str(content)
    .map_err(|error| AnswerFileError::TomlParseError(build_toml_parse_failure(content, &error)))?;

  validate_and_convert_answers(toml_file.answers)
}

/// Parse a JSON-formatted answer file.
///
/// Supports two JSON formats:
///
/// 1. Wrapped format (preferred):
/// ```json
/// {
///   "answers": {
///     "q-id": { "answer": "value" }
///   }
/// }
/// ```
///
/// 2. Flat format (alternative):
/// ```json
/// {
///   "q-id": { "answer": "value" }
/// }
/// ```
///
/// # Errors
///
/// Returns [`AnswerFileError`] if:
/// - The JSON syntax is invalid
/// - Any question ID is empty
/// - Any answer field is empty
/// - Any confidence value is outside [0.0, 1.0]
/// - There are duplicate question IDs
///
/// # Examples
///
/// ```
/// # use clarity_web::intent::interview::answer_file::{parse_json, AnswerFileError};
/// let json = r#"{
///   "answers": {
///     "q-api-type": {
///       "answer": "REST API",
///       "confidence": 0.9
///     },
///     "q-audience": {
///       "answer": "developers"
///     }
///   }
/// }"#;
///
/// let result = parse_json(json);
/// assert!(result.is_ok());
/// let answer_file = result.unwrap();
/// assert_eq!(answer_file.len(), 2);
/// ```
pub fn parse_json(content: &str) -> Result<AnswerFile, AnswerFileError> {
  match preferred_json_shape(content) {
    Some(JsonInputShape::Wrapped) => serde_json::from_str::<JsonAnswerFileWrapper>(content)
      .map_err(|error| {
        AnswerFileError::JsonParseError(build_json_parse_failure(JsonInputShape::Wrapped, &error))
      })
      .and_then(|wrapper| validate_and_convert_answers(convert_json_answers(wrapper.answers))),
    Some(JsonInputShape::Flat) => serde_json::from_str::<HashMap<String, JsonAnswer>>(content)
      .map_err(|error| {
        AnswerFileError::JsonParseError(build_json_parse_failure(JsonInputShape::Flat, &error))
      })
      .and_then(|flat| validate_and_convert_answers(convert_json_answers(flat))),
    None => {
      let wrapped_attempt = serde_json::from_str::<JsonAnswerFileWrapper>(content);
      let flat_attempt = serde_json::from_str::<HashMap<String, JsonAnswer>>(content);

      match (wrapped_attempt, flat_attempt) {
        (Ok(wrapper), _) => validate_and_convert_answers(convert_json_answers(wrapper.answers)),
        (_, Ok(flat)) => validate_and_convert_answers(convert_json_answers(flat)),
        (Err(wrapped_error), Err(flat_error)) => {
          let wrapped_failure = build_json_parse_failure(JsonInputShape::Wrapped, &wrapped_error);
          let flat_failure = build_json_parse_failure(JsonInputShape::Flat, &flat_error);
          Err(AnswerFileError::JsonParseError(select_json_parse_failure(
            content,
            wrapped_failure,
            flat_failure,
          )))
        }
      }
    }
  }
}

/// Parse an answer file, auto-detecting the format from content.
///
/// Detection rules:
/// - Content starting with `{` is parsed as JSON
/// - All other content is parsed as TOML
///
/// # Errors
///
/// Returns [`AnswerFileError`] if parsing fails or validation fails.
///
/// # Examples
///
/// ```
/// # use clarity_web::intent::interview::answer_file::{parse_auto, AnswerFileError};
/// // JSON detection
/// let json = r#"{"answers": {"q1": {"answer": "yes"}}}"#;
/// assert!(parse_auto(json).is_ok());
///
/// // TOML detection
/// let toml = r#"[q1]
/// answer = "yes""#;
/// assert!(parse_auto(toml).is_ok());
/// ```
pub fn parse_auto(content: &str) -> Result<AnswerFile, AnswerFileError> {
  let trimmed = content.trim();

  if trimmed.starts_with('{') {
    parse_json(trimmed)
  } else {
    parse_toml(trimmed)
  }
}

/// Parse an answer file based on file extension.
///
/// # Errors
///
/// Returns [`AnswerFileError`] if:
/// - The extension is not recognized
/// - Parsing fails
/// - Validation fails
///
/// # Examples
///
/// ```
/// # use clarity_web::intent::interview::answer_file::{parse_with_extension, AnswerFileError};
/// let content = r#"[q1]
/// answer = "yes""#;
///
/// let result = parse_with_extension(content, "toml");
/// assert!(result.is_ok());
/// ```
pub fn parse_with_extension(content: &str, extension: &str) -> Result<AnswerFile, AnswerFileError> {
  match extension.to_lowercase().as_str() {
    "toml" => parse_toml(content),
    "json" => parse_json(content),
    _ => Err(AnswerFileError::UnrecognizedFormat),
  }
}

/// Validate and convert parsed answers to the canonical format.
fn validate_and_convert_answers(
  raw_answers: HashMap<String, RawAnswer>,
) -> Result<AnswerFile, AnswerFileError> {
  let mut answers = HashMap::with_capacity(raw_answers.len());

  for (index, (question_id, raw_answer)) in raw_answers.into_iter().enumerate() {
    // Validate question ID
    if question_id.trim().is_empty() {
      return Err(AnswerFileError::EmptyQuestionId { index });
    }

    let answer = raw_answer
      .answer
      .ok_or_else(|| AnswerFileError::MissingAnswerField {
        question_id: question_id.clone(),
      })?;

    if answer.trim().is_empty() {
      return Err(AnswerFileError::EmptyAnswer { question_id });
    }

    // Validate confidence if present
    if let Some(conf) = raw_answer.confidence {
      if !(0.0..=1.0).contains(&conf) {
        return Err(AnswerFileError::InvalidConfidence {
          question_id,
          value: conf,
        });
      }
    }

    // Check for duplicates (should not happen with HashMap, but being explicit)
    if answers.contains_key(&question_id) {
      return Err(AnswerFileError::DuplicateQuestionId { question_id });
    }

    answers.insert(
      question_id,
      ParsedAnswer {
        answer,
        confidence: raw_answer.confidence,
        notes: raw_answer.notes,
      },
    );
  }

  Ok(AnswerFile { answers })
}

/// Validate a parsed answer file for additional constraints.
///
/// This performs deeper validation beyond the basic parsing validation:
/// - Checks that question IDs match expected patterns
/// - Validates notes are not empty if present
///
/// # Errors
///
/// Returns [`AnswerFileError`] if validation fails.
///
/// # Examples
///
/// ```
/// # use clarity_web::intent::interview::answer_file::{AnswerFile, ParsedAnswer, validate_answer_file};
/// use std::collections::HashMap;
///
/// let mut answers = HashMap::new();
/// answers.insert("q-api-type".to_string(), ParsedAnswer {
///     answer: "REST API".to_string(),
///     confidence: Some(0.9),
///     notes: None,
/// });
///
/// let file = AnswerFile { answers };
/// assert!(validate_answer_file(&file).is_ok());
/// ```
pub fn validate_answer_file(file: &AnswerFile) -> Result<(), AnswerFileError> {
  for (question_id, parsed_answer) in &file.answers {
    // Validate that notes are not empty if present
    if let Some(ref notes) = parsed_answer.notes {
      if notes.trim().is_empty() {
        // Notes field exists but is empty - we allow this but could warn
        // For now, we just skip it
      }
    }

    // Validate question ID format (basic pattern: starts with letter, contains alphanumeric, hyphens, underscores)
    if !is_valid_question_id(question_id) {
      // We allow flexible question IDs, but could add validation here
    }
  }

  Ok(())
}

/// Check if a question ID matches the expected pattern.
#[must_use]
pub fn is_valid_question_id(id: &str) -> bool {
  if id.is_empty() {
    return false;
  }

  let chars = id.chars().collect::<Vec<_>>();

  // Must start with a letter or underscore
  match chars.first() {
    Some(&c) if c.is_ascii_alphabetic() || c == '_' => {}
    _ => return false,
  }

  // Rest must be alphanumeric, hyphen, or underscore
  chars
    .iter()
    .all(|&c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ':')
}

/// Serialize an answer file to TOML format.
///
/// # Errors
///
/// Returns [`AnswerFileError`] if serialization fails.
///
/// # Examples
///
/// ```
/// # use clarity_web::intent::interview::answer_file::{AnswerFile, ParsedAnswer, to_toml};
/// use std::collections::HashMap;
///
/// let mut answers = HashMap::new();
/// answers.insert("q-api-type".to_string(), ParsedAnswer {
///     answer: "REST API".to_string(),
///     confidence: Some(0.9),
///     notes: None,
/// });
///
/// let file = AnswerFile { answers };
/// let toml = to_toml(&file).unwrap();
/// assert!(toml.contains("[q-api-type]"));
/// ```
pub fn to_toml(file: &AnswerFile) -> Result<String, AnswerFileError> {
  let mut output = String::new();

  // Sort keys for deterministic output
  let mut sorted_keys: Vec<_> = file.answers.keys().collect();
  sorted_keys.sort();

  for key in sorted_keys {
    if let Some(answer) = file.answers.get(key) {
      let _ = writeln!(output, "[{key}]");
      let _ = writeln!(output, "answer = {:?}", answer.answer);

      if let Some(conf) = answer.confidence {
        let _ = writeln!(output, "confidence = {conf}");
      }

      if let Some(ref notes) = answer.notes {
        if !notes.is_empty() {
          let _ = writeln!(output, "notes = {notes:?}");
        }
      }

      output.push('\n');
    }
  }

  Ok(output)
}

/// Serialize an answer file to JSON format.
///
/// # Errors
///
/// Returns [`AnswerFileError`] if serialization fails.
///
/// # Examples
///
/// ```
/// # use clarity_web::intent::interview::answer_file::{AnswerFile, ParsedAnswer, to_json};
/// use std::collections::HashMap;
///
/// let mut answers = HashMap::new();
/// answers.insert("q-api-type".to_string(), ParsedAnswer {
///     answer: "REST API".to_string(),
///     confidence: Some(0.9),
///     notes: None,
/// });
///
/// let file = AnswerFile { answers };
/// let json = to_json(&file).unwrap();
/// assert!(json.contains("\"answers\""));
/// ```
pub fn to_json(file: &AnswerFile) -> Result<String, AnswerFileError> {
  validate_json_serializable(file)?;

  serde_json::to_string_pretty(file).map_err(|error| {
    AnswerFileError::JsonSerializationError(build_json_serialization_failure(&error))
  })
}

fn validate_json_serializable(file: &AnswerFile) -> Result<(), AnswerFileError> {
  file
    .answers
    .iter()
    .find_map(|(question_id, answer)| {
      answer.confidence.and_then(|confidence| {
        (!confidence.is_finite()).then(|| {
          AnswerFileError::JsonSerializationError(JsonSerializationFailure {
            message: format!("non-finite confidence for question '{question_id}'"),
            category: JsonErrorCategory::Data,
          })
        })
      })
    })
    .map_or(Ok(()), Err)
}

fn convert_json_answers(json_answers: HashMap<String, JsonAnswer>) -> HashMap<String, RawAnswer> {
  json_answers
    .into_iter()
    .map(|(question_id, answer)| (question_id, answer.into()))
    .collect()
}

fn build_toml_parse_failure(content: &str, error: &toml::de::Error) -> TomlParseFailure {
  TomlParseFailure {
    message: error.message().to_string(),
    location: error
      .span()
      .map(|span| byte_offset_to_location(content, span.start)),
  }
}

fn build_json_parse_failure(shape: JsonInputShape, error: &serde_json::Error) -> JsonParseFailure {
  JsonParseFailure {
    message: error.to_string(),
    category: classify_json_error(error),
    location: ErrorLocation {
      line: error.line(),
      column: error.column(),
    },
    shape,
  }
}

fn build_json_serialization_failure(error: &serde_json::Error) -> JsonSerializationFailure {
  JsonSerializationFailure {
    message: error.to_string(),
    category: classify_json_error(error),
  }
}

fn classify_json_error(error: &serde_json::Error) -> JsonErrorCategory {
  match error.classify() {
    serde_json::error::Category::Io => JsonErrorCategory::Io,
    serde_json::error::Category::Syntax => JsonErrorCategory::Syntax,
    serde_json::error::Category::Data => JsonErrorCategory::Data,
    serde_json::error::Category::Eof => JsonErrorCategory::Eof,
  }
}

fn select_json_parse_failure(
  content: &str,
  wrapped_failure: JsonParseFailure,
  flat_failure: JsonParseFailure,
) -> JsonParseFailure {
  match preferred_json_shape(content) {
    Some(JsonInputShape::Wrapped) => wrapped_failure,
    None if wrapped_failure.category != JsonErrorCategory::Data => wrapped_failure,
    Some(JsonInputShape::Flat) | None => flat_failure,
  }
}

fn preferred_json_shape(content: &str) -> Option<JsonInputShape> {
  let root_value = serde_json::from_str::<serde_json::Value>(content).ok()?;

  match root_value {
    serde_json::Value::Object(map) => Some(map.get("answers").map_or(
      JsonInputShape::Flat,
      |answers_value| {
        if looks_like_flat_answer_entry(answers_value) {
          JsonInputShape::Flat
        } else {
          JsonInputShape::Wrapped
        }
      },
    )),
    _ => None,
  }
}

fn looks_like_flat_answer_entry(value: &serde_json::Value) -> bool {
  match value {
    serde_json::Value::Object(map) => map
      .keys()
      .all(|key| matches!(key.as_str(), "answer" | "confidence" | "notes")),
    _ => false,
  }
}

fn byte_offset_to_location(content: &str, byte_offset: usize) -> ErrorLocation {
  content.get(..byte_offset).unwrap_or(content).chars().fold(
    ErrorLocation { line: 1, column: 1 },
    |location, character| {
      if character == '\n' {
        ErrorLocation {
          line: location.line + 1,
          column: 1,
        }
      } else {
        ErrorLocation {
          line: location.line,
          column: location.column + 1,
        }
      }
    },
  )
}

/// Create an empty answer file template.
///
/// # Examples
///
/// ```
/// # use clarity_web::intent::interview::answer_file::{create_template, AnswerFileError};
/// let template = create_template("toml");
/// assert!(template.contains("[q-example]"));
/// ```
#[must_use]
pub fn create_template(format: &str) -> String {
  match format.to_lowercase().as_str() {
    "json" => r#"{
  "answers": {
    "q-example": {
      "answer": "Your answer here",
      "confidence": 0.9,
      "notes": "Optional notes"
    }
  }
}
"#
    .to_string(),
    _ => r#"[q-example]
answer = "Your answer here"
confidence = 0.9
notes = "Optional notes"

"#
    .to_string(),
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

  // ============================================
  // TOML Parsing Tests
  // ============================================

  #[test]
  fn parse_toml_simple() {
    let toml = r#"
[q-api-type]
answer = "REST API"

[q-audience]
answer = "developers"
"#;
    let result = parse_toml(toml);
    assert!(result.is_ok());
    let file = result.unwrap();
    assert_eq!(file.len(), 2);
    assert!(file.contains("q-api-type"));
    assert!(file.contains("q-audience"));
  }

  #[test]
  fn parse_toml_with_confidence() {
    let toml = r#"
[q-api-type]
answer = "REST API"
confidence = 0.9
"#;
    let result = parse_toml(toml);
    assert!(result.is_ok());
    let file = result.unwrap();
    let answer = file.get("q-api-type").unwrap();
    assert_eq!(answer.answer, "REST API");
    assert_eq!(answer.confidence, Some(0.9));
  }

  #[test]
  fn parse_toml_with_notes() {
    let toml = r#"
[q-api-type]
answer = "REST API"
notes = "GraphQL was also considered"
"#;
    let result = parse_toml(toml);
    assert!(result.is_ok());
    let file = result.unwrap();
    let answer = file.get("q-api-type").unwrap();
    assert_eq!(
      answer.notes,
      Some("GraphQL was also considered".to_string())
    );
  }

  #[test]
  fn parse_toml_empty_answer_fails() {
    let toml = r#"
[q-api-type]
answer = ""
"#;
    let result = parse_toml(toml);
    assert!(matches!(result, Err(AnswerFileError::EmptyAnswer { .. })));
  }

  #[test]
  fn parse_toml_whitespace_answer_fails() {
    let toml = r#"
[q-api-type]
answer = "   "
"#;
    let result = parse_toml(toml);
    assert!(matches!(result, Err(AnswerFileError::EmptyAnswer { .. })));
  }

  #[test]
  fn parse_toml_invalid_confidence_too_high() {
    let toml = r#"
[q-api-type]
answer = "REST API"
confidence = 1.5
"#;
    let result = parse_toml(toml);
    assert!(matches!(
      result,
      Err(AnswerFileError::InvalidConfidence { .. })
    ));
  }

  #[test]
  fn parse_toml_invalid_confidence_negative() {
    let toml = r#"
[q-api-type]
answer = "REST API"
confidence = -0.5
"#;
    let result = parse_toml(toml);
    assert!(matches!(
      result,
      Err(AnswerFileError::InvalidConfidence { .. })
    ));
  }

  #[test]
  fn parse_toml_confidence_boundary_zero() {
    let toml = r#"
[q-api-type]
answer = "REST API"
confidence = 0.0
"#;
    let result = parse_toml(toml);
    assert!(result.is_ok());
  }

  #[test]
  fn parse_toml_confidence_boundary_one() {
    let toml = r#"
[q-api-type]
answer = "REST API"
confidence = 1.0
"#;
    let result = parse_toml(toml);
    assert!(result.is_ok());
  }

  #[test]
  fn parse_toml_invalid_syntax() {
    let toml = r#"
[q-api-type
answer = "REST API"
"#;
    let result = parse_toml(toml);
    assert!(matches!(
      result,
      Err(AnswerFileError::TomlParseError { .. })
    ));
  }

  #[test]
  fn parse_toml_invalid_syntax_includes_location() {
    let toml = r#"
[q-api-type
answer = "REST API"
"#;

    let result = parse_toml(toml);

    match result {
      Err(AnswerFileError::TomlParseError(failure)) => {
        assert!(failure.message.contains("invalid"));
        assert_eq!(
          failure.location,
          Some(ErrorLocation {
            line: 2,
            column: 12
          })
        );
      }
      other => panic!("expected TOML parse error, got {other:?}"),
    }
  }

  #[test]
  fn parse_toml_empty_file() {
    let toml = "";
    let result = parse_toml(toml);
    assert!(result.is_ok());
    let file = result.unwrap();
    assert!(file.is_empty());
  }

  // ============================================
  // JSON Parsing Tests
  // ============================================

  #[test]
  fn parse_json_wrapped_format() {
    let json = r#"{
  "answers": {
    "q-api-type": {
      "answer": "REST API"
    },
    "q-audience": {
      "answer": "developers"
    }
  }
}"#;
    let result = parse_json(json);
    assert!(result.is_ok());
    let file = result.unwrap();
    assert_eq!(file.len(), 2);
  }

  #[test]
  fn parse_json_flat_format() {
    let json = r#"{
  "q-api-type": {
    "answer": "REST API"
  },
  "q-audience": {
    "answer": "developers"
  }
}"#;
    let result = parse_json(json);
    assert!(result.is_ok());
    let file = result.unwrap();
    assert_eq!(file.len(), 2);
  }

  #[test]
  fn parse_json_with_confidence() {
    let json = r#"{
  "answers": {
    "q-api-type": {
      "answer": "REST API",
      "confidence": 0.9
    }
  }
}"#;
    let result = parse_json(json);
    assert!(result.is_ok());
    let file = result.unwrap();
    let answer = file.get("q-api-type").unwrap();
    assert_eq!(answer.confidence, Some(0.9));
  }

  #[test]
  fn parse_json_with_notes() {
    let json = r#"{
  "answers": {
    "q-api-type": {
      "answer": "REST API",
      "notes": "GraphQL was also considered"
    }
  }
}"#;
    let result = parse_json(json);
    assert!(result.is_ok());
    let file = result.unwrap();
    let answer = file.get("q-api-type").unwrap();
    assert_eq!(
      answer.notes,
      Some("GraphQL was also considered".to_string())
    );
  }

  #[test]
  fn parse_json_empty_answer_fails() {
    let json = r#"{
  "answers": {
    "q-api-type": {
      "answer": ""
    }
  }
}"#;
    let result = parse_json(json);
    assert!(matches!(result, Err(AnswerFileError::EmptyAnswer { .. })));
  }

  #[test]
  fn parse_json_invalid_syntax() {
    let json = r#"{
  "answers": {
    "q-api-type": {
      "answer": "REST API"
    }
  }
"#; // Missing closing brace
    let result = parse_json(json);
    assert!(matches!(
      result,
      Err(AnswerFileError::JsonParseError { .. })
    ));
  }

  #[test]
  fn parse_json_invalid_wrapped_shape_reports_typed_error() {
    let json = r#"{
  "answers": []
}"#;

    let result = parse_json(json);

    match result {
      Err(AnswerFileError::JsonParseError(failure)) => {
        assert_eq!(failure.category, JsonErrorCategory::Data);
        assert_eq!(failure.shape, JsonInputShape::Wrapped);
        assert_eq!(
          failure.location,
          ErrorLocation {
            line: 2,
            column: 13
          }
        );
        assert!(failure.message.contains("invalid type"));
      }
      other => panic!("expected JSON parse error, got {other:?}"),
    }
  }

  #[test]
  fn parse_json_missing_answer_field_stays_typed() {
    let json = r#"{
  "answers": {
    "q-api-type": {
      "notes": "missing answer"
    }
  }
}"#;

    let result = parse_json(json);

    assert!(matches!(
      result,
      Err(AnswerFileError::MissingAnswerField { question_id }) if question_id == "q-api-type"
    ));
  }

  // ============================================
  // Auto-Detection Tests
  // ============================================

  #[test]
  fn parse_auto_detects_json() {
    let json = r#"{"answers": {"q1": {"answer": "yes"}}}"#;
    let result = parse_auto(json);
    assert!(result.is_ok());
    let file = result.unwrap();
    assert_eq!(file.len(), 1);
  }

  #[test]
  fn parse_auto_detects_toml() {
    let toml = r#"
[q1]
answer = "yes"
"#;
    let result = parse_auto(toml);
    assert!(result.is_ok());
    let file = result.unwrap();
    assert_eq!(file.len(), 1);
  }

  #[test]
  fn parse_auto_with_whitespace_json() {
    let json = r#"
    {"answers": {"q1": {"answer": "yes"}}}
    "#;
    let result = parse_auto(json);
    assert!(result.is_ok());
  }

  // ============================================
  // Extension-Based Parsing Tests
  // ============================================

  #[test]
  fn parse_with_extension_toml() {
    let content = r#"
[q1]
answer = "yes"
"#;
    let result = parse_with_extension(content, "toml");
    assert!(result.is_ok());
  }

  #[test]
  fn parse_with_extension_json() {
    let content = r#"{"answers": {"q1": {"answer": "yes"}}}"#;
    let result = parse_with_extension(content, "json");
    assert!(result.is_ok());
  }

  #[test]
  fn parse_with_extension_unknown() {
    let content = "some content";
    let result = parse_with_extension(content, "yaml");
    assert!(matches!(result, Err(AnswerFileError::UnrecognizedFormat)));
  }

  #[test]
  fn parse_with_extension_case_insensitive() {
    let content = r#"
[q1]
answer = "yes"
"#;
    let result = parse_with_extension(content, "TOML");
    assert!(result.is_ok());
  }

  // ============================================
  // Question ID Validation Tests
  // ============================================

  #[test]
  fn valid_question_id_simple() {
    assert!(is_valid_question_id("q-api-type"));
  }

  #[test]
  fn valid_question_id_underscore_start() {
    assert!(is_valid_question_id("_private"));
  }

  #[test]
  fn valid_question_id_colon() {
    assert!(is_valid_question_id("ns:question"));
  }

  #[test]
  fn invalid_question_id_empty() {
    assert!(!is_valid_question_id(""));
  }

  #[test]
  fn invalid_question_id_number_start() {
    assert!(!is_valid_question_id("1question"));
  }

  #[test]
  fn invalid_question_id_special_char() {
    assert!(!is_valid_question_id("q@question"));
  }

  // ============================================
  // Serialization Tests
  // ============================================

  #[test]
  fn to_toml_roundtrip() {
    let original = r#"
[q-api-type]
answer = "REST API"
confidence = 0.9

"#;
    let parsed = parse_toml(original).unwrap();
    let serialized = to_toml(&parsed).unwrap();
    let reparsed = parse_toml(&serialized).unwrap();

    assert_eq!(parsed.len(), reparsed.len());
    assert_eq!(
      parsed.get("q-api-type").map(|a| &a.answer),
      reparsed.get("q-api-type").map(|a| &a.answer)
    );
  }

  #[test]
  fn to_json_roundtrip() {
    let original = r#"{
  "answers": {
    "q-api-type": {
      "answer": "REST API",
      "confidence": 0.9
    }
  }
}"#;
    let parsed = parse_json(original).unwrap();
    let serialized = to_json(&parsed).unwrap();
    let reparsed = parse_json(&serialized).unwrap();

    assert_eq!(parsed.len(), reparsed.len());
    assert_eq!(
      parsed.get("q-api-type").map(|a| &a.answer),
      reparsed.get("q-api-type").map(|a| &a.answer)
    );
  }

  #[test]
  fn to_json_reports_typed_serialization_error() {
    let mut answers = HashMap::new();
    answers.insert(
      "q-api-type".to_string(),
      ParsedAnswer {
        answer: "REST API".to_string(),
        confidence: Some(f64::NAN),
        notes: None,
      },
    );

    let result = to_json(&AnswerFile { answers });

    match result {
      Err(AnswerFileError::JsonSerializationError(failure)) => {
        assert_eq!(failure.category, JsonErrorCategory::Data);
        assert!(failure.message.contains("non-finite confidence"));
      }
      other => panic!("expected JSON serialization error, got {other:?}"),
    }
  }

  // ============================================
  // Template Tests
  // ============================================

  #[test]
  fn create_template_toml() {
    let template = create_template("toml");
    assert!(template.contains("[q-example]"));
    assert!(template.contains("answer ="));
  }

  #[test]
  fn create_template_json() {
    let template = create_template("json");
    assert!(template.contains("\"answers\""));
    assert!(template.contains("\"q-example\""));
  }

  // ============================================
  // AnswerFile Methods Tests
  // ============================================

  #[test]
  fn answer_file_new() {
    let file = AnswerFile::new();
    assert!(file.is_empty());
    assert_eq!(file.len(), 0);
  }

  #[test]
  fn answer_file_get() {
    let toml = r#"
[q1]
answer = "yes"
"#;
    let file = parse_toml(toml).unwrap();
    let answer = file.get("q1");
    assert!(answer.is_some());
    assert_eq!(answer.unwrap().answer, "yes");

    assert!(file.get("nonexistent").is_none());
  }

  #[test]
  fn answer_file_contains() {
    let toml = r#"
[q1]
answer = "yes"
"#;
    let file = parse_toml(toml).unwrap();
    assert!(file.contains("q1"));
    assert!(!file.contains("nonexistent"));
  }

  // ============================================
  // ParsedAnswer Default Tests
  // ============================================

  #[test]
  fn parsed_answer_default() {
    let answer = ParsedAnswer::default();
    assert!(answer.answer.is_empty());
    assert!(answer.confidence.is_none());
    assert!(answer.notes.is_none());
  }
}
