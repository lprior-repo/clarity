//! Bead Form Component
//!
//! Form component for creating and editing beads using direct database access.
//! Features real-time validation with debounced field-level error reporting.
//! Supports keyboard shortcuts for quick actions.

// Dioxus rsx! macro internally uses unwrap, so we allow the disallowed_methods lint.
#![allow(clippy::disallowed_methods)]

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

/// Bead form component
///
/// Renders a form for bead creation or editing with real-time validation.
#[component]
fn BeadForm(mode: FormMode) -> Element {
  // Form state
  let mut title = use_signal(String::new);
  let mut description = use_signal(String::new);
  let mut status = use_signal(|| String::from("open"));
  let mut bead_type = use_signal(|| String::from("feature"));
  let mut priority = use_signal(|| 2_i16);
  let is_loading = use_signal(|| false);
  let load_error = use_signal(|| Option::<String>::None);

  // Validation state with debounced field-level validation
  let (validation_state, field_errors, touch_field, validate, is_valid) = use_form_validation();

  // Clone mode for use in closures
  let mode_clone = mode.clone();
  let is_edit = matches!(mode_clone, FormMode::Edit(_));
  let edit_id = match &mode_clone {
    FormMode::Edit(id) => Some(id.clone()),
    FormMode::Create => None,
  };

  // If editing, load the existing bead data
  let existing_bead = use_signal(|| Option::<Bead>::None);

  // Load bead data if editing
  {
    use_effect(move || {
      if is_edit && existing_bead.read().is_none() && !*is_loading.read() {
        if let Some(ref id_str) = edit_id {
          if let Ok(bead_id) = BeadId::from_str(id_str) {
            let mut existing_bead = existing_bead;
            let mut is_loading = is_loading;
            let mut load_error = load_error;

            is_loading.set(true);

            dioxus::prelude::spawn(async move {
              match try_load_bead_async(bead_id).await {
                Ok(bead) => {
                  existing_bead.set(Some(bead));
                  is_loading.set(false);
                }
                Err(e) => {
                  load_error.set(Some(e));
                  is_loading.set(false);
                }
              }
            });
          }
        }
      }
    });
  }

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
  let submit_trigger = use_signal(|| false);

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

  // Create form data for validation
  let form_data = BeadFormData {
    title: title.read().clone(),
    description: description.read().clone(),
    status: status.read().clone(),
    priority: *priority.read(),
    bead_type: bead_type.read().clone(),
  };

  // Set up keyboard shortcuts handler
  let navigator = crate::hooks::use_state::use_navigator();
  let mode_for_keyboard = mode.clone();
  let submit_trigger_for_keyboard = submit_trigger;
  let _keyboard_handler = use_keyboard_with_handler(move |action: Action| {
    match action {
      Action::SaveForm => {
        // Trigger form submission
        let mut trigger = submit_trigger_for_keyboard;
        trigger.set(true);
      }
      Action::Cancel => {
        // Navigate back to list or detail view
        let nav = navigator.clone();
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

  let onsubmit = move |evt: Event<FormData>| {
    evt.prevent_default();

    // Validate before submission
    let form_data = BeadFormData {
      title: title.read().clone(),
      description: description.read().clone(),
      status: status.read().clone(),
      priority: *priority.read(),
      bead_type: bead_type.read().clone(),
    };

    // Mark all fields as touched on submission attempt
    touch_field("title".to_string());
    touch_field("description".to_string());
    touch_field("status".to_string());
    touch_field("priority".to_string());
    touch_field("type".to_string());

    // Only submit if valid
    if form_data.validate().is_valid() {
      is_submitting.set(true);
    }
  };

  // Handle keyboard-triggered submission
  {
    let mut submit_trigger = submit_trigger;
    let mut is_submitting = is_submitting;
    let form_data_clone = form_data;
    let _is_valid = is_valid();

    use_effect(move || {
      if *submit_trigger.read() {
        // Mark all fields as touched
        touch_field("title".to_string());
        touch_field("description".to_string());
        touch_field("status".to_string());
        touch_field("priority".to_string());
        touch_field("type".to_string());

        // Only submit if valid
        if form_data_clone.validate().is_valid() {
          is_submitting.set(true);
        }

        // Reset trigger
        submit_trigger.set(false);
      }
    });
  }

  // Helper to get field error state
  let get_field_error = move |field: &str| -> FieldErrorState {
    field_errors.read().get(field).cloned().unwrap_or_default()
  };

  // Helper to get field CSS classes
  let get_field_classes = move |field: &str| -> String {
    let error_state = get_field_error(field);
    let base_class = "form-group";

    match (error_state.touched, error_state.has_errors()) {
      (true, true) => format!("{base_class} has-error"),
      (true, false) => format!("{base_class} is-valid"),
      _ => base_class.to_string(),
    }
  };

  // Helper to get input CSS classes
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

  rsx! {
      div { class: "bead-form",
          h1 { "{title_text}" }

          // Show loading state while fetching bead for editing
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

          // Show error if loading failed
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

          // Global validation errors (aria-live for screen readers)
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

          form {
              onsubmit: onsubmit,

              // Title field with validation
              div { class: "{get_field_classes(r#\"title\"#)}",
                  label { r#for: "title", "Title *" }
                  input {
                      id: "title",
                      class: "{get_input_classes(r#\"title\"#)}",
                      r#type: "text",
                      required: true,
                      value: "{title}",
                      "aria-invalid": "{get_field_error(r#\"title\"#).has_errors()}",
                      "aria-describedby": "title-error",
                      oninput: move |evt: Event<FormData>| {
                          title.set(evt.value());
                          // Trigger debounced validation
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
                          // Re-validate on blur
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

              // Description field with validation
              div { class: "{get_field_classes(r#\"description\"#)}",
                  label { r#for: "description", "Description" }
                  textarea {
                      id: "description",
                      class: "{get_input_classes(r#\"description\"#)}",
                      rows: "5",
                      value: "{description}",
                      "aria-invalid": "{get_field_error(r#\"description\"#).has_errors()}",
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

              div { class: "form-row",
                  // Status field with validation
                  div { class: "{get_field_classes(r#\"status\"#)}",
                      label { r#for: "status", "Status" }
                      select {
                          id: "status",
                          class: "{get_input_classes(r#\"status\"#)}",
                          value: "{status}",
                          "aria-invalid": "{get_field_error(r#\"status\"#).has_errors()}",
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

                  // Type field with validation
                  div { class: "{get_field_classes(r#\"type\"#)}",
                      label { r#for: "bead_type", "Type" }
                      select {
                          id: "bead_type",
                          class: "{get_input_classes(r#\"type\"#)}",
                          value: "{bead_type}",
                          "aria-invalid": "{get_field_error(r#\"type\"#).has_errors()}",
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

                  // Priority field with validation
                  div { class: "{get_field_classes(r#\"priority\"#)}",
                      label { r#for: "priority", "Priority" }
                      select {
                          id: "priority",
                          class: "{get_input_classes(r#\"priority\"#)}",
                          value: "{priority}",
                          "aria-invalid": "{get_field_error(r#\"priority\"#).has_errors()}",
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

          {if *is_submitting.read() {
              let mode_clone = mode;
              let title_val = title.read().clone();
              let description_val = description.read().clone();
              let status_val = status.read().clone();
              let bead_type_val = bead_type.read().clone();
              let priority_val = *priority.read();

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
                      on_complete: move |result| {
                          is_submitting.set(false);
                          match result {
                              Ok(_bead_id) => {
                                  // Success - navigation will happen via link
                              }
                              Err(e) => {
                                  // Show error - could add error state here
                                  let _ = e;
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
    // Always consider equal since we can't compare Callbacks
  }
}

impl Eq for SubmitHandlerProps {}

/// Submit handler component
///
/// Handles the form submission by calling direct database functions.
#[component]
fn SubmitHandler(props: SubmitHandlerProps) -> Element {
  let mut is_done = use_signal(|| false);
  let mut has_started = use_signal(|| false);
  let mode = props.mode;
  let title = props.title;
  let description = props.description;
  let status = props.status;
  let bead_type = props.bead_type;
  let priority = props.priority;
  let on_complete = props.on_complete;
  let mut result = use_signal(|| Option::<Result<String, String>>::None);

  // Start submission on mount
  use_effect(move || {
    if *has_started.read() {
      return;
    }
    has_started.set(true);

    // Validate title
    if title.is_empty() {
      is_done.set(true);
      result.set(Some(Err("Title is required".to_string())));
      on_complete.call(Err("Title is required".to_string()));
      return;
    }

    let bead_status = match status.as_str() {
      "open" => BeadStatus::Open,
      "in_progress" => BeadStatus::InProgress,
      "blocked" => BeadStatus::Blocked,
      "deferred" => BeadStatus::Deferred,
      "closed" => BeadStatus::Closed,
      _ => {
        let error = format!("Invalid status: {status}");
        is_done.set(true);
        result.set(Some(Err(error.clone())));
        on_complete.call(Err(error));
        return;
      }
    };

    let new_bead_type = match bead_type.as_str() {
      "feature" => BeadType::Feature,
      "bugfix" => BeadType::Bugfix,
      "refactor" => BeadType::Refactor,
      "test" => BeadType::Test,
      "docs" => BeadType::Docs,
      _ => {
        let error = format!("Invalid type: {bead_type}");
        is_done.set(true);
        result.set(Some(Err(error.clone())));
        on_complete.call(Err(error));
        return;
      }
    };

    let new_bead = NewBead {
      title: title.clone(),
      description: if description.is_empty() {
        None
      } else {
        Some(description.clone())
      },
      status: bead_status,
      priority: BeadPriority(priority),
      bead_type: new_bead_type,
      created_by: None,
    };

    let mode = mode.clone();
    let mut result_signal = result;
    let mut is_done_signal = is_done;
    let on_complete = on_complete;

    spawn(async move {
      eprintln!("[SubmitHandler] Saving bead: {}", new_bead.title);
      let save_result = match mode {
        FormMode::Create => try_create_bead_async(new_bead).await,
        FormMode::Edit(id) => match BeadId::from_str(&id) {
          Ok(bead_id) => try_update_bead_async(bead_id, new_bead).await,
          Err(e) => Err(format!("Invalid bead ID: {e}")),
        },
      };

      match save_result {
        Ok(bead) => {
          eprintln!("[SubmitHandler] Successfully saved bead: {}", bead.id);
          let bead_id = bead.id.to_string();
          is_done_signal.set(true);
          result_signal.set(Some(Ok(bead_id.clone())));
          on_complete.call(Ok(bead_id));
        }
        Err(e) => {
          eprintln!("[SubmitHandler] Error saving bead: {e:?}");
          is_done_signal.set(true);
          result_signal.set(Some(Err(e.clone())));
          on_complete.call(Err(e));
        }
      }
    });
  });

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
}

/// Async helper function to load a bead from the database
///
/// This function attempts to initialize the database and load a bead by ID asynchronously.
async fn try_load_bead_async(id: BeadId) -> Result<Bead, String> {
  let db = crate::db::DesktopDb::new_async()
    .await
    .map_err(|e| format!("Failed to initialize database: {e}"))?;

  db.get_bead(id)
    .await
    .map_err(|e| format!("Failed to load bead: {e}"))
}

/// Async helper function to create a bead in the database
///
/// This function attempts to initialize the database and create a new bead.
async fn try_create_bead_async(bead: NewBead) -> Result<Bead, String> {
  let db = crate::db::DesktopDb::new_async()
    .await
    .map_err(|e| format!("Failed to initialize database: {e}"))?;

  db.create_bead(bead)
    .await
    .map_err(|e| format!("Failed to create bead: {e}"))
}

/// Async helper function to update a bead in the database
///
/// This function attempts to initialize the database and update an existing bead.
async fn try_update_bead_async(id: BeadId, bead: NewBead) -> Result<Bead, String> {
  let db = crate::db::DesktopDb::new_async()
    .await
    .map_err(|e| format!("Failed to initialize database: {e}"))?;

  db.update_bead(id, bead)
    .await
    .map_err(|e| format!("Failed to update bead: {e}"))
}
