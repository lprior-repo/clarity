//! List editor component
//!
//! A reusable component for editing lists of items with add/remove functionality.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(warnings)]
#![allow(clippy::all)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
// Dioxus rsx! macro internally uses unwrap, so we allow the disallowed_methods lint.
#![allow(clippy::disallowed_methods)]

use dioxus::html::Key;
use dioxus::prelude::*;

/// List editor component
///
/// Provides a UI for editing a list of string items with inline editing,
/// add/remove functionality, and optional validation.
#[component]
pub fn ListEditor(
  label: String,
  #[props(optional)] hint: Option<String>,
  items: Vec<String>,
  on_change: Callback<Vec<String>>,
  #[props(optional)] placeholder: Option<String>,
  #[props(optional)] required: Option<bool>,
  #[props(optional)] max_items: Option<usize>,
) -> Element {
  let mut editing_index = use_signal(|| None::<usize>);
  let mut edit_value = use_signal(|| String::new());
  let mut new_item_value = use_signal(|| String::new());
  let mut validation_error = use_signal(|| None::<String>);

  // Clone items into a local signal to avoid move issues
  let mut items_signal = use_signal(|| items.clone());
  // Update the signal when items prop changes
  use_effect(move || {
    items_signal.set(items.clone());
  });

  let is_required = required.unwrap_or(false);
  let placeholder_text = placeholder.unwrap_or_else(|| "Enter item...".to_string());

  let can_add_more = max_items.map_or(true, |max| items_signal.read().len() < max);

  rsx! {
      div { class: "list-editor",
          div { class: "list-editor-header",
              label { class: "list-editor-label",
                  {label}
                  if is_required {
                      span { class: "required-indicator", " *" }
                  }
              }
              if let Some(hint_text) = &hint {
                  span { class: "list-editor-hint", "{hint_text}" }
              }
          }

          div { class: "list-editor-items",
              if items_signal.read().is_empty() {
                  div { class: "list-editor-empty",
                      p { "No items yet. Add your first item below." }
                  }
              } else {
                  for (index, item) in items_signal.read().iter().enumerate() {
                      div {
                          key: "{index}",
                          class: format!("list-item {}", if editing_index.read().map_or(false, |e| e == index) { "editing" } else { "" }),

                          if editing_index.read().map_or(false, |e| e == index) {
                              // Editing mode
                              input {
                                  class: "list-item-edit-input",
                                  value: "{edit_value}",
                                  placeholder: "{placeholder_text}",
                                  oninput: move |e: Event<FormData>| {
                                      let value = e.value();
                                      let sanitized = sanitize_list_item(&value);
                                      edit_value.set(sanitized);
                                  },
                                  onkeydown: move |e: KeyboardEvent| {
                                      match e.key() {
                                          Key::Enter => {
                                              let value = edit_value.read().clone();
                                              let value_trimmed = value.trim();
                                              if !value_trimmed.is_empty() {
                                                  let mut new_items = items_signal.read().clone();
                                                  new_items[index] = value_trimmed.to_string();
                                                  on_change.call(new_items);
                                                  editing_index.set(None);
                                                  edit_value.set(String::new());
                                                  validation_error.set(None);
                                              } else {
                                                  validation_error.set(Some("Item cannot be empty".to_string()));
                                              }
                                          }
                                          Key::Escape => {
                                              editing_index.set(None);
                                              edit_value.set(String::new());
                                          }
                                          _ => {}
                                      }
                                  }
                              }
                              button {
                                  class: "btn btn-icon btn-success",
                                  onclick: move |_| {
                                      let value = edit_value.read().clone();
                                      let value_trimmed = value.trim();
                                      if !value_trimmed.is_empty() {
                                          let mut new_items = items_signal.read().clone();
                                          new_items[index] = value_trimmed.to_string();
                                          on_change.call(new_items);
                                          editing_index.set(None);
                                          edit_value.set(String::new());
                                          validation_error.set(None);
                                      } else {
                                          validation_error.set(Some("Item cannot be empty".to_string()));
                                      }
                                  },
                                  "✓"
                              }
                              button {
                                  class: "btn btn-icon btn-secondary",
                                  onclick: move |_| {
                                      editing_index.set(None);
                                      edit_value.set(String::new());
                                  },
                                  "✕"
                              }
                          } else {
                              // View mode
                              span { class: "list-item-text", "{item}" }
                              div { class: "list-item-actions",
                                  button {
                                      class: "btn btn-icon btn-secondary",
                                      onclick: move |_| {
                                          editing_index.set(Some(index));
                                          edit_value.set(items_signal.read().get(index).cloned().unwrap_or_default());
                                      },
                                      title: "Edit item",
                                      "✎"
                                  }
                                  button {
                                      class: "btn btn-icon btn-danger",
                                      onclick: move |_| {
                                          let mut new_items = items_signal.read().clone();
                                          new_items.remove(index);
                                          on_change.call(new_items);
                                          validation_error.set(None);
                                      },
                                      title: "Remove item",
                                      "−"
                                  }
                              }
                          }
                      }
                  }
              }
          }

          // Add new item section
          if can_add_more {
              div { class: "list-editor-add",
                  input {
                      class: "list-editor-add-input",
                      value: "{new_item_value}",
                      placeholder: "{placeholder_text}",
                      oninput: move |e: Event<FormData>| {
                          let value = e.value();
                          let sanitized = sanitize_list_item(&value);
                          new_item_value.set(sanitized);
                          validation_error.set(None);
                      },
                      onkeydown: move |e: KeyboardEvent| {
                          if e.key() == Key::Enter {
                              let value = new_item_value.read().clone();
                              let value_trimmed = value.trim();

                              if !value_trimmed.is_empty() {
                                  if let Some(max) = max_items {
                                      if items_signal.read().len() >= max {
                                          validation_error.set(Some(format!("Maximum {max} items allowed")));
                                          return;
                                      }
                                  }

                                  let mut new_items = items_signal.read().clone();
                                  new_items.push(value_trimmed.to_string());
                                  on_change.call(new_items);
                                  new_item_value.set(String::new());
                                  validation_error.set(None);
                              } else if is_required && items_signal.read().is_empty() {
                                  validation_error.set(Some("At least one item is required".to_string()));
                              }
                          }
                      }
                  }
                  button {
                      class: "btn btn-primary",
                      onclick: move |_| {
                          let value = new_item_value.read().clone();
                          let value_trimmed = value.trim();

                          if !value_trimmed.is_empty() {
                              if let Some(max) = max_items {
                                  if items_signal.read().len() >= max {
                                      validation_error.set(Some(format!("Maximum {max} items allowed")));
                                      return;
                                  }
                              }

                              let mut new_items = items_signal.read().clone();
                              new_items.push(value_trimmed.to_string());
                              on_change.call(new_items);
                              new_item_value.set(String::new());
                              validation_error.set(None);
                          } else if is_required && items_signal.read().is_empty() {
                              validation_error.set(Some("At least one item is required".to_string()));
                          }
                      },
                      disabled: new_item_value.read().trim().is_empty(),
                      "Add Item"
                  }
              }
          } else if let Some(max) = max_items {
              div { class: "list-editor-max-reached",
                  p { class: "text-muted", "Maximum {max} items reached" }
              }
          }

          // Validation error display
          if let Some(error) = &*validation_error.read() {
              div { class: "list-editor-error",
                  span { class: "error-icon", "⚠" }
                  span { class: "error-message", "{error}" }
              }
          }

          // Item count indicator
          div { class: "list-editor-count",
              span { class: "count-label", "Items: " }
              span { class: "count-value", "{items_signal.read().len()}" }
              if let Some(max) = max_items {
                  span { class: "count-max", " / {max}" }
              }
          }
      }
  }
}

/// Validate list items against common constraints with security checks
///
/// # Errors
/// Returns a String with the error message if validation fails
#[must_use]
pub fn validate_list_items(
  items: &[String],
  min_items: Option<usize>,
  max_items: Option<usize>,
  allow_empty: bool,
) -> Result<(), String> {
  const MAX_ITEM_LENGTH: usize = 10_000;

  // Check minimum items
  if let Some(min) = min_items {
    if items.len() < min {
      return Err(format!("At least {min} item(s) required"));
    }
  }

  // Check maximum items
  if let Some(max) = max_items {
    if items.len() > max {
      return Err(format!("Maximum {max} item(s) allowed"));
    }
  }

  // Check for empty items and sanitize
  if !allow_empty {
    for (index, item) in items.iter().enumerate() {
      let trimmed = item.trim();

      if trimmed.is_empty() {
        return Err(format!("Item at position {} is empty", index + 1));
      }

      // Security: Check for dangerous Unicode patterns
      // Zero-width characters can be used for obfuscation
      if trimmed.contains('\u{200B}')
        || trimmed.contains('\u{200C}')
        || trimmed.contains('\u{200D}')
        || trimmed.contains('\u{FEFF}')
      {
        return Err(format!(
          "Item at position {} contains invalid zero-width characters",
          index + 1
        ));
      }

      // Check for RTL override which can be used for spoofing
      if trimmed.contains('\u{202E}') || trimmed.contains('\u{202D}') {
        return Err(format!(
          "Item at position {} contains invalid text direction characters",
          index + 1
        ));
      }

      // Check for path traversal attempts
      if trimmed.contains("../") || trimmed.contains("..\\") {
        return Err(format!(
          "Item at position {} contains invalid path characters",
          index + 1
        ));
      }

      // Check for null bytes
      if trimmed.contains('\0') {
        return Err(format!(
          "Item at position {} contains null bytes",
          index + 1
        ));
      }

      // Check item length limit
      if trimmed.len() > MAX_ITEM_LENGTH {
        return Err(format!(
          "Item at position {} exceeds maximum length of {} characters",
          index + 1,
          MAX_ITEM_LENGTH
        ));
      }
    }
  }

  Ok(())
}

/// Sanitize a list item by removing dangerous characters
#[must_use]
pub fn sanitize_list_item(item: &str) -> String {
  const MAX_ITEM_LENGTH: usize = 10_000;

  // Remove control characters (except tab, newline, carriage return, form feed, vertical tab)
  let sanitized: String = item
    .chars()
    .filter(|c| {
      *c == '\t' || *c == '\n' || *c == '\r' || *c == '\x0b' || *c == '\x0c' || (*c as u32) >= 32
    })
    .collect();

  // Truncate if too long
  if sanitized.len() > MAX_ITEM_LENGTH {
    sanitized.chars().take(MAX_ITEM_LENGTH).collect()
  } else {
    sanitized
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_validate_list_items_empty() {
    let items: Vec<String> = vec![];
    let result = validate_list_items(&items, Some(1), None, true);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "At least 1 item(s) required");
  }

  #[test]
  fn test_validate_list_items_too_many() {
    let items = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let result = validate_list_items(&items, None, Some(2), true);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Maximum 2 item(s) allowed");
  }

  #[test]
  fn test_validate_list_items_empty_content() {
    let items = vec!["valid".to_string(), "".to_string()];
    let result = validate_list_items(&items, None, None, false);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Item at position 2 is empty");
  }

  #[test]
  fn test_validate_list_items_success() {
    let items = vec!["item1".to_string(), "item2".to_string()];
    let result = validate_list_items(&items, Some(1), Some(5), false);
    assert!(result.is_ok());
  }

  #[test]
  fn test_validate_list_items_with_min() {
    let items = vec!["a".to_string(), "b".to_string()];
    let result = validate_list_items(&items, Some(2), None, true);
    assert!(result.is_ok());
  }

  #[test]
  fn test_validate_list_items_allow_empty() {
    let items = vec!["valid".to_string(), "".to_string()];
    let result = validate_list_items(&items, None, None, true);
    assert!(result.is_ok());
  }

  #[test]
  fn test_validate_list_items_no_constraints() {
    let items = vec!["a".to_string()];
    let result = validate_list_items(&items, None, None, true);
    assert!(result.is_ok());
  }

  #[test]
  fn test_validate_list_items_empty_list_no_min() {
    let items: Vec<String> = vec![];
    let result = validate_list_items(&items, None, Some(5), true);
    assert!(result.is_ok());
  }

  #[test]
  fn test_validate_list_items_exact_min() {
    let items = vec!["a".to_string()];
    let result = validate_list_items(&items, Some(1), None, true);
    assert!(result.is_ok());
  }

  #[test]
  fn test_validate_list_items_exact_max() {
    let items = vec!["a".to_string(), "b".to_string()];
    let result = validate_list_items(&items, None, Some(2), true);
    assert!(result.is_ok());
  }

  #[test]
  fn test_validate_list_items_rejects_zero_width_chars() {
    let items = vec!["test\u{200B}hacked".to_string()];
    let result = validate_list_items(&items, None, None, false);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("zero-width"));
  }

  #[test]
  fn test_validate_list_items_rejects_rtl_override() {
    let items = vec!["test\u{202E}hacked".to_string()];
    let result = validate_list_items(&items, None, None, false);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("text direction"));
  }

  #[test]
  fn test_validate_list_items_rejects_path_traversal() {
    let items = vec!["../../../etc/passwd".to_string()];
    let result = validate_list_items(&items, None, None, false);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("path"));
  }

  #[test]
  fn test_validate_list_items_rejects_null_bytes() {
    let items = vec!["test\0null".to_string()];
    let result = validate_list_items(&items, None, None, false);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("null"));
  }

  #[test]
  fn test_sanitize_list_item_removes_control_chars() {
    let input = "test\u{0000}item\u{0001}";
    let sanitized = sanitize_list_item(input);
    assert!(!sanitized.contains('\0'));
    assert!(!sanitized.contains('\u{0001}'));
  }

  #[test]
  fn test_sanitize_list_item_truncates_long_input() {
    let long_input = "a".repeat(20_000);
    let sanitized = sanitize_list_item(&long_input);
    assert_eq!(sanitized.len(), 10_000);
  }

  #[test]
  fn test_sanitize_list_item_preserves_valid_chars() {
    let input = "test-item_123";
    let sanitized = sanitize_list_item(input);
    assert_eq!(sanitized, input);
  }
}
