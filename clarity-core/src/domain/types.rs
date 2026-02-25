#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Core types with validation

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Macro to generate UUID-based ID types
macro_rules! id_type {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(pub Uuid);

        impl $name {
            /// Create a new random ID
            #[must_use]
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            /// Parse an ID from a string
            ///
            /// # Errors
            /// Returns a `ValidationError` if the string is not a valid UUID.
            pub fn parse(s: &str) -> Result<Self, ValidationError> {
                Uuid::parse_str(s)
                    .map(Self)
                    .map_err(|_| ValidationError::InvalidUuid(s.to_string()))
            }

            pub const fn as_uuid(&self) -> Uuid {
                self.0
            }

            #[must_use]
            pub fn as_str(&self) -> String {
                self.0.to_string()
            }
        }

        impl std::str::FromStr for $name {
            type Err = ValidationError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Self::parse(s)
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<$name> for Uuid {
            fn from(id: $name) -> Self {
                id.0
            }
        }

        impl From<Uuid> for $name {
            fn from(uuid: Uuid) -> Self {
                Self(uuid)
            }
        }
    };
}

id_type!(
    /// Unique identifier for users
    UserId
);

id_type!(
    /// Unique identifier for beads (issues/tasks)
    BeadId
);

/// Bead status with valid transitions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "bead_status", rename_all = "lowercase")]
pub enum BeadStatus {
  #[default]
  Open,
  InProgress,
  Blocked,
  Deferred,
  Closed,
}

impl BeadStatus {
  #[must_use]
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::Open => "open",
      Self::InProgress => "in_progress",
      Self::Blocked => "blocked",
      Self::Deferred => "deferred",
      Self::Closed => "closed",
    }
  }

  #[must_use]
  #[expect(
    clippy::match_like_matches_macro,
    reason = "Explicit match keeps transition table easy to audit"
  )]
  pub const fn can_transition_to(&self, to: Self) -> bool {
    match (*self, to) {
      (Self::Open, Self::Open)
      | (Self::InProgress, Self::InProgress)
      | (Self::Blocked, Self::Blocked)
      | (Self::Deferred, Self::Deferred)
      | (Self::Closed, Self::Closed) => true,
      (Self::Open, Self::InProgress | Self::Blocked | Self::Deferred | Self::Closed) => true,
      (Self::InProgress, Self::Blocked | Self::Closed) => true,
      (Self::Blocked, Self::Open | Self::InProgress | Self::Deferred) => true,
      (Self::Deferred, Self::Open) => true,
      (Self::Closed, Self::Open) => true,
      _ => false,
    }
  }
}

impl fmt::Display for BeadStatus {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.as_str())
  }
}

impl std::str::FromStr for BeadStatus {
  type Err = ValidationError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "open" => Ok(Self::Open),
      "in_progress" => Ok(Self::InProgress),
      "blocked" => Ok(Self::Blocked),
      "deferred" => Ok(Self::Deferred),
      "closed" => Ok(Self::Closed),
      _ => Err(ValidationError::InvalidStatus(s.to_string())),
    }
  }
}

/// Bead type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "bead_type", rename_all = "lowercase")]
pub enum BeadType {
  #[default]
  Feature,
  Bugfix,
  Refactor,
  Test,
  Docs,
}

impl BeadType {
  #[must_use]
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::Feature => "feature",
      Self::Bugfix => "bugfix",
      Self::Refactor => "refactor",
      Self::Test => "test",
      Self::Docs => "docs",
    }
  }
}

impl fmt::Display for BeadType {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.as_str())
  }
}

impl std::str::FromStr for BeadType {
  type Err = ValidationError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "feature" => Ok(Self::Feature),
      "bugfix" => Ok(Self::Bugfix),
      "refactor" => Ok(Self::Refactor),
      "test" => Ok(Self::Test),
      "docs" => Ok(Self::Docs),
      _ => Err(ValidationError::InvalidType(s.to_string())),
    }
  }
}

/// Bead priority - semantic enum per Scott Wlaschin DDD principles
///
/// Makes illegal states unrepresentable by using an enum instead of primitive integers.
/// Higher priority items should be addressed first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "bead_priority", rename_all = "lowercase")]
pub enum BeadPriority {
  /// High priority - urgent issues that need immediate attention
  High,
  /// Medium priority - standard priority (default)
  #[default]
  Medium,
  /// Low priority - nice to have, can be deferred
  Low,
}

impl BeadPriority {
  /// Get the priority as a lowercase string
  #[must_use]
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::High => "high",
      Self::Medium => "medium",
      Self::Low => "low",
    }
  }

  /// Get numeric value for sorting (1=high, 2=medium, 3=low)
  #[must_use]
  pub const fn sort_value(&self) -> i16 {
    match self {
      Self::High => 1,
      Self::Medium => 2,
      Self::Low => 3,
    }
  }

  /// Create priority from numeric value (for backward compatibility)
  ///
  /// # Errors
  /// Returns `ValidationError` if value is not 1, 2, or 3
  pub const fn from_value(value: i16) -> Result<Self, ValidationError> {
    match value {
      1 => Ok(Self::High),
      2 => Ok(Self::Medium),
      3 => Ok(Self::Low),
      _ => Err(ValidationError::InvalidPriority(value)),
    }
  }

  /// Check if this is high priority
  #[must_use]
  pub const fn is_high(&self) -> bool {
    matches!(self, Self::High)
  }

  /// Check if this is medium priority
  #[must_use]
  pub const fn is_medium(&self) -> bool {
    matches!(self, Self::Medium)
  }

  /// Check if this is low priority
  #[must_use]
  pub const fn is_low(&self) -> bool {
    matches!(self, Self::Low)
  }
}

impl fmt::Display for BeadPriority {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.as_str())
  }
}

impl std::str::FromStr for BeadPriority {
  type Err = ValidationError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "high" => Ok(Self::High),
      "medium" => Ok(Self::Medium),
      "low" => Ok(Self::Low),
      _ => Err(ValidationError::InvalidPriorityString(s.to_string())),
    }
  }
}

impl TryFrom<i16> for BeadPriority {
  type Error = ValidationError;

  fn try_from(value: i16) -> Result<Self, Self::Error> {
    Self::from_value(value)
  }
}

// Legacy support constants (deprecated)
impl BeadPriority {
  /// Legacy constant - use BeadPriority::High instead
  #[deprecated(since = "0.1.0", note = "Use BeadPriority::High instead")]
  pub const HIGH: Self = Self::High;

  /// Legacy constant - use BeadPriority::Medium instead
  #[deprecated(since = "0.1.0", note = "Use BeadPriority::Medium instead")]
  pub const MEDIUM: Self = Self::Medium;

  /// Legacy constant - use BeadPriority::Low instead
  #[deprecated(since = "0.1.0", note = "Use BeadPriority::Low instead")]
  pub const LOW: Self = Self::Low;
}

/// Validation errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
  InvalidUuid(String),
  InvalidStatus(String),
  InvalidType(String),
  InvalidPriority(i16),
  InvalidPriorityString(String),
}

impl fmt::Display for ValidationError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::InvalidUuid(uuid) => write!(f, "Invalid UUID format: {uuid}"),
      Self::InvalidStatus(status) => write!(f, "Invalid status: {status}"),
      Self::InvalidType(t) => write!(f, "Invalid type: {t}"),
      Self::InvalidPriority(p) => write!(
        f,
        "Invalid priority: {p}. Must be 1 (high), 2 (medium), or 3 (low)"
      ),
      Self::InvalidPriorityString(s) => write!(
        f,
        "Invalid priority: {s}. Must be 'high', 'medium', or 'low'"
      ),
    }
  }
}

impl std::error::Error for ValidationError {}

#[cfg(test)]
mod tests {
  use std::str::FromStr;

  use super::*;

  #[test]
  fn test_user_id_new_unique() {
    let id1 = UserId::new();
    let id2 = UserId::new();
    assert_ne!(id1, id2);
  }

  #[test]
  fn test_user_id_from_str() {
    let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
    let result = UserId::from_str(uuid_str);
    assert!(result.is_ok());
    let id = result.map_or_else(|_| Uuid::nil(), |id| id.as_uuid());
    assert_eq!(id.to_string(), uuid_str);
  }

  #[test]
  fn test_bead_id_new_unique() {
    let id1 = BeadId::new();
    let id2 = BeadId::new();
    assert_ne!(id1, id2);
  }

  #[test]
  fn test_bead_priority_from_value_valid() {
    assert_eq!(BeadPriority::from_value(1), Ok(BeadPriority::High));
    assert_eq!(BeadPriority::from_value(2), Ok(BeadPriority::Medium));
    assert_eq!(BeadPriority::from_value(3), Ok(BeadPriority::Low));
  }

  #[test]
  fn test_bead_priority_from_value_invalid() {
    assert!(BeadPriority::from_value(0).is_err());
    assert!(BeadPriority::from_value(4).is_err());
  }

  #[test]
  fn test_bead_priority_from_str() {
    assert_eq!(BeadPriority::from_str("high"), Ok(BeadPriority::High));
    assert_eq!(BeadPriority::from_str("medium"), Ok(BeadPriority::Medium));
    assert_eq!(BeadPriority::from_str("low"), Ok(BeadPriority::Low));
    assert!(BeadPriority::from_str("invalid").is_err());
  }

  #[test]
  fn test_bead_priority_sort_value() {
    assert_eq!(BeadPriority::High.sort_value(), 1);
    assert_eq!(BeadPriority::Medium.sort_value(), 2);
    assert_eq!(BeadPriority::Low.sort_value(), 3);
  }

  #[test]
  fn test_bead_status_transitions() {
    assert!(BeadStatus::Open.can_transition_to(BeadStatus::InProgress));
    assert!(BeadStatus::Open.can_transition_to(BeadStatus::Closed));
    assert!(!BeadStatus::Closed.can_transition_to(BeadStatus::InProgress));
    assert!(BeadStatus::Closed.can_transition_to(BeadStatus::Open));
  }

  #[test]
  fn test_bead_type_from_str() {
    assert_eq!(
      BeadType::from_str("feature").map_or(BeadType::Feature, |t| t),
      BeadType::Feature
    );
    assert_eq!(
      BeadType::from_str("bugfix").map_or(BeadType::Feature, |t| t),
      BeadType::Bugfix
    );
    assert!(BeadType::from_str("invalid").is_err());
  }
}
