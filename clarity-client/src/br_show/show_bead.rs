//! br Show Page Component
//!
//! Displays detailed information about a bead/issue from the `br` command.

// Dioxus rsx! macro internally uses unwrap, so we allow the disallowed_methods lint.
#![allow(clippy::disallowed_methods)]

use crate::beads::list::LoadingState;
use crate::br_show::{fetch_br_issue, BrIssue};
use dioxus::prelude::*;
use std::rc::Rc;

// ===== Loading State for br_show =====

/// Type alias for br show loading state
type BrShowLoadingState = LoadingState<Rc<BrIssue>>;

/// Br show page component
///
/// Shows details from the `br show` command for a specific bead ID.
/// Loads data from the br command using async execution.
/// Uses explicit LoadingState enum per Scott Wlaschin DDD principles.
#[component]
pub fn BrShowPage(id: String) -> Element {
  // Explicit loading state (replaces Option-as-state anti-pattern)
  let loading_state = use_signal(|| LoadingState::idle());

  // Load br data on mount
  use_effect({
    let mut loading_state = loading_state;
    move || {
      // Only load once
      if !loading_state.read().is_idle() {
        return;
      }

      // Set loading state
      loading_state.set(LoadingState::loading());

      let id = id.clone();
      let mut loading_state = loading_state.clone();

      spawn(async move {
        match fetch_br_issue(&id).await {
          Ok(issue) => {
            loading_state.set(LoadingState::loaded(Rc::new(issue)));
          }
          Err(e) => {
            loading_state.set(LoadingState::failed(format!("Failed to load br issue: {e}")));
          }
        }
      });
    }
  });

  rsx! {
      div { class: "br-show-page",
          // Show error state using explicit LoadingState enum
          {if loading_state.read().is_failed() {
              let error_msg = loading_state.read()
                  .error()
                  .cloned()
                  .unwrap_or_else(|| "Unknown error".to_string());
              rsx! {
                  div { class: "error",
                      h2 { "Error Loading Issue" }
                      p { "{error_msg}" }
                      crate::app::NavLink {
                          to: crate::app::Route::BeadsList,
                          class: "back-link",
                          "Back to Beads"
                      }
                  }
              }
          } else {
              rsx! {}
          }}

          // Show loading state using explicit LoadingState enum
          {if loading_state.read().is_loading() {
              rsx! {
                  div { class: "loading",
                      p { "Loading issue..." }
                  }
              }
          } else {
              rsx! {}
          }}

          // Show br issue data using explicit LoadingState enum
          {if loading_state.read().is_loaded() {
              let issue_data = loading_state.read().data().unwrap().clone();
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
          } else {
              rsx! {}
          }}
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
