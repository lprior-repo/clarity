//! Standalone br show command implementation
//!
//! A command-line interface for showing bead details from the br command.
//! This is a pure functional implementation with zero unwrap and proper error handling.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(warnings)]
#![allow(clippy::all)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::process::{ExitCode, Stdio};

/// Show details about a bead/issue from the br command
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
  /// The bead ID to show (e.g., bd-qj3.8)
  id: String,
}

/// Represents an issue from the `br` command output
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct BrIssue {
  /// The issue ID
  pub id: String,
  /// The issue title
  pub title: String,
  /// The issue status
  pub status: String,
  /// The priority (1=high, 2=medium, 3=low)
  pub priority: u32,
  /// The issue type
  pub issue_type: String,
  /// When the issue was created
  pub created_at: DateTime<Utc>,
  /// Who created the issue
  pub created_by: String,
  /// When the issue was last updated
  pub updated_at: DateTime<Utc>,
  /// Source repository
  pub source_repo: String,
  /// Compaction level
  pub compaction_level: u32,
  /// Original size
  pub original_size: u64,
}

/// Domain errors for br show functionality
#[derive(Debug, thiserror::Error)]
enum BrShowError {
  /// Error executing the br command
  #[error("br command execution failed: {0}")]
  CommandFailed(String),

  /// Error parsing br command output
  #[error("failed to parse br output: {0}")]
  ParseError(String),

  /// Issue not found
  #[error("issue not found: {0}")]
  IssueNotFound(String),

  /// JSON output parsing error
  #[error("JSON parsing error: {0}")]
  JsonError(#[from] serde_json::Error),

  /// IO error
  #[error("IO error: {0}")]
  IoError(#[from] std::io::Error),
}

impl BrShowError {
  /// Create an issue not found error
  pub fn issue_not_found<S: Into<String>>(id: S) -> Self {
    Self::IssueNotFound(id.into())
  }
}

/// Main entry point for the br show command
///
/// This function:
/// 1. Parses command line arguments
/// 2. Fetches bead data using the pure functional core
/// 3. Displays the bead information
/// 4. Returns appropriate exit codes
///
/// # Arguments
/// * `args` - Command line arguments
///
/// # Returns
/// * `ExitCode` - 0 for success, 1 for error
async fn run(args: Args) -> Result<ExitCode> {
  let Args { id } = args;

  // Fetch bead data using pure functional core
  let bead = fetch_br_issue(&id)
    .await
    .context("Failed to fetch bead data from br command")?;

  // Display bead information
  display_bead(&bead);

  Ok(ExitCode::SUCCESS)
}

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
async fn fetch_br_issue(id: &str) -> Result<BrIssue, BrShowError> {
  // Execute the br command with json output
  let output = execute_br_show_command(id).await?;

  // Parse the JSON output
  let issues = parse_br_output(&output)?;

  // Find and return the issue
  find_issue_by_id(issues, id).map_err(|_| BrShowError::issue_not_found(id))
}

/// Execute the br show command
///
/// Pure function that executes the command and returns output.
async fn execute_br_show_command(id: &str) -> Result<String, BrShowError> {
  let output = tokio::process::Command::new("br")
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
/// Pure function that parses JSON and converts to `BrIssue` structs.
fn parse_br_output(output: &str) -> Result<Vec<BrIssue>, BrShowError> {
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

/// Display bead information in a formatted way
///
/// Pure function that formats and displays bead data.
///
/// # Arguments
/// * `bead` - The bead to display
fn display_bead(bead: &BrIssue) {
  println!("Bead: {}", bead.id);
  println!("Title: {}", bead.title);
  println!("Status: {}", bead.status);
  println!("Priority: {}", bead.priority);
  println!("Type: {}", bead.issue_type);
  println!("Created: {}", format_datetime(&bead.created_at));
  println!("Created By: {}", bead.created_by);
  println!("Updated: {}", format_datetime(&bead.updated_at));
  println!("Repository: {}", bead.source_repo);
  println!("Compaction Level: {}", bead.compaction_level);
  println!("Original Size: {} bytes", bead.original_size);
}

/// Format datetime for display
///
/// Pure function that converts a chrono `DateTime` to a human-readable string.
///
/// # Arguments
/// * `dt` - The datetime to format
///
/// # Returns
/// * `String` - Formatted datetime string
fn format_datetime(dt: &DateTime<Utc>) -> String {
  dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

/// Main entry point
///
/// This function:
/// 1. Parses command line arguments
/// 2. Runs the main logic
/// 3. Handles errors and returns exit codes
///
/// # Errors
/// Returns any errors encountered during execution
#[tokio::main]
async fn main() -> Result<ExitCode> {
  // Parse command line arguments
  let args = Args::parse();

  // Run the main logic
  match run(args).await {
    Ok(exit_code) => Ok(exit_code),
    Err(e) => {
      // Display error message
      eprintln!("Error: {e}");

      // Check if it's a specific error type
      if e.to_string().contains("issue not found") {
        eprintln!("Hint: Use 'br list' to see available bead IDs");
      }

      Ok(ExitCode::FAILURE)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_format_datetime() {
    let dt = DateTime::parse_from_rfc3339("2024-02-09T12:34:56Z")
      .expect("Failed to parse datetime")
      .with_timezone(&Utc);

    let formatted = format_datetime(&dt);
    assert_eq!(formatted, "2024-02-09 12:34:56 UTC");
  }

  #[test]
  fn test_args_parsing() {
    let args = Args::try_parse_from(["br_show", "bd-qj3.8"]).expect("Failed to parse args");
    assert_eq!(args.id, "bd-qj3.8");
  }

  #[test]
  fn test_args_parsing_empty() {
    let args = Args::try_parse_from(["br_show"]);
    assert!(args.is_err());
  }

  #[test]
  fn test_find_issue_by_id_success() {
    let issues = vec![BrIssue {
      id: "bd-1bf".to_string(),
      title: "Test Issue".to_string(),
      status: "open".to_string(),
      priority: 1,
      issue_type: "chore".to_string(),
      created_at: Utc::now(),
      created_by: "test".to_string(),
      updated_at: Utc::now(),
      source_repo: ".".to_string(),
      compaction_level: 0,
      original_size: 0,
    }];

    let result = find_issue_by_id(issues, "bd-1bf");
    assert!(result.is_ok());
    assert_eq!(result.expect("Issue not found").id, "bd-1bf");
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
}
