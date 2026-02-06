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

/// User identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub Uuid);

impl UserId {
    /// Create a new random UserId
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create from Uuid string
    pub fn from_str(s: &str) -> DbResult<Self> {
        Uuid::parse_str(s)
            .map(Self)
            .map_err(|_| DbError::InvalidUuid(s.to_string()))
    }

    /// Get underlying Uuid
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Bead identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BeadId(pub Uuid);

impl BeadId {
    /// Create a new random BeadId
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create from Uuid string
    pub fn from_str(s: &str) -> DbResult<Self> {
        Uuid::parse_str(s)
            .map(Self)
            .map_err(|_| DbError::InvalidUuid(s.to_string()))
    }

    /// Get underlying Uuid
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl std::fmt::Display for BeadId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Email address with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Email(String);

impl Email {
    /// Create a new Email with validation
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
    pub fn as_str(&self) -> &str {
        &self.0
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

/// Bead priority (1 = high, 2 = medium, 3 = low)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BeadPriority(pub i16);

impl BeadPriority {
    pub const HIGH: Self = Self(1);
    pub const MEDIUM: Self = Self(2);
    pub const LOW: Self = Self(3);

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
