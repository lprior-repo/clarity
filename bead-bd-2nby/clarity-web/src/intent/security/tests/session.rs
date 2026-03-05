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
