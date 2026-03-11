//! Domain Newtypes
//!
//! Validated identifier types that make illegal states unrepresentable.

use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::fmt;
use std::str::FromStr;

/// Error type for newtype parsing failures
#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize, Deserialize)]
pub enum NewtypeError {
  /// The string was empty when it should contain content
  #[error("value cannot be empty")]
  Empty,
}

// =============================================================================
// AnswerId - Non-empty identifier for answers
// =============================================================================

/// Unique identifier for an Answer.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct AnswerId(String);

impl AnswerId {
  /// Create a new AnswerId.
  pub fn new(s: String) -> Result<Self, NewtypeError> {
    Self::try_from(s)
  }

  /// Get the inner string as a str slice.
  #[must_use]
  pub fn as_str(&self) -> &str {
    &self.0
  }
}

impl fmt::Display for AnswerId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.0.fmt(f)
  }
}

impl From<AnswerId> for String {
  fn from(id: AnswerId) -> Self {
    id.0
  }
}

impl TryFrom<String> for AnswerId {
  type Error = NewtypeError;

  fn try_from(s: String) -> Result<Self, Self::Error> {
    if s.trim().is_empty() {
      return Err(NewtypeError::Empty);
    }
    Ok(Self(s))
  }
}

impl FromStr for AnswerId {
  type Err = NewtypeError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Self::try_from(s.to_string())
  }
}

// =============================================================================
// StepId - Non-empty identifier for steps
// =============================================================================

/// Unique identifier for a Step.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct StepId(String);

impl StepId {
  /// Create a new StepId.
  pub fn new(s: String) -> Result<Self, NewtypeError> {
    Self::try_from(s)
  }

  /// Get the inner string as a str slice.
  #[must_use]
  pub fn as_str(&self) -> &str {
    &self.0
  }
}

impl fmt::Display for StepId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.0.fmt(f)
  }
}

impl From<StepId> for String {
  fn from(id: StepId) -> Self {
    id.0
  }
}

impl TryFrom<String> for StepId {
  type Error = NewtypeError;

  fn try_from(s: String) -> Result<Self, Self::Error> {
    if s.trim().is_empty() {
      return Err(NewtypeError::Empty);
    }
    Ok(Self(s))
  }
}

impl FromStr for StepId {
  type Err = NewtypeError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Self::try_from(s.to_string())
  }
}

// =============================================================================
// BeadId - Non-empty identifier for beads
// =============================================================================

/// Unique identifier for a Bead.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct BeadId(String);

impl BeadId {
  /// Create a new BeadId.
  pub fn new(s: String) -> Result<Self, NewtypeError> {
    Self::try_from(s)
  }

  /// Get the inner string as a str slice.
  #[must_use]
  pub fn as_str(&self) -> &str {
    &self.0
  }
}

impl fmt::Display for BeadId {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.0.fmt(f)
  }
}

impl From<BeadId> for String {
  fn from(id: BeadId) -> Self {
    id.0
  }
}

impl TryFrom<String> for BeadId {
  type Error = NewtypeError;

  fn try_from(s: String) -> Result<Self, Self::Error> {
    if s.trim().is_empty() {
      return Err(NewtypeError::Empty);
    }
    Ok(Self(s))
  }
}

impl FromStr for BeadId {
  type Err = NewtypeError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Self::try_from(s.to_string())
  }
}

// =============================================================================
// AnswerValue - Can be empty, represents user input
// =============================================================================

/// The value of an answer - can be empty.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct AnswerValue(String);

impl AnswerValue {
  /// Create a new AnswerValue.
  #[must_use]
  pub const fn new(s: String) -> Self {
    Self(s)
  }

  /// Check if the value is empty.
  #[must_use]
  pub fn is_empty(&self) -> bool {
    self.0.is_empty()
  }

  /// Get the inner string as a str slice.
  #[must_use]
  pub fn as_str(&self) -> &str {
    &self.0
  }
}

impl fmt::Display for AnswerValue {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.0.fmt(f)
  }
}

impl From<String> for AnswerValue {
  fn from(s: String) -> Self {
    Self(s)
  }
}

impl From<&str> for AnswerValue {
  fn from(s: &str) -> Self {
    Self(s.to_string())
  }
}

impl From<AnswerValue> for String {
  fn from(v: AnswerValue) -> Self {
    v.0
  }
}

// =============================================================================
// Timestamp - ISO-8601 formatted timestamp
// =============================================================================

/// An ISO-8601 / RFC 3339 formatted timestamp.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Timestamp(String);

impl Timestamp {
  /// Create a new Timestamp, validating ISO-8601 format.
  pub fn new(s: String) -> Result<Self, NewtypeError> {
    Self::try_from(s)
  }

  /// Create a Timestamp from the current UTC time.
  #[must_use]
  pub fn now() -> Self {
    Self(chrono::Utc::now().to_rfc3339())
  }

  /// Get the inner string as a str slice.
  #[must_use]
  pub fn as_str(&self) -> &str {
    &self.0
  }
}

impl fmt::Display for Timestamp {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    self.0.fmt(f)
  }
}

impl From<Timestamp> for String {
  fn from(ts: Timestamp) -> Self {
    ts.0
  }
}

impl TryFrom<String> for Timestamp {
  type Error = NewtypeError;

  fn try_from(s: String) -> Result<Self, Self::Error> {
    if s.trim().is_empty() {
      return Err(NewtypeError::Empty);
    }
    chrono::DateTime::parse_from_rfc3339(&s)
      .map(|_| Self(s))
      .map_err(|_| NewtypeError::Empty)
  }
}

impl FromStr for Timestamp {
  type Err = NewtypeError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Self::try_from(s.to_string())
  }
}

impl Default for Timestamp {
  fn default() -> Self {
    Self::now()
  }
}
