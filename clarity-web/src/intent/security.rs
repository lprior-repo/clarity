//! Security Validation Module (WP05)
//!
//! Provides comprehensive security validation for file paths, regex patterns,
//! and session identifiers to prevent injection attacks and malicious input.
//!
//! ## Design Principles
//!
//! - **Zero panics**: All fallible operations return `Result<T, E>`
//! - **Defense in depth**: Multiple layers of validation
//! - **Explicit errors**: Detailed error types for debugging and logging
//!
//! ## Validation Categories
//!
//! - **Path traversal**: Literal `..`, URL-encoded, double-encoded, backslash, null bytes
//! - **Shell metacharacters**: Command injection prevention
//! - **`ReDoS` patterns**: Catastrophic backtracking detection
//! - **Session IDs**: Length and character restrictions

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]
#![allow(clippy::missing_errors_doc)]

use itertools::Itertools;
use thiserror::Error;

// =============================================================================
// Result Type
// =============================================================================

/// Security validation result type
pub type SecurityResult<T> = Result<T, SecurityError>;

// =============================================================================
// Error Types
// =============================================================================

/// Security validation error
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SecurityError {
    /// Path traversal attempt detected
    #[error("path traversal detected: {details}")]
    PathTraversal { details: String },

    /// URL-encoded path traversal detected
    #[error("URL-encoded path traversal detected: {encoding_type:?}")]
    EncodedPathTraversal { encoding_type: PathEncodingType },

    /// Shell metacharacter detected in input
    #[error("shell metacharacter detected: category={category:?}, char='{ch}'")]
    ShellMetacharacter { category: MetacharCategory, ch: char },

    /// `ReDoS` vulnerability detected in regex pattern
    #[error("ReDoS vulnerability detected: {vulnerability:?}")]
    ReDoSVulnerability { vulnerability: RegexVulnerability },

    /// Session ID validation failed
    #[error("session ID validation failed: {error:?}")]
    SessionIdValidation { error: SessionIdError },

    /// Null byte detected in input
    #[error("null byte detected in input")]
    NullByteDetected,

    /// Backslash in path (Windows-style traversal)
    #[error("backslash detected in path (potential Windows traversal)")]
    BackslashInPath,

    /// Empty input provided
    #[error("empty input provided")]
    EmptyInput,
}

/// Path encoding type for detailed error reporting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum PathEncodingType {
    /// Single URL encoding (e.g., %2e%2e)
    #[error("single URL encoding")]
    SingleEncoded,

    /// Double URL encoding (e.g., %252e%252e)
    #[error("double URL encoding")]
    DoubleEncoded,

    /// Mixed encoding (combination of techniques)
    #[error("mixed encoding")]
    MixedEncoding,
}

/// Shell metacharacter category for classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum MetacharCategory {
    /// Command separators: `;`, `|`, `&`
    #[error("command separator")]
    CommandSeparator,

    /// Variable expansion: `$`, `` ` ``
    #[error("variable expansion")]
    VariableExpansion,

    /// Grouping: `(`, `)`, `{`, `}`, `[`, `]`
    #[error("grouping")]
    Grouping,

    /// Redirection: `<`, `>`
    #[error("redirection")]
    Redirection,

    /// Escape/quote: `\`, `!`, `*`, `?`, `"`, `'`
    #[error("escape or quote")]
    EscapeQuote,

    /// Control character (ASCII < 32)
    #[error("control character")]
    ControlCharacter,
}

/// Regex vulnerability type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum RegexVulnerability {
    /// Nested quantifiers: (.+)+, (a*)*
    #[error("nested quantifiers")]
    NestedQuantifiers,

    /// Overlapping wildcards: .*.*
    #[error("overlapping wildcards")]
    OverlappingWildcards,

    /// Alternation with common prefix
    #[error("alternation overlap")]
    AlternationOverlap,

    /// Exponential backtracking potential
    #[error("exponential backtracking")]
    ExponentialBacktracking,
}

/// Session ID validation error
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum SessionIdError {
    /// Session ID exceeds maximum length
    #[error("session ID exceeds maximum length of {max} characters")]
    TooLong { max: usize },

    /// Session ID contains invalid characters
    #[error("session ID contains invalid character: '{ch}'")]
    InvalidCharacter { ch: char },

    /// Session ID is empty
    #[error("session ID is empty")]
    Empty,
}

// =============================================================================
// Constants
// =============================================================================

/// Maximum allowed session ID length
const MAX_SESSION_ID_LENGTH: usize = 499;

/// Shell metacharacters that need to be blocked
const SHELL_METACHARACTERS: &[char] = &[
    ';', '|', '&', '$', '`', '(', ')', '{', '}', '[', ']', '<', '>', '\\', '!', '*', '?', '"', '\'',
];

/// Control character range end (exclusive)
const CONTROL_CHAR_MAX: u8 = 32;

// =============================================================================
// Helper Functions
// =============================================================================

/// Check if a character is a shell metacharacter
fn is_shell_metachar(ch: char) -> bool {
    SHELL_METACHARACTERS.contains(&ch)
}

/// Get the category of a shell metacharacter
const fn classify_metachar(ch: char) -> Option<MetacharCategory> {
    match ch {
        ';' | '|' | '&' => Some(MetacharCategory::CommandSeparator),
        '$' | '`' => Some(MetacharCategory::VariableExpansion),
        '(' | ')' | '{' | '}' | '[' | ']' => Some(MetacharCategory::Grouping),
        '<' | '>' => Some(MetacharCategory::Redirection),
        '\\' | '!' | '*' | '?' | '"' | '\'' => Some(MetacharCategory::EscapeQuote),
        _ => None,
    }
}

/// Check if a character is a control character (ASCII < 32)
fn is_control_character(ch: char) -> bool {
    let code = ch as u32;
    code < u32::from(CONTROL_CHAR_MAX)
}

/// Case-insensitive check for URL-encoded path traversal patterns
fn contains_encoded_traversal(input: &str) -> Option<PathEncodingType> {
    let lower = input.to_lowercase();

    // Check for double-encoded patterns first (more specific)
    // %252e = double-encoded '.' (%2e -> %252e)
    // %255c = double-encoded '\' (%5c -> %255c)
    let double_encoded_patterns = [
        "%252e", "%255c", "%250a", "%250d", "%252f",
    ];

    for pattern in double_encoded_patterns {
        if lower.contains(pattern) {
            return Some(PathEncodingType::DoubleEncoded);
        }
    }

    // Check for single-encoded patterns
    // %2e = '.', %2f = '/', %5c = '\', %00 = null, %0a = newline, %0d = carriage return
    let single_encoded_patterns = [
        "%2e", "%2f", "%5c", "%00", "%0a", "%0d",
    ];

    for pattern in single_encoded_patterns {
        if lower.contains(pattern) {
            return Some(PathEncodingType::SingleEncoded);
        }
    }

    None
}

/// Check for dangerous regex patterns (`ReDoS` vulnerabilities)
///
/// Detects patterns like `(.+)+`, `(a*)*`, `.*.*` that can cause
/// catastrophic backtracking in regex engines.
fn detect_redos_patterns(pattern: &str) -> Option<RegexVulnerability> {
    // Check for patterns that can cause exponential backtracking first
    // These are the most dangerous and specific patterns
    let exp_patterns = ["(.*)*", "(.+)+", "(.?)?", "(.+)*", "(.*)+"];

    for exp in exp_patterns {
        if pattern.contains(exp) {
            return Some(RegexVulnerability::ExponentialBacktracking);
        }
    }

    // Check for overlapping wildcards: .*.*
    if pattern.contains(".*.*") || pattern.contains(".+.+") || pattern.contains(".?.?") {
        return Some(RegexVulnerability::OverlappingWildcards);
    }

    // Check for nested quantifiers like (.+)+, (a*)*, (a+)+
    // These are more general patterns that might indicate nested quantifiers
    let nested_patterns = [
        ")+)+", "*)*", "]+]+", ")+*", "*)+", "?)+", ")+?",
    ];

    for nested in nested_patterns {
        if pattern.contains(nested) {
            return Some(RegexVulnerability::NestedQuantifiers);
        }
    }

    None
}

/// Check if session ID character is valid (alphanumeric, hyphen, underscore)
const fn is_valid_session_id_char(ch: char) -> bool {
    matches!(ch, 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_')
}

// =============================================================================
// Public API
// =============================================================================

/// Check if a path is safe (no traversal or injection attacks)
///
/// This is a convenience function that returns a boolean for quick checks.
/// For detailed error information, use [`validate_file_path`].
///
/// # Arguments
///
/// * `path` - The path to validate
///
/// # Returns
///
/// `true` if the path is safe, `false` otherwise
#[must_use]
pub fn is_safe_path(path: &str) -> bool {
    validate_file_path(path).is_ok()
}

/// Validate a file path for security issues
///
/// Performs comprehensive validation including:
/// - Path traversal detection (literal "..")
/// - URL-encoded traversal (single and double encoded)
/// - Backslash detection (Windows-style traversal)
/// - Null byte injection
/// - Shell metacharacters
/// - Control characters
///
/// # Arguments
///
/// * `path` - The path to validate
///
/// # Returns
///
/// `Ok(sanitized_path)` if valid, `Err(SecurityError)` with details otherwise
///
/// # Example
///
/// ```
/// use clarity_web::intent::security::{validate_file_path, SecurityError};
///
/// // Valid path
/// assert!(validate_file_path("safe/path/file.txt").is_ok());
///
/// // Path traversal
/// let result = validate_file_path("../../../etc/passwd");
/// assert!(matches!(result, Err(SecurityError::PathTraversal { .. })));
/// ```
pub fn validate_file_path(path: &str) -> SecurityResult<String> {
    // Check for empty input
    if path.is_empty() {
        return Err(SecurityError::EmptyInput);
    }

    // Check for null bytes first (most dangerous)
    if path.contains('\0') {
        return Err(SecurityError::NullByteDetected);
    }

    // Check for backslash (Windows-style path traversal)
    if path.contains('\\') {
        return Err(SecurityError::BackslashInPath);
    }

    // Check for literal path traversal
    if path.contains("..") {
        return Err(SecurityError::PathTraversal {
            details: "literal '..' sequence detected".to_string(),
        });
    }

    // Check for URL-encoded traversal patterns
    if let Some(encoding_type) = contains_encoded_traversal(path) {
        return Err(SecurityError::EncodedPathTraversal { encoding_type });
    }

    // Check for shell metacharacters and control characters
    for ch in path.chars() {
        if is_control_character(ch) {
            return Err(SecurityError::ShellMetacharacter {
                category: MetacharCategory::ControlCharacter,
                ch,
            });
        }

        if is_shell_metachar(ch) {
            let category = classify_metachar(ch).unwrap_or(MetacharCategory::EscapeQuote);
            return Err(SecurityError::ShellMetacharacter { category, ch });
        }
    }

    // Return the validated (sanitized) path
    Ok(path.to_string())
}

/// Validate a regex pattern for `ReDoS` vulnerabilities
///
/// Checks for patterns that could cause catastrophic backtracking:
/// - Nested quantifiers: `(.+)+`, `(a*)*`
/// - Overlapping wildcards: `.*.*`
/// - Exponential backtracking patterns
///
/// # Arguments
///
/// * `pattern` - The regex pattern to validate
///
/// # Returns
///
/// `Ok(pattern)` if safe, `Err(SecurityError)` with vulnerability details otherwise
///
/// # Example
///
/// ```
/// use clarity_web::intent::security::{validate_regex_pattern, SecurityError, RegexVulnerability};
///
/// // Safe pattern
/// assert!(validate_regex_pattern("^\\w+$").is_ok());
///
/// // Dangerous nested quantifier
/// let result = validate_regex_pattern("(.+)+");
/// assert!(matches!(
///     result,
///     Err(SecurityError::ReDoSVulnerability { vulnerability: RegexVulnerability::NestedQuantifiers })
/// ));
/// ```
pub fn validate_regex_pattern(pattern: &str) -> SecurityResult<String> {
    // Check for empty input
    if pattern.is_empty() {
        return Err(SecurityError::EmptyInput);
    }

    // Check for null bytes
    if pattern.contains('\0') {
        return Err(SecurityError::NullByteDetected);
    }

    // Check for ReDoS patterns
    if let Some(vulnerability) = detect_redos_patterns(pattern) {
        return Err(SecurityError::ReDoSVulnerability { vulnerability });
    }

    Ok(pattern.to_string())
}

/// Validate a session ID for security and format compliance
///
/// Ensures session IDs conform to security requirements:
/// - Maximum length: 499 characters
/// - Allowed characters: alphanumeric, hyphens, underscores
/// - Non-empty
///
/// # Arguments
///
/// * `session_id` - The session ID to validate
///
/// # Returns
///
/// `Ok(session_id)` if valid, `Err(SecurityError)` with details otherwise
///
/// # Example
///
/// ```
/// use clarity_web::intent::security::{validate_session_id, SecurityError};
///
/// // Valid session ID
/// assert!(validate_session_id("session-123_ABC").is_ok());
///
/// // Invalid character
/// let result = validate_session_id("session@123");
/// assert!(result.is_err());
///
/// // Too long
/// let long_id = "a".repeat(500);
/// let result = validate_session_id(&long_id);
/// assert!(result.is_err());
/// ```
pub fn validate_session_id(session_id: &str) -> SecurityResult<String> {
    // Check for empty input
    if session_id.is_empty() {
        return Err(SecurityError::SessionIdValidation {
            error: SessionIdError::Empty,
        });
    }

    // Check length
    if session_id.len() > MAX_SESSION_ID_LENGTH {
        return Err(SecurityError::SessionIdValidation {
            error: SessionIdError::TooLong {
                max: MAX_SESSION_ID_LENGTH,
            },
        });
    }

    // Check for invalid characters
    for ch in session_id.chars() {
        if !is_valid_session_id_char(ch) {
            return Err(SecurityError::SessionIdValidation {
                error: SessionIdError::InvalidCharacter { ch },
            });
        }
    }

    Ok(session_id.to_string())
}

/// Validate multiple file paths at once
///
/// Uses iterator pipelines to validate all paths, returning the first error
/// encountered or a vector of validated paths.
///
/// # Arguments
///
/// * `paths` - Slice of paths to validate
///
/// # Returns
///
/// `Ok(Vec<String>)` with validated paths, or `Err(SecurityError)` on first failure
///
/// # Example
///
/// ```
/// use clarity_web::intent::security::validate_file_paths;
///
/// let paths = vec!["safe/path1.txt", "safe/path2.txt"];
/// let result = validate_file_paths(&paths);
/// assert!(result.is_ok());
/// assert_eq!(result.unwrap().len(), 2);
/// ```
pub fn validate_file_paths(paths: &[&str]) -> SecurityResult<Vec<String>> {
    paths
        .iter()
        .map(|&path| validate_file_path(path))
        .try_collect()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // is_safe_path tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_is_safe_path_valid() {
        assert!(is_safe_path("safe/path/file.txt"));
        assert!(is_safe_path("file.txt"));
        assert!(is_safe_path("path/to/file"));
        assert!(is_safe_path("a"));
    }

    #[test]
    fn test_is_safe_path_traversal() {
        assert!(!is_safe_path("../etc/passwd"));
        assert!(!is_safe_path("path/../../../etc"));
        assert!(!is_safe_path(".."));
        assert!(!is_safe_path("path/.."));
    }

    #[test]
    fn test_is_safe_path_encoded() {
        assert!(!is_safe_path("%2e%2e/etc/passwd"));
        assert!(!is_safe_path("%2E%2E/etc/passwd")); // uppercase
        assert!(!is_safe_path("%252e%252e/etc/passwd")); // double encoded
    }

    #[test]
    fn test_is_safe_path_backslash() {
        assert!(!is_safe_path("..\\windows\\system32"));
        assert!(!is_safe_path("path\\to\\file"));
    }

    #[test]
    fn test_is_safe_path_null_byte() {
        assert!(!is_safe_path("file.txt\0.exe"));
        assert!(!is_safe_path("\0"));
    }

    #[test]
    fn test_is_safe_path_empty() {
        assert!(!is_safe_path(""));
    }

    // -------------------------------------------------------------------------
    // validate_file_path tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_validate_file_path_valid() {
        let result = validate_file_path("safe/path/file.txt");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "safe/path/file.txt");
    }

    #[test]
    fn test_validate_file_path_empty() {
        let result = validate_file_path("");
        assert!(matches!(result, Err(SecurityError::EmptyInput)));
    }

    #[test]
    fn test_validate_file_path_null_byte() {
        let result = validate_file_path("file\0.txt");
        assert!(matches!(result, Err(SecurityError::NullByteDetected)));
    }

    #[test]
    fn test_validate_file_path_backslash() {
        let result = validate_file_path("path\\to\\file");
        assert!(matches!(result, Err(SecurityError::BackslashInPath)));
    }

    #[test]
    fn test_validate_file_path_literal_traversal() {
        let result = validate_file_path("../../etc/passwd");
        assert!(matches!(
            result,
            Err(SecurityError::PathTraversal { .. })
        ));
    }

    #[test]
    fn test_validate_file_path_encoded_traversal() {
        let result = validate_file_path("%2e%2e/etc/passwd");
        assert!(matches!(
            result,
            Err(SecurityError::EncodedPathTraversal {
                encoding_type: PathEncodingType::SingleEncoded
            })
        ));
    }

    #[test]
    fn test_validate_file_path_double_encoded() {
        let result = validate_file_path("%252e%252e/etc/passwd");
        assert!(matches!(
            result,
            Err(SecurityError::EncodedPathTraversal {
                encoding_type: PathEncodingType::DoubleEncoded
            })
        ));
    }

    #[test]
    fn test_validate_file_path_shell_metachar_semicolon() {
        let result = validate_file_path("file;rm -rf /");
        assert!(matches!(
            result,
            Err(SecurityError::ShellMetacharacter {
                category: MetacharCategory::CommandSeparator,
                ..
            })
        ));
    }

    #[test]
    fn test_validate_file_path_shell_metachar_pipe() {
        let result = validate_file_path("file|cat /etc/passwd");
        assert!(matches!(
            result,
            Err(SecurityError::ShellMetacharacter {
                category: MetacharCategory::CommandSeparator,
                ..
            })
        ));
    }

    #[test]
    fn test_validate_file_path_shell_metachar_variable() {
        let result = validate_file_path("file$HOME");
        assert!(matches!(
            result,
            Err(SecurityError::ShellMetacharacter {
                category: MetacharCategory::VariableExpansion,
                ..
            })
        ));
    }

    #[test]
    fn test_validate_file_path_control_character() {
        let result = validate_file_path("file\x01.txt");
        assert!(matches!(
            result,
            Err(SecurityError::ShellMetacharacter {
                category: MetacharCategory::ControlCharacter,
                ..
            })
        ));
    }

    // -------------------------------------------------------------------------
    // validate_regex_pattern tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_validate_regex_pattern_valid() {
        assert!(validate_regex_pattern("^\\w+$").is_ok());
        assert!(validate_regex_pattern("[a-z]+").is_ok());
        assert!(validate_regex_pattern("test.*pattern").is_ok());
    }

    #[test]
    fn test_validate_regex_pattern_empty() {
        let result = validate_regex_pattern("");
        assert!(matches!(result, Err(SecurityError::EmptyInput)));
    }

    #[test]
    fn test_validate_regex_pattern_null_byte() {
        let result = validate_regex_pattern("pattern\0");
        assert!(matches!(result, Err(SecurityError::NullByteDetected)));
    }

    #[test]
    fn test_validate_regex_pattern_exponential_plus() {
        // (.+)+ is a classic exponential backtracking pattern
        let result = validate_regex_pattern("(.+)+");
        assert!(matches!(
            result,
            Err(SecurityError::ReDoSVulnerability {
                vulnerability: RegexVulnerability::ExponentialBacktracking
            })
        ));
    }

    #[test]
    fn test_validate_regex_pattern_exponential_star() {
        // (.*)* is a classic exponential backtracking pattern
        let result = validate_regex_pattern("(.*)*");
        assert!(matches!(
            result,
            Err(SecurityError::ReDoSVulnerability {
                vulnerability: RegexVulnerability::ExponentialBacktracking
            })
        ));
    }

    #[test]
    fn test_validate_regex_pattern_nested_quantifiers_general() {
        // General nested quantifier pattern - uses *)* detection
        // Pattern like (x*)* triggers nested quantifier via *)* substring
        let result = validate_regex_pattern("(x*)*");
        // This contains *)* which is detected as nested quantifier
        // Note: Since this is NOT in the exp_patterns list, it falls through to nested detection
        assert!(result.is_err());
        // Should be either NestedQuantifiers or ExponentialBacktracking depending on pattern
        match result {
            Err(SecurityError::ReDoSVulnerability { vulnerability }) => {
                assert!(matches!(
                    vulnerability,
                    RegexVulnerability::NestedQuantifiers | RegexVulnerability::ExponentialBacktracking
                ));
            }
            _ => panic!("Expected ReDoSVulnerability error"),
        }
    }

    #[test]
    fn test_validate_regex_pattern_nested_star_general() {
        // General nested star pattern
        let result = validate_regex_pattern("(a*)*");
        assert!(matches!(
            result,
            Err(SecurityError::ReDoSVulnerability {
                vulnerability: RegexVulnerability::NestedQuantifiers
            })
        ));
    }

    #[test]
    fn test_validate_regex_pattern_overlapping_wildcards() {
        let result = validate_regex_pattern(".*.*");
        assert!(matches!(
            result,
            Err(SecurityError::ReDoSVulnerability {
                vulnerability: RegexVulnerability::OverlappingWildcards
            })
        ));
    }

    // -------------------------------------------------------------------------
    // validate_session_id tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_validate_session_id_valid() {
        assert!(validate_session_id("session123").is_ok());
        assert!(validate_session_id("session-123").is_ok());
        assert!(validate_session_id("session_123").is_ok());
        assert!(validate_session_id("SESSION-123_ABC").is_ok());
    }

    #[test]
    fn test_validate_session_id_empty() {
        let result = validate_session_id("");
        assert!(matches!(
            result,
            Err(SecurityError::SessionIdValidation {
                error: SessionIdError::Empty
            })
        ));
    }

    #[test]
    fn test_validate_session_id_too_long() {
        let long_id = "a".repeat(500);
        let result = validate_session_id(&long_id);
        assert!(matches!(
            result,
            Err(SecurityError::SessionIdValidation {
                error: SessionIdError::TooLong { max: 499 }
            })
        ));
    }

    #[test]
    fn test_validate_session_id_max_length() {
        let max_id = "a".repeat(499);
        let result = validate_session_id(&max_id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_session_id_invalid_char_space() {
        let result = validate_session_id("session 123");
        assert!(matches!(
            result,
            Err(SecurityError::SessionIdValidation {
                error: SessionIdError::InvalidCharacter { ch: ' ' }
            })
        ));
    }

    #[test]
    fn test_validate_session_id_invalid_char_at() {
        let result = validate_session_id("session@123");
        assert!(matches!(
            result,
            Err(SecurityError::SessionIdValidation {
                error: SessionIdError::InvalidCharacter { ch: '@' }
            })
        ));
    }

    #[test]
    fn test_validate_session_id_invalid_char_dot() {
        let result = validate_session_id("session.123");
        assert!(matches!(
            result,
            Err(SecurityError::SessionIdValidation {
                error: SessionIdError::InvalidCharacter { ch: '.' }
            })
        ));
    }

    // -------------------------------------------------------------------------
    // validate_file_paths tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_validate_file_paths_all_valid() {
        let paths = vec!["path1.txt", "path2.txt", "path3.txt"];
        let result = validate_file_paths(&paths);
        assert!(result.is_ok());
        let validated = result.unwrap();
        assert_eq!(validated.len(), 3);
    }

    #[test]
    fn test_validate_file_paths_one_invalid() {
        let paths = vec!["path1.txt", "../etc/passwd", "path3.txt"];
        let result = validate_file_paths(&paths);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_file_paths_empty() {
        let paths: Vec<&str> = vec![];
        let result = validate_file_paths(&paths);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    // -------------------------------------------------------------------------
    // Helper function tests
    // -------------------------------------------------------------------------

    #[test]
    fn test_classify_metachar() {
        assert_eq!(
            classify_metachar(';'),
            Some(MetacharCategory::CommandSeparator)
        );
        assert_eq!(
            classify_metachar('$'),
            Some(MetacharCategory::VariableExpansion)
        );
        assert_eq!(classify_metachar('('), Some(MetacharCategory::Grouping));
        assert_eq!(classify_metachar('<'), Some(MetacharCategory::Redirection));
        assert_eq!(classify_metachar('*'), Some(MetacharCategory::EscapeQuote));
    }

    #[test]
    fn test_is_valid_session_id_char() {
        // Valid characters
        assert!(is_valid_session_id_char('a'));
        assert!(is_valid_session_id_char('Z'));
        assert!(is_valid_session_id_char('0'));
        assert!(is_valid_session_id_char('9'));
        assert!(is_valid_session_id_char('-'));
        assert!(is_valid_session_id_char('_'));

        // Invalid characters
        assert!(!is_valid_session_id_char(' '));
        assert!(!is_valid_session_id_char('.'));
        assert!(!is_valid_session_id_char('@'));
        assert!(!is_valid_session_id_char('/'));
    }

    #[test]
    fn test_contains_encoded_traversal_single() {
        assert_eq!(
            contains_encoded_traversal("%2e%2e"),
            Some(PathEncodingType::SingleEncoded)
        );
        assert_eq!(
            contains_encoded_traversal("%2E%2E"), // uppercase
            Some(PathEncodingType::SingleEncoded)
        );
        assert_eq!(
            contains_encoded_traversal("%5c"),
            Some(PathEncodingType::SingleEncoded)
        );
    }

    #[test]
    fn test_contains_encoded_traversal_double() {
        assert_eq!(
            contains_encoded_traversal("%252e"),
            Some(PathEncodingType::DoubleEncoded)
        );
        assert_eq!(
            contains_encoded_traversal("%255c"),
            Some(PathEncodingType::DoubleEncoded)
        );
    }

    #[test]
    fn test_contains_encoded_traversal_none() {
        assert_eq!(contains_encoded_traversal("normal/path"), None);
        assert_eq!(contains_encoded_traversal("file.txt"), None);
    }
}
