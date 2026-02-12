#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
// Dioxus rsx! macro internally uses unwrap
#![allow(clippy::disallowed_methods)]

//! Toast notification system for the Clarity desktop application
//!
//! This module provides a toast notification system with:
//! - Multiple toast types (Success, Error, Warning, Info)
//! - Auto-dismiss after 5 seconds
//! - Global state management via `ToastSignal`
//! - Functional patterns with no unwrap/mut on signals
//!
//! # Example
//! ```ignore
//! // In your app root, wrap with ToastProvider
//! ToastProvider {
//!     App {}
//! }
//!
//! // In any component, use the hook
//! let toast = use_toast();
//! toast.success("Saved!", "Your changes have been saved.");
//! toast.error("Error", "Failed to save changes.");
//! ```

use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use std::time::Duration;
use uuid::Uuid;

/// Duration before a toast auto-dismisses (5 seconds)
const TOAST_AUTO_DISMISS_DURATION: Duration = Duration::from_secs(5);

/// Toast type variants
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ToastType {
  /// Success toast (green)
  Success,
  /// Error toast (red)
  Error,
  /// Warning toast (yellow/amber)
  Warning,
  /// Info toast (blue)
  #[default]
  Info,
}

impl ToastType {
  /// Returns the CSS class for this toast type
  #[must_use]
  pub const fn as_class(&self) -> &'static str {
    match self {
      Self::Success => "toast-success",
      Self::Error => "toast-error",
      Self::Warning => "toast-warning",
      Self::Info => "toast-info",
    }
  }

  /// Returns the icon SVG path for this toast type
  #[must_use]
  pub const fn icon_path(&self) -> &'static str {
    match self {
            Self::Success => "M5 13l4 4L19 7",
            Self::Error => "M6 18L18 6M6 6l12 12",
            Self::Warning => "M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z",
            Self::Info => "M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z",
        }
  }
}

/// A single toast notification
#[derive(Clone, Debug, PartialEq)]
#[allow(clippy::derive_partial_eq_without_eq)]
pub struct Toast {
  /// Unique identifier for this toast
  pub id: Uuid,
  /// Type of toast (determines styling)
  pub toast_type: ToastType,
  /// Toast title
  pub title: String,
  /// Toast message body
  pub message: String,
  /// When this toast was created
  pub created_at: DateTime<Utc>,
}

impl Toast {
  /// Creates a new toast with the given type, title, and message
  #[must_use]
  pub fn new(toast_type: ToastType, title: impl Into<String>, message: impl Into<String>) -> Self {
    Self {
      id: Uuid::new_v4(),
      toast_type,
      title: title.into(),
      message: message.into(),
      created_at: Utc::now(),
    }
  }

  /// Creates a success toast
  #[must_use]
  pub fn success(title: impl Into<String>, message: impl Into<String>) -> Self {
    Self::new(ToastType::Success, title, message)
  }

  /// Creates an error toast
  #[must_use]
  pub fn error(title: impl Into<String>, message: impl Into<String>) -> Self {
    Self::new(ToastType::Error, title, message)
  }

  /// Creates a warning toast
  #[must_use]
  pub fn warning(title: impl Into<String>, message: impl Into<String>) -> Self {
    Self::new(ToastType::Warning, title, message)
  }

  /// Creates an info toast
  #[must_use]
  pub fn info(title: impl Into<String>, message: impl Into<String>) -> Self {
    Self::new(ToastType::Info, title, message)
  }
}

/// Global toast state signal type
pub type ToastSignal = Signal<Vec<Toast>>;

/// Controller for managing toast notifications
///
/// This controller wraps a Signal<Vec<Toast>> and provides
/// methods for showing and dismissing toasts. The Signal
/// uses interior mutability, so methods take `&self`.
#[derive(Clone, Copy, Debug)]
pub struct ToastController {
  toasts: ToastSignal,
}

impl ToastController {
  /// Creates a new `ToastController` with the given signal
  #[must_use]
  pub const fn new(toasts: ToastSignal) -> Self {
    Self { toasts }
  }

  /// Show a custom toast
  pub fn show(&self, toast_type: ToastType, title: impl Into<String>, message: impl Into<String>) {
    let toast = Toast::new(toast_type, title, message);
    self.add_toast(toast);
  }

  /// Show a success toast
  pub fn success(&self, title: impl Into<String>, message: impl Into<String>) {
    let toast = Toast::success(title, message);
    self.add_toast(toast);
  }

  /// Show an error toast
  pub fn error(&self, title: impl Into<String>, message: impl Into<String>) {
    let toast = Toast::error(title, message);
    self.add_toast(toast);
  }

  /// Show a warning toast
  pub fn warning(&self, title: impl Into<String>, message: impl Into<String>) {
    let toast = Toast::warning(title, message);
    self.add_toast(toast);
  }

  /// Show an info toast
  pub fn info(&self, title: impl Into<String>, message: impl Into<String>) {
    let toast = Toast::info(title, message);
    self.add_toast(toast);
  }

  /// Dismiss a specific toast by ID
  pub fn dismiss(&self, id: Uuid) {
    // Signal is Copy, so we can copy it and modify
    let mut signal = self.toasts;
    signal.write().retain(|t| t.id != id);
  }

  /// Dismiss all toasts
  pub fn dismiss_all(&self) {
    // Signal is Copy, so we can copy it and modify
    let mut signal = self.toasts;
    signal.write().clear();
  }

  /// Add a toast to the stack
  fn add_toast(&self, toast: Toast) {
    // Signal is Copy, so we can copy it and modify
    let mut signal = self.toasts;
    signal.write().push(toast);
  }

  /// Get the current number of toasts
  #[must_use]
  pub fn len(&self) -> usize {
    self.toasts.read().len()
  }

  /// Check if there are no toasts
  #[must_use]
  pub fn is_empty(&self) -> bool {
    self.toasts.read().is_empty()
  }
}

/// Hook to access the toast controller
///
/// This hook provides access to the global toast state and methods
/// to show and dismiss toast notifications.
///
/// # Panics
/// This hook will panic if used outside of a `ToastProvider` context.
#[must_use]
pub fn use_toast() -> ToastController {
  use_context::<ToastController>()
}

/// Props for `ToastProvider` component
#[derive(Clone, Debug, PartialEq, Props)]
pub struct ToastProviderProps {
  /// Child components
  children: Element,
}

/// Provider component that wraps the app and provides toast state
///
/// This component must be placed at the root of your app (or near it)
/// to enable toast functionality throughout the component tree.
///
/// # Example
/// ```ignore
/// ToastProvider {
///     Router {
///         App {}
///     }
/// }
/// ```
#[component]
pub fn ToastProvider(props: ToastProviderProps) -> Element {
  // Initialize global toast state
  let toasts = use_signal(Vec::new);
  let controller = use_hook(|| ToastController::new(toasts));

  // Provide the controller to all child components
  use_context_provider(|| controller);

  rsx! {
      {props.children}

      // Render the toast container
      ToastContainer {
          toasts: toasts
      }
  }
}

/// Props for `ToastContainer` component
#[derive(Clone, Copy, Debug, PartialEq, Eq, Props)]
pub struct ToastContainerProps {
  /// The toast signal to render
  toasts: ToastSignal,
}

/// Container component that renders all active toasts
///
/// This component is automatically included in `ToastProvider`.
/// It positions toasts in the top-right corner of the screen.
#[component]
pub fn ToastContainer(props: ToastContainerProps) -> Element {
  let toasts = props.toasts.read();

  rsx! {
      div { class: "toast-container",
          for toast in toasts.iter() {
              ToastItem {
                  key: "{toast.id}",
                  toast: toast.clone(),
                  toasts: props.toasts
              }
          }
      }
  }
}

/// Props for `ToastItem` component
#[derive(Clone, Debug, PartialEq, Props)]
pub struct ToastItemProps {
  /// The toast to display
  toast: Toast,
  /// Reference to the toast signal for dismissal
  toasts: ToastSignal,
}

/// Individual toast item component
///
/// Displays a single toast notification with:
/// - Icon based on toast type
/// - Title and message
/// - Dismiss button
/// - Auto-dismiss after 5 seconds
#[component]
pub fn ToastItem(props: ToastItemProps) -> Element {
  let toast_id = props.toast.id;
  let toast_type = props.toast.toast_type;
  let title = props.toast.title.clone();
  let message = props.toast.message.clone();
  let icon_path = toast_type.icon_path();
  let class = toast_type.as_class();

  // Set up auto-dismiss - use shadowing with mut for the signal
  let mut toasts_for_spawn = props.toasts;
  use_effect(move || {
    spawn(async move {
      tokio::time::sleep(TOAST_AUTO_DISMISS_DURATION).await;
      toasts_for_spawn.write().retain(|t| t.id != toast_id);
    });
  });

  // Dismiss handler - use shadowing with mut for the signal
  let mut toasts_for_dismiss = props.toasts;
  let handle_dismiss = move |_| {
    toasts_for_dismiss.write().retain(|t| t.id != toast_id);
  };

  rsx! {
      div { class: "toast-item {class}",
          div { class: "toast-icon",
              svg {
                  xmlns: "http://www.w3.org/2000/svg",
                  fill: "none",
                  view_box: "0 0 24 24",
                  stroke: "currentColor",
                  stroke_width: 2,
                  path {
                      d: "{icon_path}"
                  }
              }
          }
          div { class: "toast-content",
              div { class: "toast-title", "{title}" }
              div { class: "toast-message", "{message}" }
          }
          button {
              class: "toast-dismiss",
              onclick: handle_dismiss,
              svg {
                  xmlns: "http://www.w3.org/2000/svg",
                  fill: "none",
                  view_box: "0 0 24 24",
                  stroke: "currentColor",
                  stroke_width: 2,
                  path {
                      d: "M6 18L18 6M6 6l12 12"
                  }
              }
          }
      }
  }
}

/// CSS styles for the toast notification system
///
/// Include these styles in your application's CSS or use the `toast_styles()` function
/// to get a CSS string that can be injected.
#[must_use]
pub const fn toast_styles() -> &'static str {
  "
/* Toast Container - positioned top-right */
.toast-container {
    position: fixed;
    top: 1rem;
    right: 1rem;
    z-index: 9999;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    max-width: 400px;
    pointer-events: none;
}

/* Toast Item Base Styles */
.toast-item {
    display: flex;
    align-items: flex-start;
    gap: 0.75rem;
    padding: 1rem;
    border-radius: 0.5rem;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    pointer-events: auto;
    animation: toast-slide-in 0.3s ease-out;
    min-width: 300px;
}

@keyframes toast-slide-in {
    from {
        transform: translateX(100%);
        opacity: 0;
    }
    to {
        transform: translateX(0);
        opacity: 1;
    }
}

/* Toast Icon */
.toast-icon {
    flex-shrink: 0;
    width: 1.5rem;
    height: 1.5rem;
}

.toast-icon svg {
    width: 100%;
    height: 100%;
}

/* Toast Content */
.toast-content {
    flex: 1;
    min-width: 0;
}

.toast-title {
    font-weight: 600;
    font-size: 0.875rem;
    margin-bottom: 0.25rem;
}

.toast-message {
    font-size: 0.8125rem;
    opacity: 0.9;
    word-wrap: break-word;
}

/* Toast Dismiss Button */
.toast-dismiss {
    flex-shrink: 0;
    width: 1.25rem;
    height: 1.25rem;
    padding: 0;
    background: transparent;
    border: none;
    cursor: pointer;
    opacity: 0.5;
    transition: opacity 0.2s;
}

.toast-dismiss:hover {
    opacity: 1;
}

.toast-dismiss svg {
    width: 100%;
    height: 100%;
}

/* Toast Type Variants */
.toast-success {
    background-color: #10b981;
    color: white;
}

.toast-error {
    background-color: #ef4444;
    color: white;
}

.toast-warning {
    background-color: #f59e0b;
    color: white;
}

.toast-info {
    background-color: #3b82f6;
    color: white;
}
"
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_toast_type_as_class() {
    assert_eq!(ToastType::Success.as_class(), "toast-success");
    assert_eq!(ToastType::Error.as_class(), "toast-error");
    assert_eq!(ToastType::Warning.as_class(), "toast-warning");
    assert_eq!(ToastType::Info.as_class(), "toast-info");
  }

  #[test]
  fn test_toast_type_default() {
    assert_eq!(ToastType::default(), ToastType::Info);
  }

  #[test]
  fn test_toast_new() {
    let toast = Toast::new(ToastType::Success, "Test Title", "Test Message");

    assert!(!toast.id.is_nil());
    assert_eq!(toast.toast_type, ToastType::Success);
    assert_eq!(toast.title, "Test Title");
    assert_eq!(toast.message, "Test Message");
  }

  #[test]
  fn test_toast_success() {
    let toast = Toast::success("Success!", "Operation completed.");

    assert_eq!(toast.toast_type, ToastType::Success);
    assert_eq!(toast.title, "Success!");
    assert_eq!(toast.message, "Operation completed.");
  }

  #[test]
  fn test_toast_error() {
    let toast = Toast::error("Error!", "Something went wrong.");

    assert_eq!(toast.toast_type, ToastType::Error);
    assert_eq!(toast.title, "Error!");
    assert_eq!(toast.message, "Something went wrong.");
  }

  #[test]
  fn test_toast_warning() {
    let toast = Toast::warning("Warning!", "Please check your input.");

    assert_eq!(toast.toast_type, ToastType::Warning);
    assert_eq!(toast.title, "Warning!");
    assert_eq!(toast.message, "Please check your input.");
  }

  #[test]
  fn test_toast_info() {
    let toast = Toast::info("Info", "Here is some information.");

    assert_eq!(toast.toast_type, ToastType::Info);
    assert_eq!(toast.title, "Info");
    assert_eq!(toast.message, "Here is some information.");
  }

  #[test]
  fn test_toast_unique_ids() {
    let toast1 = Toast::info("A", "B");
    let toast2 = Toast::info("C", "D");

    // Each toast should have a unique ID
    assert_ne!(toast1.id, toast2.id);
  }

  #[test]
  fn test_toast_styles_returns_string() {
    let styles = toast_styles();

    assert!(styles.contains(".toast-container"));
    assert!(styles.contains(".toast-item"));
    assert!(styles.contains(".toast-success"));
    assert!(styles.contains(".toast-error"));
    assert!(styles.contains(".toast-warning"));
    assert!(styles.contains(".toast-info"));
  }

  #[test]
  fn test_toast_controller_type_check() {
    // This test just ensures the ToastController compiles with the correct types
    // Actual testing would require a Dioxus runtime
    let _ = || {
      fn component() -> Element {
        let toast = use_toast();
        toast.success("Title", "Message");
        toast.error("Title", "Message");
        toast.warning("Title", "Message");
        toast.info("Title", "Message");
        toast.dismiss(Uuid::nil());
        toast.dismiss_all();
        let _len = toast.len();
        let _empty = toast.is_empty();
        rsx! { div {} }
      }
      let _ = component;
    };
  }
}
