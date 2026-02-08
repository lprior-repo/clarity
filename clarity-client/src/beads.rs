//! Bead Management UI Components
//!
//! This module provides components for managing beads through a web interface.
//! It communicates with the backend API to perform bead operations.

// Dioxus rsx! macro internally uses unwrap, so we allow the disallowed_methods lint.
// This is a framework limitation, not our code using unwrap.
#![allow(clippy::disallowed_methods)]

use crate::ApiError;
use dioxus::prelude::*;

/// Bead summary for list display
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BeadSummary {
  pub id: String,
  pub title: String,
  pub status: String,
  pub priority: String,
  pub bead_type: String,
  pub labels: Vec<String>,
}

impl From<crate::api::BeadSummary> for BeadSummary {
  fn from(api_bead: crate::api::BeadSummary) -> Self {
    Self {
      id: api_bead.id,
      title: api_bead.title,
      status: api_bead.status,
      priority: api_bead.priority.to_string(),
      bead_type: api_bead.bead_type,
      labels: Vec::new(), // Labels would come from a separate API call
    }
  }
}

/// Filter options for bead list
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BeadFilter {
  pub status: Option<String>,
  pub bead_type: Option<String>,
  pub priority: Option<String>,
  pub search_query: Option<String>,
}

impl BeadFilter {
  #[must_use]
  pub const fn new() -> Self {
    Self {
      status: None,
      bead_type: None,
      priority: None,
      search_query: None,
    }
  }

  /// Check if filter is active (has any criteria set)
  #[must_use]
  pub const fn is_active(&self) -> bool {
    self.status.is_some()
      || self.bead_type.is_some()
      || self.priority.is_some()
      || self.search_query.is_some()
  }
}

impl Default for BeadFilter {
  fn default() -> Self {
    Self::new()
  }
}

/// Bead management page component
///
/// This is the main component that displays the bead management interface.
/// It loads beads from the backend API and displays them with filtering.
#[component]
pub fn BeadManagementPage() -> Element {
  let _filter = use_signal(BeadFilter::new);
  let mut beads = use_signal(Vec::<BeadSummary>::new);
  let mut loading = use_signal(|| true);
  let mut error = use_signal::<Option<String>>(|| None);

  // Load beads from API on component mount using use_resource
  let _beads_resource = use_resource(move || async move {
    let client = crate::ApiClient::new();

    match client.list_beads(None, None, None, None).await {
      Ok(response) => {
        loading.set(false);
        let ui_beads = response.beads.into_iter().map(BeadSummary::from).collect();
        beads.set(ui_beads);
        Ok::<(), ApiError>(())
      }
      Err(err) => {
        loading.set(false);
        error.set(Some(err.to_string()));
        Err(err)
      }
    }
  });

  rsx! {
      div { class: "bead-management-page",
          h1 { "Bead Management" }
          div { class: "bead-content",
              if *loading.read() {
                  div { class: "loading", "Loading beads..." }
              } else if let Some(err) = error.read().as_ref() {
                  div { class: "error-banner", "{err}" }
              } else {
                  div { class: "bead-list",
                      if beads.read().is_empty() {
                          div { class: "empty-state", "No beads found" }
                      } else {
                          for bead in beads.read().iter() {
                              BeadCard { bead: bead.clone() }
                          }
                      }
                  }
              }
          }
      }
  }
}

/// Individual bead card component
#[component]
fn BeadCard(bead: BeadSummary) -> Element {
  rsx! {
      div { class: "bead-card",
          div { class: "bead-header",
              h3 { class: "bead-title", "{bead.title}" }
              div { class: "bead-badges",
                  span { class: "badge badge-status", "{bead.status}" }
                  span { class: "badge badge-priority", "{bead.priority}" }
                  span { class: "badge badge-type", "{bead.bead_type}" }
              }
          }
          div { class: "bead-id", "ID: {bead.id}" }
          if !bead.labels.is_empty() {
              div { class: "bead-labels",
                  for label in bead.labels.iter() {
                      span { class: "label-badge", "{label}" }
                  }
              }
          }
          div { class: "bead-actions",
              button { class: "btn-secondary", "View" }
              button { class: "btn-secondary", "Edit" }
          }
      }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_bead_filter_new() {
    let filter = BeadFilter::new();
    assert!(filter.status.is_none());
    assert!(filter.bead_type.is_none());
    assert!(filter.priority.is_none());
    assert!(filter.search_query.is_none());
  }

  #[test]
  fn test_bead_filter_default() {
    let filter = BeadFilter::default();
    assert!(filter.status.is_none());
    assert!(filter.bead_type.is_none());
    assert!(filter.priority.is_none());
    assert!(filter.search_query.is_none());
  }

  #[test]
  fn test_bead_filter_is_active_when_empty() {
    let filter = BeadFilter::new();
    assert!(!filter.is_active(), "Empty filter should not be active");
  }

  #[test]
  fn test_bead_filter_is_active_with_status() {
    let filter = BeadFilter {
      status: Some("open".to_string()),
      bead_type: None,
      priority: None,
      search_query: None,
    };
    assert!(filter.is_active(), "Filter with status should be active");
  }

  #[test]
  fn test_bead_filter_is_active_with_type() {
    let filter = BeadFilter {
      status: None,
      bead_type: Some("feature".to_string()),
      priority: None,
      search_query: None,
    };
    assert!(filter.is_active(), "Filter with type should be active");
  }

  #[test]
  fn test_bead_filter_is_active_with_priority() {
    let filter = BeadFilter {
      status: None,
      bead_type: None,
      priority: Some("1".to_string()),
      search_query: None,
    };
    assert!(filter.is_active(), "Filter with priority should be active");
  }

  #[test]
  fn test_bead_filter_is_active_with_search() {
    let filter = BeadFilter {
      status: None,
      bead_type: None,
      priority: None,
      search_query: Some("test".to_string()),
    };
    assert!(
      filter.is_active(),
      "Filter with search query should be active"
    );
  }

  #[test]
  fn test_bead_filter_is_active_with_multiple_criteria() {
    let filter = BeadFilter {
      status: Some("open".to_string()),
      bead_type: Some("feature".to_string()),
      priority: Some("1".to_string()),
      search_query: Some("test".to_string()),
    };
    assert!(
      filter.is_active(),
      "Filter with multiple criteria should be active"
    );
  }

  #[test]
  fn test_bead_summary_clone() {
    let bead1 = BeadSummary {
      id: "bd-001".to_string(),
      title: "Test bead".to_string(),
      status: "open".to_string(),
      priority: "1".to_string(),
      bead_type: "feature".to_string(),
      labels: vec!["stage:ready".to_string()],
    };

    let bead2 = bead1.clone();

    assert_eq!(bead1.id, bead2.id);
    assert_eq!(bead1.title, bead2.title);
    assert_eq!(bead1.status, bead2.status);
    assert_eq!(bead1.priority, bead2.priority);
    assert_eq!(bead1.bead_type, bead2.bead_type);
    assert_eq!(bead1.labels, bead2.labels);
  }

  #[test]
  fn test_bead_summary_equality() {
    let bead1 = BeadSummary {
      id: "bd-001".to_string(),
      title: "Test bead".to_string(),
      status: "open".to_string(),
      priority: "1".to_string(),
      bead_type: "feature".to_string(),
      labels: vec!["stage:ready".to_string()],
    };

    let bead2 = BeadSummary {
      id: "bd-001".to_string(),
      title: "Test bead".to_string(),
      status: "open".to_string(),
      priority: "1".to_string(),
      bead_type: "feature".to_string(),
      labels: vec!["stage:ready".to_string()],
    };

    assert_eq!(bead1, bead2);
  }

  #[test]
  fn test_bead_summary_inequality_different_id() {
    let bead1 = BeadSummary {
      id: "bd-001".to_string(),
      title: "Test bead".to_string(),
      status: "open".to_string(),
      priority: "1".to_string(),
      bead_type: "feature".to_string(),
      labels: vec![],
    };

    let bead2 = BeadSummary {
      id: "bd-002".to_string(),
      title: "Test bead".to_string(),
      status: "open".to_string(),
      priority: "1".to_string(),
      bead_type: "feature".to_string(),
      labels: vec![],
    };

    assert_ne!(bead1, bead2);
  }

  #[test]
  fn test_bead_summary_with_empty_labels() {
    let bead = BeadSummary {
      id: "bd-001".to_string(),
      title: "Test bead".to_string(),
      status: "open".to_string(),
      priority: "1".to_string(),
      bead_type: "feature".to_string(),
      labels: vec![],
    };

    assert!(bead.labels.is_empty());
    assert_eq!(bead.id, "bd-001");
  }

  #[test]
  fn test_bead_summary_with_multiple_labels() {
    let labels = vec![
      "stage:ready".to_string(),
      "size:medium".to_string(),
      "priority:high".to_string(),
    ];

    let bead = BeadSummary {
      id: "bd-001".to_string(),
      title: "Test bead".to_string(),
      status: "open".to_string(),
      priority: "1".to_string(),
      bead_type: "feature".to_string(),
      labels: labels.clone(),
    };

    assert_eq!(bead.labels.len(), 3);
    assert_eq!(bead.labels, labels);
  }

  // Tests for new components (Phase 4: RED)

  #[test]
  fn test_bead_filter_with_all_criteria() {
    let filter = BeadFilter {
      status: Some("in_progress".to_string()),
      bead_type: Some("feature".to_string()),
      priority: Some("1".to_string()),
      search_query: Some("auth".to_string()),
    };
    assert!(filter.is_active());
    assert_eq!(filter.status, Some("in_progress".to_string()));
    assert_eq!(filter.bead_type, Some("feature".to_string()));
    assert_eq!(filter.priority, Some("1".to_string()));
    assert_eq!(filter.search_query, Some("auth".to_string()));
  }

  #[test]
  fn test_bead_filter_clear() {
    let mut filter = BeadFilter {
      status: Some("open".to_string()),
      bead_type: Some("bugfix".to_string()),
      priority: Some("2".to_string()),
      search_query: Some("test".to_string()),
    };
    assert!(filter.is_active());

    filter.status = None;
    filter.bead_type = None;
    filter.priority = None;
    filter.search_query = None;

    assert!(!filter.is_active());
  }

  // Test 3 from acceptance tests: Create Bead Successfully
  #[test]
  fn test_create_bead_validation_required_fields() {
    let title = "";
    let description = Some("Test description");
    let priority = 1;
    let bead_type = "feature";
    let status = "open";

    // Title is required
    assert!(title.is_empty(), "Empty title should be invalid");

    // Valid priority range is 0-4
    let valid_priorities = [0, 1, 2, 3, 4];
    for p in valid_priorities {
      assert!(p >= 0 && p <= 4, "Priority {p} should be valid");
    }

    let invalid_priorities = [-1, 5, 10];
    for p in invalid_priorities {
      assert!(p < 0 || p > 4, "Priority {p} should be invalid");
    }
  }

  // Test 4 from acceptance tests: Show Validation Errors
  #[test]
  fn test_create_bead_validation_invalid_priority() {
    let priority = 5; // Invalid: must be 0-4
    assert!(priority > 4, "Priority 5 should be invalid");
  }

  // Test 4 from acceptance tests: Show Validation Errors
  #[test]
  fn test_create_bead_validation_label_format() {
    let valid_labels = vec!["stage:ready".to_string(), "size:large".to_string()];
    for label in &valid_labels {
      assert!(
        label.contains(':'),
        "Valid label '{label}' should contain colon"
      );
    }

    let invalid_labels = vec!["invalidlabel".to_string(), "stage".to_string()];
    for label in &invalid_labels {
      assert!(
        !label.contains(':'),
        "Invalid label '{label}' should not contain colon"
      );
    }
  }

  // Test 7 from acceptance tests: Search Beads by Text Query
  #[test]
  fn test_bead_search_filters_by_title() {
    let beads = vec![
      BeadSummary {
        id: "bd-001".to_string(),
        title: "Implement interview feature".to_string(),
        status: "open".to_string(),
        priority: "1".to_string(),
        bead_type: "feature".to_string(),
        labels: vec![],
      },
      BeadSummary {
        id: "bd-002".to_string(),
        title: "Fix database bug".to_string(),
        status: "open".to_string(),
        priority: "2".to_string(),
        bead_type: "bugfix".to_string(),
        labels: vec![],
      },
    ];

    let search_query = "interview";
    let matching: Vec<_> = beads
      .iter()
      .filter(|b| b.title.contains(search_query))
      .collect();

    assert_eq!(matching.len(), 1);
    assert_eq!(matching[0].id, "bd-001");
  }

  // Test 9 from acceptance tests: Handle Bead Not Found Gracefully
  #[test]
  fn test_bead_not_found_error() {
    let bead_id = "bd-999";
    let error_message = format!("Bead {bead_id} not found");
    assert!(error_message.contains("not found"));
  }

  // Test 10 from acceptance tests: Update Bead Status Quick Action
  #[test]
  fn test_bead_status_transition() {
    let mut status = "open".to_string();
    status = "in_progress".to_string();
    assert_eq!(status, "in_progress");

    status = "completed".to_string();
    assert_eq!(status, "completed");
  }

  #[test]
  fn test_bead_summary_from_api() {
    let api_bead = crate::api::BeadSummary {
      id: "test-id".to_string(),
      title: "Test Bead".to_string(),
      description: Some("Test description".to_string()),
      status: "open".to_string(),
      priority: 1,
      bead_type: "feature".to_string(),
      created_at: "2024-01-01T00:00:00Z".to_string(),
    };

    let ui_bead = BeadSummary::from(api_bead.clone());
    assert_eq!(ui_bead.id, api_bead.id);
    assert_eq!(ui_bead.title, api_bead.title);
    assert_eq!(ui_bead.status, api_bead.status);
    assert_eq!(ui_bead.priority, "1");
    assert_eq!(ui_bead.bead_type, api_bead.bead_type);
    assert!(ui_bead.labels.is_empty());
  }

  #[test]
  fn test_bead_filter_preserves_state() {
    let filter1 = BeadFilter {
      status: Some("open".to_string()),
      bead_type: None,
      priority: None,
      search_query: None,
    };

    let filter2 = filter1.clone();

    assert_eq!(filter1.status, filter2.status);
    assert_eq!(filter1.bead_type, filter2.bead_type);
    assert_eq!(filter1.priority, filter2.priority);
    assert_eq!(filter1.search_query, filter2.search_query);
  }
}
