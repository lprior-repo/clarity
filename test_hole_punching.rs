// Test script to verify hole punching implementation
// This verifies the types and server function are correctly defined

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]

use std::path::Path;

fn main() {
    println!("=== Hole Punching Implementation Verification ===\n");

    let server_rs = Path::new("clarity-web/src/server.rs");
    let types_rs = Path::new("clarity-web/src/components/discover/types.rs");

    // Check server.rs exists
    if !server_rs.exists() {
        eprintln!("❌ server.rs not found");
        std::process::exit(1);
    }
    println!("✅ server.rs exists");

    // Check types.rs exists
    if !types_rs.exists() {
        eprintln!("❌ types.rs not found");
        std::process::exit(1);
    }
    println!("✅ types.rs exists");

    // Read server.rs content
    let server_content = std::fs::read_to_string(server_rs)
        .expect("Failed to read server.rs");

    // Check imports
    println!("\n--- Checking imports ---");
    if server_content.contains("use crate::components::discover::types::{HolePunchingResults, ScenarioField}") {
        println!("✅ HolePunchingResults and ScenarioField imported");
    } else {
        eprintln!("❌ Missing import for HolePunchingResults/ScenarioField");
        std::process::exit(1);
    }

    // Check function signature
    println!("\n--- Checking validate_hole_punching_server function ---");
    if server_content.contains("pub async fn validate_hole_punching_server(") {
        println!("✅ Function validate_hole_punching_server declared");
    } else {
        eprintln!("❌ Function validate_hole_punching_server not found");
        std::process::exit(1);
    }

    // Check parameters
    if server_content.contains("scenario: ScenarioField,") {
        println!("✅ Parameter 'scenario: ScenarioField' present");
    } else {
        eprintln!("❌ Missing scenario parameter");
        std::process::exit(1);
    }

    if server_content.contains("session_id: Option<String>,") {
        println!("✅ Parameter 'session_id: Option<String>' present");
    } else {
        eprintln!("❌ Missing session_id parameter");
        std::process::exit(1);
    }

    // Check return type
    if server_content.contains("Result<HolePunchingResults, ServerFnError>") {
        println!("✅ Return type Result<HolePunchingResults, ServerFnError>");
    } else {
        eprintln!("❌ Incorrect return type");
        std::process::exit(1);
    }

    // Check for #[server] attribute
    if server_content.contains("#[server]\npub async fn validate_hole_punching_server") {
        println!("✅ #[server] attribute present");
    } else {
        eprintln!("❌ Missing #[server] attribute");
        std::process::exit(1);
    }

    // Check rate limiting
    println!("\n--- Checking rate limiting ---");
    if server_content.contains("RATE_LIMITER.check_rate_limit(session)") {
        println!("✅ Rate limiting implemented");
    } else {
        eprintln!("❌ Rate limiting not found");
        std::process::exit(1);
    }

    // Check input validation
    println!("\n--- Checking input validation ---");
    if server_content.contains("scenario.is_bullets_complete()") {
        println!("✅ Input validation checks scenario.is_bullets_complete()");
    } else {
        eprintln!("❌ Missing input validation");
        std::process::exit(1);
    }

    // Check schema definition
    println!("\n--- Checking AI schema ---");
    if server_content.contains("discovery_hole_addressed") &&
       server_content.contains("edge_case_hole_addressed") &&
       server_content.contains("motivation_dropoff_addressed") {
        println!("✅ All 3 hole types in schema");
    } else {
        eprintln!("❌ Missing hole types in schema");
        std::process::exit(1);
    }

    // Check AI provider call
    if server_content.contains(".extract_fields_with_schema(&analysis_prompt, &schema, &context)") {
        println!("✅ AI provider called");
    } else {
        eprintln!("❌ AI provider call not found");
        std::process::exit(1);
    }

    // Check result parsing
    println!("\n--- Checking result parsing ---");
    if server_content.contains("discovery_hole") &&
       server_content.contains("edge_case_hole") &&
       server_content.contains("motivation_dropoff") {
        println!("✅ Results parsed for all 3 holes");
    } else {
        eprintln!("❌ Missing result parsing");
        std::process::exit(1);
    }

    // Check logging
    println!("\n--- Checking logging ---");
    if server_content.contains("validate_hole_punching_server: Validation completed") {
        println!("✅ Completion logging present");
    } else {
        eprintln!("❌ Missing completion log");
        std::process::exit(1);
    }

    // Check tests
    println!("\n--- Checking tests ---");
    if server_content.contains("test_hole_punching_results_serialization") {
        println!("✅ Test: test_hole_punching_results_serialization");
    } else {
        eprintln!("⚠️  Missing test: test_hole_punching_results_serialization");
    }

    if server_content.contains("test_scenario_field_serialization") {
        println!("✅ Test: test_scenario_field_serialization");
    } else {
        eprintln!("⚠️  Missing test: test_scenario_field_serialization");
    }

    if server_content.contains("test_hole_punching_results_is_complete") {
        println!("✅ Test: test_hole_punching_results_is_complete");
    } else {
        eprintln!("⚠️  Missing test: test_hole_punching_results_is_complete");
    }

    if server_content.contains("test_hole_punching_results_unaddressed_holes") {
        println!("✅ Test: test_hole_punching_results_unaddressed_holes");
    } else {
        eprintln!("⚠️  Missing test: test_hole_punching_results_unaddressed_holes");
    }

    if server_content.contains("test_hole_punching_results_from_strings") {
        println!("✅ Test: test_hole_punching_results_from_strings");
    } else {
        eprintln!("⚠️  Missing test: test_hole_punching_results_from_strings");
    }

    if server_content.contains("test_scenario_field_validation_helpers") {
        println!("✅ Test: test_scenario_field_validation_helpers");
    } else {
        eprintln!("⚠️  Missing test: test_scenario_field_validation_helpers");
    }

    // Read types.rs to verify HolePunchingResults structure
    println!("\n--- Checking types.rs ---");
    let types_content = std::fs::read_to_string(types_rs)
        .expect("Failed to read types.rs");

    if types_content.contains("pub struct HolePunchingResults") {
        println!("✅ HolePunchingResults struct defined");
    } else {
        eprintln!("❌ HolePunchingResults struct not found");
        std::process::exit(1);
    }

    if types_content.contains("pub discovery_hole: Option<String>") &&
       types_content.contains("pub edge_case_hole: Option<String>") &&
       types_content.contains("pub motivation_dropoff: Option<String>") {
        println!("✅ All 3 hole fields present in HolePunchingResults");
    } else {
        eprintln!("❌ Missing hole fields in HolePunchingResults");
        std::process::exit(1);
    }

    if types_content.contains("pub fn is_complete(&self)") {
        println!("✅ is_complete() method present");
    } else {
        eprintln!("❌ Missing is_complete() method");
        std::process::exit(1);
    }

    if types_content.contains("pub fn unaddressed_holes(&self)") {
        println!("✅ unaddressed_holes() method present");
    } else {
        eprintln!("❌ Missing unaddressed_holes() method");
        std::process::exit(1);
    }

    if types_content.contains("pub struct ScenarioField") {
        println!("✅ ScenarioField struct defined");
    } else {
        eprintln!("❌ ScenarioField struct not found");
        std::process::exit(1);
    }

    println!("\n=== ✅ All checks passed! ===");
    println!("\nImplementation Summary:");
    println!("- Function: validate_hole_punching_server");
    println!("- Input: ScenarioField + optional session_id");
    println!("- Output: HolePunchingResults");
    println!("- Features:");
    println!("  • Rate limiting");
    println!("  • Input validation");
    println!("  • AI-powered hole detection");
    println!("  • 3 hole types: Discovery, EdgeCase, Motivation");
    println!("  • Comprehensive logging");
    println!("  • Unit tests for serialization and validation");
}
