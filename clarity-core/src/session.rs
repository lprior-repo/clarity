#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::missing_const_for_fn)]

//! Session types for Clarity
//!
//! This module defines the core domain types for sessions:
//! - Session identifiers (strongly typed)
//! - Session states (type-safe state machine)
//! - Session types (Interview, Analysis, Planning)
//! - Session metadata and results
//!
//! All types follow functional programming principles:
//! - Validation at construction time
//! - Immutable by default
//! - No unwraps or panics
//! - Result types for error handling

use std::fmt::{self, Display};
use thiserror::Error;

/// Unique identifier for a session
///
/// Session IDs are strongly typed wrappers around UUIDs.
/// They ensure type safety and prevent mixing IDs from different domains.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SessionId(String);

impl SessionId {
  /// Creates a new `SessionId` from a string
  ///
  /// # Errors
  ///
  /// Returns `SessionError::InvalidIdFormat` if the string is not a valid UUID
  ///
  /// # Examples
  ///
  /// ```rust
  /// use clarity_core::session::SessionId;
  ///
  /// // Valid UUID
  /// let id = SessionId::new("550e8400-e29b-41d4-a716-446655440000".to_string());
  /// assert!(id.is_ok());
  ///
  /// // Invalid UUID
  /// let id = SessionId::new("not-a-uuid".to_string());
  /// assert!(id.is_err());
  /// ```
  pub fn new(id: String) -> Result<Self, SessionError> {
    // Validate UUID format
    if is_valid_uuid(&id) {
      Ok(Self(id))
    } else {
      Err(SessionError::InvalidIdFormat(id))
    }
  }

  /// Get the underlying UUID string
  #[must_use]
  pub fn as_str(&self) -> &str {
    &self.0
  }
}

impl Display for SessionId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

/// The type of session
///
/// Different session types represent different activities in the Clarity system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SessionKind {
  /// User interview session - gathering requirements and understanding user needs
  Interview,

  /// Analysis session - running KIRK analysis on specifications
  Analysis,

  /// Planning session - organizing work and creating execution plans
  Planning,
}

impl Display for SessionKind {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Interview => write!(f, "interview"),
      Self::Analysis => write!(f, "analysis"),
      Self::Planning => write!(f, "planning"),
    }
  }
}

/// The state of a session in its lifecycle
///
/// Sessions follow a strict state machine to prevent invalid transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SessionState {
  /// Session has been created but not started
  Created,

  /// Session is currently running
  InProgress,

  /// Session completed successfully
  Completed,

  /// Session failed with an error
  Failed,

  /// Session was cancelled before completion
  Cancelled,
}

impl Display for SessionState {
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

/// A session in the Clarity system
///
/// Sessions represent discrete units of work: interviews, analyses, or planning activities.
/// They are immutable snapshots - state transitions create new Session instances.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
  /// Unique identifier for this session
  pub id: SessionId,

  /// The type of session
  pub kind: SessionKind,

  /// Current state of the session
  pub state: SessionState,

  /// When the session was created
  pub created_at: Timestamp,

  /// When the session was last updated
  pub updated_at: Timestamp,

  /// Optional title for the session
  pub title: Option<String>,

  /// Optional description of the session
  pub description: Option<String>,
}

impl Session {
  /// Create a new session
  ///
  /// # Errors
  ///
  /// Returns `SessionError::InvalidIdFormat` if the ID is not a valid UUID
  ///
  /// # Examples
  ///
  /// ```rust
  /// use clarity_core::session::{Session, SessionKind, SessionState, Timestamp};
  ///
  /// let session = Session::builder()
  ///     .id("550e8400-e29b-41d4-a716-446655440000".to_string())
  ///     .kind(SessionKind::Interview)
  ///     .title("User Requirements Interview".to_string())
  ///     .build()
  ///     .expect("valid session ID and kind provided");
  ///
  /// assert_eq!(session.kind, SessionKind::Interview);
  /// assert_eq!(session.state, SessionState::Created);
  /// ```
  pub fn new(
    id: SessionId,
    kind: SessionKind,
    created_at: Timestamp,
  ) -> Result<Self, SessionError> {
    Ok(Self {
      id,
      kind,
      state: SessionState::Created,
      created_at,
      updated_at: created_at,
      title: None,
      description: None,
    })
  }

  /// Private constructor that accepts all fields for builder use
  #[allow(clippy::unnecessary_wraps)]
  fn with_optional_fields(
    id: SessionId,
    kind: SessionKind,
    created_at: Timestamp,
    title: Option<String>,
    description: Option<String>,
  ) -> Result<Self, SessionError> {
    Ok(Self {
      id,
      kind,
      state: SessionState::Created,
      created_at,
      updated_at: created_at,
      title,
      description,
    })
  }

  /// Create a builder for constructing a Session
  #[must_use]
  pub fn builder() -> SessionBuilder {
    SessionBuilder::new()
  }

  /// Transition the session to a new state
  ///
  /// This validates that the state transition is allowed.
  ///
  /// # Errors
  ///
  /// Returns `SessionError::InvalidStateTransition` if the transition is not allowed
  ///
  /// # Examples
  ///
  /// ```rust
  /// use clarity_core::session::{Session, SessionKind, SessionState, Timestamp};
  ///
  /// let session = Session::builder()
  ///     .id("550e8400-e29b-41d4-a716-446655440000".to_string())
  ///     .kind(SessionKind::Interview)
  ///     .build()
  ///     .expect("valid session ID and kind provided");
  ///
  /// // Valid transition: Created -> InProgress
  /// let updated = session.transition_to(
  ///     SessionState::InProgress,
  ///     Timestamp::now().expect("system time is valid")
  /// );
  /// assert!(updated.is_ok());
  ///
  /// // Invalid transition: Completed -> InProgress
  /// let completed = updated
  ///     .expect("transition succeeds")
  ///     .transition_to(
  ///         SessionState::Completed,
  ///         Timestamp::now().expect("system time is valid")
  ///     )
  ///     .expect("transition succeeds");
  /// let invalid = completed.transition_to(
  ///     SessionState::InProgress,
  ///     Timestamp::now().expect("system time is valid")
  /// );
  /// assert!(invalid.is_err());
  /// ```
  pub fn transition_to(
    &self,
    new_state: SessionState,
    updated_at: Timestamp,
  ) -> Result<Self, SessionError> {
    // Validate state transition
    if is_valid_transition(self.state, new_state) {
      Ok(Self {
        id: self.id.clone(),
        kind: self.kind,
        state: new_state,
        created_at: self.created_at,
        updated_at,
        title: self.title.clone(),
        description: self.description.clone(),
      })
    } else {
      Err(SessionError::InvalidStateTransition {
        from: self.state,
        to: new_state,
      })
    }
  }

  /// Check if the session is in a terminal state (completed, failed, or cancelled)
  #[must_use]
  pub const fn is_terminal(&self) -> bool {
    matches!(
      self.state,
      SessionState::Completed | SessionState::Failed | SessionState::Cancelled
    )
  }

  /// Check if the session is active (not in a terminal state)
  #[must_use]
  pub const fn is_active(&self) -> bool {
    !self.is_terminal()
  }
}

/// Builder for constructing Session instances
///
/// Provides a fluent API for creating sessions with all optional fields.
#[derive(Debug, Clone, Default)]
pub struct SessionBuilder {
  id: Option<String>,
  kind: Option<SessionKind>,
  created_at: Option<Timestamp>,
  title: Option<String>,
  description: Option<String>,
}

impl SessionBuilder {
  /// Create a new `SessionBuilder`
  #[must_use]
  pub fn new() -> Self {
    Self::default()
  }

  /// Set the session ID
  #[must_use]
  pub fn id(mut self, id: String) -> Self {
    self.id = Some(id);
    self
  }

  /// Set the session kind
  #[must_use]
  pub fn kind(mut self, kind: SessionKind) -> Self {
    self.kind = Some(kind);
    self
  }

  /// Set the creation timestamp
  #[must_use]
  pub fn created_at(mut self, timestamp: Timestamp) -> Self {
    self.created_at = Some(timestamp);
    self
  }

  /// Set the session title
  #[must_use]
  pub fn title(mut self, title: String) -> Self {
    self.title = Some(title);
    self
  }

  /// Set the session description
  #[must_use]
  pub fn description(mut self, description: String) -> Self {
    self.description = Some(description);
    self
  }

  /// Build the Session
  ///
  /// # Errors
  ///
  /// Returns `SessionError::MissingField` if required fields are not set
  /// Returns `SessionError::InvalidIdFormat` if the ID is not a valid UUID
  /// Returns `SessionError::SystemTimeInvalid` if no timestamp is provided and
  /// the system time is invalid
  pub fn build(self) -> Result<Session, SessionError> {
    let id = self
      .id
      .ok_or_else(|| SessionError::MissingField("id".to_string()))?;
    let kind = self
      .kind
      .ok_or_else(|| SessionError::MissingField("kind".to_string()))?;
    let created_at = match self.created_at {
      Some(ts) => ts,
      None => Timestamp::now()?,
    };

    let session_id = SessionId::new(id)?;
    Session::with_optional_fields(session_id, kind, created_at, self.title, self.description)
  }
}

/// Timestamp for session events
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
  /// Returns `SessionError::SystemTimeInvalid` if the system time is invalid
  /// (e.g., due to clock skew or being set before `UNIX_EPOCH`)
  pub fn now() -> Result<Self, SessionError> {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .map(|d| Self(d.as_secs().cast_signed()))
      .map_err(|_| SessionError::SystemTimeInvalid)
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

/// Errors that can occur when working with sessions
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SessionError {
  /// Invalid session ID format (not a valid UUID)
  #[error("invalid session ID format: {0}")]
  InvalidIdFormat(String),

  /// Attempted an invalid state transition
  #[error("invalid state transition from {from} to {to}")]
  InvalidStateTransition {
    from: SessionState,
    to: SessionState,
  },

  /// Missing required field when building a session
  #[error("missing required field: {0}")]
  MissingField(String),

  /// System time is invalid (clock skew or other time-related error)
  #[error("system time is invalid, cannot create timestamp")]
  SystemTimeInvalid,
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
fn is_valid_transition(from: SessionState, to: SessionState) -> bool {
  match (from, to) {
    // Valid transitions
    (SessionState::Created, SessionState::InProgress) => true,
    (SessionState::Created, SessionState::Cancelled) => true,
    (SessionState::InProgress, SessionState::Completed) => true,
    (SessionState::InProgress, SessionState::Failed) => true,
    (SessionState::InProgress, SessionState::Cancelled) => true,

    // Can always stay in the same state
    (s, t) if s == t => true,

    // All other transitions are invalid
    _ => false,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[allow(clippy::unwrap_used)]
  #[test]
  fn test_session_id_new_valid_uuid() {
    let result = SessionId::new("550e8400-e29b-41d4-a716-446655440000".to_string());
    assert!(result.is_ok());
    let id = result.unwrap();
    assert_eq!(id.as_str(), "550e8400-e29b-41d4-a716-446655440000");
  }

  #[allow(clippy::panic)]
  #[test]
  fn test_session_id_new_invalid_format() {
    let result = SessionId::new("not-a-uuid".to_string());
    assert!(result.is_err());
    match result {
      Err(SessionError::InvalidIdFormat(s)) => {
        assert_eq!(s, "not-a-uuid");
      }
      _ => panic!("Expected InvalidIdFormat error"),
    }
  }

  #[test]
  fn test_session_id_new_empty_string() {
    let result = SessionId::new(String::new());
    assert!(result.is_err());
  }

  #[test]
  fn test_session_id_new_too_short() {
    let result = SessionId::new("short".to_string());
    assert!(result.is_err());
  }

  #[allow(clippy::unwrap_used)]
  #[allow(clippy::uninlined_format_args)]
  #[test]
  fn test_session_id_display() {
    let id = SessionId::new("550e8400-e29b-41d4-a716-446655440000".to_string()).unwrap();
    assert_eq!(format!("{id}"), "550e8400-e29b-41d4-a716-446655440000");
  }

  #[allow(clippy::uninlined_format_args)]
  #[test]
  fn test_session_kind_display() {
    assert_eq!(format!("{}", SessionKind::Interview), "interview");
    assert_eq!(format!("{}", SessionKind::Analysis), "analysis");
    assert_eq!(format!("{}", SessionKind::Planning), "planning");
  }

  #[allow(clippy::uninlined_format_args)]
  #[test]
  fn test_session_state_display() {
    assert_eq!(format!("{}", SessionState::Created), "created");
    assert_eq!(format!("{}", SessionState::InProgress), "in_progress");
    assert_eq!(format!("{}", SessionState::Completed), "completed");
    assert_eq!(format!("{}", SessionState::Failed), "failed");
    assert_eq!(format!("{}", SessionState::Cancelled), "cancelled");
  }

  #[allow(clippy::unwrap_used)]
  #[test]
  fn test_session_new() {
    let id = SessionId::new("550e8400-e29b-41d4-a716-446655440000".to_string()).unwrap();
    let kind = SessionKind::Interview;
    let created_at = Timestamp::from_secs(1_234_567_890);

    let session = Session::new(id.clone(), kind, created_at).unwrap();

    assert_eq!(session.id, id);
    assert_eq!(session.kind, SessionKind::Interview);
    assert_eq!(session.state, SessionState::Created);
    assert_eq!(session.created_at, created_at);
    assert_eq!(session.updated_at, created_at);
    assert!(session.title.is_none());
    assert!(session.description.is_none());
  }

  #[allow(clippy::unwrap_used)]
  #[test]
  fn test_session_builder_minimal() {
    let session = Session::builder()
      .id("550e8400-e29b-41d4-a716-446655440000".to_string())
      .kind(SessionKind::Analysis)
      .build()
      .unwrap();

    assert_eq!(session.kind, SessionKind::Analysis);
    assert_eq!(session.state, SessionState::Created);
    assert!(session.title.is_none());
  }

  #[allow(clippy::unwrap_used)]
  #[test]
  fn test_session_builder_full() {
    let session = Session::builder()
      .id("550e8400-e29b-41d4-a716-446655440000".to_string())
      .kind(SessionKind::Planning)
      .title("Planning Session".to_string())
      .description("Plan the implementation".to_string())
      .build()
      .unwrap();

    assert_eq!(session.kind, SessionKind::Planning);
    assert_eq!(session.title.as_deref(), Some("Planning Session"));
    assert_eq!(
      session.description.as_deref(),
      Some("Plan the implementation")
    );
  }

  #[allow(clippy::panic)]
  #[test]
  fn test_session_builder_missing_id() {
    let result = Session::builder().kind(SessionKind::Interview).build();

    assert!(result.is_err());
    match result {
      Err(SessionError::MissingField(field)) => {
        assert_eq!(field, "id");
      }
      _ => panic!("Expected MissingField error for 'id'"),
    }
  }

  #[allow(clippy::panic)]
  #[test]
  fn test_session_builder_missing_kind() {
    let result = Session::builder()
      .id("550e8400-e29b-41d4-a716-446655440000".to_string())
      .build();

    assert!(result.is_err());
    match result {
      Err(SessionError::MissingField(field)) => {
        assert_eq!(field, "kind");
      }
      _ => panic!("Expected MissingField error for 'kind'"),
    }
  }

  #[allow(clippy::panic)]
  #[test]
  fn test_session_builder_invalid_id() {
    let result = Session::builder()
      .id("invalid-uuid".to_string())
      .kind(SessionKind::Interview)
      .build();

    assert!(result.is_err());
    match result {
      Err(SessionError::InvalidIdFormat(_)) => {
        // Expected
      }
      _ => panic!("Expected InvalidIdFormat error"),
    }
  }

  #[allow(clippy::unwrap_used)]
  #[test]
  fn test_session_transition_to_in_progress() {
    let session = Session::builder()
      .id("550e8400-e29b-41d4-a716-446655440000".to_string())
      .kind(SessionKind::Interview)
      .build()
      .unwrap();

    let updated = session
      .transition_to(
        SessionState::InProgress,
        Timestamp::from_secs(1_234_567_891),
      )
      .unwrap();

    assert_eq!(updated.state, SessionState::InProgress);
    assert_eq!(updated.updated_at.as_secs(), 1_234_567_891);
    assert_eq!(updated.created_at, session.created_at);
  }

  #[allow(clippy::unwrap_used)]
  #[test]
  fn test_session_transition_to_completed() {
    let session = Session::builder()
      .id("550e8400-e29b-41d4-a716-446655440000".to_string())
      .kind(SessionKind::Interview)
      .build()
      .unwrap();

    let in_progress = session
      .transition_to(
        SessionState::InProgress,
        Timestamp::from_secs(1_234_567_891),
      )
      .unwrap();

    let completed = in_progress
      .transition_to(SessionState::Completed, Timestamp::from_secs(1_234_567_892))
      .unwrap();

    assert_eq!(completed.state, SessionState::Completed);
  }

  #[allow(clippy::unwrap_used)]
  #[test]
  fn test_session_transition_to_failed() {
    let session = Session::builder()
      .id("550e8400-e29b-41d4-a716-446655440000".to_string())
      .kind(SessionKind::Interview)
      .build()
      .unwrap();

    let in_progress = session
      .transition_to(
        SessionState::InProgress,
        Timestamp::from_secs(1_234_567_891),
      )
      .unwrap();

    let failed = in_progress
      .transition_to(SessionState::Failed, Timestamp::from_secs(1_234_567_892))
      .unwrap();

    assert_eq!(failed.state, SessionState::Failed);
  }

  #[allow(clippy::unwrap_used)]
  #[test]
  fn test_session_transition_to_cancelled() {
    let session = Session::builder()
      .id("550e8400-e29b-41d4-a716-446655440000".to_string())
      .kind(SessionKind::Interview)
      .build()
      .unwrap();

    let cancelled = session
      .transition_to(SessionState::Cancelled, Timestamp::from_secs(1_234_567_891))
      .unwrap();

    assert_eq!(cancelled.state, SessionState::Cancelled);
  }

  #[allow(clippy::unwrap_used)]
  #[allow(clippy::panic)]
  #[test]
  fn test_session_invalid_transition_created_to_completed() {
    let session = Session::builder()
      .id("550e8400-e29b-41d4-a716-446655440000".to_string())
      .kind(SessionKind::Interview)
      .build()
      .unwrap();

    let result =
      session.transition_to(SessionState::Completed, Timestamp::from_secs(1_234_567_891));

    assert!(result.is_err());
    match result {
      Err(SessionError::InvalidStateTransition { from, to }) => {
        assert_eq!(from, SessionState::Created);
        assert_eq!(to, SessionState::Completed);
      }
      _ => panic!("Expected InvalidStateTransition error"),
    }
  }

  #[allow(clippy::unwrap_used)]
  #[test]
  fn test_session_invalid_transition_completed_to_in_progress() {
    let session = Session::builder()
      .id("550e8400-e29b-41d4-a716-446655440000".to_string())
      .kind(SessionKind::Interview)
      .build()
      .unwrap();

    let completed = session
      .transition_to(
        SessionState::InProgress,
        Timestamp::from_secs(1_234_567_891),
      )
      .unwrap()
      .transition_to(SessionState::Completed, Timestamp::from_secs(1_234_567_892))
      .unwrap();

    let result = completed.transition_to(
      SessionState::InProgress,
      Timestamp::from_secs(1_234_567_893),
    );

    assert!(result.is_err());
  }

  #[allow(clippy::unwrap_used)]
  #[test]
  fn test_session_invalid_transition_failed_to_in_progress() {
    let session = Session::builder()
      .id("550e8400-e29b-41d4-a716-446655440000".to_string())
      .kind(SessionKind::Interview)
      .build()
      .unwrap();

    let failed = session
      .transition_to(
        SessionState::InProgress,
        Timestamp::from_secs(1_234_567_891),
      )
      .unwrap()
      .transition_to(SessionState::Failed, Timestamp::from_secs(1_234_567_892))
      .unwrap();

    let result = failed.transition_to(
      SessionState::InProgress,
      Timestamp::from_secs(1_234_567_893),
    );

    assert!(result.is_err());
  }

  #[allow(clippy::unwrap_used)]
  #[test]
  fn test_session_is_terminal() {
    let session = Session::builder()
      .id("550e8400-e29b-41d4-a716-446655440000".to_string())
      .kind(SessionKind::Interview)
      .build()
      .unwrap();

    assert!(!session.is_terminal());

    let in_progress = session
      .transition_to(
        SessionState::InProgress,
        Timestamp::from_secs(1_234_567_891),
      )
      .unwrap();
    assert!(!in_progress.is_terminal());

    let completed = in_progress
      .transition_to(SessionState::Completed, Timestamp::from_secs(1_234_567_892))
      .unwrap();
    assert!(completed.is_terminal());

    let failed = in_progress
      .transition_to(SessionState::Failed, Timestamp::from_secs(1_234_567_892))
      .unwrap();
    assert!(failed.is_terminal());

    let cancelled = session
      .transition_to(SessionState::Cancelled, Timestamp::from_secs(1_234_567_891))
      .unwrap();
    assert!(cancelled.is_terminal());
  }

  #[allow(clippy::unwrap_used)]
  #[test]
  fn test_session_is_active() {
    let session = Session::builder()
      .id("550e8400-e29b-41d4-a716-446655440000".to_string())
      .kind(SessionKind::Interview)
      .build()
      .unwrap();

    assert!(session.is_active());

    let in_progress = session
      .transition_to(
        SessionState::InProgress,
        Timestamp::from_secs(1_234_567_891),
      )
      .unwrap();
    assert!(in_progress.is_active());

    let completed = in_progress
      .transition_to(SessionState::Completed, Timestamp::from_secs(1_234_567_892))
      .unwrap();
    assert!(!completed.is_active());
  }

  #[test]
  fn test_timestamp_from_secs() {
    let ts = Timestamp::from_secs(1_234_567_890);
    assert_eq!(ts.as_secs(), 1_234_567_890);
  }

  #[allow(clippy::unwrap_used)]
  #[test]
  fn test_timestamp_now() {
    let ts1 = Timestamp::now().unwrap();
    std::thread::sleep(std::time::Duration::from_secs(1));
    let ts2 = Timestamp::now().unwrap();

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
  fn test_session_error_invalid_id_format_display() {
    let error = SessionError::InvalidIdFormat("bad-id".to_string());
    assert_eq!(format!("{error}"), "invalid session ID format: bad-id");
  }

  #[test]
  fn test_session_error_invalid_state_transition_display() {
    let error = SessionError::InvalidStateTransition {
      from: SessionState::Completed,
      to: SessionState::InProgress,
    };
    assert_eq!(
      format!("{error}"),
      "invalid state transition from completed to in_progress"
    );
  }

  #[test]
  fn test_session_error_missing_field_display() {
    let error = SessionError::MissingField("id".to_string());
    assert_eq!(format!("{error}"), "missing required field: id");
  }

  #[test]
  fn test_session_error_system_time_invalid_display() {
    let error = SessionError::SystemTimeInvalid;
    assert_eq!(
      format!("{error}"),
      "system time is invalid, cannot create timestamp"
    );
  }

  #[test]
  fn test_timestamp_now_returns_result() {
    let result = Timestamp::now();
    assert!(result.is_ok());
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
      SessionState::Created,
      SessionState::Created
    ));
    assert!(is_valid_transition(
      SessionState::InProgress,
      SessionState::InProgress
    ));
  }

  #[test]
  fn test_is_valid_transition_created_to_in_progress() {
    assert!(is_valid_transition(
      SessionState::Created,
      SessionState::InProgress
    ));
  }

  #[test]
  fn test_is_valid_transition_created_to_cancelled() {
    assert!(is_valid_transition(
      SessionState::Created,
      SessionState::Cancelled
    ));
  }

  #[test]
  fn test_is_valid_transition_in_progress_to_completed() {
    assert!(is_valid_transition(
      SessionState::InProgress,
      SessionState::Completed
    ));
  }

  #[test]
  fn test_is_valid_transition_in_progress_to_failed() {
    assert!(is_valid_transition(
      SessionState::InProgress,
      SessionState::Failed
    ));
  }

  #[test]
  fn test_is_valid_transition_in_progress_to_cancelled() {
    assert!(is_valid_transition(
      SessionState::InProgress,
      SessionState::Cancelled
    ));
  }

  #[test]
  fn test_is_valid_transition_invalid() {
    assert!(!is_valid_transition(
      SessionState::Completed,
      SessionState::InProgress
    ));
    assert!(!is_valid_transition(
      SessionState::Failed,
      SessionState::InProgress
    ));
    assert!(!is_valid_transition(
      SessionState::Cancelled,
      SessionState::Created
    ));
    assert!(!is_valid_transition(
      SessionState::Created,
      SessionState::Completed
    ));
    assert!(!is_valid_transition(
      SessionState::Created,
      SessionState::Failed
    ));
  }
}
