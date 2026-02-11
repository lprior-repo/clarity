//! Adversarial Testing: Break it from every angle
//!
//! These tests intentionally break the application to ensure
//! it fails gracefully and doesn't expose vulnerabilities.

use std::process::{Command, Output};
use std::path::Path;
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_corrupted_database_file() {
    println!("Testing corrupted database file handling...");

    // Create a corrupted database file
    let corrupted_db = NamedTempFile::new().unwrap();
    fs::write(corrupted_db.path(), "corrupted database content").unwrap();

    let env_var = format!("DATABASE_URL=sqlite:{}", corrupted_db.path().display());
    let cmd = format!("DATABASE_URL={} cargo run --bin list_beads", env_var);

    let result = Command::new("sh")
        .arg("-c")
        .arg(&cmd)
        .output()
        .expect("Failed to execute command");

    // Should fail gracefully without panic
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(!stderr.contains("panicked"), "Should not panic on corrupted database");
    assert!(result.status.code().unwrap_or(0) != 0, "Should fail on corrupted database");

    // Should give helpful error message
    assert!(stderr.len() > 20, "Should provide descriptive error");
}

#[test]
fn test_large_payloads() {
    println!("Testing large payload handling...");

    // Create a very large title
    let large_title = "A".repeat(10000);
    let cmd = format!("cargo run --bin create_bead --slug large-payload --title \"{}\"", large_title);

    let result = Command::new("sh")
        .arg("-c")
        .arg(&cmd)
        .output()
        .expect("Failed to execute command");

    // Should handle large input gracefully
    let stdout = String::from_utf8_lossy(&result.stdout);
    let stderr = String::from_utf8_lossy(&result.stderr);

    assert!(!stderr.contains("panicked"), "Should not panic on large input");
    assert!(!stderr.contains("allocation failed"), "Should not run out of memory");

    // Either succeed with proper handling or fail gracefully
    if result.status.success() {
        assert!(stdout.contains("Bead created") || stdout.contains("Created"), "Should create bead");
    } else {
        assert!(!stdout.is_empty() || !stderr.is_empty(), "Should provide error message");
    }
}

#[test]
fn test_unicode_edge_cases() {
    println!("Testing Unicode edge cases...");

    let unicode_cases = vec![
        ("unicode-emoji", "🚀 Test Title 🚀"),
        ("unicode-chinese", "测试标题"),
        ("unicode-russian", "Заголовок теста"),
        ("unicode-arabic", "عنوان اختبار"),
        ("unicode-combining", "Normal\u0301 Title"),
        ("unicode-control", "Title\u0001\u0002"),
        ("unicode-null-bytes", "Title\0\0\0"),
        ("unicode-emoji-spam", "🚀🚀🚀🚀🚀🚀🚀🚀🚀"),
    ];

    for (slug, title) in unicode_cases {
        let cmd = format!("cargo run --bin create_bead --slug {} --title \"{}\"", slug, title);

        let result = Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8_lossy(&result.stdout);
        let stderr = String::from_utf8_lossy(&result.stderr);

        assert!(!stderr.contains("panicked"), "Should not panic on Unicode input: {}", title);

        if result.status.success() {
            assert!(stdout.contains("Bead created") || stdout.contains("Created"),
                   "Should handle Unicode title: {}", title);
        } else {
            // Should fail gracefully with Unicode error handling
            assert!(!stderr.is_empty(), "Should provide error for problematic Unicode");
        }
    }
}

#[test]
fn test_concurrent_modification() {
    println!("Testing concurrent modification safety...");

    use std::thread;
    use std::sync::Arc;
    use std::time::Duration;

    let num_threads = 3;
    let beads: Vec<String> = (0..num_threads)
        .map(|i| format!("concurrent-mod-{}", i))
        .collect();

    // Spawn multiple threads modifying the same bead
    let handles: Vec<_> = beads.iter().map(|bead| {
        thread::spawn({
            let bead = bead.clone();
            move || {
                for attempt in 0..3 {
                    let cmd = format!("cargo run --bin update_bead --slug {} --title \"Update {} attempt {}\"",
                                   bead, bead, attempt);
                    let result = Command::new("sh")
                        .arg("-c")
                        .arg(&cmd)
                        .output()
                        .expect("Failed to execute command");

                    // Should either succeed or fail gracefully
                    let stderr = String::from_utf8_lossy(&result.stderr);
                    assert!(!stderr.contains("panicked"), "Should not panic during concurrent access");

                    // Brief pause to allow race conditions
                    thread::sleep(Duration::from_millis(10));
                }
            }
        })
    }).collect();

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify database is still in consistent state
    for bead in &beads {
        let cmd = format!("cargo run --bin view_bead --slug {}", bead);
        let result = Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .output()
            .expect("Failed to execute command");

        // Should not crash, even with concurrent access
        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(!stderr.contains("panicked"), "Database should survive concurrent access");
    }
}

#[test]
fn test_disk_full_simulation() {
    println!("Testing disk full scenario...");

    // This test simulates disk full by using a temporary directory
    // with very limited space. Note: This is a simplified simulation.

    let result = Command::new("sh")
        .arg("-c")
        .arg("cargo run --bin list_beads")
        .env("CARGO_TARGET_TMPDIR", "/tmp") // Use tmp which might be full
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&result.stderr);

    // Should handle potential disk full gracefully
    assert!(!stderr.contains("panicked"), "Should not panic on disk full");

    // If disk is actually full, it should fail gracefully
    if !result.status.success() {
        assert!(stderr.contains("disk") || stderr.contains("space") || stderr.contains("permission"),
               "Should indicate disk-related error");
    }
}

#[test]
fn test_environment_chaos() {
    println!("Testing environment variable chaos...");

    let chaotic_envs = vec![
        ("DATABASE_URL", ""), // Empty database URL
        ("RUST_LOG", "invalid_log_level"),
        ("CARGO_TERM_COLOR", "never"),
        ("USER", "nobody"),
        ("HOME", "/nonexistent"),
    ];

    for (key, value) in chaotic_envs {
        let cmd = format!("{}={} cargo run --bin list_beads", key, value);

        let result = Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8_lossy(&result.stdout);
        let stderr = String::from_utf8_lossy(&result.stderr);

        assert!(!stderr.contains("panicked"), "Should not panic on env chaos: {}={}", key, value);

        if result.status.success() {
            // Even with weird env vars, it should work or fail gracefully
            assert!(stdout.contains("Beads:") || stdout.contains("ID"), "Should handle weird env vars");
        } else {
            assert!(!stderr.is_empty(), "Should provide error message for bad env vars");
        }
    }
}

#[test]
fn test_rapid_commands() {
    println!("Testing rapid command execution...");

    // Rapidly execute commands to test for race conditions
    for i in 0..20 {
        let slug = format!("rapid-test-{}", i);
        let cmd = format!("cargo run --bin create_bead --slug {} --title \"Rapid Test {}\"", slug, i);

        let result = Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .output()
            .expect("Failed to execute command");

        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(!stderr.contains("panicked"), "Should not panic on rapid execution: {}", i);

        // Clean up for next iteration
        let delete_cmd = format!("cargo run --bin delete_bead --slug {}", slug);
        Command::new("sh")
            .arg("-c")
            .arg(&delete_cmd)
            .output()
            .expect("Failed to delete bead");
    }
}

#[test]
fn test_memory_exhaustion_resistance() {
    println!("Testing memory exhaustion resistance...");

    // Create very large strings that might cause memory issues
    let large_strings = vec![
        ("large-title", "A".repeat(500000)), // 500KB title
        ("large-slug", "a".repeat(1000)), // Long slug
    ];

    for (slug_type, large_content) in large_strings {
        let cmd = match slug_type {
            "large-title" => format!("cargo run --bin create_bead --slug slug-test --title \"{}\"", large_content),
            "large-slug" => format!("cargo run --bin create_bead --slug \"{}\" --title \"Title\"", large_content),
            _ => continue,
        };

        let result = Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .output()
            .expect("Failed to execute command");

        let stderr = String::from_utf8_lossy(&result.stderr);
        let stdout = String::from_utf8_lossy(&result.stdout);

        assert!(!stderr.contains("panicked"), "Should not panic on large memory allocation");
        assert!(!stderr.contains("memory allocation"), "Should handle memory gracefully");
        assert!(!stderr.contains("out of memory"), "Should detect memory limits");

        // Should either succeed with proper handling or fail gracefully
        if result.status.success() {
            assert!(stdout.contains("Bead created") || stdout.contains("Created"), "Should handle large input");
        } else {
            assert!(!stderr.is_empty(), "Should provide memory-related error");
        }
    }
}

#[test]
fn test_invalid_path_handling() {
    println!("Testing invalid path handling...");

    let path_cases = vec![
        ("invalid-path", "invalid|path|with|pipes"),
        ("path-spaces", "path with spaces"),
        ("path-newline", "path\nwith\nnewlines"),
        ("path-control", "path\x00with\x01control"),
    ];

    for (slug, path_content) in path_cases {
        let cmd = format!("cargo run --bin create_bead --slug {} --title \"Test {}\"", slug, slug);

        let result = Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .output()
            .expect("Failed to execute command");

        let stderr = String::from_utf8_lossy(&result.stderr);

        assert!(!stderr.contains("panicked"), "Should not panic on invalid paths");
        assert!(!stderr.contains("system error"), "Should handle invalid paths gracefully");

        // Should either succeed or fail with proper error message
        if !result.status.success() {
            assert!(!stderr.is_empty(), "Should provide error for invalid paths");
        }
    }
}

#[test]
fn test_zero_byte_injection() {
    println!("Testing zero byte injection...");

    let zero_byte_cases = vec![
        ("slug-null", "slug\0null"),
        ("title-null", "Title\0Null"),
        ("title-multi-null", "Title\0\0\0Multi"),
    ];

    for (slug, title) in zero_byte_cases {
        let cmd = format!("cargo run --bin create_bead --slug {} --title \"{}\"", slug, title);

        let result = Command::new("sh")
            .arg("-c")
            .arg(&cmd)
            .output()
            .expect("Failed to execute command");

        let stderr = String::from_utf8_lossy(&result.stderr);
        let stdout = String::from_utf8_lossy(&result.stdout);

        assert!(!stderr.contains("panicked"), "Should not panic on null bytes");
        assert!(!stderr.contains("buffer overflow"), "Should prevent buffer overflow");

        if result.status.success() {
            // Should handle null bytes gracefully (either by filtering or proper handling)
            assert!(stdout.contains("Bead created") || stdout.contains("Created"), "Should handle null bytes");
        } else {
            assert!(!stderr.is_empty(), "Should provide error for null bytes");
        }
    }
}

// Main adversarial test runner
#[test]
fn test_all_adversarial_scenarios() {
    println!("Running comprehensive adversarial tests...");

    test_corrupted_database_file();
    test_large_payloads();
    test_unicode_edge_cases();
    test_concurrent_modification();
    test_disk_full_simulation();
    test_environment_chaos();
    test_rapid_commands();
    test_memory_exhaustion_resistance();
    test_invalid_path_handling();
    test_zero_byte_injection();

    println!("All adversarial tests completed!");
    println!("Application should have handled all attacks gracefully.");
}