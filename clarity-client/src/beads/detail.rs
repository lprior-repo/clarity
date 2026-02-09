//! Bead Detail Component
//!
//! Displays detailed information about a single bead with edit and delete actions.

// Dioxus rsx! macro internally uses unwrap, so we allow the disallowed_methods lint.
#![allow(clippy::disallowed_methods)]

use clarity_core::db::models::{Bead, BeadId};
use dioxus::prelude::*;

/// Bead detail page component
///
/// Shows full details of a bead and provides actions to edit or delete it.
/// Uses direct database access to fetch bead data.
#[component]
pub fn BeadDetailPage(id: String) -> Element {
    // Parse the bead ID from the string
    let bead_id = match BeadId::from_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return rsx! {
                div { class: "bead-detail-page",
                    div { class: "error",
                        h2 { "Error Loading Bead" }
                        p { "Invalid bead ID format" }
                        a { href: "/beads", class: "back-link", "Back to Beads" }
                    }
                }
            };
        }
    };

    // Load bead synchronously from database
    let bead = use_signal(|| {
        try_load_bead(bead_id).ok()
    });

    rsx! {
        div { class: "bead-detail-page",
            match &*bead.read() {
                None => rsx! {
                    div { class: "error",
                        h2 { "Error Loading Bead" }
                        p { "Bead not found or database error" }
                        a { href: "/beads", class: "back-link", "Back to Beads" }
                    }
                },
                Some(bead_data) => rsx! {
                    BeadDetail {
                        id: bead_data.id,
                        title: bead_data.title.clone(),
                        description: bead_data.description.clone(),
                        status: bead_data.status,
                        bead_type: bead_data.bead_type,
                        priority: bead_data.priority,
                        created_by: bead_data.created_by,
                        created_at: bead_data.created_at,
                        updated_at: bead_data.updated_at,
                    }
                },
            }
        }
    }
}

/// Bead detail component
///
/// Renders the full bead information with action buttons.
#[component]
fn BeadDetail(
    id: BeadId,
    title: String,
    description: Option<String>,
    status: clarity_core::db::models::BeadStatus,
    bead_type: clarity_core::db::models::BeadType,
    priority: clarity_core::db::models::BeadPriority,
    created_by: Option<clarity_core::db::models::UserId>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
) -> Element {
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
                {match created_by {
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

/// Delete bead button component
///
/// A button that shows a confirmation dialog before deleting a bead.
/// Uses direct database access to perform the deletion.
#[component]
fn DeleteBeadButton(bead_id: BeadId) -> Element {
    let mut show_confirm = use_signal(|| false);
    let mut is_deleting = use_signal(|| false);

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
                                    // Success - navigation will happen via link
                                }
                                Err(e) => {
                                    // Error handling would go here
                                    eprintln!("Delete error: {}", e);
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

/// Delete handler component
///
/// Handles the actual deletion by calling direct database functions.
#[component]
fn DeleteHandler(
    bead_id: BeadId,
    on_complete: EventHandler<Result<(), String>>,
) -> Element {
    let mut is_done = use_signal(|| false);
    let result = use_signal(|| {
        match try_delete_bead(bead_id) {
            Ok(()) => {
                is_done.set(true);
                Some(Ok(()))
            },
            Err(e) => Some(Err(e)),
        }
    });

    rsx! {
        {match &*result.read() {
            None => rsx! {
                div { class: "deleting", "Deleting..." }
            },
            Some(Ok(())) => {
                if !*is_done.read() {
                    is_done.set(true);
                    on_complete.call(Ok(()));
                }
                rsx! {
                    div { class: "success-message",
                        "Bead deleted successfully! "
                        a { href: "/beads", "Back to list" }
                    }
                }
            }
            Some(Err(e)) => {
                if !*is_done.read() {
                    is_done.set(true);
                    on_complete.call(Err(e.clone()));
                }
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
/// Converts a chrono DateTime to a human-readable date and time string.
fn format_datetime(dt: &chrono::DateTime<chrono::Utc>) -> String {
    dt.format("%Y-%m-%d at %H:%M UTC").to_string()
}

/// Helper function to load a bead from the database
///
/// This function attempts to initialize the database and load a bead by ID.
fn try_load_bead(id: BeadId) -> Result<Bead, String> {
    let db = crate::db::DesktopDb::new()
        .map_err(|e| format!("Failed to initialize database: {}", e))?;

    db.get_bead(id)
        .map_err(|e| format!("Failed to load bead: {}", e))
}

/// Helper function to delete a bead from the database
///
/// This function attempts to initialize the database and delete a bead by ID.
fn try_delete_bead(id: BeadId) -> Result<(), String> {
    let db = crate::db::DesktopDb::new()
        .map_err(|e| format!("Failed to initialize database: {}", e))?;

    db.delete_bead(id)
        .map_err(|e| format!("Failed to delete bead: {}", e))
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
