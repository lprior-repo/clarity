//! CUE Loader for Intent Specs (WP18)
//!
//! This module provides CUE file loading and validation functionality,
//! using the `cue` command-line tool to validate and export CUE specs to JSON.
//!
//! ## Design Principles
//!
//! - **Zero panics**: All fallible operations return `Result<T, LoaderError>`
//! - **Zero unwrap/expect**: No panics in production code
//! - **Security first**: All paths are validated before use
//! - **Functional core**: Pure functions for validation and parsing
//!
//! ## Functions
//!
//! - [`load_cue_file`]: Load and validate a CUE file, returning a parsed Spec
//! - [`validate_cue_file`]: Validate a CUE file without parsing
//! - [`export_cue_to_json`]: Export a CUE file to JSON string
//!
//! ## Example
//!
//! ```rust,ignore
//! use clarity_web::intent::loader::load_cue_file;
//! use std::path::Path;
//!
//! let spec = load_cue_file(Path::new("spec.cue"))?;
//! println!("Loaded spec: {}", spec.name);
//! ```

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::process::Command;

use thiserror::Error;

use super::parser::{parse_spec, ParseError};
use super::security::{validate_file_path, SecurityError};
use super::types::Spec;

// =============================================================================
// Error Types
// =============================================================================

/// Error type for CUE loading operations
#[derive(Debug, Error, Clone, PartialEq)]
pub enum LoaderError {
  /// I/O error (file not found, permission denied, etc.)
  #[error("I/O error: {0}")]
  Io(String),

  /// JSON parsing error
  #[error("JSON error: {0}")]
  Json(String),

  /// Command execution failed (cue vet/export)
  #[error("command execution failed: {0}")]
  CommandFailed(String),

  /// Validation error from parser
  #[error("validation error: {0}")]
  Validation(String),

  /// Session not found (for future use)
  #[error("session not found: {0}")]
  SessionNotFound(String),

  /// Invalid spec for field
  #[error("invalid spec for field '{field}': expected {expected}, got {actual}")]
  InvalidSpec {
    /// Field name
    field: String,
    /// Expected type/value
    expected: String,
    /// Actual type/value found
    actual: String,
  },

  /// Empty required field
  #[error("empty required field: {0}")]
  EmptyField(String),

  /// Invalid CUE output
  #[error("invalid CUE output: {0}")]
  InvalidCueOutput(String),

  /// CUE binary not found
  #[error("CUE binary not found: {0}")]
  CueBinaryNotFound(String),

  /// Security validation error
  #[error("security error: {0}")]
  Security(String),

  /// File not found
  #[error("file not found: {0}")]
  FileNotFound(String),
}

// =============================================================================
// Conversions
// =============================================================================

impl From<ParseError> for LoaderError {
  fn from(err: ParseError) -> Self {
    match err {
      ParseError::JsonError(msg) => LoaderError::Json(msg),
      ParseError::MissingField(field) => {
        LoaderError::Validation(format!("missing required field: {field}"))
      }
      ParseError::InvalidType {
        field,
        expected,
        actual,
      } => LoaderError::InvalidSpec {
        field,
        expected,
        actual,
      },
      ParseError::EmptyField(field) => LoaderError::EmptyField(field),
    }
  }
}

impl From<SecurityError> for LoaderError {
  fn from(err: SecurityError) -> Self {
    match err {
      SecurityError::PathTraversal { details } => {
        LoaderError::Security(format!("path traversal: {details}"))
      }
      SecurityError::EncodedPathTraversal { encoding_type } => {
        LoaderError::Security(format!("encoded path traversal: {encoding_type}"))
      }
      SecurityError::ShellMetacharacter { category, ch } => {
        LoaderError::Security(format!("shell metacharacter '{ch}' ({category})"))
      }
      SecurityError::ReDoSVulnerability { vulnerability } => {
        LoaderError::Security(format!("ReDoS vulnerability: {vulnerability}"))
      }
      SecurityError::SessionIdValidation { error } => {
        LoaderError::Security(format!("session ID validation: {error}"))
      }
      SecurityError::NullByteDetected => LoaderError::Security("null byte detected".into()),
      SecurityError::BackslashInPath => LoaderError::Security("backslash in path".into()),
      SecurityError::EmptyInput => LoaderError::Security("empty input".into()),
    }
  }
}

// =============================================================================
// Core Functions
// =============================================================================

/// Load a CUE file and parse it into a Spec
///
/// This function:
/// 1. Validates the file path for security
/// 2. Checks that the file exists and is readable
/// 3. Runs `cue vet` to validate the CUE syntax
/// 4. Runs `cue export` to convert to JSON
/// 5. Parses the JSON into a Spec struct
///
/// # Arguments
///
/// * `path` - Path to the CUE file
///
/// # Returns
///
/// `Ok(Spec)` if successful, `Err(LoaderError)` otherwise
///
/// # Errors
///
/// Returns `LoaderError` if:
/// - Path contains security issues (traversal, metacharacters, etc.)
/// - File does not exist or is not readable
/// - CUE validation fails
/// - CUE export fails
/// - JSON parsing fails
/// - Spec validation fails
///
/// # Example
///
/// ```rust,ignore
/// use clarity_web::intent::loader::load_cue_file;
/// use std::path::Path;
///
/// let spec = load_cue_file(Path::new("spec.cue"))?;
/// assert_eq!(spec.name, "my-spec");
/// ```
pub fn load_cue_file(path: &Path) -> Result<Spec, LoaderError> {
  // Step 1: Validate path for security
  let path_str = path.to_string_lossy();
  let validated_path = validate_file_path(&path_str)?;

  // Step 2: Check file exists and is readable
  validate_file_exists(&PathBuf::from(&validated_path))?;

  // Step 3: Validate CUE syntax
  validate_cue_file(&PathBuf::from(&validated_path))?;

  // Step 4: Export to JSON
  let json = export_cue_to_json(&PathBuf::from(&validated_path))?;

  // Step 5: Parse JSON into Spec
  let spec = parse_spec(&json)?;

  Ok(spec)
}

/// Validate that a file exists and is readable
///
/// # Arguments
///
/// * `path` - Path to validate
///
/// # Returns
///
/// `Ok(())` if file exists and is readable, `Err(LoaderError)` otherwise
fn validate_file_exists(path: &Path) -> Result<(), LoaderError> {
  if !path.exists() {
    return Err(LoaderError::FileNotFound(
      path.to_string_lossy().to_string(),
    ));
  }

  if !path.is_file() {
    return Err(LoaderError::Io(format!(
      "Path is not a file: {}",
      path.to_string_lossy()
    )));
  }

  // Check if readable by trying to get metadata
  path.metadata().map_err(|e| {
    LoaderError::Io(format!(
      "Cannot read file {}: {}",
      path.to_string_lossy(),
      e
    ))
  })?;

  Ok(())
}

/// Validate a CUE file using `cue vet`
///
/// Runs `cue vet <path>` to validate CUE syntax without exporting.
///
/// # Arguments
///
/// * `path` - Path to the CUE file
///
/// # Returns
///
/// `Ok(())` if validation succeeds, `Err(LoaderError)` otherwise
///
/// # Errors
///
/// Returns `LoaderError::CommandFailed` if `cue vet` fails.
/// Returns `LoaderError::CueBinaryNotFound` if `cue` is not in PATH.
pub fn validate_cue_file(path: &Path) -> Result<(), LoaderError> {
  let path_str = path.to_string_lossy();

  // Check cue binary exists
  check_cue_binary()?;

  // Run cue vet
  let output = Command::new("cue")
    .args(["vet", &path_str])
    .output()
    .map_err(|e| LoaderError::CommandFailed(format!("Failed to execute cue vet: {e}")))?;

  if output.status.success() {
    Ok(())
  } else {
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    Err(LoaderError::CommandFailed(format!(
      "cue vet failed for {}: {}",
      path_str, stderr
    )))
  }
}

/// Export a CUE file to JSON using `cue export`
///
/// Runs `cue export <path> -e spec` to export the `spec` field to JSON.
///
/// # Arguments
///
/// * `path` - Path to the CUE file
///
/// # Returns
///
/// `Ok(String)` containing JSON if successful, `Err(LoaderError)` otherwise
///
/// # Errors
///
/// Returns `LoaderError::CommandFailed` if `cue export` fails.
/// Returns `LoaderError::CueBinaryNotFound` if `cue` is not in PATH.
/// Returns `LoaderError::InvalidCueOutput` if output is not valid UTF-8.
pub fn export_cue_to_json(path: &Path) -> Result<String, LoaderError> {
  let path_str = path.to_string_lossy();

  // Check cue binary exists
  check_cue_binary()?;

  // Run cue export
  let output = Command::new("cue")
    .args(["export", &path_str, "-e", "spec"])
    .output()
    .map_err(|e| LoaderError::CommandFailed(format!("Failed to execute cue export: {e}")))?;

  if output.status.success() {
    let stdout = String::from_utf8(output.stdout)
      .map_err(|e| LoaderError::InvalidCueOutput(format!("Invalid UTF-8 in cue output: {e}")))?;
    Ok(stdout)
  } else {
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    Err(LoaderError::CommandFailed(format!(
      "cue export failed for {}: {}",
      path_str, stderr
    )))
  }
}

/// Check if the `cue` binary is available in PATH
///
/// # Returns
///
/// `Ok(())` if `cue` is found, `Err(LoaderError::CueBinaryNotFound)` otherwise
fn check_cue_binary() -> Result<(), LoaderError> {
  let result = Command::new("cue").arg("version").output();

  match result {
    Ok(output) if output.status.success() => Ok(()),
    Ok(_) => Err(LoaderError::CueBinaryNotFound(
      "cue command found but returned error. Ensure CUE is properly installed.".into(),
    )),
    Err(_) => Err(LoaderError::CueBinaryNotFound(
      "cue command not found in PATH. Install CUE from https://cuelang.org/docs/install/".into(),
    )),
  }
}

/// Format a LoaderError as a human-readable string
///
/// # Arguments
///
/// * `error` - The error to format
///
/// # Returns
///
/// A human-readable error message
#[must_use]
pub fn format_loader_error(error: &LoaderError) -> String {
  match error {
    LoaderError::Io(msg) => format!("I/O Error: {msg}"),
    LoaderError::Json(msg) => format!("JSON Error: {msg}"),
    LoaderError::CommandFailed(msg) => format!("Command Failed: {msg}"),
    LoaderError::Validation(msg) => format!("Validation Error: {msg}"),
    LoaderError::SessionNotFound(id) => format!("Session Not Found: {id}"),
    LoaderError::InvalidSpec {
      field,
      expected,
      actual,
    } => {
      format!("Invalid Spec: field '{field}' expected {expected}, got {actual}")
    }
    LoaderError::EmptyField(field) => {
      format!("Empty Field: '{field}' is required and cannot be empty")
    }
    LoaderError::InvalidCueOutput(msg) => format!("Invalid CUE Output: {msg}"),
    LoaderError::CueBinaryNotFound(msg) => format!("CUE Binary Not Found: {msg}"),
    LoaderError::Security(msg) => format!("Security Error: {msg}"),
    LoaderError::FileNotFound(path) => format!("File Not Found: {path}"),
  }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
  use super::*;
  use std::fs;
  use std::io::Write;
  use tempfile::TempDir;

  // =========================================================================
  // Helper Functions
  // =========================================================================

  /// Create a temporary CUE file with the given content
  fn create_temp_cue_file(content: &str) -> (TempDir, PathBuf) {
    let dir = TempDir::new().expect("temp dir should exist");

    let file_path = dir.path().join("test.cue");
    let mut file = fs::File::create(&file_path).expect("file should exist");
    file.write_all(content.as_bytes()).expect("Failed to write");
    drop(file);
    (dir, file_path)
  }

  /// Check if cue binary is available
  fn cue_available() -> bool {
    Command::new("cue")
      .arg("version")
      .output()
      .map(|o| o.status.success())
      .map_or(false, |v| v)
  }

  // =========================================================================
  // Error Display Tests
  // =========================================================================

  #[test]
  fn test_loader_error_display_io() {
    let err = LoaderError::Io("file not found".into());
    let msg = format!("{err}");
    assert!(msg.contains("I/O error"));
    assert!(msg.contains("file not found"));
  }

  #[test]
  fn test_loader_error_display_json() {
    let err = LoaderError::Json("parse error at line 1".into());
    let msg = format!("{err}");
    assert!(msg.contains("JSON error"));
    assert!(msg.contains("parse error"));
  }

  #[test]
  fn test_loader_error_display_command_failed() {
    let err = LoaderError::CommandFailed("cue vet failed".into());
    let msg = format!("{err}");
    assert!(msg.contains("command execution failed"));
    assert!(msg.contains("cue vet failed"));
  }

  #[test]
  fn test_loader_error_display_validation() {
    let err = LoaderError::Validation("missing name field".into());
    let msg = format!("{err}");
    assert!(msg.contains("validation error"));
    assert!(msg.contains("missing name field"));
  }

  #[test]
  fn test_loader_error_display_session_not_found() {
    let err = LoaderError::SessionNotFound("session-123".into());
    let msg = format!("{err}");
    assert!(msg.contains("session not found"));
    assert!(msg.contains("session-123"));
  }

  #[test]
  fn test_loader_error_display_invalid_spec() {
    let err = LoaderError::InvalidSpec {
      field: "name".into(),
      expected: "string".into(),
      actual: "number".into(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("invalid spec for field 'name'"));
    assert!(msg.contains("expected string"));
    assert!(msg.contains("got number"));
  }

  #[test]
  fn test_loader_error_display_empty_field() {
    let err = LoaderError::EmptyField("description".into());
    let msg = format!("{err}");
    assert!(msg.contains("empty required field"));
    assert!(msg.contains("description"));
  }

  #[test]
  fn test_loader_error_display_invalid_cue_output() {
    let err = LoaderError::InvalidCueOutput("not utf-8".into());
    let msg = format!("{err}");
    assert!(msg.contains("invalid CUE output"));
    assert!(msg.contains("not utf-8"));
  }

  #[test]
  fn test_loader_error_display_cue_binary_not_found() {
    let err = LoaderError::CueBinaryNotFound("install cue".into());
    let msg = format!("{err}");
    assert!(msg.contains("CUE binary not found"));
    assert!(msg.contains("install cue"));
  }

  #[test]
  fn test_loader_error_display_security() {
    let err = LoaderError::Security("path traversal detected".into());
    let msg = format!("{err}");
    assert!(msg.contains("security error"));
    assert!(msg.contains("path traversal"));
  }

  #[test]
  fn test_loader_error_display_file_not_found() {
    let err = LoaderError::FileNotFound("/path/to/file.cue".into());
    let msg = format!("{err}");
    assert!(msg.contains("file not found"));
    assert!(msg.contains("/path/to/file.cue"));
  }

  // =========================================================================
  // Format Loader Error Tests
  // =========================================================================

  #[test]
  fn test_format_loader_error_io() {
    let err = LoaderError::Io("permission denied".into());
    let formatted = format_loader_error(&err);
    assert!(formatted.contains("I/O Error"));
    assert!(formatted.contains("permission denied"));
  }

  #[test]
  fn test_format_loader_error_command() {
    let err = LoaderError::CommandFailed("exit code 1".into());
    let formatted = format_loader_error(&err);
    assert!(formatted.contains("Command Failed"));
    assert!(formatted.contains("exit code 1"));
  }

  #[test]
  fn test_format_loader_error_invalid_spec() {
    let err = LoaderError::InvalidSpec {
      field: "version".into(),
      expected: "semver".into(),
      actual: "date".into(),
    };
    let formatted = format_loader_error(&err);
    assert!(formatted.contains("Invalid Spec"));
    assert!(formatted.contains("'version'"));
    assert!(formatted.contains("semver"));
    assert!(formatted.contains("date"));
  }

  // =========================================================================
  // From<ParseError> Tests
  // =========================================================================

  #[test]
  fn test_from_parse_error_json() {
    let parse_err = ParseError::JsonError("bad json".into());
    let loader_err: LoaderError = parse_err.into();
    assert!(matches!(loader_err, LoaderError::Json(msg) if msg == "bad json"));
  }

  #[test]
  fn test_from_parse_error_missing_field() {
    let parse_err = ParseError::MissingField("name".into());
    let loader_err: LoaderError = parse_err.into();
    match loader_err {
      LoaderError::Validation(msg) => {
        assert!(msg.contains("missing required field"));
        assert!(msg.contains("name"));
      }
      _ => panic!("Expected Validation error"),
    }
  }

  #[test]
  fn test_from_parse_error_invalid_type() {
    let parse_err = ParseError::InvalidType {
      field: "count".into(),
      expected: "number".into(),
      actual: "string".into(),
    };
    let loader_err: LoaderError = parse_err.into();
    match loader_err {
      LoaderError::InvalidSpec {
        field,
        expected,
        actual,
      } => {
        assert_eq!(field, "count");
        assert_eq!(expected, "number");
        assert_eq!(actual, "string");
      }
      _ => panic!("Expected InvalidSpec error"),
    }
  }

  #[test]
  fn test_from_parse_error_empty_field() {
    let parse_err = ParseError::EmptyField("name".into());
    let loader_err: LoaderError = parse_err.into();
    assert!(matches!(loader_err, LoaderError::EmptyField(f) if f == "name"));
  }

  // =========================================================================
  // From<SecurityError> Tests
  // =========================================================================

  #[test]
  fn test_from_security_error_path_traversal() {
    let sec_err = SecurityError::PathTraversal {
      details: "literal .. detected".into(),
    };
    let loader_err: LoaderError = sec_err.into();
    match loader_err {
      LoaderError::Security(msg) => {
        assert!(msg.contains("path traversal"));
      }
      _ => panic!("Expected Security error"),
    }
  }

  #[test]
  fn test_from_security_error_shell_metachar() {
    let sec_err = SecurityError::ShellMetacharacter {
      category: super::super::security::MetacharCategory::CommandSeparator,
      ch: ';',
    };
    let loader_err: LoaderError = sec_err.into();
    match loader_err {
      LoaderError::Security(msg) => {
        assert!(msg.contains("shell metacharacter"));
        assert!(msg.contains(';'));
      }
      _ => panic!("Expected Security error"),
    }
  }

  #[test]
  fn test_from_security_error_null_byte() {
    let sec_err = SecurityError::NullByteDetected;
    let loader_err: LoaderError = sec_err.into();
    match loader_err {
      LoaderError::Security(msg) => {
        assert!(msg.contains("null byte"));
      }
      _ => panic!("Expected Security error"),
    }
  }

  // =========================================================================
  // validate_file_exists Tests
  // =========================================================================

  #[test]
  fn test_validate_file_exists_valid() {
    let dir = TempDir::new().expect("temp dir");

    let file_path = dir.path().join("exists.txt");
    fs::write(&file_path, "content").expect("write should succeed");

    let result = validate_file_exists(&file_path);
    assert!(result.is_ok());
  }

  #[test]
  fn test_validate_file_exists_not_found() {
    let result = validate_file_exists(Path::new("/nonexistent/path/file.cue"));
    assert!(result.is_err());
    match result {
      Err(LoaderError::FileNotFound(path)) => {
        assert!(path.contains("nonexistent"));
      }
      _ => panic!("Expected FileNotFound error"),
    }
  }

  #[test]
  fn test_validate_file_exists_directory() {
    let dir = TempDir::new().expect("temp dir");
    let dir_path = dir.path().to_path_buf();

    // Directory exists but is not a file
    let result = validate_file_exists(&dir_path);
    assert!(result.is_err());
    match result {
      Err(LoaderError::Io(msg)) => {
        assert!(msg.contains("not a file"));
      }
      _ => panic!("Expected Io error"),
    }
  }

  // =========================================================================
  // check_cue_binary Tests
  // =========================================================================

  #[test]
  fn test_check_cue_binary() {
    let result = check_cue_binary();
    if cue_available() {
      assert!(result.is_ok());
    } else {
      assert!(result.is_err());
      match result {
        Err(LoaderError::CueBinaryNotFound(msg)) => {
          assert!(msg.contains("not found") || msg.contains("error"));
        }
        _ => panic!("Expected CueBinaryNotFound error"),
      }
    }
  }

  // =========================================================================
  // validate_cue_file Tests
  // =========================================================================

  #[test]
  fn test_validate_cue_file_not_found() {
    if !cue_available() {
      return; // Skip if cue not installed
    }

    let result = validate_cue_file(Path::new("/nonexistent/file.cue"));
    assert!(result.is_err());
  }

  #[test]
  fn test_validate_cue_file_valid() {
    if !cue_available() {
      return; // Skip if cue not installed
    }

    let cue_content = r#"
package main

spec: {
    name: "test-spec"
    description: "A test specification"
}
"#;
    let (_temp_dir, file_path) = create_temp_cue_file(cue_content);

    let result = validate_cue_file(&file_path);
    assert!(result.is_ok());
  }

  #[test]
  fn test_validate_cue_file_invalid_syntax() {
    if !cue_available() {
      return; // Skip if cue not installed
    }

    let cue_content = r#"
package main

spec: {
    name: "test-spec"
    // Missing closing brace - invalid syntax
"#;
    let (_temp_dir, file_path) = create_temp_cue_file(cue_content);

    let result = validate_cue_file(&file_path);
    assert!(result.is_err());
    match result {
      Err(LoaderError::CommandFailed(msg)) => {
        assert!(msg.contains("cue vet failed"));
      }
      _ => panic!("Expected CommandFailed error"),
    }
  }

  // =========================================================================
  // export_cue_to_json Tests
  // =========================================================================

  #[test]
  fn test_export_cue_to_json_valid() {
    if !cue_available() {
      return; // Skip if cue not installed
    }

    let cue_content = r#"
package main

spec: {
    name: "test-spec"
    description: "A test specification"
}
"#;
    let (_temp_dir, file_path) = create_temp_cue_file(cue_content);

    let result = export_cue_to_json(&file_path);
    assert!(result.is_ok());

    let json = result.map_err(|_| ()).ok();
    let json = json.expect("json should exist");
    assert!(json.contains("test-spec"));
    assert!(json.contains("A test specification"));
  }

  #[test]
  fn test_export_cue_to_json_no_spec_field() {
    if !cue_available() {
      return; // Skip if cue not installed
    }

    let cue_content = r#"
package main

other: {
    name: "test"
}
"#;
    let (_temp_dir, file_path) = create_temp_cue_file(cue_content);

    let result = export_cue_to_json(&file_path);
    // Should fail because -e spec references non-existent field
    assert!(result.is_err());
  }

  // =========================================================================
  // load_cue_file Tests
  // =========================================================================

  #[test]
  fn test_load_cue_file_valid() {
    if !cue_available() {
      return; // Skip if cue not installed
    }

    let cue_content = r#"
package main

spec: {
    name: "test-spec"
    description: "A test specification"
    features: [{
        name: "auth"
        behaviors: [{name: "login"}]
    }]
}
"#;
    let (_temp_dir, file_path) = create_temp_cue_file(cue_content);

    let result = load_cue_file(&file_path);
    assert!(result.is_ok());

    let spec = result.map_err(|_| ()).ok();
    let spec = spec.expect("spec should exist");
    assert_eq!(spec.name, "test-spec");
    assert_eq!(spec.description, "A test specification");
    assert_eq!(spec.features.len(), 1);
    assert_eq!(spec.features[0].name, "auth");
  }

  #[test]
  fn test_load_cue_file_missing_name() {
    if !cue_available() {
      return; // Skip if cue not installed
    }

    let cue_content = r#"
package main

spec: {
    description: "A test specification without name"
}
"#;
    let (_temp_dir, file_path) = create_temp_cue_file(cue_content);

    let result = load_cue_file(&file_path);
    // Should fail because name is required
    assert!(result.is_err());
  }

  #[test]
  fn test_load_cue_file_empty_name() {
    if !cue_available() {
      return; // Skip if cue not installed
    }

    let cue_content = r#"
package main

spec: {
    name: ""
    description: "A test specification with empty name"
}
"#;
    let (_temp_dir, file_path) = create_temp_cue_file(cue_content);

    let result = load_cue_file(&file_path);
    // Should fail because name cannot be empty
    assert!(result.is_err());
    match result {
      Err(LoaderError::EmptyField(field)) => {
        assert_eq!(field, "name");
      }
      Err(LoaderError::Validation(msg)) => {
        assert!(msg.contains("name"));
      }
      _ => {} // Other error types are acceptable
    }
  }

  #[test]
  fn test_load_cue_file_with_features() {
    if !cue_available() {
      return; // Skip if cue not installed
    }

    let cue_content = r#"
package main

spec: {
    name: "full-spec"
    description: "A complete specification"
    features: [
        {
            name: "auth"
            description: "Authentication"
            behaviors: [
                {
                    name: "login"
                    description: "User login"
                }
            ]
        },
        {
            name: "users"
            description: "User management"
            depends_on: ["auth"]
            behaviors: [
                {
                    name: "create_user"
                    description: "Create a new user"
                }
            ]
        }
    ]
    invariants: [
        {name: "unique_email", description: "Emails must be unique"}
    ]
}
"#;
    let (_temp_dir, file_path) = create_temp_cue_file(cue_content);

    let result = load_cue_file(&file_path);
    assert!(result.is_ok());

    let spec = result.map_err(|_| ()).ok();
    let spec = spec.expect("spec should exist");
    assert_eq!(spec.name, "full-spec");
    assert_eq!(spec.features.len(), 2);
    assert_eq!(spec.features[0].name, "auth");
    assert_eq!(spec.features[1].name, "users");
    assert_eq!(spec.features[1].depends_on, vec!["auth"]);
    assert_eq!(spec.invariants.len(), 1);
  }

  #[test]
  fn test_load_cue_file_security_traversal() {
    let result = load_cue_file(Path::new("../../../etc/passwd"));
    assert!(result.is_err());
    match result {
      Err(LoaderError::Security(msg)) => {
        assert!(msg.contains("traversal") || msg.contains("metacharacter"));
      }
      Err(LoaderError::FileNotFound(_)) => {
        // This is also acceptable - path validation might pass but file doesn't exist
      }
      _ => panic!("Expected Security or FileNotFound error"),
    }
  }

  #[test]
  fn test_load_cue_file_security_shell_metachar() {
    let result = load_cue_file(Path::new("file;rm -rf /"));
    assert!(result.is_err());
    match result {
      Err(LoaderError::Security(msg)) => {
        assert!(msg.contains("metacharacter") || msg.contains("shell"));
      }
      _ => panic!("Expected Security error"),
    }
  }

  #[test]
  fn test_load_cue_file_nonexistent() {
    let result = load_cue_file(Path::new("/nonexistent/spec.cue"));
    assert!(result.is_err());
    match result {
      Err(LoaderError::FileNotFound(path)) => {
        assert!(path.contains("nonexistent"));
      }
      _ => panic!("Expected FileNotFound error"),
    }
  }

  // =========================================================================
  // Integration Tests
  // =========================================================================

  #[test]
  fn test_full_workflow() {
    if !cue_available() {
      return; // Skip if cue not installed
    }

    let cue_content = r#"
package main

spec: {
    name: "workflow-test"
    description: "Testing the full loader workflow"
    features: [
        {
            name: "core"
            behaviors: [
                {name: "init", description: "Initialize the system"},
                {name: "shutdown", description: "Shutdown cleanly"}
            ]
        }
    ]
    anti_patterns: [
        {name: "god_object", description: "Avoid god objects"}
    ]
    ai_hints: {
        preferred_libraries: ["serde", "tokio"]
        style_hints: ["Use functional patterns"]
    }
}
"#;
    let (_temp_dir, file_path) = create_temp_cue_file(cue_content);

    // Validate
    let validate_result = validate_cue_file(&file_path);
    assert!(validate_result.is_ok());

    // Export
    let export_result = export_cue_to_json(&file_path);
    assert!(export_result.is_ok());

    // Load full spec
    let load_result = load_cue_file(&file_path);
    assert!(load_result.is_ok());

    let spec = load_result.map_err(|_| ()).ok();
    let spec = spec.expect("spec should exist");
    assert_eq!(spec.name, "workflow-test");
    assert_eq!(spec.features[0].behaviors.len(), 2);
    assert_eq!(spec.anti_patterns.len(), 1);
    assert_eq!(spec.ai_hints.preferred_libraries, vec!["serde", "tokio"]);
  }
}
