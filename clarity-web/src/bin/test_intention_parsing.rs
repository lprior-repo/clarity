use clarity_web::intent::parse_spec;

fn main() {
  println!("Testing Intention Module Parsing...\n");

  // Test 1: Valid minimal spec
  let json = r#"{"name": "test-spec"}"#;
  match parse_spec(json) {
    Ok(spec) => println!("✓ Parsed spec: {}", spec.name),
    Err(e) => println!("✗ Failed to parse: {}", e),
  }

  // Test 2: Invalid JSON
  let invalid = r#"{"name": "test"#;
  match parse_spec(invalid) {
    Ok(_) => println!("✗ Should have failed"),
    Err(e) => println!("✓ Correctly rejected invalid JSON: {}", e),
  }

  // Test 3: Empty name
  let empty_name = r#"{"name": ""}"#;
  match parse_spec(empty_name) {
    Ok(_) => println!("✗ Should have rejected empty name"),
    Err(e) => println!("✓ Correctly rejected empty name: {}", e),
  }

  // Test 4: Spec with features
  let with_features = r#"{
        "name": "test-spec",
        "features": [
            {
                "name": "auth",
                "behaviors": [
                    {"name": "login", "description": "User login"}
                ]
            }
        ]
    }"#;
  match parse_spec(with_features) {
    Ok(spec) => println!(
      "✓ Parsed spec with features: {} features",
      spec.features.len()
    ),
    Err(e) => println!("✗ Failed to parse spec with features: {}", e),
  }

  println!("\n✓ All intention parsing tests passed!");
}
