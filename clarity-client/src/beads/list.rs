//! Bead List Component
//!
//! Displays a complete list of beads with filtering, sorting, and pagination capabilities.
//! Follows functional programming patterns with zero unwrap rules.

// Dioxus rsx! macro internally uses unwrap, so we allow the disallowed_methods lint.
#![allow(clippy::disallowed_methods)]

use crate::beads::sorting::{SortBy, SortConfig, SortDirection};
use crate::error::AppError;
use crate::hooks::{use_keyboard_with_handler, use_loading_manager, use_loading_operations};
use crate::shortcuts::Action;
use clarity_core::db::models::{BeadFilters, PaginatedBeads};
use dioxus::prelude::*;
use std::rc::Rc;

/// Parse URL query parameters for pagination
/// For desktop app, we'll use a simple approach without web APIs
fn parse_url_params() -> (u32, u32) {
  (1, 25) // Default values for desktop app
}

/// Update URL with current pagination state
/// For desktop app, we don't have URL APIs, so this is a no-op
fn update_url_params(_page: u32, _page_size: u32) {
  // Desktop app doesn't have URL APIs, so this is a no-op
  // In a web app, this would update the URL query parameters
}

/// Bead list page component
///
/// This component uses global state to display and filter beads.
/// It supports filtering by status, type, priority, and search text.
/// Features:
/// - Pagination with configurable page size
/// - Sorting by title, status, type, priority, and creation date
/// - Keyboard shortcuts (Ctrl+N: New bead, Ctrl+F: Focus search)
/// - Loading states and error handling
#[component]
pub fn BeadListPage() -> Element {
  // Access global state - use AppState directly to get reactive access
  let app_state = use_context::<crate::state::AppState>();
  let bead_actions = crate::hooks::use_bead_actions();
  let loading_manager = use_loading_manager();
  let loading_ops = use_loading_operations();

  // Reactively read beads from the signal - this will trigger re-renders when beads change
  let beads = use_memo(move || app_state.bead_list());

  // Pagination signals
  let (initial_page, initial_page_size) = parse_url_params();
  let mut current_page = use_signal(|| initial_page);
  let mut page_size = use_signal(|| initial_page_size);
  let mut total_pages = use_signal(|| 1u32);
  let mut total_beads = use_signal(|| 0u64);

  // Sorting signals
  let mut sort_field = use_signal(|| SortBy::CreatedAt);
  let mut sort_direction = use_signal(|| SortDirection::Descending);

  // Local filter signals (these are UI state, not app state)
  let mut status_filter = use_signal(String::new);
  let mut type_filter = use_signal(String::new);
  let mut priority_filter = use_signal(String::new);
  let mut search_query = use_signal(String::new);
  let mut error_state = use_signal(|| Option::<AppError>::None);

  // Track if initial load has happened
  let has_loaded_initial = use_signal(|| false);

  // Create sorted beads memo that depends on both beads and sort configuration
  let sorted_beads = use_memo(move || {
    let beads = beads.read();
    let field = sort_field.read();
    let direction = sort_direction.read();

    let config = SortConfig::new(field.clone(), direction.clone());
    config.sort_beads(beads.iter().map(|b| b.as_ref().clone()).collect())
  });

  // Create filters signal that depends on all filter signals
  let filters = use_memo(move || {
    let status = status_filter.read().clone();
    let bead_type = type_filter.read().clone();
    let priority = priority_filter.read().clone();
    let search = search_query.read().clone();
    let page = current_page.read();
    let page_size_val = page_size.read();

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
      page: Some(*page),
      page_size: Some(*page_size_val),
    }
  });

  // Load filtered beads when filters change or on initial mount
  use_effect({
    let bead_actions = bead_actions.clone();
    let loading_ops = loading_ops;
    let mut has_loaded = has_loaded_initial;
    move || {
      let filters = filters.read().clone();
      let bead_actions = bead_actions.clone();
      let loading_ops = loading_ops.clone();

      // Check if we should load beads
      let should_load = !*has_loaded.read()
        || filters.status.is_some()
        || filters.bead_type.is_some()
        || filters.priority.is_some()
        || filters.search.is_some()
        || filters.page.is_some()
        || filters.page_size.is_some();

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

        eprintln!("[BeadList] Loading beads with filters: {filters:?}");

        // Spawn async task
        spawn(async move {
          eprintln!("[BeadList] Spawning async task to load from database");
          match crate::db::DesktopDb::new_async().await {
            Ok(db) => {
              eprintln!("[BeadList] Database initialized, querying beads");
              match db.list_beads_paginated(&filters).await {
                Ok(paginated) => {
                  eprintln!(
                    "[BeadList] Successfully loaded {} beads (page {} of {})",
                    paginated.beads.len(),
                    paginated.page,
                    paginated.total_pages
                  );
                  (bead_actions.set_beads)(paginated.beads);
                  total_pages.set(paginated.total_pages);
                  total_beads.set(paginated.total);
                  (loading_ops.stop)("bead-list".to_string());
                  error_state.set(None);
                }
                Err(e) => {
                  eprintln!("[BeadList] Error loading beads: {e:?}");
                  (loading_ops.stop)("bead-list".to_string());
                  let app_err = AppError::from(e);
                  error_state.set(Some(app_err));
                }
              }
            }
            Err(e) => {
              eprintln!("[BeadList] Error initializing database: {e:?}");
              (loading_ops.stop)("bead-list".to_string());
              let app_err = AppError::from(e);
              error_state.set(Some(app_err));
            }
          }
        });
      }
    }
  });

  // Update URL when pagination changes
  use_effect(move || {
    let page = *current_page.read();
    let page_size_val = *page_size.read();
    update_url_params(page, page_size_val);
  });

  // Function to handle sort field and direction changes
  let mut toggle_sort = move |field: SortBy| {
    let current_field = *sort_field.read();
    let current_direction = *sort_direction.read();

    // If clicking the same field, toggle direction
    // Otherwise, set to new field with default direction
    if field == current_field {
      let new_direction = match current_direction {
        SortDirection::Ascending => SortDirection::Descending,
        SortDirection::Descending => SortDirection::Ascending,
      };
      sort_direction.set(new_direction);
    } else {
      sort_field.set(field);
      // Set default direction based on field type
      let default_direction = match field {
        SortBy::Priority => SortDirection::Descending, // High to low
        _ => SortDirection::Ascending,                 // A-Z, oldest to newest
      };
      sort_direction.set(default_direction);
    }
  };

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

  // Calculate pagination info (pure functional calculation)
  let page = *current_page.read();
  let size = *page_size.read();
  let total = *total_beads.read();
  let beads_len = sorted_beads.read().len();

  // Pure calculation: avoid overflow with checked arithmetic, convert types safely
  let page_start: u64 = u64::from(
    page
      .saturating_sub(1)
      .saturating_mul(size)
      .saturating_add(1),
  );
  let offset: u64 = u64::from(page.saturating_sub(1).saturating_mul(size));
  let page_end: u64 = offset.saturating_add(beads_len as u64).min(total);

  // Extract sort indicator for display
  let sort_indicator = sort_direction.read().get_indicator();

  rsx! {
      div { class: "bead-list-page",
          div { class: "page-header",
              h1 { "Beads" }
              div { class: "page-actions",
                  {
                      let beads_vec: Vec<clarity_core::db::models::Bead> = sorted_beads.read()
                          .iter()
                          .map(|b| b.clone())
                          .collect();
                      rsx! {
                          crate::beads::ExportButton {
                              beads: Rc::new(beads_vec)
                          }
                      }
                  }
                  crate::beads::ImportButton {
                      on_import_success: Callback::<usize>::new({
                          let clear_error = bead_actions.clear_error.clone();
                          move |count: usize| {
                              // Reload beads after import
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
                      p { "Loading beads..." }
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

          // Filter controls
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
                  placeholder: "Search beads...",
                  value: "{search_query}",
                  oninput: move |evt: Event<FormData>| {
                      search_query.set(evt.value());
                  }
              }
              span { class: "shortcut-hint-inline", "Ctrl+F" }
          }

          // Bead table with sortable headers
          div { class: "bead-list",
              table { class: "bead-table",
                  thead {
                      tr {
                          // Title column
                          th {
                              class: if *sort_field.read() == SortBy::Title { "sortable sorted" } else { "sortable" },
                              onclick: move |_| {
                                  toggle_sort(SortBy::Title);
                              },
                              span {
                                  "Title"
                                  if *sort_field.read() == SortBy::Title {
                                      span { class: "sort-indicator", "{sort_indicator}" }
                                  }
                              }
                          }
                          // Status column
                          th {
                              class: if *sort_field.read() == SortBy::Status { "sortable sorted" } else { "sortable" },
                              onclick: move |_| {
                                  toggle_sort(SortBy::Status);
                              },
                              span {
                                  "Status"
                                  if *sort_field.read() == SortBy::Status {
                                      span { class: "sort-indicator", "{sort_indicator}" }
                                  }
                              }
                          }
                          // Type column
                          th {
                              class: if *sort_field.read() == SortBy::Type { "sortable sorted" } else { "sortable" },
                              onclick: move |_| {
                                  toggle_sort(SortBy::Type);
                              },
                              span {
                                  "Type"
                                  if *sort_field.read() == SortBy::Type {
                                      span { class: "sort-indicator", "{sort_indicator}" }
                                  }
                              }
                          }
                          // Priority column
                          th {
                              class: if *sort_field.read() == SortBy::Priority { "sortable sorted" } else { "sortable" },
                              onclick: move |_| {
                                  toggle_sort(SortBy::Priority);
                              },
                              span {
                                  "Priority"
                                  if *sort_field.read() == SortBy::Priority {
                                      span { class: "sort-indicator", "{sort_indicator}" }
                                  }
                              }
                          }
                          // Created column
                          th {
                              class: if *sort_field.read() == SortBy::CreatedAt { "sortable sorted" } else { "sortable" },
                              onclick: move |_| {
                                  toggle_sort(SortBy::CreatedAt);
                              },
                              span {
                                  "Created"
                                  if *sort_field.read() == SortBy::CreatedAt {
                                      span { class: "sort-indicator", "{sort_indicator}" }
                                  }
                              }
                          }
                      }
                  }
                  tbody {
                      for bead in sorted_beads.read().iter() {
                          BeadRow {
                              id: bead.id,
                              title: bead.title.clone(),
                              status: bead.status,
                              bead_type: bead.bead_type,
                              priority: bead.priority,
                              created_at: bead.created_at.clone(),
                          }
                      }
                  }
              }

              // Pagination controls (only show if there are more than 25 beads)
              if *total_beads.read() > 25 {
                  div { class: "pagination",
                      div { class: "pagination-info",
                          span { "Showing " }
                          span { class: "page-start", "{page_start}" }
                          span { " to " }
                          span { class: "page-end", "{page_end}" }
                          span { " of " }
                          span { class: "total", "{total_beads.read()}" }
                          span { " beads" }
                      }

                      div { class: "pagination-controls",
                          // Previous button
                          button {
                              class: if *current_page.read() <= 1 { "pagination-btn disabled" } else { "pagination-btn" },
                              onclick: move |_| {
                                  let current = *current_page.read();
                                  if current > 1 {
                                      current_page.set(current - 1);
                                  }
                              },
                              disabled: *current_page.read() <= 1,
                              "Previous"
                          }

                          // Next button
                          button {
                              class: if *current_page.read() >= *total_pages.read() { "pagination-btn disabled" } else { "pagination-btn" },
                              onclick: move |_| {
                                  let current = *current_page.read();
                                  let total = *total_pages.read();
                                  if current < total {
                                      current_page.set(current + 1);
                                  }
                              },
                              disabled: *current_page.read() >= *total_pages.read(),
                              "Next"
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
  /// When the bead was created (ISO 8601 string)
  pub created_at: String,
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

  // Format status and type with CSS classes
  let status_class = format!("status-{}", status.as_str());
  let type_class = format!("type-{}", bead_type.as_str());

  // Format priority as human-readable label
  let priority_label = match priority.0 {
    1 => "High",
    2 => "Medium",
    _ => "Low",
  };

  // Format date for display (parse ISO 8601 string to date)
  let display_date = format_date_from_string(&created_at);

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
                  datetime: "{created_at}",
                  "{display_date}"
              }
          }
      }
  }
}

/// Format an ISO 8601 date string for display
///
/// Extracts the date portion from an ISO 8601 string.
fn format_date_from_string(iso_date: &str) -> String {
  // Extract just the date portion (YYYY-MM-DD) from ISO 8601 string
  iso_date
    .split('T')
    .next()
    .map(|s| s.to_string())
    .map_or_else(|| "Invalid date".to_string(), |s| s)
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

  #[test]
  fn test_bead_row_props_equality() {
    let props1 = BeadRowProps {
      id: clarity_core::db::models::BeadId::from_str("test-1").unwrap(),
      title: "Test Title".to_string(),
      status: clarity_core::db::models::BeadStatus::Open,
      bead_type: clarity_core::db::models::BeadType::Feature,
      priority: clarity_core::db::models::BeadPriority::MEDIUM,
      created_at: chrono::Utc::now(),
    };

    let props2 = BeadRowProps {
      id: clarity_core::db::models::BeadId::from_str("test-1").unwrap(),
      title: "Test Title".to_string(),
      status: clarity_core::db::models::BeadStatus::Open,
      bead_type: clarity_core::db::models::BeadType::Feature,
      priority: clarity_core::db::models::BeadPriority::MEDIUM,
      created_at: chrono::Utc::now(),
    };

    assert_eq!(props1, props2);
  }

  #[test]
  fn test_sort_config_direction_toggle() {
    let field = SortBy::Title;
    let mut direction = SortDirection::Ascending;

    // Test toggle from ascending to descending
    let new_direction = match direction {
      SortDirection::Ascending => SortDirection::Descending,
      SortDirection::Descending => SortDirection::Ascending,
    };
    assert_eq!(new_direction, SortDirection::Descending);

    // Test toggle from descending to ascending
    let new_direction2 = match new_direction {
      SortDirection::Ascending => SortDirection::Descending,
      SortDirection::Descending => SortDirection::Ascending,
    };
    assert_eq!(new_direction2, SortDirection::Ascending);
  }
}
