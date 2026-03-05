//! Pure core functions for CUE validation and processing.
//!
//! This module contains the functional core for CUE operations, following
//! Scott Wlaschin's DDD principle of separating pure logic from I/O.
//!
//! ## Design Principles
//!
//! - **Pure functions**: No I/O, no side effects, deterministic
//! - **Testable**: All functions can be unit tested without mocks
//! - **Composable**: Functions can be combined in pipelines
//!
//! ## Architecture
//!
//! ```text
//! Shell (I/O) -> Core (Pure) -> Shell (I/O)
//!    cue.rs    ->  cue_core.rs  ->  cue.rs
//! ```

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use std::path::Path;

/// Result of validating a command output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutput {
  /// Whether the command succeeded (exit code 0)
  pub success: bool,
  /// The exit code, if available
  pub exit_code: Option<i32>,
  /// Standard output content
  pub stdout: String,
  /// Standard error content
  pub stderr: String,
}

impl CommandOutput {
  /// Create a successful command output.
  #[must_use]
  pub const fn success(stdout: String) -> Self {
    Self {
      success: true,
      exit_code: Some(0),
      stdout,
      stderr: String::new(),
    }
  }

  /// Create a failed command output.
  #[must_use]
  pub const fn failure(exit_code: i32, stderr: String) -> Self {
    Self {
      success: false,
      exit_code: Some(exit_code),
      stdout: String::new(),
      stderr,
    }
  }

  /// Create from raw exit code and output.
  #[must_use]
  pub fn from_raw(exit_code: i32, stdout: String, stderr: String) -> Self {
    Self {
      success: exit_code == 0,
      exit_code: Some(exit_code),
      stdout,
      stderr,
    }
  }
}

/// Result of checking if a binary is available.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryCheck {
  /// Binary is available and working
  Available,
  /// Binary exists but returned an error
  ExecutionError { message: String },
  /// Binary not found in PATH
  NotFound,
}

impl BinaryCheck {
  /// Check if the binary is available.
  #[must_use]
  pub const fn is_available(&self) -> bool {
    matches!(self, Self::Available)
  }
}

/// Validate that a path string is non-empty and well-formed.
///
/// This is a pure function that checks the path without touching the filesystem.
///
/// # Arguments
///
/// * `path_str` - The path string to validate
///
/// # Returns
///
/// `Some(())` if valid, `None` if invalid
#[must_use]
pub fn validate_path_string(path_str: &str) -> Option<()> {
  if path_str.trim().is_empty() {
    return None;
  }
  // Check for null bytes (security issue)
  if path_str.contains('\0') {
    return None;
  }
  Some(())
}

/// Build the arguments for `cue vet` command.
///
/// This is a pure function that constructs command arguments.
///
/// # Arguments
///
/// * `path_str` - The path to the CUE file
///
/// # Returns
///
/// A vector of arguments for the `cue vet` command
#[must_use]
pub fn build_vet_args(path_str: &str) -> Vec<String> {
  vec!["vet".to_string(), path_str.to_string()]
}

/// Build the arguments for `cue export` command.
///
/// This is a pure function that constructs command arguments.
///
/// # Arguments
///
/// * `path_str` - The path to the CUE file
///
/// # Returns
///
/// A vector of arguments for the `cue export` command
#[must_use]
pub fn build_export_args(path_str: &str) -> Vec<String> {
  vec![
    "export".to_string(),
    path_str.to_string(),
    "-e".to_string(),
    "spec".to_string(),
  ]
}

/// Validate command output for success.
///
/// This is a pure function that checks if a command succeeded.
///
/// # Arguments
///
/// * `output` - The command output to validate
///
/// # Returns
///
/// `Ok(())` if successful, `Err(exit_code, stderr)` if failed
#[must_use]
pub fn validate_command_output(output: &CommandOutput) -> Result<(), (i32, String)> {
  if output.success {
    Ok(())
  } else {
    Err((
      output.exit_code.unwrap_or(-1),
      output.stderr.clone(),
    ))
  }
}

/// Parse UTF-8 output from raw bytes.
///
/// This is a pure function that converts bytes to a string.
///
/// # Arguments
///
/// * `bytes` - The raw bytes to parse
///
/// # Returns
///
/// `Ok(String)` if valid UTF-8, `Err(error_message)` otherwise
#[must_use]
pub fn parse_utf8_output(bytes: &[u8]) -> Result<String, String> {
  String::from_utf8(bytes.to_vec()).map_err(|e| e.to_string())
}

/// Check if a version command output indicates success.
///
/// This is a pure function that checks the version output.
///
/// # Arguments
///
/// * `output` - The command output from `cue version`
///
/// # Returns
///
/// `BinaryCheck` indicating the binary status
#[must_use]
pub fn check_version_output(output: &CommandOutput) -> BinaryCheck {
  if output.success {
    BinaryCheck::Available
  } else {
    BinaryCheck::ExecutionError {
      message: "cue command found but returned error. Ensure CUE is properly installed."
        .to_string(),
    }
  }
}

/// Validate that stdout is valid JSON.
///
/// This is a pure function that checks if a string is valid JSON.
///
/// # Arguments
///
/// * `content` - The string content to validate
///
/// # Returns
///
/// `Ok(())` if valid JSON, `Err(error_message)` otherwise
#[must_use]
pub fn validate_json_content(content: &str) -> Result<(), String> {
  if content.trim().is_empty() {
    return Err("output is empty".to_string());
  }
  serde_json::from_str::<serde_json::Value>(content)
    .map(|_| ())
    .map_err(|e| e.to_string())
}

/// Extract file metadata information from a path string.
///
/// This is a pure function that parses path components without I/O.
///
/// # Arguments
///
/// * `path_str` - The path string to analyze
///
/// # Returns
///
/// A tuple of (directory, filename) if parseable, `None` otherwise
#[must_use]
pub fn extract_path_components(path_str: &str) -> Option<(String, String)> {
  let path = Path::new(path_str);
  let parent = path.parent()?.to_str()?.to_string();
  let filename = path.file_name()?.to_str()?.to_string();
  Some((parent, filename))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn command_output_success_works() {
    let output = CommandOutput::success("hello".to_string());
    assert!(output.success);
    assert_eq!(output.exit_code, Some(0));
    assert_eq!(output.stdout, "hello");
    assert!(output.stderr.is_empty());
  }

  #[test]
  fn command_output_failure_works() {
    let output = CommandOutput::failure(1, "error".to_string());
    assert!(!output.success);
    assert_eq!(output.exit_code, Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(output.stderr, "error");
  }

  #[test]
  fn command_output_from_raw_works() {
    let output = CommandOutput::from_raw(0, "out".to_string(), "err".to_string());
    assert!(output.success);
    assert_eq!(output.stdout, "out");
    assert_eq!(output.stderr, "err");
  }

  #[test]
  fn validate_path_string_accepts_valid() {
    assert!(validate_path_string("/path/to/file.cue").is_some());
    assert!(validate_path_string("relative/path.cue").is_some());
  }

  #[test]
  fn validate_path_string_rejects_empty() {
    assert!(validate_path_string("").is_none());
    assert!(validate_path_string("   ").is_none());
  }

  #[test]
  fn validate_path_string_rejects_null_bytes() {
    assert!(validate_path_string("/path/to\0/file.cue").is_none());
  }

  #[test]
  fn build_vet_args_constructs_correctly() {
    let args = build_vet_args("/path/to/file.cue");
    assert_eq!(args, vec!["vet", "/path/to/file.cue"]);
  }

  #[test]
  fn build_export_args_constructs_correctly() {
    let args = build_export_args("/path/to/file.cue");
    assert_eq!(args, vec!["export", "/path/to/file.cue", "-e", "spec"]);
  }

  #[test]
  fn validate_command_output_succeeds_for_success() {
    let output = CommandOutput::success(String::new());
    assert!(validate_command_output(&output).is_ok());
  }

  #[test]
  fn validate_command_output_fails_for_failure() {
    let output = CommandOutput::failure(1, "error message".to_string());
    let result = validate_command_output(&output);
    assert!(result.is_err());
    let (code, stderr) = result.unwrap_err();
    assert_eq!(code, 1);
    assert_eq!(stderr, "error message");
  }

  #[test]
  fn parse_utf8_output_succeeds_for_valid_utf8() {
    let bytes = b"hello world";
    assert_eq!(parse_utf8_output(bytes), Ok("hello world".to_string()));
  }

  #[test]
  fn parse_utf8_output_fails_for_invalid_utf8() {
    let bytes = &[0xff, 0xfe];
    assert!(parse_utf8_output(bytes).is_err());
  }

  #[test]
  fn check_version_output_available_for_success() {
    let output = CommandOutput::success("cue version 0.4.0".to_string());
    assert_eq!(check_version_output(&output), BinaryCheck::Available);
  }

  #[test]
  fn check_version_output_execution_error_for_failure() {
    let output = CommandOutput::failure(1, "error".to_string());
    let result = check_version_output(&output);
    assert!(matches!(result, BinaryCheck::ExecutionError { .. }));
  }

  #[test]
  fn validate_json_content_succeeds_for_valid_json() {
    assert!(validate_json_content(r#"{"key": "value"}"#).is_ok());
    assert!(validate_json_content(r#"["a", "b"]"#).is_ok());
  }

  #[test]
  fn validate_json_content_fails_for_invalid_json() {
    assert!(validate_json_content("not json").is_err());
    assert!(validate_json_content("{invalid}").is_err());
  }

  #[test]
  fn validate_json_content_fails_for_empty() {
    assert!(validate_json_content("").is_err());
    assert!(validate_json_content("   ").is_err());
  }

  #[test]
  fn extract_path_components_works_for_valid_paths() {
    let result = extract_path_components("/path/to/file.cue");
    assert_eq!(result, Some(("/path/to".to_string(), "file.cue".to_string())));
  }

  #[test]
  fn extract_path_components_fails_for_empty() {
    assert!(extract_path_components("").is_none());
  }

  #[test]
  fn extract_path_components_fails_for_root() {
    assert!(extract_path_components("/").is_none());
  }

  #[test]
  fn binary_check_is_available_works() {
    assert!(BinaryCheck::Available.is_available());
    assert!(!BinaryCheck::NotFound.is_available());
    assert!(!BinaryCheck::ExecutionError { message: String::new() }.is_available());
  }
}
