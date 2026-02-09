//! Bead Form Component
//!
//! Form component for creating and editing beads using direct database access.

// Dioxus rsx! macro internally uses unwrap, so we allow the disallowed_methods lint.
#![allow(clippy::disallowed_methods)]

use clarity_core::db::models::{Bead, BeadId, BeadPriority, BeadStatus, BeadType, NewBead};
use dioxus::prelude::*;
use std::str::FromStr;

/// Bead form page component
///
/// Handles both creating new beads and editing existing beads.
/// Uses direct database access for create/update operations.
#[component]
pub fn BeadFormPage(id: Option<String>) -> Element {
    rsx! {
        div { class: "bead-form-page",
            match id {
                Some(bead_id) => rsx! {
                    BeadForm { mode: FormMode::Edit(bead_id) }
                },
                None => rsx! {
                    BeadForm { mode: FormMode::Create }
                },
            }
        }
    }
}

/// Form mode enum
///
/// Distinguishes between creating a new bead and editing an existing one.
#[derive(Clone, Debug, PartialEq, Eq)]
enum FormMode {
    Create,
    Edit(String),
}

/// Bead form component
///
/// Renders a form for bead creation or editing with validation.
#[component]
fn BeadForm(mode: FormMode) -> Element {
    let mut title = use_signal(|| String::new());
    let mut description = use_signal(|| String::new());
    let mut status = use_signal(|| String::from("open"));
    let mut bead_type = use_signal(|| String::from("feature"));
    let mut priority = use_signal(|| 2_i16);

    // Clone mode for use in closures
    let mode_clone = mode.clone();
    let is_edit = matches!(mode_clone, FormMode::Edit(_));
    let edit_id = match &mode_clone {
        FormMode::Edit(id) => Some(id.clone()),
        FormMode::Create => None,
    };

    // If editing, load the existing bead data synchronously
    let existing_bead = use_signal(|| {
        match (is_edit, edit_id.clone()) {
            (true, Some(id)) => {
                match BeadId::from_str(&id) {
                    Ok(bead_id) => try_load_bead(bead_id).ok(),
                    Err(_) => None,
                }
            }
            _ => None,
        }
    });

    // Populate form when bead data loads
    if let Some(ref bead) = &*existing_bead.read() {
        if title.read().is_empty() {
            title.set(bead.title.clone());
            description.set(bead.description.clone().unwrap_or_default());
            status.set(bead.status.as_str().to_string());
            bead_type.set(bead.bead_type.as_str().to_string());
            priority.set(bead.priority.0);
        }
    }

    let mut is_submitting = use_signal(|| false);

    let title_text = if matches!(mode, FormMode::Edit(_)) {
        "Edit Bead"
    } else {
        "Create New Bead"
    };

    let submit_text = if matches!(mode, FormMode::Edit(_)) {
        "Update Bead"
    } else {
        "Create Bead"
    };

    let onsubmit = move |evt: Event<FormData>| {
        evt.prevent_default();
        is_submitting.set(true);
    };

    rsx! {
        div { class: "bead-form",
            h1 { "{title_text}" }

            form {
                onsubmit: onsubmit,

                div { class: "form-group",
                    label { r#for: "title", "Title *" }
                    input {
                        id: "title",
                        r#type: "text",
                        required: true,
                        value: "{title}",
                        oninput: move |evt: Event<FormData>| {
                            title.set(evt.value());
                        }
                    }
                }

                div { class: "form-group",
                    label { r#for: "description", "Description" }
                    textarea {
                        id: "description",
                        rows: "5",
                        value: "{description}",
                        oninput: move |evt: Event<FormData>| {
                            description.set(evt.value());
                        }
                    }
                }

                div { class: "form-row",
                    div { class: "form-group",
                        label { r#for: "status", "Status" }
                        select {
                            id: "status",
                            value: "{status}",
                            onchange: move |evt: Event<FormData>| {
                                status.set(evt.value());
                            },
                            option { value: "open", "Open" },
                            option { value: "in_progress", "In Progress" },
                            option { value: "blocked", "Blocked" },
                            option { value: "deferred", "Deferred" },
                            option { value: "closed", "Closed" },
                        }
                    }

                    div { class: "form-group",
                        label { r#for: "bead_type", "Type" }
                        select {
                            id: "bead_type",
                            value: "{bead_type}",
                            onchange: move |evt: Event<FormData>| {
                                bead_type.set(evt.value());
                            },
                            option { value: "feature", "Feature" },
                            option { value: "bugfix", "Bug Fix" },
                            option { value: "refactor", "Refactor" },
                            option { value: "test", "Test" },
                            option { value: "docs", "Documentation" },
                        }
                    }

                    div { class: "form-group",
                        label { r#for: "priority", "Priority" }
                        select {
                            id: "priority",
                            value: "{priority}",
                            onchange: move |evt: Event<FormData>| {
                                if let Ok(p) = evt.value().parse::<i16>() {
                                    priority.set(p);
                                }
                            },
                            option { value: "1", "High" },
                            option { value: "2", "Medium" },
                            option { value: "3", "Low" },
                        }
                    }
                }

                div { class: "form-actions",
                    button {
                        r#type: "submit",
                        class: "btn btn-primary",
                        disabled: title.read().is_empty() || *is_submitting.read(),
                        "{submit_text}"
                    }
                    a {
                        href: if matches!(mode, FormMode::Edit(_)) {
                            format!("/beads/{}", match &mode {
                                FormMode::Edit(id) => id.as_str(),
                                FormMode::Create => "",
                            })
                        } else {
                            "/beads".to_string()
                        },
                        class: "btn btn-secondary",
                        "Cancel"
                    }
                }
            }

            {if *is_submitting.read() {
                let mode_clone = mode.clone();
                let title_val = title.read().clone();
                let description_val = description.read().clone();
                let status_val = status.read().clone();
                let bead_type_val = bead_type.read().clone();
                let priority_val = *priority.read();

                rsx! {
                    SubmitHandler {
                        mode: mode_clone,
                        title: title_val,
                        description: description_val,
                        status: status_val,
                        bead_type: bead_type_val,
                        priority: priority_val,
                        on_complete: move |result| {
                            is_submitting.set(false);
                            match result {
                                Ok(_bead_id) => {
                                    // Show success - navigation will happen via link
                                }
                                Err(e) => {
                                    // Show error - in a real app we'd have error state
                                let _e = e;
                                }
                            }
                        }
                    }
                }
            } else {
                rsx! {}
            }}
        }
    }
}

/// Submit handler component
///
/// Handles the form submission by calling direct database functions.
#[component]
fn SubmitHandler(
    mode: FormMode,
    title: String,
    description: String,
    status: String,
    bead_type: String,
    priority: i16,
    on_complete: EventHandler<Result<String, String>>,
) -> Element {
    let mut is_done = use_signal(|| false);
    let result = use_signal(|| {
        if title.is_empty() {
            return Err("Title is required".to_string());
        }

        let bead_status = match BeadStatus::from_str(&status) {
            Ok(s) => s,
            Err(e) => return Err(format!("Invalid status: {}", e)),
        };

        let new_bead_type = match BeadType::from_str(&bead_type) {
            Ok(t) => t,
            Err(e) => return Err(format!("Invalid type: {}", e)),
        };

        let new_bead = NewBead {
            title,
            description: if description.is_empty() {
                None
            } else {
                Some(description)
            },
            status: bead_status,
            priority: BeadPriority(priority),
            bead_type: new_bead_type,
            created_by: None,
        };

        let result = match mode {
            FormMode::Create => try_create_bead(new_bead),
            FormMode::Edit(id) => {
                match BeadId::from_str(&id) {
                    Ok(bead_id) => try_update_bead(bead_id, new_bead),
                    Err(e) => Err(format!("Invalid bead ID: {}", e)),
                }
            }
        };

        result.map(|bead| bead.id.to_string())
    });

    rsx! {
        {match &*result.read() {
            Err(e) => {
                if !*is_done.read() {
                    is_done.set(true);
                    on_complete.call(Err(e.clone()));
                }
                rsx! {
                    div { class: "error",
                        "Error: {e}"
                    }
                }
            }
            Ok(bead_id) => {
                if !*is_done.read() {
                    is_done.set(true);
                    on_complete.call(Ok(bead_id.clone()));
                }
                rsx! {
                    div { class: "success",
                        "Bead saved successfully! "
                        a { href: format!("/beads/{}", bead_id), "View bead" }
                    }
                }
            }
        }}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_form_mode_create() {
        let mode = FormMode::Create;
        assert!(!matches!(mode, FormMode::Edit(_)));
    }

    #[test]
    fn test_form_mode_edit() {
        let mode = FormMode::Edit("test-id".to_string());
        assert!(matches!(mode, FormMode::Edit(_)));
        if let FormMode::Edit(id) = mode {
            assert_eq!(id, "test-id");
        }
    }
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

/// Helper function to create a bead in the database
///
/// This function attempts to initialize the database and create a new bead.
fn try_create_bead(bead: NewBead) -> Result<Bead, String> {
    let db = crate::db::DesktopDb::new()
        .map_err(|e| format!("Failed to initialize database: {}", e))?;

    db.create_bead(bead)
        .map_err(|e| format!("Failed to create bead: {}", e))
}

/// Helper function to update a bead in the database
///
/// This function attempts to initialize the database and update an existing bead.
fn try_update_bead(id: BeadId, bead: NewBead) -> Result<Bead, String> {
    let db = crate::db::DesktopDb::new()
        .map_err(|e| format!("Failed to initialize database: {}", e))?;

    db.update_bead(id, bead)
        .map_err(|e| format!("Failed to update bead: {}", e))
}
