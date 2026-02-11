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
        pub struct $name(Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            pub fn from_str(s: &str) -> Result<Self, ValidationError> {
                Uuid::parse_str(s)
                    .map(Self)
                    .map_err(|_| ValidationError::InvalidUuid(s.to_string()))
            }

            pub const fn as_uuid(&self) -> Uuid {
                self.0
            }

            pub fn as_str(&self) -> String {
                self.0.to_string()
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

/// Email address with validation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Email(String);

impl Email {
    pub fn new(email: String) -> Result<Self, ValidationError> {
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 {
            return Err(ValidationError::InvalidEmail(email));
        }

        let local = parts[0];
        let domain = parts[1];

        if local.is_empty() || domain.is_empty() {
            return Err(ValidationError::InvalidEmail(email));
        }

        if !domain.contains('.') || domain.ends_with('.') || domain.starts_with('.') {
            return Err(ValidationError::InvalidEmail(email));
        }

        let domain_parts: Vec<&str> = domain.split('.').collect();
        if domain_parts.len() < 2 || domain_parts[0].is_empty() {
            return Err(ValidationError::InvalidEmail(email));
        }

        if domain_parts.last().map_or(true, |s| s.is_empty()) {
            return Err(ValidationError::InvalidEmail(email));
        }

        Ok(Self(email.to_lowercase()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn local_part(&self) -> &str {
        self.0.split('@').next().unwrap_or("")
    }

    pub fn domain(&self) -> &str {
        self.0.split('@').nth(1).unwrap_or("")
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for Email {
    type Error = ValidationError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::new(s)
    }
}

impl TryFrom<&str> for Email {
    type Error = ValidationError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::new(s.to_string())
    }
}

/// User role for authorization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    User,
}

impl UserRole {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::User => "user",
        }
    }
}

impl fmt::Display for UserRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Default for UserRole {
    fn default() -> Self {
        Self::User
    }
}

impl std::str::FromStr for UserRole {
    type Err = ValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "admin" => Ok(Self::Admin),
            "user" => Ok(Self::User),
            _ => Err(ValidationError::InvalidRole(s.to_string())),
        }
    }
}

/// Bead status with valid transitions
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
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InProgress => "in_progress",
            Self::Blocked => "blocked",
            Self::Deferred => "deferred",
            Self::Closed => "closed",
        }
    }

    pub const fn can_transition_to(&self, to: Self) -> bool {
        match (*self, to) {
            (s1, s2) if s1 as i32 == s2 as i32 => true,
            (Self::Open, Self::InProgress) => true,
            (Self::Open, Self::Blocked) => true,
            (Self::Open, Self::Deferred) => true,
            (Self::Open, Self::Closed) => true,
            (Self::InProgress, Self::Blocked) => true,
            (Self::InProgress, Self::Closed) => true,
            (Self::Blocked, Self::Open) => true,
            (Self::Blocked, Self::InProgress) => true,
            (Self::Blocked, Self::Deferred) => true,
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

impl Default for BeadStatus {
    fn default() -> Self {
        Self::Open
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

impl Default for BeadType {
    fn default() -> Self {
        Self::Feature
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

/// Bead priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BeadPriority(i16);

impl BeadPriority {
    pub const HIGH: Self = Self(1);
    pub const MEDIUM: Self = Self(2);
    pub const LOW: Self = Self(3);

    pub fn new(priority: i16) -> Result<Self, ValidationError> {
        match priority {
            1 | 2 | 3 => Ok(Self(priority)),
            _ => Err(ValidationError::InvalidPriority(priority)),
        }
    }

    pub const fn value(&self) -> i16 {
        self.0
    }

    pub const fn is_high(&self) -> bool {
        self.0 == 1
    }

    pub const fn is_medium(&self) -> bool {
        self.0 == 2
    }

    pub const fn is_low(&self) -> bool {
        self.0 == 3
    }

    pub const fn as_str(&self) -> &'static str {
        match self.0 {
            1 => "high",
            2 => "medium",
            3 => "low",
            _ => "unknown",
        }
    }
}

impl fmt::Display for BeadPriority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl Default for BeadPriority {
    fn default() -> Self {
        Self::MEDIUM
    }
}

impl TryFrom<i16> for BeadPriority {
    type Error = ValidationError;

    fn try_from(value: i16) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

/// Validation errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    InvalidUuid(String),
    InvalidEmail(String),
    InvalidRole(String),
    InvalidStatus(String),
    InvalidType(String),
    InvalidPriority(i16),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUuid(uuid) => write!(f, "Invalid UUID format: {}", uuid),
            Self::InvalidEmail(email) => write!(f, "Invalid email format: {}", email),
            Self::InvalidRole(role) => write!(f, "Invalid role: {}", role),
            Self::InvalidStatus(status) => write!(f, "Invalid status: {}", status),
            Self::InvalidType(t) => write!(f, "Invalid type: {}", t),
            Self::InvalidPriority(p) => write!(
                f,
                "Invalid priority: {}. Must be 1 (high), 2 (medium), or 3 (low)",
                p
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
        assert_eq!(result.unwrap().as_uuid().to_string(), uuid_str);
    }

    #[test]
    fn test_email_valid() {
        let valid = vec![
            "user@example.com",
            "test.user@domain.co.uk",
            "user+tag@example.com",
        ];

        for email in valid {
            assert!(
                Email::new(email.to_string()).is_ok(),
                "{} should be valid",
                email
            );
        }
    }

    #[test]
    fn test_email_invalid() {
        let invalid = vec!["notanemail", "@example.com", "user@", "user@.com", ""];

        for email in invalid {
            assert!(
                Email::new(email.to_string()).is_err(),
                "{} should be invalid",
                email
            );
        }
    }

    #[test]
    fn test_email_normalizes_case() {
        let email = Email::new("USER@EXAMPLE.COM".to_string()).unwrap();
        assert_eq!(email.as_str(), "user@example.com");
    }

    #[test]
    fn test_bead_priority_valid() {
        assert!(BeadPriority::new(1).is_ok());
        assert!(BeadPriority::new(2).is_ok());
        assert!(BeadPriority::new(3).is_ok());
    }

    #[test]
    fn test_bead_priority_invalid() {
        assert!(BeadPriority::new(0).is_err());
        assert!(BeadPriority::new(4).is_err());
    }

    #[test]
    fn test_bead_status_transitions() {
        assert!(BeadStatus::Open.can_transition_to(BeadStatus::InProgress));
        assert!(BeadStatus::Open.can_transition_to(BeadStatus::Closed));
        assert!(!BeadStatus::Closed.can_transition_to(BeadStatus::InProgress));
        assert!(BeadStatus::Closed.can_transition_to(BeadStatus::Open));
    }

    #[test]
    fn test_user_role_from_str() {
        assert_eq!(UserRole::from_str("admin").unwrap(), UserRole::Admin);
        assert_eq!(UserRole::from_str("ADMIN").unwrap(), UserRole::Admin);
        assert_eq!(UserRole::from_str("user").unwrap(), UserRole::User);
        assert!(UserRole::from_str("invalid").is_err());
    }

    #[test]
    fn test_bead_type_from_str() {
        assert_eq!(BeadType::from_str("feature").unwrap(), BeadType::Feature);
        assert_eq!(BeadType::from_str("bugfix").unwrap(), BeadType::Bugfix);
        assert!(BeadType::from_str("invalid").is_err());
    }
}
