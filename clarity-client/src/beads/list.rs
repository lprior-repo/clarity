//! Bead List Component
//!
//! Displays a list of beads with filtering capabilities using direct database access.

// Dioxus rsx! macro internally uses unwrap, so we allow the disallowed_methods lint.
#![allow(clippy::disallowed_methods)]

use clarity_core::db::models::BeadFilters;
use dioxus::prelude::*;
use std::rc::Rc;

/// Bead list page component
///
/// This component uses direct database access to fetch and display beads.
/// It supports filtering by status, type, priority, and search text.
#[component]
pub fn BeadListPage() -> Element {
    let mut status_filter = use_signal(|| String::new());
    let mut type_filter = use_signal(|| String::new());
    let mut priority_filter = use_signal(|| String::new());
    let mut search_query = use_signal(|| String::new());

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
            search: if search.is_empty() { None } else { Some(search) },
        }
    });

    // Initialize database and load beads synchronously
    let mut beads = use_signal(|| {
        // Try to initialize the database and load beads
        // If this fails, return an empty vec
        Rc::new(
            try_init_db_and_load(None)
                .unwrap_or_else(|_| Vec::new())
        )
    });

    // Update beads when filters change
    use_effect(move || {
        let filters = filters.read().clone();
        let current_filters = if filters.is_active() {
            Some(filters)
        } else {
            None
        };

        if let Ok(loaded_beads) = try_init_db_and_load(current_filters) {
            beads.set(Rc::new(loaded_beads));
        }
    });

    rsx! {
        div { class: "bead-list-page",
            h1 { "Beads" }

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
                    placeholder: "Search...",
                    value: "{search_query}",
                    oninput: move |evt: Event<FormData>| {
                        search_query.set(evt.value());
                    }
                }
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

/// Single bead row component
///
/// Displays a single bead in the table with a link to its detail page.
#[component]
fn BeadRow(
    id: clarity_core::db::models::BeadId,
    title: String,
    status: clarity_core::db::models::BeadStatus,
    bead_type: clarity_core::db::models::BeadType,
    priority: clarity_core::db::models::BeadPriority,
    created_at: chrono::DateTime<chrono::Utc>,
) -> Element {
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
                a {
                    href: format!("/beads/{}", id),
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
/// Converts a chrono DateTime to a human-readable date string.
fn format_date(dt: &chrono::DateTime<chrono::Utc>) -> String {
    dt.format("%Y-%m-%d").to_string()
}

/// Helper function to initialize database and load beads
///
/// This function attempts to initialize the database and load beads.
/// It returns a Result to allow graceful error handling.
fn try_init_db_and_load(filters: Option<BeadFilters>) -> Result<Vec<clarity_core::db::models::Bead>, String> {
    // Initialize database
    let db = crate::db::DesktopDb::new()
        .map_err(|e: anyhow::Error| format!("Failed to initialize database: {}", e))?;

    // Load beads with optional filters
    match filters {
        Some(f) => db.list_beads_filtered(&f)
            .map_err(|e: anyhow::Error| format!("Failed to load beads: {}", e)),
        None => db.list_beads()
            .map_err(|e: anyhow::Error| format!("Failed to load beads: {}", e)),
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
