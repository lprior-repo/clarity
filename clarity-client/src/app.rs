//! Main Dioxus application component
//!
//! This module contains the root App component and its supporting functionality.

use dioxus::prelude::*;
use std::result::Result;

/// Application state that manages shared data across components
#[derive(Clone, Debug, PartialEq)]
pub struct AppState {
    /// Current route path
    pub current_route: String,
    /// Application error state, if any
    pub error: Option<AppError>,
}

impl AppState {
    /// Create a new application state with default values
    #[must_use]
    pub const fn new() -> Self {
        Self {
            current_route: String::new(),
            error: None,
        }
    }

    /// Navigate to a new route
    ///
    /// # Errors
    /// Returns an error if the route path is invalid
    pub fn navigate_to(&mut self, path: String) -> Result<(), AppError> {
        if path.is_empty() {
            return Err(AppError::InvalidRoute("Route path cannot be empty".to_string()));
        }

        if !path.starts_with('/') {
            return Err(AppError::InvalidRoute(format!(
                "Route path must start with '/', got: {}",
                path
            )));
        }

        self.current_route = path;
        Ok(())
    }

    /// Set an application error
    pub fn set_error(&mut self, error: AppError) {
        self.error = Some(error);
    }

    /// Clear any application error
    pub fn clear_error(&mut self) {
        self.error = None;
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Application-specific errors
#[derive(Clone, Debug, PartialEq)]
pub enum AppError {
    /// Invalid route path
    InvalidRoute(String),
    /// Component initialization error
    ComponentInit(String),
    /// State update error
    StateUpdate(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidRoute(msg) => write!(f, "Invalid route: {}", msg),
            Self::ComponentInit(msg) => write!(f, "Component initialization failed: {}", msg),
            Self::StateUpdate(msg) => write!(f, "State update failed: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

/// Main application component
///
/// This is the root component that manages routing and global application state.
#[component]
pub fn App() -> Element {
    // Initialize application state
    let mut state = use_signal(AppState::new);

    rsx! {
        div { class: "app-container",
            h1 { "Clarity" }
            div { class: "content",
                match state.read().current_route.as_str() {
                    "" => rsx! {
                        HomePage {}
                    },
                    "/about" => rsx! {
                        AboutPage {}
                    },
                    path => rsx! {
                        NotFoundPage { path: path.to_string() }
                    },
                }
            }
            // Display error if present
            if let Some(ref error) = state.read().error {
                div { class: "error-banner",
                    {error.to_string()}
                }
            }
        }
    }
}

/// Home page component
#[component]
fn HomePage() -> Element {
    rsx! {
        div { class: "home-page",
            h2 { "Welcome to Clarity" }
            p { "A modern web application built with Dioxus" }
            Link { to: "/about", text: "Learn More" }
        }
    }
}

/// About page component
#[component]
fn AboutPage() -> Element {
    rsx! {
        div { class: "about-page",
            h2 { "About Clarity" }
            p { "Clarity is a web application for managing interviews and documentation." }
            Link { to: "/", text: "Back Home" }
        }
    }
}

/// 404 Not Found page component
#[component]
fn NotFoundPage(path: String) -> Element {
    rsx! {
        div { class: "not-found-page",
            h2 { "404 - Page Not Found" }
            p { "The page '{path}' does not exist." }
            Link { to: "/", text: "Go Home" }
        }
    }
}

/// Navigation link component
#[component]
fn Link(to: String, text: String) -> Element {
    rsx! {
        a {
            href: "{to}",
            class: "nav-link",
            "{text}"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_state_new() {
        let state = AppState::new();
        assert_eq!(state.current_route, "");
        assert!(state.error.is_none());
    }

    #[test]
    fn test_app_state_default() {
        let state = AppState::default();
        assert_eq!(state.current_route, "");
        assert!(state.error.is_none());
    }

    #[test]
    fn test_navigate_to_valid_route() {
        let mut state = AppState::new();
        let result = state.navigate_to("/about".to_string());
        assert!(result.is_ok(), "Navigation should succeed for valid route");
        assert_eq!(state.current_route, "/about");
    }

    #[test]
    fn test_navigate_to_empty_route_fails() {
        let mut state = AppState::new();
        let result = state.navigate_to("".to_string());
        assert!(result.is_err(), "Navigation should fail for empty route");
        assert!(matches!(
            result,
            Err(AppError::InvalidRoute(_))
        ));
    }

    #[test]
    fn test_navigate_to_route_without_leading_slash_fails() {
        let mut state = AppState::new();
        let result = state.navigate_to("about".to_string());
        assert!(result.is_err(), "Navigation should fail for route without leading slash");
        assert!(matches!(
            result,
            Err(AppError::InvalidRoute(_))
        ));
    }

    #[test]
    fn test_set_and_clear_error() {
        let mut state = AppState::new();
        assert!(state.error.is_none());

        state.set_error(AppError::ComponentInit("Test error".to_string()));
        assert!(state.error.is_some());
        assert_eq!(
            state.error,
            Some(AppError::ComponentInit("Test error".to_string()))
        );

        state.clear_error();
        assert!(state.error.is_none());
    }

    #[test]
    fn test_app_error_display() {
        let err = AppError::InvalidRoute("test error".to_string());
        assert_eq!(err.to_string(), "Invalid route: test error");

        let err = AppError::ComponentInit("init failed".to_string());
        assert_eq!(err.to_string(), "Component initialization failed: init failed");

        let err = AppError::StateUpdate("update failed".to_string());
        assert_eq!(err.to_string(), "State update failed: update failed");
    }

    #[test]
    fn test_app_state_multiple_navigations() {
        let mut state = AppState::new();

        // First navigation
        let result = state.navigate_to("/about".to_string());
        assert!(result.is_ok());
        assert_eq!(state.current_route, "/about");

        // Second navigation
        let result = state.navigate_to("/".to_string());
        assert!(result.is_ok());
        assert_eq!(state.current_route, "/");

        // Invalid navigation
        let result = state.navigate_to("invalid".to_string());
        assert!(result.is_err());
        // State should remain unchanged after failed navigation
        assert_eq!(state.current_route, "/");
    }
}
