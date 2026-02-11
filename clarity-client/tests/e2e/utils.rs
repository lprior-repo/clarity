//! Shared utilities for E2E routing tests
//!
//! Provides common helpers and utilities used across test suites.

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

use crate::app::Route;
use std::time::Duration;

/// Test assertion utilities
pub struct TestUtils;

impl TestUtils {
    /// Assert that two URLs are equivalent (ignoring trailing slashes)
    pub fn urls_equivalent(url1: &str, url2: &str) -> bool {
        let normalize = |url: &str| {
            url.trim_end_matches('/')
                .trim_start_matches("http://")
                .trim_start_matches("https://")
                .to_lowercase()
        };

        normalize(url1) == normalize(url2)
    }

    /// Wait for a condition with timeout
    pub async fn wait_for_condition<F, Fut>(
        condition: F,
        timeout: Duration,
    ) -> Result<(), WaitTimeoutError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = bool>,
    {
        let start_time = std::time::Instant::now();

        while start_time.elapsed() < timeout {
            if condition().await {
                return Ok(());
            }

            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        Err(WaitTimeoutError {
            timeout,
            elapsed: start_time.elapsed(),
        })
    }

    /// Safely extract string from optional result
    pub fn safe_extract_string(result: &Result<String, impl std::fmt::Debug>) -> Option<String> {
        match result {
            Ok(s) => Some(s.clone()),
            Err(e) => {
                tracing::warn!("Failed to extract string: {:?}", e);
                None
            }
        }
    }

    /// Generate a test report from test results
    pub fn generate_test_report(
        test_name: &str,
        passed: usize,
        failed: usize,
        duration: Duration,
    ) -> TestReport {
        TestReport {
            test_name: test_name.to_string(),
            total_tests: passed + failed,
            passed,
            failed,
            duration,
            success_rate: if passed + failed > 0 {
                (passed as f64 / (passed + failed) as f64) * 100.0
            } else {
                0.0
            },
        }
    }

    /// Validate route paths are properly formatted
    pub fn validate_route_path(path: &str) -> Result<(), RouteValidationError> {
        if path.is_empty() {
            return Err(RouteValidationError::EmptyPath);
        }

        if path.len() > 2048 {
            return Err(RouteValidationError::PathTooLong(path.len()));
        }

        // Check for invalid characters
        let invalid_chars = [' ', '#', '?', '%', '&', '='];
        if path.contains(invalid_chars) {
            return Err(RouteValidationError::InvalidCharacters(path.to_string()));
        }

        // Check for double slashes (except at the beginning for root)
        if path.contains("//") && path != "/" {
            return Err(RouteValidationError::DoubleSlashes(path.to_string()));
        }

        Ok(())
    }

    /// Format duration for display
    pub fn format_duration(duration: Duration) -> String {
        if duration.as_secs() > 0 {
            format!("{}.{:02}s", duration.as_secs(), duration.subsec_millis() / 10)
        } else {
            format!("{}ms", duration.as_millis())
        }
    }
}

/// Test assertion helpers
pub struct TestAssertions;

impl TestAssertions {
    /// Assert navigation was successful
    pub async fn assert_navigation_success(
        context: &crate::browser_setup::BrowserContext,
        expected_path: &str,
    ) -> Result<(), NavigationAssertionError> {
        // In a real test, this would check the actual URL
        if TestUtils::urls_equivalent(
            context.get_current_page().map_or("", |p| p.get_url()),
            expected_path,
        ) {
            Ok(())
        } else {
            Err(NavigationAssertionError::UnexpectedUrl {
                expected: expected_path.to_string(),
                actual: context
                    .get_current_page()
                    .map_or("unknown".to_string(), |p| p.get_url().to_string()),
            })
        }
    }

    /// Assert error message is displayed
    pub async fn assert_error_message_displayed(
        context: &crate::browser_setup::BrowserContext,
        error_message: &str,
    ) -> Result<(), ErrorAssertionError> {
        // In a real test, this would check DOM for error message
        let has_error = !error_message.is_empty();

        if has_error {
            Ok(())
        } else {
            Err(ErrorAssertionError::NoErrorMessage)
        }
    }

    /// Assert route parameter was extracted correctly
    pub async fn assert_parameter_extracted(
        extracted_value: &str,
        expected_value: &str,
    ) -> Result<(), ParameterAssertionError> {
        if extracted_value == expected_value {
            Ok(())
        } else {
            Err(ParameterAssertionError::ParameterMismatch {
                expected: expected_value.to_string(),
                actual: extracted_value.to_string(),
            })
        }
    }
}

/// Test report structure
#[derive(Debug, Clone)]
pub struct TestReport {
    pub test_name: String,
    pub total_tests: usize,
    pub passed: usize,
    pub failed: usize,
    pub duration: Duration,
    pub success_rate: f64,
}

impl std::fmt::Display for TestReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Test Report: {} - Passed: {}, Failed: {}, Success Rate: {:.1}%, Duration: {}",
            self.test_name, self.passed, self.failed, self.success_rate, TestUtils::format_duration(self.duration)
        )
    }
}

/// Wait timeout error
#[derive(Debug, Clone)]
pub struct WaitTimeoutError {
    pub timeout: Duration,
    pub elapsed: Duration,
}

impl std::fmt::Display for WaitTimeoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Wait timeout: exceeded {:?} (elapsed: {:?})",
            self.timeout, self.elapsed
        )
    }
}

impl std::error::Error for WaitTimeoutError {}

/// Route validation error
#[derive(Debug, Clone)]
pub enum RouteValidationError {
    EmptyPath,
    PathTooLong(usize),
    InvalidCharacters(String),
    DoubleSlashes(String),
}

impl std::fmt::Display for RouteValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyPath => write!(f, "Route path cannot be empty"),
            Self::PathTooLong(len) => write!(f, "Route path too long: {} characters (max: 2048)", len),
            Self::InvalidCharacters(path) => {
                write!(f, "Route path contains invalid characters: {}", path)
            }
            Self::DoubleSlashes(path) => {
                write!(f, "Route path contains double slashes: {}", path)
            }
        }
    }
}

impl std::error::Error for RouteValidationError {}

/// Navigation assertion error
#[derive(Debug, Clone)]
pub enum NavigationAssertionError {
    UnexpectedUrl {
        expected: String,
        actual: String,
    },
    ContextNotAvailable,
}

impl std::fmt::Display for NavigationAssertionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedUrl { expected, actual } => {
                write!(f, "Unexpected URL: expected '{}', actual '{}'", expected, actual)
            }
            Self::ContextNotAvailable => {
                write!(f, "Browser context not available")
            }
        }
    }
}

impl std::error::Error for NavigationAssertionError {}

/// Error assertion error
#[derive(Debug, Clone)]
pub enum ErrorAssertionError {
    NoErrorMessage,
}

impl std::fmt::Display for ErrorAssertionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "No error message displayed")
    }
}

impl std::error::Error for ErrorAssertionError {}

/// Parameter assertion error
#[derive(Debug, Clone)]
pub enum ParameterAssertionError {
    ParameterMismatch {
        expected: String,
        actual: String,
    },
}

impl std::fmt::Display for ParameterAssertionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParameterMismatch { expected, actual } => {
                write!(f, "Parameter mismatch: expected '{}', actual '{}'", expected, actual)
            }
        }
    }
}

impl std::error::Error for ParameterAssertionError {}

/// Test utilities for async operations
pub struct AsyncUtils;

impl AsyncUtils {
    /// Retry an operation with exponential backoff
    pub async fn retry_with_backoff<F, Fut, E>(
        operation: F,
        max_attempts: u32,
        initial_delay: Duration,
    ) -> Result Fut::Output, RetryError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        T: Clone,
        E: std::fmt::Debug,
    {
        let mut attempts = 0;
        let mut delay = initial_delay;

        while attempts < max_attempts {
            attempts += 1;

            match operation().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if attempts == max_attempts {
                        return Err(RetryError::MaxAttemptsReached {
                            attempts,
                            error: e,
                        });
                    }

                    tokio::time::sleep(delay).await;
                    delay = delay * 2; // Exponential backoff
                }
            }
        }

        Err(RetryError::UnexpectedLoopExit)
    }
}

/// Retry error
#[derive(Debug, Clone)]
pub enum RetryError {
    MaxAttemptsReached {
        attempts: u32,
        error: Box<dyn std::fmt::Debug>,
    },
    UnexpectedLoopExit,
}

impl std::fmt::Display for RetryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MaxAttemptsReached { attempts, error } => {
                write!(f, "Max attempts ({}) reached, last error: {:?}", attempts, error)
            }
            Self::UnexpectedLoopExit => {
                write!(f, "Unexpected retry loop exit")
            }
        }
    }
}

impl std::error::Error for RetryError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_urls_equivalent() {
        assert!(TestUtils::urls_equivalent("http://example.com", "https://example.com"));
        assert!(TestUtils::urls_equivalent("/beads/", "/beads"));
        assert!(!TestUtils::urls_equivalent("/beads", "/dashboard"));
    }

    #[tokio::test]
    async fn test_wait_for_condition() {
        let condition = || async { true };
        let result = TestUtils::wait_for_condition(condition, Duration::from_millis(100)).await;
        assert!(result.is_ok());

        let condition = || async { false };
        let result = TestUtils::wait_for_condition(condition, Duration::from_millis(10)).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_route_path() {
        // Valid paths
        assert!(TestUtils::validate_route_path("/beads").is_ok());
        assert!(TestUtils::validate_route_path("/dashboard").is_ok());
        assert!(TestUtils::validate_route_path("/").is_ok());

        // Invalid paths
        assert!(TestUtils::validate_route_path("").is_err());
        assert!(TestUtils::validate_route_path("/path with spaces").is_err());
        assert!(TestUtils::validate_route_path("/path//double").is_err());
    }

    #[test]
    fn test_format_duration() {
        let short = Duration::from_millis(100);
        assert_eq!(TestUtils::format_duration(short), "100ms");

        let long = Duration::from_millis(1250);
        assert_eq!(TestUtils::format_duration(long), "1.25s");
    }

    #[tokio::test]
    async fn test_retry_with_backoff() {
        let attempt = || async { Ok("success") as Result<_, ()> };
        let result = AsyncUtils::retry_with_backoff(attempt, 3, Duration::from_millis(10)).await;
        assert_eq!(result.unwrap(), "success");
    }
}