#!/bin/bash

# Create a temporary directory for testing
TEMP_DIR=$(mktemp -d)
echo "Created temporary directory: $TEMP_DIR"

# Copy the checker test file
cp test/intent/checker_test.gleam "$TEMP_DIR/checker_runner.gleam"

# Create a minimal main function
cat >> "$TEMP_DIR/checker_runner.gleam" << 'EOF'

// Main function to run all tests
pub fn main() {
  // Run all test functions
  check_response_status_match_test()
  check_response_status_mismatch_test()
  check_response_various_status_codes_test()
  check_response_no_checks_test()
  check_response_single_passing_check_test()
  check_response_single_failing_check_test()
  check_response_multiple_checks_all_pass_test()
  check_response_multiple_checks_some_fail_test()
  check_response_header_exact_match_test()
  check_response_header_mismatch_test()
  check_response_multiple_headers_test()
  check_response_missing_header_test()
  check_response_all_aspects_pass_test()
  check_response_all_aspects_fail_test()
  check_response_nested_field_access_test()
  check_response_deeply_nested_field_test()
  check_response_equals_rule_test()
  check_response_contains_rule_test()
  check_response_exists_rule_test()
  check_response_absent_rule_test()
  check_response_missing_field_test()
  check_response_null_value_test()
  check_response_empty_array_test()
  check_response_empty_object_test()

  io.println("All checker tests completed successfully!")
}
EOF

# Copy necessary project files to temp directory
cp -r ../build/packages/intent "$TEMP_DIR/"
cp ../gleam.toml "$TEMP_DIR/"
cp -r ../src "$TEMP_DIR/"

# Navigate to the temp directory
cd "$TEMP_DIR"

# Try to run the test
echo "Attempting to run checker tests..."
if gleam run checker_runner.gleam; then
    echo "✅ All checker tests passed!"
    SUCCESS=true
else
    echo "❌ Checker tests failed"
    SUCCESS=false
fi

# Cleanup
rm -rf "$TEMP_DIR"

if [ "$SUCCESS" = true ]; then
    exit 0
else
    exit 1
fi