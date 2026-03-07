//! Batch Processing Module
//!
//! Processes multiple spec files and generates summary reports.
//!
//! Ported from intent-cli/src/intent/batch.gleam

#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use itertools::Itertools;
use thiserror::Error;

/// Structured reasons for invalid spec paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidPathReason {
  Empty,
  NullByte,
  PathTraversal,
  ShellMetacharacter(ShellMetacharacter),
}

impl std::fmt::Display for InvalidPathReason {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Empty => write!(f, "path is empty"),
      Self::NullByte => write!(f, "path contains a null byte"),
      Self::PathTraversal => write!(f, "path traversal is not allowed"),
      Self::ShellMetacharacter(character) => {
        write!(
          f,
          "path contains shell metacharacter '{}'",
          character.as_char()
        )
      }
    }
  }
}

/// Typed shell metacharacters rejected from file paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellMetacharacter {
  Pipe,
  Semicolon,
  Ampersand,
  Backtick,
  Dollar,
  OpenParen,
  CloseParen,
}

impl ShellMetacharacter {
  const fn as_char(self) -> char {
    match self {
      Self::Pipe => '|',
      Self::Semicolon => ';',
      Self::Ampersand => '&',
      Self::Backtick => '`',
      Self::Dollar => '$',
      Self::OpenParen => '(',
      Self::CloseParen => ')',
    }
  }
}

/// Typed reasons for per-file processing failures.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum SpecProcessingError {
  #[error("invalid file path '{path}': {reason}")]
  InvalidPath {
    path: String,
    reason: InvalidPathReason,
  },

  #[error("file not found: {path}")]
  FileNotFound { path: String },

  #[error("cannot access file '{path}': {reason}")]
  CannotAccessFile {
    path: String,
    reason: FileAccessReason,
  },
}

/// Typed reasons a file could not be accessed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileAccessReason {
  NotAFile,
  PermissionDenied,
}

impl std::fmt::Display for FileAccessReason {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::NotAFile => write!(f, "path does not point to a regular file"),
      Self::PermissionDenied => write!(f, "permission denied"),
    }
  }
}

/// Typed reasons a directory read failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectoryReadErrorKind {
  PermissionDenied,
  Other,
}

impl std::fmt::Display for DirectoryReadErrorKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::PermissionDenied => write!(f, "permission denied"),
      Self::Other => write!(f, "other I/O error"),
    }
  }
}

/// Batch processing configuration
#[derive(Debug, Clone)]
pub struct BatchConfig {
  pub output_dir: String,
  pub parallel: bool,
  pub verbose: bool,
  pub continue_on_error: bool,
}

impl Default for BatchConfig {
  fn default() -> Self {
    Self {
      output_dir: String::new(),
      parallel: false,
      verbose: false,
      continue_on_error: true,
    }
  }
}

/// Result of processing a single spec
#[derive(Debug, Clone)]
pub struct SpecResult {
  pub file: String,
  pub status: BatchStatus,
  pub behaviors_count: usize,
  pub quality_score: u32,
  pub error: String,
  pub error_detail: Option<SpecProcessingError>,
}

impl Default for SpecResult {
  fn default() -> Self {
    Self {
      file: String::new(),
      status: BatchStatus::Skipped,
      behaviors_count: 0,
      quality_score: 0,
      error: String::new(),
      error_detail: None,
    }
  }
}

/// Batch processing status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatchStatus {
  Success,
  Failed,
  Skipped,
}

impl BatchStatus {
  /// Convert status to string
  #[must_use]
  pub const fn as_str(&self) -> &'static str {
    match self {
      Self::Success => "success",
      Self::Failed => "failed",
      Self::Skipped => "skipped",
    }
  }
}

/// Summary report for batch processing
#[derive(Debug, Clone, Default)]
pub struct BatchSummary {
  pub total_files: usize,
  pub successful: usize,
  pub failed: usize,
  pub skipped: usize,
  pub total_behaviors: usize,
  pub average_quality: u32,
  pub results: Vec<SpecResult>,
}

/// Errors during batch processing
#[derive(Debug, Clone, Error)]
pub enum BatchError {
  #[error("no spec files provided")]
  NoSpecFiles,

  #[error("invalid file path '{path}': {reason}")]
  InvalidPath {
    path: String,
    reason: InvalidPathReason,
  },

  #[error("file not found: {path}")]
  FileNotFound { path: String },

  #[error("cannot access file '{path}': {reason}")]
  CannotAccessFile {
    path: String,
    reason: FileAccessReason,
  },

  #[error("not a directory: {path}")]
  NotADirectory { path: String },

  #[error("failed to read directory '{path}': {kind}")]
  ReadDirectoryError {
    path: String,
    kind: DirectoryReadErrorKind,
  },

  #[error("no .cue files found in: {path}")]
  NoCueFilesFound { path: String },
}

/// Process multiple spec files sequentially
///
#[must_use]
pub fn process_specs(files: &[String], config: &BatchConfig) -> BatchSummary {
  if files.is_empty() {
    return BatchSummary::default();
  }

  let results: Vec<SpecResult> = files
    .iter()
    .enumerate()
    .map(|(index, file)| process_single_spec(file, index + 1, files.len(), config))
    .collect();

  generate_summary(&results)
}

/// Process a single spec file
fn process_single_spec(
  file: &str,
  _position: usize,
  _total: usize,
  _config: &BatchConfig,
) -> SpecResult {
  // Validate file path for security
  if let Err(reason) = validate_file_path(file) {
    return spec_result_with_error(
      file,
      BatchStatus::Failed,
      SpecProcessingError::InvalidPath {
        path: file.to_string(),
        reason,
      },
    );
  }

  // Check if file exists
  let path = std::path::Path::new(file);
  if !path.exists() {
    return spec_result_with_error(
      file,
      BatchStatus::Skipped,
      SpecProcessingError::FileNotFound {
        path: file.to_string(),
      },
    );
  }

  if !path.is_file() {
    return spec_result_with_error(
      file,
      BatchStatus::Failed,
      SpecProcessingError::CannotAccessFile {
        path: file.to_string(),
        reason: FileAccessReason::NotAFile,
      },
    );
  }

  // For now, return a placeholder result
  // In a real implementation, this would load and analyze the spec
  SpecResult {
    file: file.to_string(),
    status: BatchStatus::Success,
    behaviors_count: 0,
    quality_score: 0,
    error: String::new(),
    error_detail: None,
  }
}

fn spec_result_with_error(
  file: &str,
  status: BatchStatus,
  error: SpecProcessingError,
) -> SpecResult {
  SpecResult {
    file: file.to_string(),
    status,
    error: error.to_string(),
    error_detail: Some(error),
    ..SpecResult::default()
  }
}

/// Validate file path for security
fn validate_file_path(path: &str) -> Result<(), InvalidPathReason> {
  if path.is_empty() {
    return Err(InvalidPathReason::Empty);
  }

  if path.contains('\0') {
    return Err(InvalidPathReason::NullByte);
  }

  if path.split('/').any(|segment| segment == "..") {
    return Err(InvalidPathReason::PathTraversal);
  }

  let shell_metacharacter = path.chars().find_map(|character| match character {
    '|' => Some(ShellMetacharacter::Pipe),
    ';' => Some(ShellMetacharacter::Semicolon),
    '&' => Some(ShellMetacharacter::Ampersand),
    '`' => Some(ShellMetacharacter::Backtick),
    '$' => Some(ShellMetacharacter::Dollar),
    '(' => Some(ShellMetacharacter::OpenParen),
    ')' => Some(ShellMetacharacter::CloseParen),
    _ => None,
  });

  shell_metacharacter.map_or(Ok(()), |character| {
    Err(InvalidPathReason::ShellMetacharacter(character))
  })
}

fn is_valid_file_path(path: &str) -> bool {
  validate_file_path(path).is_ok()
}

/// Generate summary from results
fn generate_summary(results: &[SpecResult]) -> BatchSummary {
  let successful = results
    .iter()
    .filter(|r| r.status == BatchStatus::Success)
    .count();

  let failed = results
    .iter()
    .filter(|r| r.status == BatchStatus::Failed)
    .count();

  let skipped = results
    .iter()
    .filter(|r| r.status == BatchStatus::Skipped)
    .count();

  let total_behaviors: usize = results
    .iter()
    .filter(|r| r.status == BatchStatus::Success)
    .map(|r| r.behaviors_count)
    .sum();

  let successful_results: Vec<&SpecResult> = results
    .iter()
    .filter(|r| r.status == BatchStatus::Success)
    .collect();

  let average_quality = if successful_results.is_empty() {
    0
  } else {
    let sum: u32 = successful_results.iter().map(|r| r.quality_score).sum();
    match u32::try_from(successful_results.len()) {
      Ok(length) if length > 0 => sum / length,
      _ => 0,
    }
  };

  BatchSummary {
    total_files: results.len(),
    successful,
    failed,
    skipped,
    total_behaviors,
    average_quality,
    results: results.to_vec(),
  }
}

/// Get spec files from directory
///
/// # Errors
/// Returns `BatchError` if the directory cannot be read or contains no .cue files.
pub fn get_specs_from_dir(dir: &str) -> Result<Vec<String>, BatchError> {
  let path = std::path::Path::new(dir);

  if !path.exists() {
    return Err(BatchError::FileNotFound {
      path: dir.to_string(),
    });
  }

  if !path.is_dir() {
    return Err(BatchError::NotADirectory {
      path: dir.to_string(),
    });
  }

  let entries = std::fs::read_dir(path).map_err(|error| BatchError::ReadDirectoryError {
    path: dir.to_string(),
    kind: match error.kind() {
      std::io::ErrorKind::PermissionDenied => DirectoryReadErrorKind::PermissionDenied,
      _ => DirectoryReadErrorKind::Other,
    },
  })?;

  let cue_files: Vec<String> = entries
    .filter_map(std::result::Result::ok)
    .filter_map(|entry| entry.file_name().to_str().map(String::from))
    .filter(|name| {
      std::path::Path::new(name)
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("cue"))
        && !name.starts_with('.')
    })
    .map(|name| {
      if name.starts_with('/') {
        name
      } else {
        format!("{dir}/{name}")
      }
    })
    .sorted()
    .collect();

  if cue_files.is_empty() {
    Err(BatchError::NoCueFilesFound {
      path: dir.to_string(),
    })
  } else {
    Ok(cue_files)
  }
}

/// Format batch status as string
#[must_use]
pub fn status_to_string(status: BatchStatus) -> String {
  status.as_str().to_string()
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

  #[test]
  fn test_batch_status_as_str() {
    assert_eq!(BatchStatus::Success.as_str(), "success");
    assert_eq!(BatchStatus::Failed.as_str(), "failed");
    assert_eq!(BatchStatus::Skipped.as_str(), "skipped");
  }

  #[test]
  fn test_status_to_string() {
    assert_eq!(status_to_string(BatchStatus::Success), "success");
    assert_eq!(status_to_string(BatchStatus::Failed), "failed");
    assert_eq!(status_to_string(BatchStatus::Skipped), "skipped");
  }

  #[test]
  fn test_process_specs_empty_files() {
    let config = BatchConfig::default();
    let summary = process_specs(&[], &config);
    assert_eq!(summary.total_files, 0);
  }

  #[test]
  fn test_process_specs_invalid_path() {
    let config = BatchConfig::default();
    let files = vec!["../etc/passwd".to_string()];
    let summary = process_specs(&files, &config);
    assert_eq!(summary.total_files, 1);
    assert_eq!(summary.failed, 1);
    assert!(matches!(
      summary.results.first().and_then(|result| result.error_detail.as_ref()),
      Some(SpecProcessingError::InvalidPath {
        path,
        reason: InvalidPathReason::PathTraversal,
      }) if path == "../etc/passwd"
    ));
  }

  #[test]
  fn test_process_specs_nonexistent_file() {
    let config = BatchConfig::default();
    let files = vec!["/nonexistent/file.cue".to_string()];
    let summary = process_specs(&files, &config);
    assert_eq!(summary.skipped, 1);
    assert!(matches!(
      summary.results.first().and_then(|result| result.error_detail.as_ref()),
      Some(SpecProcessingError::FileNotFound { path }) if path == "/nonexistent/file.cue"
    ));
  }

  #[test]
  fn test_generate_summary_all_success() {
    let results = vec![
      SpecResult {
        file: "a.cue".to_string(),
        status: BatchStatus::Success,
        behaviors_count: 5,
        quality_score: 80,
        error: String::new(),
        error_detail: None,
      },
      SpecResult {
        file: "b.cue".to_string(),
        status: BatchStatus::Success,
        behaviors_count: 10,
        quality_score: 90,
        error: String::new(),
        error_detail: None,
      },
    ];
    let summary = generate_summary(&results);
    assert_eq!(summary.total_files, 2);
    assert_eq!(summary.successful, 2);
    assert_eq!(summary.failed, 0);
    assert_eq!(summary.skipped, 0);
    assert_eq!(summary.total_behaviors, 15);
    assert_eq!(summary.average_quality, 85);
  }

  #[test]
  fn test_generate_summary_mixed() {
    let results = vec![
      SpecResult {
        file: "a.cue".to_string(),
        status: BatchStatus::Success,
        behaviors_count: 5,
        quality_score: 80,
        error: String::new(),
        error_detail: None,
      },
      SpecResult {
        file: "b.cue".to_string(),
        status: BatchStatus::Failed,
        behaviors_count: 0,
        quality_score: 0,
        error: "Parse error".to_string(),
        error_detail: None,
      },
      SpecResult {
        file: "c.cue".to_string(),
        status: BatchStatus::Skipped,
        behaviors_count: 0,
        quality_score: 0,
        error: "File not found".to_string(),
        error_detail: None,
      },
    ];
    let summary = generate_summary(&results);
    assert_eq!(summary.total_files, 3);
    assert_eq!(summary.successful, 1);
    assert_eq!(summary.failed, 1);
    assert_eq!(summary.skipped, 1);
    assert_eq!(summary.total_behaviors, 5);
    assert_eq!(summary.average_quality, 80);
  }

  #[test]
  fn test_generate_summary_empty() {
    let results: Vec<SpecResult> = Vec::new();
    let summary = generate_summary(&results);
    assert_eq!(summary.total_files, 0);
    assert_eq!(summary.average_quality, 0);
  }

  #[test]
  fn test_is_valid_file_path() {
    assert!(is_valid_file_path("valid.cue"));
    assert!(is_valid_file_path("/path/to/valid.cue"));
    assert!(!is_valid_file_path(""));
    assert!(!is_valid_file_path("path|with|pipes"));
    assert!(!is_valid_file_path("path;with;semicolons"));
    assert!(!is_valid_file_path("$(command)"));
    assert!(!is_valid_file_path("`backtick`"));
  }

  #[test]
  fn test_get_specs_from_dir_not_found() {
    let result = get_specs_from_dir("/nonexistent/directory");
    assert!(matches!(
      result,
      Err(BatchError::FileNotFound { path }) if path == "/nonexistent/directory"
    ));
  }

  #[test]
  fn test_batch_config_default() {
    let config = BatchConfig::default();
    assert!(!config.parallel);
    assert!(!config.verbose);
    assert!(config.continue_on_error);
  }

  #[test]
  fn test_spec_result_default() {
    let result = SpecResult::default();
    assert_eq!(result.status, BatchStatus::Skipped);
    assert_eq!(result.behaviors_count, 0);
    assert_eq!(result.quality_score, 0);
  }

  #[test]
  fn test_batch_summary_default() {
    let summary = BatchSummary::default();
    assert_eq!(summary.total_files, 0);
    assert_eq!(summary.successful, 0);
    assert_eq!(summary.failed, 0);
    assert_eq!(summary.skipped, 0);
  }
}
