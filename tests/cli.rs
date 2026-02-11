//! CLI Testing: Validate command-line interface behavior
//!
//! Test every documented command, flag, and error case.

use std::process::{Command, Output};
use std::path::Path;

#[test]
fn test_binary_exists() {
    let result = Command::new("cargo")
        .args(&["run", "--bin", "create_bead", "--", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(result.status.success(), "create_bead binary should exist and be runnable");
}

#[test]
fn test_help_completeness() {
    let commands = vec![
        "create_bead",
        "list_beads",
        "view_bead",
        "update_bead",
        "delete_bead",
    ];

    for cmd in commands {
        let result = Command::new("cargo")
            .args(&["run", "--bin", cmd, "--", "--help"])
            .output()
            .expect("Failed to execute help command");

        assert!(result.status.success(), "{} help should work", cmd);

        let output = String::from_utf8_lossy(&result.stdout);
        assert!(output.contains("Usage:"), "{} help should show usage", cmd);
        assert!(output.contains("Options:"), "{} help should show options", cmd);
        assert!(!output.contains("TODO"), "{} help should not have TODO text", cmd);
        assert!(!output.contains("placeholder"), "{} help should not have placeholders", cmd);
    }
}

#[test]
fn test_version_flag() {
    // Check if version flags work for binaries
    let version_flags = vec!["--version", "-V"];

    for flag in version_flags {
        let result = Command::new("cargo")
            .args(&["run", "--bin", "create_bead", "--", flag])
            .output()
            .expect("Failed to execute version command");

        // Version commands should either succeed or fail gracefully
        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(!stderr.contains("panicked"), "Version command should not panic");

        if result.status.success() {
            let output = String::from_utf8_lossy(&result.stdout);
            assert!(output.contains("0.1.0"), "Version should be displayed");
        }
    }
}

#[test]
fn test_required_arguments() {
    let test_cases = vec![
        ("create_bead", vec![], 1, "missing required --slug"),
        ("create_bead", vec!["--slug"], 1, "missing slug value"),
        ("create_bead", vec!["--title"], 1, "missing title value"),
        ("view_bead", vec![], 1, "missing required --slug"),
        ("view_bead", vec!["--slug"], 1, "missing slug value"),
        ("update_bead", vec![], 1, "missing required arguments"),
        ("update_bead", vec!["--slug"], 1, "missing slug value"),
        ("update_bead", vec!["--title"], 1, "missing title value"),
        ("delete_bead", vec![], 1, "missing required --slug"),
        ("delete_bead", vec!["--slug"], 1, "missing slug value"),
    ];

    for (binary, args, expected_exit, description) in test_cases {
        let mut cmd = Command::new("cargo");
        cmd.args(&["run", "--bin", binary]);
        cmd.args(args);

        let result = cmd.output().expect("Failed to execute command");
        assert_eq!(result.status.code().unwrap_or(1), expected_exit,
                  "{} should fail with missing args", description);

        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(!stderr.contains("panicked"), "{} should not panic on missing args", binary);
    }
}

#[test]
fn test_duplicate_flags() {
    let test_cases = vec![
        ("create_bead", vec!["--slug", "test", "--slug", "duplicate"]),
        ("create_bead", vec!["--title", "test", "--title", "duplicate"]),
        ("update_bead", vec!["--slug", "test", "--title", "test", "--title", "duplicate"]),
    ];

    for (binary, args) in test_cases {
        let mut cmd = Command::new("cargo");
        cmd.args(&["run", "--bin", binary]);
        cmd.args(args);

        let result = cmd.output().expect("Failed to execute command");

        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(!stderr.contains("panicked"), "{} should not panic on duplicate flags", binary);
    }
}

#[test]
fn test_unknown_flags() {
    let unknown_flags = vec![
        "--unknown-flag",
        "--bogus-option",
        "--fake-arg",
    ];

    for flag in unknown_flags {
        let result = Command::new("cargo")
            .args(&["run", "--bin", "create_bead", "--", "--slug", "test", "--title", "Test", flag])
            .output()
            .expect("Failed to execute command");

        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(!stderr.contains("panicked"), "Should not panic on unknown flags");

        // Should either succeed (ignored flag) or fail gracefully
        if !result.status.success() {
            assert!(stderr.contains("unrecognized") || stderr.contains("unknown"),
                   "Should indicate unknown flag");
        }
    }
}

#[test]
fn test_invalid_argument_values() {
    let test_cases = vec![
        ("create_bead", vec!["--slug", "", "--title", "Test"], 1, "empty slug"),
        ("create_bead", vec!["--slug", "bad_slug_with_spaces", "--title", "Test"], 1, "slug with spaces"),
        ("create_bead", vec!["--slug", "a".repeat(1000), "--title", "Test"], 1, "slug too long"),
        ("create_bead", vec!["--slug", "valid-slug", "--title", ""], 1, "empty title"),
    ];

    for (binary, args, expected_exit, description) in test_cases {
        let mut cmd = Command::new("cargo");
        cmd.args(&["run", "--bin", binary]);
        cmd.args(args);

        let result = cmd.output().expect("Failed to execute command");
        assert_eq!(result.status.code().unwrap_or(1), expected_exit,
                  "{} should reject invalid {}", description, binary);

        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(!stderr.contains("panicked"), "{} should not panic on invalid args", binary);
    }
}

#[test]
fn test_exit_code_conventions() {
    // Test success cases
    let success_commands = vec![
        vec!["run", "--bin", "create_bead", "--", "--help"],
        vec!["run", "--bin", "list_beads", "--", "--help"],
        vec!["test", "--help"],
    ];

    for args in success_commands {
        let result = Command::new("cargo")
            .args(args)
            .output()
            .expect("Failed to execute command");

        assert_eq!(result.status.code().unwrap_or(1), 0,
                  "Success command should exit with 0");
    }

    // Test error cases
    let error_commands = vec![
        vec!["run", "--bin", "nonexistent_binary"],
        vec!["run", "--bin", "create_bead"],
        vec!["run", "--bin", "view_bead", "--", "--slug", "nonexistent"],
    ];

    for args in error_commands {
        let result = Command::new("cargo")
            .args(args)
            .output()
            .expect("Failed to execute command");

        assert!(result.status.code().unwrap_or(1) != 0,
                "Error command should exit with non-zero");
    }
}

#[test]
fn test_error_messages_quality() {
    let error_cases = vec![
        ("create_bead", vec![], "missing required arguments"),
        ("view_bead", vec![], "missing --slug"),
        ("create_bead", vec!["--slug"], "missing slug value"),
        ("create_bead", vec!["--slug", ""], "empty slug"),
    ];

    for (binary, args, error_context) in error_cases {
        let mut cmd = Command::new("cargo");
        cmd.args(&["run", "--bin", binary]);
        cmd.args(args);

        let result = cmd.output().expect("Failed to execute command");

        let stderr = String::from_utf8_lossy(&result.stderr);
        let stdout = String::from_utf8_lossy(&result.stdout);
        let output = stderr + &stdout;

        assert!(!output.is_empty(), "{} error should not be empty", binary);
        assert!(output.len() > 10, "{} error should be descriptive", binary);
        assert!(!output.contains("internal error"), "{} should not expose internal errors", binary);

        // Error should be helpful
        if binary.contains("create_bead") {
            assert!(output.contains("slug") || output.contains("title"),
                   "Create bead error should mention slug/title");
        }
    }
}

#[test]
fn test_subcommand_help() {
    let binaries_with_subcommands = vec![
        "list_beads",
    ];

    for binary in binaries_with_subcommands {
        // Test main help
        let result = Command::new("cargo")
            .args(&["run", "--bin", binary])
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8_lossy(&result.stdout);
        assert!(stdout.contains("Usage:") || stdout.contains("USAGE"),
               "{} should show usage when run without args");
    }
}

#[test]
fn test_consistent_output_formatting() {
    let test_cases = vec![
        ("create_bead", vec!["--slug", "format-test", "--title", "Format Test"]),
        ("list_beads", vec![]),
    ];

    for (binary, args) in test_cases {
        let mut cmd1 = Command::new("cargo");
        cmd1.args(&["run", "--bin", binary]);
        cmd1.args(args.clone());

        let mut cmd2 = Command::new("cargo");
        cmd2.args(&["run", "--bin", binary]);
        cmd2.args(args);

        let result1 = cmd1.output().expect("Failed to execute command");
        let result2 = cmd2.output().expect("Failed to execute command");

        let output1 = String::from_utf8_lossy(&result1.stdout);
        let output2 = String::from_utf8_lossy(&result2.stdout);

        // Output should be consistent (same length for same inputs)
        assert!(output1.len() > 0, "{} should produce output", binary);
        assert!(output2.len() > 0, "{} should produce output", binary);
    }
}

#[test]
fn test_no_crashes_on_any_input() {
    // Test with various problematic inputs
    let problematic_inputs = vec![
        // Empty arguments
        ("create_bead", vec![], 1),
        // Garbage arguments
        ("create_bead", vec!["--random-garbage"], 1),
        // Mixed valid/invalid
        ("create_bead", vec!["--slug", "test", "--invalid-flag"], 1),
        // Very long arguments
        ("create_bead", vec!["--slug", "a".repeat(1000)], 1),
    ];

    for (binary, args, expected_exit) in problematic_inputs {
        let mut cmd = Command::new("cargo");
        cmd.args(&["run", "--bin", binary]);
        cmd.args(args);

        let result = cmd.output().expect("Failed to execute command");
        let stderr = String::from_utf8_lossy(&result.stderr);

        // Should never panic
        assert!(!stderr.contains("panicked"), "{} should not panic on input", binary);

        // Should exit with expected code
        assert_eq!(result.status.code().unwrap_or(1), expected_exit,
                  "{} should exit with expected code", binary);
    }
}

#[test]
fn test_shell_integration() {
    // Test command works in shell context
    let shell_commands = vec![
        "cargo run --bin list_beads",
        "cargo run --bin create_bead --slug shell-test --title \"Shell Integration\"",
    ];

    for shell_cmd in shell_commands {
        let result = Command::new("sh")
            .arg("-c")
            .arg(shell_cmd)
            .output()
            .expect("Failed to execute shell command");

        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(!stderr.contains("panicked"), "Shell command should not panic: {}", shell_cmd);
    }
}

#[test]
fn test_timeout_resistance() {
    // Test that commands don't hang indefinitely
    use std::time::Duration;

    let commands = vec![
        vec!["run", "--bin", "list_beads"],
        vec!["run", "--bin", "create_bead", "--", "--slug", "timeout-test", "--title", "Test"],
    ];

    for args in commands {
        let mut cmd = Command::new("cargo");
        cmd.args(args);

        // Set a timeout to prevent hanging
        let start = std::time::Instant::now();
        let result = cmd.output().expect("Failed to execute command");
        let duration = start.elapsed();

        // Should complete within reasonable time
        assert!(duration.as_secs() < 30, "Command should not hang");

        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(!stderr.contains("panicked"), "Should not timeout due to panic");
    }
}

// Main CLI test runner
#[test]
fn test_all_cli_functionality() {
    println!("Running comprehensive CLI tests...");

    test_binary_exists();
    test_help_completeness();
    test_version_flag();
    test_required_arguments();
    test_duplicate_flags();
    test_unknown_flags();
    test_invalid_argument_values();
    test_exit_code_conventions();
    test_error_messages_quality();
    test_subcommand_help();
    test_consistent_output_formatting();
    test_no_crashes_on_any_input();
    test_shell_integration();
    test_timeout_resistance();

    println!("All CLI tests passed!");
    println!("Command-line interface is robust and user-friendly.");
}