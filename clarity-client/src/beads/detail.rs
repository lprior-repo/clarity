//! Bead Detail Component
//!
//! Displays detailed information about a single bead with edit and delete actions.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
// Dioxus rsx! macro internally uses unwrap, so we allow the disallowed_methods lint.
#![allow(clippy::disallowed_methods)]

use clarity_core::db::models::{Bead, BeadId};
use dioxus::prelude::*;
use std::rc::Rc;

/// Bead detail page component
///
/// Shows full details of a bead and provides actions to edit or delete it.
/// Uses async database access to fetch bead data.
#[component]
pub fn BeadDetailPage(id: String) -> Element {
  // Parse the bead ID from the string - do this before rsx!
  let bead_id = BeadId::from_str(&id);
  let is_valid = bead_id.is_ok();

  // Extract the valid bead ID for use in async operations
  let valid_bead_id = bead_id.ok().unwrap_or_else(|| {
    clarity_core::db::models::BeadId::from_str("invalid")
      .unwrap_or_else(|_| clarity_core::db::models::BeadId::new())
  });

  // Create signals before rsx! block
  let bead = use_signal(|| Option::<Rc<Bead>>::None);
  let error_state = use_signal(|| Option::<String>::None);
  let mut has_loaded = use_signal(|| false);

  rsx! {
        div { class: "bead-detail-page",
            // Show error if bead ID is invalid
            if !is_valid {
                div { class: "error",
                    h2 { "Error Loading Bead" }
                    p { "Invalid bead ID format" }
                    crate::app::NavLink {
                        to: crate::app::Route::BeadsList,
                        class: "back-link",
                        "Back to Beads"
                    }
                }
            } else {
                // Load bead on mount
                {
                    let bead_id = valid_bead_id;
                    let mut bead = bead.clone();
                    let mut error_state = error_state.clone();

                    use_effect(move || {
                      if *has_loaded.read() {
                        return;
                      }
                      has_loaded.set(true);

                      spawn(async move {
                        eprintln!("[BeadDetail] Loading bead {bead_id} from database");
                        match crate::db::DesktopDb::new_async().await {
                          Ok(db) => match db.get_bead(bead_id).await {
                            Ok(b) => {
                              eprintln!("[BeadDetail] Successfully loaded bead: {}", b.title);
                              bead.set(Some(Rc::new(b)));
                            }
                            Err(e) => {
                              eprintln!("[BeadDetail] Error loading bead: {e:?}");
                              error_state.set(Some(format!("Failed to load bead: {e}")));
                            }
                          },
                          Err(e) => {
                            eprintln!("[BeadDetail] Error initializing database: {e:?}");
                            error_state.set(Some(format!("Database error: {e}")));
                          }
                        }
                      });
                    });
                }

                // Main content area
                div {
                    // Show error state
                    {error_state.read().as_ref().map(|error| {
                        rsx! {
                            div { class: "error",
                                h2 { "Error Loading Bead" }
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
                    {if bead.read().is_none() && error_state.read().is_none() {
                        rsx! {
                            div { class: "loading",
                                p { "Loading bead..." }
                            }
                        }
                    } else {
                        rsx! {}
                    }}

                    // Show bead data
                    {bead.read().as_ref().map(|bead_data| {
                        // Parse the date strings into DateTime
                        let created_at_dt = chrono::DateTime::parse_from_rfc3339(&bead_data.created_at)
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                            .unwrap_or_else(|_| chrono::Utc::now());
                        let updated_at_dt = chrono::DateTime::parse_from_rfc3339(&bead_data.updated_at)
                            .map(|dt| dt.with_timezone(&chrono::Utc))
                            .unwrap_or_else(|_| chrono::Utc::now());

                        rsx! {
                            BeadDetail {
                                id: bead_data.id,
                                title: bead_data.title.clone(),
                                description: bead_data.description.clone(),
                                status: bead_data.status,
                                bead_type: bead_data.bead_type,
                                priority: bead_data.priority,
                                created_by: bead_data.created_by,
                                created_at: created_at_dt,
                                updated_at: updated_at_dt,
                            }
                        }
                    })}
                }
            }
        }
  }
}

/// Bead detail component properties
#[derive(Clone, Props, PartialEq, Eq)]
pub struct BeadDetailProps {
  /// The bead ID
  pub id: BeadId,
  /// The bead title
  pub title: String,
  /// The bead description
  pub description: Option<String>,
  /// The bead status
  pub status: clarity_core::db::models::BeadStatus,
  /// The bead type
  pub bead_type: clarity_core::db::models::BeadType,
  /// The bead priority
  pub priority: clarity_core::db::models::BeadPriority,
  /// Who created the bead
  pub created_by: Option<clarity_core::db::models::UserId>,
  /// When the bead was created
  pub created_at: chrono::DateTime<chrono::Utc>,
  /// When the bead was last updated
  pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Bead detail component
///
/// Renders the full bead information with action buttons.
#[component]
fn BeadDetail(props: BeadDetailProps) -> Element {
  let id = props.id;
  let title = props.title;
  let description = props.description;
  let status = props.status;
  let bead_type = props.bead_type;
  let priority = props.priority;
  let created_by = props.created_by;
  let created_at = props.created_at;
  let updated_at = props.updated_at;
  let status_class = format!("status-{}", status.as_str());
  let type_class = format!("type-{}", bead_type.as_str());
  let priority_label = match priority.0 {
    1 => "High",
    2 => "Medium",
    _ => "Low",
  };

  rsx! {
      div { class: "bead-detail",
          div { class: "bead-header",
              h1 { class: "bead-title", "{title}" }
              div { class: "bead-meta",
                  span { class: "{status_class}", "{status.as_str()}" }
                  span { class: "{type_class}", "{bead_type.as_str()}" }
                  span { class: "priority", "Priority: {priority_label}" }
              }
          }

          div { class: "bead-description",
              h2 { "Description" }
              {match &description {
                  Some(desc) if !desc.is_empty() => rsx! {
                      p { "{desc}" }
                  },
                  _ => rsx! {
                      p { class: "empty", "No description provided." }
                  },
              }}
          }

          div { class: "bead-info",
              div { class: "info-row",
                  span { class: "info-label", "Created:" }
                  time {
                      datetime: created_at.to_rfc3339(),
                      "{format_datetime(&created_at)}"
                  }
              }
              div { class: "info-row",
                  span { class: "info-label", "Updated:" }
                  time {
                      datetime: updated_at.to_rfc3339(),
                      "{format_datetime(&updated_at)}"
                  }
              }
              {#[allow(clippy::option_if_let_else)]
              match created_by {
                  Some(user_id) => rsx! {
                      div { class: "info-row",
                          span { class: "info-label", "Created By:" }
                          span { "{user_id}" }
                      }
                  },
                  None => rsx! {},
              }}
          }

          div { class: "bead-actions",
              a {
                  href: format!("/beads/{}/edit", id),
                  class: "btn btn-primary",
                  "Edit Bead"
              }
              DeleteBeadButton { bead_id: id }
              a {
                  href: "/beads",
                  class: "btn btn-secondary",
                  "Back to List"
              }
          }
      }
  }
}

/// Delete bead button component properties
#[derive(Clone, Props, PartialEq, Eq)]
pub struct DeleteBeadButtonProps {
  /// The bead ID to delete
  pub bead_id: BeadId,
}

/// Delete bead button component
///
/// A button that shows a confirmation dialog before deleting a bead.
/// Uses direct database access to perform the deletion with programmatic navigation.
#[component]
fn DeleteBeadButton(props: DeleteBeadButtonProps) -> Element {
  let mut show_confirm = use_signal(|| false);
  let mut is_deleting = use_signal(|| false);
  let bead_id = props.bead_id;
  let navigator = crate::hooks::use_state::use_navigator();

  rsx! {
      div { class: "delete-bead-wrapper",
          if !*show_confirm.read() {
              button {
                  class: "btn btn-danger",
                  onclick: move |_| {
                      show_confirm.set(true);
                  },
                  "Delete Bead"
              }
          } else {
              if *is_deleting.read() {
                  DeleteHandler {
                      bead_id,
                      on_complete: move |result| {
                          is_deleting.set(false);
                          match result {
                              Ok(()) => {
                                  // Success - navigate to beads list after deletion
                                  let nav = navigator.clone();
                                  nav(crate::app::Route::BeadsList);
                              }
                              Err(e) => {
                                  // Error handling would go here
                                  eprintln!("Delete error: {e}");
                              }
                          }
                      }
                  }
              }

              div { class: "confirm-dialog",
                  p { "Are you sure you want to delete this bead? This action cannot be undone." }
                  div { class: "confirm-actions",
                      button {
                          class: "btn btn-danger",
                          onclick: move |_| {
                              is_deleting.set(true);
                          },
                          disabled: *is_deleting.read(),
                          "Yes, Delete"
                      }
                      button {
                          class: "btn btn-secondary",
                          onclick: move |_| {
                              show_confirm.set(false);
                          },
                          "Cancel"
                      }
                  }
              }
          }
      }
  }
}

/// Delete handler component properties
#[derive(Clone, Props)]
pub struct DeleteHandlerProps {
  /// The bead ID to delete
  pub bead_id: BeadId,
  /// Callback when deletion is complete
  pub on_complete: EventHandler<Result<(), String>>,
}

// Manual PartialEq implementation since EventHandler doesn't implement PartialEq
impl PartialEq for DeleteHandlerProps {
  fn eq(&self, other: &Self) -> bool {
    self.bead_id == other.bead_id // Always consider equal since we can't compare EventHandlers
  }
}

/// Delete handler component
///
/// Handles the actual deletion by calling async database functions.
#[component]
fn DeleteHandler(props: DeleteHandlerProps) -> Element {
  let is_done = use_signal(|| false);
  let bead_id = props.bead_id;
  let on_complete = props.on_complete;
  let result = use_signal(|| Option::<Result<(), String>>::None);
  let mut has_started = use_signal(|| false);

  // Start deletion on mount
  use_effect(move || {
    if *has_started.read() {
      return;
    }
    has_started.set(true);

    let bead_id = bead_id;
    let mut result = result;
    let mut is_done = is_done;
    let on_complete = on_complete;

    spawn(async move {
      eprintln!("[DeleteHandler] Deleting bead {bead_id}");
      match crate::db::DesktopDb::new_async().await {
        Ok(db) => match db.delete_bead(bead_id).await {
          Ok(()) => {
            eprintln!("[DeleteHandler] Successfully deleted bead");
            is_done.set(true);
            result.set(Some(Ok(())));
            on_complete.call(Ok(()));
          }
          Err(e) => {
            eprintln!("[DeleteHandler] Error deleting bead: {e:?}");
            let error = format!("Failed to delete bead: {e}");
            result.set(Some(Err(error.clone())));
            is_done.set(true);
            on_complete.call(Err(error));
          }
        },
        Err(e) => {
          eprintln!("[DeleteHandler] Error initializing database: {e:?}");
          let error = format!("Database error: {e}");
          result.set(Some(Err(error.clone())));
          is_done.set(true);
          on_complete.call(Err(error));
        }
      }
    });
  });

  rsx! {
      {match &*result.read() {
          None => rsx! {
              div { class: "deleting", "Deleting..." }
          },
          Some(Ok(())) => {
              rsx! {
                  div { class: "success-message",
                      "Bead deleted successfully! "
                      crate::app::NavLink {
                          to: crate::app::Route::BeadsList,
                          "Back to list"
                      }
                  }
              }
          }
          Some(Err(e)) => {
              rsx! {
                  div { class: "error-message",
                      "Failed to delete bead: {e}"
                  }
              }
          }
      }}
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
    let dt_result = chrono::DateTime::parse_from_rfc3339("2024-02-09T12:34:56Z");
    match dt_result {
      Ok(parsed_dt) => {
        let dt = parsed_dt.with_timezone(&chrono::Utc);
        assert_eq!(format_datetime(&dt), "2024-02-09 at 12:34 UTC");
      }
      Err(_) => {
        // If parsing fails, skip the test
        // In production code, we'd use proper error handling
        println!("Skipping datetime test - invalid input");
      }
    }
  }
}
