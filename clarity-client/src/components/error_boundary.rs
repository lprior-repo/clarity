#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
// Dioxus rsx! macro internally uses unwrap
#![allow(clippy::disallowed_methods)]

//! Error boundary component for catching and displaying errors
//!
//! This module provides a React-like error boundary that catches errors
//! from child components and displays a user-friendly error UI.

use crate::error::{AppError, RecoveryAction};
use dioxus::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

/// Global error state managed at the app root
#[derive(Debug, Clone, PartialEq)]
struct ErrorState {
  current_error: Option<Arc<AppError>>,
  error_count: usize,
}

/// Interior mutable wrapper for error state
#[derive(Debug, Clone)]
struct MutableErrorState {
  inner: Rc<RefCell<ErrorState>>,
}

impl MutableErrorState {
  fn new() -> Self {
    Self {
      inner: Rc::new(RefCell::new(ErrorState::new())),
    }
  }

  fn trigger_error(&self, error: AppError) {
    let mut current = self.inner.borrow_mut();
    *current = current.with_error(error);
  }

  fn clear_error(&self) {
    let mut current = self.inner.borrow_mut();
    *current = current.clear();
  }

  fn has_error(&self) -> bool {
    self.inner.borrow().current_error.is_some()
  }

  fn current_error(&self) -> Option<Arc<AppError>> {
    self.inner.borrow().current_error.clone()
  }

  fn error_count(&self) -> usize {
    self.inner.borrow().error_count
  }
}

impl ErrorState {
  #[must_use]
  const fn new() -> Self {
    Self {
      current_error: None,
      error_count: 0,
    }
  }

  #[must_use]
  fn with_error(&self, error: AppError) -> Self {
    Self {
      current_error: Some(Arc::new(error)),
      error_count: self.error_count + 1,
    }
  }

  #[must_use]
  const fn clear(&self) -> Self {
    Self {
      current_error: None,
      error_count: self.error_count,
    }
  }
}

/// Hook to access the global error handler
///
/// Provides functions to trigger errors and clear error state
#[must_use]
pub fn use_error_handler() -> ErrorHandler {
  let error_state = use_context_provider(MutableErrorState::new);

  ErrorHandler {
    error_state,
  }
}

/// Error handler for triggering and clearing errors
#[derive(Clone, Debug)]
pub struct ErrorHandler {
  error_state: MutableErrorState,
}

impl ErrorHandler {
  /// Trigger an error (will be caught by `ErrorBoundary`)
  pub fn trigger_error(&self, error: AppError) {
    self.error_state.trigger_error(error);
  }

  /// Clear the current error state
  pub fn clear_error(&self) {
    self.error_state.clear_error();
  }

  /// Check if there's an active error
  #[must_use]
  pub fn has_error(&self) -> bool {
    self.error_state.has_error()
  }

  /// Get the current error if any
  #[must_use]
  pub fn current_error(&self) -> Option<Arc<AppError>> {
    self.error_state.current_error()
  }

  /// Get the total error count
  #[must_use]
  pub fn error_count(&self) -> usize {
    self.error_state.error_count()
  }
}

/// Props for the `ErrorBoundary` component
#[derive(Clone, Debug, PartialEq, Props)]
pub struct ErrorBoundaryProps {
  /// The children to render when no error occurs
  children: Element,
  /// Whether to show error details for debugging
  #[props(default = false)]
  show_details: bool,
}

/// Error boundary component that catches errors from children
///
/// This component wraps the entire application and catches any errors
/// that occur during rendering or in event handlers.
///
/// # Example
/// ```rsx
/// ErrorBoundary {
///     show_details: true,
///     RouteProvider {
///         route: current_route,
///         children: App {}
///     }
/// }
/// ```
#[component]
pub fn ErrorBoundary(props: ErrorBoundaryProps) -> Element {
  let error_state = use_context_provider(MutableErrorState::new);

  // Check if there's an active error
  let _has_error = error_state.has_error();
  let error_clone = error_state.current_error();

  if let Some(error) = error_clone {
    // Render fallback UI
    rsx! {
        ErrorFallback {
            error: (*error).clone(),
            show_details: props.show_details,
        }
    }
  } else {
    // Render children normally
    rsx! {
        {props.children}
    }
  }
}

/// Props for the error fallback UI
#[derive(Clone, Debug, PartialEq, Eq, Props)]
pub struct ErrorFallbackProps {
  error: AppError,
  show_details: bool,
}

/// Error fallback UI component
///
/// Displays a user-friendly error page with recovery actions.
#[component]
fn ErrorFallback(props: ErrorFallbackProps) -> Element {
  let (current_route, navigate) = crate::navigation::use_navigation();
  let recovery_actions = props.error.recovery_actions();

  // Create button handlers using navigation
  let route_for_retry = current_route.read().clone();
  let handle_retry = Callback::new(move |_| {
    navigate(route_for_retry.clone());
  });

  let handle_go_home = Callback::new(move |_| {
    navigate("/".to_string());
  });

  let handle_go_back = Callback::new(move |_| {
    navigate("/".to_string());
  });

  let _handle_go_forward = Callback::new(move |_: MouseEvent| {
    navigate("/".to_string());
  });

  let user_message = props.error.user_message().to_string();
  let internal_message = props.error.internal().to_string();

  rsx! {
      div { class: "error-boundary",
          div { class: "error-container",
              // Error icon
              div { class: "error-icon",
                  svg {
                      xmlns: "http://www.w3.org/2000/svg",
                      fill: "none",
                      view_box: "0 0 24 24",
                      stroke: "currentColor",
                      stroke_width: 2,
                      path {
                          d: "M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                      }
                  }
              }

              // Error heading
              h1 { class: "error-title", "Something went wrong" }

              // User-facing error message
              p { class: "error-message",
                  {user_message}
              }

              // Error details (debug mode)
              if props.show_details {
                  div { class: "error-details",
                      h3 { "Technical Details" }
                      pre { class: "error-internal",
                          {internal_message}
                      }

                      if props.error.is_critical() {
                          div { class: "error-badge critical", "Critical Error" }
                      } else {
                          div { class: "error-badge non-critical", "Non-Critical Error" }
                      }

                      if props.error.can_retry() {
                          div { class: "error-badge retryable", "Recoverable - Can Retry" }
                      }
                  }
              }

              // Recovery actions
              div { class: "error-actions",
                  // Suggested actions based on error type
                  for action in recovery_actions.iter() {
                      match action {
                          RecoveryAction::Retry => {
                              rsx! {
                                  button {
                                      class: "error-button primary",
                                      onclick: handle_retry,
                                      "Try Again"
                                  }
                              }
                          }
                          RecoveryAction::GoBack => {
                              rsx! {
                                  button {
                                      class: "error-button secondary",
                                      onclick: handle_go_back,
                                      "Go Back"
                                  }
                              }
                          }
                          RecoveryAction::GoHome => {
                              rsx! {
                                  button {
                                      class: "error-button secondary",
                                      onclick: handle_go_home,
                                      "Go Home"
                                  }
                              }
                          }
                          RecoveryAction::ContactSupport => {
                              rsx! {
                                  a {
                                      class: "error-button link",
                                      href: "mailto:support@clarity.app",
                                      "Contact Support"
                                  }
                              }
                          }
                      }
                  }
              }

              // Additional help section
              div { class: "error-help",
                  p { "Need help? Check our documentation or contact support." }
                  div { class: "error-help-links",
                      a {
                          class: "help-link",
                          href: "https://docs.clarity.app",
                          target: "_blank",
                          "Documentation"
                      }
                      a {
                          class: "help-link",
                          href: "https://github.com/clarity/clarity/issues",
                          target: "_blank",
                          "Report an Issue"
                      }
                  }
              }
          }
      }
  }
}

/// Convenience hook to trigger errors from within components
///
/// # Example
/// ```ignore
/// let trigger_error = use_error_trigger();
///
/// let handle_action = move |_| {
///     match some_fallible_operation() {
///         Ok(_) => {},
///         Err(e) => trigger_error(AppError::from(e)),
///     }
/// };
/// ```
#[must_use]
pub fn use_error_trigger() -> Callback<AppError> {
  let handler = use_error_handler();
  Callback::new(move |error| {
    handler.trigger_error(error);
  })
}

/// Re-export `ErrorFallbackProps` for use in app.rs
pub type ErrorFallbackPropsPublic = ErrorBoundaryProps;

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_error_state_new() {
    let state = ErrorState::new();
    assert!(state.current_error.is_none());
    assert_eq!(state.error_count, 0);
  }

  #[test]
  fn test_error_state_with_error() {
    let state = ErrorState::new();
    let error = AppError::validation("Test error", "Internal");
    let new_state = state.with_error(error);

    assert!(new_state.current_error.is_some());
    assert_eq!(new_state.error_count, 1);
    assert_eq!(
      new_state.current_error.as_ref().map(|e| e.user_message()),
      Some("Test error")
    );
  }

  #[test]
  fn test_error_state_clear() {
    let state = ErrorState::new();
    let error = AppError::validation("Test", "Internal");
    let with_error = state.with_error(error);
    let cleared = with_error.clear();

    assert!(cleared.current_error.is_none());
    assert_eq!(cleared.error_count, 1); // Count preserved
  }

  #[test]
  fn test_recovery_action_labels() {
    assert_eq!(RecoveryAction::Retry.label(), "Try Again");
    assert_eq!(RecoveryAction::GoBack.label(), "Go Back");
    assert_eq!(RecoveryAction::GoHome.label(), "Go Home");
    assert_eq!(RecoveryAction::ContactSupport.label(), "Contact Support");
  }
}
