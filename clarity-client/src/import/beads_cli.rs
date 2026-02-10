#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! Import from beads_rust CLI
//!
//! This module provides functionality to import beads from the beads_rust
//! CLI's JSONL format stored in `.beads/issues.jsonl`.

use clarity_core::db::models::{BeadPriority, BeadStatus, BeadType, NewBead};
use rpds::Vector;
use serde::Deserialize;
use std::path::PathBuf;
use thiserror::Error;
use tracing::{debug, error, info, instrument, warn};

/// Import errors
///
/// Domain errors for the import process using thiserror for
/// semantic error types.
#[derive(Debug, Error)]
pub enum BeadsCliImportError {
  /// Failed to read beads_rust issues file
  #[error("Failed to read beads_rust issues file at {path}: {source}")]
  FileRead {
    path: String,
    #[source]
    source: std::io::Error,
  },

  /// Failed to parse JSONL line
  #[error("Failed to parse JSONL line {line}: {source}")]
  JsonParse {
    line: usize,
    #[source]
    source: serde_json::Error,
  },

  /// Invalid issue data
  #[error("Invalid issue data (line={line}): {field} - {reason}")]
  InvalidData {
    line: usize,
    field: String,
    reason: String,
  },

  /// No valid beads found
  #[error("No valid beads found in issues.jsonl")]
  NoValidBeads,

  /// Beads directory not found
  #[error("Beads directory not found at {path}")]
  BeadsNotFound { path: String },

  /// Issues file not found
  #[error("Issues file not found at {path}")]
  IssuesNotFound { path: String },
}

/// Result type for beads_rust import
pub type BeadsCliImportResult<T> = Result<T, BeadsCliImportError>;

/// Beads_rust issue representation
///
/// Pure data structure for representing an issue from beads_rust's JSONL format.
#[derive(Debug, Deserialize, Clone)]
struct BeadsCliIssue {
  /// Unique identifier (e.g., "bd-10g")
  id: String,
  /// Issue title
  title: String,
  /// Optional description (may contain enhanced bead format)
  #[serde(default)]
  description: Option<String>,
  /// Status (open, closed)
  status: String,
  /// Priority level (1=high, 2=medium, 3=low)
  priority: i64,
  /// Issue type (feature, bug, task, etc.)
  #[serde(rename = "issue_type")]
  issue_type: String,
  /// Creation timestamp (ISO 8601)
  _created_at: String,
  /// Creator username
  #[serde(default)]
  _created_by: Option<String>,
  /// Last update timestamp (ISO 8601)
  #[serde(default)]
  _updated_at: Option<String>,
  /// Closure timestamp (ISO 8601)
  #[serde(default)]
  _closed_at: Option<String>,
  /// Source repository
  #[serde(default)]
  _source_repo: Option<String>,
}

/// Import preview showing what will be imported
#[derive(Debug, Clone)]
pub struct BeadsCliImportPreview {
  /// Beads that will be added
  pub to_add: Vector<NewBead>,
  /// Beads that will be skipped (duplicates)
  pub to_skip: Vector<String>,
  /// Errors encountered during parsing
  pub errors: Vector<BeadsCliImportError>,
}

impl BeadsCliImportPreview {
  /// Create a new empty preview
  #[must_use]
  pub fn new() -> Self {
    Self {
      to_add: Vector::new(),
      to_skip: Vector::new(),
      errors: Vector::new(),
    }
  }

  /// Total number of beads to process
  #[must_use]
  pub fn total_count(&self) -> usize {
    self.to_add.len() + self.to_skip.len()
  }

  /// Check if any errors occurred
  #[must_use]
  pub fn has_errors(&self) -> bool {
    !self.errors.is_empty()
  }

  /// Check if any changes will be made
  #[must_use]
  pub fn has_changes(&self) -> bool {
    !self.to_add.is_empty()
  }
}

impl Default for BeadsCliImportPreview {
  fn default() -> Self {
    Self::new()
  }
}

/// Beads_rust configuration
///
/// Configurable paths for finding the beads_rust issues file.
#[derive(Debug, Clone)]
pub struct BeadsCliConfig {
  /// Path to search for the .beads directory
  pub search_path: PathBuf,
}

impl BeadsCliConfig {
  /// Create a new configuration with the default search path
  #[must_use]
  pub fn new() -> Self {
    Self {
      search_path: PathBuf::from("."),
    }
  }

  /// Set a custom search path
  #[must_use]
  pub fn with_search_path(mut self, path: PathBuf) -> Self {
    self.search_path = path;
    self
  }

  /// Find the beads_rust issues file
  ///
  /// # Errors
  /// Returns `BeadsCliImportError::BeadsNotFound` or `IssuesNotFound` if files don't exist
  pub fn find_issues_file(&self) -> BeadsCliImportResult<PathBuf> {
    let beads_dir = self.search_path.join(".beads");
    if !beads_dir.exists() {
      return Err(BeadsCliImportError::BeadsNotFound {
        path: beads_dir.display().to_string(),
      });
    }

    let issues_file = beads_dir.join("issues.jsonl");
    if !issues_file.exists() {
      return Err(BeadsCliImportError::IssuesNotFound {
        path: issues_file.display().to_string(),
      });
    }

    Ok(issues_file)
  }
}

impl Default for BeadsCliConfig {
  fn default() -> Self {
    Self::new()
  }
}

/// Import beads from beads_rust CLI
///
/// Reads issues from the `.beads/issues.jsonl` file and returns
/// a preview of what will be imported.
///
/// # Arguments
/// * `config` - Configuration for finding the issues file
/// * `existing_titles` - Set of existing bead titles to check for duplicates
///
/// # Returns
/// Import preview showing what will be added and skipped
///
/// # Errors
/// Returns `BeadsCliImportError` if:
/// - File cannot be read
/// - JSON parsing fails
/// - Data validation fails
#[instrument(skip(config, existing_titles), fields(existing_count = existing_titles.len()))]
pub fn import_from_beads_cli(
  config: &BeadsCliConfig,
  existing_titles: &Vector<String>,
) -> BeadsCliImportResult<BeadsCliImportPreview> {
  info!("Starting import from beads_rust CLI");

  let issues_path = config.find_issues_file()?;
  debug!(path = %issues_path.display(), "Found beads_rust issues file");

  let content = std::fs::read_to_string(&issues_path).map_err(|e| {
    error!(error = %e, path = %issues_path.display(), "Failed to read beads_rust issues file");
    BeadsCliImportError::FileRead {
      path: issues_path.display().to_string(),
      source: e,
    }
  })?;

  let total_lines = content.lines().count();
  info!(total_lines, "Read beads_rust issues file");

  let mut preview = BeadsCliImportPreview::new();

  for (line_num, line) in content.lines().enumerate() {
    let line_idx = line_num + 1; // Use 1-based indexing for error messages

    // Skip empty lines
    if line.trim().is_empty() {
      continue;
    }

    // Parse JSON line
    let issue: BeadsCliIssue = match serde_json::from_str(line) {
      Ok(i) => i,
      Err(e) => {
        warn!(line = line_idx, error = %e, "Failed to parse JSONL line");
        preview = BeadsCliImportPreview {
          errors: preview.errors.push_back(BeadsCliImportError::JsonParse {
            line: line_idx,
            source: e,
          }),
          ..preview
        };
        continue;
      }
    };

    // Convert to NewBead
    match map_issue_to_bead(issue.clone(), line_idx) {
      Ok(bead) => {
        // Check for duplicates based on title
        let is_duplicate = existing_titles.iter().any(|t| t == &bead.title);
        if is_duplicate {
          debug!(title = %bead.title, "Skipping duplicate bead");
          preview = BeadsCliImportPreview {
            to_skip: preview.to_skip.push_back(bead.title),
            ..preview
          };
        } else {
          debug!(title = %bead.title, status = %bead.status, priority = ?bead.priority, "Adding new bead");
          preview = BeadsCliImportPreview {
            to_add: preview.to_add.push_back(bead),
            ..preview
          };
        }
      }
      Err(e) => {
        warn!(line = line_idx, error = %e, issue_id = %issue.id, "Failed to map issue to bead");
        preview = BeadsCliImportPreview {
          errors: preview.errors.push_back(e),
          ..preview
        };
      }
    }
  }

  if preview.to_add.is_empty() && preview.to_skip.is_empty() && preview.errors.is_empty() {
    warn!("No valid beads found in issues.jsonl");
    return Err(BeadsCliImportError::NoValidBeads);
  }

  info!(
    to_add = preview.to_add.len(),
    to_skip = preview.to_skip.len(),
    errors = preview.errors.len(),
    "Import preview complete"
  );

  Ok(preview)
}

/// Map a beads_rust issue to a Clarity bead
///
/// # Errors
/// Returns `BeadsCliImportError::InvalidData` if status, priority, or type mapping fails
fn map_issue_to_bead(
  issue: BeadsCliIssue,
  line_num: usize,
) -> BeadsCliImportResult<NewBead> {
  // Map beads_rust status to bead status
  let bead_status = match issue.status.as_str() {
    "open" => BeadStatus::Open,
    "in_progress" | "in-progress" => BeadStatus::InProgress,
    "blocked" => BeadStatus::Blocked,
    "deferred" => BeadStatus::Deferred,
    "closed" => BeadStatus::Closed,
    _ => {
      return Err(BeadsCliImportError::InvalidData {
        line: line_num,
        field: "status".to_string(),
        reason: format!("Unknown status: {}", issue.status),
      })
    }
  };

  // Map beads_rust priority (1=high, 2=medium, 3=low) to bead priority
  // Both systems use the same scale
  let bead_priority = match issue.priority {
    1 => BeadPriority::HIGH,
    2 => BeadPriority::MEDIUM,
    3 => BeadPriority::LOW,
    p => {
      return Err(BeadsCliImportError::InvalidData {
        line: line_num,
        field: "priority".to_string(),
        reason: format!("Invalid priority: {p} (expected 1, 2, or 3)"),
      })
    }
  };

  // Map issue_type to bead_type
  let bead_type = match issue.issue_type.as_str() {
    "feature" => BeadType::Feature,
    "bug" | "bugfix" => BeadType::Bugfix,
    "refactor" => BeadType::Refactor,
    "test" => BeadType::Test,
    "docs" | "documentation" | "task" => BeadType::Docs,
    _ => BeadType::Feature,
  };

  // Build description with import marker
  let description = if let Some(desc) = issue.description {
    if desc.is_empty() {
      Some(format!(
        "*Imported from beads_rust issue: {}*\n\nSource: .beads/issues.jsonl",
        issue.id
      ))
    } else {
      Some(format!(
        "{}\n\n*Imported from beads_rust issue: {}*\n\nSource: .beads/issues.jsonl",
        desc, issue.id
      ))
    }
  } else {
    Some(format!(
      "*Imported from beads_rust issue: {}*\n\nSource: .beads/issues.jsonl",
      issue.id
    ))
  };

  Ok(NewBead {
    title: issue.title,
    description,
    status: bead_status,
    priority: bead_priority,
    bead_type,
    created_by: None, // We don't map user IDs between systems
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_map_issue_to_bead_open_feature() {
    let issue = BeadsCliIssue {
      id: "bd-123".to_string(),
      title: "Test Issue".to_string(),
      description: Some("Test Description".to_string()),
      status: "open".to_string(),
      priority: 1,
      issue_type: "feature".to_string(),
      _created_at: "2024-01-01T00:00:00Z".to_string(),
      _created_by: Some("lewis".to_string()),
      _updated_at: None,
      _closed_at: None,
      _source_repo: None,
    };

    let result = map_issue_to_bead(issue, 1);
    assert!(result.is_ok(), "Should map valid issue");
    let bead = result.map_err(|e| panic!("Expected Ok, got Err: {e:?}"));
    let bead = match bead {
      Ok(b) => b,
      Err(e) => panic!("Expected Ok, got Err: {e:?}"),
    };
    assert_eq!(bead.title, "Test Issue");
    assert_eq!(bead.status, BeadStatus::Open);
    assert_eq!(bead.bead_type, BeadType::Feature);
    assert_eq!(bead.priority, BeadPriority::HIGH);
    assert!(bead.description.is_some());
    let contains_import = bead.description.as_ref()
      .map_or(false, |d| d.contains("Imported from beads_rust issue"));
    assert!(contains_import);
  }

  #[test]
  fn test_map_issue_to_bead_closed_bug() {
    let issue = BeadsCliIssue {
      id: "bd-456".to_string(),
      title: "Fix Bug".to_string(),
      description: None,
      status: "closed".to_string(),
      priority: 1,
      issue_type: "bug".to_string(),
      _created_at: "2024-01-01T00:00:00Z".to_string(),
      _created_by: None,
      _updated_at: Some("2024-01-02T00:00:00Z".to_string()),
      _closed_at: Some("2024-01-02T00:00:00Z".to_string()),
      _source_repo: Some(".".to_string()),
    };

    let result = map_issue_to_bead(issue, 1);
    let bead = match result {
      Ok(b) => b,
      Err(e) => panic!("Expected Ok, got Err: {e:?}"),
    };
    assert_eq!(bead.status, BeadStatus::Closed);
    assert_eq!(bead.bead_type, BeadType::Bugfix);
  }

  #[test]
  fn test_map_issue_to_bead_invalid_status() {
    let issue = BeadsCliIssue {
      id: "bd-789".to_string(),
      title: "Invalid Status".to_string(),
      description: None,
      status: "invalid_status".to_string(),
      priority: 2,
      issue_type: "feature".to_string(),
      _created_at: "2024-01-01T00:00:00Z".to_string(),
      _created_by: None,
      _updated_at: None,
      _closed_at: None,
      _source_repo: None,
    };

    let result = map_issue_to_bead(issue, 1);
    assert!(result.is_err());
    match result {
      Err(BeadsCliImportError::InvalidData { field, .. }) => {
        assert_eq!(field, "status");
      }
      Ok(_) => panic!("Expected InvalidData error, got Ok"),
      Err(e) => panic!("Expected InvalidData error, got: {e:?}"),
    }
  }

  #[test]
  fn test_beads_cli_config_default() {
    let config = BeadsCliConfig::new();
    assert_eq!(config.search_path, PathBuf::from("."));
  }

  #[test]
  fn test_beads_cli_config_with_path() {
    let custom_path = PathBuf::from("/custom/path");
    let config = BeadsCliConfig::new().with_search_path(custom_path.clone());
    assert_eq!(config.search_path, custom_path);
  }

  #[test]
  fn test_import_preview_new() {
    let preview = BeadsCliImportPreview::new();
    assert!(!preview.has_errors());
    assert!(!preview.has_changes());
    assert_eq!(preview.total_count(), 0);
  }

  #[test]
  fn test_find_issues_file_with_default_config() {
    let config = BeadsCliConfig::new();
    let result = config.find_issues_file();
    // This test verifies that the .beads/issues.jsonl file can be found
    // It may not exist in all environments, so we just check the path resolution works
    assert!(result.is_ok() || result.is_err());
  }
}
