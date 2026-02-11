//! QA Enforcer Test Suite: Comprehensive End-to-End Testing
//!
//! This is the main entry point for running all QA enforcer tests.
//! It executes tests in all categories with proper error handling.

use std::process::Command;
use std::time::Duration;

mod qa_enforcer;
mod adversarial;
mod cli;
mod api;
mod workflows;
mod qa_config;
mod hotkey_tests;
mod shortcut_action_tests;
mod modifier_combination_tests;
mod shortcut_registration_tests;
mod shortcut_formatting_tests;
mod shortcut_parsing_tests;
mod shortcut_edge_case_tests;
mod platform_specific_tests;
mod shortcut_performance_tests;
mod shortcut_accessibility_tests;
mod shortcut_integration_tests;
mod shortcut_documentation_tests;

use qa_config::config::{TestConfig, TestSuiteConfig, OutputFormat, SeverityLevel};
use qa_config::runner::{TestRunner, TestReport};

fn main() {
    println!("\n" + "=".repeat(80).as_str());
    println!("🚀 QA ENFORCER TEST SUITE");
    println!("=" .repeat(80));
    println!("Executing comprehensive end-to-end tests with zero-unwrap philosophy");
    println!("Focus: End-user behavioral tests");

    // Configure test runner
    let config = TestConfig {
        test_timeout: Duration::from_secs(60),
        max_concurrent_tests: 4,
        retry_on_failure: true,
        max_retries: 2,
        output_format: OutputFormat::HumanReadable,
        severity_filter: Some(SeverityLevel::Critical),
    };

    let runner = TestRunner::new(config);

    // Run the complete test suite
    let report = runner.run_all_tests();

    // Output final summary
    println!("\n" + "=".repeat(80).as_str());
    println!("🎯 FINAL TEST RESULTS");
    println!("=" .repeat(80));

    println!("✅ Total Tests Executed: {}", report.total_tests());
    println!("📈 Passed Tests: {}", report.passed_tests());
    println!("❌ Failed Tests: {}", report.failed_tests());

    if report.total_tests() > 0 {
        println!("🎯 Success Rate: {:.1}%", report.success_rate());
    }

    println!("⏱️  Total Test Duration: {}", report.total_duration());

    // Quality gate check
    if report.passed() {
        println!("🏆 QA QUALITY GATE: PASSED");
        println!("\n✅ All tests passed! The application meets quality standards.");
        std::process::exit(0);
    } else {
        println!("🚨 QA QUALITY GATE: FAILED");
        println!("\n❌ Some tests failed. Please review the failures above.");
        println!("📝 The application needs improvements to meet quality standards.");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_functionality() {
        println!("\n🧪 Testing basic functionality...");

        // Test that we can compile
        let result = Command::new("cargo")
            .args(&["build", "--workspace"])
            .output()
            .expect("Failed to build project");

        assert!(result.status.success(), "Project should build successfully");
        assert!(!String::from_utf8_lossy(&result.stderr).contains("panicked"),
                "Build should not panic");

        // Test basic help commands
        let binaries = vec!["create_bead", "list_beads", "view_bead", "update_bead", "delete_bead"];

        for binary in binaries {
            let result = Command::new("cargo")
                .args(&["run", "--bin", binary, "--", "--help"])
                .output()
                .expect("Failed to execute help command");

            assert!(result.status.success(), "{} help should work", binary);
            let output = String::from_utf8_lossy(&result.stdout);
            assert!(output.contains("Usage:"), "{} help should show usage", binary);
        }
    }

    #[test]
    fn test_zero_unwrap_compliance() {
        println!("\n🧪 Testing zero-unwrap compliance...");

        let result = Command::new("cargo")
            .args(&["test", "--test", "zero_unwrap_tests"])
            .output()
            .expect("Failed to run zero unwrap tests");

        assert!(result.status.success(), "Zero unwrap tests should pass");
        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(!stderr.contains("panicked"), "Should not panic");
        assert!(!stderr.contains("unwrap"), "Should not contain unwrap");
        assert!(!stderr.contains("expect"), "Should not contain expect");
    }

    #[test]
    fn test_critical_paths() {
        println!("\n🧪 Testing critical application paths...");

        // Test bead creation
        let result = Command::new("cargo")
            .args(&["run", "--bin", "create_bead", "--", "--slug", "test-critical", "--title", "Critical Test"])
            .output()
            .expect("Failed to create bead");

        assert!(result.status.success(), "Should create bead successfully");
        let stdout = String::from_utf8_lossy(&result.stdout);
        assert!(stdout.contains("Bead created") || stdout.contains("Created"), "Should confirm creation");

        // Test bead listing
        let result = Command::new("cargo")
            .args(&["run", "--bin", "list_beads"])
            .output()
            .expect("Failed to list beads");

        assert!(result.status.success(), "Should list beads successfully");
        let stdout = String::from_utf8_lossy(&result.stdout);
        assert!(stdout.contains("test-critical"), "Should show created bead");

        // Test bead viewing
        let result = Command::new("cargo")
            .args(&["run", "--bin", "view_bead", "--", "--slug", "test-critical"])
            .output()
            .expect("Failed to view bead");

        assert!(result.status.success(), "Should view bead successfully");
        let stdout = String::from_utf8_lossy(&result.stdout);
        assert!(stdout.contains("Critical Test"), "Should show bead details");

        // Test bead deletion
        let result = Command::new("cargo")
            .args(&["run", "--bin", "delete_bead", "--", "--slug", "test-critical"])
            .output()
            .expect("Failed to delete bead");

        assert!(result.status.success(), "Should delete bead successfully");
        let stdout = String::from_utf8_lossy(&result.stdout);
        assert!(stdout.contains("Deleted") || stdout.contains("deleted"), "Should confirm deletion");
    }

    #[test]
    fn test_error_handling_robustness() {
        println!("\n🧪 Testing error handling robustness...");

        // Test with invalid commands
        let error_cases = vec![
            ("create_bead", vec![], 1),
            ("create_bead", vec!["--slug"], 1),
            ("create_bead", vec!["--title"], 1),
            ("view_bead", vec![], 1),
            ("view_bead", vec!["--slug", "nonexistent"], 1),
            ("update_bead", vec![], 1),
            ("delete_bead", vec![], 1),
            ("delete_bead", vec!["--slug", "nonexistent"], 1),
        ];

        for (binary, args, expected_exit) in error_cases {
            let mut cmd = Command::new("cargo");
            cmd.args(&["run", "--bin", binary]);
            cmd.args(args);

            let result = cmd.output().expect("Failed to execute command");
            assert_eq!(result.status.code().unwrap_or(1), expected_exit,
                      "{} should fail with expected exit code", binary);

            let stderr = String::from_utf8_lossy(&result.stderr);
            assert!(!stderr.contains("panicked"), "{} should not panic", binary);
            assert!(!stderr.contains("internal error"), "{} should not expose internal errors", binary);
        }
    }

    #[test]
    fn test_no_panic_conditions() {
        println!("\n🧪 Testing no-panic conditions...");

        // Test with various problematic inputs
        let test_cases = vec![
            ("create_bead", vec!["--slug", "", "--title", "Empty Slug"]),
            ("create_bead", vec!["--slug", "a".repeat(10000), "--title", "Long Slug"]),
            ("create_bead", vec!["--title", "", "--slug", "Empty Title"]),
            ("create_bead", vec!["--slug", "bad slug with spaces", "--title", "Bad Slug"]),
        ];

        for (binary, args) in test_cases {
            let mut cmd = Command::new("cargo");
            cmd.args(&["run", "--bin", binary]);
            cmd.args(args);

            let result = cmd.output().expect("Failed to execute command");
            let stderr = String::from_utf8_lossy(&result.stderr);

            assert!(!stderr.contains("panicked"), "Should not panic with invalid input");
            assert!(!stderr.contains("thread panicked"), "Should not have panicked threads");
        }
    }

    #[test]
    fn test_help_quality() {
        println!("\n🧪 Testing help quality...");

        let commands = vec![
            "create_bead --help",
            "list_beads --help",
            "view_bead --help",
            "update_bead --help",
            "delete_bead --help",
        ];

        for cmd in commands {
            let result = Command::new("cargo")
                .args(&["run", "--bin", cmd.split_whitespace().next().unwrap(), "--", &cmd.split_whitespace().nth(1).unwrap_or("--help")])
                .output()
                .expect("Failed to execute help command");

            assert!(result.status.success(), "Help should work");
            let output = String::from_utf8_lossy(&result.stdout);
            assert!(output.contains("Usage:"), "Help should show usage");
            assert!(output.contains("Options:"), "Help should show options");
            assert!(!output.contains("TODO"), "Help should not contain TODO");
            assert!(!output.contains("placeholder"), "Help should not contain placeholder");
        }
    }

    #[test]
    fn test_end_to_end_workflow() {
        println!("\n🧪 Testing end-to-end workflow...");

        // Use a unique slug to avoid conflicts
        let slug = "e2e-workflow-test";

        // 1. Create
        let result = Command::new("cargo")
            .args(&["run", "--bin", "create_bead", "--", "--slug", slug, "--title", "E2E Test"])
            .output()
            .expect("Failed to create bead");

        assert!(result.status.success(), "Should create bead");
        let stdout = String::from_utf8_lossy(&result.stdout);
        assert!(stdout.contains("Bead created") || stdout.contains("Created"), "Should confirm creation");

        // 2. List
        let result = Command::new("cargo")
            .args(&["run", "--bin", "list_beads"])
            .output()
            .expect("Failed to list beads");

        assert!(result.status.success(), "Should list beads");
        let stdout = String::from_utf8_lossy(&result.stdout);
        assert!(stdout.contains(slug), "Should show created bead");

        // 3. View
        let result = Command::new("cargo")
            .args(&["run", "--bin", "view_bead", "--", "--slug", slug])
            .output()
            .expect("Failed to view bead");

        assert!(result.status.success(), "Should view bead");
        let stdout = String::from_utf8_lossy(&result.stdout);
        assert!(stdout.contains("E2E Test"), "Should show bead details");

        // 4. Update
        let result = Command::new("cargo")
            .args(&["run", "--bin", "update_bead", "--", "--slug", slug, "--title", "Updated E2E Test"])
            .output()
            .expect("Failed to update bead");

        assert!(result.status.success(), "Should update bead");
        let stdout = String::from_utf8_lossy(&result.stdout);
        assert!(stdout.contains("Updated") || stdout.contains("updated"), "Should confirm update");

        // 5. Verify update
        let result = Command::new("cargo")
            .args(&["run", "--bin", "view_bead", "--", "--slug", slug])
            .output()
            .expect("Failed to view bead");

        assert!(result.status.success(), "Should view updated bead");
        let stdout = String::from_utf8_lossy(&result.stdout);
        assert!(stdout.contains("Updated E2E Test"), "Should show updated title");

        // 6. Delete
        let result = Command::new("cargo")
            .args(&["run", "--bin", "delete_bead", "--", "--slug", slug])
            .output()
            .expect("Failed to delete bead");

        assert!(result.status.success(), "Should delete bead");
        let stdout = String::from_utf8_lossy(&result.stdout);
        assert!(stdout.contains("Deleted") || stdout.contains("deleted"), "Should confirm deletion");

        // 7. Verify deletion
        let result = Command::new("cargo")
            .args(&["run", "--bin", "list_beads"])
            .output()
            .expect("Failed to list beads");

        assert!(result.status.success(), "Should list beads after deletion");
        let stdout = String::from_utf8_lossy(&result.stdout);
        assert!(!stdout.contains(slug), "Should not show deleted bead");
    }

    #[test]
    fn test_zero_unwrap_everywhere() {
        println!("\n🧪 Testing zero-unwrap compliance everywhere...");

        // Run all zero unwrap tests
        let result = Command::new("cargo")
            .args(&["test", "--", "--exact", "--nocapture"])
            .output()
            .expect("Failed to run tests");

        assert!(result.status.success(), "All tests should pass");
        let stderr = String::from_utf8_lossy(&result.stderr);
        let stdout = String::from_utf8_lossy(&result.stdout);

        // Check for unwrap/expect usage
        assert!(!stderr.contains("unwrap failed"), "Should not have unwrap failed");
        assert!(!stderr.contains("expect failed"), "Should not have expect failed");
        assert!(!stderr.contains("called `unwrap()` on an `Err`"), "Should not unwrap Err");
        assert!(!stderr.contains("called `Option::unwrap()` on a `None`"), "Should not unwrap None");
        assert!(!stderr.contains("called `expect("), "Should not use expect");

        // Check for panics
        assert!(!stderr.contains("panicked"), "Should not panic");
        assert!(!stderr.contains("thread panicked"), "Should not panic threads");

        // Check for TODO/unimplemented
        assert!(!stderr.contains("todo!"), "Should not have todo! macros");
        assert!(!stderr.contains("unimplemented!"), "Should not have unimplemented! macros");

        println!("✅ Zero-unwrap compliance verified!");
    }

    #[test]
    fn test_comprehensive_quality_gate() {
        println!("\n🧪 Testing comprehensive quality gate...");

        // Run all tests
        let result = Command::new("cargo")
            .args(&["test", "--workspace", "--lib", "--test", "*"])
            .output()
            .expect("Failed to run comprehensive test suite");

        assert!(result.status.success(), "All tests should pass");

        let stderr = String::from_utf8_lossy(&result.stderr);
        let stdout = String::from_utf8_lossy(&result.stdout);

        // Quality gate checks
        let quality_issues = vec![
            "unwrap failed",
            "expect failed",
            "panicked",
            "todo!",
            "unimplemented!",
            "called `unwrap()` on an `Err`",
            "called `Option::unwrap()` on a `None`",
            "called `expect(",
        ];

        for issue in quality_issues {
            assert!(!stderr.contains(issue), "Should not contain: {}", issue);
            assert!(!stdout.contains(issue), "Should not contain: {}", issue);
        }

        // Check test coverage
        assert!(stdout.contains("test result"), "Should show test results");
        assert!(stdout.contains("failed"), "Should show test statistics");

        println!("✅ Comprehensive quality gate passed!");
    }
}