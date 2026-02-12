//! br Show Page Component
//!
//! Displays detailed information about a bead/issue from the `br` command.

// Dioxus rsx! macro internally uses unwrap, so we allow the disallowed_methods lint.
#![allow(clippy::disallowed_methods)]

use crate::br_show::{fetch_br_issue, BrIssue};
use dioxus::prelude::*;
use std::rc::Rc;

/// Br show page component
///
/// Shows details from the `br show` command for a specific bead ID.
/// Loads data from the br command using async execution.
#[component]
pub fn BrShowPage(id: String) -> Element {
  // Load br data using async loading
  let br_issue = use_signal(|| Option::<Rc<BrIssue>>::None);
  let error_state = use_signal(|| Option::<String>::None);
  let mut has_loaded = use_signal(|| false);

  // Load br data on mount
  use_effect(move || {
    if *has_loaded.read() {
      return;
    }
    has_loaded.set(true);

    let id = id.clone();
    let mut br_issue = br_issue;
    let mut error_state = error_state;

    spawn(async move {
      match fetch_br_issue(&id).await {
        Ok(issue) => {
          br_issue.set(Some(Rc::new(issue)));
        }
        Err(e) => {
          error_state.set(Some(format!("Failed to load br issue: {e}")));
        }
      }
    });
  });

  rsx! {
      div { class: "br-show-page",
          // Show error state
          {error_state.read().as_ref().map(|error| {
              rsx! {
                  div { class: "error",
                      h2 { "Error Loading Issue" }
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
                      p { "Loading issue..." }
                  }
              }
          } else {
              rsx! {}
          }}

          // Show br issue data
          {br_issue.read().as_ref().map(|issue_data| {
              rsx! {
                  BrShow {
                      id: issue_data.id.clone(),
                      title: issue_data.title.clone(),
                      status: issue_data.status.clone(),
                      priority: issue_data.priority,
                      issue_type: issue_data.issue_type.clone(),
                      created_at: issue_data.created_at,
                      created_by: issue_data.created_by.clone(),
                      updated_at: issue_data.updated_at,
                      source_repo: issue_data.source_repo.clone(),
                  }
              }
          })}
      }
  }
}

/// Br show component properties
#[derive(Clone, Props, PartialEq, Eq)]
pub struct BrShowProps {
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
}

/// Br show component
///
/// Renders the br issue information with formatted display.
#[component]
fn BrShow(props: BrShowProps) -> Element {
  let BrShowProps {
    id,
    title,
    status,
    priority,
    issue_type,
    created_at,
    created_by,
    updated_at,
    source_repo,
  } = props;

  let status_class = format!("status-{}", status.to_lowercase());
  let priority_label = match priority {
    1 => "High",
    2 => "Medium",
    3 => "Low",
    _ => "Unknown",
  };

  rsx! {
      div { class: "br-show",
          div { class: "br-header",
              h1 { class: "br-title", "{title}" }
              div { class: "br-meta",
                  span { class: "{status_class}", "{status}" }
                  span { class: "type", "Type: {issue_type}" }
                  span { class: "priority", "Priority: {priority_label}" }
              }
          }

          div { class: "br-info",
              h2 { "Issue Details" }
              div { class: "info-grid",
                  div { class: "info-section",
                      h3 { "Metadata" }
                      div { class: "info-row",
                          span { class: "info-label", "ID:" }
                          span { class: "info-value", "{id}" }
                      }
                      div { class: "info-row",
                          span { class: "info-label", "Status:" }
                          span { class: "info-value", "{status}" }
                      }
                      div { class: "info-row",
                          span { class: "info-label", "Type:" }
                          span { class: "info-value", "{issue_type}" }
                      }
                      div { class: "info-row",
                          span { class: "info-label", "Priority:" }
                          span { class: "info-value", "{priority_label}" }
                      }
                  }

                  div { class: "info-section",
                      h3 { "Dates" }
                      div { class: "info-row",
                          span { class: "info-label", "Created:" }
                          span { class: "info-value", "{format_datetime(&created_at)}" }
                      }
                      div { class: "info-row",
                          span { class: "info-label", "Created By:" }
                          span { class: "info-value", "{created_by}" }
                      }
                      div { class: "info-row",
                          span { class: "info-label", "Updated:" }
                          span { class: "info-value", "{format_datetime(&updated_at)}" }
                      }
                      div { class: "info-row",
                          span { class: "info-label", "Repository:" }
                          span { class: "info-value", "{source_repo}" }
                      }
                  }
              }
          }

          div { class: "br-actions",
              crate::app::NavLink {
                  to: crate::app::Route::BeadsList,
                  class: "btn btn-secondary",
                  "Back to Beads"
              }
          }
      }
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
  fn test_format_datetime() {
    let dt = chrono::DateTime::parse_from_rfc3339("2024-02-09T12:34:56Z")
      .unwrap()
      .with_timezone(&chrono::Utc);
    assert_eq!(format_datetime(&dt), "2024-02-09 at 12:34 UTC");
  }
}
