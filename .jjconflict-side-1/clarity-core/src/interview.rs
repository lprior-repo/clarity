#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Interview types for Clarity
//!
//! This module defines the core domain types for interviews:
//! - Interview identifiers (strongly typed)
//! - Interview questions with validation
//! - Interview states (type-safe state machine)
//! - Interview metadata and results
//!
//! All types follow functional programming principles:
//! - Validation at construction time
//! - Immutable by default
//! - No unwraps or panics
//! - Result types for error handling

use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use thiserror::Error;

/// Unique identifier for an interview
///
/// Interview IDs are strongly typed wrappers around UUIDs.
/// They ensure type safety and prevent mixing IDs from different domains.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InterviewId(String);

impl InterviewId {
  /// Creates a new `InterviewId` from a string
  ///
  /// # Errors
  ///
  /// Returns `InterviewError::InvalidIdFormat` if the string is not a valid UUID
  ///
  /// # Examples
  ///
  /// ```rust
  /// use clarity_core::interview::InterviewId;
  ///
  /// // Valid UUID
  /// let id = InterviewId::new("550e8400-e29b-41d4-a716-446655440000".to_string());
  /// assert!(id.is_ok());
  ///
  /// // Invalid UUID
  /// let id = InterviewId::new("not-a-uuid".to_string());
  /// assert!(id.is_err());
  /// ```
  pub fn new(id: String) -> Result<Self, InterviewError> {
    if is_valid_uuid(&id) {
      Ok(Self(id))
    } else {
      Err(InterviewError::InvalidIdFormat(id))
    }
  }

  /// Get the underlying UUID string
  #[must_use]
  pub fn as_str(&self) -> &str {
    &self.0
  }
}

impl Display for InterviewId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

/// The state of an interview in its lifecycle
///
/// Interviews follow a strict state machine to prevent invalid transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InterviewState {
  /// Interview has been created but not started
  Created,

  /// Interview is currently in progress
  InProgress,

  /// Interview completed successfully
  Completed,

  /// Interview failed with an error
  Failed,

  /// Interview was cancelled before completion
  Cancelled,
}

impl Display for InterviewState {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Created => write!(f, "created"),
      Self::InProgress => write!(f, "in_progress"),
      Self::Completed => write!(f, "completed"),
      Self::Failed => write!(f, "failed"),
      Self::Cancelled => write!(f, "cancelled"),
    }
  }
}

/// An interview question with validation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
  /// Question text
  pub text: String,

  /// Optional help text
  pub help_text: Option<String>,

  /// Whether the question is required
  pub required: bool,

  /// Question type
  pub question_type: QuestionType,
}

/// Type of interview question
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuestionType {
  /// Free-form text input
  Text,

  /// Yes/No question
  Boolean,

  /// Multiple choice from options
  MultipleChoice,

  /// Numeric input
  Numeric,
}

/// An interview in the Clarity system
///
/// Interviews represent structured conversations to gather requirements.
/// They are immutable snapshots - state transitions create new Interview instances.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Interview {
  /// Unique identifier for this interview
  pub id: InterviewId,

  /// Spec name this interview is for
  pub spec_name: String,

  /// Current state of the interview
  pub state: InterviewState,

  /// Questions to ask
  pub questions: Vec<Question>,

  /// Answers provided
  pub answers: Vec<Answer>,

  /// When the interview was created
  pub created_at: Timestamp,

  /// When the interview was last updated
  pub updated_at: Timestamp,

  /// Optional title for the interview
  pub title: Option<String>,

  /// Optional description of the interview
  pub description: Option<String>,
}

impl Interview {
  /// Create a new interview
  ///
  /// # Errors
  ///
  /// Returns `InterviewError::InvalidIdFormat` if the ID is not a valid UUID
  /// Returns `InterviewError::EmptySpecName` if `spec_name` is empty
  ///
  /// # Examples
  ///
  /// ```rust
  /// use clarity_core::interview::{Interview, InterviewState, Timestamp};
  ///
  /// let interview = Interview::builder()
  ///     .id("550e8400-e29b-41d4-a716-446655440000".to_string())
  ///     .spec_name("my_spec".to_string())
  ///     .title("Requirements Interview".to_string())
  ///     .build();
  ///
  /// assert!(interview.is_ok());
  /// let interview = interview.unwrap();
  /// assert_eq!(interview.state, InterviewState::Created);
  /// ```
  pub fn new(
    id: InterviewId,
    spec_name: String,
    created_at: Timestamp,
  ) -> Result<Self, InterviewError> {
    if spec_name.trim().is_empty() {
      return Err(InterviewError::EmptySpecName);
    }

    Ok(Self {
      id,
      spec_name,
      state: InterviewState::Created,
      questions: Vec::new(),
      answers: Vec::new(),
      created_at,
      updated_at: created_at,
      title: None,
      description: None,
    })
  }

  /// Create a builder for constructing an Interview
  #[must_use]
  pub fn builder() -> InterviewBuilder {
    InterviewBuilder::new()
  }

  /// Transition the interview to a new state
  ///
  /// This validates that the state transition is allowed.
  ///
  /// # Errors
  ///
  /// Returns `InterviewError::InvalidStateTransition` if the transition is not allowed
  ///
  /// # Examples
  ///
  /// ```rust
  /// use clarity_core::interview::{Interview, InterviewState, Timestamp};
  ///
  /// let interview = Interview::builder()
  ///     .id("550e8400-e29b-41d4-a716-446655440000".to_string())
  ///     .spec_name("my_spec".to_string())
  ///     .build();
  ///
  /// assert!(interview.is_ok());
  /// let interview = interview.unwrap();
  ///
  /// // Valid transition: Created -> InProgress
  /// let ts = Timestamp::now();
  /// assert!(ts.is_ok());
  /// let updated = interview.transition_to(InterviewState::InProgress, ts.unwrap());
  /// assert!(updated.is_ok());
  /// ```
  pub fn transition_to(
    &self,
    new_state: InterviewState,
    updated_at: Timestamp,
  ) -> Result<Self, InterviewError> {
    if is_valid_transition(self.state, new_state) {
      Ok(Self {
        id: self.id.clone(),
        spec_name: self.spec_name.clone(),
        state: new_state,
        questions: self.questions.clone(),
        answers: self.answers.clone(),
        created_at: self.created_at,
        updated_at,
        title: self.title.clone(),
        description: self.description.clone(),
      })
    } else {
      Err(InterviewError::InvalidStateTransition {
        from: self.state,
        to: new_state,
      })
    }
  }

  /// Check if the interview is in a terminal state (completed, failed, or cancelled)
  #[must_use]
  pub const fn is_terminal(&self) -> bool {
    matches!(
      self.state,
      InterviewState::Completed | InterviewState::Failed | InterviewState::Cancelled
    )
  }

  /// Check if the interview is active (not in a terminal state)
  #[must_use]
  pub const fn is_active(&self) -> bool {
    !self.is_terminal()
  }
}

/// Builder for constructing Interview instances
///
/// Provides a fluent API for creating interviews with all optional fields.
#[derive(Debug, Clone, Default)]
pub struct InterviewBuilder {
  id: Option<String>,
  spec_name: Option<String>,
  created_at: Option<Timestamp>,
  title: Option<String>,
  description: Option<String>,
  questions: Vec<Question>,
}

impl InterviewBuilder {
  /// Create a new `InterviewBuilder`
  #[must_use]
  pub fn new() -> Self {
    Self::default()
  }

  /// Set the interview ID
  #[must_use]
  pub fn id(mut self, id: String) -> Self {
    self.id = Some(id);
    self
  }

  /// Set the spec name
  #[must_use]
  pub fn spec_name(mut self, spec_name: String) -> Self {
    self.spec_name = Some(spec_name);
    self
  }

  /// Set the creation timestamp
  #[must_use]
  pub const fn created_at(mut self, timestamp: Timestamp) -> Self {
    self.created_at = Some(timestamp);
    self
  }

  /// Set the interview title
  #[must_use]
  pub fn title(mut self, title: String) -> Self {
    self.title = Some(title);
    self
  }

  /// Set the interview description
  #[must_use]
  pub fn description(mut self, description: String) -> Self {
    self.description = Some(description);
    self
  }

  /// Add a question to the interview
  #[must_use]
  pub fn add_question(mut self, question: Question) -> Self {
    self.questions.push(question);
    self
  }

  /// Build the Interview
  ///
  /// # Errors
  ///
  /// Returns `InterviewError::MissingField` if required fields are not set
  /// Returns `InterviewError::InvalidIdFormat` if the ID is not a valid UUID
  /// Returns `InterviewError::SystemTimeInvalid` if no timestamp is provided and
  /// the system time is invalid
  /// Returns `InterviewError::EmptySpecName` if `spec_name` is empty
  pub fn build(self) -> Result<Interview, InterviewError> {
    let id = self
      .id
      .ok_or_else(|| InterviewError::MissingField("id".to_string()))?;
    let spec_name = self
      .spec_name
      .ok_or_else(|| InterviewError::MissingField("spec_name".to_string()))?;
    let created_at = match self.created_at {
      Some(ts) => ts,
      None => Timestamp::now()?,
    };

    let interview_id = InterviewId::new(id)?;
    let mut interview = Interview::new(interview_id, spec_name, created_at)?;
    interview.title = self.title;
    interview.description = self.description;
    interview.questions = self.questions;

    Ok(interview)
  }
}

/// An answer to an interview question
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answer {
  /// Index of the question this answers
  pub question_index: usize,

  /// The answer value
  pub value: AnswerValue,
}

/// The value of an answer
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnswerValue {
  /// Text answer
  Text(String),

  /// Boolean answer
  Boolean(bool),

  /// Multiple choice selection
  MultipleChoice(usize),

  /// Numeric answer
  Numeric(i64),
}

/// Timestamp for interview events
///
/// Represented as Unix timestamp (seconds since epoch).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(i64);

impl Timestamp {
  /// Create a new Timestamp from seconds since epoch
  #[must_use]
  pub const fn from_secs(secs: i64) -> Self {
    Self(secs)
  }

  /// Get the current time as a Timestamp
  ///
  /// # Errors
  ///
  /// Returns `InterviewError::SystemTimeInvalid` if the system time is invalid
  /// (e.g., due to clock skew or being set before `UNIX_EPOCH`)
  pub fn now() -> Result<Self, InterviewError> {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .map(|d| Self(d.as_secs().cast_signed()))
      .map_err(|_| InterviewError::SystemTimeInvalid)
  }

  /// Get the underlying seconds value
  #[must_use]
  pub const fn as_secs(&self) -> i64 {
    self.0
  }
}

impl Display for Timestamp {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

/// Errors that can occur when working with interviews
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum InterviewError {
  /// Invalid interview ID format (not a valid UUID)
  #[error("invalid interview ID format: {0}")]
  InvalidIdFormat(String),

  /// Attempted an invalid state transition
  #[error("invalid state transition from {from} to {to}")]
  InvalidStateTransition {
    from: InterviewState,
    to: InterviewState,
  },

  /// Missing required field when building an interview
  #[error("missing required field: {0}")]
  MissingField(String),

  /// Spec name cannot be empty
  #[error("spec name cannot be empty")]
  EmptySpecName,

  /// System time is invalid (clock skew or other time-related error)
  #[error("system time is invalid, cannot create timestamp")]
  SystemTimeInvalid,

  /// Invalid question index
  #[error("invalid question index: {0}")]
  InvalidQuestionIndex(usize),
}

/// Check if a string is a valid UUID format
fn is_valid_uuid(s: &str) -> bool {
  // Simple UUID format validation
  // UUIDs are 36 characters: 8-4-4-4-12 hex digits
  s.len() == 36
    && s.split('-').enumerate().all(|(i, part)| {
      let expected_len = [8, 4, 4, 4, 12][i];
      part.len() == expected_len && part.bytes().all(|b| b.is_ascii_hexdigit())
    })
}

/// Check if a state transition is valid
fn is_valid_transition(from: InterviewState, to: InterviewState) -> bool {
  from == to
    || matches!(
      (from, to),
      (
        InterviewState::Created,
        InterviewState::InProgress | InterviewState::Cancelled
      ) | (
        InterviewState::InProgress,
        InterviewState::Completed | InterviewState::Failed | InterviewState::Cancelled
      )
    )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_interview_id_new_valid_uuid() {
    let result = InterviewId::new("550e8400-e29b-41d4-a716-446655440000".to_string());
    assert!(result.is_ok());
    let id = match result {
      Ok(id) => id,
      Err(_) => panic!("Expected Ok InterviewId"),
    };
    assert_eq!(id.as_str(), "550e8400-e29b-41d4-a716-446655440000");
  }

  #[test]
  fn test_interview_id_new_invalid_format() {
    let result = InterviewId::new("not-a-uuid".to_string());
    assert!(result.is_err());
    match result {
      Err(InterviewError::InvalidIdFormat(s)) => {
        assert_eq!(s, "not-a-uuid");
      }
      _ => panic!("Expected InvalidIdFormat error"),
    }
  }

  #[test]
  fn test_interview_id_new_empty_string() {
    let result = InterviewId::new(String::new());
    assert!(result.is_err());
  }

  #[test]
  fn test_interview_id_new_too_short() {
    let result = InterviewId::new("short".to_string());
    assert!(result.is_err());
  }

  #[test]
  fn test_interview_id_display() {
    let id_result = InterviewId::new("550e8400-e29b-41d4-a716-446655440000".to_string());
    assert!(id_result.is_ok());
    let id = match id_result {
      Ok(id) => id,
      Err(_) => panic!("Expected Ok InterviewId"),
    };
    assert_eq!(format!("{id}"), "550e8400-e29b-41d4-a716-446655440000");
  }

  #[test]
  fn test_interview_state_display() {
    assert_eq!(format!("{}", InterviewState::Created), "created");
    assert_eq!(format!("{}", InterviewState::InProgress), "in_progress");
    assert_eq!(format!("{}", InterviewState::Completed), "completed");
    assert_eq!(format!("{}", InterviewState::Failed), "failed");
    assert_eq!(format!("{}", InterviewState::Cancelled), "cancelled");
  }

  #[test]
  fn test_interview_new() {
    let id_result = InterviewId::new("550e8400-e29b-41d4-a716-446655440000".to_string());
    assert!(id_result.is_ok());
    let id = match id_result {
      Ok(id) => id,
      Err(_) => panic!("Expected Ok InterviewId"),
    };
    let spec_name = "my_spec".to_string();
    let created_at = Timestamp::from_secs(1_234_567_890);

    let interview_result = Interview::new(id.clone(), spec_name, created_at);
    assert!(interview_result.is_ok());
    let interview = match interview_result {
      Ok(i) => i,
      Err(_) => panic!("Expected Ok Interview"),
    };

    assert_eq!(interview.id, id);
    assert_eq!(interview.spec_name, "my_spec");
    assert_eq!(interview.state, InterviewState::Created);
    assert_eq!(interview.created_at, created_at);
    assert_eq!(interview.updated_at, created_at);
    assert!(interview.title.is_none());
    assert!(interview.description.is_none());
    assert!(interview.questions.is_empty());
    assert!(interview.answers.is_empty());
  }

  #[test]
  fn test_interview_new_empty_spec_name() {
    let id_result = InterviewId::new("550e8400-e29b-41d4-a716-446655440000".to_string());
    assert!(id_result.is_ok());
    let id = match id_result {
      Ok(id) => id,
      Err(_) => panic!("Expected Ok InterviewId"),
    };

    let result = Interview::new(id, String::new(), Timestamp::from_secs(1_234_567_890));
    assert!(result.is_err());
    match result {
      Err(InterviewError::EmptySpecName) => {
        // Expected
      }
      _ => panic!("Expected EmptySpecName error"),
    }
  }

  #[test]
  fn test_interview_new_whitespace_spec_name() {
    let id_result = InterviewId::new("550e8400-e29b-41d4-a716-446655440000".to_string());
    assert!(id_result.is_ok());
    let id = match id_result {
      Ok(id) => id,
      Err(_) => panic!("Expected Ok InterviewId"),
    };

    let result = Interview::new(id, "   ".to_string(), Timestamp::from_secs(1_234_567_890));
    assert!(result.is_err());
    match result {
      Err(InterviewError::EmptySpecName) => {
        // Expected
      }
      _ => panic!("Expected EmptySpecName error"),
    }
  }

  #[test]
  fn test_interview_builder_minimal() {
    let interview_result = Interview::builder()
      .id("550e8400-e29b-41d4-a716-446655440000".to_string())
      .spec_name("my_spec".to_string())
      .build();

    assert!(interview_result.is_ok());
    let interview = match interview_result {
      Ok(i) => i,
      Err(_) => panic!("Expected Ok Interview"),
    };

    assert_eq!(interview.spec_name, "my_spec");
    assert_eq!(interview.state, InterviewState::Created);
    assert!(interview.title.is_none());
    assert!(interview.questions.is_empty());
  }

  #[test]
  fn test_interview_builder_full() {
    let interview_result = Interview::builder()
      .id("550e8400-e29b-41d4-a716-446655440000".to_string())
      .spec_name("my_spec".to_string())
      .title("Requirements Interview".to_string())
      .description("Gather user requirements".to_string())
      .build();

    assert!(interview_result.is_ok());
    let interview = match interview_result {
      Ok(i) => i,
      Err(_) => panic!("Expected Ok Interview"),
    };

    assert_eq!(interview.spec_name, "my_spec");
    assert_eq!(interview.title.as_deref(), Some("Requirements Interview"));
    assert_eq!(
      interview.description.as_deref(),
      Some("Gather user requirements")
    );
  }

  #[test]
  fn test_interview_builder_with_questions() {
    let interview_result = Interview::builder()
      .id("550e8400-e29b-41d4-a716-446655440000".to_string())
      .spec_name("my_spec".to_string())
      .add_question(Question {
        text: "What is your name?".to_string(),
        help_text: None,
        required: true,
        question_type: QuestionType::Text,
      })
      .add_question(Question {
        text: "Do you like Rust?".to_string(),
        help_text: Some("Please answer honestly".to_string()),
        required: true,
        question_type: QuestionType::Boolean,
      })
      .build();

    assert!(interview_result.is_ok());
    let interview = match interview_result {
      Ok(i) => i,
      Err(_) => panic!("Expected Ok Interview"),
    };

    assert_eq!(interview.questions.len(), 2);
    assert_eq!(interview.questions[0].text, "What is your name?");
    assert_eq!(interview.questions[1].text, "Do you like Rust?");
  }

  #[test]
  fn test_interview_builder_missing_id() {
    let result = Interview::builder()
      .spec_name("my_spec".to_string())
      .build();

    assert!(result.is_err());
    match result {
      Err(InterviewError::MissingField(field)) => {
        assert_eq!(field, "id");
      }
      _ => panic!("Expected MissingField error for 'id'"),
    }
  }

  #[test]
  fn test_interview_builder_missing_spec_name() {
    let result = Interview::builder()
      .id("550e8400-e29b-41d4-a716-446655440000".to_string())
      .build();

    assert!(result.is_err());
    match result {
      Err(InterviewError::MissingField(field)) => {
        assert_eq!(field, "spec_name");
      }
      _ => panic!("Expected MissingField error for 'spec_name'"),
    }
  }

  #[test]
  fn test_interview_builder_invalid_id() {
    let result = Interview::builder()
      .id("invalid-uuid".to_string())
      .spec_name("my_spec".to_string())
      .build();

    assert!(result.is_err());
    match result {
      Err(InterviewError::InvalidIdFormat(_)) => {
        // Expected
      }
      _ => panic!("Expected InvalidIdFormat error"),
    }
  }

  #[test]
  fn test_interview_builder_empty_spec_name() {
    let result = Interview::builder()
      .id("550e8400-e29b-41d4-a716-446655440000".to_string())
      .spec_name(String::new())
      .build();

    assert!(result.is_err());
    match result {
      Err(InterviewError::EmptySpecName) => {
        // Expected
      }
      _ => panic!("Expected EmptySpecName error"),
    }
  }

  #[test]
  fn test_interview_transition_to_in_progress() {
    let interview_result = Interview::builder()
      .id("550e8400-e29b-41d4-a716-446655440000".to_string())
      .spec_name("my_spec".to_string())
      .build();

    assert!(interview_result.is_ok());
    let interview = match interview_result {
      Ok(i) => i,
      Err(_) => panic!("Expected Ok Interview"),
    };

    let updated_result = interview.transition_to(
      InterviewState::InProgress,
      Timestamp::from_secs(1_234_567_891),
    );

    assert!(updated_result.is_ok());
    let updated = match updated_result {
      Ok(i) => i,
      Err(_) => panic!("Expected Ok Interview"),
    };

    assert_eq!(updated.state, InterviewState::InProgress);
    assert_eq!(updated.updated_at.as_secs(), 1_234_567_891);
    assert_eq!(updated.created_at, interview.created_at);
  }

  #[test]
  fn test_interview_transition_to_completed() {
    let interview_result = Interview::builder()
      .id("550e8400-e29b-41d4-a716-446655440000".to_string())
      .spec_name("my_spec".to_string())
      .build();

    assert!(interview_result.is_ok());
    let interview = match interview_result {
      Ok(i) => i,
      Err(_) => panic!("Expected Ok Interview"),
    };

    let in_progress_result = interview.transition_to(
      InterviewState::InProgress,
      Timestamp::from_secs(1_234_567_891),
    );

    assert!(in_progress_result.is_ok());
    let in_progress = match in_progress_result {
      Ok(i) => i,
      Err(_) => panic!("Expected Ok Interview"),
    };

    let completed_result = in_progress.transition_to(
      InterviewState::Completed,
      Timestamp::from_secs(1_234_567_892),
    );

    assert!(completed_result.is_ok());
    let completed = match completed_result {
      Ok(i) => i,
      Err(_) => panic!("Expected Ok Interview"),
    };

    assert_eq!(completed.state, InterviewState::Completed);
  }

  #[test]
  fn test_interview_transition_to_failed() {
    let interview_result = Interview::builder()
      .id("550e8400-e29b-41d4-a716-446655440000".to_string())
      .spec_name("my_spec".to_string())
      .build();

    assert!(interview_result.is_ok());
    let interview = match interview_result {
      Ok(i) => i,
      Err(_) => panic!("Expected Ok Interview"),
    };

    let in_progress_result = interview.transition_to(
      InterviewState::InProgress,
      Timestamp::from_secs(1_234_567_891),
    );

    assert!(in_progress_result.is_ok());
    let in_progress = match in_progress_result {
      Ok(i) => i,
      Err(_) => panic!("Expected Ok Interview"),
    };

    let failed_result =
      in_progress.transition_to(InterviewState::Failed, Timestamp::from_secs(1_234_567_892));

    assert!(failed_result.is_ok());
    let failed = match failed_result {
      Ok(i) => i,
      Err(_) => panic!("Expected Ok Interview"),
    };

    assert_eq!(failed.state, InterviewState::Failed);
  }

  #[test]
  fn test_interview_transition_to_cancelled() {
    let interview_result = Interview::builder()
      .id("550e8400-e29b-41d4-a716-446655440000".to_string())
      .spec_name("my_spec".to_string())
      .build();

    assert!(interview_result.is_ok());
    let interview = match interview_result {
      Ok(i) => i,
      Err(_) => panic!("Expected Ok Interview"),
    };

    let cancelled_result = interview.transition_to(
      InterviewState::Cancelled,
      Timestamp::from_secs(1_234_567_891),
    );

    assert!(cancelled_result.is_ok());
    let cancelled = match cancelled_result {
      Ok(i) => i,
      Err(_) => panic!("Expected Ok Interview"),
    };

    assert_eq!(cancelled.state, InterviewState::Cancelled);
  }

  #[test]
  fn test_interview_invalid_transition_created_to_completed() {
    let interview_result = Interview::builder()
      .id("550e8400-e29b-41d4-a716-446655440000".to_string())
      .spec_name("my_spec".to_string())
      .build();

    assert!(interview_result.is_ok());
    let interview = match interview_result {
      Ok(i) => i,
      Err(_) => panic!("Expected Ok Interview"),
    };

    let result = interview.transition_to(
      InterviewState::Completed,
      Timestamp::from_secs(1_234_567_891),
    );

    assert!(result.is_err());
    match result {
      Err(InterviewError::InvalidStateTransition { from, to }) => {
        assert_eq!(from, InterviewState::Created);
        assert_eq!(to, InterviewState::Completed);
      }
      _ => panic!("Expected InvalidStateTransition error"),
    }
  }

  #[test]
  fn test_interview_invalid_transition_completed_to_in_progress() {
    let interview_result = Interview::builder()
      .id("550e8400-e29b-41d4-a716-446655440000".to_string())
      .spec_name("my_spec".to_string())
      .build();

    assert!(interview_result.is_ok());
    let interview = match interview_result {
      Ok(i) => i,
      Err(_) => panic!("Expected Ok Interview"),
    };

    let in_progress_result = interview.transition_to(
      InterviewState::InProgress,
      Timestamp::from_secs(1_234_567_891),
    );

    assert!(in_progress_result.is_ok());
    let in_progress = match in_progress_result {
      Ok(i) => i,
      Err(_) => panic!("Expected Ok Interview"),
    };

    let completed_result = in_progress.transition_to(
      InterviewState::Completed,
      Timestamp::from_secs(1_234_567_892),
    );

    assert!(completed_result.is_ok());
    let completed = match completed_result {
      Ok(i) => i,
      Err(_) => panic!("Expected Ok Interview"),
    };

    let result = completed.transition_to(
      InterviewState::InProgress,
      Timestamp::from_secs(1_234_567_893),
    );

    assert!(result.is_err());
  }

  #[test]
  fn test_interview_invalid_transition_failed_to_in_progress() {
    let interview_result = Interview::builder()
      .id("550e8400-e29b-41d4-a716-446655440000".to_string())
      .spec_name("my_spec".to_string())
      .build();

    assert!(interview_result.is_ok());
    let interview = match interview_result {
      Ok(i) => i,
      Err(_) => panic!("Expected Ok Interview"),
    };

    let in_progress_result = interview.transition_to(
      InterviewState::InProgress,
      Timestamp::from_secs(1_234_567_891),
    );

    assert!(in_progress_result.is_ok());
    let in_progress = match in_progress_result {
      Ok(i) => i,
      Err(_) => panic!("Expected Ok Interview"),
    };

    let failed_result = in_progress.transition_to(InterviewState::Failed, Timestamp::from_secs(1_234_567_892));

    assert!(failed_result.is_ok());
    let failed = match failed_result {
      Ok(i) => i,
      Err(_) => panic!("Expected Ok Interview"),
    };

    let result = failed.transition_to(
      InterviewState::InProgress,
      Timestamp::from_secs(1_234_567_893),
    );

    assert!(result.is_err());
  }

  #[test]
  fn test_interview_is_terminal() {
    let interview_result = Interview::builder()
      .id("550e8400-e29b-41d4-a716-446655440000".to_string())
      .spec_name("my_spec".to_string())
      .build();

    assert!(interview_result.is_ok());
    let interview = match interview_result {
      Ok(i) => i,
      Err(_) => panic!("Expected Ok Interview"),
    };

    assert!(!interview.is_terminal());

    let in_progress_result = interview.transition_to(
      InterviewState::InProgress,
      Timestamp::from_secs(1_234_567_891),
    );

    assert!(in_progress_result.is_ok());
    let in_progress = match in_progress_result {
      Ok(i) => i,
      Err(_) => panic!("Expected Ok Interview"),
    };
    assert!(!in_progress.is_terminal());

    let completed_result = in_progress.transition_to(
      InterviewState::Completed,
      Timestamp::from_secs(1_234_567_892),
    );

    assert!(completed_result.is_ok());
    let completed = match completed_result {
      Ok(i) => i,
      Err(_) => panic!("Expected Ok Interview"),
    };
    assert!(completed.is_terminal());

    let failed_result = in_progress.transition_to(InterviewState::Failed, Timestamp::from_secs(1_234_567_892));

    assert!(failed_result.is_ok());
    let failed = match failed_result {
      Ok(i) => i,
      Err(_) => panic!("Expected Ok Interview"),
    };
    assert!(failed.is_terminal());

    let cancelled_result = interview.transition_to(
      InterviewState::Cancelled,
      Timestamp::from_secs(1_234_567_891),
    );

    assert!(cancelled_result.is_ok());
    let cancelled = match cancelled_result {
      Ok(i) => i,
      Err(_) => panic!("Expected Ok Interview"),
    };
    assert!(cancelled.is_terminal());
  }

  #[test]
  fn test_interview_is_active() {
    let interview_result = Interview::builder()
      .id("550e8400-e29b-41d4-a716-446655440000".to_string())
      .spec_name("my_spec".to_string())
      .build();

    assert!(interview_result.is_ok());
    let interview = match interview_result {
      Ok(i) => i,
      Err(_) => panic!("Expected Ok Interview"),
    };

    assert!(interview.is_active());

    let in_progress_result = interview.transition_to(
      InterviewState::InProgress,
      Timestamp::from_secs(1_234_567_891),
    );

    assert!(in_progress_result.is_ok());
    let in_progress = match in_progress_result {
      Ok(i) => i,
      Err(_) => panic!("Expected Ok Interview"),
    };
    assert!(in_progress.is_active());

    let completed_result = in_progress.transition_to(
      InterviewState::Completed,
      Timestamp::from_secs(1_234_567_892),
    );

    assert!(completed_result.is_ok());
    let completed = match completed_result {
      Ok(i) => i,
      Err(_) => panic!("Expected Ok Interview"),
    };
    assert!(!completed.is_active());
  }

  #[test]
  fn test_timestamp_from_secs() {
    let ts = Timestamp::from_secs(1_234_567_890);
    assert_eq!(ts.as_secs(), 1_234_567_890);
  }

  #[test]
  fn test_timestamp_now() {
    let ts1_result = Timestamp::now();
    assert!(ts1_result.is_ok());
    let ts1 = match ts1_result {
      Ok(ts) => ts,
      Err(_) => panic!("Expected Ok Timestamp"),
    };

    std::thread::sleep(std::time::Duration::from_secs(1));
    let ts2_result = Timestamp::now();
    assert!(ts2_result.is_ok());
    let ts2 = match ts2_result {
      Ok(ts) => ts,
      Err(_) => panic!("Expected Ok Timestamp"),
    };

    assert!(ts2 > ts1);
  }

  #[test]
  fn test_timestamp_display() {
    let ts = Timestamp::from_secs(1_234_567_890);
    assert_eq!(format!("{ts}"), "1234567890");
  }

  #[test]
  fn test_timestamp_ord() {
    let ts1 = Timestamp::from_secs(100);
    let ts2 = Timestamp::from_secs(200);

    assert!(ts1 < ts2);
    assert!(ts2 > ts1);
    assert_eq!(ts1, ts1);
  }

  #[test]
  fn test_interview_error_invalid_id_format_display() {
    let error = InterviewError::InvalidIdFormat("bad-id".to_string());
    assert_eq!(format!("{error}"), "invalid interview ID format: bad-id");
  }

  #[test]
  fn test_interview_error_invalid_state_transition_display() {
    let error = InterviewError::InvalidStateTransition {
      from: InterviewState::Completed,
      to: InterviewState::InProgress,
    };
    assert_eq!(
      format!("{error}"),
      "invalid state transition from completed to in_progress"
    );
  }

  #[test]
  fn test_interview_error_missing_field_display() {
    let error = InterviewError::MissingField("id".to_string());
    assert_eq!(format!("{error}"), "missing required field: id");
  }

  #[test]
  fn test_interview_error_empty_spec_name_display() {
    let error = InterviewError::EmptySpecName;
    assert_eq!(format!("{error}"), "spec name cannot be empty");
  }

  #[test]
  fn test_interview_error_system_time_invalid_display() {
    let error = InterviewError::SystemTimeInvalid;
    assert_eq!(
      format!("{error}"),
      "system time is invalid, cannot create timestamp"
    );
  }

  #[test]
  fn test_interview_error_invalid_question_index_display() {
    let error = InterviewError::InvalidQuestionIndex(42);
    assert_eq!(format!("{error}"), "invalid question index: 42");
  }

  #[test]
  fn test_is_valid_uuid_valid() {
    assert!(is_valid_uuid("550e8400-e29b-41d4-a716-446655440000"));
    assert!(is_valid_uuid("00000000-0000-0000-0000-000000000000"));
    assert!(is_valid_uuid("ffffffff-ffff-ffff-ffff-ffffffffffff"));
  }

  #[test]
  fn test_is_valid_uuid_invalid() {
    assert!(!is_valid_uuid("not-a-uuid"));
    assert!(!is_valid_uuid(""));
    assert!(!is_valid_uuid("550e8400-e29b-41d4-a716"));
    assert!(!is_valid_uuid("550e8400-e29b-41d4-a716-446655440000-extra"));
    assert!(!is_valid_uuid("550e8400-e29b-41d4-a716-44665544000g"));
    assert!(!is_valid_uuid("550e8400e29b41d4a716446655440000"));
  }

  #[test]
  fn test_is_valid_transition_same_state() {
    assert!(is_valid_transition(
      InterviewState::Created,
      InterviewState::Created
    ));
    assert!(is_valid_transition(
      InterviewState::InProgress,
      InterviewState::InProgress
    ));
  }

  #[test]
  fn test_is_valid_transition_created_to_in_progress() {
    assert!(is_valid_transition(
      InterviewState::Created,
      InterviewState::InProgress
    ));
  }

  #[test]
  fn test_is_valid_transition_created_to_cancelled() {
    assert!(is_valid_transition(
      InterviewState::Created,
      InterviewState::Cancelled
    ));
  }

  #[test]
  fn test_is_valid_transition_in_progress_to_completed() {
    assert!(is_valid_transition(
      InterviewState::InProgress,
      InterviewState::Completed
    ));
  }

  #[test]
  fn test_is_valid_transition_in_progress_to_failed() {
    assert!(is_valid_transition(
      InterviewState::InProgress,
      InterviewState::Failed
    ));
  }

  #[test]
  fn test_is_valid_transition_in_progress_to_cancelled() {
    assert!(is_valid_transition(
      InterviewState::InProgress,
      InterviewState::Cancelled
    ));
  }

  #[test]
  fn test_is_valid_transition_invalid() {
    assert!(!is_valid_transition(
      InterviewState::Completed,
      InterviewState::InProgress
    ));
    assert!(!is_valid_transition(
      InterviewState::Failed,
      InterviewState::InProgress
    ));
    assert!(!is_valid_transition(
      InterviewState::Cancelled,
      InterviewState::Created
    ));
    assert!(!is_valid_transition(
      InterviewState::Created,
      InterviewState::Completed
    ));
    assert!(!is_valid_transition(
      InterviewState::Created,
      InterviewState::Failed
    ));
  }

  #[test]
  fn test_answer_value_variants() {
    let text_answer = AnswerValue::Text("Hello".to_string());
    assert_eq!(text_answer, AnswerValue::Text("Hello".to_string()));

    let bool_answer = AnswerValue::Boolean(true);
    assert_eq!(bool_answer, AnswerValue::Boolean(true));

    let choice_answer = AnswerValue::MultipleChoice(2);
    assert_eq!(choice_answer, AnswerValue::MultipleChoice(2));

    let numeric_answer = AnswerValue::Numeric(42);
    assert_eq!(numeric_answer, AnswerValue::Numeric(42));
  }

  #[test]
  fn test_question_type_variants() {
    assert_eq!(QuestionType::Text, QuestionType::Text);
    assert_eq!(QuestionType::Boolean, QuestionType::Boolean);
    assert_eq!(QuestionType::MultipleChoice, QuestionType::MultipleChoice);
    assert_eq!(QuestionType::Numeric, QuestionType::Numeric);
  }

  #[test]
  fn test_question_creation() {
    let question = Question {
      text: "What is your name?".to_string(),
      help_text: Some("Enter your full name".to_string()),
      required: true,
      question_type: QuestionType::Text,
    };

    assert_eq!(question.text, "What is your name?");
    assert_eq!(question.help_text, Some("Enter your full name".to_string()));
    assert!(question.required);
    assert_eq!(question.question_type, QuestionType::Text);
  }

  #[test]
  fn test_answer_creation() {
    let answer = Answer {
      question_index: 0,
      value: AnswerValue::Text("Alice".to_string()),
    };

    assert_eq!(answer.question_index, 0);
    assert_eq!(answer.value, AnswerValue::Text("Alice".to_string()));
  }
}
