#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use itertools::Itertools;

use super::constants::MAX_SESSION_ID_LENGTH;
use super::helpers::{
  classify_metachar, contains_encoded_traversal, detect_redos_patterns, is_control_character,
  is_shell_metachar, is_valid_session_id_char,
};
use super::types::{MetacharCategory, SecurityError, SecurityResult, SessionIdError};

#[must_use]
pub fn is_safe_path(path: &str) -> bool {
  validate_file_path(path).is_ok()
}

pub fn validate_file_path(path: &str) -> SecurityResult<String> {
  if path.is_empty() {
    return Err(SecurityError::EmptyInput);
  }

  if path.contains('\0') {
    return Err(SecurityError::NullByteDetected);
  }

  if path.contains('\\') {
    return Err(SecurityError::BackslashInPath);
  }

  if path.contains("..") {
    return Err(SecurityError::PathTraversal {
      details: "literal '..' sequence detected".to_owned(),
    });
  }

  if let Some(encoding_type) = contains_encoded_traversal(path) {
    return Err(SecurityError::EncodedPathTraversal { encoding_type });
  }

  path
    .chars()
    .find_map(|ch| {
      if is_control_character(ch) {
        Some(SecurityError::ShellMetacharacter {
          category: MetacharCategory::ControlCharacter,
          ch,
        })
      } else if is_shell_metachar(ch) {
        let category = classify_metachar(ch).map_or(MetacharCategory::EscapeQuote, |found| found);
        Some(SecurityError::ShellMetacharacter { category, ch })
      } else {
        None
      }
    })
    .map_or_else(|| Ok(path.to_owned()), Err)
}

pub fn validate_regex_pattern(pattern: &str) -> SecurityResult<String> {
  if pattern.is_empty() {
    return Err(SecurityError::EmptyInput);
  }

  if pattern.contains('\0') {
    return Err(SecurityError::NullByteDetected);
  }

  detect_redos_patterns(pattern).map_or_else(
    || Ok(pattern.to_owned()),
    |vulnerability| Err(SecurityError::ReDoSVulnerability { vulnerability }),
  )
}

pub fn validate_session_id(session_id: &str) -> SecurityResult<String> {
  if session_id.is_empty() {
    return Err(SecurityError::SessionIdValidation {
      error: SessionIdError::Empty,
    });
  }

  if session_id.len() > MAX_SESSION_ID_LENGTH {
    return Err(SecurityError::SessionIdValidation {
      error: SessionIdError::TooLong {
        max: MAX_SESSION_ID_LENGTH,
      },
    });
  }

  session_id
    .chars()
    .find(|&ch| !is_valid_session_id_char(ch))
    .map_or_else(
      || Ok(session_id.to_owned()),
      |ch| {
        Err(SecurityError::SessionIdValidation {
          error: SessionIdError::InvalidCharacter { ch },
        })
      },
    )
}

pub fn validate_file_paths(paths: &[&str]) -> SecurityResult<Vec<String>> {
  paths
    .iter()
    .map(|&path| validate_file_path(path))
    .try_collect()
}
