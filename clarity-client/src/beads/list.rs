//! Bead List Component
//!
//! Displays a list of beads with filtering capabilities using global state.
//! Supports keyboard shortcuts for quick navigation and actions.

// Dioxus rsx! macro internally uses unwrap, so we allow the disallowed_methods lint.
#![allow(clippy::disallowed_methods)]

use crate::error::AppError;
use crate::hooks::{use_keyboard_with_handler, use_loading_manager, use_loading_operations};
use crate::shortcuts::Action;
use clarity_core::db::models::BeadFilters;
use dioxus::prelude::*;
use std::rc::Rc;

/// Bead list page component
///
/// This component uses global state to display and filter beads.
/// It supports filtering by status, type, priority, and search text.
/// Keyboard shortcuts:
/// - Ctrl+N: Create new bead
/// - Ctrl+F: Focus search
/// - Ctrl+?: Show help
#[component]
pub fn BeadListPage() -> Element {
  // Access global state - use AppState directly to get reactive access
  let app_state = use_context::<crate::state::AppState>();
  let bead_actions = crate::hooks::use_bead_actions();
  let loading_manager = use_loading_manager();
  let loading_ops = use_loading_operations();

  // Reactively read beads from the signal - this will trigger re-renders when beads change
  let beads = use_memo(move || app_state.bead_list());

  // Local filter signals (these are UI state, not app state)
  let mut status_filter = use_signal(String::new);
  let mut type_filter = use_signal(String::new);
  let mut priority_filter = use_signal(String::new);
  let mut search_query = use_signal(String::new);
  let mut error_state = use_signal(|| Option::<AppError>::None);

  // Track if initial load has happened
  let has_loaded_initial = use_signal(|| false);

  // Create filters signal that depends on all filter signals
  let filters = use_memo(move || {
    let status = status_filter.read().clone();
    let bead_type = type_filter.read().clone();
    let priority = priority_filter.read().clone();
    let search = search_query.read().clone();

    BeadFilters {
      status: if status.is_empty() {
        None
      } else {
        Some(status)
      },
      bead_type: if bead_type.is_empty() {
        None
      } else {
        Some(bead_type)
      },
      priority: priority.parse::<i16>().ok(),
      created_by: None,
      search: if search.is_empty() {
        None
      } else {
        Some(search)
      },
    }
  });

  // Load filtered beads when filters change or on initial mount
  use_effect({
    let bead_actions = bead_actions.clone();
    let loading_ops = loading_ops;
    let mut has_loaded = has_loaded_initial.clone();
    move || {
      let filters = filters.read().clone();
      let bead_actions = bead_actions.clone();
      let loading_ops = loading_ops.clone();

      // Check if we should load beads
      let should_load = !*has_loaded.read()
        || filters.status.is_some()
        || filters.bead_type.is_some()
        || filters.priority.is_some()
        || filters.search.is_some();

      eprintln!(
        "[BeadList] Effect triggered - should_load: {}, has_loaded: {}, filters: {:?}",
        should_load,
        *has_loaded.read(),
        filters
      );

      if should_load {
        // Mark as loaded
        has_loaded.set(true);

        // Set loading state
        (loading_ops.start)(("bead-list".to_string(), "Loading beads...".to_string()));

        eprintln!("[BeadList] Loading beads with filters: {:?}", filters);

        // Spawn async task
        spawn(async move {
          eprintln!("[BeadList] Spawning async task to load from database");
          match crate::db::DesktopDb::new_async().await {
            Ok(db) => {
              eprintln!("[BeadList] Database initialized, querying beads");
              match db.list_beads_filtered(&filters).await {
                Ok(beads) => {
                  eprintln!("[BeadList] Successfully loaded {} beads", beads.len());
                  (bead_actions.set_beads)(beads);
                  (loading_ops.stop)("bead-list".to_string());
                  error_state.set(None);
                }
                Err(e) => {
                  eprintln!("[BeadList] Error loading beads: {:?}", e);
                  (loading_ops.stop)("bead-list".to_string());
                  let app_err = AppError::from(e);
                  error_state.set(Some(app_err));
                }
              }
            }
            Err(e) => {
              eprintln!("[BeadList] Error initializing database: {:?}", e);
              (loading_ops.stop)("bead-list".to_string());
              let app_err = AppError::from(e);
              error_state.set(Some(app_err));
            }
          }
        });
      }
    }
  });

  // Set up keyboard shortcuts handler
  // Use interior mutability (Rc<RefCell>) to work with Fn closure requirement
  let route = use_context::<Signal<crate::app::Route>>();
  let route_cell = std::rc::Rc::new(std::cell::RefCell::new(route));
  let _keyboard_handler = use_keyboard_with_handler(move |action: Action| {
    match action {
      Action::NewBead => {
        // Navigate to new bead form
        if let Ok(mut route_ref) = route_cell.try_borrow_mut() {
          route_ref.set(crate::app::Route::BeadNew);
        }
      }
      Action::ShowHelp => {
        // Show keyboard help dialog
        // This would be handled by a parent component with the help dialog
      }
      _ => {}
    }
  });

  rsx! {
      div { class: "bead-list-page",
          div { class: "page-header",
              h1 { "Beads" }
              div { class: "page-actions",
                  crate::beads::ExportButton {
                      beads: Rc::new(beads.read().iter().map(|b: &Rc<clarity_core::db::models::Bead>| b.as_ref().clone()).collect())
                  }
                  crate::beads::ImportButton {
                      on_import_success: Callback::<usize>::new({
                          let clear_error = bead_actions.clear_error.clone();
                          move |count: usize| {
                              // Reload beads after import
                              // For now, just clear the error state - actual reload will happen on next render
                              // TODO: Implement proper async callback handling
                              eprintln!("[BeadList] Imported {count} beads, reloading...");
                              clear_error();
                          }
                      })
                  }
              }
          }

          // Show loading state
          {if loading_manager.read().is_loading_key("bead-list") {
              rsx! {
                  div { class: "loading",
                      p { "Loading..." }
                  }
              }
          } else {
              rsx! {}
          }}

          // Show error state
          {error_state.read().as_ref().map(|error| {
              let error_message = error.to_string();
              rsx! {
                  div { class: "error",
                      p { "{error_message}" }
                  }
              }
          })}

          div { class: "filters",
              select {
                  value: "{status_filter}",
                  onchange: move |evt: Event<FormData>| {
                      status_filter.set(evt.value());
                  },
                  option { value: "", "All Statuses" },
                  option { value: "open", "Open" },
                  option { value: "in_progress", "In Progress" },
                  option { value: "blocked", "Blocked" },
                  option { value: "deferred", "Deferred" },
                  option { value: "closed", "Closed" },
              }

              select {
                  value: "{type_filter}",
                  onchange: move |evt: Event<FormData>| {
                      type_filter.set(evt.value());
                  },
                  option { value: "", "All Types" },
                  option { value: "feature", "Feature" },
                  option { value: "bugfix", "Bug Fix" },
                  option { value: "refactor", "Refactor" },
                  option { value: "test", "Test" },
                  option { value: "docs", "Documentation" },
              }

              select {
                  value: "{priority_filter}",
                  onchange: move |evt: Event<FormData>| {
                      priority_filter.set(evt.value());
                  },
                  option { value: "", "All Priorities" },
                  option { value: "1", "High" },
                  option { value: "2", "Medium" },
                  option { value: "3", "Low" },
              }

              input {
                  r#type: "text",
                  id: "bead-search",
                  placeholder: "Search...",
                  value: "{search_query}",
                  oninput: move |evt: Event<FormData>| {
                      search_query.set(evt.value());
                  }
              }
              span { class: "shortcut-hint-inline", "Ctrl+F" }
          }

          div { class: "bead-list",
              table { class: "bead-table",
                  thead {
                      tr {
                          th { "Title" }
                          th { "Status" }
                          th { "Type" }
                          th { "Priority" }
                          th { "Created" }
                      }
                  }
                  tbody {
                      for bead in beads.read().iter() {
                          BeadRow {
                              id: bead.id,
                              title: bead.title.clone(),
                              status: bead.status,
                              bead_type: bead.bead_type,
                              priority: bead.priority,
                              created_at: bead.created_at,
                          }
                      }
                  }
              }
          }
      }
  }
}

/// Single bead row component properties
#[derive(Clone, Props, PartialEq, Eq)]
pub struct BeadRowProps {
  /// The bead ID
  pub id: clarity_core::db::models::BeadId,
  /// The bead title
  pub title: String,
  /// The bead status
  pub status: clarity_core::db::models::BeadStatus,
  /// The bead type
  pub bead_type: clarity_core::db::models::BeadType,
  /// The bead priority
  pub priority: clarity_core::db::models::BeadPriority,
  /// When the bead was created
  pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Single bead row component
///
/// Displays a single bead in the table with a link to its detail page.
#[component]
fn BeadRow(props: BeadRowProps) -> Element {
  let id = props.id;
  let title = props.title;
  let status = props.status;
  let bead_type = props.bead_type;
  let priority = props.priority;
  let created_at = props.created_at;
  let status_class = format!("status-{}", status.as_str());
  let type_class = format!("type-{}", bead_type.as_str());
  let priority_label = match priority.0 {
    1 => "High",
    2 => "Medium",
    _ => "Low",
  };

  rsx! {
      tr {
          td {
              crate::app::NavLink {
                  to: crate::app::Route::BeadDetail { id: id.to_string() },
                  class: "bead-link",
                  "{title}"
              }
          }
          td { class: "{status_class}", "{status.as_str()}" }
          td { class: "{type_class}", "{bead_type.as_str()}" }
          td { "{priority_label}" }
          td {
              time {
                  datetime: created_at.to_rfc3339(),
                  "{format_date(&created_at)}"
              }
          }
      }
  }
}

/// Format a datetime for display
///
/// Converts a chrono `DateTime` to a human-readable date string.
fn format_date(dt: &chrono::DateTime<chrono::Utc>) -> String {
  dt.format("%Y-%m-%d").to_string()
}

/// Helper function to initialize database and load beads
///
/// This function attempts to initialize the database and load beads.
/// It returns a Result to allow graceful error handling.
#[allow(dead_code)]
fn try_init_db_and_load(
  filters: Option<BeadFilters>,
) -> Result<Vec<clarity_core::db::models::Bead>, String> {
  // Initialize database
  let db =
    crate::db::DesktopDb::new().map_err(|e| format!("Failed to initialize database: {e}"))?;

  // Load beads with optional filters
  match filters {
    Some(f) => db
      .list_beads_filtered_sync(&f)
      .map_err(|e| format!("Failed to load beads: {e}")),
    None => db
      .list_beads_sync()
      .map_err(|e| format!("Failed to load beads: {e}")),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_format_date() {
    let dt = chrono::DateTime::parse_from_rfc3339("2024-02-09T12:00:00Z")
      .unwrap()
      .with_timezone(&chrono::Utc);
    assert_eq!(format_date(&dt), "2024-02-09");
  }
}
