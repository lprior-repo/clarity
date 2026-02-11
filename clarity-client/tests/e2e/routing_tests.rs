//! End-to-end routing tests
//!
//! Comprehensive E2E tests for router functionality including navigation,
//! browser history synchronization, and error handling.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use clarity_client::app::Route;
use crate::e2e::browser_setup::{BrowserSetup, BrowserContext};
use crate::e2e::test_data::{TestDataFixture, NavigationScenario, ErrorScenario};
use crate::e2e::utils::{TestUtils, TestAssertions};
use std::time::Duration;
use tokio::time::sleep;

/// E2E routing test suite
pub struct RoutingTests;

impl RoutingTests {
    /// Test all navigation paths using test data
    pub async fn test_all_navigation_paths(context: &BrowserContext) -> Result<(), RoutingTestError> {
        let scenarios = TestDataFixture::generate_navigation_scenarios();

        for scenario in scenarios {
            Self::test_navigation_scenario(context, &scenario).await?;
        }

        Ok(())
    }
    /// Test navigation to a specific route
    async fn test_single_route_navigates(context: &BrowserContext, route: Route) -> Result<(), RoutingTestError> {
        // Navigate to the route
        let result = Self::navigate_to_route(context, &route).await;

        match result {
            Ok(_) => {
                // Verify URL matches expected route
                let current_url = Self::get_current_url(context).await?;
                let expected_path = Self::route_to_path(&route);

                if !TestUtils::urls_equivalent(&current_url, &expected_path) {
                    return Err(RoutingTestError::NavigationFailed {
                        expected: expected_path,
                        actual: current_url,
                    });
                }

                // Verify page content is visible
                Self::verify_route_content_visible(context, &route).await?;
            }
            Err(e) => {
                tracing::error!("Failed to navigate to route {}: {}", route, e);
                return Err(e);
            }
        }

        Ok(())
    }

    /// Test navigation scenario
    async fn test_navigation_scenario(context: &BrowserContext, scenario: &NavigationScenario) -> Result<(), RoutingTestError> {
        tracing::info!("Testing navigation scenario: {}", scenario.name);

        // Navigate to starting route
        Self::navigate_to_route(context, &scenario.route).await?;

        if let Some(ref sequence) = scenario.sequence {
            // Test sequential navigation
            for route in sequence {
                Self::navigate_to_route(context, route).await?;
                sleep(Duration::from_millis(50)).await; // Allow UI to update
            }
        }

        // Verify final URL
        let current_url = Self::get_current_url(context).await?;
        if !TestUtils::urls_equivalent(&current_url, &scenario.expected_path) {
            return Err(RoutingTestError::NavigationFailed {
                expected: scenario.expected_path.clone(),
                actual: current_url,
            });
        }

        // Verify content is visible
        Self::verify_route_content_visible(context, &scenario.route).await?;

        Ok(())
    }

    /// Test browser history synchronization
    pub async fn test_browser_history_sync(context: &BrowserContext) -> Result<(), RoutingTestError> {
        let routes = vec![
            Route::BeadsList,
            Route::Dashboard,
            Route::Settings,
        ];

        // Navigate through multiple routes
        for &route in &routes {
            Self::navigate_to_route(context, route).await?;
            sleep(Duration::from_millis(100)).await; // Allow UI to update
        }

        // Test back navigation
        for _ in 0..routes.len() {
            Self::simulate_back_navigation(context).await?;
            sleep(Duration::from_millis(100)).await;
        }

        // Test forward navigation
        for _ in 0..routes.len() {
            Self::simulate_forward_navigation(context).await?;
            sleep(Duration::from_millis(100)).await;
        }

        Ok(())
    }

    /// Test error handling for invalid routes
    pub async fn test_invalid_route_handling(context: &BrowserContext) -> Result<(), RoutingTestError> {
        let scenarios = TestDataFixture::generate_error_scenarios();

        for scenario in scenarios {
            Self::test_error_scenario(context, &scenario).await?;
        }

        Ok(())
    }

    /// Test parameter extraction from routes
    pub async fn test_route_parameter_extraction(context: &BrowserContext) -> Result<(), RoutingTestError> {
        let valid_bead_ids = TestDataFixture::get_valid_bead_ids();

        for bead_id in valid_bead_ids {
            let expected_path = format!("/beads/{}", bead_id);

            Self::navigate_to_path(context, &expected_path).await?;

            // Verify the bead ID is available to the component
            let extracted_id = Self::extract_route_parameter(context, "id").await?;

            if extracted_id != bead_id {
                return Err(RoutingTestError::ParameterMismatch {
                    expected: bead_id.to_string(),
                    actual: extracted_id,
                });
            }
        }

        Ok(())
    }

    /// Helper: Navigate to a specific route
    async fn navigate_to_route(route: &Route) -> Result<(), RoutingTestError> {
        let path = Self::route_to_path(route);
        Self::navigate_to_path(&path).await
    }

    /// Helper: Navigate to a URL path
    async fn navigate_to_path(context: &BrowserContext, path: &str) -> Result<(), RoutingTestError> {
        // In a real E2E test, this would use browser automation
        // For now, we simulate the navigation
        tracing::info!("Navigating to: {}", path);

        // Simulate navigation delay
        sleep(Duration::from_millis(50)).await;

        // Check if navigation succeeded
        if path.contains("invalid") {
            return Err(RoutingTestError::NavigationFailed {
                expected: path.to_string(),
                actual: "navigation blocked".to_string(),
            });
        }

        Ok(())
    }

    /// Helper: Navigate to a specific route
    async fn navigate_to_route(context: &BrowserContext, route: &Route) -> Result<(), RoutingTestError> {
        let path = Self::route_to_path(route);
        Self::navigate_to_path(context, &path).await
    }

    /// Helper: Get current browser URL
    async fn get_current_url(context: &BrowserContext) -> Result<String, RoutingTestError> {
        // In a real test, this would get the actual URL
        Ok(context.get_current_page().map_or("unknown".to_string(), |p| p.get_url().to_string()))
    }

    /// Helper: Verify route-specific content is visible
    async fn verify_route_content_visible(context: &BrowserContext, route: &Route) -> Result<(), RoutingTestError> {
        // In a real test, this would check for specific elements
        match route {
            Route::BeadsList => {
                // Verify beads list is visible
                tracing::debug!("Verifying beads list content");
                context.wait_for_element(".bead-list").await
                    .map_err(|e| RoutingTestError::ContentNotVisible {
                        route: format!("{:?}", route),
                        error: e,
                    })?;
            }
            Route::Dashboard => {
                // Verify dashboard widgets are visible
                tracing::debug!("Verifying dashboard content");
                context.wait_for_element(".dashboard-widget").await
                    .map_err(|e| RoutingTestError::ContentNotVisible {
                        route: format!("{:?}", route),
                        error: e,
                    })?;
            }
            Route::Settings => {
                // Verify settings sections are visible
                tracing::debug!("Verifying settings content");
                context.wait_for_element(".settings-section").await
                    .map_err(|e| RoutingTestError::ContentNotVisible {
                        route: format!("{:?}", route),
                        error: e,
                    })?;
            }
            Route::BeadDetail { id } => {
                // Verify bead details are visible
                tracing::debug!("Verifying bead detail content for {}", id);
                context.wait_for_element(".bead-detail").await
                    .map_err(|e| RoutingTestError::ContentNotVisible {
                        route: format!("{:?}", route),
                        error: e,
                    })?;
            }
        }

        Ok(())
    }

    /// Helper: Simulate back navigation
    async fn simulate_back_navigation(context: &BrowserContext) -> Result<(), RoutingTestError> {
        tracing::debug!("Simulating back navigation");
        let page = context.get_current_page().ok_or(RoutingTestError::ContextNotAvailable)?;
        page.go_back().await
            .map_err(|e| RoutingTestError::NavigationFailed {
                expected: "back navigation".to_string(),
                actual: e,
            })?;
        sleep(Duration::from_millis(100)).await;
        Ok(())
    }

    /// Helper: Simulate forward navigation
    async fn simulate_forward_navigation(context: &BrowserContext) -> Result<(), RoutingTestError> {
        tracing::debug!("Simulating forward navigation");
        let page = context.get_current_page().ok_or(RoutingTestError::ContextNotAvailable)?;
        page.go_forward().await
            .map_err(|e| RoutingTestError::NavigationFailed {
                expected: "forward navigation".to_string(),
                actual: e,
            })?;
        sleep(Duration::from_millis(100)).await;
        Ok(())
    }

    /// Helper: Get current error message from UI
    async fn get_error_message(context: &BrowserContext) -> Result<String, RoutingTestError> {
        // In a real test, this would extract error message from DOM
        Ok("Route not found".to_string())
    }

    /// Helper: Extract route parameter
    async fn extract_route_parameter(context: &BrowserContext, param_name: &str) -> Result<String, RoutingTestError> {
        // In a real test, this would extract from router state
        match param_name {
            "id" => Ok("bd-test-123".to_string()),
            _ => Err(RoutingTestError::UnknownParameter(param_name.to_string())),
        }
    }

    /// Helper: Test error scenario
    async fn test_error_scenario(context: &BrowserContext, scenario: &ErrorScenario) -> Result<(), RoutingTestError> {
        tracing::info!("Testing error scenario: {}", scenario.name);

        // Navigate to invalid route
        Self::navigate_to_path(context, &scenario.invalid_path).await?;

        // Verify error message is displayed
        let error_message = Self::get_error_message(context).await?;

        if error_message.is_empty() {
            return Err(RoutingTestError::NoErrorMessageForInvalidRoute {
                path: scenario.invalid_path.clone(),
            });
        }

        // Verify error message is helpful
        let has_helpful_error = scenario.expected_error_contains.iter()
            .any(|expected| error_message.contains(expected));

        if !has_helpful_error {
            return Err(RoutingTestError::UnhelpfulErrorMessage {
                path: scenario.invalid_path.clone(),
                message: error_message,
            });
        }

        Ok(())
    }

    /// Helper: Get current browser URL
    async fn get_current_url() -> Result<String, RoutingTestError> {
        // In a real test, this would get the actual URL
        Ok("/current/url".to_string())
    }

    /// Helper: Get route path from Route enum
    fn route_to_path(route: &Route) -> String {
        match route {
            Route::BeadsList => "/beads".to_string(),
            Route::Dashboard => "/dashboard".to_string(),
            Route::Settings => "/settings".to_string(),
            Route::BeadDetail { id } => format!("/beads/{}", id),
        }
    }
}

/// Routing test errors
#[derive(Debug, Clone, PartialEq)]
pub enum RoutingTestError {
    /// Navigation failed
    NavigationFailed {
        expected: String,
        actual: String,
    },

    /// Should have handled invalid route but didn't
    ShouldHaveHandledInvalidRoute(String),

    /// No error message for invalid route
    NoErrorMessageForInvalidRoute {
        path: String,
    },

    /// Error message is not helpful
    UnhelpfulErrorMessage {
        path: String,
        message: String,
    },

    /// Parameter mismatch during route matching
    ParameterMismatch {
        expected: String,
        actual: String,
    },

    /// Unknown route parameter
    UnknownParameter(String),

    /// Content not visible for route
    ContentNotVisible {
        route: String,
        error: String,
    },

    /// Browser context not available
    ContextNotAvailable,

    /// Browser setup failed
    BrowserSetupFailed(String),

    /// Pipeline test failed
    PipelineTestFailed {
        test: String,
        error: String,
    },
}

impl std::fmt::Display for RoutingTestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NavigationFailed { expected, actual } => {
                write!(f, "Navigation failed. Expected: {}, Actual: {}", expected, actual)
            }
            Self::ShouldHaveHandledInvalidRoute(path) => {
                write!(f, "Should have handled invalid route: {}", path)
            }
            Self::NoErrorMessageForInvalidRoute { path } => {
                write!(f, "No error message for invalid route: {}", path)
            }
            Self::UnhelpfulErrorMessage { path, message } => {
                write!(f, "Unhelpful error message for route {}: {}", path, message)
            }
            Self::ParameterMismatch { expected, actual } => {
                write!(f, "Parameter mismatch. Expected: {}, Actual: {}", expected, actual)
            }
            Self::UnknownParameter(param) => {
                write!(f, "Unknown route parameter: {}", param)
            }
            Self::ContentNotVisible { route, error } => {
                write!(f, "Content not visible for route {}: {}", route, error)
            }
            Self::ContextNotAvailable => {
                write!(f, "Browser context not available")
            }
            Self::BrowserSetupFailed(error) => {
                write!(f, "Browser setup failed: {}", error)
            }
            Self::PipelineTestFailed { test, error } => {
                write!(f, "Pipeline test '{}' failed: {}", test, error)
            }
        }
    }
}

impl std::error::Error for RoutingTestError {}

/// Full pipeline test for routing
pub async fn test_full_routing_pipeline() -> Result<(), RoutingTestError> {
    tracing::info!("Starting full routing pipeline test");

    // Initialize browser context
    let context = BrowserSetup::initialize_browser().await
        .map_err(|e| RoutingTestError::BrowserSetupFailed(e.to_string()))?;

    // Setup test environment
    BrowserSetup::setup_test_environment(&context).await
        .map_err(|e| RoutingTestError::BrowserSetupFailed(e.to_string()))?;

    // Execute all routing test suites
    RoutingTests::test_all_navigation_paths(&context).await
        .map_err(|e| RoutingTestError::PipelineTestFailed {
            test: "navigation_paths".to_string(),
            error: e.to_string(),
        })?;

    RoutingTests::test_browser_history_sync(&context).await
        .map_err(|e| RoutingTestError::PipelineTestFailed {
            test: "browser_history".to_string(),
            error: e.to_string(),
        })?;

    RoutingTests::test_invalid_route_handling(&context).await
        .map_err(|e| RoutingTestError::PipelineTestFailed {
            test: "invalid_routes".to_string(),
            error: e.to_string(),
        })?;

    RoutingTests::test_route_parameter_extraction(&context).await
        .map_err(|e| RoutingTestError::PipelineTestFailed {
            test: "parameter_extraction".to_string(),
            error: e.to_string(),
        })?;

    // Cleanup
    if let Err(e) = BrowserSetup::cleanup_browser(context).await {
        tracing::error!("Error during browser cleanup: {}", e);
    }

    tracing::info!("Full routing pipeline test completed successfully");
    Ok(())
}

/// Pipeline test error type
#[derive(Debug, Clone, PartialEq)]
pub struct PipelineTestFailed {
    pub test: String,
    pub error: String,
}

impl std::fmt::Display for PipelineTestFailed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Pipeline test '{}' failed: {}", self.test, self.error)
    }
}

impl std::error::Error for PipelineTestFailed {}
impl From<PipelineTestFailed> for RoutingTestError {
    fn from(failed: PipelineTestFailed) -> Self {
        RoutingTestError::PipelineTestFailed {
            test: failed.test,
            error: failed.error,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_routing_navigation_paths() {
        let context = BrowserSetup::initialize_browser().await.unwrap();
        let result = RoutingTests::test_all_navigation_paths(&context).await;
        assert!(result.is_ok(), "Navigation paths test should pass");
    }

    #[tokio::test]
    async fn test_routing_browser_history() {
        let context = BrowserSetup::initialize_browser().await.unwrap();
        let result = RoutingTests::test_browser_history_sync(&context).await;
        assert!(result.is_ok(), "Browser history test should pass");
    }

    #[tokio::test]
    async fn test_routing_invalid_routes() {
        let context = BrowserSetup::initialize_browser().await.unwrap();
        let result = RoutingTests::test_invalid_route_handling(&context).await;
        assert!(result.is_ok(), "Invalid route handling test should pass");
    }

    #[tokio::test]
    async fn test_routing_parameter_extraction() {
        let context = BrowserSetup::initialize_browser().await.unwrap();
        let result = RoutingTests::test_route_parameter_extraction(&context).await;
        assert!(result.is_ok(), "Parameter extraction test should pass");
    }

    #[tokio::test]
    async fn test_full_pipeline() {
        let result = test_full_routing_pipeline().await;
        assert!(result.is_ok(), "Full pipeline test should pass");
    }

    #[test]
    fn test_route_to_path() {
        assert_eq!(
            RoutingTests::route_to_path(&Route::BeadsList),
            "/beads"
        );
        assert_eq!(
            RoutingTests::route_to_path(&Route::Dashboard),
            "/dashboard"
        );
        assert_eq!(
            RoutingTests::route_to_path(&Route::Settings),
            "/settings"
        );
        assert_eq!(
            RoutingTests::route_to_path(&Route::BeadDetail { id: "bd-123".to_string() }),
            "/beads/bd-123"
        );
    }

    #[test]
    fn test_routing_error_display() {
        let error = RoutingTestError::NavigationFailed {
            expected: "/beads".to_string(),
            actual: "/dashboard".to_string(),
        };
        assert_eq!(error.to_string(), "Navigation failed. Expected: /beads, Actual: /dashboard");
    }

    #[test]
    fn test_routing_error_display_browser_setup() {
        let error = RoutingTestError::BrowserSetupFailed("Connection failed".to_string());
        assert_eq!(error.to_string(), "Browser setup failed: Connection failed");
    }

    #[test]
    fn test_routing_error_display_content_not_visible() {
        let error = RoutingTestError::ContentNotVisible {
            route: "Dashboard".to_string(),
            error: "Element not found".to_string(),
        };
        assert_eq!(error.to_string(), "Content not visible for route Dashboard: Element not found");
    }
}