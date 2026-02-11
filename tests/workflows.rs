//! End-to-End Workflow Tests: Real user scenarios
//!
//! Test complete user workflows from beginning to end.

use std::process::{Command, Output};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

// Helper function to run a command and check for panics
fn run_command_safe(cmd: &str) -> Result<Output, String> {
    let result = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&result.stderr);
    if stderr.contains("panicked") {
        return Err(format!("Command panicked: {}", stderr));
    }

    Ok(result)
}

// Helper function to check if a command succeeded
fn command_succeeded(cmd: &str) -> bool {
    run_command_safe(cmd).map_or(false, |output| output.status.success())
}

// Helper function to get command output
fn command_output(cmd: &str) -> String {
    run_command_safe(cmd)
        .map(|output| String::from_utf8_lossy(&output.stdout).to_string())
        .unwrap_or_default()
}

#[test]
fn test_complete_bead_lifecycle_workflow() {
    println!("Testing complete bead lifecycle workflow...");

    let bead_slug = "lifecycle-test";
    let bead_title = "Lifecycle Test Bead";

    // Step 1: Create a bead
    println!("Step 1: Creating bead...");
    let create_cmd = format!("cargo run --bin create_bead --slug {} --title \"{}\"", bead_slug, bead_title);

    match run_command_safe(&create_cmd) {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.contains("Bead created") || stdout.contains("Created"),
                   "Should create bead successfully");
        }
        Err(e) => panic!("Failed to create bead: {}", e),
    }

    // Step 2: List beads and verify creation
    println!("Step 2: Listing beads...");
    match run_command_safe("cargo run --bin list_beads") {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.contains(bead_slug), "Should show created bead in list");
            assert!(stdout.contains(bead_title), "Should show bead title");
        }
        Err(e) => panic!("Failed to list beads: {}", e),
    }

    // Step 3: View bead details
    println!("Step 3: Viewing bead details...");
    let view_cmd = format!("cargo run --bin view_bead --slug {}", bead_slug);

    match run_command_safe(&view_cmd) {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.contains(bead_title), "Should show bead title");
            assert!(stdout.contains(bead_slug), "Should show bead slug");
        }
        Err(e) => panic!("Failed to view bead: {}", e),
    }

    // Step 4: Update bead
    println!("Step 4: Updating bead...");
    let update_cmd = format!("cargo run --bin update_bead --slug {} --title \"Updated {}\"",
                            bead_slug, bead_title);

    match run_command_safe(&update_cmd) {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.contains("Updated") || stdout.contains("updated"),
                   "Should update bead successfully");
        }
        Err(e) => panic!("Failed to update bead: {}", e),
    }

    // Step 5: Verify update
    println!("Step 5: Verifying update...");
    let view_after_update = format!("cargo run --bin view_bead --slug {}", bead_slug);

    match run_command_safe(&view_after_update) {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.contains("Updated"), "Should show updated title");
        }
        Err(e) => panic!("Failed to verify update: {}", e),
    }

    // Step 6: Delete bead
    println!("Step 6: Deleting bead...");
    let delete_cmd = format!("cargo run --bin delete_bead --slug {}", bead_slug);

    match run_command_safe(&delete_cmd) {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.contains("Deleted") || stdout.contains("deleted"),
                   "Should delete bead successfully");
        }
        Err(e) => panic!("Failed to delete bead: {}", e),
    }

    // Step 7: Verify deletion
    println!("Step 7: Verifying deletion...");
    match run_command_safe(&view_cmd) {
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(stderr.contains("not found") || stderr.contains("Bead not found"),
                   "Should indicate bead not found");
        }
        Err(e) => {
            // Command might fail, which is expected
        }
    }

    // Step 8: Verify bead not in list
    println!("Step 8: Verifying bead not in list...");
    match run_command_safe("cargo run --bin list_beads") {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(!stdout.contains(bead_slug), "Should not show deleted bead");
        }
        Err(e) => panic!("Failed to list beads after deletion: {}", e),
    }

    println!("✅ Complete lifecycle workflow successful!");
}

#[test]
fn test_concurrent_user_workflow() {
    println!("Testing concurrent user workflow simulation...");

    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    let num_users = 3;
    let operations_per_user = 2;
    let success_count = Arc::new(AtomicUsize::new(0));
    let thread_handles = Vec::new();

    // Create user-specific slugs
    let user_slugs: Vec<String> = (0..num_users)
        .map(|i| format!("concurrent-user-{}", i))
        .collect();

    // Start concurrent operations
    let mut handles = Vec::new();
    for (user_idx, slug) in user_slugs.iter().enumerate() {
        let slug = slug.clone();
        let success_count = success_count.clone();
        let user_idx = user_idx;

        let handle = thread::spawn(move || {
            for op_idx in 0..operations_per_user {
                let op_slug = format!("{}-op-{}", slug, op_idx);

                // Create bead
                let create_cmd = format!("cargo run --bin create_bead --slug {} --title \"User {} Op {}\"",
                                       op_slug, user_idx, op_idx);

                if command_succeeded(&create_cmd) {
                    success_count.fetch_add(1, Ordering::Relaxed);

                    // Verify creation
                    let view_cmd = format!("cargo run --bin view_bead --slug {}", op_slug);
                    if command_succeeded(&view_cmd) {
                        success_count.fetch_add(1, Ordering::Relaxed);
                    }

                    // Clean up
                    let delete_cmd = format!("cargo run --bin delete_bead --slug {}", op_slug);
                    let _ = command_succeeded(&delete_cmd);
                }

                // Small delay between operations
                thread::sleep(Duration::from_millis(10));
            }
        });

        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    let total_successes = success_count.load(Ordering::Relaxed);
    let expected_min_successes = num_users * operations_per_user * 2; // 2 successes per operation

    assert!(total_successes >= expected_min_successes / 2,
           "Concurrent operations should be mostly successful");

    println!("✅ Concurrent workflow test completed with {} successes", total_successes);
}

#[test]
fn test_error_recovery_workflow() {
    println!("Testing error recovery workflow...");

    // Step 1: Try to view non-existent bead
    println!("Step 1: Testing non-existent bead access...");
    let view_cmd = "cargo run --bin view_bead --slug nonexistent";

    match run_command_safe(view_cmd) {
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(stderr.contains("not found") || stderr.contains("Bead not found"),
                   "Should handle non-existent bead gracefully");
        }
        Err(e) => panic!("Should not panic on non-existent bead: {}", e),
    }

    // Step 2: Create a bead with invalid input
    println!("Step 2: Testing invalid bead creation...");
    let invalid_create = "cargo run --bin create_bead --slug \"\" --title \"Empty Slug\"";

    match run_command_safe(invalid_create) {
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(!stderr.is_empty(), "Should provide error message for invalid input");
            assert!(!stderr.contains("panicked"), "Should not panic on invalid input");
        }
        Err(e) => panic!("Should not panic on invalid input: {}", e),
    }

    // Step 3: Try to update non-existent bead
    println!("Step 3: Testing update of non-existent bead...");
    let update_cmd = "cargo run --bin update_bead --slug nonexistent --title \"Update\"";

    match run_command_safe(update_cmd) {
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(stderr.contains("not found") || stderr.contains("Bead not found"),
                   "Should handle update of non-existent bead gracefully");
        }
        Err(e) => panic!("Should not panic on update of non-existent bead: {}", e),
    }

    // Step 4: Perform a valid operation after errors
    println!("Step 4: Performing valid operation after errors...");
    let valid_create = "cargo run --bin create_bead --slug recovery-test --title \"Recovery Test\"";

    match run_command_safe(&valid_create) {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.contains("Bead created") || stdout.contains("Created"),
                   "Should succeed after previous errors");
        }
        Err(e) => panic!("Should succeed after previous errors: {}", e),
    }

    // Step 5: Verify the bead was created
    let view_cmd = "cargo run --bin view_bead --slug recovery-test";
    match run_command_safe(&view_cmd) {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.contains("Recovery Test"), "Should show recovery test bead");
        }
        Err(e) => panic!("Should be able to view created bead: {}", e),
    }

    println!("✅ Error recovery workflow successful!");
}

#[test]
fn test_data_integrity_workflow() {
    println!("Testing data integrity workflow...");

    let test_slug = "integrity-test";
    let test_title = "Data Integrity Test";

    // Create multiple beads with same slug (should fail gracefully)
    println!("Step 1: Testing duplicate creation...");
    let create_cmd = format!("cargo run --bin create_bead --slug {} --title \"First\"", test_slug);

    // First creation should succeed
    match run_command_safe(&create_cmd) {
        Ok(_) => {
            // Second creation should fail or succeed with warning
            let second_create = format!("cargo run --bin create_bead --slug {} --title \"Second\"", test_slug);
            match run_command_safe(&second_create) {
                Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    // Should either fail or handle duplicate gracefully
                    assert!(stderr.contains("already exists") || stderr.is_empty() || output.status.success(),
                           "Should handle duplicate bead gracefully");
                }
                Err(e) => {
                    // Error is expected
                }
            }
        }
        Err(e) => panic!("First creation should succeed: {}", e),
    }

    // Verify data consistency
    println!("Step 2: Verifying data consistency...");
    let view_cmd = format!("cargo run --bin view_bead --slug {}", test_slug);

    match run_command_safe(&view_cmd) {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.contains("First"), "Should keep original title");
            assert!(!stdout.contains("Second"), "Should not have second title");
        }
        Err(e) => panic!("Should be able to view bead: {}", e),
    }

    // Update and verify update integrity
    println!("Step 3: Testing update integrity...");
    let update_cmd = format!("cargo run --bin update_bead --slug {} --title \"Updated Title\"", test_slug);

    match run_command_safe(&update_cmd) {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.contains("Updated") || stdout.contains("updated"),
                   "Should update successfully");
        }
        Err(e) => panic!("Should update successfully: {}", e),
    }

    // Verify update was applied
    match run_command_safe(&view_cmd) {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.contains("Updated Title"), "Should show updated title");
            assert!(!stdout.contains("First"), "Should not have original title");
        }
        Err(e) => panic!("Should show updated bead: {}", e),
    }

    // Clean up
    let delete_cmd = format!("cargo run --bin delete_bead --slug {}", test_slug);
    let _ = run_command_safe(&delete_cmd);

    println!("✅ Data integrity workflow successful!");
}

#[test]
fn test_performance_workflow() {
    println!("Testing performance workflow...");

    let num_beads = 10;
    let mut created_slugs = Vec::new();

    // Step 1: Create multiple beads
    println!("Step 1: Creating multiple beads...");
    for i in 0..num_beads {
        let slug = format!("perf-test-{}", i);
        let title = format!("Performance Test {}", i);
        let create_cmd = format!("cargo run --bin create_bead --slug {} --title \"{}\"", slug, title);

        match run_command_safe(&create_cmd) {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                assert!(stdout.contains("Bead created") || stdout.contains("Created"),
                       "Should create bead {} successfully", i);
                created_slugs.push(slug);
            }
            Err(e) => panic!("Failed to create bead {}: {}", i, e),
        }
    }

    // Step 2: List all beads
    println!("Step 2: Listing all beads...");
    match run_command_safe("cargo run --bin list_beads") {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for slug in &created_slugs {
                assert!(stdout.contains(slug), "Should show {} in list", slug);
            }
        }
        Err(e) => panic!("Failed to list beads: {}", e),
    }

    // Step 3: View multiple beads
    println!("Step 3: Viewing multiple beads...");
    for slug in &created_slugs {
        let view_cmd = format!("cargo run --bin view_bead --slug {}", slug);
        match run_command_safe(&view_cmd) {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                assert!(stdout.contains(slug), "Should show bead details for {}", slug);
            }
            Err(e) => panic!("Failed to view bead {}: {}", slug, e),
        }
    }

    // Step 4: Update all beads
    println!("Step 4: Updating all beads...");
    for slug in &created_slugs {
        let update_cmd = format!("cargo run --bin update_bead --slug {} --title \"Updated {}\"",
                               slug, slug);
        let _ = run_command_safe(&update_cmd);
    }

    // Step 5: Delete all beads
    println!("Step 5: Deleting all beads...");
    for slug in &created_slugs {
        let delete_cmd = format!("cargo run --bin delete_bead --slug {}", slug);
        let _ = run_command_safe(&delete_cmd);
    }

    // Step 6: Verify all deleted
    println!("Step 6: Verifying all deleted...");
    match run_command_safe("cargo run --bin list_beads") {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for slug in &created_slugs {
                assert!(!stdout.contains(slug), "Should not show {} in list after deletion", slug);
            }
        }
        Err(e) => panic!("Failed to list beads after deletion: {}", e),
    }

    println!("✅ Performance workflow completed successfully!");
}

#[test]
fn test_user_experience_workflow() {
    println!("Testing user experience workflow...");

    // Step 1: User starts with no beads
    println!("Step 1: Checking initial state...");
    match run_command_safe("cargo run --bin list_beads") {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Should be empty or show helpful message
            assert!(stdout.contains("Beads:") || stdout.contains("No beads") || stdout.contains("ID"),
                   "Should show helpful message when no beads exist");
        }
        Err(e) => panic!("Should list beads even when empty: {}", e),
    }

    // Step 2: User creates first bead
    println!("Step 2: Creating first bead...");
    let create_cmd = "cargo run --bin create_bead --slug first-bead --title \"My First Bead\"";
    match run_command_safe(create_cmd) {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.contains("Bead created") || stdout.contains("Created"),
                   "Should give positive feedback on creation");
        }
        Err(e) => panic!("Should create first bead successfully: {}", e),
    }

    // Step 3: User sees the bead in the list
    println!("Step 3: Viewing bead in list...");
    match run_command_safe("cargo run --bin list_beads") {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.contains("first-bead"), "Should show bead in list");
            assert!(stdout.contains("My First Bead"), "Should show bead title");
        }
        Err(e) => panic!("Should show bead in list: {}", e),
    }

    // Step 4: User updates the bead
    println!("Step 4: Updating bead...");
    let update_cmd = "cargo run --bin update_bead --slug first-bead --title \"My Updated Bead\"";
    match run_command_safe(update_cmd) {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.contains("Updated") || stdout.contains("updated"),
                   "Should confirm update");
        }
        Err(e) => panic!("Should update bead successfully: {}", e),
    }

    // Step 5: User views the updated bead
    println!("Step 5: Viewing updated bead...");
    match run_command_safe("cargo run --bin view_bead --slug first-bead") {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            assert!(stdout.contains("My Updated Bead"), "Should show updated title");
        }
        Err(e) => panic!("Should show updated bead: {}", e),
    }

    // Step 6: User creates multiple beads
    println!("Step 6: Creating multiple beads...");
    let beads = vec![
        ("work-bead", "Work Bead"),
        ("personal-bead", "Personal Bead"),
        ("ideas-bead", "Ideas Bead"),
    ];

    for (slug, title) in beads {
        let create_cmd = format!("cargo run --bin create_bead --slug {} --title \"{}\"", slug, title);
        match run_command_safe(&create_cmd) {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                assert!(stdout.contains("Bead created") || stdout.contains("Created"),
                       "Should create bead {} successfully", slug);
            }
            Err(e) => panic!("Should create bead {}: {}", slug, e),
        }
    }

    // Step 7: User sees all beads
    println!("Step 7: Viewing all beads...");
    match run_command_safe("cargo run --bin list_beads") {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for (slug, _) in &beads {
                assert!(stdout.contains(slug), "Should show {} in list", slug);
            }
        }
        Err(e) => panic!("Should show all beads: {}", e),
    }

    // Step 8: User cleans up
    println!("Step 8: Cleaning up...");
    let delete_cmd = "cargo run --bin delete_bead --slug first-bead";
    let _ = run_command_safe(delete_cmd);

    for (slug, _) in beads {
        let delete_cmd = format!("cargo run --bin delete_bead --slug {}", slug);
        let _ = run_command_safe(delete_cmd);
    }

    println!("✅ User experience workflow completed successfully!");
}

// Main workflow test runner
#[test]
fn test_all_workflows() {
    println!("Running comprehensive workflow tests...");

    test_complete_bead_lifecycle_workflow();
    test_concurrent_user_workflow();
    test_error_recovery_workflow();
    test_data_integrity_workflow();
    test_performance_workflow();
    test_user_experience_workflow();

    println!("✅ All workflow tests completed!");
    println!("🎉 End-to-end user workflows are robust and user-friendly!");
}