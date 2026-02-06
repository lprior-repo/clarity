#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::float_cmp)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::match_same_arms)]

//! Progress tracking and dashboard module for Clarity
//!
//! This module provides:
//! - Progress metrics calculation for beads and sessions
//! - Dashboard display formatting
//! - Progress output utilities for different formats (terminal, JSON, etc.)
//! - Progress visualization helpers
//!
//! All functions follow functional programming principles:
//! - Pure functions with no side effects
//! - Result types for error handling
//! - Immutable data structures
//! - No unwraps or panics

use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// Progress status for an item (bead or session)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProgressStatus {
  /// Item is not started
  NotStarted,
  /// Item is in progress
  InProgress,
  /// Item is completed
  Completed,
  /// Item is blocked or failed
  Blocked,
  /// Item is deferred or not applicable
  Deferred,
}

impl ProgressStatus {
  /// Get all possible progress statuses
  #[must_use]
  pub const fn all() -> [Self; 5] {
    [
      Self::NotStarted,
      Self::InProgress,
      Self::Completed,
      Self::Blocked,
      Self::Deferred,
    ]
  }

  /// Get the count of all possible statuses
  #[must_use]
  pub const fn count() -> usize {
    5
  }

  /// Check if this status is terminal (completed)
  #[must_use]
  pub const fn is_completed(self) -> bool {
    matches!(self, Self::Completed)
  }

  /// Check if this status is active (not completed or blocked)
  #[must_use]
  pub const fn is_active(self) -> bool {
    matches!(self, Self::InProgress)
  }

  /// Check if this status is not started
  #[must_use]
  pub const fn is_not_started(self) -> bool {
    matches!(self, Self::NotStarted)
  }

  /// Check if this status is blocked
  #[must_use]
  pub const fn is_blocked(self) -> bool {
    matches!(self, Self::Blocked)
  }

  /// Check if this status is deferred
  #[must_use]
  pub const fn is_deferred(self) -> bool {
    matches!(self, Self::Deferred)
  }
}

impl Display for ProgressStatus {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::NotStarted => write!(f, "not started"),
      Self::InProgress => write!(f, "in progress"),
      Self::Completed => write!(f, "completed"),
      Self::Blocked => write!(f, "blocked"),
      Self::Deferred => write!(f, "deferred"),
    }
  }
}

/// Progress metrics for a collection of items
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProgressMetrics {
  /// Total number of items tracked
  pub total: usize,

  /// Number of items completed
  pub completed: usize,

  /// Number of items in progress
  pub in_progress: usize,

  /// Number of items blocked
  pub blocked: usize,

  /// Number of items deferred
  pub deferred: usize,

  /// Number of items not started
  pub not_started: usize,

  /// Completion percentage (0-100)
  pub completion_percentage: f64,

  /// Progress status distribution
  pub status_distribution: ProgressDistribution,
}

/// Distribution of progress statuses
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProgressDistribution {
  /// Percentage of completed items
  pub completed_pct: f64,

  /// Percentage of in-progress items
  pub in_progress_pct: f64,

  /// Percentage of blocked items
  pub blocked_pct: f64,

  /// Percentage of deferred items
  pub deferred_pct: f64,

  /// Percentage of not-started items
  pub not_started_pct: f64,
}

impl ProgressMetrics {
  /// Create new progress metrics from a status count map
  ///
  /// # Errors
  ///
  /// Returns `ProgressError::InvalidCount` if total count doesn't match sum of all statuses
  pub fn new(
    total: usize,
    completed: usize,
    in_progress: usize,
    blocked: usize,
    deferred: usize,
    not_started: usize,
  ) -> Result<Self, ProgressError> {
    if total != completed + in_progress + blocked + deferred + not_started {
      return Err(ProgressError::InvalidCount {
        total,
        sum: completed + in_progress + blocked + deferred + not_started,
      });
    }

    #[allow(clippy::cast_precision_loss)]
    let completion_percentage = if total > 0 {
      let completed_f64 = completed as f64;
      let total_f64 = total as f64;
      (completed_f64 / total_f64) * 100.0
    } else {
      0.0
    };

    #[allow(clippy::cast_precision_loss)]
    let status_distribution = ProgressDistribution {
      completed_pct: if total > 0 {
        completion_percentage
      } else {
        0.0
      },
      in_progress_pct: if total > 0 {
        let in_progress_f64 = in_progress as f64;
        let total_f64 = total as f64;
        (in_progress_f64 / total_f64) * 100.0
      } else {
        0.0
      },
      blocked_pct: if total > 0 {
        let blocked_f64 = blocked as f64;
        let total_f64 = total as f64;
        (blocked_f64 / total_f64) * 100.0
      } else {
        0.0
      },
      deferred_pct: if total > 0 {
        let deferred_f64 = deferred as f64;
        let total_f64 = total as f64;
        (deferred_f64 / total_f64) * 100.0
      } else {
        0.0
      },
      not_started_pct: if total > 0 {
        let not_started_f64 = not_started as f64;
        let total_f64 = total as f64;
        (not_started_f64 / total_f64) * 100.0
      } else {
        0.0
      },
    };

    Ok(Self {
      total,
      completed,
      in_progress,
      blocked,
      deferred,
      not_started,
      completion_percentage,
      status_distribution,
    })
  }

  /// Calculate metrics from a slice of `ProgressStatus` values
  #[must_use]
  pub fn from_statuses(statuses: &[ProgressStatus]) -> Self {
    let counts = statuses.iter().fold(
      (0usize, 0usize, 0usize, 0usize, 0usize),
      |(completed, in_progress, blocked, deferred, not_started), status| match status {
        ProgressStatus::Completed => (completed + 1, in_progress, blocked, deferred, not_started),
        ProgressStatus::InProgress => (completed, in_progress + 1, blocked, deferred, not_started),
        ProgressStatus::Blocked => (completed, in_progress, blocked + 1, deferred, not_started),
        ProgressStatus::Deferred => (completed, in_progress, blocked, deferred + 1, not_started),
        ProgressStatus::NotStarted => (completed, in_progress, blocked, deferred, not_started + 1),
      },
    );

    Self::new(
      counts.0 + counts.1 + counts.2 + counts.3 + counts.4,
      counts.0,
      counts.1,
      counts.2,
      counts.3,
      counts.4,
    )
    .unwrap_or_else(|_| Self::empty())
  }

  /// Create empty progress metrics
  #[must_use]
  pub const fn empty() -> Self {
    Self {
      total: 0,
      completed: 0,
      in_progress: 0,
      blocked: 0,
      deferred: 0,
      not_started: 0,
      completion_percentage: 0.0,
      status_distribution: ProgressDistribution {
        completed_pct: 0.0,
        in_progress_pct: 0.0,
        blocked_pct: 0.0,
        deferred_pct: 0.0,
        not_started_pct: 0.0,
      },
    }
  }

  /// Check if progress is complete (100% completion)
  #[must_use]
  pub fn is_complete(&self) -> bool {
    self.completion_percentage >= 100.0
  }

  /// Check if progress is stalled (no progress and some work remaining)
  #[must_use]
  pub fn is_stalled(&self) -> bool {
    self.in_progress == 0 && self.completed < self.total && self.total > 0
  }

  /// Get the number of remaining items to complete
  #[must_use]
  pub fn remaining_items(&self) -> usize {
    self.total.saturating_sub(self.completed)
  }
}

impl Display for ProgressMetrics {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
            f,
            "Progress: {}/{}, {:.1}%, completed: {}, in progress: {}, blocked: {}, deferred: {}, not started: {}",
            self.completed,
            self.total,
            self.completion_percentage,
            self.completed,
            self.in_progress,
            self.blocked,
            self.deferred,
            self.not_started
        )
  }
}

/// Progress dashboard display
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProgressDashboard {
  /// Title of the dashboard
  pub title: String,

  /// Overall progress metrics
  pub metrics: ProgressMetrics,

  /// Detailed breakdown by category (e.g., by session type)
  pub category_breakdown: Vec<CategoryProgress>,

  /// Timestamp when dashboard was generated
  pub generated_at: i64,
}

/// Progress breakdown by category
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CategoryProgress {
  /// Category name
  pub category: String,

  /// Items in this category
  pub total: usize,

  /// Progress metrics for this category
  pub metrics: ProgressMetrics,
}

impl Display for ProgressDashboard {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    writeln!(f, "=== {} ===", self.title)?;
    writeln!(f, "Generated: {}", self.generated_at)?;
    writeln!(f)?;
    writeln!(f, "{}", self.metrics)?;
    if !self.category_breakdown.is_empty() {
      writeln!(f)?;
      writeln!(f, "Category Breakdown:")?;
      for category in &self.category_breakdown {
        writeln!(f, "  - {}: {}", category.category, category.metrics)?;
      }
    }
    Ok(())
  }
}

/// Progress output format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProgressFormat {
  /// Terminal/CLI format with progress bars
  Terminal,
  /// JSON format for machine-readable output
  Json,
  /// Markdown format for documentation
  Markdown,
}

impl Display for ProgressFormat {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Terminal => write!(f, "terminal"),
      Self::Json => write!(f, "json"),
      Self::Markdown => write!(f, "markdown"),
    }
  }
}

/// Progress output options
#[derive(Debug, Clone)]
pub struct ProgressOutputOptions {
  /// Format for output
  pub format: ProgressFormat,
  /// Show category breakdown
  pub show_categories: bool,
  /// Show detailed metrics
  pub show_details: bool,
}

impl Default for ProgressOutputOptions {
  fn default() -> Self {
    Self {
      format: ProgressFormat::Terminal,
      show_categories: true,
      show_details: true,
    }
  }
}

/// Errors that can occur when calculating progress
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProgressError {
  /// Invalid count values that don't sum to total
  InvalidCount { total: usize, sum: usize },
  /// JSON serialization failed
  SerializationFailed(String),
}

impl Display for ProgressError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::InvalidCount { total, sum } => {
        write!(f, "invalid count values: total={total}, sum={sum}")
      }
      Self::SerializationFailed(msg) => {
        write!(f, "{msg}")
      }
    }
  }
}

impl std::error::Error for ProgressError {}

/// Calculate progress metrics from a collection of items
///
/// This is a pure function that takes a slice of `ProgressStatus` values
/// and returns the calculated metrics.
///
/// # Examples
///
/// ```
/// use clarity_core::progress::{ProgressStatus, ProgressMetrics};
///
/// let statuses = vec![
///     ProgressStatus::Completed,
///     ProgressStatus::InProgress,
///     ProgressStatus::NotStarted,
///     ProgressStatus::Completed,
/// ];
///
/// let metrics = ProgressMetrics::from_statuses(&statuses);
/// assert_eq!(metrics.total, 4);
/// assert_eq!(metrics.completed, 2);
/// assert_eq!(metrics.in_progress, 1);
/// assert_eq!(metrics.not_started, 1);
/// ```
#[must_use]
pub fn calculate_progress(statuses: &[ProgressStatus]) -> ProgressMetrics {
  ProgressMetrics::from_statuses(statuses)
}

/// Format progress metrics as a terminal-friendly string
///
/// Returns a string with progress bars and status indicators.
///
/// # Examples
///
/// ```
/// use clarity_core::progress::{ProgressStatus, ProgressMetrics, format_terminal_progress};
///
/// let metrics = ProgressMetrics::new(10, 7, 2, 0, 1, 0).unwrap();
/// let output = format_terminal_progress(&metrics);
/// assert!(output.contains("[============================            ] 70.0%"));
/// ```
#[must_use]
pub fn format_terminal_progress(metrics: &ProgressMetrics) -> String {
  let bar_length = 40;
  #[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
  )]
  let filled = (metrics.completed as f64 / metrics.total as f64 * bar_length as f64) as usize;
  let remaining = bar_length - filled;

  let progress_bar = format!(
    "[{}{}] {:.1}%",
    "=".repeat(filled),
    " ".repeat(remaining),
    metrics.completion_percentage
  );

  format!(
    "{}\n\nCompleted: {} | In Progress: {} | Blocked: {} | Deferred: {} | Not Started: {}",
    progress_bar,
    metrics.completed,
    metrics.in_progress,
    metrics.blocked,
    metrics.deferred,
    metrics.not_started
  )
}

/// Format progress metrics as JSON
///
/// Returns a JSON string representation of the metrics.
///
/// # Errors
///
/// Returns `ProgressError::SerializationFailed` if JSON serialization fails
///
/// # Examples
///
/// ```
/// use clarity_core::progress::{ProgressStatus, ProgressMetrics, format_json_progress};
///
/// let metrics = ProgressMetrics::new(10, 7, 2, 0, 1, 0).unwrap();
/// let json = format_json_progress(&metrics).unwrap();
/// assert!(json.contains("\"completed\":7"));
/// ```
pub fn format_json_progress(metrics: &ProgressMetrics) -> Result<String, ProgressError> {
  serde_json::to_string(metrics)
    .map_err(|e| ProgressError::SerializationFailed(format!("JSON serialization failed: {e}")))
}

/// Format progress metrics as Markdown
///
/// Returns a Markdown table representation of the metrics.
///
/// # Examples
///
/// ```
/// use clarity_core::progress::{ProgressStatus, ProgressMetrics, format_markdown_progress};
///
/// let metrics = ProgressMetrics::new(10, 7, 2, 0, 1, 0).unwrap();
/// let md = format_markdown_progress(&metrics);
/// assert!(md.contains("| Metric | Value |"));
/// ```
#[must_use]
pub fn format_markdown_progress(metrics: &ProgressMetrics) -> String {
  format!(
    "# Progress Dashboard\n\n\
        ## Overview\n\n\
        | Metric | Value |\n\
        |--------|-------|\n\
        | Total | {} |\n\
        | Completed | {} |\n\
        | In Progress | {} |\n\
        | Blocked | {} |\n\
        | Deferred | {} |\n\
        | Not Started | {} |\n\
        | Completion | {:.1}% |\n\n\
        ## Status Distribution\n\n\
        | Status | Count | Percentage |\n\
        |--------|-------|------------|\n\
        | Completed | {} | {:.1}% |\n\
        | In Progress | {} | {:.1}% |\n\
        | Blocked | {} | {:.1}% |\n\
        | Deferred | {} | {:.1}% |\n\
        | Not Started | {} | {:.1}% |\n",
    metrics.total,
    metrics.completed,
    metrics.in_progress,
    metrics.blocked,
    metrics.deferred,
    metrics.not_started,
    metrics.status_distribution.completed_pct,
    metrics.completed,
    metrics.status_distribution.completed_pct,
    metrics.in_progress,
    metrics.status_distribution.in_progress_pct,
    metrics.blocked,
    metrics.status_distribution.blocked_pct,
    metrics.deferred,
    metrics.status_distribution.deferred_pct,
    metrics.not_started,
    metrics.status_distribution.not_started_pct
  )
}

/// Generate a progress dashboard from progress metrics
///
/// # Examples
///
/// ```
/// use clarity_core::progress::{ProgressStatus, ProgressMetrics, ProgressDashboard, generate_dashboard};
///
/// let metrics = ProgressMetrics::new(10, 7, 2, 0, 1, 0).unwrap();
/// let dashboard = generate_dashboard("Project Progress".to_string(), metrics, vec![]);
/// assert_eq!(dashboard.title, "Project Progress");
/// ```
#[must_use]
pub fn generate_dashboard(
  title: String,
  metrics: ProgressMetrics,
  category_breakdown: Vec<CategoryProgress>,
) -> ProgressDashboard {
  ProgressDashboard {
    title,
    metrics,
    category_breakdown,
    generated_at: (std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .map(|d| d.as_secs().cast_signed())
      .unwrap_or(i64::MIN)),
  }
}

/// Format progress dashboard based on output options
///
/// # Errors
///
/// Returns `ProgressError::SerializationFailed` if JSON serialization fails when format is Json
///
/// # Examples
///
/// ```
/// use clarity_core::progress::{ProgressStatus, ProgressMetrics, ProgressFormat, ProgressOutputOptions, format_progress};
///
/// let metrics = ProgressMetrics::new(10, 7, 2, 0, 1, 0).unwrap();
/// let output = format_progress(&metrics, ProgressOutputOptions {
///     format: ProgressFormat::Terminal,
///     ..Default::default()
/// }).unwrap();
/// assert!(output.contains("[============================            ] 70.0%"));
/// ```
pub fn format_progress(
  metrics: &ProgressMetrics,
  options: &ProgressOutputOptions,
) -> Result<String, ProgressError> {
  match options.format {
    ProgressFormat::Terminal => Ok(format_terminal_progress(metrics)),
    ProgressFormat::Json => format_json_progress(metrics),
    ProgressFormat::Markdown => Ok(format_markdown_progress(metrics)),
  }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
#[allow(clippy::expect_used)]
#[allow(clippy::panic)]
#[allow(clippy::uninlined_format_args)]
#[allow(clippy::single_char_pattern)]
mod tests {
  use super::*;
  #[test]
  fn test_progress_status_all() {
    let statuses = ProgressStatus::all();
    assert_eq!(statuses.len(), 5);
    assert!(statuses.contains(&ProgressStatus::NotStarted));
    assert!(statuses.contains(&ProgressStatus::InProgress));
    assert!(statuses.contains(&ProgressStatus::Completed));
    assert!(statuses.contains(&ProgressStatus::Blocked));
    assert!(statuses.contains(&ProgressStatus::Deferred));
  }

  #[test]
  fn test_progress_status_count() {
    assert_eq!(ProgressStatus::count(), 5);
  }

  #[test]
  fn test_progress_status_is_completed() {
    assert!(ProgressStatus::Completed.is_completed());
    assert!(!ProgressStatus::InProgress.is_completed());
    assert!(!ProgressStatus::NotStarted.is_completed());
  }

  #[test]
  fn test_progress_status_is_active() {
    assert!(ProgressStatus::InProgress.is_active());
    assert!(!ProgressStatus::Completed.is_active());
    assert!(!ProgressStatus::NotStarted.is_active());
  }

  #[test]
  fn test_progress_status_is_not_started() {
    assert!(ProgressStatus::NotStarted.is_not_started());
    assert!(!ProgressStatus::Completed.is_not_started());
  }

  #[test]
  fn test_progress_status_is_blocked() {
    assert!(ProgressStatus::Blocked.is_blocked());
    assert!(!ProgressStatus::Completed.is_blocked());
  }

  #[test]
  fn test_progress_status_is_deferred() {
    assert!(ProgressStatus::Deferred.is_deferred());
    assert!(!ProgressStatus::Completed.is_deferred());
  }

  #[test]
  fn test_progress_status_display() {
    assert_eq!(format!("{}", ProgressStatus::NotStarted), "not started");
    assert_eq!(format!("{}", ProgressStatus::InProgress), "in progress");
    assert_eq!(format!("{}", ProgressStatus::Completed), "completed");
    assert_eq!(format!("{}", ProgressStatus::Blocked), "blocked");
    assert_eq!(format!("{}", ProgressStatus::Deferred), "deferred");
  }

  #[test]
  fn test_progress_metrics_new_valid() {
    let metrics = ProgressMetrics::new(10, 7, 2, 0, 1, 0).unwrap();
    assert_eq!(metrics.total, 10);
    assert_eq!(metrics.completed, 7);
    assert_eq!(metrics.in_progress, 2);
    assert_eq!(metrics.blocked, 0);
    assert_eq!(metrics.deferred, 1);
    assert_eq!(metrics.not_started, 0);
    assert_eq!(metrics.completion_percentage, 70.0);
  }

  #[test]
  fn test_progress_metrics_new_invalid() {
    let result = ProgressMetrics::new(10, 5, 3, 2, 1, 0);
    assert!(result.is_err());
    match result {
      Err(ProgressError::InvalidCount { total, sum }) => {
        assert_eq!(total, 10);
        assert_eq!(sum, 11);
      }
      _ => panic!("Expected InvalidCount error"),
    }
  }

  #[test]
  fn test_progress_metrics_from_statuses() {
    let statuses = vec![
      ProgressStatus::Completed,
      ProgressStatus::InProgress,
      ProgressStatus::NotStarted,
      ProgressStatus::Completed,
    ];

    let metrics = ProgressMetrics::from_statuses(&statuses);
    assert_eq!(metrics.total, 4);
    assert_eq!(metrics.completed, 2);
    assert_eq!(metrics.in_progress, 1);
    assert_eq!(metrics.not_started, 1);
    assert_eq!(metrics.blocked, 0);
    assert_eq!(metrics.deferred, 0);
  }

  #[test]
  fn test_progress_metrics_empty() {
    let metrics = ProgressMetrics::empty();
    assert_eq!(metrics.total, 0);
    assert_eq!(metrics.completed, 0);
    assert_eq!(metrics.in_progress, 0);
    assert_eq!(metrics.blocked, 0);
    assert_eq!(metrics.deferred, 0);
    assert_eq!(metrics.not_started, 0);
    assert_eq!(metrics.completion_percentage, 0.0);
  }

  #[test]
  fn test_progress_metrics_is_complete() {
    assert!(ProgressMetrics::new(10, 10, 0, 0, 0, 0)
      .unwrap()
      .is_complete());
    assert!(!ProgressMetrics::new(10, 9, 1, 0, 0, 0)
      .unwrap()
      .is_complete());
  }

  #[test]
  fn test_progress_metrics_is_stalled() {
    assert!(ProgressMetrics::new(10, 8, 0, 0, 0, 2)
      .unwrap()
      .is_stalled());
    assert!(!ProgressMetrics::new(10, 8, 2, 0, 0, 0)
      .unwrap()
      .is_stalled());
    assert!(!ProgressMetrics::new(10, 10, 0, 0, 0, 0)
      .unwrap()
      .is_stalled());
    assert!(ProgressMetrics::new(10, 0, 0, 0, 0, 10)
      .unwrap()
      .is_stalled());
  }

  #[test]
  fn test_progress_metrics_remaining_items() {
    assert_eq!(
      ProgressMetrics::new(10, 7, 2, 0, 1, 0)
        .unwrap()
        .remaining_items(),
      3
    );
    assert_eq!(
      ProgressMetrics::new(10, 10, 0, 0, 0, 0)
        .unwrap()
        .remaining_items(),
      0
    );
    assert_eq!(
      ProgressMetrics::new(10, 0, 0, 0, 0, 10)
        .unwrap()
        .remaining_items(),
      10
    );
  }

  #[test]
  fn test_progress_metrics_display() {
    let metrics = ProgressMetrics::new(10, 7, 2, 0, 1, 0).unwrap();
    let display = format!("{}", metrics);
    assert!(display.contains("Progress: 7/10"));
    assert!(display.contains("70.0%"));
  }

  #[test]
  fn test_progress_metrics_json() {
    let metrics = ProgressMetrics::new(10, 7, 2, 0, 1, 0).unwrap();
    let json = format_json_progress(&metrics).unwrap();
    assert!(json.contains("\"total\":10"));
    assert!(json.contains("\"completed\":7"));
    assert!(json.contains("\"in_progress\":2"));
  }

  #[test]
  fn test_progress_metrics_markdown() {
    let metrics = ProgressMetrics::new(10, 7, 2, 0, 1, 0).unwrap();
    let md = format_markdown_progress(&metrics);
    assert!(md.contains("# Progress Dashboard"));
    assert!(md.contains("| Metric | Value |"));
    assert!(md.contains("| Completed | 7 |"));
  }

  #[test]
  fn test_progress_metrics_terminal() {
    let metrics = ProgressMetrics::new(10, 7, 2, 0, 1, 0).unwrap();
    let terminal = format_terminal_progress(&metrics);
    assert!(terminal.contains("["));
    assert!(terminal.contains("]"));
    assert!(terminal.contains("70.0%"));
    assert!(terminal.contains("Completed: 7"));
  }

  #[test]
  fn test_calculate_progress() {
    let statuses = vec![
      ProgressStatus::Completed,
      ProgressStatus::InProgress,
      ProgressStatus::NotStarted,
    ];

    let metrics = calculate_progress(&statuses);
    assert_eq!(metrics.total, 3);
    assert_eq!(metrics.completed, 1);
    assert_eq!(metrics.in_progress, 1);
    assert_eq!(metrics.not_started, 1);
  }

  #[test]
  fn test_progress_dashboard() {
    let metrics = ProgressMetrics::new(10, 7, 2, 0, 1, 0).unwrap();
    let dashboard = generate_dashboard("Project Progress".to_string(), metrics, vec![]);

    assert_eq!(dashboard.title, "Project Progress");
    assert_eq!(dashboard.metrics.total, 10);
    assert_eq!(dashboard.metrics.completed, 7);
    assert_eq!(dashboard.category_breakdown.len(), 0);
  }

  #[test]
  fn test_progress_dashboard_with_categories() {
    let metrics = ProgressMetrics::new(10, 7, 2, 0, 1, 0).unwrap();
    let categories = vec![
      CategoryProgress {
        category: "Core".to_string(),
        total: 5,
        metrics: ProgressMetrics::new(5, 5, 0, 0, 0, 0).unwrap(),
      },
      CategoryProgress {
        category: "Web".to_string(),
        total: 5,
        metrics: ProgressMetrics::new(5, 2, 2, 0, 1, 0).unwrap(),
      },
    ];

    let dashboard = generate_dashboard("Project Progress".to_string(), metrics, categories);

    assert_eq!(dashboard.category_breakdown.len(), 2);
    assert_eq!(dashboard.category_breakdown[0].category, "Core");
    assert_eq!(dashboard.category_breakdown[1].category, "Web");
  }

  #[test]
  fn test_progress_error_display() {
    let error = ProgressError::InvalidCount { total: 10, sum: 15 };
    assert_eq!(
      format!("{}", error),
      "invalid count values: total=10, sum=15"
    );
  }

  #[test]
  fn test_progress_format_display() {
    assert_eq!(format!("{}", ProgressFormat::Terminal), "terminal");
    assert_eq!(format!("{}", ProgressFormat::Json), "json");
    assert_eq!(format!("{}", ProgressFormat::Markdown), "markdown");
  }

  #[test]
  fn test_progress_output_options_default() {
    let options = ProgressOutputOptions::default();
    assert_eq!(options.format, ProgressFormat::Terminal);
    assert!(options.show_categories);
    assert!(options.show_details);
  }

  #[test]
  fn test_format_progress_terminal() {
    let metrics = ProgressMetrics::new(10, 7, 2, 0, 1, 0).unwrap();
    let output = format_progress(&metrics, &ProgressOutputOptions::default()).unwrap();
    assert!(output.contains("\"total\":10"));
  }

  #[test]
  fn test_format_progress_markdown() {
    let metrics = ProgressMetrics::new(10, 7, 2, 0, 1, 0).unwrap();
    let options = ProgressOutputOptions {
      format: ProgressFormat::Markdown,
      ..Default::default()
    };
    let output = format_progress(&metrics, &options).unwrap();
    assert!(output.contains("# Progress Dashboard"));
  }

  #[test]
  fn test_progress_distribution() {
    let metrics = ProgressMetrics::new(10, 7, 2, 0, 1, 0).unwrap();
    assert_eq!(metrics.status_distribution.completed_pct, 70.0);
    assert_eq!(metrics.status_distribution.in_progress_pct, 20.0);
    assert_eq!(metrics.status_distribution.blocked_pct, 0.0);
    assert_eq!(metrics.status_distribution.deferred_pct, 10.0);
    assert_eq!(metrics.status_distribution.not_started_pct, 0.0);
  }

  #[test]
  fn test_progress_distribution_zero_total() {
    let metrics = ProgressMetrics::empty();
    assert_eq!(metrics.status_distribution.completed_pct, 0.0);
    assert_eq!(metrics.status_distribution.in_progress_pct, 0.0);
    assert_eq!(metrics.status_distribution.blocked_pct, 0.0);
    assert_eq!(metrics.status_distribution.deferred_pct, 0.0);
    assert_eq!(metrics.status_distribution.not_started_pct, 0.0);
  }

  #[allow(clippy::unwrap_used)]
  #[test]
  fn test_category_progress() {
    let category = CategoryProgress {
      category: "Backend".to_string(),
      total: 5,
      metrics: ProgressMetrics::new(5, 3, 1, 0, 1, 0).unwrap(),
    };

    assert_eq!(category.category, "Backend");
    assert_eq!(category.total, 5);
    assert_eq!(category.metrics.completed, 3);
  }

  #[allow(clippy::unwrap_used)]
  #[test]
  fn test_dashboard_display() {
    let metrics = ProgressMetrics::new(10, 7, 2, 0, 1, 0).unwrap();
    let dashboard = generate_dashboard("Test Dashboard".to_string(), metrics, vec![]);
    let display = format!("{dashboard}");

    assert!(display.contains("=== Test Dashboard ==="));
    assert!(display.contains("Progress: 7/10"));
    assert!(!display.contains("Category Breakdown"));
  }

  #[test]
  fn test_all_statuses_covered() {
    let statuses = ProgressStatus::all();
    assert!(statuses.contains(&ProgressStatus::NotStarted));
    assert!(statuses.contains(&ProgressStatus::InProgress));
    assert!(statuses.contains(&ProgressStatus::Completed));
    assert!(statuses.contains(&ProgressStatus::Blocked));
    assert!(statuses.contains(&ProgressStatus::Deferred));
  }
}
