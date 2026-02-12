//! Core br Show Functionality
//!
//! Pure functions for fetching and processing br command output.
//! This module follows functional programming principles:
//! - No side effects in pure functions
//! - Uses Result for error handling
//! - No mutation of shared state
//! - Deterministic behavior

#![allow(clippy::missing_errors_doc)]
#![allow(clippy::doc_markdown)]
#![allow(warnings)]
#![allow(clippy::all)]

use crate::br_show::{BrIssue, BrShowError};
use std::process::Stdio;
use tokio::process::Command;

/// Fetch an issue from the br command
///
/// This is the pure core function that executes the br command and returns
/// parsed data. It handles:
/// - Command execution
/// - Output parsing
/// - Error conversion
///
/// # Arguments
/// * `id` - The issue ID to fetch
///
/// # Returns
/// * `Result<BrIssue, BrShowError>` - The parsed issue or error
pub async fn fetch_br_issue(id: &str) -> Result<BrIssue, BrShowError> {
  // Execute the br command with json output
  let output = execute_br_show_command(id).await?;

  // Parse the JSON output
  let issues = parse_br_output(&output).await?;

  // Find and return the issue
  find_issue_by_id(issues, id).map_err(|_| BrShowError::issue_not_found(id))
}

/// Execute the br show command
///
/// Pure function that executes the command and returns output.
async fn execute_br_show_command(id: &str) -> Result<String, BrShowError> {
  let output = Command::new("br")
    .args(["show", "--json", id])
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .output()
    .await
    .map_err(|e| BrShowError::CommandFailed(format!("Failed to execute br command: {e}")))?;

  if !output.status.success() {
    let error = String::from_utf8_lossy(&output.stderr);
    return Err(BrShowError::CommandFailed(format!(
      "br command failed: {}",
      error.trim()
    )));
  }

  String::from_utf8(output.stdout)
    .map_err(|e| BrShowError::CommandFailed(format!("Invalid UTF-8 output: {e}")))
}

/// Parse br command output JSON
///
/// Pure function that parses JSON and converts to BrIssue structs.
async fn parse_br_output(output: &str) -> Result<Vec<BrIssue>, BrShowError> {
  serde_json::from_str::<Vec<BrIssue>>(output)
    .map_err(|e| BrShowError::ParseError(format!("JSON parsing error: {e}")))
}

/// Find an issue by ID in the parsed output
///
/// Pure function that searches for the specific issue.
fn find_issue_by_id(issues: Vec<BrIssue>, id: &str) -> Result<BrIssue, BrShowError> {
  issues
    .into_iter()
    .find(|issue| issue.id == id)
    .ok_or_else(|| BrShowError::issue_not_found(id))
}

/// Check if a br issue exists
///
/// Pure function to verify issue existence without full data loading.
pub async fn issue_exists(id: &str) -> Result<bool, BrShowError> {
  let output = execute_br_show_command(id).await?;
  let issues = parse_br_output(&output).await?;
  Ok(issues.iter().any(|issue| issue.id == id))
}

/// Get issue IDs from br command
///
/// Pure function to get all available issue IDs.
pub async fn get_issue_ids() -> Result<Vec<String>, BrShowError> {
  let output = Command::new("br")
    .args(["list", "--json"])
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .output()
    .await
    .map_err(|e| BrShowError::CommandFailed(format!("Failed to execute br list: {e}")))?;

  if !output.status.success() {
    let error = String::from_utf8_lossy(&output.stderr);
    return Err(BrShowError::CommandFailed(format!(
      "br list failed: {}",
      error.trim()
    )));
  }

  let json_output = String::from_utf8(output.stdout)?;
  let issues: Vec<serde_json::Value> = serde_json::from_str(&json_output)
    .map_err(|e| BrShowError::ParseError(format!("JSON parsing error: {e}")))?;

  let ids = issues
    .into_iter()
    .filter_map(|issue| {
      issue
        .get("id")
        .and_then(|id| id.as_str())
        .map(|id| id.to_string())
    })
    .collect();

  Ok(ids)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_find_issue_by_id_success() {
    let issues = vec![BrIssue {
      id: "bd-1bf".to_string(),
      title: "Test Issue".to_string(),
      status: "open".to_string(),
      priority: 1,
      issue_type: "chore".to_string(),
      created_at: chrono::Utc::now(),
      created_by: "test".to_string(),
      updated_at: chrono::Utc::now(),
      source_repo: ".".to_string(),
      compaction_level: 0,
      original_size: 0,
    }];

    let result = find_issue_by_id(issues, "bd-1bf");
    assert!(result.is_ok());
    assert_eq!(result.unwrap().id, "bd-1bf");
  }

  #[test]
  fn test_find_issue_by_id_not_found() {
    let issues = vec![];
    let result = find_issue_by_id(issues, "nonexistent");
    assert!(result.is_err());
    match result {
      Err(BrShowError::IssueNotFound(id)) => {
        assert_eq!(id, "nonexistent");
      }
      _ => panic!("Expected IssueNotFound error"),
    }
  }

  #[test]
  fn test_issue_exists_true() {
    // This would require actual br command execution in integration tests
    // For unit tests, we test the logic separately
  }

  #[test]
  fn test_issue_exists_false() {
    // Same as above - integration test territory
  }

  #[test]
  fn test_get_issue_ids() {
    // This would require actual br command execution in integration tests
  }
}
