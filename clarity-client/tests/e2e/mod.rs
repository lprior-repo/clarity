//! End-to-end (E2E) test suite for Clarity application
//!
//! This module provides comprehensive end-to-end tests for routing,
//! browser automation, and user interaction scenarios.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

pub mod browser_setup;
pub mod routing_tests;
pub mod test_data;
pub mod utils;

// Re-export commonly used types and functions
pub use self::{
    browser_setup::{BrowserSetup, BrowserContext},
    routing_tests::{RoutingTests, test_full_routing_pipeline},
    test_data::TestDataFixture,
    utils::{TestUtils, TestAssertions},
};

/// E2E test configuration
#[derive(Debug, Clone)]
pub struct E2EConfig {
    /// Base URL for application testing
    pub base_url: String,
    /// Browser headless mode
    pub headless: bool,
    /// Test execution timeout
    pub timeout: std::time::Duration,
    /// Number of retry attempts
    pub retries: u32,
}

impl Default for E2EConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8080".to_string(),
            headless: true,
            timeout: std::time::Duration::from_secs(30),
            retries: 3,
        }
    }
}

/// E2E test runner
pub struct E2ETestRunner {
    config: E2EConfig,
}

impl E2ETestRunner {
    /// Create new test runner with configuration
    pub fn new(config: E2EConfig) -> Self {
        Self { config }
    }

    /// Run all E2E tests
    pub async fn run_all_tests(&self) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("Starting E2E test suite with config: {:?}", self.config);

        // Initialize browser
        let context = BrowserSetup::initialize_browser()
            .await
            .map_err(|e| format!("Failed to initialize browser: {}", e))?;

        // Setup test environment
        BrowserSetup::setup_test_environment(&context).await
            .map_err(|e| format!("Failed to setup test environment: {}", e))?;

        // Run individual test suites
        let mut results = Vec::new();

        // Navigation tests
        let navigation_result = RoutingTests::test_all_navigation_paths(&context).await;
        results.push(("navigation_paths", navigation_result));

        // History tests
        let history_result = RoutingTests::test_browser_history_sync(&context).await;
        results.push(("browser_history", history_result));

        // Error handling tests
        let error_result = RoutingTests::test_invalid_route_handling(&context).await;
        results.push(("invalid_routes", error_result));

        // Parameter extraction tests
        let parameter_result = RoutingTests::test_route_parameter_extraction(&context).await;
        results.push(("parameter_extraction", parameter_result));

        // Full pipeline test
        let pipeline_result = test_full_routing_pipeline().await;
        results.push(("full_pipeline", pipeline_result));

        // Cleanup
        let cleanup_result = BrowserSetup::cleanup_browser(context).await;

        // Generate test report
        let report = self.generate_test_report(&results, cleanup_result);

        tracing::info!("{}", report);

        // Check for failures
        let failures: Vec<_> = results
            .iter()
            .filter(|(_, result)| result.is_err())
            .collect();

        if !failures.is_empty() {
            tracing::error!("{} tests failed", failures.len());
            for (name, result) in failures {
                tracing::error!("Test '{}' failed: {}", name, result.as_ref().unwrap_err());
            }
            return Err(format!("{} tests failed", failures.len()).into());
        }

        Ok(())
    }

    /// Generate test report
    fn generate_test_report(
        &self,
        results: &[(&str, Result<(), String>)],
        cleanup_result: Result<(), String>,
    ) -> String {
        let total = results.len();
        let passed = results.iter().filter(|(_, r)| r.is_ok()).count();
        let failed = total - passed;

        let mut report = format!(
            "=== E2E Test Report ===\n",
            "Total Tests: {}\n",
            "Passed: {}\n",
            "Failed: {}\n",
            "Success Rate: {:.1}%\n",
            "Duration: {:?}\n",
        );

        report.push_str("Test Results:\n");
        for (name, result) in results {
            let status = if result.is_ok() { "✅ PASS" } else { "❌ FAIL" };
            let details = result
                .as_ref()
                .map(|_| "No issues")
                .unwrap_or_else(|e| e);
            report.push_str(&format!("  {}: {}\n", name, status));
            report.push_str(&format!("    {}\n", details));
        }

        report.push_str(&format!(
            "\nCleanup: {}\n",
            if cleanup_result.is_ok() { "✅ SUCCESS" } else { "❌ FAILED" }
        ));

        if let Err(e) = cleanup_result {
            report.push_str(&format!("  Error: {}\n", e));
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_e2e_runner_creation() {
        let config = E2EConfig::default();
        let runner = E2ETestRunner::new(config);
        assert_eq!(runner.config.base_url, "http://localhost:8080");
        assert!(runner.config.headless);
    }

    #[tokio::test]
    async fn test_e2e_runner_with_custom_config() {
        let config = E2EConfig {
            base_url: "http://test.example.com".to_string(),
            headless: false,
            timeout: std::time::Duration::from_secs(60),
            retries: 5,
        };
        let runner = E2ETestRunner::new(config);
        assert_eq!(runner.config.base_url, "http://test.example.com");
        assert!(!runner.config.headless);
        assert_eq!(runner.config.retries, 5);
    }
}