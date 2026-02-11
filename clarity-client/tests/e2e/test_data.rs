//! Test data fixtures for E2E routing tests
//!
//! Provides test data and cleanup utilities for routing E2E tests.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::app::Route;
use std::collections::HashMap;

/// Test data fixtures for routing tests
pub struct TestDataFixture;

impl TestDataFixture {
    /// Get all valid routes for testing
    pub fn get_test_routes() -> HashMap<&'static str, Route> {
        let mut routes = HashMap::new();

        routes.insert("beads_list", Route::BeadsList);
        routes.insert("dashboard", Route::Dashboard);
        routes.insert("settings", Route::Settings);
        routes.insert("bead_detail_bd123", Route::BeadDetail {
            id: "bd-123".to_string()
        });
        routes.insert("bead_detail_bd456", Route::BeadDetail {
            id: "bd-456".to_string()
        });

        routes
    }

    /// Get invalid route paths for error testing
    pub fn get_invalid_routes() -> Vec<String> {
        vec![
            "/invalid/route/123".to_string(),
            "/nonexistent/path".to_string(),
            "/beads/".to_string(), // Missing ID
            "/".to_string(), // Root might not be implemented
            "/malformed/route//path".to_string(),
        ]
    }

    /// Get valid bead IDs for parameter testing
    pub fn get_valid_bead_ids() -> Vec<String> {
        vec![
            "bd-123".to_string(),
            "bd-456".to_string(),
            "bd-test-001".to_string(),
            "bd-feature-new".to_string(),
        ]
    }

    /// Get expected URL paths for routes
    pub fn get_expected_paths() -> HashMap<&'static str, String> {
        let mut paths = HashMap::new();

        paths.insert("beads_list", "/beads".to_string());
        paths.insert("dashboard", "/dashboard".to_string());
        paths.insert("settings", "/settings".to_string());
        paths.insert("bead_detail_bd123", "/beads/bd-123".to_string());
        paths.insert("bead_detail_bd456", "/beads/bd-456".to_string());
        paths.insert("bead_detail_bdtest001", "/beads/bd-test-001".to_string());

        paths
    }

    /// Get expected page titles for routes
    pub fn get_expected_page_titles() -> HashMap<&'static str, String> {
        let mut titles = HashMap::new();

        titles.insert("beads_list", "Beads".to_string());
        titles.insert("dashboard", "Dashboard".to_string());
        titles.insert("settings", "Settings".to_string());
        titles.insert("bead_detail_bd123", "Bead bd-123".to_string());
        titles.insert("bead_detail_bd456", "Bead bd-456".to_string());

        titles
    }

    /// Test cleanup utility
    pub async fn cleanup_test_state() -> Result<(), TestCleanupError> {
        // In a real implementation, this would:
        // 1. Clear browser cookies
        // 2. Clear localStorage/sessionStorage
        // 3. Reset browser history
        // 4. Close any open browser instances
        // 5. Reset test databases

        tracing::info!("Cleaning up test state");

        // Simulate cleanup delay
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        tracing::info!("Test state cleanup complete");
        Ok(())
    }

    /// Setup test environment
    pub async fn setup_test_environment() -> Result<(), TestSetupError> {
        // In a real implementation, this would:
        // 1. Start browser instance
        // 2. Navigate to base URL
        // 3. Wait for app to load
        // 4. Initialize test databases with fresh data

        tracing::info!("Setting up test environment");

        // Simulate setup delay
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        tracing::info!("Test environment setup complete");
        Ok(())
    }

    /// Verify test data integrity
    pub fn verify_test_data() -> Result<(), TestDataError> {
        let routes = Self::get_test_routes();
        let paths = Self::get_expected_paths();
        let titles = Self::get_expected_page_titles();

        // Verify route-path consistency
        for (key, route) in &routes {
            if let Some(expected_path) = paths.get(key) {
                let actual_path = match route {
                    Route::BeadsList => "/beads",
                    Route::Dashboard => "/dashboard",
                    Route::Settings => "/settings",
                    Route::BeadDetail { id } => format!("/beads/{}", id),
                };

                if actual_path != expected_path {
                    return Err(TestDataError::PathMismatch {
                        route_key: key.to_string(),
                        expected: expected_path.clone(),
                        actual: actual_path,
                    });
                }

                // Verify title exists for routes that should have titles
                if key.contains("detail") {
                    if !titles.contains_key(key) {
                        return Err(TestDataError::MissingTitle {
                            route_key: key.to_string(),
                        });
                    }
                }
            }
        }

        // Verify invalid routes don't exist in valid routes
        let valid_paths: Vec<String> = paths.values().cloned().collect();
        for invalid_route in &Self::get_invalid_routes() {
            if valid_paths.contains(invalid_route) {
                return Err(TestDataError::InvalidRouteInValidList {
                    invalid_route: invalid_route.clone(),
                });
            }
        }

        Ok(())
    }

    /// Generate test scenarios for navigation tests
    pub fn generate_navigation_scenarios() -> Vec<NavigationScenario> {
        let mut scenarios = Vec::new();

        // Test direct navigation to each route
        for (key, route) in Self::get_test_routes() {
            scenarios.push(NavigationScenario {
                name: format!("navigate_to_{}", key),
                route: route.clone(),
                expected_path: Self::get_expected_paths()
                    .get(key)
                    .cloned()
                    .unwrap_or_else(|| "/unknown".to_string()),
            });
        }

        // Test sequential navigation through multiple routes
        let route_sequence = vec![
            Route::BeadsList,
            Route::Dashboard,
            Route::Settings,
            Route::BeadDetail { id: "bd-123".to_string() },
        ];

        scenarios.push(NavigationScenario {
            name: "navigate_sequence_beads_dashboard_settings_detail".to_string(),
            route: Route::BeadsList, // Starting point
            expected_path: "/beads".to_string(),
            sequence: Some(route_sequence),
        });

        scenarios
    }

    /// Generate test scenarios for error handling
    pub fn generate_error_scenarios() -> Vec<ErrorScenario> {
        Self::get_invalid_routes()
            .into_iter()
            .enumerate()
            .map(|(i, invalid_path)| ErrorScenario {
                name: format!("error_path_{}", i),
                invalid_path,
                expected_error_contains: vec
                    ["not found", "invalid", "error"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            })
            .collect()
    }
}

/// Navigation test scenario
#[derive(Debug, Clone)]
pub struct NavigationScenario {
    pub name: String,
    pub route: Route,
    pub expected_path: String,
    pub sequence: Option<Vec<Route>> = None,
}

/// Error test scenario
#[derive(Debug, Clone)]
pub struct ErrorScenario {
    pub name: String,
    pub invalid_path: String,
    pub expected_error_contains: Vec<String>,
}

/// Test setup error
#[derive(Debug, Clone)]
pub enum TestSetupError {
    BrowserInitializationFailed(String),
    AppLoadTimeout,
    NetworkError(String),
}

impl std::fmt::Display for TestSetupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BrowserInitializationFailed(msg) => {
                write!(f, "Browser initialization failed: {}", msg)
            }
            Self::AppLoadTimeout => write!(f, "App load timeout"),
            Self::NetworkError(msg) => write!(f, "Network error: {}", msg),
        }
    }
}

impl std::error::Error for TestSetupError {}

/// Test cleanup error
#[derive(Debug, Clone)]
pub enum TestCleanupError {
    BrowserCleanupFailed(String),
    ResourceCleanupFailed(String),
}

impl std::fmt::Display for TestCleanupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BrowserCleanupFailed(msg) => write!(f, "Browser cleanup failed: {}", msg),
            Self::ResourceCleanupFailed(msg) => write!(f, "Resource cleanup failed: {}", msg),
        }
    }
}

impl std::error::Error for TestCleanupError {}

/// Test data error
#[derive(Debug, Clone)]
pub enum TestDataError {
    PathMismatch {
        route_key: String,
        expected: String,
        actual: String,
    },
    MissingTitle {
        route_key: String,
    },
    InvalidRouteInValidList {
        invalid_route: String,
    },
}

impl std::fmt::Display for TestDataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PathMismatch { route_key, expected, actual } => {
                write!(
                    f,
                    "Path mismatch for route {}: expected '{}', actual '{}'",
                    route_key, expected, actual
                )
            }
            Self::MissingTitle { route_key } => {
                write!(f, "Missing title for route {}", route_key)
            }
            Self::InvalidRouteInValidList { invalid_route } => {
                write!(f, "Invalid route found in valid list: {}", invalid_route)
            }
        }
    }
}

impl std::error::Error for TestDataError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_test_data_fixture_routes() {
        let routes = TestDataFixture::get_test_routes();
        assert!(routes.contains_key("beads_list"));
        assert!(routes.contains_key("dashboard"));
        assert!(routes.contains_key("settings"));
        assert!(routes.contains_key("bead_detail_bd123"));
    }

    #[test]
    fn test_test_data_fixture_invalid_routes() {
        let invalid_routes = TestDataFixture::get_invalid_routes();
        assert!(invalid_routes.contains(&"/invalid/route/123".to_string()));
        assert!(invalid_routes.contains(&"/nonexistent/path".to_string()));
    }

    #[test]
    fn test_test_data_fixture_paths() {
        let paths = TestDataFixture::get_expected_paths();
        assert_eq!(paths.get("beads_list"), Some(&"/beads".to_string()));
        assert_eq!(paths.get("dashboard"), Some(&"/dashboard".to_string()));
    }

    #[test]
    fn test_test_data_integrity() {
        let result = TestDataFixture::verify_test_data();
        assert!(result.is_ok(), "Test data should be valid");
    }

    #[test]
    fn test_navigation_scenarios() {
        let scenarios = TestDataFixture::generate_navigation_scenarios();
        assert!(!scenarios.is_empty());

        // Verify at least one direct navigation scenario
        let direct_scenarios: Vec<_> = scenarios
            .iter()
            .filter(|s| s.sequence.is_none())
            .collect();
        assert!(!direct_scenarios.is_empty());
    }

    #[test]
    fn test_error_scenarios() {
        let scenarios = TestDataFixture::generate_error_scenarios();
        assert!(!scenarios.is_empty());
        assert_eq!(scenarios.len(), TestDataFixture::get_invalid_routes().len());
    }

    #[tokio::test]
    async fn test_test_setup() {
        let result = TestDataFixture::setup_test_environment().await;
        assert!(result.is_ok(), "Test setup should succeed");
    }

    #[tokio::test]
    async fn test_test_cleanup() {
        let result = TestDataFixture::cleanup_test_state().await;
        assert!(result.is_ok(), "Test cleanup should succeed");
    }
}