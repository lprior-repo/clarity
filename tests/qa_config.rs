//! QA Enforcer Configuration: Test orchestration and reporting
//!
//! This module provides configuration and utilities for running
//! comprehensive QA enforcer tests across all categories.

pub mod config {
    use std::path::Path;
    use std::process::Command;
    use std::time::Duration;

    /// Test configuration
    #[derive(Debug, Clone)]
    pub struct TestConfig {
        pub test_timeout: Duration,
        pub max_concurrent_tests: usize,
        pub retry_on_failure: bool,
        pub max_retries: u32,
        pub output_format: OutputFormat,
        pub severity_filter: Option<SeverityLevel>,
    }

    /// Output format for test results
    #[derive(Debug, Clone)]
    pub enum OutputFormat {
        HumanReadable,
        Json,
        Tap,
        JUnit,
    }

    /// Severity level filtering
    #[derive(Debug, Clone)]
    pub enum SeverityLevel {
        Critical,
        Major,
        Minor,
        Observation,
    }

    /// Default test configuration
    impl Default for TestConfig {
        fn default() -> Self {
            Self {
                test_timeout: Duration::from_secs(30),
                max_concurrent_tests: 4,
                retry_on_failure: true,
                max_retries: 2,
                output_format: OutputFormat::HumanReadable,
                severity_filter: None,
            }
        }
    }

    /// Test suite configuration
    pub struct TestSuiteConfig {
        pub categories: Vec<TestCategory>,
        pub setup_commands: Vec<String>,
        pub cleanup_commands: Vec<String>,
        pub dependencies: Vec<String>,
    }

    /// Test category configuration
    pub struct TestCategory {
        pub name: String,
        pub enabled: bool,
        pub priority: TestPriority,
        pub timeout: Duration,
        pub max_retries: u32,
    }

    /// Test priority levels
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum TestPriority {
        Critical = 1,
        Major = 2,
        Minor = 3,
        Observation = 4,
    }

    impl TestSuiteConfig {
        pub fn default() -> Self {
            Self {
                categories: vec![
                    TestCategory {
                        name: "unit".to_string(),
                        enabled: true,
                        priority: TestPriority::Critical,
                        timeout: Duration::from_secs(10),
                        max_retries: 0,
                    },
                    TestCategory {
                        name: "integration".to_string(),
                        enabled: true,
                        priority: TestPriority::Major,
                        timeout: Duration::from_secs(30),
                        max_retries: 1,
                    },
                    TestCategory {
                        name: "api".to_string(),
                        enabled: true,
                        priority: TestPriority::Major,
                        timeout: Duration::from_secs(30),
                        max_retries: 1,
                    },
                    TestCategory {
                        name: "cli".to_string(),
                        enabled: true,
                        priority: TestPriority::Major,
                        timeout: Duration::from_secs(15),
                        max_retries: 1,
                    },
                    TestCategory {
                        name: "workflow".to_string(),
                        enabled: true,
                        priority: TestPriority::Major,
                        timeout: Duration::from_secs(60),
                        max_retries: 1,
                    },
                    TestCategory {
                        name: "adversarial".to_string(),
                        enabled: true,
                        priority: TestPriority::Minor,
                        timeout: Duration::from_secs(30),
                        max_retries: 0,
                    },
                ],
                setup_commands: vec![
                    "cargo build --workspace".to_string(),
                ],
                cleanup_commands: vec![
                    "cargo clean".to_string(),
                ],
                dependencies: vec![
                    "cargo".to_string(),
                    "git".to_string(),
                ],
            }
        }

        /// Get all enabled categories
        pub fn enabled_categories(&self) -> Vec<&TestCategory> {
            self.categories.iter().filter(|cat| cat.enabled).collect()
        }

        /// Get categories by priority
        pub fn categories_by_priority(&self) -> Vec<&TestCategory> {
            let mut enabled = self.enabled_categories();
            enabled.sort_by(|a, b| a.priority.cmp(&b.priority));
            enabled
        }
    }
}

/// Test runner utility
pub mod runner {
    use super::config::*;
    use std::process::Command;
    use std::time::{Duration, Instant};
    use std::thread;
    use std::sync::mpsc;
    use std::sync::Arc;
    use std::sync::Mutex;

    pub struct TestRunner {
        config: TestConfig,
        suite_config: TestSuiteConfig,
    }

    impl TestRunner {
        pub fn new(config: TestConfig) -> Self {
            Self {
                config,
                suite_config: TestSuiteConfig::default(),
            }
        }

        /// Run all enabled tests
        pub fn run_all_tests(&self) -> TestReport {
            println!("🚀 Starting QA Enforcer test suite...");

            // Check dependencies
            self.check_dependencies();

            // Run setup commands
            self.run_setup_commands();

            // Run all tests
            let mut results = Vec::new();

            for category in self.suite_config.categories_by_priority() {
                println!("\n📋 Running {} tests...", category.name);
                let category_results = self.run_category_tests(category);
                results.extend(category_results);
            }

            // Run cleanup commands
            self.run_cleanup_commands();

            // Generate report
            let report = TestReport::new(results);
            self.generate_report(&report);

            report
        }

        /// Check test dependencies
        fn check_dependencies(&self) {
            println!("🔍 Checking dependencies...");

            for dep in &self.suite_config.dependencies {
                let result = Command::new("which")
                    .arg(dep)
                    .output();

                match result {
                    Ok(output) if output.status.success() => {
                        println!("✅ {} is available", dep);
                    }
                    _ => {
                        println!("❌ {} is not available", dep);
                        std::process::exit(1);
                    }
                }
            }
        }

        /// Run setup commands
        fn run_setup_commands(&self) {
            println!("🔧 Running setup commands...");

            for cmd in &self.suite_config.setup_commands {
                println!("Running: {}", cmd);

                let result = Command::new("sh")
                    .arg("-c")
                    .arg(cmd)
                    .output();

                match result {
                    Ok(output) if output.status.success() => {
                        println!("✅ Setup command completed: {}", cmd);
                    }
                    Ok(output) => {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        eprintln!("❌ Setup command failed: {}", cmd);
                        eprintln!("Error: {}", stderr);
                    }
                    Err(e) => {
                        eprintln!("❌ Setup command failed: {}", cmd);
                        eprintln!("Error: {}", e);
                    }
                }
            }
        }

        /// Run cleanup commands
        fn run_cleanup_commands(&self) {
            println!("🧹 Running cleanup commands...");

            for cmd in &self.suite_config.cleanup_commands {
                println!("Running: {}", cmd);

                let _ = Command::new("sh")
                    .arg("-c")
                    .arg(cmd)
                    .output();
            }
        }

        /// Run tests for a category
        fn run_category_tests(&self, category: &TestCategory) -> Vec<TestResult> {
            let test_commands = self.get_test_commands(&category.name);

            if test_commands.is_empty() {
                println!("No tests found for category: {}", category.name);
                return Vec::new();
            }

            // Run tests in parallel if possible
            if self.config.max_concurrent_tests > 1 && test_commands.len() > 1 {
                self.run_tests_parallel(test_commands, category)
            } else {
                self.run_tests_sequential(test_commands, category)
            }
        }

        /// Get test commands for a category
        fn get_test_commands(&self, category: &str) -> Vec<String> {
            match category {
                "unit" => vec![
                    "cargo test --lib".to_string(),
                    "cargo test --test zero_unwrap_tests".to_string(),
                    "cargo test --test test_db_init".to_string(),
                ],
                "integration" => vec![
                    "cargo test --test integration_test".to_string(),
                    "cargo test --test functional_navigation_test".to_string(),
                    "cargo test --test sorting_test".to_string(),
                ],
                "api" => vec![
                    "cargo test --test api".to_string(),
                ],
                "cli" => vec![
                    "cargo test --test cli".to_string(),
                ],
                "workflow" => vec![
                    "cargo test --test workflows".to_string(),
                ],
                "adversarial" => vec![
                    "cargo test --test adversarial".to_string(),
                ],
                _ => Vec::new(),
            }
        }

        /// Run tests sequentially
        fn run_tests_sequential(&self, test_commands: Vec<String>, category: &TestCategory) -> Vec<TestResult> {
            let mut results = Vec::new();

            for cmd in test_commands {
                let result = self.run_test_with_retry(&cmd, category);
                results.push(result);
            }

            results
        }

        /// Run tests in parallel
        fn run_tests_parallel(&self, test_commands: Vec<String>, category: &TestCategory) -> Vec<TestResult> {
            let (tx, rx) = mpsc::channel();
            let results = Arc::new(Mutex::new(Vec::new()));

            // Start test threads
            for cmd in test_commands {
                let tx = tx.clone();
                let category = category.clone();

                thread::spawn(move || {
                    let result = Self::run_test_with_retry(&cmd, &category);
                    tx.send(result).unwrap();
                });
            }

            // Collect results
            drop(tx); // Close the channel
            let mut collected_results = Vec::new();
            while let Ok(result) = rx.recv() {
                collected_results.push(result);
            }

            collected_results
        }

        /// Run a test with retry logic
        fn run_test_with_retry(&self, cmd: &str, category: &TestCategory) -> TestResult {
            let start_time = Instant::now();
            let mut attempt = 0;

            loop {
                attempt += 1;
                println!("Running: {} (attempt {}/{})", cmd, attempt, category.max_retries + 1);

                let result = self.run_single_test(cmd, category);
                let duration = start_time.elapsed();

                if result.status == TestStatus::Passed {
                    return TestResult {
                        name: cmd.to_string(),
                        status: TestStatus::Passed,
                        duration,
                        attempt,
                        output: result.output,
                        error: None,
                    };
                }

                // Check if we should retry
                if attempt > category.max_retries || !self.config.retry_on_failure {
                    return TestResult {
                        name: cmd.to_string(),
                        status: TestStatus::Failed,
                        duration,
                        attempt,
                        output: result.output,
                        error: result.error,
                    };
                }

                println!("Retrying test in 1 second...");
                thread::sleep(Duration::from_secs(1));
            }
        }

        /// Run a single test
        fn run_single_test(&self, cmd: &str, _category: &TestCategory) -> TestResult {
            let start_time = Instant::now();

            let result = Command::new("sh")
                .arg("-c")
                .arg(cmd)
                .output()
                .expect("Failed to execute test command");

            let duration = start_time.elapsed();
            let output = String::from_utf8_lossy(&result.stdout).to_string();
            let stderr = String::from_utf8_lossy(&result.stderr).to_string();

            if result.status.success() {
                TestResult {
                    name: cmd.to_string(),
                    status: TestStatus::Passed,
                    duration,
                    attempt: 1,
                    output,
                    error: None,
                }
            } else {
                TestResult {
                    name: cmd.to_string(),
                    status: TestStatus::Failed,
                    duration,
                    attempt: 1,
                    output,
                    error: Some(stderr),
                }
            }
        }

        /// Generate test report
        fn generate_report(&self, report: &TestReport) {
            match self.config.output_format {
                OutputFormat::HumanReadable => self.print_human_report(report),
                OutputFormat::Json => self.print_json_report(report),
                OutputFormat::Tap => self.print_tap_report(report),
                OutputFormat::JUnit => self.print_junit_report(report),
            }
        }

        /// Print human-readable report
        fn print_human_report(&self, report: &TestReport) {
            println!("\n" + "=".repeat(80).as_str());
            println!("📊 QA ENFORCER TEST REPORT");
            println!("=" .repeat(80));

            println!("\n📈 Summary:");
            println!("  Total Tests: {}", report.total_tests());
            println!("  Passed: {}", report.passed_tests());
            println!("  Failed: {}", report.failed_tests());
            println!("  Success Rate: {:.1}%", report.success_rate());

            if report.has_failures() {
                println!("\n❌ Failed Tests:");
                for result in &report.results {
                    if result.status == TestStatus::Failed {
                        println!("  - {}: {}s", result.name, result.duration.as_secs());
                        if let Some(error) = &result.error {
                            println!("    Error: {}", error.lines().next().unwrap_or("Unknown error"));
                        }
                    }
                }
            }

            println!("\n⏱️  Total Duration: {}", report.total_duration());
            println!("🏆 Quality Gate: {}", if report.passed() { "PASSED" } else { "FAILED" });
        }

        /// Print JSON report
        fn print_json_report(&self, report: &TestReport) {
            let json = serde_json::json!({
                "summary": {
                    "total_tests": report.total_tests(),
                    "passed_tests": report.passed_tests(),
                    "failed_tests": report.failed_tests(),
                    "success_rate": report.success_rate(),
                    "total_duration_ms": report.total_duration().as_millis(),
                    "quality_gate": report.passed(),
                },
                "results": report.results.iter().map(|r| {
                    serde_json::json!({
                        "name": r.name,
                        "status": r.status.as_str(),
                        "duration_ms": r.duration.as_millis(),
                        "attempt": r.attempt,
                        "output": r.output,
                        "error": r.error,
                    })
                }).collect::<Vec<_>>(),
            });

            println!("{}", serde_json::to_string_pretty(&json).unwrap());
        }

        /// Print TAP report
        fn print_tap_report(&self, _report: &TestReport) {
            println!("TAP version 13");
            println!("1..{}", _report.total_tests());

            let mut test_num = 1;
            for result in &_report.results {
                if result.status == TestStatus::Passed {
                    println!("ok {} - {}", test_num, result.name);
                } else {
                    println!("not ok {} - {}", test_num, result.name);
                    if let Some(error) = &result.error {
                        println!("  ---");
                        println!("  {}", error);
                        println!("  ...");
                    }
                }
                test_num += 1;
            }
        }

        /// Print JUnit report
        fn print_junit_report(&self, report: &TestReport) {
            println!(r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuites>
  <testsuite name="QA Enforcer" tests="{}" failures="{}" time="{}">"#,
                report.total_tests(),
                report.failed_tests(),
                report.total_duration().as_secs_f64());

            for result in &report.results {
                println!("    <testcase name=\"{}\" time=\"{}\">", result.name, result.duration.as_secs_f64());
                if result.status == TestStatus::Failed {
                    println!("      <failure type=\"TestFailed\">");
                    if let Some(error) = &result.error {
                        println!("        {}", error);
                    }
                    println!("      </failure>");
                }
                println!("    </testcase>");
            }

            println!("  </testsuite>
</testsuites>");
        }
    }

    /// Test result structure
    #[derive(Debug, Clone)]
    pub struct TestResult {
        pub name: String,
        pub status: TestStatus,
        pub duration: Duration,
        pub attempt: u32,
        pub output: String,
        pub error: Option<String>,
    }

    /// Test status
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum TestStatus {
        Passed,
        Failed,
    }

    impl TestStatus {
        pub fn as_str(&self) -> &'static str {
            match self {
                TestStatus::Passed => "passed",
                TestStatus::Failed => "failed",
            }
        }
    }

    /// Test report structure
    #[derive(Debug, Clone)]
    pub struct TestReport {
        pub results: Vec<TestResult>,
    }

    impl TestReport {
        pub fn new(results: Vec<TestResult>) -> Self {
            Self { results }
        }

        pub fn total_tests(&self) -> usize {
            self.results.len()
        }

        pub fn passed_tests(&self) -> usize {
            self.results.iter().filter(|r| r.status == TestStatus::Passed).count()
        }

        pub fn failed_tests(&self) -> usize {
            self.results.iter().filter(|r| r.status == TestStatus::Failed).count()
        }

        pub fn success_rate(&self) -> f64 {
            if self.results.is_empty() {
                0.0
            } else {
                (self.passed_tests() as f64 / self.total_tests() as f64) * 100.0
            }
        }

        pub fn total_duration(&self) -> Duration {
            self.results.iter().map(|r| r.duration).sum()
        }

        pub fn passed(&self) -> bool {
            self.failed_tests() == 0
        }

        pub fn has_failures(&self) -> bool {
            self.failed_tests() > 0
        }
    }
}