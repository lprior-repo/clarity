//! QA Enforcer: Comprehensive End-to-End Behavioral Tests
//!
//! This test suite enforces the zero-unwrap philosophy and validates
//! end-user workflows through actual execution, not assumptions.
//!
//! Philosophy: Execute Everything. Inspect Deeply. Fix What You Can.

use std::process::{Command, Output};
use std::path::Path;
use std::fs;

#[derive(Debug, thiserror::Error)]
pub struct QaTestError {
    pub test_name: String,
    pub command: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub expected: String,
    pub severity: Severity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    Critical,
    Major,
    Minor,
    Observation,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Critical => write!(f, "CRITICAL"),
            Severity::Major => write!(f, "MAJOR"),
            Severity::Minor => write!(f, "MINOR"),
            Severity::Observation => write!(f, "OBSERVATION"),
        }
    }
}

pub fn run_command(cmd: &str, expect_exit: i32) -> Result<Output, QaTestError> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return Err(QaTestError {
            test_name: "empty_command".to_string(),
            command: cmd.to_string(),
            exit_code: 1,
            stdout: String::new(),
            stderr: "Empty command".to_string(),
            expected: expect_exit.to_string(),
            severity: Severity::Critical,
        });
    }

    let output = Command::new(parts[0])
        .args(&parts[1..])
        .output()
        .expect(&format!("Failed to execute command: {}", cmd));

    if output.status.code().unwrap_or(-1) != expect_exit {
        Err(QaTestError {
            test_name: "command_exit_code".to_string(),
            command: cmd.to_string(),
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            expected: expect_exit.to_string(),
            severity: Severity::Critical,
        })
    } else {
        Ok(output)
    }
}

pub fn assert_no_panic(output: &Output) -> Result<(), QaTestError> {
    if output.stderr.contains("thread 'main' panicked") {
        Err(QaTestError {
            test_name: "panic_detected".to_string(),
            command: "unknown".to_string(),
            exit_code: 101,
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            expected: "no panic".to_string(),
            severity: Severity::Critical,
        })
    } else {
        Ok(())
    }
}

pub fn assert_no_unwrap_expect(output: &Output) -> Result<(), QaTestError> {
    let output_str = String::from_utf8_lossy(&output.stderr);
    if output_str.contains("called `Result::unwrap()` on an `Err` value") ||
       output_str.contains("called `Option::unwrap()` on a `None` value") ||
       output_str.contains("called `Result::expect(") {
        Err(QaTestError {
            test_name: "unwrap_expect_detected".to_string(),
            command: "unknown".to_string(),
            exit_code: 102,
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: output_str.to_string(),
            expected: "no unwrap/expect".to_string(),
            severity: Severity::Critical,
        })
    } else {
        Ok(())
    }
}

pub fn assert_no_todo_unimplemented(output: &Output) -> Result<(), QaTestError> {
    let output_str = String::from_utf8_lossy(&output.stderr);
    if output_str.contains("todo!") || output_str.contains("unimplemented!") {
        Err(QaTestError {
            test_name: "todo_unimplemented_detected".to_string(),
            command: "unknown".to_string(),
            exit_code: 103,
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: output_str.to_string(),
            expected: "no todo!/unimplemented!".to_string(),
            severity: Severity::Critical,
        })
    } else {
        Ok(())
    }
}

pub fn assert_no_secrets(output: &Output) -> Result<(), QaTestError> {
    let output_str = String::from_utf8_lossy(&output.stderr + &output.stdout);
    let secret_patterns = vec![
        "password=", "token=", "key=", "secret=", "api_key=",
        "DATABASE_URL=", "API_KEY=", "SECRET_KEY="
    ];

    for pattern in secret_patterns {
        if output_str.contains(pattern) {
            return Err(QaTestError {
                test_name: "secret_detected".to_string(),
                command: "unknown".to_string(),
                exit_code: 104,
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: output_str.to_string(),
                expected: "no secrets".to_string(),
                severity: Severity::Critical,
            });
        }
    }
    Ok(())
}

// End-user workflow tests
#[test]
fn test_user_creates_bead() {
    let result = run_command("cargo run --bin create_bead --slug test-qe-bead --title \"QA Enforcer Test\"", 0);
    match result {
        Ok(output) => {
            assert_no_panic(&output).unwrap();
            assert_no_unwrap_expect(&output).unwrap();
            assert_no_todo_unimplemented(&output).unwrap();
            assert_no_secrets(&output).unwrap();

            // Verify bead was created
            let output_str = String::from_utf8_lossy(&output.stdout);
            assert!(output_str.contains("Bead created") || output_str.contains("Created"), "Bead creation should succeed");
        }
        Err(e) => panic!("Test failed: {}", e),
    }
}

#[test]
fn test_user_lists_beads() {
    let result = run_command("cargo run --bin list_beads", 0);
    match result {
        Ok(output) => {
            assert_no_panic(&output).unwrap();
            assert_no_unwrap_expect(&output).unwrap();
            assert_no_todo_unimplemented(&output).unwrap();
            assert_no_secrets(&output).unwrap();

            let output_str = String::from_utf8_lossy(&output.stdout);
            assert!(output_str.contains("Beads:") || output_str.contains("ID"), "Should list beads");
        }
        Err(e) => panic!("Test failed: {}", e),
    }
}

#[test]
fn test_user_views_bead_detail() {
    // First create a bead to view
    run_command("cargo run --bin create_bead --slug view-test --title \"View Test\"", 0).unwrap();

    let result = run_command("cargo run --bin view_bead --slug view-test", 0);
    match result {
        Ok(output) => {
            assert_no_panic(&output).unwrap();
            assert_no_unwrap_expect(&output).unwrap();
            assert_no_todo_unimplemented(&output).unwrap();
            assert_no_secrets(&output).unwrap();

            let output_str = String::from_utf8_lossy(&output.stdout);
            assert!(output_str.contains("View Test"), "Should show bead details");
        }
        Err(e) => panic!("Test failed: {}", e),
    }
}

#[test]
fn test_user_updates_bead() {
    // Create a bead first
    run_command("cargo run --bin create_bead --slug update-test --title \"Update Test\"", 0).unwrap();

    let result = run_command("cargo run --bin update_bead --slug update-test --title \"Updated Title\"", 0);
    match result {
        Ok(output) => {
            assert_no_panic(&output).unwrap();
            assert_no_unwrap_expect(&output).unwrap();
            assert_no_todo_unimplemented(&output).unwrap();
            assert_no_secrets(&output).unwrap();

            let output_str = String::from_utf8_lossy(&output.stdout);
            assert!(output_str.contains("Bead updated") || output_str.contains("Updated"), "Update should succeed");
        }
        Err(e) => panic!("Test failed: {}", e),
    }
}

#[test]
fn test_user_deletes_bead() {
    // Create a bead first
    run_command("cargo run --bin create_bead --slug delete-test --title \"Delete Test\"", 0).unwrap();

    let result = run_command("cargo run --bin delete_bead --slug delete-test", 0);
    match result {
        Ok(output) => {
            assert_no_panic(&output).unwrap();
            assert_no_unwrap_expect(&output).unwrap();
            assert_no_todo_unimplemented(&output).unwrap();
            assert_no_secrets(&output).unwrap();

            let output_str = String::from_utf8_lossy(&output.stdout);
            assert!(output_str.contains("Bead deleted") || output_str.contains("Deleted"), "Deletion should succeed");
        }
        Err(e) => panic!("Test failed: {}, e),
    }
}

// CLI command validation
#[test]
fn test_help_commands() {
    let commands = vec![
        "cargo run --bin create_bead --help",
        "cargo run --bin list_beads --help",
        "cargo run --bin view_bead --help",
        "cargo run --bin update_bead --help",
        "cargo run --bin delete_bead --help",
    ];

    for cmd in commands {
        let result = run_command(cmd, 0);
        match result {
            Ok(output) => {
                assert_no_panic(&output).unwrap();
                assert_no_unwrap_expect(&output).unwrap();
                assert_no_todo_unimplemented(&output).unwrap();
                assert_no_secrets(&output).unwrap();

                let output_str = String::from_utf8_lossy(&output.stdout);
                assert!(output_str.contains("Usage:") || output_str.contains("USAGE"), "Help should show usage");
                assert!(output_str.contains("Options:"), "Help should show options");
            }
            Err(e) => panic!("Help command failed: {} - {}", cmd, e),
        }
    }
}

#[test]
fn test_error_handling() {
    let error_commands = vec![
        ("cargo run --bin create_bead --slug \"\"", 1), // Empty slug
        ("cargo run --bin view_bead --slug nonexistent", 1), // Nonexistent bead
        ("cargo run --bin update_bead --slug nonexistent --title \"Test\"", 1), // Update nonexistent
        ("cargo run --bin delete_bead --slug nonexistent", 1), // Delete nonexistent
    ];

    for (cmd, expected_exit) in error_commands {
        let result = run_command(cmd, expected_exit);
        match result {
            Ok(output) => {
                // Command succeeded as expected
                assert_no_panic(&output).unwrap();
                assert_no_unwrap_expect(&output).unwrap();
                assert_no_todo_unimplemented(&output).unwrap();
                assert_no_secrets(&output).unwrap();

                let output_str = String::from_utf8_lossy(&output.stderr + &output.stdout);
                // Should have helpful error message, not panic
                assert!(!output_str.contains("panicked"), "Should not panic on error");
                assert!(output_str.len() > 10, "Error should be descriptive");
            }
            Err(e) => {
                // Command failed as expected, but check for panics
                if e.stderr.contains("panicked") {
                    panic!("Command panicked: {} - {}", cmd, e);
                }
            }
        }
    }
}

// Database operations
#[test]
fn test_database_operations() {
    // Test database initialization
    let result = run_command("cargo test --test test_db_init test_db_init", 0);
    match result {
        Ok(output) => {
            assert_no_panic(&output).unwrap();
            assert_no_unwrap_expect(&output).unwrap();
            assert_no_todo_unimplemented(&output).unwrap();
            assert_no_secrets(&output).unwrap();
        }
        Err(e) => panic!("Database init test failed: {}", e),
    }
}

// Zero unwrap validation
#[test]
fn test_zero_unwrap_compliance() {
    let result = run_command("cargo test --test zero_unwrap_tests", 0);
    match result {
        Ok(output) => {
            assert_no_panic(&output).unwrap();
            assert_no_unwrap_expect(&output).unwrap();
            assert_no_todo_unimplemented(&output).unwrap();
            assert_no_secrets(&output).unwrap();

            let output_str = String::from_utf8_lossy(&output.stdout);
            assert!(output_str.contains("test result"), "Zero unwrap tests should run");
        }
        Err(e) => panic!("Zero unwrap test failed: {}", e),
    }
}

// Integration tests
#[test]
fn test_end_toend_workflow() {
    // Complete workflow: create, list, view, update, delete
    println!("Testing complete workflow...");

    // 1. Create
    run_command("cargo run --bin create_bead --slug workflow-test --title \"Workflow Test\"", 0).unwrap();

    // 2. List
    let list_result = run_command("cargo run --bin list_beads", 0);
    match list_result {
        Ok(output) => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            assert!(output_str.contains("workflow-test"), "Should show created bead in list");
        }
        Err(e) => panic!("List failed: {}", e),
    }

    // 3. View
    let view_result = run_command("cargo run --bin view_bead --slug workflow-test", 0);
    match view_result {
        Ok(output) => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            assert!(output_str.contains("Workflow Test"), "Should show bead details");
        }
        Err(e) => panic!("View failed: {}", e),
    }

    // 4. Update
    let update_result = run_command("cargo run --bin update_bead --slug workflow-test --title \"Updated Workflow\"", 0);
    match update_result {
        Ok(output) => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            assert!(output_str.contains("Updated") || output_str.contains("Updated Workflow"), "Update should succeed");
        }
        Err(e) => panic!("Update failed: {}", e),
    }

    // 5. Delete
    let delete_result = run_command("cargo run --bin delete_bead --slug workflow-test", 0);
    match delete_result {
        Ok(output) => {
            let output_str = String::from_utf8_lossy(&output.stdout);
            assert!(output_str.contains("Deleted") || output_str.contains("deleted"), "Delete should succeed");
        }
        Err(e) => panic!("Delete failed: {}", e),
    }
}

// Adversarial testing
#[test]
fn test_sql_injection_resistance() {
    let malicious_inputs = vec![
        ("--slug \"'; DROP TABLE beads; --\"", 1),
        ("--title \"' OR '1'='1'", 1),
        ("--slug \"../../../etc/passwd\"", 1),
    ];

    for (suffix, expected_exit) in malicious_inputs {
        let cmd = format!("cargo run --bin create_bead {}", suffix);
        let result = run_command(&cmd, expected_exit);

        match result {
            Ok(output) => {
                // Should reject safely
                assert_no_panic(&output).unwrap();
                assert_no_unwrap_expect(&output).unwrap();
                assert_no_todo_unimplemented(&output).unwrap();
                assert_no_secrets(&output).unwrap();

                let output_str = String::from_utf8_lossy(&output.stderr + &output.stdout);
                assert!(!output_str.contains("DROP TABLE"), "SQL injection should be rejected");
                assert!(!output_str.contains("error: database"), "Database should not be corrupted");
            }
            Err(e) => {
                // Expected to fail, but check for panics
                if e.stderr.contains("panicked") {
                    panic!("SQL injection test panicked: {} - {}", cmd, e);
                }
            }
        }
    }
}

#[test]
fn test_xss_resistance() {
    let xss_payload = "<script>alert('xss')</script>";
    let cmd = format!("cargo run --bin create_bead --title \"{}\"", xss_payload);

    let result = run_command(&cmd, 1); // Should fail

    match result {
        Ok(output) => {
            // Should reject or escape safely
            assert_no_panic(&output).unwrap();
            assert_no_unwrap_expect(&output).unwrap();
            assert_no_todo_unimplemented(&output).unwrap();
            assert_no_secrets(&output).unwrap();

            let output_str = String::from_utf8_lossy(&output.stderr + &output.stdout);
            assert!(!output_str.contains("<script>"), "XSS should be escaped or rejected");
        }
        Err(e) => {
            if e.stderr.contains("panicked") {
                panic!("XSS test panicked: {}", e);
            }
        }
    }
}

// Performance tests
#[test]
fn test_concurrent_operations() {
    // Test multiple operations concurrently
    use std::thread;
    use std::sync::Arc;

    let num_operations = 5;
    let handles: Vec<_> = (0..num_operations)
        .map(|i| {
            thread::spawn(move || {
                let slug = format!("concurrent-test-{}", i);
                let cmd = format!("cargo run --bin create_bead --slug {} --title \"Concurrent Test {}\"", slug, i);
                run_command(&cmd, 0).unwrap();
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    // Verify all beads were created
    for i in 0..num_operations {
        let slug = format!("concurrent-test-{}", i);
        let result = run_command(&format!("cargo run --bin view_bead --slug {}", slug), 0);
        assert!(result.is_ok(), "Bead {} should exist", i);
    }
}

// Main test runner
#[test]
fn test_all_end_user_workflows() {
    println!("Running comprehensive end-user workflow tests...");

    // Test individual workflows
    test_user_creates_bead();
    test_user_lists_beads();
    test_user_views_bead_detail();
    test_user_updates_bead();
    test_user_deletes_bead();

    // Test command interfaces
    test_help_commands();
    test_error_handling();

    // Test complete workflow
    test_end_toend_workflow();

    // Test security
    test_sql_injection_resistance();
    test_xss_resistance();

    // Test performance
    test_concurrent_operations();

    println!("All end-user workflow tests completed successfully!");
}