#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

//! Security validation module for clarity-core
//!
//! Provides validation utilities with functional error handling.
//! All functions return Result<T, E> - no unwraps, no panics.

use thiserror::Error;

/// Validation errors that can occur
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ValidationError {
    #[error("input cannot be empty")]
    EmptyInput,

    #[error("input exceeds maximum length of {max_length}")]
    InputTooLong { max_length: usize },

    #[error("input contains invalid characters: {chars}")]
    InvalidCharacters { chars: String },

    #[error("input format is invalid: {reason}")]
    InvalidFormat { reason: String },
}

/// Validates that a string is non-empty
///
/// # Examples
///
/// ```
/// use clarity_core::validation::validate_non_empty;
///
/// assert!(validate_non_empty("test").is_ok());
/// assert!(validate_non_empty("").is_err());
/// ```
///
/// # Errors
///
/// Returns `ValidationError::EmptyInput` if the input string is empty
pub const fn validate_non_empty(input: &str) -> Result<&str, ValidationError> {
    if input.is_empty() {
        Err(ValidationError::EmptyInput)
    } else {
        Ok(input)
    }
}

/// Validates that a string does not exceed maximum length
///
/// # Examples
///
/// ```
/// use clarity_core::validation::validate_max_length;
///
/// assert!(validate_max_length("test", 10).is_ok());
/// assert!(validate_max_length("test", 3).is_err());
/// ```
///
/// # Errors
///
/// Returns `ValidationError::InputTooLong` if the input string length exceeds `max_length`
pub const fn validate_max_length(input: &str, max_length: usize) -> Result<&str, ValidationError> {
    if input.len() > max_length {
        Err(ValidationError::InputTooLong { max_length })
    } else {
        Ok(input)
    }
}

/// Validates that a string contains only alphanumeric characters
///
/// # Examples
///
/// ```
/// use clarity_core::validation::validate_alphanumeric;
///
/// assert!(validate_alphanumeric("test123").is_ok());
/// assert!(validate_alphanumeric("test-123").is_err());
/// ```
///
/// # Errors
///
/// Returns `ValidationError::InvalidCharacters` if the input contains non-alphanumeric characters
pub fn validate_alphanumeric(input: &str) -> Result<&str, ValidationError> {
    if input.chars().all(char::is_alphanumeric) {
        Ok(input)
    } else {
        let invalid: String = input.chars().filter(|c| !c.is_alphanumeric()).collect();
        Err(ValidationError::InvalidCharacters { chars: invalid })
    }
}

/// Validates an email address format (basic validation)
///
/// # Examples
///
/// ```
/// use clarity_core::validation::validate_email_format;
///
/// assert!(validate_email_format("test@example.com").is_ok());
/// assert!(validate_email_format("invalid").is_err());
/// ```
///
/// # Errors
///
/// Returns `ValidationError::InvalidFormat` if the email doesn't contain @ and . or is too short
pub fn validate_email_format(input: &str) -> Result<&str, ValidationError> {
    let has_at = input.contains('@');
    let has_dot = input.contains('.');

    if has_at && has_dot && input.len() > 5 {
        Ok(input)
    } else {
        Err(ValidationError::InvalidFormat {
            reason: "Email must contain @ and . and be at least 6 characters".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_non_empty_with_valid_input() {
        let result = validate_non_empty("test");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test");
    }

    #[test]
    fn test_validate_non_empty_with_empty_input() {
        let result = validate_non_empty("");
        assert!(result.is_err());
        assert_eq!(result, Err(ValidationError::EmptyInput));
    }

    #[test]
    fn test_validate_max_length_within_limit() {
        let result = validate_max_length("test", 10);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_max_length_exceeds_limit() {
        let result = validate_max_length("test", 3);
        assert!(result.is_err());
        match result {
            Err(ValidationError::InputTooLong { max_length }) => {
                assert_eq!(max_length, 3);
            }
            _ => panic!("Expected InputTooLong error"),
        }
    }

    #[test]
    fn test_validate_alphanumeric_valid() {
        let result = validate_alphanumeric("test123");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_alphanumeric_invalid() {
        let result = validate_alphanumeric("test-123");
        assert!(result.is_err());
        match result {
            Err(ValidationError::InvalidCharacters { chars }) => {
                assert_eq!(chars, "-");
            }
            _ => panic!("Expected InvalidCharacters error"),
        }
    }

    #[test]
    fn test_validate_email_format_valid() {
        let result = validate_email_format("test@example.com");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_email_format_missing_at() {
        let result = validate_email_format("testexample.com");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_email_format_missing_dot() {
        let result = validate_email_format("test@examplecom");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_email_format_too_short() {
        let result = validate_email_format("t@e.c");
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_chain_valid() {
        let result = validate_non_empty("test123")
            .and_then(|s| validate_max_length(s, 10))
            .and_then(|s| validate_alphanumeric(s));

        assert!(result.is_ok());
    }

    #[test]
    fn test_validation_chain_fails_at_first_step() {
        let result = validate_non_empty("")
            .and_then(|s| validate_max_length(s, 10))
            .and_then(|s| validate_alphanumeric(s));

        assert_eq!(result, Err(ValidationError::EmptyInput));
    }

    #[test]
    fn test_validation_chain_fails_at_second_step() {
        let result = validate_non_empty("this is a very long string")
            .and_then(|s| validate_max_length(s, 10))
            .and_then(|s| validate_alphanumeric(s));

        assert!(result.is_err());
    }
}
