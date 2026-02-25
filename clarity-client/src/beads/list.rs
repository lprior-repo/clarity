//! Bead List Component
//!
//! Displays a complete list of beads with filtering, sorting, and pagination capabilities.
//! Follows functional programming patterns with zero unwrap rules.

// Dioxus rsx! macro internally uses unwrap, so we allow the disallowed_methods lint.
#![allow(clippy::disallowed_methods)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::clone_on_copy)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::map_clone)]
#![allow(clippy::unnecessary_option_map_or_else)]

use crate::beads::sorting::{SortBy, SortConfig, SortDirection};
use crate::error::AppError;
use crate::hooks::{use_keyboard_with_handler, use_loading_manager, use_loading_operations};
use crate::shortcuts::Action;
use clarity_core::db::models::{BeadFilters, BeadStatus, BeadType};
use clarity_core::domain::types::BeadPriority;
use dioxus::prelude::*;
use std::rc::Rc;

// ===== Filter State Store =====

/// Bead list filter state using Dioxus 0.7 Store pattern
///
/// Consolidates all filter-related UI state into a single reactive structure.
/// This replaces multiple independent signals with a coherent state machine.
#[derive(Clone, PartialEq, Eq, Store, Debug)]
pub struct BeadListFilters {
  /// Status filter (None = all statuses)
  pub status: Option<BeadStatus>,
  /// Type filter (None = all types)
  pub bead_type: Option<BeadType>,
  /// Priority filter (None = all priorities)
  pub priority: Option<BeadPriority>,
  /// Search query text (None = no search)
  pub search_query: Option<String>,
  /// Current page number
  pub page: u32,
  /// Page size
  pub_size: u32,
}

impl BeadListFilters {
  /// Create default filters (no filters applied, page 1, size 25)
  #[must_use]
  pub const fn new() -> Self {
    Self {
      status: None,
      bead_type: None,
      priority: None,
      search_query: None,
      page: 1,
      pub_size: 25,
    }
  }

  /// Check if any non-pagination filters are active
  #[must_use]
  pub const fn has_filters(&self) -> bool {
    self.status.is_some()
      || self.bead_type.is_some()
      || self.priority.is_some()
      || self.search_query.is_some()
  }

  /// Create with status filter
  #[must_use]
  pub const fn with_status(mut self, status: Option<BeadStatus>) -> Self {
    self.status = status;
    self
  }

  /// Create with type filter
  #[must_use]
  pub const fn with_type(mut self, bead_type: Option<BeadType>) -> Self {
    self.bead_type = bead_type;
    self
  }

  /// Create with priority filter
  #[must_use]
  pub const fn with_priority(mut self, priority: Option<BeadPriority>) -> Self {
    self.priority = priority;
    self
  }

  /// Create with search query
  #[must_use]
  pub fn with_search(mut self, search: &str) -> Self {
    self.search_query = if search.is_empty() {
      None
    } else {
      Some(search.to_string())
    };
    self
  }

  /// Create with page
  #[must_use]
  pub const fn with_page(mut self, page: u32) -> Self {
    self.page = page;
    self
  }

  /// Reset all filters (keep pagination)
  #[must_use]
  pub fn reset_filters(mut self) -> Self {
    self.status = None;
    self.bead_type = None;
    self.priority = None;
    self.search_query = None;
    self
  }
}

impl Default for BeadListFilters {
  fn default() -> Self {
    Self::new()
  }
}

// ===== LoadingState Constructors =====

impl<T> LoadingState<T> {
  /// Create an idle state (no load attempted yet)
  #[must_use]
  pub const fn idle() -> Self {
    Self::Idle
  }

  /// Create a loading state
  #[must_use]
  pub const fn loading() -> Self {
    Self::Loading
  }

  /// Create a loaded state with data
  #[must_use]
  pub const fn loaded(data: T) -> Self {
    Self::Loaded(data)
  }

  /// Create a failed state with error message
  #[must_use]
  pub fn failed(msg: String) -> Self {
    Self::Failed(msg)
  }
}

// ===== Loading State Enum =====

/// Explicit loading state per Scott Wlaschin DDD principles
///
/// Replaces Option-as-state pattern with explicit state machine.
/// Makes illegal states unrepresentable.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum LoadingState<T> {
  /// Initial state, no data loaded yet
  Idle,
  /// Currently loading data
  Loading,
  /// Successfully loaded data
  Loaded(T),
  /// Failed to load data
  Failed(String),
}

impl<T> LoadingState<T> {
  /// Check if currently loading
  #[must_use]
  pub const fn is_loading(&self) -> bool {
    matches!(self, Self::Loading)
  }

  /// Check if is idle (no load attempted)
  #[must_use]
  pub const fn is_idle(&self) -> bool {
    matches!(self, Self::Idle)
  }

  /// Check if has successfully loaded
  #[must_use]
  pub const fn is_loaded(&self) -> bool {
    matches!(self, Self::Loaded(_))
  }

  /// Check if failed
  #[must_use]
  pub const fn is_failed(&self) -> bool {
    matches!(self, Self::Failed(_))
  }

  /// Get reference to loaded data if available
  #[must_use]
  pub fn data(&self) -> Option<&T> {
    match self {
      Self::Loaded(data) => Some(data),
      _ => None,
    }
  }

  /// Map over the loaded data
  #[must_use]
  pub fn map<U, F>(self, f: F) -> LoadingState<U>
  where
    F: FnOnce(T) -> U,
  {
    match self {
      Self::Idle => LoadingState::idle(),
      Self::Loading => LoadingState::loading(),
      Self::Loaded(data) => LoadingState::loaded(f(data)),
      Self::Failed(err) => LoadingState::failed(err),
    }
  }

  /// Get the error message if failed
  #[must_use]
  pub fn error(&self) -> Option<&String> {
    match self {
      Self::Failed(msg) => Some(msg),
      _ => None,
    }
  }
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
  let loading_ops = use_loading_operations();

  // Reactively read beads from the signal - this will trigger re-renders when beads change
  let beads = use_memo(move || app_state.bead_list());

  // Unified filter state using Dioxus 0.7 Store pattern
  // Consolidates all filter-related state into a single reactive structure
  let mut filters = use_signal(BeadListFilters::new);

  // Explicit loading state (replaces Option-as-state anti-pattern)
  let loading_state = use_signal(|| LoadingState::idle());

  // Sorting signals (kept for future use)
  let _sort_field = use_signal(|| SortBy::CreatedAt);
  let _sort_direction = use_signal(|| SortDirection::Descending);

  // Pagination state signals
  let mut total_pages = use_signal(|| 1u32);
  let mut total_beads = use_signal(|| 0u64);

  // Sorting signals
  let mut sort_field = use_signal(|| SortBy::CreatedAt);
  let mut sort_direction = use_signal(|| SortDirection::Descending);

  // Create sorted beads memo that depends on both beads and sort configuration
  let sorted_beads = use_memo(move || {
    let beads = beads.read();
    let field = sort_field.read();
    let direction = sort_direction.read();

    let config = SortConfig::new(field.clone(), direction.clone());
    config.sort_beads(beads.iter().map(|b| b.as_ref().clone()).collect())
  });


  // Load filtered beads when filters change or on initial mount
  use_effect({
    let bead_actions = bead_actions.clone();
    let loading_ops = loading_ops;
    let mut loading_state = loading_state;
    move || {
      let bead_actions = bead_actions.clone();
      let loading_ops = loading_ops.clone();

      // Check if we should load beads (first load or filters changed)
      let should_load = matches!(*loading_state.read(), LoadingState::Idle)
        || filters.read().has_filters();

      eprintln!(
        "[BeadList] Effect triggered - should_load: {}, state: {:?}, filters: {:?}",
        should_load,
        loading_state.read(),
        filters.read()
      );

      if should_load {
        // Set loading state
        loading_state.set(LoadingState::loading());
        (loading_ops.start)(("bead-list".to_string(), "Loading beads...".to_string()));

        // Convert Store filters to DB filters
        let f = filters.read();
        let db_filters = BeadFilters {
          status: f.status.map(|s| s.to_string()),
          bead_type: f.bead_type.map(|t| t.to_string()),
          priority: f.priority.map(|p| p.sort_value()),
          created_by: None,
          search: f.search_query.clone(),
          page: Some(f.page),
          page_size: Some(f.pub_size),
        };

        eprintln!("[BeadList] Loading beads with filters: {db_filters:?}");

        // Spawn async task
        spawn(async move {
          eprintln!("[BeadList] Spawning async task to load from database");
          match crate::db::DesktopDb::new_async().await {
            Ok(db) => {
              eprintln!("[BeadList] Database initialized, querying beads");
              match db.list_beads_paginated(&db_filters).await {
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
                  loading_state.set(LoadingState::loaded(()));
                }
                Err(e) => {
                  eprintln!("[BeadList] Error loading beads: {e:?}");
                  (loading_ops.stop)("bead-list".to_string());
                  let app_err = AppError::from(e);
                  loading_state.set(LoadingState::failed(app_err.to_string()));
                }
              }
            }
            Err(e) => {
              eprintln!("[BeadList] Error initializing database: {e:?}");
              (loading_ops.stop)("bead-list".to_string());
              let app_err = AppError::from(e);
              loading_state.set(LoadingState::failed(app_err.to_string()));
            }
          }
        });
      }
    }
  });

  // Update URL when pagination changes (no-op for desktop)
  use_effect(move || {
    let page = filters.read().page;
    let page_size_val = filters.read().pub_size;
    // Desktop app doesn't have URL APIs
    let _ = (page, page_size_val);
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
  let page = filters.read().page;
  let size = filters.read().pub_size;
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

          // Show loading state using explicit LoadingState enum
          {if loading_state.read().is_loading() {
              rsx! {
                  div { class: "loading",
                      p { "Loading beads..." }
                  }
              }
          } else {
              rsx! {}
          }}

          // Show error state using explicit LoadingState enum
          {if loading_state.read().is_failed() {
              let error_msg = loading_state.read()
                  .error()
                  .cloned()
                  .unwrap_or_else(|| "Unknown error".to_string());
              rsx! {
                  div { class: "error",
                      p { "{error_msg}" }
                  }
              }
          } else {
              rsx! {}
          }}

          // Filter controls using Store-based state
          div { class: "filters",
              select {
                  value: "{filters.read().status.map_or(String::new(), |s| s.to_string())}",
                  onchange: move |evt: Event<FormData>| {
                      let value = evt.value();
                      let new_status = if value.is_empty() {
                          None
                      } else {
                          value.parse().ok()
                      };
                      filters.write().status = new_status;
                  },
                  option { value: "", "All Statuses" },
                  option { value: "open", "Open" },
                  option { value: "in_progress", "In Progress" },
                  option { value: "blocked", "Blocked" },
                  option { value: "deferred", "Deferred" },
                  option { value: "closed", "Closed" },
              }

              select {
                  value: "{filters.read().bead_type.map_or(String::new(), |t| t.to_string())}",
                  onchange: move |evt: Event<FormData>| {
                      let value = evt.value();
                      let new_type = if value.is_empty() {
                          None
                      } else {
                          value.parse().ok()
                      };
                      filters.write().bead_type = new_type;
                  },
                  option { value: "", "All Types" },
                  option { value: "feature", "Feature" },
                  option { value: "bugfix", "Bug Fix" },
                  option { value: "refactor", "Refactor" },
                  option { value: "test", "Test" },
                  option { value: "docs", "Documentation" },
              }

              select {
                  value: "{filters.read().priority.map_or(String::new(), |p| p.sort_value().to_string())}",
                  onchange: move |evt: Event<FormData>| {
                      let value = evt.value();
                      let new_priority = if value.is_empty() {
                          None
                      } else {
                          value.parse::<i16>().ok().and_then(|v| BeadPriority::from_value(v).ok())
                      };
                      filters.write().priority = new_priority;
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
                  value: "{filters.read().search_query.clone().unwrap_or_default()}",
                  oninput: move |evt: Event<FormData>| {
                      let value = evt.value();
                      let new_search = if value.is_empty() {
                          None
                      } else {
                          Some(value)
                      };
                      filters.write().search_query = new_search;
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
                              class: if filters.read().page <= 1 { "pagination-btn disabled" } else { "pagination-btn" },
                              onclick: move |_| {
                                  let current = filters.read().page;
                                  if current > 1 {
                                      filters.write().page = current - 1;
                                  }
                              },
                              disabled: filters.read().page <= 1,
                              "Previous"
                          }

                          // Next button
                          button {
                              class: if filters.read().page >= *total_pages.read() { "pagination-btn disabled" } else { "pagination-btn" },
                              onclick: move |_| {
                                  let current = filters.read().page;
                                  let total = *total_pages.read();
                                  if current < total {
                                      filters.write().page = current + 1;
                                  }
                              },
                              disabled: filters.read().page >= *total_pages.read(),
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
#[cfg_attr(test, derive(Debug))]
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
  let priority_label = match priority {
    BeadPriority::High => "High",
    BeadPriority::Medium => "Medium",
    BeadPriority::Low => "Low",
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
  fn test_format_date_from_string() {
    let iso_date = "2024-02-09T12:00:00Z";
    assert_eq!(format_date_from_string(iso_date), "2024-02-09");
  }

  #[test]
  fn test_format_date_from_string_invalid() {
    let invalid = "not-a-date";
    assert_eq!(format_date_from_string(invalid), "not-a-date");
  }

  #[test]
  fn test_bead_row_props_equality() -> Result<(), clarity_core::db::error::DbError> {
    let id = clarity_core::db::models::BeadId::from_str("550e8400-e29b-41d4-a716-446655440000")?;
    let created_at = "2024-02-09T12:00:00Z".to_string();

    let props1 = BeadRowProps {
      id: id.clone(),
      title: "Test Title".to_string(),
      status: clarity_core::db::models::BeadStatus::Open,
      bead_type: clarity_core::db::models::BeadType::Feature,
      priority: clarity_core::db::models::BeadPriority::MEDIUM,
      created_at: created_at.clone(),
    };

    let props2 = BeadRowProps {
      id,
      title: "Test Title".to_string(),
      status: clarity_core::db::models::BeadStatus::Open,
      bead_type: clarity_core::db::models::BeadType::Feature,
      priority: clarity_core::db::models::BeadPriority::MEDIUM,
      created_at,
    };

    assert_eq!(props1, props2);
    Ok(())
  }

  #[test]
  fn test_sort_config_direction_toggle() {
    let direction = SortDirection::Ascending;

    // Test toggle from ascending to descending using functional pattern
    let new_direction = direction.toggle();
    assert_eq!(new_direction, SortDirection::Descending);

    // Test toggle from descending to ascending
    let new_direction2 = new_direction.toggle();
    assert_eq!(new_direction2, SortDirection::Ascending);
  }
}
