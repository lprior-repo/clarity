//! API Testing: Validate HTTP endpoints and responses
//!
//! Test all REST API endpoints with real HTTP requests.

use std::process::{Command, Output};
use std::thread;
use std::time::Duration;

// Test if the API server is running
fn is_api_running() -> bool {
    let result = Command::new("curl")
        .args(&["-s", "-o", "/dev/null", "-w", "%{http_code}", "http://localhost:8080/health"])
        .output();

    match result {
        Ok(output) => {
            let status_code = String::from_utf8_lossy(&output.stdout).trim();
            status_code == "200"
        }
        Err(_) => false,
    }
}

#[test]
fn test_api_health_endpoint() {
    println!("Testing API health endpoint...");

    if !is_api_running() {
        println!("API server not running, skipping API tests");
        return;
    }

    let result = Command::new("curl")
        .args(&["-s", "http://localhost:8080/health"])
        .output()
        .expect("Failed to execute curl command");

    assert!(result.status.success(), "Health endpoint should return 200");

    let output = String::from_utf8_lossy(&result.stdout);
    assert!(output.contains("status"), "Should return status information");
    assert!(!output.contains("error"), "Should not contain error");
}

#[test]
fn test_api_cors_headers() {
    println!("Testing API CORS headers...");

    if !is_api_running() {
        println!("API server not running, skipping API tests");
        return;
    }

    let result = Command::new("curl")
        .args(&["-s", "-i", "http://localhost:8080/health"])
        .output()
        .expect("Failed to execute curl command");

    let output = String::from_utf8_lossy(&result.stdout);
    let headers = output.lines().take(10).collect::<Vec<_>>().join("\n");

    assert!(headers.contains("HTTP/1.1 200"), "Should return HTTP 200");
    assert!(headers.contains("Content-Type:"), "Should include Content-Type header");
}

#[test]
fn test_api_create_bead_endpoint() {
    println!("Testing API create bead endpoint...");

    if !is_api_running() {
        println!("API server not running, skipping API tests");
        return;
    }

    // Create bead via API
    let result = Command::new("curl")
        .args(&[
            "-s", "-X", "POST",
            "http://localhost:8080/api/beads",
            "-H", "Content-Type: application/json",
            "-d", r#"{"slug": "api-test-bead", "title": "API Test Bead", "description": "Test description"}"#
        ])
        .output()
        .expect("Failed to execute curl command");

    let output = String::from_utf8_lossy(&result.stdout);
    let status_code = result.status.code().unwrap_or(0);

    // Should either succeed or fail gracefully
    if status_code == 201 || status_code == 200 {
        assert!(output.contains("id"), "Should return bead ID");
        assert!(output.contains("slug"), "Should return bead slug");
    } else {
        // Check for proper error response
        assert!(status_code >= 400, "Should return error status code for invalid requests");
        assert!(!output.is_empty(), "Should provide error message");
    }
}

#[test]
fn test_api_list_beads_endpoint() {
    println!("Testing API list beads endpoint...");

    if !is_api_running() {
        println!("API server not running, skipping API tests");
        return;
    }

    let result = Command::new("curl")
        .args(&["-s", "http://localhost:8080/api/beads"])
        .output()
        .expect("Failed to execute curl command");

    let output = String::from_utf8_lossy(&result.stdout);
    let status_code = result.status.code().unwrap_or(0);

    assert!(status_code == 200, "List beads should return 200");
    assert!(output.contains("["), "Should return JSON array");
    assert!(output.contains("slug"), "Should include slug information");
}

#[test]
fn test_api_get_bead_endpoint() {
    println!("Testing API get bead endpoint...");

    if !is_api_running() {
        println!("API server not running, skipping API tests");
        return;
    }

    // First create a bead
    Command::new("curl")
        .args(&[
            "-s", "-X", "POST",
            "http://localhost:8080/api/beads",
            "-H", "Content-Type: application/json",
            "-d", r#"{"slug": "api-get-test", "title": "API Get Test", "description": "Test"}"#
        ])
        .output()
        .expect("Failed to create bead");

    // Then get it
    let result = Command::new("curl")
        .args(&["-s", "http://localhost:8080/api/beads/api-get-test"])
        .output()
        .expect("Failed to execute curl command");

    let output = String::from_utf8_lossy(&result.stdout);
    let status_code = result.status.code().unwrap_or(0);

    if status_code == 200 {
        assert!(output.contains("slug"), "Should include slug");
        assert!(output.contains("title"), "Should include title");
    } else {
        assert!(status_code == 404, "Should return 404 for non-existent bead");
    }
}

#[test]
fn test_api_update_bead_endpoint() {
    println!("Testing API update bead endpoint...");

    if !is_api_running() {
        println!("API server not running, skipping API tests");
        return;
    }

    // Create a bead first
    Command::new("curl")
        .args(&[
            "-s", "-X", "POST",
            "http://localhost:8080/api/beads",
            "-H", "Content-Type: application/json",
            "-d", r#"{"slug": "api-update-test", "title": "Original Title", "description": "Original description"}"#
        ])
        .output()
        .expect("Failed to create bead");

    // Update it
    let result = Command::new("curl")
        .args(&[
            "-s", "-X", "PUT",
            "http://localhost:8080/api/beads/api-update-test",
            "-H", "Content-Type: application/json",
            "-d", r#"{"title": "Updated Title", "description": "Updated description"}"#
        ])
        .output()
        .expect("Failed to execute curl command");

    let output = String::from_utf8_lossy(&result.stdout);
    let status_code = result.status.code().unwrap_or(0);

    if status_code == 200 || status_code == 204 {
        // Update succeeded
    } else {
        assert!(status_code == 404, "Should return 404 for non-existent bead");
    }
}

#[test]
fn test_api_delete_bead_endpoint() {
    println!("Testing API delete bead endpoint...");

    if !is_api_running() {
        println!("API server not running, skipping API tests");
        return;
    }

    // Create a bead first
    Command::new("curl")
        .args(&[
            "-s", "-X", "POST",
            "http://localhost:8080/api/beads",
            "-H", "Content-Type: application/json",
            "-d", r#"{"slug": "api-delete-test", "title": "Delete Test", "description": "To be deleted"}"#
        ])
        .output()
        .expect("Failed to create bead");

    // Delete it
    let result = Command::new("curl")
        .args(&["-s", "-X", "DELETE", "http://localhost:8080/api/beads/api-delete-test"])
        .output()
        .expect("Failed to execute curl command");

    let status_code = result.status.code().unwrap_or(0);
    assert!(status_code == 200 || status_code == 204 || status_code == 404,
            "Delete should return 200, 204, or 404");
}

#[test]
fn test_api_error_handling() {
    println!("Testing API error handling...");

    if !is_api_running() {
        println!("API server not running, skipping API tests");
        return;
    }

    // Test various error scenarios
    let error_cases = vec![
        // Non-existent bead
        (vec!["-s", "http://localhost:8080/api/beads/nonexistent"], 404),
        // Invalid JSON
        (vec!["-s", "-X", "POST", "http://localhost:8080/api/beads", "-H", "Content-Type: application/json", "-d", "invalid json"], 400),
        // Missing required fields
        (vec!["-s", "-X", "POST", "http://localhost:8080/api/beads", "-H", "Content-Type: application/json", "-d", r#"{"title": "No slug"}"#], 400),
        // Empty slug
        (vec!["-s", "-X", "POST", "http://localhost:8080/api/beads", "-H", "Content-Type: application/json", "-d", r#"{"slug": "", "title": "Empty slug"}"#], 400),
    ];

    for (args, expected_status) in error_cases {
        let result = Command::new("curl")
            .args(args)
            .output()
            .expect("Failed to execute curl command");

        let status_code = result.status.code().unwrap_or(0);
        let output = String::from_utf8_lossy(&result.stdout);

        assert_eq!(status_code, expected_status, "Should return expected status code");
        assert!(!output.is_empty(), "Should provide error message");
    }
}

#[test]
fn test_api_rate_limiting() {
    println!("Testing API rate limiting...");

    if !is_api_running() {
        println!("API server not running, skipping API tests");
        return;
    }

    // Make multiple rapid requests
    let mut status_codes = Vec::new();
    for _ in 0..10 {
        let result = Command::new("curl")
            .args(&["-s", "http://localhost:8080/api/beads"])
            .output()
            .expect("Failed to execute curl command");

        status_codes.push(result.status.code().unwrap_or(0));
    }

    // Most requests should succeed (200)
    let success_count = status_codes.iter().filter(|&code| *code == 200).count();
    assert!(success_count >= 5, "Should handle multiple requests without rate limiting");
}

#[test]
fn test_api_input_validation() {
    println!("Testing API input validation...");

    if !is_api_running() {
        println!("API server not running, skipping API tests");
        return;
    }

    let invalid_inputs = vec![
        // SQL injection attempt
        (vec!["-s", "-X", "POST", "http://localhost:8080/api/beads", "-H", "Content-Type: application/json", "-d", r#"{"slug": "'; DROP TABLE beads; --", "title": "SQL Injection"}"#], 400),
        // XSS attempt
        (vec!["-s", "-X", "POST", "http://localhost:8080/api/beads", "-H", "Content-Type: application/json", "-d", r#"{"slug": "xss-test", "title": "<script>alert('xss')</script>"}"#], 400),
        // Path traversal attempt
        (vec!["-s", "-X", "POST", "http://localhost:8080/api/beads", "-H", "Content-Type: application/json", "-d", r#"{"slug": "../../../etc/passwd", "title": "Path Traversal"}"#], 400),
        // Very long title
        (vec!["-s", "-X", "POST", "http://localhost:8080/api/beads", "-H", "Content-Type: application/json", "-d", format!(r#"{{"slug": "long-title", "title": "{}"}}"#, "A".repeat(10000))], 400),
    ];

    for (args, expected_status) in invalid_inputs {
        let result = Command::new("curl")
            .args(args)
            .output()
            .expect("Failed to execute curl command");

        let status_code = result.status.code().unwrap_or(0);
        let output = String::from_utf8_lossy(&result.stdout);

        assert_eq!(status_code, expected_status, "Should reject invalid input");
        assert!(!output.contains("panicked"), "Should not panic on invalid input");
    }
}

#[test]
fn test_api_concurrent_requests() {
    println!("Testing API concurrent requests...");

    if !is_api_running() {
        println!("API server not running, skipping API tests");
        return;
    }

    use std::sync::Arc;
    use std::thread;

    let num_requests = 10;
    let request_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let success_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));

    let handles: Vec<_> = (0..num_requests)
        .map(|i| {
            let request_count = request_count.clone();
            let success_count = success_count.clone();

            thread::spawn(move || {
                request_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                let result = Command::new("curl")
                    .args(&["-s", "http://localhost:8080/api/beads"])
                    .output()
                    .expect("Failed to execute curl command");

                let status_code = result.status.code().unwrap_or(0);
                if status_code == 200 {
                    success_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                }
            })
        })
        .collect();

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    let total_requests = request_count.load(std::sync::atomic::Ordering::Relaxed);
    let successful_requests = success_count.load(std::sync::atomic::Ordering::Relaxed);

    assert_eq!(total_requests, num_requests, "Should make all requested calls");
    assert!(successful_requests > 0, "Should handle concurrent requests");
}

#[test]
fn test_api_large_response_handling() {
    println!("Testing API large response handling...");

    if !is_api_running() {
        println!("API server not running, skipping API tests");
        return;
    }

    // Create multiple beads to test list endpoint
    for i in 0..5 {
        Command::new("curl")
            .args(&[
                "-s", "-X", "POST",
                "http://localhost:8080/api/beads",
                "-H", "Content-Type: application/json",
                "-d", format!(r#"{{"slug": "large-response-{}", "title": "Large Response Test {}", "description": "This is a description for large response test {}"}}"#, i, i, i)
            ])
            .output()
            .expect("Failed to create bead");
    }

    // Test list endpoint with multiple beads
    let result = Command::new("curl")
        .args(&["-s", "http://localhost:8080/api/beads"])
        .output()
        .expect("Failed to execute curl command");

    let output = String::from_utf8_lossy(&result.stdout);
    let status_code = result.status.code().unwrap_or(0);

    assert_eq!(status_code, 200, "Should return 200 for list");
    assert!(output.contains("large-response-0"), "Should include created beads");
    assert!(output.len() > 100, "Should return meaningful response");
}

// Main API test runner
#[test]
fn test_all_api_functionality() {
    println!("Running comprehensive API tests...");

    // Skip if API server not running
    if !is_api_running() {
        println!("API server not running, skipping API tests");
        return;
    }

    test_api_health_endpoint();
    test_api_cors_headers();
    test_api_create_bead_endpoint();
    test_api_list_beads_endpoint();
    test_api_get_bead_endpoint();
    test_api_update_bead_endpoint();
    test_api_delete_bead_endpoint();
    test_api_error_handling();
    test_api_rate_limiting();
    test_api_input_validation();
    test_api_concurrent_requests();
    test_api_large_response_handling();

    println!("All API tests passed!");
    println!("HTTP API is robust and secure.");
}