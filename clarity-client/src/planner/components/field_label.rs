//! Field label component
//!
//! A reusable component for form field labels with optional hints.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(warnings)]
#![allow(clippy::all)]
// Dioxus rsx! macro internally uses unwrap, so we allow the disallowed_methods lint.
#![allow(clippy::disallowed_methods)]

use dioxus::prelude::*;

/// Props for the `FieldLabel` component
#[derive(Clone, Debug, PartialEq, Props)]
pub struct FieldLabelProps {
  /// The label text
  pub label: String,
  /// Optional hint/help text
  pub hint: Option<String>,
  /// Whether the field is required
  pub required: bool,
}

/// Field hint content for displaying additional help
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldHint {
  /// The hint text
  pub text: String,
  /// Whether to show as a tooltip (true) or inline hint (false)
  pub tooltip: bool,
}

impl FieldHint {
  /// Create a new field hint
  #[must_use]
  pub const fn new(text: String) -> Self {
    Self {
      text,
      tooltip: false,
    }
  }

  /// Create a tooltip hint
  #[must_use]
  pub const fn tooltip(text: String) -> Self {
    Self {
      text,
      tooltip: true,
    }
  }

  /// Create an inline hint
  #[must_use]
  pub const fn inline(text: String) -> Self {
    Self {
      text,
      tooltip: false,
    }
  }
}

impl From<String> for FieldHint {
  fn from(text: String) -> Self {
    Self::new(text)
  }
}

impl From<&str> for FieldHint {
  fn from(text: &str) -> Self {
    Self::new(text.to_string())
  }
}

/// Field label component
///
/// Displays a form field label with optional hint text.
/// Automatically shows an asterisk for required fields.
///
/// # Example
///
/// ```ignore
/// rsx! {
///     FieldLabel {
///         label: "Product Name".to_string(),
///         hint: Some("Enter a clear, descriptive name".to_string()),
///         required: true,
///     }
/// }
/// ```
#[component]
pub fn FieldLabel(props: FieldLabelProps) -> Element {
  rsx! {
      div {
          class: "flex flex-col gap-1 mb-2",
          label {
              class: "flex items-center gap-1 text-sm font-medium text-gray-700",
              "{props.label}"
              if props.required {
                  span {
                      class: "text-red-500",
                      "*"
                  }
              }
          }
          if let Some(hint) = &props.hint {
              p {
                  class: "text-xs text-gray-500",
                  "{hint}"
              }
          }
      }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_field_hint_new() {
    let hint = FieldHint::new("Help text".to_string());
    assert_eq!(hint.text, "Help text");
    assert!(!hint.tooltip);
  }

  #[test]
  fn test_field_hint_tooltip() {
    let hint = FieldHint::tooltip("Tooltip text".to_string());
    assert_eq!(hint.text, "Tooltip text");
    assert!(hint.tooltip);
  }

  #[test]
  fn test_field_hint_inline() {
    let hint = FieldHint::inline("Inline hint".to_string());
    assert_eq!(hint.text, "Inline hint");
    assert!(!hint.tooltip);
  }

  #[test]
  fn test_field_hint_from_string() {
    let hint: FieldHint = "Test hint".to_string().into();
    assert_eq!(hint.text, "Test hint");
  }

  #[test]
  fn test_field_hint_from_str() {
    let hint: FieldHint = "Test hint".into();
    assert_eq!(hint.text, "Test hint");
  }

  #[test]
  fn test_field_label_props_required() {
    let props = FieldLabelProps {
      label: "Name".to_string(),
      hint: None,
      required: true,
    };

    assert!(props.required);
  }

  #[test]
  fn test_field_label_props_optional() {
    let props = FieldLabelProps {
      label: "Description".to_string(),
      hint: Some("Optional field".to_string()),
      required: false,
    };

    assert!(!props.required);
    assert_eq!(props.hint, Some("Optional field".to_string()));
  }
}
