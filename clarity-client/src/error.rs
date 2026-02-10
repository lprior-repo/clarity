#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Global error handling for Clarity desktop application
//!
//! This module provides a comprehensive error type system covering all application errors,
//! with user-friendly messages and internal details for logging.

use clarity_core::db::DbError;
use std::fmt::{self, Display};

/// Application error type covering all error cases
///
/// Each error variant has:
/// - A user-facing message (what the user sees)
/// - An internal message (for debugging/logs)
/// - Optional context for recovery actions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppError {
  /// Database connection or query errors
  Database {
    user_message: String,
    internal: String,
    can_retry: bool,
  },

  /// Validation errors for user input
  Validation {
    user_message: String,
    field: Option<String>,
    internal: String,
  },

  /// Navigation/routing errors
  Navigation {
    user_message: String,
    route: Option<String>,
    internal: String,
  },

  /// File I/O errors
  FileSystem {
    user_message: String,
    path: Option<String>,
    internal: String,
  },

  /// Network errors (future use)
  Network {
    user_message: String,
    internal: String,
    can_retry: bool,
  },

  /// Configuration errors
  Configuration {
    user_message: String,
    internal: String,
  },

  /// Permission errors
  Permission {
    user_message: String,
    resource: Option<String>,
    internal: String,
  },

  /// Generic unexpected errors
  Unexpected {
    user_message: String,
    internal: String,
  },
}

impl AppError {
  /// Create a database error
  #[must_use]
  pub fn database<S1, S2>(user_message: S1, internal: S2, can_retry: bool) -> Self
  where
    S1: Into<String>,
    S2: Into<String>,
  {
    Self::Database {
      user_message: user_message.into(),
      internal: internal.into(),
      can_retry,
    }
  }

  /// Create a validation error
  #[must_use]
  pub fn validation<S1, S2>(user_message: S1, internal: S2) -> Self
  where
    S1: Into<String>,
    S2: Into<String>,
  {
    Self::Validation {
      user_message: user_message.into(),
      field: None,
      internal: internal.into(),
    }
  }

  /// Create a validation error with field name
  #[must_use]
  pub fn validation_with_field<S1, S2, S3>(user_message: S1, field: S2, internal: S3) -> Self
  where
    S1: Into<String>,
    S2: Into<String>,
    S3: Into<String>,
  {
    Self::Validation {
      user_message: user_message.into(),
      field: Some(field.into()),
      internal: internal.into(),
    }
  }

  /// Create a navigation error
  #[must_use]
  pub fn navigation<S1, S2>(user_message: S1, internal: S2) -> Self
  where
    S1: Into<String>,
    S2: Into<String>,
  {
    Self::Navigation {
      user_message: user_message.into(),
      route: None,
      internal: internal.into(),
    }
  }

  /// Create a file system error
  #[must_use]
  pub fn file_system<S1, S2>(user_message: S1, internal: S2) -> Self
  where
    S1: Into<String>,
    S2: Into<String>,
  {
    Self::FileSystem {
      user_message: user_message.into(),
      path: None,
      internal: internal.into(),
    }
  }

  /// Create a network error
  #[must_use]
  pub fn network<S1, S2>(user_message: S1, internal: S2, can_retry: bool) -> Self
  where
    S1: Into<String>,
    S2: Into<String>,
  {
    Self::Network {
      user_message: user_message.into(),
      internal: internal.into(),
      can_retry,
    }
  }

  /// Create a configuration error
  #[must_use]
  pub fn configuration<S1, S2>(user_message: S1, internal: S2) -> Self
  where
    S1: Into<String>,
    S2: Into<String>,
  {
    Self::Configuration {
      user_message: user_message.into(),
      internal: internal.into(),
    }
  }

  /// Create a permission error
  #[must_use]
  pub fn permission<S1, S2>(user_message: S1, resource: S2) -> Self
  where
    S1: Into<String>,
    S2: Into<String> + Clone,
  {
    let resource_string = resource.into();
    Self::Permission {
      user_message: user_message.into(),
      resource: Some(resource_string.clone()),
      internal: format!("Permission denied for resource: {resource_string}"),
    }
  }

  /// Create an unexpected error
  #[must_use]
  pub fn unexpected<S1, S2>(user_message: S1, internal: S2) -> Self
  where
    S1: Into<String>,
    S2: Into<String>,
  {
    Self::Unexpected {
      user_message: user_message.into(),
      internal: internal.into(),
    }
  }

  /// Get the user-facing message
  #[must_use]
  pub fn user_message(&self) -> &str {
    match self {
      Self::Database { user_message, .. }
      | Self::Validation { user_message, .. }
      | Self::Navigation { user_message, .. }
      | Self::FileSystem { user_message, .. }
      | Self::Network { user_message, .. }
      | Self::Configuration { user_message, .. }
      | Self::Permission { user_message, .. }
      | Self::Unexpected { user_message, .. } => user_message,
    }
  }

  /// Get the internal message for logging
  #[must_use]
  pub fn internal(&self) -> &str {
    match self {
      Self::Database { internal, .. }
      | Self::Validation { internal, .. }
      | Self::Navigation { internal, .. }
      | Self::FileSystem { internal, .. }
      | Self::Network { internal, .. }
      | Self::Configuration { internal, .. }
      | Self::Permission { internal, .. }
      | Self::Unexpected { internal, .. } => internal,
    }
  }

  /// Check if the error is recoverable (can retry)
  #[must_use]
  pub const fn can_retry(&self) -> bool {
    match self {
      Self::Database { can_retry, .. } | Self::Network { can_retry, .. } => *can_retry,
      _ => false,
    }
  }

  /// Check if the error is critical (should show detailed info)
  #[must_use]
  pub const fn is_critical(&self) -> bool {
    matches!(
      self,
      Self::Database { .. } | Self::FileSystem { .. } | Self::Permission { .. }
    )
  }

  /// Get suggested recovery actions
  #[must_use]
  pub fn recovery_actions(&self) -> Vec<RecoveryAction> {
    match self {
      Self::Database {
        can_retry: true, ..
      } => vec![RecoveryAction::Retry, RecoveryAction::GoHome],
      Self::Database { .. } => vec![RecoveryAction::GoHome],
      Self::Validation { .. } => vec![RecoveryAction::GoBack, RecoveryAction::ContactSupport],
      Self::Navigation { .. } => vec![RecoveryAction::GoHome, RecoveryAction::Retry],
      Self::FileSystem { .. } => vec![RecoveryAction::GoHome, RecoveryAction::ContactSupport],
      Self::Network {
        can_retry: true, ..
      } => vec![RecoveryAction::Retry, RecoveryAction::GoHome],
      Self::Network { .. } => vec![RecoveryAction::GoHome, RecoveryAction::ContactSupport],
      Self::Configuration { .. } => vec![RecoveryAction::GoHome, RecoveryAction::ContactSupport],
      Self::Permission { .. } => vec![RecoveryAction::GoHome, RecoveryAction::ContactSupport],
      Self::Unexpected { .. } => vec![RecoveryAction::GoHome, RecoveryAction::ContactSupport],
    }
  }
}

impl Display for AppError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", self.user_message())
  }
}

impl std::error::Error for AppError {}

/// Convert from database errors
impl From<DbError> for AppError {
  fn from(err: DbError) -> Self {
    match err {
            DbError::Connection(e) => Self::database(
                "Unable to connect to the database. Please check if the application is properly configured.",
                format!("Database connection error: {e}"),
                true,
            ),
            DbError::Migration(e) => Self::database(
                "Database setup failed. Please restart the application.",
                format!("Database migration error: {e}"),
                true,
            ),
            DbError::NotFound { entity, id } => Self::database(
                format!("The requested {entity} could not be found."),
                format!("Record not found: {entity} with id '{id}'"),
                false,
            ),
            DbError::Validation(msg) => Self::validation(
                "Please check your input and try again.",
                format!("Validation error: {msg}"),
            ),
            DbError::Duplicate(msg) => Self::database(
                "This record already exists.",
                format!("Duplicate record error: {msg}"),
                false,
            ),
            DbError::InvalidUuid(id) => Self::validation(
                "Invalid ID format. Please check your link.",
                format!("Invalid UUID: {id}"),
            ),
            DbError::InvalidEmail(email) => Self::validation_with_field(
                "Invalid email address format.",
                "email",
                format!("Invalid email: {email}"),
            ),
            DbError::BundledDbExtraction(msg) => Self::database(
                "Failed to initialize application database.",
                format!("Bundled DB extraction error: {msg}"),
                true,
            ),
            DbError::BundledDbConnection(msg) => Self::database(
                "Failed to connect to application database.",
                format!("Bundled DB connection error: {msg}"),
                true,
            ),
        }
  }
}

/// Convert from I/O errors
impl From<std::io::Error> for AppError {
  fn from(err: std::io::Error) -> Self {
    match err.kind() {
      std::io::ErrorKind::NotFound => Self::file_system(
        "The required file or directory was not found.",
        format!("File not found: {err}"),
      ),
      std::io::ErrorKind::PermissionDenied => Self::permission(
        "You don't have permission to access this resource.",
        err.to_string(),
      ),
      std::io::ErrorKind::ConnectionRefused
      | std::io::ErrorKind::ConnectionReset
      | std::io::ErrorKind::ConnectionAborted => Self::network(
        "Network connection failed. Please check your internet connection.",
        format!("Network error: {err}"),
        true,
      ),
      _ => Self::file_system(
        "An error occurred while accessing files.",
        format!("I/O error: {err}"),
      ),
    }
  }
}

/// Convert from anyhow errors (used in shell/imperative code)
impl From<anyhow::Error> for AppError {
  fn from(err: anyhow::Error) -> Self {
    Self::unexpected(
      "An unexpected error occurred. Please try again.",
      format!("Unexpected error: {err:?}"),
    )
  }
}

/// Recovery action buttons for error UI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryAction {
  /// Retry the failed operation
  Retry,

  /// Go back to the previous page
  GoBack,

  /// Go to the home page
  GoHome,

  /// Contact support
  ContactSupport,
}

impl RecoveryAction {
  /// Get the display label for this action
  #[must_use]
  pub const fn label(&self) -> &str {
    match self {
      Self::Retry => "Try Again",
      Self::GoBack => "Go Back",
      Self::GoHome => "Go Home",
      Self::ContactSupport => "Contact Support",
    }
  }
}

/// Result type for application operations
pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_database_error_messages() {
    let err = AppError::database("User sees this", "Internal: connection failed", true);
    assert_eq!(err.user_message(), "User sees this");
    assert_eq!(err.internal(), "Internal: connection failed");
    assert!(err.can_retry());
    assert!(err.is_critical());
  }

  #[test]
  fn test_validation_error_messages() {
    let err = AppError::validation("Invalid input", "Field validation failed");
    assert_eq!(err.user_message(), "Invalid input");
    assert_eq!(err.internal(), "Field validation failed");
    assert!(!err.can_retry());
    assert!(!err.is_critical());
  }

  #[test]
  fn test_validation_error_with_field() {
    let err = AppError::validation_with_field("Invalid email", "email", "Email format invalid");
    assert_eq!(err.user_message(), "Invalid email");
    assert!(!err.can_retry());
  }

  #[test]
  fn test_navigation_error_messages() {
    let err = AppError::navigation("Route not found", "Invalid path");
    assert_eq!(err.user_message(), "Route not found");
    assert_eq!(err.internal(), "Invalid path");
    assert!(!err.can_retry());
    assert!(!err.is_critical());
  }

  #[test]
  fn test_file_system_error_messages() {
    let err = AppError::file_system("Cannot save file", "Disk full");
    assert_eq!(err.user_message(), "Cannot save file");
    assert_eq!(err.internal(), "Disk full");
    assert!(!err.can_retry());
    assert!(err.is_critical());
  }

  #[test]
  fn test_network_error_messages() {
    let err = AppError::network("Connection failed", "Timeout after 30s", true);
    assert_eq!(err.user_message(), "Connection failed");
    assert_eq!(err.internal(), "Timeout after 30s");
    assert!(err.can_retry());
    assert!(!err.is_critical());
  }

  #[test]
  fn test_configuration_error_messages() {
    let err = AppError::configuration("Invalid configuration", "Missing DATABASE_URL");
    assert_eq!(err.user_message(), "Invalid configuration");
    assert_eq!(err.internal(), "Missing DATABASE_URL");
    assert!(!err.can_retry());
    assert!(!err.is_critical());
  }

  #[test]
  fn test_permission_error_messages() {
    let err = AppError::permission("Access denied", "/etc/hosts");
    assert_eq!(err.user_message(), "Access denied");
    assert!(!err.can_retry());
    assert!(err.is_critical());
  }

  #[test]
  fn test_unexpected_error_messages() {
    let err = AppError::unexpected("Something went wrong", "Panic in async task");
    assert_eq!(err.user_message(), "Something went wrong");
    assert_eq!(err.internal(), "Panic in async task");
    assert!(!err.can_retry());
    assert!(!err.is_critical());
  }

  #[test]
  fn test_recovery_actions_for_database_error() {
    let retryable = AppError::database("Error", "Internal", true);
    assert_eq!(
      retryable.recovery_actions(),
      vec![RecoveryAction::Retry, RecoveryAction::GoHome]
    );

    let non_retryable = AppError::database("Error", "Internal", false);
    assert_eq!(
      non_retryable.recovery_actions(),
      vec![RecoveryAction::GoHome]
    );
  }

  #[test]
  fn test_recovery_actions_for_validation_error() {
    let err = AppError::validation("Invalid", "Validation failed");
    assert_eq!(
      err.recovery_actions(),
      vec![RecoveryAction::GoBack, RecoveryAction::ContactSupport]
    );
  }

  #[test]
  fn test_recovery_actions_for_navigation_error() {
    let err = AppError::navigation("Not found", "Invalid route");
    assert_eq!(
      err.recovery_actions(),
      vec![RecoveryAction::GoHome, RecoveryAction::Retry]
    );
  }

  #[test]
  fn test_recovery_action_labels() {
    assert_eq!(RecoveryAction::Retry.label(), "Try Again");
    assert_eq!(RecoveryAction::GoBack.label(), "Go Back");
    assert_eq!(RecoveryAction::GoHome.label(), "Go Home");
    assert_eq!(RecoveryAction::ContactSupport.label(), "Contact Support");
  }

  #[test]
  fn test_display_shows_user_message() {
    let err = AppError::validation("User message", "Internal");
    assert_eq!(format!("{err}"), "User message");
  }

  #[test]
  fn test_from_io_error_not_found() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file.txt");
    let app_err = AppError::from(io_err);
    assert_eq!(
      app_err.user_message(),
      "The required file or directory was not found."
    );
    assert!(app_err.is_critical());
  }

  #[test]
  fn test_from_io_error_permission_denied() {
    let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
    let app_err = AppError::from(io_err);
    assert_eq!(
      app_err.user_message(),
      "You don't have permission to access this resource."
    );
    assert!(app_err.is_critical());
  }

  #[test]
  fn test_from_io_error_connection_refused() {
    let io_err = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "no server");
    let app_err = AppError::from(io_err);
    assert_eq!(
      app_err.user_message(),
      "Network connection failed. Please check your internet connection."
    );
    assert!(app_err.can_retry());
  }
}
