#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::storage::types::Confidence;
use dioxus::prelude::*;

/// Props for the `ConfidenceBadge` component
#[derive(Clone, Debug, PartialEq, Eq, Props)]
pub struct ConfidenceBadgeProps {
  /// Confidence level to display
  pub confidence: Confidence,
  /// Additional CSS classes to apply
  #[props(default)]
  pub class: String,
}

/// Get the color classes for a given confidence level
const fn confidence_color_classes(confidence: Confidence) -> &'static str {
  match confidence {
    Confidence::High => "bg-emerald-500/60 text-emerald-400 border-transparent",
    Confidence::Inferred => "bg-amber-500/60 text-amber-400 border-transparent",
    Confidence::Uncertain => "bg-red-500/60 text-red-400 border-transparent",
  }
}

/// Get the icon SVG for a given confidence level
fn confidence_icon(confidence: Confidence) -> Element {
  match confidence {
    Confidence::High => rsx! {
        svg {
            class: "h-3 w-3",
            xmlns: "http://www.w3.org/2000/svg",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "3",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path {
                d: "M20 6L9 17l-5-5"
            }
        }
    },
    Confidence::Inferred => rsx! {
        svg {
            class: "h-3 w-3",
            xmlns: "http://www.w3.org/2000/svg",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path {
                d: "M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"
            }
            line {
                x1: "12",
                y1: "9",
                x2: "12",
                y2: "13"
            }
            line {
                x1: "12",
                y1: "17",
                x2: "12.01",
                y2: "17"
            }
        }
    },
    Confidence::Uncertain => rsx! {
        svg {
            class: "h-3 w-3",
            xmlns: "http://www.w3.org/2000/svg",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle {
                cx: "12",
                cy: "12",
                r: "10"
            }
            path {
                d: "M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"
            }
            line {
                x1: "12",
                y1: "17",
                x2: "12.01",
                y2: "17"
            }
        }
    },
  }
}

/// Get the label text for a given confidence level
const fn confidence_label(confidence: Confidence) -> &'static str {
  match confidence {
    Confidence::High => "High",
    Confidence::Inferred => "Inferred",
    Confidence::Uncertain => "Uncertain",
  }
}

/// `ConfidenceBadge` component - displays confidence level with color coding and icons
///
/// # Color Coding
/// - High: Green with checkmark icon
/// - Inferred: Yellow/amber with alert icon
/// - Uncertain: Red with question mark icon
///
/// # Example
/// ```rust
/// use dioxus::prelude::*;
/// use clarity_web::ui::ConfidenceBadge;
/// use clarity_web::storage::types::Confidence;
///
/// fn app() -> Element {
///     rsx! {
///         ConfidenceBadge {
///             confidence: Confidence::High,
///         }
///     }
/// }
/// ```
#[component]
pub fn ConfidenceBadge(props: ConfidenceBadgeProps) -> Element {
  let color_classes = confidence_color_classes(props.confidence);
  let icon = confidence_icon(props.confidence);
  let label = confidence_label(props.confidence);

  rsx! {
      div {
          class: format!(
              "inline-flex items-center gap-1.5 rounded-full border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 {} {}",
              color_classes,
              props.class
          ),
          {icon}
          span {
              {label}
          }
      }
  }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::float_cmp, clippy::needless_collect, clippy::unnecessary_debug_formatting, clippy::match_same_arms, clippy::option_if_let_else, clippy::suspicious_else_formatting, clippy::manual_let_else, clippy::match_wild_err_arm, clippy::match_like_matches_macro, clippy::needless_pass_by_value)]
mod tests {
  use super::*;

  #[test]
  fn test_confidence_color_classes_high() {
    let classes = confidence_color_classes(Confidence::High);
    assert!(classes.contains("bg-emerald-500/60"));
    assert!(classes.contains("text-emerald-400"));
    assert!(classes.contains("border-transparent"));
  }

  #[test]
  fn test_confidence_color_classes_inferred() {
    let classes = confidence_color_classes(Confidence::Inferred);
    assert!(classes.contains("bg-amber-500/60"));
    assert!(classes.contains("text-amber-400"));
    assert!(classes.contains("border-transparent"));
  }

  #[test]
  fn test_confidence_color_classes_uncertain() {
    let classes = confidence_color_classes(Confidence::Uncertain);
    assert!(classes.contains("bg-red-500/60"));
    assert!(classes.contains("text-red-400"));
    assert!(classes.contains("border-transparent"));
  }

  #[test]
  fn test_confidence_label_high() {
    let label = confidence_label(Confidence::High);
    assert_eq!(label, "High");
  }

  #[test]
  fn test_confidence_label_inferred() {
    let label = confidence_label(Confidence::Inferred);
    assert_eq!(label, "Inferred");
  }

  #[test]
  fn test_confidence_label_uncertain() {
    let label = confidence_label(Confidence::Uncertain);
    assert_eq!(label, "Uncertain");
  }

  #[test]
  fn test_confidence_badge_props_default_class() {
    let props = ConfidenceBadgeProps {
      confidence: Confidence::High,
      class: String::new(),
    };
    assert_eq!(props.confidence, Confidence::High);
    assert!(props.class.is_empty());
  }

  #[test]
  fn test_confidence_badge_props_with_class() {
    let props = ConfidenceBadgeProps {
      confidence: Confidence::Uncertain,
      class: "custom-class".to_string(),
    };
    assert_eq!(props.confidence, Confidence::Uncertain);
    assert_eq!(props.class, "custom-class");
  }

  #[test]
  fn test_confidence_equality() {
    assert_eq!(Confidence::High, Confidence::High);
    assert_eq!(Confidence::Inferred, Confidence::Inferred);
    assert_eq!(Confidence::Uncertain, Confidence::Uncertain);

    assert_ne!(Confidence::High, Confidence::Inferred);
    assert_ne!(Confidence::Inferred, Confidence::Uncertain);
    assert_ne!(Confidence::High, Confidence::Uncertain);
  }
}
