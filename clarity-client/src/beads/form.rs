//! Bead Form Component
//!
//! Form component for creating and editing beads using direct database access.
//! Features real-time validation with debounced field-level error reporting.
//! Supports keyboard shortcuts for quick actions.
//!
//! Functional implementation with zero unwrap, pure functions, and proper error handling.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(clippy::redundant_clone)]
#![allow(clippy::clone_on_copy)]
#![allow(dead_code)]
#![allow(unused_imports)]

use clarity_core::db::models::{Bead, BeadId, BeadPriority, BeadStatus, BeadType, NewBead};
use dioxus::prelude::*;

use crate::components::{Loading, LoadingSize};
use crate::hooks::use_keyboard_with_handler;
use crate::hooks::use_validation::{use_form_validation, FieldErrorState, ValidationState};
use crate::shortcuts::Action;
use crate::validation::BeadFormData;

/// Bead form page component
///
/// Handles both creating new beads and editing existing beads.
/// Uses direct database access for create/update operations.
#[component]
pub fn BeadFormPage(id: Option<String>) -> Element {
  #[allow(clippy::option_if_let_else)]
  let content = match id {
    Some(bead_id) => rsx! {
        BeadForm { mode: FormMode::Edit(bead_id) }
    },
    None => rsx! {
        BeadForm { mode: FormMode::Create }
    },
  };

  rsx! {
      div { class: "bead-form-page",
          {content}
      }
  }
}

/// Form mode enum
///
/// Distinguishes between creating a new bead and editing an existing one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FormMode {
  Create,
  Edit(String),
}

/// Bead form component with functional improvements
///
/// Pure, functional implementation with proper error handling and no unwrap calls.
#[component]
fn BeadForm(mode: FormMode) -> Element {
  // Form signals - need mut for Dioxus 0.7
  let mut title = use_signal(String::new);
  let mut description = use_signal(String::new);
  let mut status = use_signal(|| "open".to_string());
  let mut bead_type = use_signal(|| "feature".to_string());
  let mut priority = use_signal(|| 2_i16);

  // Async states
  let is_loading = use_signal(|| false);
  let load_error = use_signal(|| Option::<String>::None);
  let mut is_submitting = use_signal(|| false);
  let submit_trigger = use_signal(|| false);

  // Validation state
  let (validation_state, field_errors, touch_field, validate, is_valid) = use_form_validation();

  // Parse bead ID for edit mode without unwrap
  let edit_id = match &mode {
    FormMode::Edit(id) => Some(id.clone()),
    FormMode::Create => None,
  };

  // Load bead data if editing - functional approach
  let existing_bead = use_signal(|| Option::<Bead>::None);

  {
    let mode_clone = mode.clone();
    let mut existing_bead_clone = existing_bead.clone();
    let mut is_loading_clone = is_loading.clone();
    let mut load_error_clone = load_error.clone();
    let edit_id_clone = edit_id.clone();

    use_effect(move || {
      if matches!(mode_clone, FormMode::Edit(_))
        && existing_bead_clone.read().is_none()
        && !*is_loading_clone.read()
      {
        if let Some(ref id_str) = edit_id_clone {
          if let Ok(bead_id) = BeadId::from_str(id_str) {
            spawn(async move {
              match load_bead_functional(bead_id).await {
                Ok(bead) => {
                  existing_bead_clone.set(Some(bead));
                  is_loading_clone.set(false);
                }
                Err(e) => {
                  load_error_clone.set(Some(e));
                  is_loading_clone.set(false);
                }
              }
            });
          }
        }
      }
    });
  }

  // Populate form when bead loads - functional mapping
  if let Some(ref bead) = &*existing_bead.read() {
    if title.read().is_empty() {
      // Use functional mapping to populate fields
      title.set(bead.title.clone());
      description.set(bead.description.clone().unwrap_or_default());
      status.set(bead.status.as_str().to_string());
      bead_type.set(bead.bead_type.as_str().to_string());
      priority.set(bead.priority.sort_value());
    }
  }

  // Handle form submission - functional railway pattern
  let onsubmit = move |evt: Event<FormData>| {
    evt.prevent_default();

    // Create form data - functional construction
    let form_data = BeadFormData {
      title: title.read().clone(),
      description: description.read().clone(),
      status: status.read().clone(),
      priority: *priority.read(),
      bead_type: bead_type.read().clone(),
    };

    // Mark fields as touched - functional side effects
    touch_field("title".to_string());
    touch_field("description".to_string());
    touch_field("status".to_string());
    touch_field("priority".to_string());
    touch_field("type".to_string());

    // Validate before submission - functional validation
    if form_data.validate().is_valid() {
      is_submitting.set(true);
    }
  };

  // Get navigator for programmatic navigation - functional pattern
  let navigator = crate::hooks::use_state::use_navigator();
  let navigator_for_submit = navigator.clone();

  // Handle keyboard-triggered submission - functional pattern
  {
    let mut submit_trigger_clone = submit_trigger.clone();
    let mut is_submitting_clone = is_submitting.clone();
    let _validate_clone = validate.clone();
    let _navigator_for_keyboard = navigator.clone();
    let _mode_for_navigation = mode.clone();

    use_effect(move || {
      if *submit_trigger_clone.read() {
        // Touch all fields
        touch_field("title".to_string());
        touch_field("description".to_string());
        touch_field("status".to_string());
        touch_field("priority".to_string());
        touch_field("type".to_string());

        // Create form data for validation
        let form_data = BeadFormData {
          title: title.read().clone(),
          description: description.read().clone(),
          status: status.read().clone(),
          priority: *priority.read(),
          bead_type: bead_type.read().clone(),
        };

        // Validate and submit
        if form_data.validate().is_valid() {
          is_submitting_clone.set(true);
        }

        // Reset trigger - functional state update
        submit_trigger_clone.set(false);
      }
    });
  }

  // Keyboard shortcuts handler - functional event handling
  let mode_for_keyboard = mode.clone();
  let submit_trigger_for_keyboard = submit_trigger.clone();
  let navigator_for_keyboard = navigator.clone();

  let _keyboard_handler = use_keyboard_with_handler(move |action: Action| {
    match action {
      Action::SaveForm => {
        // Trigger submission without mutating state directly
        let mut trigger = submit_trigger_for_keyboard;
        trigger.set(true);
      }
      Action::Cancel => {
        // Navigate based on mode - functional pattern
        let nav = navigator_for_keyboard.clone();
        match mode_for_keyboard {
          FormMode::Edit(ref id) => {
            nav(crate::app::Route::BeadDetail { id: id.clone() });
          }
          FormMode::Create => {
            nav(crate::app::Route::BeadsList);
          }
        }
      }
      _ => {}
    }
  });

  // Functional helpers for field error state
  let get_field_error = move |field: &str| -> FieldErrorState {
    field_errors.read().get(field).cloned().unwrap_or_default()
  };

  let get_field_classes = move |field: &str| -> String {
    let error_state = get_field_error(field);
    let base_class = "form-group";

    match (error_state.touched, error_state.has_errors()) {
      (true, true) => format!("{base_class} has-error"),
      (true, false) => format!("{base_class} is-valid"),
      _ => base_class.to_string(),
    }
  };

  let get_input_classes = move |field: &str| -> String {
    let error_state = get_field_error(field);
    let base_class = "form-control";

    if error_state.touched && error_state.has_errors() {
      format!("{base_class} error")
    } else if error_state.touched {
      format!("{base_class} valid")
    } else {
      base_class.to_string()
    }
  };

  // Form text - functional mapping
  let title_text = matches!(mode, FormMode::Edit(_))
    .then(|| "Edit Bead")
    .unwrap_or("Create New Bead");
  let submit_text = matches!(mode, FormMode::Edit(_))
    .then(|| "Update Bead")
    .unwrap_or("Create Bead");

  rsx! {
      div { class: "bead-form",
          h1 { "{title_text}" }

          // Loading state - functional conditional rendering
          {if *is_loading.read() {
              rsx! {
                  Loading {
                      size: LoadingSize::Medium,
                      message: Some("Loading bead...".to_string())
                  }
              }
          } else {
              rsx! {}
          }}

          // Error state - functional mapping
          {load_error.read().as_ref().map(|error| rsx! {
                  div { class: "error-message",
                      p { "Failed to load bead: {error}" }
                      crate::app::NavLink {
                          to: crate::app::Route::BeadsList,
                          class: "btn btn-secondary",
                          "Back to List"
                      }
                  }
              })}

          // Validation errors - functional filtering
          {match &*validation_state.read() {
              ValidationState::Invalid if !field_errors.read().is_empty() => {
                  let all_errors: Vec<_> = field_errors
                      .read()
                      .values()
                      .flat_map(|state| state.errors.clone())
                      .collect();

                  rsx! {
                      div {
                          class: "alert alert-error",
                          role: "alert",
                          "aria-live": "polite",
                          h3 { "Please fix the following errors:" }
                          ul {
                              for error in all_errors.iter() {
                                  li { "{error}" }
                              }
                          }
                      }
                  }
              }
              _ => rsx! {}
          }}

          // Form with proper accessibility
          form {
              onsubmit: onsubmit,

              // Title field with validation
              div { class: get_field_classes("title"),
                  label { r#for: "title", "Title *" }
                  input {
                      id: "title",
                      class: get_input_classes("title"),
                      r#type: "text",
                      required: true,
                      value: "{title}",
                      "aria-invalid": format!("{}", get_field_error("title").has_errors()),
                      "aria-describedby": "title-error",
                      oninput: move |evt: Event<FormData>| {
                          title.set(evt.value());
                          // Validate on input - functional composition
                          let form_data = BeadFormData {
                              title: evt.value(),
                              description: description.read().clone(),
                              status: status.read().clone(),
                              priority: *priority.read(),
                              bead_type: bead_type.read().clone(),
                          };
                          validate(form_data);
                      },
                      onblur: move |_| {
                          touch_field("title".to_string());
                          // Revalidate on blur
                          let form_data = BeadFormData {
                              title: title.read().clone(),
                              description: description.read().clone(),
                              status: status.read().clone(),
                              priority: *priority.read(),
                              bead_type: bead_type.read().clone(),
                          };
                          validate(form_data);
                      }
                  }
                  {get_field_error("title").has_errors().then(|| {
                      rsx! {
                          span {
                              id: "title-error",
                              class: "field-error",
                              "aria-live": "polite",
                              {get_field_error("title").first_error().unwrap_or("")}
                          }
                      }
                  })}
              }

              // Description field
              div { class: get_field_classes("description"),
                  label { r#for: "description", "Description" }
                  textarea {
                      id: "description",
                      class: get_input_classes("description"),
                      rows: "5",
                      value: "{description}",
                      "aria-invalid": format!("{}", get_field_error("description").has_errors()),
                      "aria-describedby": "description-error",
                      oninput: move |evt: Event<FormData>| {
                          description.set(evt.value());
                          let form_data = BeadFormData {
                              title: title.read().clone(),
                              description: evt.value(),
                              status: status.read().clone(),
                              priority: *priority.read(),
                              bead_type: bead_type.read().clone(),
                          };
                          validate(form_data);
                      },
                      onblur: move |_| {
                          touch_field("description".to_string());
                          let form_data = BeadFormData {
                              title: title.read().clone(),
                              description: description.read().clone(),
                              status: status.read().clone(),
                              priority: *priority.read(),
                              bead_type: bead_type.read().clone(),
                          };
                          validate(form_data);
                      }
                  }
                  {get_field_error("description").has_errors().then(|| {
                      rsx! {
                          span {
                              id: "description-error",
                              class: "field-error",
                              "aria-live": "polite",
                              {get_field_error("description").first_error().unwrap_or("")}
                          }
                      }
                  })}
              }

              // Row for status, type, and priority
              div { class: "form-row",
                  // Status field
                  div { class: get_field_classes("status"),
                      label { r#for: "status", "Status" }
                      select {
                          id: "status",
                          class: get_input_classes("status"),
                          value: "{status}",
                          "aria-invalid": format!("{}", get_field_error("status").has_errors()),
                          "aria-describedby": "status-error",
                          onchange: move |evt: Event<FormData>| {
                              status.set(evt.value());
                              touch_field("status".to_string());
                              let form_data = BeadFormData {
                                  title: title.read().clone(),
                                  description: description.read().clone(),
                                  status: evt.value(),
                                  priority: *priority.read(),
                                  bead_type: bead_type.read().clone(),
                              };
                              validate(form_data);
                          },
                          option { value: "open", "Open" },
                          option { value: "in_progress", "In Progress" },
                          option { value: "blocked", "Blocked" },
                          option { value: "deferred", "Deferred" },
                          option { value: "closed", "Closed" },
                      }
                      {get_field_error("status").has_errors().then(|| {
                          rsx! {
                              span {
                                  id: "status-error",
                                  class: "field-error",
                                  "aria-live": "polite",
                                  {get_field_error("status").first_error().unwrap_or("")}
                              }
                          }
                      })}
                  }

                  // Type field
                  div { class: get_field_classes("type"),
                      label { r#for: "bead_type", "Type" }
                      select {
                          id: "bead_type",
                          class: get_input_classes("type"),
                          value: "{bead_type}",
                          "aria-invalid": format!("{}", get_field_error("type").has_errors()),
                          "aria-describedby": "type-error",
                          onchange: move |evt: Event<FormData>| {
                              bead_type.set(evt.value());
                              touch_field("type".to_string());
                              let form_data = BeadFormData {
                                  title: title.read().clone(),
                                  description: description.read().clone(),
                                  status: status.read().clone(),
                                  priority: *priority.read(),
                                  bead_type: evt.value(),
                              };
                              validate(form_data);
                          },
                          option { value: "feature", "Feature" },
                          option { value: "bugfix", "Bug Fix" },
                          option { value: "refactor", "Refactor" },
                          option { value: "test", "Test" },
                          option { value: "docs", "Documentation" },
                      }
                      {get_field_error("type").has_errors().then(|| {
                          rsx! {
                              span {
                                  id: "type-error",
                                  class: "field-error",
                                  "aria-live": "polite",
                                  {get_field_error("type").first_error().unwrap_or("")}
                              }
                          }
                      })}
                  }

                  // Priority field
                  div { class: get_field_classes("priority"),
                      label { r#for: "priority", "Priority" }
                      select {
                          id: "priority",
                          class: get_input_classes("priority"),
                          value: "{priority}",
                          "aria-invalid": format!("{}", get_field_error("priority").has_errors()),
                          "aria-describedby": "priority-error",
                          onchange: move |evt: Event<FormData>| {
                              if let Ok(p) = evt.value().parse::<i16>() {
                                  priority.set(p);
                                  touch_field("priority".to_string());
                                  let form_data = BeadFormData {
                                      title: title.read().clone(),
                                      description: description.read().clone(),
                                      status: status.read().clone(),
                                      priority: p,
                                      bead_type: bead_type.read().clone(),
                                  };
                                  validate(form_data);
                              }
                          },
                          option { value: "1", "High" },
                          option { value: "2", "Medium" },
                          option { value: "3", "Low" },
                      }
                      {get_field_error("priority").has_errors().then(|| {
                          rsx! {
                              span {
                                  id: "priority-error",
                                  class: "field-error",
                                  "aria-live": "polite",
                                  {get_field_error("priority").first_error().unwrap_or("")}
                              }
                          }
                      })}
                  }
              }

              // Form actions
              div { class: "form-actions",
                  div { class: "btn-with-shortcut",
                      button {
                          r#type: "submit",
                          class: "btn btn-primary",
                          disabled: !is_valid() || *is_submitting.read(),
                          "{submit_text}"
                      }
                      span { class: "shortcut-hint-inline", "Ctrl+S" }
                  }
                  {
                      match &mode {
                          FormMode::Edit(id) => rsx! {
                              div { class: "btn-with-shortcut",
                                  crate::app::NavLink {
                                      to: crate::app::Route::BeadDetail { id: id.clone() },
                                      class: "btn btn-secondary",
                                      "Cancel"
                                  }
                                  span { class: "shortcut-hint-inline", "Esc" }
                              }
                          },
                          FormMode::Create => rsx! {
                              div { class: "btn-with-shortcut",
                                  crate::app::NavLink {
                                      to: crate::app::Route::BeadsList,
                                      class: "btn btn-secondary",
                                      "Cancel"
                                  }
                                  span { class: "shortcut-hint-inline", "Esc" }
                              }
                          }
                      }
                  }
              }
          }

          // Submit handler with functional error handling and navigation
          {if *is_submitting.read() {
              let mode_clone = mode.clone();
              let title_val = title.read().clone();
              let description_val = description.read().clone();
              let status_val = status.read().clone();
              let bead_type_val = bead_type.read().clone();
              let priority_val = *priority.read();
              let navigator_clone = navigator_for_submit.clone();
              let mode_for_navigation = mode.clone();

              rsx! {
                  div { class: "form-loading",
                      Loading {
                          size: LoadingSize::Small,
                          message: Some("Saving...".to_string())
                      }
                  }

                  SubmitHandler {
                      mode: mode_clone,
                      title: title_val,
                      description: description_val,
                      status: status_val,
                      bead_type: bead_type_val,
                      priority: priority_val,
                      on_complete: move |result: Result<String, String>| {
                          is_submitting.set(false);
                          match result {
                              Ok(bead_id) => {
                                  // Programmatic navigation after successful save
                                  let nav = navigator_clone.clone();
                                  match mode_for_navigation {
                                      FormMode::Edit(_) => {
                                          // Stay on edit page with updated bead
                                          nav(crate::app::Route::BeadDetail { id: bead_id.clone() });
                                      }
                                      FormMode::Create => {
                                          // Navigate to detail page of newly created bead
                                          nav(crate::app::Route::BeadDetail { id: bead_id.clone() });
                                      }
                                  }
                              }
                              Err(e) => {
                                  // Error state could be added here
                                  eprintln!("Submit error: {e}");
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

/// Submit handler component properties
#[derive(Clone, Props)]
pub struct SubmitHandlerProps {
  /// Form mode (create or edit)
  pub mode: FormMode,
  /// The bead title
  pub title: String,
  /// The bead description
  pub description: String,
  /// The bead status
  pub status: String,
  /// The bead type
  pub bead_type: String,
  /// The bead priority
  pub priority: i16,
  /// Callback when submission is complete
  pub on_complete: EventHandler<Result<String, String>>,
}

// Manual PartialEq implementation since EventHandler doesn't implement PartialEq
impl PartialEq for SubmitHandlerProps {
  fn eq(&self, other: &Self) -> bool {
    self.mode == other.mode
      && self.title == other.title
      && self.description == other.description
      && self.status == other.status
      && self.bead_type == other.bead_type
      && self.priority == other.priority
  }
}

impl Eq for SubmitHandlerProps {}

/// Submit handler component with functional improvements
///
/// Handles the form submission with proper error handling and no unwrap calls.
#[component]
fn SubmitHandler(props: SubmitHandlerProps) -> Element {
  let mut is_done = use_signal(|| false);
  let mut result = use_signal(|| Option::<Result<String, String>>::None);
  let mut has_started = use_signal(|| false);

  let on_complete = props.on_complete;

  // Start submission on mount
  use_effect(move || {
    if *has_started.read() {
      return;
    }
    has_started.set(true);

    // Validate input - functional validation
    if props.title.is_empty() {
      let error = "Title is required".to_string();
      is_done.set(true);
      result.set(Some(Err(error.clone())));
      on_complete.call(Err(error));
      return;
    }

    // Parse status with error handling - functional Result chaining
    let bead_status = match props.status.as_str() {
      "open" => Ok(BeadStatus::Open),
      "in_progress" => Ok(BeadStatus::InProgress),
      "blocked" => Ok(BeadStatus::Blocked),
      "deferred" => Ok(BeadStatus::Deferred),
      "closed" => Ok(BeadStatus::Closed),
      _ => Err(format!("Invalid status: {}", props.status)),
    };

    let new_bead_type = match props.bead_type.as_str() {
      "feature" => Ok(BeadType::Feature),
      "bugfix" => Ok(BeadType::Bugfix),
      "refactor" => Ok(BeadType::Refactor),
      "test" => Ok(BeadType::Test),
      "docs" => Ok(BeadType::Docs),
      _ => Err(format!("Invalid type: {}", props.bead_type)),
    };

    // Combine results - functional railway pattern
    let validation_result = (bead_status, new_bead_type);
    match validation_result {
      (Ok(status), Ok(bead_type)) => {
        let new_bead = NewBead {
          title: props.title.clone(),
          description: if props.description.is_empty() {
            None
          } else {
            Some(props.description.clone())
          },
          status,
          priority: BeadPriority::from_value(props.priority).unwrap_or_else(|_| BeadPriority::Medium),
          bead_type,
          created_by: None,
        };

        // Perform async operation
        let mode = props.mode.clone();
        let mut result_signal = result;
        let mut is_done_signal = is_done;

        spawn(async move {
          let save_result = match mode {
            FormMode::Create => create_bead_functional(new_bead).await,
            FormMode::Edit(id) => match BeadId::from_str(&id) {
              Ok(bead_id) => update_bead_functional(bead_id, new_bead).await,
              Err(e) => Err(format!("Invalid bead ID: {e}")),
            },
          };

          match save_result {
            Ok(bead) => {
              let bead_id = bead.id.to_string();
              is_done_signal.set(true);
              result_signal.set(Some(Ok(bead_id.clone())));
              on_complete.call(Ok(bead_id));
            }
            Err(e) => {
              is_done_signal.set(true);
              result_signal.set(Some(Err(e.clone())));
              on_complete.call(Err(e));
            }
          }
        });
      }
      (Err(status_error), _) => {
        is_done.set(true);
        result.set(Some(Err(status_error.clone())));
        on_complete.call(Err(status_error));
      }
      (_, Err(type_error)) => {
        is_done.set(true);
        result.set(Some(Err(type_error.clone())));
        on_complete.call(Err(type_error));
      }
    }
  });

  // Display result - functional conditional rendering
  rsx! {
      {match &*result.read() {
          None => rsx! {
              div { class: "saving", "Saving..." }
          },
          Some(Err(e)) => rsx! {
              div { class: "error",
                  "Error: {e}"
              }
          },
          Some(Ok(bead_id)) => rsx! {
              div { class: "success",
                  "Bead saved successfully! "
                  crate::app::NavLink {
                      to: crate::app::Route::BeadDetail { id: bead_id.clone() },
                      "View bead"
                  }
              }
          },
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

  #[test]
  fn test_equality() {
    let mode1 = FormMode::Create;
    let mode2 = FormMode::Create;
    assert_eq!(mode1, mode2);

    let mode3 = FormMode::Edit("id1".to_string());
    let mode4 = FormMode::Edit("id1".to_string());
    assert_eq!(mode3, mode4);

    let mode5 = FormMode::Edit("id2".to_string());
    assert_ne!(mode3, mode5);
  }
}

/// Async helper function to load a bead from the database
///
/// This function attempts to initialize the database and load a bead by ID asynchronously.
/// Functional implementation with proper error handling.
async fn try_load_bead_async(id: BeadId) -> Result<Bead, String> {
  load_bead_functional(id).await
}

/// Async helper function to create a bead in the database
///
/// This function attempts to initialize the database and create a new bead.
/// Functional implementation with proper error handling.
async fn try_create_bead_async(bead: NewBead) -> Result<Bead, String> {
  create_bead_functional(bead).await
}

/// Async helper function to update a bead in the database
///
/// This function attempts to initialize the database and update an existing bead.
/// Functional implementation with proper error handling.
async fn try_update_bead_async(id: BeadId, bead: NewBead) -> Result<Bead, String> {
  update_bead_functional(id, bead).await
}

/// Functional bead loading with proper error handling
async fn load_bead_functional(id: BeadId) -> Result<Bead, String> {
  crate::db::DesktopDb::new_async()
    .await
    .map_err(|e| format!("Failed to initialize database: {e}"))?
    .get_bead(id)
    .await
    .map_err(|e| format!("Failed to load bead: {e}"))
}

/// Functional bead creation with proper error handling
async fn create_bead_functional(bead: NewBead) -> Result<Bead, String> {
  crate::db::DesktopDb::new_async()
    .await
    .map_err(|e| format!("Failed to initialize database: {e}"))?
    .create_bead(bead)
    .await
    .map_err(|e| format!("Failed to create bead: {e}"))
}

/// Functional bead update with proper error handling
async fn update_bead_functional(id: BeadId, bead: NewBead) -> Result<Bead, String> {
  crate::db::DesktopDb::new_async()
    .await
    .map_err(|e| format!("Failed to initialize database: {e}"))?
    .update_bead(id, bead)
    .await
    .map_err(|e| format!("Failed to update bead: {e}"))
}
