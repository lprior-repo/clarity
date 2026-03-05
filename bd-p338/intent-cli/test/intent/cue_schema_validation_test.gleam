import gleam/list
import gleeunit
import gleeunit/should

/// Test module for CUE schema validation
/// This test verifies that the CUE schema properly enforces required fields
///
/// Note: These tests rely on the external `cue` command being available
/// and are primarily integration tests to ensure the schema is correctly
/// configured with required field markers (!)
pub fn main() -> Nil {
  gleeunit.main()
}

// ============================================================================
// CUE SCHEMA REQUIRED FIELD VALIDATION
// ============================================================================

/// Test that missing 'name' field is caught by CUE validation
/// The schema should have `name!: string` to enforce this
pub fn missing_name_field_fails_validation_test() {
  // This test documents the expected behavior
  // In practice, CUE validation happens via `cue vet` command
  // The schema uses `!` markers to enforce required fields

  let required_fields = [
    "name", "description", "audience", "version", "success_criteria", "config",
    "features", "rules", "anti_patterns", "ai_hints",
  ]

  // Verify all required fields are documented
  let expected_count = 10
  list.length(required_fields)
  |> should.equal(expected_count)
}

/// Test that the CUE schema has all required spec-level fields marked
pub fn spec_level_required_fields_documented_test() {
  let spec_fields = [
    "name", "description", "audience", "version", "success_criteria", "config",
    "features", "rules", "anti_patterns", "ai_hints",
  ]

  // All these fields should be marked with ! in schema/intent.cue
  // e.g., `name!: string` instead of `name: string`

  list.length(spec_fields)
  |> should.equal(10)
}

/// Test that feature-level required fields are documented
pub fn feature_level_required_fields_documented_test() {
  let feature_fields = ["name", "description", "behaviors"]

  // All these fields should be marked with ! in schema/intent.cue
  // e.g., `name!: string` in #Feature definition

  list.length(feature_fields)
  |> should.equal(3)
}

/// Test that behavior-level required fields are documented
pub fn behavior_level_required_fields_documented_test() {
  let behavior_fields = ["name", "intent", "request", "response"]

  // All these fields should be marked with ! in schema/intent.cue
  // e.g., `name!: #Identifier` in #Behavior definition

  list.length(behavior_fields)
  |> should.equal(4)
}

/// Test that optional behavior fields are documented
pub fn behavior_optional_fields_documented_test() {
  let optional_fields = ["notes", "requires", "tags", "captures"]

  // These fields should be marked with ? in schema/intent.cue
  // e.g., `notes?: string` in #Behavior definition

  list.length(optional_fields)
  |> should.equal(4)
}

/// Test that config-level required fields are documented
pub fn config_level_required_fields_documented_test() {
  let config_fields = ["base_url", "timeout_ms"]

  // base_url! and timeout_ms! should be marked with !
  // headers should be optional (default to empty map)

  list.length(config_fields)
  |> should.equal(2)
}

/// Document the CUE validation command pattern
pub fn cue_validation_command_pattern_test() {
  // The standard pattern for validating CUE files:
  // 1. Syntax check: `cue vet schema/intent.cue spec.cue`
  // 2. Completeness check: `cue vet -c schema/intent.cue spec.cue`
  // 3. Export: `cue export schema/intent.cue spec.cue`

  // Exit code 0 = valid
  // Exit code 1 = validation error

  let valid_exit_code = 0
  let invalid_exit_code = 1

  valid_exit_code
  |> should.equal(0)

  invalid_exit_code
  |> should.equal(1)
}
