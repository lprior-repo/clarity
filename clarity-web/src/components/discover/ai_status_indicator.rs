#![warn(clippy::unwrap_used)]
#![warn(clippy::expect_used)]
#![warn(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

//! AI Status Indicator component for the Progressive Discover flow.
//!
//! This component displays the current AI provider/model status and provides
//! feedback on AI request lifecycle (loading, success, error).
//!
//! # Requirements (from bead bd-7poi)
//!
//! - THE SYSTEM SHALL display active AI provider and model in the discover flow when AI is configured.
//! - WHEN an AI request starts, succeeds, or fails, THE SYSTEM SHALL update visible status.
//! - IF provider call fails, THE SYSTEM SHALL NOT leave stale loading indicators or hide failure context.
//! - THE SYSTEM SHALL provide a mechanism for users to request feedback on errors.

use dioxus::prelude::*;

use super::state::{AiRequestStatus, AiStatus};

/// Props for `AiStatusIndicator` component
#[derive(Clone, Props, PartialEq)]
pub struct AiStatusIndicatorProps {
  /// Current AI status
  pub status: AiStatus,
  /// Optional additional CSS classes
  #[props(default)]
  pub class: String,
  /// Whether to show detailed info (provider/model)
  #[props(default = true)]
  pub show_details: bool,
  /// Whether to show error suggestions
  #[props(default = true)]
  pub show_error_suggestion: bool,
  /// Callback when user requests feedback/help on an error
  #[props(default)]
  pub on_request_feedback: Option<EventHandler<String>>,
}

/// AI Status Indicator component
///
/// Displays the current status of AI operations including:
/// - Provider and model information
/// - Request status (loading, success, error)
/// - Error messages with suggestions
/// - Feedback request button
///
/// # Example
///
/// ```rust,ignore
/// let ai_status = use_signal(|| AiStatus::idle());
///
/// rsx! {
///     AiStatusIndicator {
///         status: ai_status.read().clone(),
///         on_request_feedback: Some(move |error: String| {
///             // Handle feedback request
///         }),
///     }
/// }
/// ```
#[component]
pub fn AiStatusIndicator(props: AiStatusIndicatorProps) -> Element {
  let mut show_feedback_form = use_signal(|| false);
  let mut feedback_text = use_signal(|| String::new());

  let status = props.status.status;
  let status_class = match status {
    AiRequestStatus::Idle => "text-muted-foreground",
    AiRequestStatus::Loading => "text-primary",
    AiRequestStatus::Success => "text-green-600 dark:text-green-400",
    AiRequestStatus::Error => "text-destructive",
  };

  let bg_class = match status {
    AiRequestStatus::Idle => "bg-muted/50",
    AiRequestStatus::Loading => "bg-primary/10",
    AiRequestStatus::Success => "bg-green-100 dark:bg-green-900/30",
    AiRequestStatus::Error => "bg-destructive/10",
  };

  rsx! {
      div {
          class: format!("rounded-lg border border-border/50 {} {}", bg_class, props.class),

          // Status header
          div {
              class: "flex items-center justify-between p-3",

              // Left side: status icon and text
              div {
                  class: "flex items-center gap-2",

                  // Status icon
                  StatusIcon {
                      status,
                  }

                  // Status text
                  div {
                      class: "flex flex-col",
                      span {
                          class: format!("text-sm font-medium {}", status_class),
                          "{status.display_name()}"
                      }

                      // Provider/model details
                      if props.show_details && props.status.provider_info.is_configured() {
                          span {
                              class: "text-xs text-muted-foreground",
                              "{props.status.provider_info.display_string()}"

                              // Duration for completed requests
                              if let Some(duration) = props.status.provider_info.processing_duration_ms {
                                  span {
                                      class: "ml-1",
                                      "({duration}ms)"
                                  }
                              }
                          }
                      }
                  }
              }

              // Right side: loading spinner
              if props.status.is_loading() {
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
              }
          }

          // Error details (if error)
          if props.status.is_error() {
              div {
                  class: "border-t border-border/50 p-3",

                  // Error message
                  if let Some(error_msg) = &props.status.error_message {
                      p {
                          class: "text-sm text-destructive",
                          "{error_msg}"
                      }
                  }

                  // Error category and suggestion
                  if let Some(category) = props.status.error_category {
                      div {
                          class: "mt-2 flex flex-col gap-1",

                          span {
                              class: "text-xs font-medium text-muted-foreground",
                              "{category.display_name()}"
                          }

                          if props.show_error_suggestion {
                              p {
                                  class: "text-xs text-muted-foreground",
                                  "{category.suggestion()}"
                              }
                          }
                      }
                  }

                  // Feedback request button
                  if props.on_request_feedback.is_some() {
                      div {
                          class: "mt-3",

                          if !*show_feedback_form.read() {
                              button {
                                  class: "text-xs text-primary hover:underline",
                                  onclick: move |_| {
                                      *show_feedback_form.write() = true;
                                  },
                                  "Report this issue / Request help"
                              }
                          } else {
                              div {
                                  class: "flex flex-col gap-2",

                                  textarea {
                                      class: "w-full rounded border border-border p-2 text-xs",
                                      rows: "2",
                                      placeholder: "Describe what happened (optional)...",
                                      value: feedback_text.read().clone(),
                                      oninput: {
                                          let mut feedback_text = feedback_text;
                                          move |e: FormEvent| {
                                              feedback_text.set(e.value());
                                          }
                                      },
                                  }

                                  div {
                                      class: "flex gap-2",
                                      button {
                                          class: "rounded bg-primary px-2 py-1 text-xs text-primary-foreground hover:bg-primary/90",
                                          onclick: {
                                              let on_feedback = props.on_request_feedback.clone();
                                              let error_msg = props.status.error_message.clone().unwrap_or_default();
                                              let feedback = feedback_text.read().clone();
                                              let mut show_form = show_feedback_form;
                                              move |_| {
                                                  if let Some(handler) = on_feedback.as_ref() {
                                                      let message = if feedback.is_empty() {
                                                          error_msg.clone()
                                                      } else {
                                                          format!("{}: {}", error_msg, feedback)
                                                      };
                                                      handler.call(message);
                                                  }
                                                  *show_form.write() = false;
                                              }
                                          },
                                          "Submit"
                                      }
                                      button {
                                          class: "rounded border border-border px-2 py-1 text-xs hover:bg-muted",
                                          onclick: {
                                              let mut show_form = show_feedback_form;
                                              let mut feedback_text = feedback_text;
                                              move |_| {
                                                  *show_form.write() = false;
                                                  feedback_text.set(String::new());
                                              }
                                          },
                                          "Cancel"
                                      }
                                  }
                              }
                          }
                      }
                  }
              }
          }
      }
  }
}

/// Props for StatusIcon component
#[derive(Clone, Debug, Props, PartialEq)]
pub struct StatusIconProps {
  /// Current request status
  pub status: AiRequestStatus,
}

/// Status icon component
///
/// Displays an appropriate icon for the current AI request status.
#[component]
fn StatusIcon(props: StatusIconProps) -> Element {
  match props.status {
    AiRequestStatus::Idle => rsx! {
        svg {
            class: "h-4 w-4 text-muted-foreground",
            xmlns: "http://www.w3.org/2000/svg",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "12", cy: "12", r: "10" }
            path { d: "M12 16v-4" }
            path { d: "M12 8h.01" }
        }
    },
    AiRequestStatus::Loading => rsx! {
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
    },
    AiRequestStatus::Success => rsx! {
        svg {
            class: "h-4 w-4 text-green-600 dark:text-green-400",
            xmlns: "http://www.w3.org/2000/svg",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M22 11.08V12a10 10 0 1 1-5.93-9.14" }
            polyline { points: "22 4 12 14.01 9 11.01" }
        }
    },
    AiRequestStatus::Error => rsx! {
        svg {
            class: "h-4 w-4 text-destructive",
            xmlns: "http://www.w3.org/2000/svg",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "12", cy: "12", r: "10" }
            line { x1: "15", y1: "9", x2: "9", y2: "15" }
            line { x1: "9", y1: "9", x2: "15", y2: "15" }
        }
    },
  }
}

/// Props for `AiStatusBadge` component
#[derive(Clone, Debug, Props, PartialEq)]
pub struct AiStatusBadgeProps {
  /// Current AI status
  pub status: AiStatus,
  /// Optional additional CSS classes
  #[props(default)]
  pub class: String,
}

/// Compact badge-style AI status indicator
///
/// A minimal inline status display for use in tight spaces.
#[component]
pub fn AiStatusBadge(props: AiStatusBadgeProps) -> Element {
  let status = props.status.status;
  let (bg_class, text_class) = match status {
    AiRequestStatus::Idle => ("bg-muted", "text-muted-foreground"),
    AiRequestStatus::Loading => ("bg-primary/20", "text-primary"),
    AiRequestStatus::Success => ("bg-green-100 dark:bg-green-900/30", "text-green-700 dark:text-green-400"),
    AiRequestStatus::Error => ("bg-destructive/20", "text-destructive"),
  };

  let dot_class = match status {
    AiRequestStatus::Idle => "bg-muted-foreground",
    AiRequestStatus::Loading => "bg-primary animate-pulse",
    AiRequestStatus::Success => "bg-green-500",
    AiRequestStatus::Error => "bg-destructive",
  };

  rsx! {
      span {
          class: format!(
              "inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-xs font-medium {} {}",
              bg_class, text_class
          ),

          // Status dot
          span {
              class: format!("h-1.5 w-1.5 rounded-full {}", dot_class),
          }

          "{status.display_name()}"

          // Provider info if available
          if props.status.provider_info.is_configured() {
              span {
                  class: "opacity-70",
                  "- {props.status.provider_info.provider}"
              }
          }
      }
  }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::float_cmp, clippy::needless_collect, clippy::unnecessary_debug_formatting, clippy::match_same_arms, clippy::option_if_let_else, clippy::suspicious_else_formatting, clippy::manual_let_else, clippy::match_wild_err_arm, clippy::match_like_matches_macro, clippy::needless_pass_by_value)]
mod tests {
  use super::*;
  use crate::components::discover::state::{AiErrorCategory, AiProviderInfo};

  #[test]
  fn test_ai_status_idle() {
    let status = AiStatus::idle();
    assert_eq!(status.status, AiRequestStatus::Idle);
    assert!(!status.is_loading());
    assert!(!status.is_success());
    assert!(!status.is_error());
    assert_eq!(status.summary(), "AI: Ready");
  }

  #[test]
  fn test_ai_status_loading() {
    let status = AiStatus::loading("opencode".to_string(), Some("glm-5".to_string()));
    assert_eq!(status.status, AiRequestStatus::Loading);
    assert!(status.is_loading());
    assert!(!status.is_success());
    assert!(!status.is_error());
    assert_eq!(status.summary(), "AI: Processing with opencode / glm-5");
  }

  #[test]
  fn test_ai_status_success() {
    let status = AiStatus::success("opencode".to_string(), Some("glm-5".to_string()), 150);
    assert_eq!(status.status, AiRequestStatus::Success);
    assert!(!status.is_loading());
    assert!(status.is_success());
    assert!(!status.is_error());
    assert_eq!(status.summary(), "AI: opencode / glm-5 in 150ms");
  }

  #[test]
  fn test_ai_status_error() {
    let status = AiStatus::error("Connection refused".to_string(), AiErrorCategory::Network);
    assert_eq!(status.status, AiRequestStatus::Error);
    assert!(!status.is_loading());
    assert!(!status.is_success());
    assert!(status.is_error());
    assert!(status.summary().contains("Network Error"));
    assert!(status.summary().contains("Connection refused"));
  }

  #[test]
  fn test_error_category_suggestions() {
    assert!(AiErrorCategory::Network.suggestion().contains("internet"));
    assert!(AiErrorCategory::Authentication.suggestion().contains("credentials"));
    assert!(AiErrorCategory::RateLimited.suggestion().contains("Wait"));
    assert!(AiErrorCategory::Timeout.suggestion().contains("long"));
  }

  #[test]
  fn test_error_category_from_message() {
    assert_eq!(
      AiErrorCategory::from_error_message("network timeout occurred"),
      AiErrorCategory::Network
    );
    assert_eq!(
      AiErrorCategory::from_error_message("Unauthorized access"),
      AiErrorCategory::Authentication
    );
    assert_eq!(
      AiErrorCategory::from_error_message("Rate limit exceeded"),
      AiErrorCategory::RateLimited
    );
    assert_eq!(
      AiErrorCategory::from_error_message("Request timed out"),
      AiErrorCategory::Timeout
    );
    assert_eq!(
      AiErrorCategory::from_error_message("Some random error"),
      AiErrorCategory::Unknown
    );
  }

  #[test]
  fn test_provider_info_display() {
    let info = AiProviderInfo::new("opencode".to_string(), Some("glm-5".to_string()));
    assert_eq!(info.display_string(), "opencode / glm-5");

    let info_no_model = AiProviderInfo::new("opencode".to_string(), None);
    assert_eq!(info_no_model.display_string(), "opencode");

    let info_with_duration = AiProviderInfo::from_extraction(
      "opencode".to_string(),
      Some("glm-5".to_string()),
      200,
    );
    assert_eq!(info_with_duration.processing_duration_ms, Some(200));
  }

  #[test]
  fn test_request_status_transitions() {
    // Idle is not active and not terminal
    assert!(!AiRequestStatus::Idle.is_active());
    assert!(!AiRequestStatus::Idle.is_terminal());

    // Loading is active but not terminal
    assert!(AiRequestStatus::Loading.is_active());
    assert!(!AiRequestStatus::Loading.is_terminal());

    // Success is not active but is terminal
    assert!(!AiRequestStatus::Success.is_active());
    assert!(AiRequestStatus::Success.is_terminal());

    // Error is not active but is terminal
    assert!(!AiRequestStatus::Error.is_active());
    assert!(AiRequestStatus::Error.is_terminal());
  }
}
