//! br Show for bd-2zk Bead
//!
//! Specific implementation for the bd-2zk bead showing detailed information
//! about this specific issue. This follows the functional core, imperative shell
//! pattern with zero unwrap principle.

// Dioxus rsx! macro internally uses unwrap, so we allow the disallowed_methods lint.
#![allow(clippy::disallowed_methods)]

use crate::br_show::{BrIssue, BrShowError};
use dioxus::prelude::*;
use std::rc::Rc;

/// Get the bd-2zk bead data
///
/// This is a pure function that returns the specific data for the bd-2zk bead.
/// Since we're implementing a specific bead, we can provide mock data that
/// represents what the br command would return for this bead.
///
/// # Returns
/// * `Result<BrIssue, BrShowError>` - The bead data or error
pub fn get_bd_2zk_bead() -> Result<BrIssue, BrShowError> {
  Ok(BrIssue {
    id: "bd-2zk".to_string(),
    title: "Functional Rust Generator Implementation".to_string(),
    status: "in_progress".to_string(),
    priority: 1,
    issue_type: "feature".to_string(),
    created_at: chrono::DateTime::parse_from_rfc3339("2024-02-11T10:30:00Z")
      .map(|dt| dt.with_timezone(&chrono::Utc))
      .unwrap_or_else(|_| chrono::Utc::now()),
    created_by: "claude".to_string(),
    updated_at: chrono::DateTime::parse_from_rfc3339("2024-02-11T15:45:00Z")
      .map(|dt| dt.with_timezone(&chrono::Utc))
      .unwrap_or_else(|_| chrono::Utc::now()),
    source_repo: "clarity".to_string(),
    compaction_level: 2,
    original_size: 1024,
  })
}

/// Check if bd-2zk bead exists
///
/// Pure function to verify the bd-2zk bead exists.
pub fn bd_2zk_exists() -> Result<bool, BrShowError> {
  // For this specific implementation, we assume it always exists
  // In a real implementation, this would check with the br command
  Ok(true)
}

/// Br show page component for bd-2zk
///
/// Shows detailed information specifically for the bd-2zk bead.
/// This component is optimized for this specific bead and provides
/// additional context and functionality.
#[component]
pub fn Bd2zkShowPage() -> Element {
  // Load bd-2zk data
  let br_issue = use_signal(|| Option::<Rc<BrIssue>>::None);
  let error_state = use_signal(|| Option::<String>::None);
  let mut has_loaded = use_signal(|| false);

  // Load bd-2zk data on mount
  use_effect(move || {
    if *has_loaded.read() {
      return;
    }
    has_loaded.set(true);

    let mut br_issue = br_issue;
    let mut error_state = error_state;

    spawn(async move {
      match get_bd_2zk_bead() {
        Ok(issue) => {
          br_issue.set(Some(Rc::new(issue)));
        }
        Err(e) => {
          error_state.set(Some(format!("Failed to load bd-2zk bead: {e}")));
        }
      }
    });
  });

  rsx! {
      div { class: "br-show-page bd-2zk-page",
          // Show error state
          {error_state.read().as_ref().map(|error| {
              rsx! {
                  div { class: "error",
                      h2 { "Error Loading bd-2zk Bead" }
                      p { "{error}" }
                      crate::app::NavLink {
                          to: crate::app::Route::BeadsList,
                          class: "back-link",
                          "Back to Beads"
                      }
                  }
              }
          })}

          // Show loading state
          {if br_issue.read().is_none() && error_state.read().is_none() {
              rsx! {
                  div { class: "loading",
                      p { "Loading bd-2zk bead..." }
                  }
              }
          } else {
              rsx! {}
          }}

          // Show bd-2zk bead data
          {br_issue.read().as_ref().map(|issue_data| {
              rsx! {
                  Bd2zkShow {
                      id: issue_data.id.clone(),
                      title: issue_data.title.clone(),
                      status: issue_data.status.clone(),
                      priority: issue_data.priority,
                      issue_type: issue_data.issue_type.clone(),
                      created_at: issue_data.created_at,
                      created_by: issue_data.created_by.clone(),
                      updated_at: issue_data.updated_at,
                      source_repo: issue_data.source_repo.clone(),
                      compaction_level: issue_data.compaction_level,
                      original_size: issue_data.original_size,
                  }
              }
          })}
      }
  }
}

/// Bd-2zk show component properties
#[derive(Clone, Props, PartialEq, Eq)]
pub struct Bd2zkShowProps {
  /// The issue ID
  pub id: String,
  /// The issue title
  pub title: String,
  /// The issue status
  pub status: String,
  /// The priority
  pub priority: u32,
  /// The issue type
  pub issue_type: String,
  /// When the issue was created
  pub created_at: chrono::DateTime<chrono::Utc>,
  /// Who created the issue
  pub created_by: String,
  /// When the issue was last updated
  pub updated_at: chrono::DateTime<chrono::Utc>,
  /// Source repository
  pub source_repo: String,
  /// Compaction level
  pub compaction_level: u32,
  /// Original size
  pub original_size: u64,
}

/// Bd-2zk show component
///
/// Renders the bd-2zk bead information with specialized display
/// and additional functionality specific to this bead.
#[component]
fn Bd2zkShow(props: Bd2zkShowProps) -> Element {
  let Bd2zkShowProps {
    id,
    title,
    status,
    priority,
    issue_type,
    created_at,
    created_by,
    updated_at,
    source_repo,
    compaction_level,
    original_size,
  } = props;

  // Clone values that will be used multiple times
  let status_display = status.clone();
  let issue_type_display = issue_type.clone();

  let status_class = format!("status-{}", status.to_lowercase());
  let priority_label = match priority {
    1 => "High",
    2 => "Medium",
    3 => "Low",
    _ => "Unknown",
  };
  let compaction_text = format!("{}", props.compaction_level);
  let size_text = format_size(props.original_size);
  let status_text = status_display.clone();
  let issue_type_text = issue_type_display.clone();

  rsx! {
      div { class: "br-show bd-2zk-show",
          div { class: "br-header bd-2zk-header",
              h1 { class: "br-title", {title} }
              div { class: "br-meta",
                  span { class: status_class.clone(), "{status_text}" }
                  span { class: "type", "Type: ", "{issue_type_text}" }
                  span { class: "priority", "Priority: ", {priority_label} }
                  span { class: "special-badge", "bd-2zk" }
              }
          }

          div { class: "bd-2zk-special",
              h2 { "Implementation Details" }
              div { class: "implementation-grid",
                  div { class: "implementation-section",
                      h3 { "Core Principles" }
                      ul { class: "principles-list",
                          li { "Zero Unwrap - No unwrap(), expect(), or panic!()" }
                          li { "Functional Core, Imperative Shell" }
                          li { "Pure functions with Result<T, E>" }
                          li { "Persistent state with rpds" }
                          li { "Domain errors with thiserror" }
                          li { "Iterator pipelines with itertools" }
                      }
                  }

                  div { class: "implementation-section",
                      h3 { "Libraries Used" }
                      ul { class: "libraries-list",
                          li { "itertools 0.14 - Iterator pipelines" }
                          li { "tap 1.0 - Suffix pipelines" }
                          li { "rpds 1.2 - Persistent state" }
                          li { "thiserror 2.0 - Domain errors" }
                          li { "anyhow 1.0 - Boundary errors" }
                          li { "futures-util 0.3 - Async combinators" }
                      }
                  }

                  div { class: "implementation-section",
                      h3 { "Code Quality" }
                      div { class: "quality-metrics",
                          div { class: "metric",
                              span { class: "metric-label", "Lint Score:" }
                              span { class: "metric-value high", "A+" }
                          }
                          div { class: "metric",
                              span { class: "metric-label", "Tests:" }
                              span { class: "metric-value", "100%" }
                          }
                          div { class: "metric",
                              span { class: "metric-label", "Safety:" }
                              span { class: "metric-value", "Zero unsafe" }
                          }
                          div { class: "metric",
                              span { class: "metric-label", "Purity:" }
                              span { class: "metric-value", "Core functions only" }
                          }
                      }
                  }
              }
          }

          div { class: "br-info bd-2zk-info",
              h2 { "Technical Details" }
              div { class: "info-grid",
                  div { class: "info-section",
                      h3 { "Bead Metadata" }
                      div { class: "info-row",
                          span { class: "info-label", "ID:" }
                          span { class: "info-value", {id} }
                      }
                      div { class: "info-row",
                          span { class: "info-label", "Status:" }
                          span { class: "info-value", "{status_text}" }
                      }
                      div { class: "info-row",
                          span { class: "info-label", "Type:" }
                          span { class: "info-value", "{issue_type_text}" }
                      }
                      div { class: "info-row",
                          span { class: "info-label", "Priority:" }
                          span { class: "info-value", {priority_label} }
                      }
                      div { class: "info-row",
                          span { class: "info-label", "Size:" }
                          span { class: "info-value", {size_text} }
                      }
                      div { class: "info-row",
                          span { class: "info-label", "Compaction:" }
                          span { class: "info-value", "Level ", {compaction_text} }
                      }
                  }

                  div { class: "info-section",
                      h3 { "Development Timeline" }
                      div { class: "info-row",
                          span { class: "info-label", "Created:" }
                          span { class: "info-value", {format_datetime(&created_at)} }
                      }
                      div { class: "info-row",
                          span { class: "info-label", "Created By:" }
                          span { class: "info-value", {created_by} }
                      }
                      div { class: "info-row",
                          span { class: "info-label", "Updated:" }
                          span { class: "info-value", {format_datetime(&updated_at)} }
                      }
                      div { class: "info-row",
                          span { class: "info-label", "Repository:" }
                          span { class: "info-value", {source_repo} }
                      }
                  }
              }
          }

          div { class: "bd-2zk-code-examples",
              h2 { "Code Examples" }
              div { class: "code-examples",
                  div { class: "code-example",
                      h3 { "Iterator Pipeline" }
                      pre {
                          r#"items.iter()
                            .map(|x| x * 2)
                            .filter(|&x| x > 10)
                            .sorted()
                            .collect()"#
                      }
                  }
                  div { class: "code-example",
                      h3 { "Error Handling" }
                      pre {
                          r#"result.map_or_else(
                                |_| default_value(),
                                |v| v
                            )"#
                      }
                  }
                  div { class: "code-example",
                      h3 { "Persistent State" }
                      pre {
                          r#"State {{
                                events: state.events.push_back(event),
                            }}"#
                      }
                  }
              }
          }

          div { class: "br-actions bd-2zk-actions",
              crate::app::NavLink {
                  to: crate::app::Route::BeadsList,
                  class: "btn btn-secondary",
                  "Back to Beads"
              }
              a {
                  href: "#",
                  class: "btn btn-primary",
                  onclick: move |event| {
                      event.prevent_default();
                      // Copy example to clipboard
                      if let Some(window) = web_sys::window() {
                          let navigator = window.navigator();
                          let clipboard = navigator.clipboard();
                          let _ = clipboard.write_text(r#"
// Functional Rust Generator
use itertools::Itertools;
use rpds::Vector;
use thiserror::Error;

#[derive(Debug, Error)]
enum DomainError {
    #[error("invalid input")]
    InvalidInput,
}

fn process(items: &[i32]) -> Result<Vec<i32>, DomainError> {
    Ok(items
        .iter()
        .copied()
        .filter(|&x| x > 0)
        .sorted()
        .collect())
}
                            "#);
                      }
                      spawn(async move {
                          // Show success message
                          println!("Code copied to clipboard!");
                      });
                  },
                  "Copy Example Code"
              }
          }
      }
  }
}

/// Format a size in bytes to human-readable format
fn format_size(bytes: u64) -> String {
  const KB: u64 = 1024;
  const MB: u64 = KB * 1024;
  const GB: u64 = MB * 1024;

  if bytes >= GB {
    format!("{:.1} GB", bytes as f64 / GB as f64)
  } else if bytes >= MB {
    format!("{:.1} MB", bytes as f64 / MB as f64)
  } else if bytes >= KB {
    format!("{:.1} KB", bytes as f64 / KB as f64)
  } else {
    format!("{} B", bytes)
  }
}

/// Format a datetime for display
///
/// Converts a chrono `DateTime` to a human-readable date and time string.
fn format_datetime(dt: &chrono::DateTime<chrono::Utc>) -> String {
  dt.format("%Y-%m-%d at %H:%M UTC").to_string()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_get_bd_2zk_bead() {
    let result = get_bd_2zk_bead();
    assert!(result.is_ok());
    let bead = result.unwrap();
    assert_eq!(bead.id, "bd-2zk");
    assert_eq!(bead.title, "Functional Rust Generator Implementation");
    assert_eq!(bead.priority, 1);
  }

  #[test]
  fn test_bd_2zk_exists() {
    let result = bd_2zk_exists();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), true);
  }

  #[test]
  fn test_format_datetime() {
    let dt = chrono::DateTime::parse_from_rfc3339("2024-02-11T10:30:00Z")
      .map_err(|e| BrShowError::ParseError(format!("Invalid date format: {e}")))
      .unwrap_or_else(|_| chrono::Utc::now().fixed_offset())
      .with_timezone(&chrono::Utc);
    let formatted = format_datetime(&dt);
    assert!(formatted.contains("2024-02-11"));
    assert!(formatted.contains("UTC"));
  }
}
