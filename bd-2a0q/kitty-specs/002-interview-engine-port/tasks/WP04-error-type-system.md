---
work_package_id: WP04
title: Error Type System
lane: planned
dependencies: []
subtasks: [T015, T016, T017, T018, T019]
---

# WP04: Error Type System

## Objective

Port `errors.gleam` (234 lines) with contextual error reporting, Levenshtein distance suggestions, and user-friendly formatting.

## Context

- **Source**: `/tmp/intent-cli/src/intent/errors.gleam` (234 lines)
- **Target**: `clarity-web/src/intent/errors.rs`
- **Priority**: P0 (Critical)

## Contract Specification

### Preconditions

| ID | Precondition | Enforcement Level |
|----|--------------|-------------------|
| P1 | thiserror dependency available | Compile-time |
| P2 | Error message is non-empty | Runtime |
| P3 | Levenshtein inputs are valid UTF-8 | Runtime |

### Postconditions

| ID | Postcondition | Verification |
|----|---------------|--------------|
| Q1 | All errors implement std::error::Error | Compile-time |
| Q2 | All errors implement Display | Compile-time |
| Q3 | Levenshtein distance is symmetric | Property test |
| Q4 | Suggestions are sorted by distance | Runtime check |

### Invariants

| ID | Invariant |
|----|-----------|
| I1 | Levenshtein distance >= 0 |
| I2 | Levenshtein distance of identical strings = 0 |
| I3 | Suggestion list is never null (empty ok) |

### Error Taxonomy

```rust
#[derive(Debug, Clone, thiserror::Error)]
pub enum IntentError {
    #[error("JSON serialization failed: {reason}")]
    JsonSerialize { reason: String },

    #[error("JSON deserialization failed: {reason}")]
    JsonDeserialize { reason: String, source: String },

    #[error("I/O error: {context}")]
    Io { context: String, #[source] source: std::io::Error },

    #[error("Validation failed: {reason}")]
    Validation { reason: String },

    #[error("Not found: {resource}")]
    NotFound { resource: String },

    #[error("Conflict detected: {description}")]
    Conflict { description: String },

    #[error("Parse error at {location}: {reason}")]
    Parse { location: String, reason: String },
}

#[derive(Debug, Clone)]
pub struct ContextualError {
    pub error: IntentError,
    pub suggestions: Vec<Suggestion>,
}

#[derive(Debug, Clone)]
pub struct Suggestion {
    pub text: String,
    pub distance: usize,
}

#[derive(Debug, Clone)]
pub struct FieldFailure {
    pub field: String,
    pub expected: String,
    pub actual: String,
}
```

### Violation Examples

```
VIOLATES P2: Empty error message
  -> panic!("Error message must be non-empty")

VIOLATES I1: Negative Levenshtein distance (impossible in correct impl)
  -> unreachable!()
```

---

## Subtasks

### T015: Create IntentError enum

### T016: Add ContextualError wrapper with suggestions

### T017: Implement Levenshtein distance

```rust
/// Compute Levenshtein distance between two strings
///
/// # Preconditions
/// - Both strings are valid UTF-8 (always true in Rust)
///
/// # Invariants
/// - Result >= 0
/// - distance(a, a) == 0
/// - distance(a, b) == distance(b, a)
///
/// # Examples
/// ```
/// assert_eq!(levenshtein("kitten", "sitting"), 3);
/// assert_eq!(levenshtein("", "abc"), 3);
/// assert_eq!(levenshtein("abc", "abc"), 0);
/// ```
pub fn levenshtein(a: &str, b: &str) -> usize {
    // Standard Wagner-Fischer algorithm
}
```

### T018: Add ValidationError and FieldFailure types

### T019: Implement user-friendly error formatting

```rust
impl ContextualError {
    /// Format error with suggestions for CLI output
    pub fn format_for_cli(&self) -> String {
        let mut output = format!("Error: {}\n", self.error);

        if !self.suggestions.is_empty() {
            output.push_str("\nDid you mean:\n");
            for suggestion in &self.suggestions {
                output.push_str(&format!("  - {} (distance: {})\n",
                    suggestion.text, suggestion.distance));
            }
        }
        output
    }
}
```

---

## Test Strategy

### Property-Based Tests

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_levenshtein_symmetric(a: String, b: String) {
        assert_eq!(levenshtein(&a, &b), levenshtein(&b, &a));
    }

    #[test]
    fn test_levenshtein_self_zero(s: String) {
        assert_eq!(levenshtein(&s, &s), 0);
    }

    #[test]
    fn test_levenshtein_non_negative(a: String, b: String) {
        assert!(levenshtein(&a, &b) >= 0);
    }
}
```

### Contract Verification Tests

```rust
#[test]
fn test_q1_all_errors_impl_std_error() {
    fn assert_error<T: std::error::Error>() {}
    assert_error::<IntentError>();
}

#[test]
fn test_q4_suggestions_sorted_by_distance() {
    let suggestions = suggest_similar("foo", &["bar", "food", "baz", "fool"]);
    for window in suggestions.windows(2) {
        assert!(window[0].distance <= window[1].distance);
    }
}
```

---

## Definition of Done

- [ ] IntentError enum with all variants
- [ ] ContextualError with suggestions
- [ ] Levenshtein distance function
- [ ] User-friendly CLI formatting
- [ ] Property tests pass
