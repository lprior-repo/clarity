/// Security utilities for input validation and sanitization
import gleam/list
import gleam/regexp
import gleam/string
import simplifile

/// Error types for security validation
pub type SecurityError {
  PathTraversalAttempt(path: String)
  InvalidPath(path: String, reason: String)
  FileNotAccessible(path: String)
  UnsafeRegexPattern(pattern: String, reason: String)
  ShellMetacharactersDetected(path: String)
}

/// Check if a path contains only safe characters to prevent command injection
///
/// Only allows alphanumeric characters, forward slashes, underscores, dots, and hyphens.
/// This prevents shell metacharacters like ; | & $ ` from being injected.
///
/// # Example
/// ```gleam
/// is_safe_path("examples/api.cue") // True
/// is_safe_path("; rm -rf /") // False
/// is_safe_path("$(whoami).cue") // False
/// ```
pub fn is_safe_path(path: String) -> Bool {
  case regexp.from_string("^[a-zA-Z0-9/_.-]+$") {
    Ok(pattern) -> regexp.check(pattern, path)
    Error(_) -> False
  }
}

/// Validate a file path to prevent path traversal attacks
///
/// Checks:
/// - Path does not contain ".." (parent directory references)
/// - URL-encoded traversal sequences (%2e%2e, %2f, %5c)
/// - Backslash variants (..\, \..\)
/// - Null byte injection (%00)
/// - Double-encoded sequences (%252e)
/// - Alternative dot representations (....)
/// - File actually exists and is accessible
///
/// # Example
/// ```gleam
/// case validate_file_path("specs/api.cue") {
///   Ok(path) -> load_file(path)
///   Error(PathTraversalAttempt(_)) -> halt_with_error()
/// }
/// ```
pub fn validate_file_path(path: String) -> Result(String, SecurityError) {
  check_shell_metacharacters(path)
  |> and_then(check_literal_traversal)
  |> and_then(check_url_encoded)
  |> and_then(check_backslash_traversal)
  |> and_then(check_null_byte)
  |> and_then(check_double_encoded)
  |> and_then(check_alternative_dots)
  |> and_then(check_file_exists)
}

fn and_then(result: Result(a, e), fun: fn(a) -> Result(b, e)) -> Result(b, e) {
  case result {
    Ok(value) -> fun(value)
    Error(e) -> Error(e)
  }
}

fn check_shell_metacharacters(path: String) -> Result(String, SecurityError) {
  case is_safe_path(path) {
    False -> Error(ShellMetacharactersDetected(path))
    True -> Ok(path)
  }
}

fn check_literal_traversal(path: String) -> Result(String, SecurityError) {
  case string.contains(path, "..") {
    True -> Error(PathTraversalAttempt(path))
    False -> Ok(path)
  }
}

fn check_url_encoded(path: String) -> Result(String, SecurityError) {
  let path_lower = string.lowercase(path)
  case
    string.contains(path_lower, "%2e")
    || string.contains(path_lower, "%2f")
    || string.contains(path_lower, "%5c")
  {
    True -> Error(PathTraversalAttempt(path))
    False -> Ok(path)
  }
}

fn check_backslash_traversal(path: String) -> Result(String, SecurityError) {
  case string.contains(path, "..\\") || string.contains(path, "\\..") {
    True -> Error(PathTraversalAttempt(path))
    False -> Ok(path)
  }
}

fn check_null_byte(path: String) -> Result(String, SecurityError) {
  let path_lower = string.lowercase(path)
  case string.contains(path_lower, "%00") {
    True -> Error(PathTraversalAttempt(path))
    False -> Ok(path)
  }
}

fn check_double_encoded(path: String) -> Result(String, SecurityError) {
  let path_lower = string.lowercase(path)
  case string.contains(path_lower, "%25") {
    True -> Error(PathTraversalAttempt(path))
    False -> Ok(path)
  }
}

fn check_alternative_dots(path: String) -> Result(String, SecurityError) {
  case string.contains(path, "....") {
    True -> Error(PathTraversalAttempt(path))
    False -> Ok(path)
  }
}

fn check_file_exists(path: String) -> Result(String, SecurityError) {
  case simplifile.verify_is_file(path) {
    Ok(True) -> Ok(path)
    Ok(False) -> Error(InvalidPath(path, "Not a regular file"))
    Error(_) -> Error(FileNotAccessible(path))
  }
}

/// Validate a regex pattern to prevent ReDoS (Regular Expression Denial of Service) attacks
///
/// Checks for known dangerous patterns that can cause exponential backtracking:
/// - Nested quantifiers like (.+)+ or ([^)]*)+
/// - Multiple overlapping quantifiers
/// - Catastrophic backtracking patterns
///
/// This is a basic check - it won't catch all ReDoS patterns but prevents the most common ones.
///
/// # Example
/// ```gleam
/// case validate_regex_pattern("^[a-z]+$") {
///   Ok(pattern) -> compile_regex(pattern)
///   Error(UnsafeRegexPattern(_, reason)) -> Error(reason)
/// }
/// ```
pub fn validate_regex_pattern(pattern: String) -> Result(String, SecurityError) {
  // List of dangerous regex patterns that can cause ReDoS
  let dangerous_patterns = [
    // Nested quantifiers
    "(.+)+", "(.*)\\+", "(.*)*", "([^)]*)+", "([^(]+)+", "(\\w+)+", "(\\d+)+",
    "(\\s+)+", "(.+)+$", "^(.+)+",
    // Multiple overlapping quantifiers
    ".*.*", ".+.+",
    // Alternation with overlapping patterns
    "(a+)+", "(a*)*",
  ]

  // Check if pattern contains any dangerous constructs
  let has_danger =
    list.any(dangerous_patterns, fn(dangerous) {
      string.contains(pattern, dangerous)
    })

  case has_danger {
    True ->
      Error(UnsafeRegexPattern(
        pattern,
        "Pattern contains potentially unsafe construct that could cause ReDoS (exponential backtracking)",
      ))
    False -> Ok(pattern)
  }
}

/// Format security error for display
pub fn format_security_error(error: SecurityError) -> String {
  case error {
    PathTraversalAttempt(path) ->
      "Security error: Path traversal attempt detected in '"
      <> path
      <> "'. Paths cannot contain '..' references or encoded variants (%2e, %2f, %5c, %25, %00, etc.)."
    InvalidPath(path, reason) ->
      "Security error: Invalid path '" <> path <> "': " <> reason
    FileNotAccessible(path) ->
      "Security error: File '"
      <> path
      <> "' is not accessible or does not exist."
    UnsafeRegexPattern(pattern, reason) ->
      "Security error: Unsafe regex pattern '"
      <> pattern
      <> "': "
      <> reason
      <> "\nFor security, patterns with nested quantifiers are not allowed."
    ShellMetacharactersDetected(path) ->
      "Security error: Invalid file path '"
      <> path
      <> "'. Path contains shell metacharacters. Only alphanumeric characters, forward slashes, underscores, dots, and hyphens are allowed."
  }
}

/// Validate multiple file paths
pub fn validate_file_paths(
  paths: List(String),
) -> Result(List(String), SecurityError) {
  paths
  |> list_try_map(validate_file_path)
}

/// Validate a session ID for security
///
/// Checks:
/// - Not empty or whitespace only
/// - Maximum length (500 characters)
/// - Only safe characters (alphanumeric, hyphens, underscores)
/// - No path traversal sequences
/// - No shell metacharacters
/// - No null bytes or control characters
///
/// # Example
/// ```gleam
/// case validate_session_id("interview-123") {
///   Ok(id) -> use_session(id)
///   Error(_) -> halt_with_error()
/// }
/// ```
pub fn validate_session_id(session_id: String) -> Result(String, SecurityError) {
  // First check for control characters (before trimming)
  case
    string.contains(session_id, "\t")
    || string.contains(session_id, "\n")
    || string.contains(session_id, "\r")
    || string.contains(session_id, "\u{000C}")
  {
    // Form feed
    True ->
      Error(InvalidPath(session_id, "Session ID contains control characters"))
    False -> {
      let trimmed = string.trim(session_id)

      // Check for empty
      case trimmed == "" {
        True -> Error(InvalidPath(session_id, "Session ID cannot be empty"))
        False -> {
          // Check length (prevent buffer overflow attempts) - use >= for 500 max
          case string.length(trimmed) >= 500 {
            True ->
              Error(InvalidPath(
                session_id,
                "Session ID too long (max 499 characters)",
              ))
            False -> {
              // Check for safe characters (alphanumeric, hyphen, underscore only)
              case regexp.from_string("^[a-zA-Z0-9_-]+$") {
                Ok(pattern) -> {
                  case regexp.check(pattern, trimmed) {
                    False -> Error(ShellMetacharactersDetected(session_id))
                    True -> {
                      // Check for path traversal
                      case string.contains(trimmed, "..") {
                        True -> Error(PathTraversalAttempt(session_id))
                        False -> Ok(trimmed)
                      }
                    }
                  }
                }
                Error(_) ->
                  Error(InvalidPath(session_id, "Invalid session ID format"))
              }
            }
          }
        }
      }
    }
  }
}

// Helper to map over list with Result
fn list_try_map(list: List(a), fun: fn(a) -> Result(b, e)) -> Result(List(b), e) {
  case list {
    [] -> Ok([])
    [head, ..tail] -> {
      case fun(head) {
        Ok(value) -> {
          case list_try_map(tail, fun) {
            Ok(rest) -> Ok([value, ..rest])
            Error(e) -> Error(e)
          }
        }
        Error(e) -> Error(e)
      }
    }
  }
}
