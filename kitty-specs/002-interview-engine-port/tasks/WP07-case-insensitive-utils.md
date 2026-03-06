---
work_package_id: WP07
title: Case-Insensitive Utilities
lane: planned
dependencies: []
subtasks: [T032, T033, T034, T035]
---

# WP07: Case-Insensitive Utilities

## Objective

Port `case_insensitive.gleam` (85 lines) for case-insensitive string matching utilities.

## Context

- **Source**: `/tmp/intent-cli/src/intent/case_insensitive.gleam` (85 lines)
- **Target**: `clarity-web/src/intent/util/case_insensitive.rs`
- **Priority**: P1 (High)

## Contract Specification

### Preconditions

| ID | Precondition | Enforcement |
|----|--------------|-------------|
| P1 | Input strings are valid UTF-8 | Compile-time |

### Postconditions

| ID | Postcondition | Verification |
|----|---------------|--------------|
| Q1 | Result is deterministic | Same input → same output |
| Q2 | Unicode is handled correctly | to_lowercase() used |

### Invariants

| ID | Invariant |
|----|-----------|
| I1 | Empty needle matches nothing |
| I2 | Empty haystack returns false |

---

## Subtasks

### T032: Create `clarity-web/src/intent/util/case_insensitive.rs`

### T033: Implement `contains_any_ignore_case(haystack: &str, needles: &[&str]) -> bool`

```rust
/// Check if haystack contains any of the needles (case-insensitive)
///
/// # Preconditions
/// - All inputs are valid UTF-8
///
/// # Invariants
/// - Empty needle list returns false
/// - Empty haystack returns false
///
/// # Examples
/// ```
/// assert!(contains_any_ignore_case("Hello World", &["HELLO", "foo"]));
/// assert!(!contains_any_ignore_case("Hello World", &["foo", "bar"]));
/// assert!(!contains_any_ignore_case("", &["test"]));
/// ```
pub fn contains_any_ignore_case(haystack: &str, needles: &[&str]) -> bool {
    if haystack.is_empty() || needles.is_empty() {
        return false;
    }
    let haystack_lower = haystack.to_lowercase();
    needles.iter().any(|needle| {
        if needle.is_empty() {
            false
        } else {
            haystack_lower.contains(&needle.to_lowercase())
        }
    })
}
```

### T034: Implement `equals_ignore_case(a: &str, b: &str) -> bool`

```rust
/// Check if two strings are equal ignoring case
///
/// # Examples
/// ```
/// assert!(equals_ignore_case("Hello", "HELLO"));
/// assert!(equals_ignore_case("Test", "test"));
/// assert!(!equals_ignore_case("Test", "Test2"));
/// ```
pub fn equals_ignore_case(a: &str, b: &str) -> bool {
    a.to_lowercase() == b.to_lowercase()
}
```

### T035: Add unit tests for case-insensitive operations

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_any_basic() {
        assert!(contains_any_ignore_case("Hello World", &["hello"]));
        assert!(contains_any_ignore_case("Hello World", &["WORLD"]));
        assert!(!contains_any_ignore_case("Hello World", &["foo"]));
    }

    #[test]
    fn test_contains_any_empty() {
        assert!(!contains_any_ignore_case("", &["test"]));
        assert!(!contains_any_ignore_case("test", &[]));
        assert!(!contains_any_ignore_case("test", &[""]));
    }

    #[test]
    fn test_equals_ignore_case() {
        assert!(equals_ignore_case("TEST", "test"));
        assert!(equals_ignore_case("Test", "TeSt"));
        assert!(!equals_ignore_case("Test1", "Test2"));
    }
}
```

---

## Definition of Done

- [ ] `contains_any_ignore_case()` implemented
- [ ] `equals_ignore_case()` implemented
- [ ] Unit tests pass
- [ ] No panics on any input
