//! Pure core functions for JSONL session storage.
//!
//! This module contains the functional core for JSONL operations, following
//! Scott Wlaschin's DDD principle of separating pure logic from I/O.
//!
//! ## Design Principles
//!
//! - **Pure functions**: No I/O, no side effects, deterministic
//! - **Testable**: All functions can be unit tested without mocks
//! - **Composable**: Functions can be combined in pipelines
//!
//! ## Architecture
//!
//! ```text
//! Shell (I/O) -> Core (Pure) -> Shell (I/O)
//!    jsonl.rs  -> jsonl_core.rs -> jsonl.rs
//! ```

#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use itertools::Itertools;
use thiserror::Error;

/// Error type for JSONL operations.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum JsonlError {
  #[error("serialization failed: {0}")]
  SerializationError(String),

  #[error("failed to build JSONL content: {0}")]
  BuildContentError(String),
}

/// Typed failures produced while transforming sessions into JSONL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonlCoreError {
  /// A session could not be serialized into JSON.
  Serialization { details: String },
}

/// Result of parsing a JSONL line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonlLineParseResult<T> {
  /// Successfully parsed a session
  Success(T),
  /// Line was empty or whitespace only
  EmptyLine,
  /// Line contained invalid JSON
  ParseError {
    /// Line number (1-indexed)
    line_number: usize,
    /// Error message
    error: String,
  },
}

impl<T> JsonlLineParseResult<T> {
  /// Check if this is a success.
  #[must_use]
  pub const fn is_success(&self) -> bool {
    matches!(self, Self::Success(_))
  }

  /// Extract the success value, if any.
  #[must_use]
  pub fn success_value(self) -> Option<T> {
    match self {
      Self::Success(value) => Some(value),
      _ => None,
    }
  }
}

/// Serialize a session to a JSONL line.
///
/// This is a pure function that converts a session to a JSON string.
///
/// # Type Parameters
///
/// * `T` - A type that implements `serde::Serialize`
///
/// # Arguments
///
/// * `session` - The session to serialize
///
/// # Returns
///
/// `Ok(String)` if serialization succeeds, `Err(JsonlError)` otherwise
pub fn serialize_to_jsonl<T: serde::Serialize>(session: &T) -> Result<String, JsonlError> {
  serde_json::to_string(session).map_err(|e| JsonlError::SerializationError(e.to_string()))
}

/// Parse a single JSONL line.
///
/// This is a pure function that parses a JSON string into a session.
///
/// # Type Parameters
///
/// * `T` - A type that implements `serde::de::DeserializeOwned`
///
/// # Arguments
///
/// * `line` - The line to parse
/// * `line_number` - The 1-indexed line number for error reporting
///
/// # Returns
///
/// `JsonlLineParseResult` indicating success, empty line, or error
pub fn parse_jsonl_line<T: serde::de::DeserializeOwned>(
  line: &str,
  line_number: usize,
) -> JsonlLineParseResult<T> {
  let trimmed = line.trim();

  if trimmed.is_empty() {
    return JsonlLineParseResult::EmptyLine;
  }

  match serde_json::from_str::<T>(trimmed) {
    Ok(session) => JsonlLineParseResult::Success(session),
    Err(error) => JsonlLineParseResult::ParseError {
      line_number,
      error: error.to_string(),
    },
  }
}

/// Parse multiple JSONL lines into sessions.
///
/// This is a pure function that processes multiple lines.
///
/// # Type Parameters
///
/// * `T` - A type that implements `serde::de::DeserializeOwned`
///
/// # Arguments
///
/// * `lines` - The lines to parse
///
/// # Returns
///
/// A tuple of (successful sessions, errors)
pub fn parse_jsonl_lines<T: serde::de::DeserializeOwned + Clone>(
  lines: &[&str],
) -> (Vec<T>, Vec<(usize, String)>) {
  lines
    .iter()
    .enumerate()
    .map(|(idx, line)| parse_jsonl_line(line, idx + 1))
    .fold(
      (Vec::new(), Vec::new()),
      |(mut sessions, mut errors), result| {
        match result {
          JsonlLineParseResult::Success(session) => sessions.push(session),
          JsonlLineParseResult::EmptyLine => {}
          JsonlLineParseResult::ParseError { line_number, error } => {
            errors.push((line_number, error));
          }
        }
        (sessions, errors)
      },
    )
}

/// Filter sessions by ID (exclude matching).
///
/// This is a pure function that filters sessions.
///
/// # Arguments
///
/// * `sessions` - The sessions to filter
/// * `exclude_id` - The ID to exclude
///
/// # Returns
///
/// A new vector with sessions that don't match the exclude ID
#[must_use]
pub fn filter_sessions_by_id<T>(sessions: &[T], exclude_id: &str) -> Vec<T>
where
  T: HasId + Clone,
{
  sessions
    .iter()
    .filter(|session| session.id() != exclude_id)
    .cloned()
    .collect()
}

/// Find a session by ID.
///
/// This is a pure function that searches for a session.
///
/// # Arguments
///
/// * `sessions` - The sessions to search
/// * `id` - The ID to find
///
/// # Returns
///
/// `Some(session)` if found, `None` otherwise
#[must_use]
pub fn find_session_by_id<T>(sessions: &[T], id: &str) -> Option<T>
where
  T: HasId + Clone,
{
  sessions.iter().find(|session| session.id() == id).cloned()
}

/// Trait for types that have an ID.
pub trait HasId {
  /// Get the ID of this item.
  fn id(&self) -> &str;
}

/// Build JSONL content from sessions.
///
/// This is a pure function that builds the full JSONL file content.
///
/// # Arguments
///
/// * `sessions` - The sessions to include
///
/// # Returns
///
/// `Ok(String)` if all sessions serialize successfully, `Err(JsonlError)` otherwise
pub fn build_jsonl_content<T: serde::Serialize>(sessions: &[T]) -> Result<String, JsonlError> {
  sessions
    .iter()
    .map(|session| serialize_to_jsonl(session))
    .try_collect::<_, Vec<_>, _>()
    .map(|lines| lines.join("\n"))
}

/// Validate that content is valid JSONL format.
///
/// This is a pure function that validates JSONL structure.
///
/// # Arguments
///
/// * `content` - The content to validate
///
/// # Returns
///
/// `Ok(())` if valid, `Err(errors)` with line numbers and error messages
pub fn validate_jsonl_content(content: &str) -> Result<(), Vec<(usize, String)>> {
  let lines: Vec<&str> = content.lines().collect();
  let (_, errors) = parse_jsonl_lines::<serde_json::Value>(&lines);

  if errors.is_empty() {
    Ok(())
  } else {
    Err(errors)
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
  use serde::{Deserialize, Serialize};

  #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
  struct TestSession {
    id: String,
    name: String,
  }

  impl HasId for TestSession {
    fn id(&self) -> &str {
      &self.id
    }
  }

  #[test]
  fn serialize_to_jsonl_works() {
    let session = TestSession {
      id: "test-1".to_string(),
      name: "Test Session".to_string(),
    };
    let result = serialize_to_jsonl(&session);
    assert!(result.is_ok());
    let Ok(line) = result else {
      panic!("serialization should succeed");
    };
    assert!(line.contains("test-1"));
  }

  #[test]
  fn serialize_to_jsonl_returns_typed_error_for_failed_serialization() {
    #[derive(Serialize)]
    struct FailingSession;

    impl FailingSession {
      fn fail<S>(_value: &Self, _serializer: S) -> Result<S::Ok, S::Error>
      where
        S: serde::Serializer,
      {
        Err(serde::ser::Error::custom("boom"))
      }
    }

    #[derive(Serialize)]
    struct Wrapper {
      #[serde(serialize_with = "FailingSession::fail")]
      item: FailingSession,
    }

    let result = serialize_to_jsonl(&Wrapper {
      item: FailingSession,
    });

    assert!(matches!(
      result,
      Err(JsonlError::SerializationError(details)) if details.contains("boom")
    ));
  }

  #[test]
  fn parse_jsonl_line_success() {
    let line = r#"{"id":"test-1","name":"Test"}"#;
    let result: JsonlLineParseResult<TestSession> = parse_jsonl_line(line, 1);
    assert!(result.is_success());
    let session = result.success_value().unwrap();
    assert_eq!(session.id, "test-1");
  }

  #[test]
  fn parse_jsonl_line_empty() {
    let result: JsonlLineParseResult<TestSession> = parse_jsonl_line("", 1);
    assert!(matches!(result, JsonlLineParseResult::EmptyLine));
  }

  #[test]
  fn parse_jsonl_line_whitespace_only() {
    let result: JsonlLineParseResult<TestSession> = parse_jsonl_line("   ", 1);
    assert!(matches!(result, JsonlLineParseResult::EmptyLine));
  }

  #[test]
  fn parse_jsonl_line_invalid_json() {
    let result: JsonlLineParseResult<TestSession> = parse_jsonl_line("not json", 1);
    assert!(matches!(result, JsonlLineParseResult::ParseError { .. }));
  }

  #[test]
  fn parse_jsonl_lines_multiple() {
    let lines = vec![
      r#"{"id":"1","name":"A"}"#,
      "",
      r#"{"id":"2","name":"B"}"#,
      "invalid",
    ];
    let (sessions, errors) = parse_jsonl_lines::<TestSession>(&lines);
    assert_eq!(sessions.len(), 2);
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].0, 4); // Line number of invalid line
  }

  #[test]
  fn filter_sessions_by_id_works() {
    let sessions = vec![
      TestSession {
        id: "1".to_string(),
        name: "A".to_string(),
      },
      TestSession {
        id: "2".to_string(),
        name: "B".to_string(),
      },
      TestSession {
        id: "3".to_string(),
        name: "C".to_string(),
      },
    ];
    let filtered = filter_sessions_by_id(&sessions, "2");
    assert_eq!(filtered.len(), 2);
    assert!(filtered.iter().all(|s| s.id != "2"));
  }

  #[test]
  fn find_session_by_id_found() {
    let sessions = vec![
      TestSession {
        id: "1".to_string(),
        name: "A".to_string(),
      },
      TestSession {
        id: "2".to_string(),
        name: "B".to_string(),
      },
    ];
    let found = find_session_by_id(&sessions, "2");
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "B");
  }

  #[test]
  fn find_session_by_id_not_found() {
    let sessions = vec![TestSession {
      id: "1".to_string(),
      name: "A".to_string(),
    }];
    let found = find_session_by_id(&sessions, "999");
    assert!(found.is_none());
  }

  #[test]
  fn build_jsonl_content_works() {
    let sessions = vec![
      TestSession {
        id: "1".to_string(),
        name: "A".to_string(),
      },
      TestSession {
        id: "2".to_string(),
        name: "B".to_string(),
      },
    ];
    let Ok(content) = build_jsonl_content(&sessions) else {
      panic!("jsonl content should build");
    };
    assert!(content.contains('1'));
    assert!(content.contains('2'));
  }

  #[test]
  fn validate_jsonl_content_valid() {
    let content = r#"{"id":"1","name":"A"}
{"id":"2","name":"B"}"#;
    let result = validate_jsonl_content(content);
    assert!(result.is_ok());
  }

  #[test]
  fn validate_jsonl_content_invalid() {
    let content = r#"{"id":"1","name":"A"}
not valid json"#;
    let result = validate_jsonl_content(content);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert_eq!(errors.len(), 1);
  }

  #[test]
  fn jsonl_line_parse_result_is_success() {
    let success: JsonlLineParseResult<TestSession> = JsonlLineParseResult::Success(TestSession {
      id: "1".to_string(),
      name: "A".to_string(),
    });
    let empty: JsonlLineParseResult<TestSession> = JsonlLineParseResult::EmptyLine;
    let error: JsonlLineParseResult<TestSession> = JsonlLineParseResult::ParseError {
      line_number: 1,
      error: "error".to_string(),
    };

    assert!(success.is_success());
    assert!(!empty.is_success());
    assert!(!error.is_success());
  }

  #[test]
  fn jsonl_line_parse_result_success_value() {
    let session = TestSession {
      id: "1".to_string(),
      name: "A".to_string(),
    };
    let success: JsonlLineParseResult<TestSession> = JsonlLineParseResult::Success(session.clone());
    let empty: JsonlLineParseResult<TestSession> = JsonlLineParseResult::EmptyLine;

    assert_eq!(success.success_value(), Some(session));
    assert_eq!(empty.success_value(), None);
  }
}
