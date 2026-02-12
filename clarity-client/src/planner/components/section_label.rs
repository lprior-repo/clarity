//! Section label component
//!
//! A reusable component for section headers and labels.

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

/// Props for the SectionLabel component
#[derive(Clone, Debug, PartialEq, Props)]
pub struct SectionLabelProps {
  /// The label text
  pub label: String,
  /// Optional description text
  pub description: Option<String>,
  /// Section level (affects styling)
  pub level: Option<SectionLevel>,
}

/// Section level for styling hierarchy
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SectionLevel {
  /// Phase level (top of Diamond)
  Phase,
  /// Top level section
  #[default]
  Primary,
  /// Secondary section
  Section,
  /// Tertiary section
  Tertiary,
}

/// Section label component
///
/// Displays a section header with optional description.
/// Used for organizing content into clear sections.
///
/// # Example
///
/// ```ignore
/// rsx! {
///     SectionLabel {
///         label: "Product Thesis".to_string(),
///         description: Some("Define your product's core value proposition".to_string()),
///         level: SectionLevel::Primary,
///     }
/// }
/// ```
#[component]
pub fn SectionLabel(props: SectionLabelProps) -> Element {
  let level = props.level.unwrap_or_default();

  let (heading_class, description_class) = match level {
    SectionLevel::Phase => (
      "text-3xl font-bold text-gray-900",
      "text-base text-gray-700 mt-2",
    ),
    SectionLevel::Primary => (
      "text-2xl font-bold text-gray-900",
      "text-sm text-gray-600 mt-1",
    ),
    SectionLevel::Section => (
      "text-xl font-semibold text-gray-800",
      "text-sm text-gray-600 mt-1",
    ),
    SectionLevel::Tertiary => (
      "text-lg font-medium text-gray-700",
      "text-xs text-gray-500 mt-0.5",
    ),
  };

  rsx! {
      div {
          class: "mb-4",
          h2 {
              class: "{heading_class}",
              "{props.label}"
          }
          if let Some(description) = &props.description {
              p {
                  class: "{description_class}",
                  "{description}"
              }
          }
      }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_section_label_props() {
    let props = SectionLabelProps {
      label: "Test Section".to_string(),
      description: Some("A description".to_string()),
      level: Some(SectionLevel::Primary),
    };

    assert_eq!(props.label, "Test Section");
    assert_eq!(props.description, Some("A description".to_string()));
  }

  #[test]
  fn test_section_level_default() {
    let level = SectionLevel::default();
    assert_eq!(level, SectionLevel::Primary);
  }
}
