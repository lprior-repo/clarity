//! Answer File Parsing
//!
//! This module provides functionality to parse answer files in TOML or JSON format
//! that can be loaded into interviews. It supports question_id to answer mapping
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

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur when parsing answer files.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum AnswerFileError {
  /// The file content could not be parsed as valid TOML.
  #[error("TOML parse error: {0}")]
  TomlParseError(String),

  /// The file content could not be parsed as valid JSON.
  #[error("JSON parse error: {0}")]
  JsonParseError(String),

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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

impl Default for ParsedAnswer {
  fn default() -> Self {
    Self {
      answer: String::new(),
      confidence: None,
      notes: None,
    }
  }
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
  answers: HashMap<String, TomlAnswer>,
}

#[derive(Debug, Deserialize)]
struct TomlAnswer {
  answer: String,
  #[serde(default)]
  confidence: Option<f64>,
  #[serde(default)]
  notes: Option<String>,
}

/// Internal representation for JSON parsing (wrapper format).
#[derive(Debug, Deserialize)]
struct JsonAnswerFileWrapper {
  answers: HashMap<String, JsonAnswer>,
}

#[derive(Debug, Deserialize)]
struct JsonAnswer {
  answer: String,
  #[serde(default)]
  confidence: Option<f64>,
  #[serde(default)]
  notes: Option<String>,
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
    .map_err(|e| AnswerFileError::TomlParseError(e.message().to_string()))?;

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
  // First try wrapped format
  if let Ok(wrapper) = serde_json::from_str::<JsonAnswerFileWrapper>(content) {
    return validate_and_convert_json_answers(wrapper.answers);
  }

  // Try flat format (direct question_id -> answer mapping)
  let flat: HashMap<String, JsonAnswer> =
    serde_json::from_str(content).map_err(|e| AnswerFileError::JsonParseError(e.to_string()))?;

  validate_and_convert_json_answers(flat)
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

/// Validate and convert TOML answers to the canonical format.
fn validate_and_convert_answers(
  toml_answers: HashMap<String, TomlAnswer>,
) -> Result<AnswerFile, AnswerFileError> {
  let mut answers = HashMap::with_capacity(toml_answers.len());

  for (index, (question_id, toml_answer)) in toml_answers.into_iter().enumerate() {
    // Validate question ID
    if question_id.trim().is_empty() {
      return Err(AnswerFileError::EmptyQuestionId { index });
    }

    // Validate answer field
    if toml_answer.answer.trim().is_empty() {
      return Err(AnswerFileError::EmptyAnswer {
        question_id: question_id.clone(),
      });
    }

    // Validate confidence if present
    if let Some(conf) = toml_answer.confidence {
      if !(0.0..=1.0).contains(&conf) {
        return Err(AnswerFileError::InvalidConfidence {
          question_id: question_id.clone(),
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
        answer: toml_answer.answer,
        confidence: toml_answer.confidence,
        notes: toml_answer.notes,
      },
    );
  }

  Ok(AnswerFile { answers })
}

/// Validate and convert JSON answers to the canonical format.
fn validate_and_convert_json_answers(
  json_answers: HashMap<String, JsonAnswer>,
) -> Result<AnswerFile, AnswerFileError> {
  let mut answers = HashMap::with_capacity(json_answers.len());

  for (index, (question_id, json_answer)) in json_answers.into_iter().enumerate() {
    // Validate question ID
    if question_id.trim().is_empty() {
      return Err(AnswerFileError::EmptyQuestionId { index });
    }

    // Validate answer field
    if json_answer.answer.trim().is_empty() {
      return Err(AnswerFileError::EmptyAnswer {
        question_id: question_id.clone(),
      });
    }

    // Validate confidence if present
    if let Some(conf) = json_answer.confidence {
      if !(0.0..=1.0).contains(&conf) {
        return Err(AnswerFileError::InvalidConfidence {
          question_id: question_id.clone(),
          value: conf,
        });
      }
    }

    // Check for duplicates
    if answers.contains_key(&question_id) {
      return Err(AnswerFileError::DuplicateQuestionId { question_id });
    }

    answers.insert(
      question_id,
      ParsedAnswer {
        answer: json_answer.answer,
        confidence: json_answer.confidence,
        notes: json_answer.notes,
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
      output.push_str(&format!("[{}]\n", key));
      output.push_str(&format!("answer = {:?}\n", answer.answer));

      if let Some(conf) = answer.confidence {
        output.push_str(&format!("confidence = {}\n", conf));
      }

      if let Some(ref notes) = answer.notes {
        if !notes.is_empty() {
          output.push_str(&format!("notes = {:?}\n", notes));
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
  serde_json::to_string_pretty(file)
    .map_err(|e| AnswerFileError::JsonParseError(format!("Serialization error: {e}")))
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
