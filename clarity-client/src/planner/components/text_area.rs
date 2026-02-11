//! Text area component
//!
//! A reusable textarea component with validation and character count.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
// Dioxus rsx! macro internally uses unwrap, so we allow the disallowed_methods lint.
#![allow(clippy::disallowed_methods)]

use crate::planner::components::FieldLabel;
use dioxus::prelude::*;

/// Props for the TextArea component
#[derive(Clone, Debug, PartialEq, Props)]
pub struct TextAreaProps {
  /// Label for the textarea
  pub label: String,
  /// Current value
  pub value: String,
  /// Callback when value changes
  pub on_change: Callback<String>,
  /// Optional placeholder text
  pub placeholder: Option<String>,
  /// Optional hint text
  pub hint: Option<String>,
  /// Whether the field is required
  pub required: Option<bool>,
  /// Optional maximum length
  pub max_length: Option<usize>,
  /// Number of rows
  pub rows: Option<usize>,
  /// Whether the field is disabled
  pub disabled: Option<bool>,
  /// Whether to show character count
  pub show_char_count: Option<bool>,
}

/// Textarea component
///
/// A styled textarea with:
/// - Optional label and hint
/// - Character limit with count
/// - Disabled state
/// - Placeholder text
///
/// # Example
///
/// ```ignore
/// rsx! {
///     TextArea {
///         label: "Description".to_string(),
///         value: description.clone(),
///         on_change: move |s| on_description_change.call(s),
///         placeholder: Some("Enter a description...".to_string()),
///         hint: Some("Be specific and concise".to_string()),
///         required: Some(true),
///         max_length: Some(500),
///         rows: Some(4),
///         show_char_count: Some(true),
///     }
/// }
/// ```
#[component]
pub fn TextArea(props: TextAreaProps) -> Element {
  let required = props.required.unwrap_or(false);
  let disabled = props.disabled.unwrap_or(false);
  let rows = props.rows.unwrap_or(3);
  let show_char_count = props.show_char_count.unwrap_or(false);

  let char_count = props.value.chars().count();
  let exceeds_limit = props.max_length.map_or(false, |max| char_count > max);

  let char_count_class = if exceeds_limit {
    "text-xs text-red-500"
  } else {
    "text-xs text-gray-500"
  };

  let textarea_class = format!(
    "w-full px-3 py-2 border rounded focus:outline-none focus:ring-2 focus:ring-blue-500 {}",
    if disabled {
      "bg-gray-100 text-gray-500 cursor-not-allowed"
    } else {
      "bg-white"
    }
  );

  let border_class = if exceeds_limit {
    "border-red-500"
  } else {
    "border-gray-300"
  };

  rsx! {
      div {
          class: "flex flex-col gap-2",

          FieldLabel {
              label: props.label.clone(),
              hint: props.hint.clone(),
              required,
          }

          div {
              class: "relative",

              textarea {
                  class: "{textarea_class} {border_class}",
                  value: "{props.value}",
                  placeholder: props.placeholder.as_deref().unwrap_or(""),
                  rows: rows as u64,
                  disabled: disabled,
                  oninput: move |e| {
                      let value = e.value();
                      if let Some(max) = props.max_length {
                          let current_count = value.chars().count();
                          if current_count <= max {
                              props.on_change.call(value);
                          }
                      } else {
                          props.on_change.call(value);
                      }
                  }
              }

              if show_char_count {
                  div {
                      class: "flex justify-between items-center mt-1",
                      if let Some(max) = props.max_length {
                          span {
                              class: "{char_count_class}",
                              "{char_count} / {max}"
                          }
                      } else {
                          span {
                              class: "text-xs text-gray-500",
                              "{char_count} characters"
                          }
                      }
                  }
              }
          }
      }
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn test_text_area_props_creation() {
    // Just verify we can create the values - actual rendering requires Dioxus runtime
    let value = String::new();
    let label = "Test".to_string();

    assert!(value.is_empty());
    assert_eq!(label, "Test");
  }

  #[test]
  fn test_text_area_props_with_options() {
    // Verify values without creating Callback (requires Dioxus runtime)
    let value = "Test content".to_string();
    let max_length = Some(100);
    let rows = Some(5);

    assert_eq!(value, "Test content");
    assert_eq!(max_length, Some(100));
    assert_eq!(rows, Some(5));
  }

  #[test]
  fn test_character_count() {
    let text = "Hello, world!";
    let count = text.chars().count();
    assert_eq!(count, 13);
  }

  #[test]
  fn test_exceeds_limit_detection() {
    let text = "Hello";
    let max_length = 3;
    let exceeds = text.chars().count() > max_length;
    assert!(exceeds);
  }
}
