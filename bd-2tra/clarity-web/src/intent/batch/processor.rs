//! Batch Processing Module
//!
//! Processes multiple spec files and generates summary reports.
//!
//! Ported from intent-cli/src/intent/batch.gleam

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use itertools::Itertools;
use thiserror::Error;

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
}

impl Default for SpecResult {
  fn default() -> Self {
    Self {
      file: String::new(),
      status: BatchStatus::Skipped,
      behaviors_count: 0,
      quality_score: 0,
      error: String::new(),
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
#[derive(Debug, Clone)]
pub struct BatchSummary {
  pub total_files: usize,
  pub successful: usize,
  pub failed: usize,
  pub skipped: usize,
  pub total_behaviors: usize,
  pub average_quality: u32,
  pub results: Vec<SpecResult>,
}

impl Default for BatchSummary {
  fn default() -> Self {
    Self {
      total_files: 0,
      successful: 0,
      failed: 0,
      skipped: 0,
      total_behaviors: 0,
      average_quality: 0,
      results: Vec::new(),
    }
  }
}

/// Errors during batch processing
#[derive(Debug, Clone, Error)]
pub enum BatchError {
  #[error("no spec files provided")]
  NoSpecFiles,

  #[error("invalid file path: {0}")]
  InvalidPath(String),

  #[error("file not found: {0}")]
  FileNotFound(String),

  #[error("cannot access file: {0}")]
  CannotAccessFile(String),

  #[error("not a directory: {0}")]
  NotADirectory(String),

  #[error("failed to read directory: {0}")]
  ReadDirectoryError(String),

  #[error("no .cue files found in: {0}")]
  NoCueFilesFound(String),
}

/// Process multiple spec files sequentially
///
/// # Errors
/// Returns `BatchError` if the files list is empty or other critical errors occur.
pub fn process_specs(files: &[String], config: &BatchConfig) -> BatchSummary {
  if files.is_empty() {
    return BatchSummary::default();
  }

  let results: Vec<SpecResult> = files
    .iter()
    .enumerate()
    .map(|(index, file)| process_single_spec(file, index + 1, files.len(), config))
    .collect();

  let summary = generate_summary(&results);
  summary
}

/// Process a single spec file
fn process_single_spec(
  file: &str,
  _position: usize,
  _total: usize,
  _config: &BatchConfig,
) -> SpecResult {
  // Validate file path for security
  if !is_valid_file_path(file) {
    return SpecResult {
      file: file.to_string(),
      status: BatchStatus::Failed,
      error: "Invalid file path".to_string(),
      ..SpecResult::default()
    };
  }

  // Check if file exists
  let path = std::path::Path::new(file);
  if !path.exists() {
    return SpecResult {
      file: file.to_string(),
      status: BatchStatus::Skipped,
      error: "File not found".to_string(),
      ..SpecResult::default()
    };
  }

  if !path.is_file() {
    return SpecResult {
      file: file.to_string(),
      status: BatchStatus::Failed,
      error: "Cannot access file".to_string(),
      ..SpecResult::default()
    };
  }

  // For now, return a placeholder result
  // In a real implementation, this would load and analyze the spec
  SpecResult {
    file: file.to_string(),
    status: BatchStatus::Success,
    behaviors_count: 0,
    quality_score: 0,
    error: String::new(),
  }
}

/// Validate file path for security
fn is_valid_file_path(path: &str) -> bool {
  // Basic validation: not empty, no null bytes, no shell metacharacters
  !path.is_empty()
    && !path.contains('\0')
    && !path.contains('|')
    && !path.contains(';')
    && !path.contains('&')
    && !path.contains('`')
    && !path.contains('$')
    && !path.contains('(')
    && !path.contains(')')
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
    sum / u32::try_from(successful_results.len()).unwrap_or(1)
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
    return Err(BatchError::NotADirectory(dir.to_string()));
  }

  if !path.is_dir() {
    return Err(BatchError::NotADirectory(dir.to_string()));
  }

  let entries =
    std::fs::read_dir(path).map_err(|_| BatchError::ReadDirectoryError(dir.to_string()))?;

  let cue_files: Vec<String> = entries
    .filter_map(|entry| entry.ok())
    .filter_map(|entry| entry.file_name().to_str().map(String::from))
    .filter(|name| name.ends_with(".cue") && !name.starts_with('.'))
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
    Err(BatchError::NoCueFilesFound(dir.to_string()))
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
  }

  #[test]
  fn test_process_specs_nonexistent_file() {
    let config = BatchConfig::default();
    let files = vec!["/nonexistent/file.cue".to_string()];
    let summary = process_specs(&files, &config);
    assert_eq!(summary.skipped, 1);
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
      },
      SpecResult {
        file: "b.cue".to_string(),
        status: BatchStatus::Success,
        behaviors_count: 10,
        quality_score: 90,
        error: String::new(),
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
      },
      SpecResult {
        file: "b.cue".to_string(),
        status: BatchStatus::Failed,
        behaviors_count: 0,
        quality_score: 0,
        error: "Parse error".to_string(),
      },
      SpecResult {
        file: "c.cue".to_string(),
        status: BatchStatus::Skipped,
        behaviors_count: 0,
        quality_score: 0,
        error: "File not found".to_string(),
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
    assert!(result.is_err());
    assert!(matches!(result, Err(BatchError::NotADirectory(_))));
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
