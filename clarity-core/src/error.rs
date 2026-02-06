#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

//! Exit code system for Clarity CLI
//!
//! This module defines exit codes following POSIX conventions:
//! - 0: Success
//! - 1-125: Application-specific errors
//! - 126: Command invoked cannot execute
//! - 127: Command not found
//! - 128+: Signal termination (128 + signal number)
//!
//! All exit codes are strongly typed and mapped from domain errors.

use std::fmt::{self, Display};

/// Exit code for CLI processes
///
/// Exit codes are constrained to 0-255 as per POSIX standard.
/// Use [`ExitCode::as_u8`] to get the raw value for process termination.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExitCode(u8);

impl ExitCode {
    /// Success - program executed successfully
    pub const SUCCESS: Self = Self(0);

    /// General error - catch-all for unspecified errors
    pub const ERROR: Self = Self(1);

    /// Invalid usage - wrong command-line arguments or options
    pub const USAGE: Self = Self(2);

    /// Input/output error
    pub const IO_ERROR: Self = Self(3);

    /// Configuration error
    pub const CONFIG_ERROR: Self = Self(4);

    /// Validation error
    pub const VALIDATION_ERROR: Self = Self(5);

    /// Network error
    pub const NETWORK_ERROR: Self = Self(6);

    /// Permission denied
    pub const PERMISSION_DENIED: Self = Self(7);

    /// File not found
    pub const NOT_FOUND: Self = Self(8);

    /// Create a new ExitCode, ensuring it's within 0-255
    ///
    /// # Errors
    ///
    /// Returns `ExitCodeError::OutOfRange` if value > 255
    pub fn new(code: u32) -> Result<Self, ExitCodeError> {
        code.try_into()
            .map(Self)
            .map_err(|_| ExitCodeError::OutOfRange(code))
    }

    /// Get the raw u8 value for process termination
    #[must_use]
    pub const fn as_u8(self) -> u8 {
        self.0
    }

    /// Check if this exit code indicates success
    #[must_use]
    pub const fn is_success(self) -> bool {
        self.0 == 0
    }

    /// Check if this exit code indicates failure
    #[must_use]
    pub const fn is_failure(self) -> bool {
        self.0 != 0
    }
}

impl Display for ExitCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "exit code {}", self.0)
    }
}

/// Errors that can occur when creating an ExitCode
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExitCodeError {
    /// Value exceeds 255 (maximum valid exit code)
    OutOfRange(u32),
}

impl Display for ExitCodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfRange(value) => {
                write!(f, "exit code {value} out of range (must be 0-255)")
            }
        }
    }
}

impl std::error::Error for ExitCodeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_constant() {
        assert_eq!(ExitCode::SUCCESS.as_u8(), 0);
        assert!(ExitCode::SUCCESS.is_success());
        assert!(!ExitCode::SUCCESS.is_failure());
    }

    #[test]
    fn test_error_constant() {
        assert_eq!(ExitCode::ERROR.as_u8(), 1);
        assert!(!ExitCode::ERROR.is_success());
        assert!(ExitCode::ERROR.is_failure());
    }

    #[test]
    fn test_usage_constant() {
        assert_eq!(ExitCode::USAGE.as_u8(), 2);
    }

    #[test]
    fn test_io_error_constant() {
        assert_eq!(ExitCode::IO_ERROR.as_u8(), 3);
    }

    #[test]
    fn test_config_error_constant() {
        assert_eq!(ExitCode::CONFIG_ERROR.as_u8(), 4);
    }

    #[test]
    fn test_validation_error_constant() {
        assert_eq!(ExitCode::VALIDATION_ERROR.as_u8(), 5);
    }

    #[test]
    fn test_network_error_constant() {
        assert_eq!(ExitCode::NETWORK_ERROR.as_u8(), 6);
    }

    #[test]
    fn test_permission_denied_constant() {
        assert_eq!(ExitCode::PERMISSION_DENIED.as_u8(), 7);
    }

    #[test]
    fn test_not_found_constant() {
        assert_eq!(ExitCode::NOT_FOUND.as_u8(), 8);
    }

    #[test]
    fn test_new_valid_code() {
        assert!(ExitCode::new(0).is_ok());
        assert!(ExitCode::new(255).is_ok());
        assert_eq!(ExitCode::new(42), Ok(ExitCode(42)));
    }

    #[test]
    fn test_new_invalid_code() {
        assert_eq!(ExitCode::new(256), Err(ExitCodeError::OutOfRange(256)));
        assert_eq!(ExitCode::new(1000), Err(ExitCodeError::OutOfRange(1000)));
    }

    #[test]
    fn test_as_u8() {
        let code = ExitCode::new(42).unwrap();
        assert_eq!(code.as_u8(), 42);
    }

    #[test]
    fn test_is_success() {
        assert!(ExitCode::SUCCESS.is_success());
        assert!(!ExitCode::ERROR.is_success());
    }

    #[test]
    fn test_is_failure() {
        assert!(ExitCode::ERROR.is_failure());
        assert!(!ExitCode::SUCCESS.is_failure());
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", ExitCode::SUCCESS), "exit code 0");
        assert_eq!(format!("{}", ExitCode::ERROR), "exit code 1");
        assert_eq!(format!("{}", ExitCode::new(42).unwrap()), "exit code 42");
    }

    #[test]
    fn test_exit_code_error_display() {
        let error = ExitCodeError::OutOfRange(256);
        assert_eq!(
            format!("{}", error),
            "exit code 256 out of range (must be 0-255)"
        );
    }
}
