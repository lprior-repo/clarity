#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::redundant_async_block)]

//! Import from intent-cli
//!
//! This module provides functionality to import issues from intent-cli's
//! beads database into Clarity's bead system.

use clarity_core::db::models::{BeadPriority, BeadStatus, BeadType, NewBead};
use futures_util::{StreamExt, TryStreamExt};
use sqlx::{Row, SqlitePool};
use std::path::PathBuf;
use thiserror::Error;

/// Import errors
///
/// Domain errors for the import process using thiserror for
/// semantic error types.
#[derive(Debug, Error)]
pub enum ImportError {
  /// Failed to connect to the intent-cli database
  #[error("Failed to connect to intent-cli database at {path}: {source}")]
  DatabaseConnection {
    path: String,
    #[source]
    source: sqlx::Error,
  },

  /// Failed to query issues from intent-cli
  #[error("Failed to query issues from intent-cli: {0}")]
  QueryFailed(#[from] sqlx::Error),

  /// Invalid issue data in the database
  #[error("Invalid issue data (id={id}): {field} - {reason}")]
  InvalidIssueData {
    id: String,
    field: String,
    reason: String,
  },

  /// Failed to check for existing beads
  #[error("Failed to check for existing beads: {0}")]
  ExistingBeadCheck(#[source] clarity_core::db::error::DbError),

  /// Failed to create bead in Clarity database
  #[error("Failed to create bead '{title}': {source}")]
  BeadCreationFailed {
    title: String,
    #[source]
    source: clarity_core::db::error::DbError,
  },

  /// Intent-cli database not found at configured paths
  #[error("Intent-cli database not found. Tried paths: {paths:?}")]
  DatabaseNotFound { paths: Vec<String> },

  /// Failed to canonicalize database path
  #[error("Failed to resolve database path '{path}': {source}")]
  PathResolution {
    path: String,
    #[source]
    source: std::io::Error,
  },
}

/// Intent-cli issue representation
///
/// Pure data structure for representing an issue from intent-cli.
#[derive(Debug, Clone)]
struct IntentIssue {
  id: String,
  title: String,
  description: String,
  status: String,
  priority: i32,
  issue_type: String,
  acceptance_criteria: String,
  design: String,
  notes: String,
}

/// Intent-cli database configuration
///
/// Configurable paths for finding the intent-cli database.
#[derive(Debug, Clone)]
pub struct IntentDbConfig {
  /// Ordered list of paths to search for the intent-cli database
  pub search_paths: Vec<PathBuf>,
}

impl IntentDbConfig {
  /// Create a new configuration with default search paths
  #[must_use]
  pub fn new() -> Self {
    Self {
      search_paths: vec![
        // Relative to clarity project
        PathBuf::from("../intent-cli/.beads/beads.db"),
        // Current directory
        PathBuf::from("./intent-cli/.beads/beads.db"),
      ],
    }
  }

  /// Add a custom search path
  #[must_use]
  pub fn with_search_path(mut self, path: PathBuf) -> Self {
    self.search_paths.push(path);
    self
  }

  /// Replace search paths with a custom list
  #[must_use]
  pub fn with_search_paths(mut self, paths: Vec<PathBuf>) -> Self {
    self.search_paths = paths;
    self
  }

  /// Find the first existing database path
  ///
  /// # Errors
  /// Returns `ImportError::DatabaseNotFound` if no database exists
  /// at any of the configured search paths.
  pub fn find_existing_db(&self) -> Result<PathBuf, ImportError> {
    self
      .search_paths
      .iter()
      .filter(|path| path.exists())
      .map(|path| {
        path
          .canonicalize()
          .map_err(|e| ImportError::PathResolution {
            path: path.display().to_string(),
            source: e,
          })
      })
      .next()
      .ok_or_else(|| ImportError::DatabaseNotFound {
        paths: self
          .search_paths
          .iter()
          .map(|p| p.display().to_string())
          .collect(),
      })?
  }
}

impl Default for IntentDbConfig {
  fn default() -> Self {
    Self::new()
  }
}

/// Import issues from intent-cli database
///
/// This function reads issues from the intent-cli beads database
/// and imports them as beads into the Clarity database.
///
/// # Arguments
/// * `intent_cli_db_path` - Path to the intent-cli beads.db file
/// * `clarity_db` - The Clarity database connection
///
/// # Returns
/// Number of beads successfully imported
///
/// # Errors
/// Returns `ImportError` if:
/// - Database connection fails
/// - Query execution fails
/// - Issue data is invalid
/// - Bead creation fails
pub async fn import_from_intent_cli(
  intent_cli_db_path: PathBuf,
  clarity_db: &crate::db::DesktopDb,
) -> Result<usize, ImportError> {
  // Connect to intent-cli database
  let database_url = format!("sqlite:{}", intent_cli_db_path.display());
  let intent_pool =
    SqlitePool::connect(&database_url)
      .await
      .map_err(|e| ImportError::DatabaseConnection {
        path: intent_cli_db_path.display().to_string(),
        source: e,
      })?;

  // Query all active issues (not deleted)
  let issues = fetch_active_issues(&intent_pool).await?;

  // Transform issues to beads, filtering out already-imported ones
  let existing_beads = clarity_db
    .list_beads()
    .await
    .map_err(ImportError::ExistingBeadCheck)?;

  let import_results = issues
    .into_iter()
    .map(|issue| {
      let bead = map_issue_to_bead(issue)?;
      let is_duplicate = check_duplicate(&bead, &existing_beads);
      Ok((bead, is_duplicate))
    })
    .collect::<Result<Vec<_>, ImportError>>()?;

  // Filter out duplicates and import new beads
  let new_beads: Vec<_> = import_results
    .into_iter()
    .filter_map(|(bead, is_duplicate)| is_duplicate.then_some(bead))
    .collect();

  let imported_count = import_beads(new_beads, clarity_db).await?;

  Ok(imported_count)
}

/// Find the intent-cli database path using default configuration
///
/// Searches for the intent-cli beads.db in common locations.
///
/// # Errors
/// Returns `ImportError::DatabaseNotFound` if the database cannot be found
pub fn find_intent_cli_db() -> Result<PathBuf, ImportError> {
  IntentDbConfig::new().find_existing_db()
}

/// Fetch active (non-deleted) issues from intent-cli database
///
/// # Errors
/// Returns `ImportError::QueryFailed` if the query fails
async fn fetch_active_issues(pool: &SqlitePool) -> Result<Vec<IntentIssue>, ImportError> {
  let rows = sqlx::query(
    r"
        SELECT id, title, description, status, priority, issue_type,
               acceptance_criteria, design, notes
        FROM issues
        WHERE deleted_at IS NULL
        ORDER BY created_at DESC
        ",
  )
  .fetch_all(pool)
  .await?;

  rows
    .into_iter()
    .map(|row| {
      let id: String = row
        .try_get("id")
        .map_err(|e| ImportError::InvalidIssueData {
          id: "unknown".to_string(),
          field: "id".to_string(),
          reason: e.to_string(),
        })?;

      let title = row
        .try_get("title")
        .map_err(|e| ImportError::InvalidIssueData {
          id: id.clone(),
          field: "title".to_string(),
          reason: e.to_string(),
        })?;

      let description = row
        .try_get("description")
        .map_err(|e| ImportError::InvalidIssueData {
          id: id.clone(),
          field: "description".to_string(),
          reason: e.to_string(),
        })?;

      let status = row
        .try_get("status")
        .map_err(|e| ImportError::InvalidIssueData {
          id: id.clone(),
          field: "status".to_string(),
          reason: e.to_string(),
        })?;

      let priority = row
        .try_get("priority")
        .map_err(|e| ImportError::InvalidIssueData {
          id: id.clone(),
          field: "priority".to_string(),
          reason: e.to_string(),
        })?;

      let issue_type = row
        .try_get("issue_type")
        .map_err(|e| ImportError::InvalidIssueData {
          id: id.clone(),
          field: "issue_type".to_string(),
          reason: e.to_string(),
        })?;

      let acceptance_criteria =
        row
          .try_get("acceptance_criteria")
          .map_err(|e| ImportError::InvalidIssueData {
            id: id.clone(),
            field: "acceptance_criteria".to_string(),
            reason: e.to_string(),
          })?;

      let design = row
        .try_get("design")
        .map_err(|e| ImportError::InvalidIssueData {
          id: id.clone(),
          field: "design".to_string(),
          reason: e.to_string(),
        })?;

      let notes = row
        .try_get("notes")
        .map_err(|e| ImportError::InvalidIssueData {
          id: id.clone(),
          field: "notes".to_string(),
          reason: e.to_string(),
        })?;

      Ok(IntentIssue {
        id,
        title,
        description,
        status,
        priority,
        issue_type,
        acceptance_criteria,
        design,
        notes,
      })
    })
    .collect()
}

/// Map an intent-cli issue to a Clarity bead
///
/// # Errors
/// Returns `ImportError::InvalidIssueData` if status, priority, or type mapping fails
fn map_issue_to_bead(issue: IntentIssue) -> Result<NewBead, ImportError> {
  // Map intent-cli status to bead status
  let bead_status = match issue.status.as_str() {
    "open" => BeadStatus::Open,
    "in_progress" | "in-progress" => BeadStatus::InProgress,
    "blocked" => BeadStatus::Blocked,
    "deferred" => BeadStatus::Deferred,
    "closed" => BeadStatus::Closed,
    _ => {
      return Err(ImportError::InvalidIssueData {
        id: issue.id.clone(),
        field: "status".to_string(),
        reason: format!("Unknown status: {}", issue.status),
      })
    }
  };

  // Map intent-cli priority (0-4) to bead priority (1-3)
  // intent-cli: 0=critical, 1=high, 2=medium, 3=low, 4=none
  // clarity: 1=high, 2=medium, 3=low
  let bead_priority = match issue.priority {
    0 | 1 => BeadPriority::High,
    2 => BeadPriority::Medium,
    3 => BeadPriority::Low,
    _ => BeadPriority::Medium,
  };

  // Map issue_type to bead_type
  let bead_type = match issue.issue_type.as_str() {
    "feature" => BeadType::Feature,
    "bug" | "bugfix" => BeadType::Bugfix,
    "refactor" => BeadType::Refactor,
    "test" => BeadType::Test,
    "docs" | "documentation" => BeadType::Docs,
    "epic" => BeadType::Feature,
    _ => BeadType::Feature,
  };

  // Build description with additional context using functional composition
  let description_parts = [
    (!issue.description.is_empty()).then_some(issue.description),
    (!issue.acceptance_criteria.is_empty()).then_some(format!(
      "\n\n**Acceptance Criteria:**\n{}",
      issue.acceptance_criteria
    )),
    (!issue.design.is_empty()).then_some(format!("\n\n**Design:**\n{}", issue.design)),
    (!issue.notes.is_empty()).then_some(format!("\n\n**Notes:**\n{}", issue.notes)),
    Some(format!(
      "\n\n*Imported from intent-cli issue: {}*",
      issue.id
    )),
  ];

  let full_description =
    description_parts
      .into_iter()
      .flatten()
      .fold(String::new(), |mut acc, part| {
        acc.push_str(&part);
        acc
      });

  Ok(NewBead {
    title: format!("[intent-cli] {}", issue.title),
    description: Some(full_description),
    status: bead_status,
    priority: bead_priority,
    bead_type,
    created_by: None,
  })
}

/// Check if a bead already exists in the database
///
/// Uses title matching and description substring matching to detect duplicates.
#[must_use]
fn check_duplicate(bead: &NewBead, existing_beads: &[clarity_core::db::models::Bead]) -> bool {
  existing_beads
    .iter()
    .filter(|b| b.title == bead.title)
    .filter(|b| {
      b.description
        .as_ref()
        .is_some_and(|d| d.contains("Imported from intent-cli issue:"))
    })
    .count()
    == 0
}

/// Import multiple beads into the Clarity database
///
/// Returns the count of successfully imported beads.
///
/// # Errors
/// Returns `ImportError::BeadCreationFailed` if any bead creation fails
async fn import_beads(
  beads: Vec<NewBead>,
  clarity_db: &crate::db::DesktopDb,
) -> Result<usize, ImportError> {
  // Process each bead sequentially, counting successful imports
  let count = futures_util::stream::iter(beads)
    .map(|bead| {
      let title = bead.title.clone();
      async move {
        clarity_db
          .create_bead(bead)
          .await
          .map_err(|e| ImportError::BeadCreationFailed {
            title: title.clone(),
            source: e,
          })
      }
    })
    .then(|future| async move { future.await })
    .try_fold(0, |acc, _| async move { Ok(acc + 1) })
    .await?;

  Ok(count)
}

#[cfg(test)]
mod tests {
  use super::*;
  use clarity_core::db::models::{BeadId, BeadStatus, BeadType};

  #[test]
  fn test_map_issue_to_bead_open() {
    let issue = IntentIssue {
      id: "test-123".to_string(),
      title: "Test Issue".to_string(),
      description: "Test Description".to_string(),
      status: "open".to_string(),
      priority: 1,
      issue_type: "feature".to_string(),
      acceptance_criteria: String::new(),
      design: String::new(),
      notes: String::new(),
    };

    let result = map_issue_to_bead(issue);
    let bead = match result {
      Ok(b) => b,
      Err(e) => panic!("Should map valid issue, got Err: {e:?}"),
    };
    assert_eq!(bead.title, "[intent-cli] Test Issue");
    assert_eq!(bead.status, BeadStatus::Open);
    assert_eq!(bead.bead_type, BeadType::Feature);
    assert!(bead.description.is_some());
  }

  #[test]
  fn test_map_issue_to_bead_in_progress() {
    let issue = IntentIssue {
      id: "test-124".to_string(),
      title: "In Progress Issue".to_string(),
      description: "Test".to_string(),
      status: "in_progress".to_string(),
      priority: 2,
      issue_type: "bug".to_string(),
      acceptance_criteria: String::new(),
      design: String::new(),
      notes: String::new(),
    };

    let result = map_issue_to_bead(issue);
    let bead = match result {
      Ok(b) => b,
      Err(e) => panic!("Should map in_progress, got Err: {e:?}"),
    };
    assert_eq!(bead.status, BeadStatus::InProgress);
    assert_eq!(bead.bead_type, BeadType::Bugfix);
  }

  #[test]
  fn test_map_issue_to_bead_invalid_status() {
    let issue = IntentIssue {
      id: "test-125".to_string(),
      title: "Invalid Status Issue".to_string(),
      description: "Test".to_string(),
      status: "invalid_status".to_string(),
      priority: 2,
      issue_type: "feature".to_string(),
      acceptance_criteria: String::new(),
      design: String::new(),
      notes: String::new(),
    };

    let result = map_issue_to_bead(issue);
    assert!(result.is_err());
    match result {
      Err(ImportError::InvalidIssueData { field, .. }) => {
        assert_eq!(field, "status");
      }
      Ok(_) => panic!("Expected InvalidIssueData error, got Ok"),
      Err(e) => panic!("Expected InvalidIssueData error, got: {e:?}"),
    }
  }

  #[test]
  fn test_check_duplicate_no_existing() {
    let bead = NewBead {
      title: "New Bead".to_string(),
      description: Some("Test".to_string()),
      status: BeadStatus::Open,
      priority: BeadPriority::MEDIUM,
      bead_type: BeadType::Feature,
      created_by: None,
    };

    let existing = [];
    let is_new = check_duplicate(&bead, &existing);
    assert!(is_new, "Should be new when no existing beads");
  }

  #[test]
  fn test_check_duplicate_with_match() {
    let bead = NewBead {
      title: "Existing Bead".to_string(),
      description: Some("Test".to_string()),
      status: BeadStatus::Open,
      priority: BeadPriority::MEDIUM,
      bead_type: BeadType::Feature,
      created_by: None,
    };

    let now_str = chrono::Utc::now().to_rfc3339();
    let existing = vec![clarity_core::db::models::Bead {
      id: BeadId::new(),
      title: "Existing Bead".to_string(),
      description: Some("Imported from intent-cli issue: 123".to_string()),
      status: BeadStatus::Open,
      priority: BeadPriority::MEDIUM,
      bead_type: BeadType::Feature,
      created_by: None,
      created_at: now_str.clone(),
      updated_at: now_str,
    }];

    let is_new = check_duplicate(&bead, &existing);
    assert!(!is_new, "Should not be new when matching bead exists");
  }

  #[test]
  fn test_check_duplicate_different_title() {
    let bead = NewBead {
      title: "New Bead".to_string(),
      description: Some("Test".to_string()),
      status: BeadStatus::Open,
      priority: BeadPriority::MEDIUM,
      bead_type: BeadType::Feature,
      created_by: None,
    };

    let now_str = chrono::Utc::now().to_rfc3339();
    let existing = vec![clarity_core::db::models::Bead {
      id: BeadId::new(),
      title: "Different Bead".to_string(),
      description: Some("Imported from intent-cli issue: 123".to_string()),
      status: BeadStatus::Open,
      priority: BeadPriority::MEDIUM,
      bead_type: BeadType::Feature,
      created_by: None,
      created_at: now_str.clone(),
      updated_at: now_str,
    }];

    let is_new = check_duplicate(&bead, &existing);
    assert!(is_new, "Should be new when titles don't match");
  }

  #[test]
  fn test_intent_db_config_default() {
    let config = IntentDbConfig::new();
    assert!(!config.search_paths.is_empty());
    assert!(config
      .search_paths
      .iter()
      .any(|p| p.to_string_lossy().contains("intent-cli")));
  }

  #[test]
  fn test_intent_db_config_with_custom_path() {
    let custom_path = PathBuf::from("/custom/path/to/beads.db");
    let config = IntentDbConfig::new().with_search_path(custom_path.clone());
    assert!(config.search_paths.contains(&custom_path));
  }

  #[test]
  fn test_intent_db_config_replace_paths() {
    let paths = vec![PathBuf::from("/path1/db"), PathBuf::from("/path2/db")];
    let config = IntentDbConfig::new().with_search_paths(paths.clone());
    assert_eq!(config.search_paths, paths);
  }
}
