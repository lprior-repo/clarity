#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::db::error::{DbError, DbResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ===== Domain Types (Newtypes) =====

/// Macro to generate UUID-based ID types with consistent behavior
macro_rules! uuid_id {
  ($(#[$meta:meta])* $name:ident) => {
    $(#[$meta])*
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct $name(pub Uuid);

    impl $name {
      /// Create a new random ID
      pub fn new() -> Self {
        Self(Uuid::new_v4())
      }

      /// Create from Uuid string
      ///
      /// # Errors
      /// Returns `DbError::InvalidUuid` if the string is not a valid UUID
      #[allow(clippy::should_implement_trait)]
      pub fn from_str(s: &str) -> DbResult<Self> {
        Uuid::parse_str(s)
          .map(Self)
          .map_err(|_| DbError::InvalidUuid(s.to_string()))
      }

      /// Get underlying Uuid
      pub const fn as_uuid(&self) -> Uuid {
        self.0
      }
    }

    impl Default for $name {
      fn default() -> Self {
        Self::new()
      }
    }

    impl std::fmt::Display for $name {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

// Apply the macro to generate ID types
uuid_id!(
  /// User identifier
  UserId
);

uuid_id!(
  /// Bead identifier
  BeadId
);

/// Email address with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Email(pub String);

impl Email {
  /// Create a new Email with validation
  ///
  /// # Errors
  /// - Returns `DbError::InvalidEmail` if the email is empty, malformed, or invalid
  pub fn new(email: String) -> DbResult<Self> {
    // Basic email validation:
    // - Must contain exactly one '@'
    // - Must have at least one character before '@'
    // - Must have at least one '.' after '@'
    // - Must have at least one character between '@' and '.'
    // - Must have at least one character after '.'
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 {
      return Err(DbError::InvalidEmail(email));
    }

    let local = parts[0];
    let domain = parts[1];

    if local.is_empty() || domain.is_empty() {
      return Err(DbError::InvalidEmail(email));
    }

    if !domain.contains('.') || domain.ends_with('.') || domain.starts_with('.') {
      return Err(DbError::InvalidEmail(email));
    }

    Ok(Self(email))
  }

  /// Get the email as a string
  #[must_use]
  pub fn as_str(&self) -> &str {
    &self.0
  }
}

impl From<String> for Email {
  fn from(s: String) -> Self {
    Self(s)
  }
}

impl std::fmt::Display for Email {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.0)
  }
}

// ===== Enums =====

/// User role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
  Admin,
  User,
}

impl std::str::FromStr for UserRole {
  type Err = DbError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "admin" => Ok(Self::Admin),
      "user" => Ok(Self::User),
      _ => Err(DbError::InvalidUuid(s.to_string())),
    }
  }
}

/// Bead status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "bead_status", rename_all = "lowercase")]
pub enum BeadStatus {
  Open,
  InProgress,
  Blocked,
  Deferred,
  Closed,
}

impl BeadStatus {
  /// Parse a string into a `BeadStatus`
  ///
  /// # Errors
  /// - Returns `DbError::Validation` if the string is not a valid status
  ///
  /// Get the status as a lowercase string
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
}

impl std::fmt::Display for BeadStatus {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.as_str())
  }
}

impl std::str::FromStr for BeadStatus {
  type Err = DbError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "open" => Ok(Self::Open),
      "in_progress" => Ok(Self::InProgress),
      "blocked" => Ok(Self::Blocked),
      "deferred" => Ok(Self::Deferred),
      "closed" => Ok(Self::Closed),
      _ => Err(DbError::validation(format!("Invalid bead status: {s}"))),
    }
  }
}

/// Bead type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "bead_type", rename_all = "lowercase")]
pub enum BeadType {
  Feature,
  Bugfix,
  Refactor,
  Test,
  Docs,
}

impl BeadType {
  /// Parse a string into a `BeadType`
  ///
  /// # Errors
  /// - Returns `DbError::Validation` if the string is not a valid type
  ///
  /// Get the type as a lowercase string
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

impl std::fmt::Display for BeadType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.as_str())
  }
}

impl std::str::FromStr for BeadType {
  type Err = DbError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s.to_lowercase().as_str() {
      "feature" => Ok(Self::Feature),
      "bugfix" => Ok(Self::Bugfix),
      "refactor" => Ok(Self::Refactor),
      "test" => Ok(Self::Test),
      "docs" => Ok(Self::Docs),
      _ => Err(DbError::validation(format!("Invalid bead type: {s}"))),
    }
  }
}

/// Bead priority (1 = high, 2 = medium, 3 = low)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BeadPriority(pub i16);

impl BeadPriority {
  pub const HIGH: Self = Self(1);
  pub const MEDIUM: Self = Self(2);
  pub const LOW: Self = Self(3);

  /// Create a new `BeadPriority` with validation
  ///
  /// # Errors
  /// - Returns `DbError::ValidationError` if priority is not 1, 2, or 3
  pub fn new(priority: i16) -> DbResult<Self> {
    if (1..=3).contains(&priority) {
      Ok(Self(priority))
    } else {
      Err(DbError::validation("Priority must be between 1 and 3"))
    }
  }
}

// ===== Domain Models =====

/// User entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
  pub id: UserId,
  pub email: Email,
  pub password_hash: String,
  pub role: UserRole,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

/// New user (without id and timestamps)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewUser {
  pub email: Email,
  pub password_hash: String,
  pub role: UserRole,
}

/// Bead entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bead {
  pub id: BeadId,
  pub title: String,
  pub description: Option<String>,
  pub status: BeadStatus,
  pub priority: BeadPriority,
  pub bead_type: BeadType,
  pub created_by: Option<UserId>,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

/// New bead (without id and timestamps)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewBead {
  pub title: String,
  pub description: Option<String>,
  pub status: BeadStatus,
  pub priority: BeadPriority,
  pub bead_type: BeadType,
  pub created_by: Option<UserId>,
}

/// Interview entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interview {
  pub id: Uuid,
  pub spec_name: String,
  pub questions: serde_json::Value,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}

/// Spec entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spec {
  pub id: Uuid,
  pub name: String,
  pub description: Option<String>,
  pub schema: serde_json::Value,
  pub created_at: DateTime<Utc>,
  pub updated_at: DateTime<Utc>,
}
