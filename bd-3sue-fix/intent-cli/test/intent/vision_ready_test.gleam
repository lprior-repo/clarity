//// Hostile/Adversarial Tests for Vision and Ready Commands
////
//// This test suite tests the security validation functions for vision and ready commands.
////
//// Note: Full CLI integration tests require a test harness.
//// These tests focus on the security validation layer.

import gleam/string
import gleeunit
import gleeunit/should
import intent/security

pub fn main() {
  gleeunit.main()
}

// ============================================================================
// INPUT VALIDATION ATTACKS (20 tests)
// ============================================================================

/// Empty session ID should be rejected
pub fn validate_session_id_empty_test() {
  let result = security.validate_session_id("")

  result
  |> should.be_error()
}

/// Whitespace-only session ID should be rejected
pub fn validate_session_id_whitespace_test() {
  let result = security.validate_session_id("   ")

  result
  |> should.be_error()
}

/// Session ID with spaces should be rejected
pub fn validate_session_id_with_spaces_test() {
  let result = security.validate_session_id("test session")

  result
  |> should.be_error()
}

/// Valid session ID should pass
pub fn validate_session_id_valid_test() {
  let result = security.validate_session_id("interview-123")

  result
  |> should.equal(Ok("interview-123"))
}

/// Valid session ID with underscores should pass
pub fn validate_session_id_with_underscores_test() {
  let result = security.validate_session_id("interview_session_123")

  result
  |> should.equal(Ok("interview_session_123"))
}

/// Session ID with special characters should be rejected
pub fn validate_session_id_special_chars_test() {
  let result = security.validate_session_id("test@#$%")

  result
  |> should.be_error()
}

/// Session ID with path traversal should be rejected
pub fn validate_session_id_path_traversal_test() {
  let result = security.validate_session_id("../../../etc/passwd")

  result
  |> should.be_error()
}

/// Session ID with backslash traversal should be rejected
pub fn validate_session_id_backslash_traversal_test() {
  let result = security.validate_session_id("..\\..\\..\\windows")

  result
  |> should.be_error()
}

/// Session ID with command injection should be rejected
pub fn validate_session_id_command_injection_test() {
  let result = security.validate_session_id("abc; rm -rf /")

  result
  |> should.be_error()
}

/// Session ID with shell metacharacters should be rejected
pub fn validate_session_id_shell_metachars_test() {
  let result = security.validate_session_id("test$USER`whoami`")

  result
  |> should.be_error()
}

/// Session ID with pipe should be rejected
pub fn validate_session_id_pipe_test() {
  let result = security.validate_session_id("test|cat")

  result
  |> should.be_error()
}

/// Session ID with semicolon should be rejected
pub fn validate_session_id_semicolon_test() {
  let result = security.validate_session_id("test;echo")

  result
  |> should.be_error()
}

/// Session ID with backtick should be rejected
pub fn validate_session_id_backtick_test() {
  let result = security.validate_session_id("test`cmd`")

  result
  |> should.be_error()
}

/// Session ID with newlines should be rejected
pub fn validate_session_id_newline_test() {
  let result = security.validate_session_id("test\ncmd")

  result
  |> should.be_error()
}

/// Extremely long session ID should be rejected
pub fn validate_session_id_too_long_test() {
  let long_id = "a" |> string.repeat(1000)
  let result = security.validate_session_id(long_id)

  result
  |> should.be_error()
}

/// Session ID at max length (500) should be rejected
pub fn validate_session_id_max_length_boundary_test() {
  let max_length = "a" |> string.repeat(500)
  let result = security.validate_session_id(max_length)

  // Should fail because 500 chars exceeds max of 499
  result
  |> should.be_error()
}

/// Session ID just under max length should pass
pub fn validate_session_id_under_max_length_test() {
  let under_max = "a" |> string.repeat(499)
  let result = security.validate_session_id(under_max)

  result
  |> should.equal(Ok(under_max))
}

/// Session ID with dots should be rejected (path traversal concern)
pub fn validate_session_id_with_dots_test() {
  let result = security.validate_session_id("test..session")

  result
  |> should.be_error()
}

/// Session ID with single dot should be rejected
pub fn validate_session_id_single_dot_test() {
  let result = security.validate_session_id(".")

  result
  |> should.be_error()
}

/// Session ID with URL encoding should be rejected
pub fn validate_session_id_url_encoded_test() {
  let result = security.validate_session_id("test%2e%2e")

  result
  |> should.be_error()
}

// ============================================================================
// VALID SESSION ID PATTERNS (10 tests)
// ============================================================================

/// Alphanumeric session ID should pass
pub fn validate_session_id_alphanumeric_test() {
  let result = security.validate_session_id("abc123XYZ")

  result
  |> should.equal(Ok("abc123XYZ"))
}

/// Session ID with hyphens should pass
pub fn validate_session_id_with_hyphens_test() {
  let result = security.validate_session_id("interview-session-123")

  result
  |> should.equal(Ok("interview-session-123"))
}

/// Session ID with multiple hyphens should pass
pub fn validate_session_id_multiple_hyphens_test() {
  let result = security.validate_session_id("my-interview-session-id")

  result
  |> should.equal(Ok("my-interview-session-id"))
}

/// Session ID with numbers only should pass
pub fn validate_session_id_numbers_only_test() {
  let result = security.validate_session_id("12345")

  result
  |> should.equal(Ok("12345"))
}

/// Session ID with letters only should pass
pub fn validate_session_id_letters_only_test() {
  let result = security.validate_session_id("interview")

  result
  |> should.equal(Ok("interview"))
}

/// Session ID with mixed case should pass
pub fn validate_session_id_mixed_case_test() {
  let result = security.validate_session_id("InterviewSession123")

  result
  |> should.equal(Ok("InterviewSession123"))
}

/// Session ID with leading underscore should pass
pub fn validate_session_id_leading_underscore_test() {
  let result = security.validate_session_id("_private-session")

  result
  |> should.equal(Ok("_private-session"))
}

/// Session ID with trailing underscore should pass
pub fn validate_session_id_trailing_underscore_test() {
  let result = security.validate_session_id("private-session_")

  result
  |> should.equal(Ok("private-session_"))
}

/// Session ID with multiple underscores should pass
pub fn validate_session_id_multiple_underscores_test() {
  let result = security.validate_session_id("my_interview_session_123")

  result
  |> should.equal(Ok("my_interview_session_123"))
}

/// Session ID with hyphens and underscores should pass
pub fn validate_session_id_hyphens_and_underscores_test() {
  let result = security.validate_session_id("my-interview_session-123")

  result
  |> should.equal(Ok("my-interview_session-123"))
}

// ============================================================================
// EDGE CASES (20 tests)
// ============================================================================

/// Session ID with only hyphens should pass
pub fn validate_session_id_only_hyphens_test() {
  let result = security.validate_session_id("---")

  result
  |> should.equal(Ok("---"))
}

/// Session ID with only underscores should pass
pub fn validate_session_id_only_underscores_test() {
  let result = security.validate_session_id("___")

  result
  |> should.equal(Ok("___"))
}

/// Session ID with single character should pass
pub fn validate_session_id_single_char_test() {
  let result = security.validate_session_id("a")

  result
  |> should.equal(Ok("a"))
}

/// Session ID trimmed of whitespace should work
pub fn validate_session_id_trim_whitespace_test() {
  let result = security.validate_session_id("  interview-123  ")

  result
  |> should.equal(Ok("interview-123"))
}

/// Session ID with tab should be rejected
pub fn validate_session_id_with_tab_test() {
  let result = security.validate_session_id("test\t")

  result
  |> should.be_error()
}

/// Session ID with carriage return should be rejected
pub fn validate_session_id_with_carriage_return_test() {
  let result = security.validate_session_id("test\r")

  result
  |> should.be_error()
}

/// Session ID with form feed should be rejected
pub fn validate_session_id_with_form_feed_test() {
  let result = security.validate_session_id("test\f")

  result
  |> should.be_error()
}

/// Session ID with forward slash should be rejected (path separator)
pub fn validate_session_id_forward_slash_test() {
  let result = security.validate_session_id("test/session")

  result
  |> should.be_error()
}

/// Session ID with backslash should be rejected (Windows path separator)
pub fn validate_session_id_backslash_test() {
  let result = security.validate_session_id("test\\session")

  result
  |> should.be_error()
}

/// Session ID with equals sign should be rejected
pub fn validate_session_id_equals_test() {
  let result = security.validate_session_id("test=123")

  result
  |> should.be_error()
}

/// Session ID with ampersand should be rejected
pub fn validate_session_id_ampersand_test() {
  let result = security.validate_session_id("test&session")

  result
  |> should.be_error()
}

/// Session ID with asterisk should be rejected
pub fn validate_session_id_asterisk_test() {
  let result = security.validate_session_id("test*")

  result
  |> should.be_error()
}

/// Session ID with parentheses should be rejected
pub fn validate_session_id_parentheses_test() {
  let result = security.validate_session_id("test(123)")

  result
  |> should.be_error()
}

/// Session ID with brackets should be rejected
pub fn validate_session_id_brackets_test() {
  let result = security.validate_session_id("test[123]")

  result
  |> should.be_error()
}

/// Session ID with braces should be rejected
pub fn validate_session_id_braces_test() {
  let result = security.validate_session_id("test{123}")

  result
  |> should.be_error()
}

/// Session ID with angle brackets should be rejected
pub fn validate_session_id_angle_brackets_test() {
  let result = security.validate_session_id("test<123>")

  result
  |> should.be_error()
}

/// Session ID with at sign should be rejected
pub fn validate_session_id_at_sign_test() {
  let result = security.validate_session_id("test@123")

  result
  |> should.be_error()
}

/// Session ID with hash should be rejected
pub fn validate_session_id_hash_test() {
  let result = security.validate_session_id("test#123")

  result
  |> should.be_error()
}

/// Session ID with exclamation should be rejected
pub fn validate_session_id_exclamation_test() {
  let result = security.validate_session_id("test!")

  result
  |> should.be_error()
}

/// Session ID with tilde should be rejected
pub fn validate_session_id_tilde_test() {
  let result = security.validate_session_id("test~")

  result
  |> should.be_error()
}

// ============================================================================
// ERROR FORMATTING (5 tests)
// ============================================================================

/// Security error for path traversal should be descriptive
pub fn format_security_error_path_traversal_test() {
  let error = security.PathTraversalAttempt("../../../etc/passwd")
  let formatted = security.format_security_error(error)

  formatted
  |> string.contains("Path traversal attempt")
  |> should.be_true()
}

/// Security error for shell metacharacters should be descriptive
pub fn format_security_error_shell_metachars_test() {
  let error = security.ShellMetacharactersDetected("test; rm -rf /")
  let formatted = security.format_security_error(error)

  formatted
  |> string.contains("shell metacharacters")
  |> should.be_true()
}

/// Security error for invalid path should be descriptive
pub fn format_security_error_invalid_path_test() {
  let error = security.InvalidPath("", "Session ID cannot be empty")
  let formatted = security.format_security_error(error)

  formatted
  |> string.contains("Session ID cannot be empty")
  |> should.be_true()
}

/// Security error for file not accessible should be descriptive
pub fn format_security_error_file_not_accessible_test() {
  let error = security.FileNotAccessible("/nonexistent/file")
  let formatted = security.format_security_error(error)

  formatted
  |> string.contains("not accessible")
  |> should.be_true()
}

/// Security error for unsafe regex should be descriptive
pub fn format_security_error_unsafe_regex_test() {
  let error = security.UnsafeRegexPattern(".+)+", "ReDoS risk")
  let formatted = security.format_security_error(error)

  formatted
  |> string.contains("Unsafe regex")
  |> should.be_true()
}
