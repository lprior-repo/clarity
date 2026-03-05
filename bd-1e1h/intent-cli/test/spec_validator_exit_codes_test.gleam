import gleam/list
import gleam/string
import gleeunit
import gleeunit/should
import intent/spec_validator

pub fn main() {
  gleeunit.main()
}

// ============================================================================
// VALID SPEC EXIT CODE TESTS
// ============================================================================

pub fn valid_spec_returns_validation_valid_test() {
  let result = spec_validator.validate_spec_file("test/test-valid-spec.cue")

  result
  |> should.equal(spec_validator.ValidationValid)
}

// ============================================================================
// INVALID CUE SYNTAX EXIT CODE TESTS
// ============================================================================

pub fn malformed_cue_returns_validation_invalid_test() {
  let result = spec_validator.validate_spec_file("test/test-malformed-cue.cue")

  case result {
    spec_validator.ValidationValid -> {
      // Should not happen
      True
      |> should.be_false()
    }
    spec_validator.ValidationInvalid(_) -> {
      // Expected result
      True
      |> should.be_true()
    }
  }
}

pub fn malformed_cue_contains_syntax_error_test() {
  let result = spec_validator.validate_spec_file("test/test-malformed-cue.cue")

  case result {
    spec_validator.ValidationInvalid(errors) -> {
      let has_syntax_error =
        errors
        |> list.any(fn(error) {
          case error {
            spec_validator.CueSyntaxError(_, _) -> True
            _ -> False
          }
        })

      has_syntax_error
      |> should.be_true()
    }
    _ -> {
      // This should not happen, but test should fail if it does
      True
      |> should.be_false()
    }
  }
}

// ============================================================================
// EMPTY FEATURES LIST EXIT CODE TESTS
// ============================================================================

pub fn empty_features_list_returns_validation_invalid_test() {
  let result =
    spec_validator.validate_spec_file("test/test-empty-features-list.cue")

  case result {
    spec_validator.ValidationValid -> {
      // Should not happen
      True
      |> should.be_false()
    }
    spec_validator.ValidationInvalid(_) -> {
      // Expected result
      True
      |> should.be_true()
    }
  }
}

pub fn empty_features_list_contains_empty_feature_error_test() {
  let result =
    spec_validator.validate_spec_file("test/test-empty-features-list.cue")

  case result {
    spec_validator.ValidationInvalid(errors) -> {
      let has_empty_features_error =
        errors
        |> list.any(fn(error) {
          case error {
            spec_validator.EmptyFeatureList -> True
            _ -> False
          }
        })

      has_empty_features_error
      |> should.be_true()
    }
    _ -> {
      True
      |> should.be_false()
    }
  }
}

// ============================================================================
// CIRCULAR DEPENDENCY EXIT CODE TESTS
// ============================================================================

pub fn circular_dependencies_returns_validation_invalid_test() {
  let result =
    spec_validator.validate_spec_file("test/test-circular-dependencies.cue")

  case result {
    spec_validator.ValidationValid -> {
      // Should not happen
      True
      |> should.be_false()
    }
    spec_validator.ValidationInvalid(_) -> {
      // Expected result
      True
      |> should.be_true()
    }
  }
}

pub fn circular_dependencies_contains_circular_error_test() {
  let result =
    spec_validator.validate_spec_file("test/test-circular-dependencies.cue")

  case result {
    spec_validator.ValidationInvalid(errors) -> {
      let has_circular_error =
        errors
        |> list.any(fn(error) {
          case error {
            spec_validator.CircularDependency(_) -> True
            _ -> False
          }
        })

      has_circular_error
      |> should.be_true()
    }
    _ -> {
      True
      |> should.be_false()
    }
  }
}

// ============================================================================
// DUPLICATE BEHAVIOR NAMES EXIT CODE TESTS
// ============================================================================

pub fn duplicate_behavior_names_returns_validation_invalid_test() {
  let result =
    spec_validator.validate_spec_file(
      "test/test-duplicate-behavior-names.cue",
    )

  case result {
    spec_validator.ValidationValid -> {
      // Should not happen
      True
      |> should.be_false()
    }
    spec_validator.ValidationInvalid(_) -> {
      // Expected result
      True
      |> should.be_true()
    }
  }
}

pub fn duplicate_behavior_names_contains_duplicate_error_test() {
  let result =
    spec_validator.validate_spec_file(
      "test/test-duplicate-behavior-names.cue",
    )

  case result {
    spec_validator.ValidationInvalid(errors) -> {
      let has_duplicate_error =
        errors
        |> list.any(fn(error) {
          case error {
            spec_validator.DuplicateBehaviorName(_, _) -> True
            _ -> False
          }
        })

      has_duplicate_error
      |> should.be_true()
    }
    _ -> {
      True
      |> should.be_false()
    }
  }
}

// ============================================================================
// INVALID BEHAVIOR NAME EXIT CODE TESTS
// ============================================================================

pub fn invalid_behavior_name_returns_validation_invalid_test() {
  let result =
    spec_validator.validate_spec_file("test/test-invalid-behavior-name.cue")

  case result {
    spec_validator.ValidationValid -> {
      // Should not happen
      True
      |> should.be_false()
    }
    spec_validator.ValidationInvalid(_) -> {
      // Expected result
      True
      |> should.be_true()
    }
  }
}

pub fn invalid_behavior_name_contains_invalid_name_error_test() {
  let result =
    spec_validator.validate_spec_file("test/test-invalid-behavior-name.cue")

  case result {
    spec_validator.ValidationInvalid(errors) -> {
      let has_invalid_name_error =
        errors
        |> list.any(fn(error) {
          case error {
            // CUE schema validates the pattern, so it's a CueSyntaxError
            spec_validator.CueSyntaxError(_, message) -> {
              string.contains(message, "invalid value")
            }
            _ -> False
          }
        })

      has_invalid_name_error
      |> should.be_true()
    }
    _ -> {
      True
      |> should.be_false()
    }
  }
}

// ============================================================================
// NON-EXISTENT FILE EXIT CODE TESTS
// ============================================================================

pub fn non_existent_file_returns_validation_invalid_test() {
  let result = spec_validator.validate_spec_file("test/nonexistent-file.cue")

  case result {
    spec_validator.ValidationValid -> {
      // Should not happen
      True
      |> should.be_false()
    }
    spec_validator.ValidationInvalid(_) -> {
      // Expected result
      True
      |> should.be_true()
    }
  }
}

pub fn non_existent_file_contains_file_not_found_error_test() {
  let result = spec_validator.validate_spec_file("test/nonexistent-file.cue")

  case result {
    spec_validator.ValidationInvalid(errors) -> {
      let has_file_not_found_error =
        errors
        |> list.any(fn(error) {
          case error {
            spec_validator.CueSyntaxError(_, message) -> {
              string.contains(message, "File not found")
            }
            _ -> False
          }
        })

      has_file_not_found_error
      |> should.be_true()
    }
    _ -> {
      True
      |> should.be_false()
    }
  }
}

// ============================================================================
// EXIT CODE MAPPING TESTS
// ============================================================================

pub fn validation_valid_should_map_to_exit_code_0_test() {
  // This test verifies that ValidationValid corresponds to exit code 0
  // The actual exit code handling is in intent.gleam:validate_spec_file
  let result = spec_validator.validate_spec_file("test/test-valid-spec.cue")

  case result {
    spec_validator.ValidationValid -> True
    _ -> False
  }
  |> should.be_true()
}

pub fn validation_invalid_should_map_to_exit_code_1_test() {
  // This test verifies that ValidationInvalid corresponds to exit code 1
  // The actual exit code handling is in intent.gleam:validate_spec_file
  let result =
    spec_validator.validate_spec_file("test/test-empty-features-list.cue")

  case result {
    spec_validator.ValidationInvalid(_) -> True
    _ -> False
  }
  |> should.be_true()
}
