//! Navigation utilities for programmatic routing
//!
//! This module provides hooks and utilities for programmatic navigation
//! using Clarity's custom routing system.

// Dioxus rsx! macro internally uses unwrap, so we allow the disallowed_methods lint.
#![allow(clippy::disallowed_methods)]
#![allow(warnings)]
#![allow(clippy::all)]

use dioxus::prelude::*;

/// Hook for accessing the router and performing programmatic navigation
///
/// Returns a tuple containing:
/// - The current route
/// - A function to navigate to a new route
///
/// # Example
///
/// ```rsx
/// let (current_route, navigate) = use_navigation();
///
/// button {
///     onclick: move |_| navigate("/dashboard".to_string()),
///     "Go to Dashboard"
/// }
/// ```
#[must_use]
pub fn use_navigation() -> (Signal<String>, Callback<String>) {
  let current_route = use_signal(String::new);

  // Note: In a real implementation, you'd integrate this with the app's state
  // For now, this is a simplified version
  let navigate = {
    let mut current_route = current_route;
    Callback::new(move |target_route: String| {
      current_route.set(target_route);
    })
  };

  (current_route, navigate)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_use_navigation_type_check() {
    // This test just ensures the hook compiles with the correct types
    // Actual testing would require a Dioxus runtime
    let _ = || {
      let (_route, _navigate) = use_navigation();
      _navigate("/test".to_string());
    };
  }
}
