#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

/// Database operation errors
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("Database connection error: {0}")]
    Connection(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Record not found: {entity} with id '{id}'")]
    NotFound { entity: String, id: String },

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Duplicate record: {0}")]
    Duplicate(String),

    #[error("Invalid UUID: {0}")]
    InvalidUuid(String),

    #[error("Invalid email format: {0}")]
    InvalidEmail(String),
}

impl DbError {
    /// Create a not found error for an entity
    pub fn not_found(entity: impl Into<String>, id: impl Into<String>) -> Self {
        Self::NotFound {
            entity: entity.into(),
            id: id.into(),
        }
    }

    /// Create a validation error
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    /// Create a duplicate error
    pub fn duplicate(msg: impl Into<String>) -> Self {
        Self::Duplicate(msg.into())
    }
}

/// Result type for database operations
pub type DbResult<T> = Result<T, DbError>;
