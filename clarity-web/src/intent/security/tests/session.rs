#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::float_cmp, clippy::needless_collect, clippy::unnecessary_debug_formatting, clippy::match_same_arms, clippy::option_if_let_else, clippy::suspicious_else_formatting, clippy::manual_let_else, clippy::match_wild_err_arm, clippy::match_like_matches_macro)]
use crate::intent::security::{validate_session_id, SecurityError, SessionIdError};

#[test]
fn test_validate_session_id_valid() {
  assert!(validate_session_id("session123").is_ok());
  assert!(validate_session_id("session-123").is_ok());
  assert!(validate_session_id("session_123").is_ok());
  assert!(validate_session_id("SESSION-123_ABC").is_ok());
}

#[test]
fn test_validate_session_id_empty() {
  assert!(matches!(
    validate_session_id(""),
    Err(SecurityError::SessionIdValidation {
      error: SessionIdError::Empty
    })
  ));
}

#[test]
fn test_validate_session_id_too_long() {
  let long_id = "a".repeat(500);
  assert!(matches!(
    validate_session_id(&long_id),
    Err(SecurityError::SessionIdValidation {
      error: SessionIdError::TooLong { max: 499 }
    })
  ));
}

#[test]
fn test_validate_session_id_max_length() {
  let max_id = "a".repeat(499);
  assert!(validate_session_id(&max_id).is_ok());
}

#[test]
fn test_validate_session_id_invalid_char_space() {
  assert!(matches!(
    validate_session_id("session 123"),
    Err(SecurityError::SessionIdValidation {
      error: SessionIdError::InvalidCharacter { ch: ' ' }
    })
  ));
}

#[test]
fn test_validate_session_id_invalid_char_at() {
  assert!(matches!(
    validate_session_id("session@123"),
    Err(SecurityError::SessionIdValidation {
      error: SessionIdError::InvalidCharacter { ch: '@' }
    })
  ));
}

#[test]
fn test_validate_session_id_invalid_char_dot() {
  assert!(matches!(
    validate_session_id("session.123"),
    Err(SecurityError::SessionIdValidation {
      error: SessionIdError::InvalidCharacter { ch: '.' }
    })
  ));
}
