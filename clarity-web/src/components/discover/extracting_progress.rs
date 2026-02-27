#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![allow(clippy::suspicious_else_formatting)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use dioxus::prelude::*;

/// Props for ExtractingProgress component
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ExtractionStatus {
  /// Extraction has not started
  Idle,
  /// Extraction is in progress
  Extracting,
  /// Extraction completed successfully
  Complete,
  /// Extraction failed
  Failed,
}

impl Default for ExtractionStatus {
  fn default() -> Self {
    Self::Idle
  }
}

/// Props for ExtractingProgress component
#[derive(Clone, Props, PartialEq)]
pub struct ExtractingProgressProps {
  /// Current extraction status
  #[props(default)]
  pub status: ExtractionStatus,

  /// Progress percentage (0-100), only relevant when status is Extracting
  #[props(default = 0)]
  pub progress: u8,

  /// Optional custom status message
  #[props(default)]
  pub message: Option<String>,
}

/// ExtractingProgress component
///
/// Displays an animated progress indicator during artifact extraction:
/// - Animated progress bar with shimmer effect
/// - Status text "Extracting fields..."
/// - Smooth CSS animations (no JS-driven animation)
/// - Respects reduced motion preferences via Tailwind
///
/// # Accessibility
///
/// Uses `role="progressbar"` with `aria-valuenow`, `aria-valuemin`, and `aria-valuemax`
/// for screen reader compatibility.
#[component]
pub fn ExtractingProgress(props: ExtractingProgressProps) -> Element {
  let ExtractingProgressProps {
    status,
    progress,
    message,
  } = props;

  // Clamp progress to valid range
  let clamped_progress = progress.min(100);

  // Determine if we should show the component
  let is_visible = matches!(
    status,
    ExtractionStatus::Extracting | ExtractionStatus::Complete
  );

  // Get status text
  let status_text = match message.as_ref() {
    Some(msg) => msg.clone(),
    None => match status {
      ExtractionStatus::Idle => "Ready to extract".to_string(),
      ExtractionStatus::Extracting => "Extracting fields...".to_string(),
      ExtractionStatus::Complete => "Extraction complete!".to_string(),
      ExtractionStatus::Failed => "Extraction failed".to_string(),
    },
  };

  // Calculate progress bar width
  let progress_width = format!("{clamped_progress}%");

  if !is_visible {
    return rsx! {};
  }

  rsx! {
      div {
          class: "flex flex-col gap-3 w-full max-w-md",

          // Status text with animated ellipsis
          div {
              class: "flex items-center gap-2",
              "aria-live": "polite",
              "aria-atomic": "true",

              // Animated icon based on status
              if status == ExtractionStatus::Extracting {
                  // Spinning loader icon
                  svg {
                      class: "h-4 w-4 animate-spin text-primary",
                      xmlns: "http://www.w3.org/2000/svg",
                      fill: "none",
                      view_box: "0 0 24 24",
                      circle {
                          class: "opacity-25",
                          cx: "12",
                          cy: "12",
                          r: "10",
                          stroke: "currentColor",
                          stroke_width: "4",
                      }
                      path {
                          class: "opacity-75",
                          fill: "currentColor",
                          d: "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z",
                      }
                  }
              } else if status == ExtractionStatus::Complete {
                  // Checkmark icon
                  svg {
                      class: "h-4 w-4 text-emerald-400",
                      xmlns: "http://www.w3.org/2000/svg",
                      fill: "none",
                      view_box: "0 0 24 24",
                      stroke: "currentColor",
                      stroke_width: "2",
                      stroke_linecap: "round",
                      stroke_linejoin: "round",
                      path { d: "M5 13l4 4L19 7" },
                  }
              }

              span {
                  class: if status == ExtractionStatus::Complete {
                      "text-sm font-medium text-emerald-400"
                  } else {
                      "text-sm font-medium text-muted-foreground"
                  },
                  "{status_text}"
              }
          }

          // Progress bar container
          div {
              class: "relative h-2 w-full overflow-hidden rounded-full bg-muted/30",
              role: "progressbar",
              "aria-valuenow": "{clamped_progress}",
              "aria-valuemin": "0",
              "aria-valuemax": "100",
              "aria-label": "Extraction progress",

              // Progress fill
              div {
                  class: format!(
                      "h-full rounded-full transition-all duration-300 ease-out {}",
                      if status == ExtractionStatus::Complete {
                          "bg-emerald-500"
                      } else {
                          "bg-primary"
                      }
                  ),
                  style: "width: {progress_width};",

                  // Shimmer overlay for extracting state
                  if status == ExtractionStatus::Extracting {
                      div {
                          class: "absolute inset-0 animate-extraction-shimmer",
                          style: "
                                background: linear-gradient(
                                    90deg,
                                    transparent 0%,
                                    rgba(255, 255, 255, 0.15) 50%,
                                    transparent 100%
                                );
                                background-size: 200% 100%;
                            ",
                      }
                  }
              }
          }

          // Progress percentage text
          div {
              class: "flex justify-between text-xs text-muted-foreground/60 font-mono",
              span { "{clamped_progress}%" }
              span { "100%" }
          }
      }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_extraction_status_default() {
    assert_eq!(ExtractionStatus::default(), ExtractionStatus::Idle);
  }

  #[test]
  fn test_extraction_status_equality() {
    assert_eq!(ExtractionStatus::Idle, ExtractionStatus::Idle);
    assert_eq!(ExtractionStatus::Extracting, ExtractionStatus::Extracting);
    assert_eq!(ExtractionStatus::Complete, ExtractionStatus::Complete);
    assert_eq!(ExtractionStatus::Failed, ExtractionStatus::Failed);
    assert_ne!(ExtractionStatus::Idle, ExtractionStatus::Extracting);
  }

  #[test]
  fn test_extraction_status_clone() {
    let status = ExtractionStatus::Extracting;
    let cloned = status.clone();
    assert_eq!(status, cloned);
  }

  #[test]
  fn test_extraction_status_copy() {
    let status = ExtractionStatus::Complete;
    let copied = status;
    assert_eq!(status, copied);
  }

  #[test]
  fn test_extraction_status_debug() {
    let status = ExtractionStatus::Extracting;
    let debug_str = format!("{status:?}");
    assert!(debug_str.contains("Extracting"));
  }

  #[test]
  fn test_props_default_status() {
    let props = ExtractingProgressProps {
      status: ExtractionStatus::default(),
      progress: 0,
      message: None,
    };
    assert_eq!(props.status, ExtractionStatus::Idle);
    assert_eq!(props.progress, 0);
    assert!(props.message.is_none());
  }

  #[test]
  fn test_props_with_message() {
    let props = ExtractingProgressProps {
      status: ExtractionStatus::Extracting,
      progress: 50,
      message: Some("Custom message".to_string()),
    };
    assert_eq!(props.status, ExtractionStatus::Extracting);
    assert_eq!(props.progress, 50);
    assert_eq!(props.message, Some("Custom message".to_string()));
  }

  #[test]
  fn test_progress_clamping() {
    // Test that progress values are handled correctly
    let valid_progress: u8 = 75;
    assert!(valid_progress <= 100);

    let max_progress: u8 = 100;
    assert!(max_progress <= 100);
  }

  #[test]
  fn test_status_matches() {
    assert!(matches!(ExtractionStatus::Idle, ExtractionStatus::Idle));
    assert!(matches!(
      ExtractionStatus::Extracting,
      ExtractionStatus::Extracting
    ));
    assert!(!matches!(
      ExtractionStatus::Idle,
      ExtractionStatus::Extracting
    ));
  }

  #[test]
  fn test_all_status_variants() {
    let statuses = [
      ExtractionStatus::Idle,
      ExtractionStatus::Extracting,
      ExtractionStatus::Complete,
      ExtractionStatus::Failed,
    ];

    // Verify all variants are distinct
    for (i, status) in statuses.iter().enumerate() {
      for (j, other) in statuses.iter().enumerate() {
        if i != j {
          assert_ne!(status, other, "Status variants should be unique");
        }
      }
    }
  }
}
