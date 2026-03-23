//! CUE file loader - Imperative shell for CUE operations.
//!
//! This module is the "imperative shell" that handles all I/O operations,
//! delegating pure logic to `cue_core`. Following Scott Wlaschin's DDD
//! principle of "functional core, imperative shell".
//!
//! ## Architecture
//!
//! ```text
//! User Code
//!     |
//!     v
//! cue.rs (Shell - I/O only)
//!     |
//!     v
//! cue_core.rs (Core - Pure logic)
//! ```
//!
//! The shell handles:
//! - File system operations (exists, metadata)
//! - Process spawning (cue commands)
//! - Error conversion to `LoaderError`
//!
//! The core handles:
//! - Path validation
//! - Argument construction
//! - Output parsing

#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

#[path = "cue_core.rs"]
mod cue_core;

use super::error::{CueBinaryError, CueOutputError, LoaderError};
use std::path::Path;
use std::process::Command;

pub use cue_core::{validate_command_output, CommandOutput};

/// Validate that a file exists at the given path.
///
/// This is an I/O operation (shell function) that checks the filesystem.
///
/// # Errors
///
/// Returns `LoaderError` if:
/// - File does not exist
/// - Path is not a file
/// - Metadata cannot be read
pub fn validate_file_exists(path: &Path) -> Result<(), LoaderError> {
  // Shell: I/O operation
  if !path.exists() {
    return Err(LoaderError::file_not_found(path.to_string_lossy()));
  }
  if !path.is_file() {
    return Err(LoaderError::io_access(
      path.to_string_lossy(),
      "path is not a file",
    ));
  }

  // Shell: I/O operation
  path
    .metadata()
    .map_err(|error| LoaderError::io_metadata(path.to_string_lossy(), error.to_string()))?;

  Ok(())
}

/// Validate a CUE file using the `cue vet` command.
///
/// This is an I/O operation (shell function) that spawns a subprocess.
///
/// # Errors
///
/// Returns `LoaderError` if:
/// - CUE binary is not available
/// - Command fails to spawn
/// - Command returns non-zero exit code
pub fn validate_cue_file(path: &Path) -> Result<(), LoaderError> {
  let path_str = path.to_string_lossy();

  // Shell: Check binary availability
  check_cue_binary()?;

  // Core: Build arguments (pure)
  let args = cue_core::build_vet_args(&path_str);

  // Shell: Execute command (I/O)
  let output = Command::new("cue")
    .args(&args)
    .output()
    .map_err(|error| LoaderError::command_spawn_failed("cue vet", error.to_string()))?;

  // Shell: Convert to our type
  let command_output = CommandOutput::from_raw(
    output.status.code().map_or(-1, |v| v),
    String::from_utf8_lossy(&output.stdout).to_string(),
    String::from_utf8_lossy(&output.stderr).to_string(),
  );

  // Core: Validate output (pure)
  validate_command_output(&command_output)
    .map_err(|(code, stderr)| LoaderError::command_exit_code("cue vet", code, stderr))
}

/// Export a CUE file to JSON using the `cue export` command.
///
/// This is an I/O operation (shell function) that spawns a subprocess.
///
/// # Errors
///
/// Returns `LoaderError` if:
/// - CUE binary is not available
/// - Command fails to spawn
/// - Command returns non-zero exit code
/// - Output is not valid UTF-8
pub fn export_cue_to_json(path: &Path) -> Result<String, LoaderError> {
  let path_str = path.to_string_lossy();

  // Shell: Check binary availability
  check_cue_binary()?;

  // Core: Build arguments (pure)
  let args = cue_core::build_export_args(&path_str);

  // Shell: Execute command (I/O)
  let output = Command::new("cue")
    .args(&args)
    .output()
    .map_err(|error| LoaderError::command_spawn_failed("cue export", error.to_string()))?;

  // Shell: Convert to our type
  let command_output = CommandOutput::from_raw(
    output.status.code().map_or(-1, |v| v),
    String::from_utf8_lossy(&output.stdout).to_string(),
    String::from_utf8_lossy(&output.stderr).to_string(),
  );

  // Core: Validate output (pure)
  validate_command_output(&command_output)
    .map_err(|(code, stderr)| LoaderError::command_exit_code("cue export", code, stderr))?;

  // Core: Parse UTF-8 output (pure)
  cue_core::parse_utf8_output(&output.stdout).map_err(|error| LoaderError::InvalidCueOutput {
    reason: map_cue_core_error(error),
  })
}

fn map_cue_core_error(error: cue_core::CueCoreError) -> CueOutputError {
  match error {
    cue_core::CueCoreError::InvalidUtf8 { details } => {
      CueOutputError::InvalidUtf8 { error: details }
    }
    cue_core::CueCoreError::EmptyOutput => CueOutputError::EmptyOutput,
    cue_core::CueCoreError::InvalidJson { details } => {
      CueOutputError::InvalidJson { error: details }
    }
  }
}

/// Check if the CUE binary is available and working.
///
/// This is an I/O operation (shell function) that spawns a subprocess.
///
/// # Errors
///
/// Returns `LoaderError::CueBinaryNotFound` if CUE is not available.
fn check_cue_binary() -> Result<(), LoaderError> {
  // Shell: Execute command (I/O)
  let output = Command::new("cue").arg("version").output();

  match output {
    Ok(output) if output.status.success() => Ok(()),
    Ok(_) => Err(LoaderError::CueBinaryNotFound {
      details: CueBinaryError::ExecutionError {
        message: "cue command found but returned error. Ensure CUE is properly installed.".into(),
      },
    }),
    Err(_) => Err(LoaderError::CueBinaryNotFound {
      details: CueBinaryError::NotInPath,
    }),
  }
}

#[cfg(test)]
#[allow(
  clippy::unwrap_used,
  clippy::expect_used,
  clippy::panic,
  clippy::float_cmp,
  clippy::needless_collect,
  clippy::unnecessary_debug_formatting,
  clippy::match_same_arms,
  clippy::option_if_let_else,
  clippy::suspicious_else_formatting,
  clippy::manual_let_else,
  clippy::match_wild_err_arm,
  clippy::match_like_matches_macro,
  clippy::needless_pass_by_value
)]
mod tests {
  use super::*;
  use crate::intent::loader::cue::cue_core::validate_path_string;

  #[test]
  fn validate_file_exists_fails_for_nonexistent() {
    let result = validate_file_exists(Path::new("/nonexistent/path/to/file.cue"));
    assert!(result.is_err());
  }

  #[test]
  fn validate_path_string_reexport_works() {
    assert!(validate_path_string("/valid/path.cue").is_some());
    assert!(validate_path_string("").is_none());
  }
}
