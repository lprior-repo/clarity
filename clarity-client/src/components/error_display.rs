#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(clippy::match_same_arms)]

//! Error display component with retry functionality
//!
//! Provides user-friendly error messages with recovery actions.

use crate::error::{AppError, RecoveryAction};
use dioxus::prelude::*;

/// Error display variant
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ErrorVariant {
  #[default]
  Inline,
  Card,
  Page,
  Banner,
}

/// Error display component properties
#[derive(Clone, Props, PartialEq, Eq)]
pub struct ErrorDisplayProps {
  /// The error to display
  pub error: AppError,
  /// Display variant
  #[props(default)]
  pub variant: ErrorVariant,
  /// Whether to show recovery actions
  #[props(default = true)]
  pub show_actions: bool,
  /// Optional context about what operation failed
  #[props(default)]
  pub context: Option<String>,
}

impl ErrorDisplayProps {
  /// Create a new error display with minimal props
  #[must_use]
  pub fn new(error: AppError) -> Self {
    Self {
      error,
      variant: ErrorVariant::default(),
      show_actions: true,
      context: None,
    }
  }

  /// Set the display variant
  #[must_use]
  pub const fn with_variant(mut self, variant: ErrorVariant) -> Self {
    self.variant = variant;
    self
  }

  /// Set whether to show actions
  #[must_use]
  pub const fn with_show_actions(mut self, show: bool) -> Self {
    self.show_actions = show;
    self
  }

  /// Set additional context
  #[must_use]
  pub fn with_context(mut self, context: String) -> Self {
    self.context = Some(context);
    self
  }
}

/// Error display component
///
/// Shows user-friendly error messages with recovery actions.
/// Automatically determines which actions to show based on error type.
///
/// # Examples
///
/// Basic error display:
/// ```rsx
/// ErrorDisplay {
///     error: AppError::database("Failed to load", "Connection error", true)
/// }
/// ```
///
/// With retry handler:
/// ```rsx
/// ErrorDisplay {
///     error: err,
///     on_retry: Rc::new(|| {
///         // Retry logic
///     })
/// }
/// ```
///
/// With additional context:
/// ```rsx
/// ErrorDisplay {
///     error: err,
///     context: "while loading bead list".to_string()
/// }
/// ```
#[component]
pub fn ErrorDisplay(props: ErrorDisplayProps) -> Element {
  let user_message = props.error.user_message().to_string();
  let recovery_actions = props.error.recovery_actions();
  let is_critical = props.error.is_critical();
  let _can_retry = props.error.can_retry();

  let base_class = match props.variant {
    ErrorVariant::Inline => "error-display-inline",
    ErrorVariant::Card => "error-display-card",
    ErrorVariant::Page => "error-display-page",
    ErrorVariant::Banner => "error-display-banner",
  };

  let icon = if is_critical {
    "error-critical"
  } else {
    "error-warning"
  };

  rsx! {
      div { class: "{base_class}",
          div { class: "error-header",
              span { class: "error-icon {icon}" }
              div { class: "error-content",
                  h3 { class: "error-title", "{user_message}" }
                  {props.context.as_ref().map(|ctx| rsx! {
                          p { class: "error-context", "Error occurred {ctx}" }
                      })}
                  {if is_critical {
                      rsx! {
                          p { class: "error-hint",
                              "If this problem persists, please contact support"
                          }
                      }
                  } else {
                      rsx! {}
                  }}
              }
          }

          {if props.show_actions && !recovery_actions.is_empty() {
              rsx! {
                  div { class: "error-actions",
                      for action in recovery_actions.iter() {
                          ErrorActionButton {
                              action: *action
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

/// Error action button component properties
#[derive(Clone, Props, PartialEq, Eq)]
pub struct ErrorActionButtonProps {
  /// The recovery action to perform
  pub action: RecoveryAction,
}

/// Error action button component
///
/// Renders a single recovery action button.
#[component]
fn ErrorActionButton(props: ErrorActionButtonProps) -> Element {
  let action = props.action;
  let button_class = match action {
    RecoveryAction::Retry => "btn btn-primary",
    RecoveryAction::GoBack => "btn btn-secondary",
    RecoveryAction::GoHome => "btn btn-secondary",
    RecoveryAction::ContactSupport => "btn btn-outline",
  };

  rsx! {
      {match action {
          RecoveryAction::Retry => rsx! {
              button {
                  class: "{button_class}",
                  disabled: true,
                  "{action.label()}"
              }
          },
          RecoveryAction::GoBack => rsx! {
              button {
                  class: "{button_class}",
                  onclick: move |_| {
                      // For now, navigate to home as fallback
                      // TODO: Implement proper back navigation with history stack
                  },
                  "{action.label()}"
              }
          },
          RecoveryAction::GoHome => rsx! {
              crate::app::NavLink {
                  class: "{button_class}",
                  to: crate::Route::Home {},
                  "{action.label()}"
              }
          },
          RecoveryAction::ContactSupport => rsx! {
              a {
                  class: "{button_class}",
                  href: "mailto:support@clarity.app",
                  "{action.label()}"
              }
          },
      }}
  }
}

/// Inline error message component properties
#[derive(Clone, Props, PartialEq, Eq)]
pub struct ErrorInlineProps {
  /// The error message to display
  pub error: String,
  /// Whether the error can be dismissed
  #[props(default)]
  pub can_dismiss: bool,
}

/// Inline error message component
///
/// A compact error display for use within forms or cards.
#[component]
pub fn ErrorInline(props: ErrorInlineProps) -> Element {
  let error = props.error;
  let can_dismiss = props.can_dismiss;
  let mut visible = use_signal(|| true);

  rsx! {
      {if *visible.read() {
          rsx! {
              div { class: "error-inline",
                  span { class: "error-inline-icon", "!" }
                  span { class: "error-inline-message", "{error}" }
                  {if can_dismiss {
                      rsx! {
                          button {
                              class: "error-dismiss",
                              onclick: move |_| {
                                  visible.set(false);
                              },
                              "×"
                          }
                      }
                  } else {
                      rsx! {}
                  }}
              }
          }
      } else {
          rsx! {}
      }}
  }
}

/// Error page component properties
#[derive(Clone, Props, PartialEq, Eq)]
pub struct ErrorPageProps {
  /// The error to display
  pub error: AppError,
}

/// Error page component
///
/// A full-page error display for critical errors.
#[component]
pub fn ErrorPage(props: ErrorPageProps) -> Element {
  let error = props.error;
  rsx! {
      div { class: "error-page",
          div { class: "error-page-content",
              ErrorDisplay {
                  error: error.clone(),
                  variant: ErrorVariant::Page,
                  show_actions: true
              }

              {if error.is_critical() {
                  rsx! {
                      div { class: "error-details-toggle",
                          details {
                              summary { "Technical Details" }
                              pre { class: "error-details", "{error.internal()}" }
                          }
                      }
                  }
              } else {
                  rsx! {}
              }}
          }
      }
  }
}

/// Error banner component properties
#[derive(Clone, Props)]
pub struct ErrorBannerProps {
  /// List of errors to display
  pub errors: Vec<AppError>,
  /// Callback when an error banner is dismissed
  pub on_dismiss: Callback<usize>,
}

// Manual PartialEq for ErrorBannerProps
impl PartialEq for ErrorBannerProps {
  fn eq(&self, other: &Self) -> bool {
    self.errors == other.errors
  }
}

impl Eq for ErrorBannerProps {}

/// Error banner component
///
/// A dismissible banner for non-critical errors at the top of the page.
#[component]
pub fn ErrorBanner(props: ErrorBannerProps) -> Element {
  let errors = props.errors;
  let on_dismiss = props.on_dismiss;
  rsx! {
      div { class: "error-banner-container",
          for (idx, error) in errors.iter().enumerate() {
              ErrorBannerItem {
                  key: "{idx}",
                  error: error.clone(),
                  on_dismiss: on_dismiss,
                  index: idx
              }
          }
      }
  }
}

/// Individual error banner item properties
#[derive(Clone, Props)]
pub struct ErrorBannerItemProps {
  /// The error to display
  pub error: AppError,
  /// Callback when this item is dismissed
  pub on_dismiss: Callback<usize>,
  /// The index of this error
  pub index: usize,
}

// Manual PartialEq for ErrorBannerItemProps
impl PartialEq for ErrorBannerItemProps {
  fn eq(&self, other: &Self) -> bool {
    self.error == other.error && self.index == other.index
  }
}

impl Eq for ErrorBannerItemProps {}

/// Individual error banner item
#[component]
fn ErrorBannerItem(props: ErrorBannerItemProps) -> Element {
  let error = props.error;
  let on_dismiss = props.on_dismiss;
  let index = props.index;
  let mut dismissed = use_signal(|| false);

  rsx! {
      {if *dismissed.read() {
          rsx! {}
      } else {
          rsx! {
              div { class: "error-banner",
                  span { class: "error-banner-icon", "!" }
                  span { class: "error-banner-message",
                      "{error.user_message()}"
                  }
                  button {
                      class: "error-banner-dismiss",
                      onclick: move |_| {
                          dismissed.set(true);
                          on_dismiss.call(index);
                      },
                      "×"
                  }
              }
          }
      }}
  }
}

/// Form error display component properties
#[derive(Clone, Props, PartialEq, Eq)]
pub struct FormErrorProps {
  /// The field name that has errors
  pub field: String,
  /// List of error messages for this field
  pub errors: Vec<String>,
}

/// Form error display component
///
/// Shows field-level errors with contextual help.
#[component]
pub fn FormError(props: FormErrorProps) -> Element {
  let field = props.field;
  let errors = props.errors;
  rsx! {
      div { class: "form-error",
          span { class: "form-error-icon", "⚠" }
          div { class: "form-error-content",
              strong { "{field}" }
              ul { class: "form-error-list",
                  for error in errors.iter() {
                      li { "{error}" }
                  }
              }
          }
      }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_error_variant_default() {
    assert_eq!(ErrorVariant::default(), ErrorVariant::Inline);
  }

  #[test]
  fn test_error_display_props_new() {
    let error = AppError::validation("Test error", "Internal");
    let props = ErrorDisplayProps::new(error.clone());

    assert_eq!(props.error, error);
    assert_eq!(props.variant, ErrorVariant::default());
    assert!(props.show_actions);
    assert!(props.context.is_none());
  }

  #[test]
  fn test_error_display_props_builder() {
    let error = AppError::validation("Test error", "Internal");

    let props = ErrorDisplayProps::new(error)
      .with_variant(ErrorVariant::Page)
      .with_show_actions(false)
      .with_context("during test".to_string());

    assert_eq!(props.variant, ErrorVariant::Page);
    assert!(!props.show_actions);
    assert_eq!(props.context, Some("during test".to_string()));
  }

  #[test]
  fn test_recovery_action_labels() {
    assert_eq!(RecoveryAction::Retry.label(), "Try Again");
    assert_eq!(RecoveryAction::GoBack.label(), "Go Back");
    assert_eq!(RecoveryAction::GoHome.label(), "Go Home");
    assert_eq!(RecoveryAction::ContactSupport.label(), "Contact Support");
  }
}
